use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::Project;
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub registry: String,
    pub cert_id: String,
    pub project_name: String,
    pub project_type: Option<String>,
    pub location: Option<String>,
    pub vintage_year: Option<i32>,
}

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub registry: String,
    pub cert_id: String,
    pub project_name: String,
    pub project_type: Option<String>,
    pub location: Option<String>,
    pub vintage_year: Option<i32>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct ProjectListQuery {
    pub registry: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            registry: p.registry,
            cert_id: p.cert_id,
            project_name: p.project_name,
            project_type: p.project_type,
            location: p.location,
            vintage_year: p.vintage_year,
            status: p.status,
            created_at: p.created_at,
        }
    }
}

pub async fn create_project(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, &'static str)> {
    if auth.role != "issuer" && auth.role != "administrator" {
        return Err((StatusCode::FORBIDDEN, "Only issuers can create projects"));
    }

    let valid_registries = ["verra", "gold_standard", "acr"];
    if !valid_registries.contains(&req.registry.to_lowercase().as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid registry"));
    }

    let record = registry::RegistryParser::parse(
        &serde_json::json!({
            "registry": req.registry,
            "cert_id": req.cert_id,
            "project_name": req.project_name,
            "project_type": req.project_type,
            "location": req.location,
            "vintage_year": req.vintage_year,
        })
        .to_string(),
    )
    .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid verification record"))?;

    let parsed = registry::RegistryParser::print(&record);
    let roundtrip = registry::RegistryParser::parse(&parsed).ok();
    if roundtrip.as_ref().map(|r| r.cert_id.as_str()) != Some(&req.cert_id) {
        return Err((StatusCode::BAD_REQUEST, "Registry validation failed"));
    }

    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (owner_id, registry, cert_id, project_name, project_type, location, vintage_year, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending') RETURNING *",
    )
    .bind(auth.user_id)
    .bind(&req.registry)
    .bind(&req.cert_id)
    .bind(&req.project_name)
    .bind(&req.project_type)
    .bind(&req.location)
    .bind(req.vintage_year)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create project"))?;

    sqlx::query(
        "INSERT INTO audit_events (event_type, actor_id, payload)
         VALUES ($1, $2, $3)",
    )
    .bind("project_created")
    .bind(auth.user_id)
    .bind(serde_json::json!({
        "project_id": project.id,
        "registry": project.registry,
        "cert_id": project.cert_id,
        "project_name": project.project_name,
    }))
    .execute(&state.db)
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(ProjectResponse::from(project))))
}

pub async fn list_projects(
    State(state): State<AppState>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<ProjectListResponse>, (StatusCode, &'static str)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let mut conditions = Vec::new();
    let mut bind_idx = 1;

    if query.registry.is_some() {
        conditions.push(format!("registry = ${}", bind_idx));
        bind_idx += 1;
    }
    if query.status.is_some() {
        conditions.push(format!("status = ${}", bind_idx));
        bind_idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_query = format!("SELECT COUNT(*) FROM projects {}", where_clause);
    let select_query = format!(
        "SELECT * FROM projects {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause,
        bind_idx,
        bind_idx + 1
    );

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
    if let Some(ref registry) = query.registry {
        count_q = count_q.bind(registry);
    }
    if let Some(ref status) = query.status {
        count_q = count_q.bind(status);
    }
    let total = count_q
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    let mut select_q = sqlx::query_as::<_, Project>(&select_query);
    if let Some(ref registry) = query.registry {
        select_q = select_q.bind(registry);
    }
    if let Some(ref status) = query.status {
        select_q = select_q.bind(status);
    }
    select_q = select_q.bind(per_page).bind(offset);

    let projects = select_q
        .fetch_all(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(Json(ProjectListResponse {
        total,
        page,
        per_page,
        projects: projects.into_iter().map(ProjectResponse::from).collect(),
    }))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, (StatusCode, &'static str)> {
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    Ok(Json(ProjectResponse::from(project)))
}
