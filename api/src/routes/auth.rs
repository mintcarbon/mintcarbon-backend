use axum::{extract::State, http::StatusCode, Json};
use base32;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::db::models::{AuthPayload, User};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub mfa_token: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Serialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_url: String,
}

#[derive(Deserialize)]
pub struct MfaVerifyRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct MfaVerifyResponse {
    pub valid: bool,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, &'static str)> {
    let role = req.role.as_deref().unwrap_or("trader");
    let valid_roles = ["issuer", "trader", "compliance_officer", "administrator"];
    if !valid_roles.contains(&role) {
        return Err((StatusCode::BAD_REQUEST, "Invalid role"));
    }

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    if existing > 0 {
        return Err((StatusCode::CONFLICT, "Email already registered"));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"))?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, role, kyc_status) VALUES ($1, $2, $3, 'pending') RETURNING *",
    )
    .bind(&req.email)
    .bind(&password_hash)
    .bind(role)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user"))?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            id: user.id,
            email: user.email,
            role: user.role,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, &'static str)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid email or password"))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid hash"))?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid email or password"))?;

    if let Some(ref mfa_secret) = user.mfa_secret {
        let mfa_token = req
            .mfa_token
            .ok_or((StatusCode::BAD_REQUEST, "MFA token required"))?;
        let secret_bytes = Secret::Encoded(mfa_secret.clone())
            .to_bytes()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid MFA secret"))?;
        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes, None, user.email.clone())
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "TOTP error"))?;
        let is_valid = totp.check_current(&mfa_token).unwrap_or(false);
        if !is_valid {
            return Err((StatusCode::UNAUTHORIZED, "Invalid MFA token"));
        }
    }

    let exp = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = AuthPayload {
        sub: user.id,
        role: user.role.clone(),
        exp,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".into());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token"))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.id,
        role: user.role,
    }))
}

pub async fn mfa_setup(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<MfaSetupResponse>, (StatusCode, &'static str)> {
    let raw_secret: [u8; 20] = rand::thread_rng().gen();
    let encoded = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &raw_secret);

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        raw_secret.to_vec(),
        Some("mintcarbon".into()),
        auth.user_id.to_string(),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "TOTP generation error"))?;

    let qr_url = totp.get_url();

    sqlx::query("UPDATE users SET mfa_secret = $1 WHERE id = $2")
        .bind(&encoded)
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to store MFA secret"))?;

    Ok(Json(MfaSetupResponse {
        secret: encoded,
        qr_url,
    }))
}

pub async fn mfa_verify(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, (StatusCode, &'static str)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))?;

    let secret_str = user
        .mfa_secret
        .ok_or((StatusCode::BAD_REQUEST, "MFA not set up"))?;
    let secret = Secret::Encoded(secret_str)
        .to_bytes()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid MFA secret"))?;

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, None, user.email)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "TOTP error"))?;

    let valid = totp.check_current(&req.code).unwrap_or(false);
    Ok(Json(MfaVerifyResponse { valid }))
}
