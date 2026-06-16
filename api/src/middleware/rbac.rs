use axum::http::StatusCode;

pub fn check_role(actual: &str, allowed: &[&str]) -> Result<(), (StatusCode, &'static str)> {
    if allowed.contains(&actual) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Insufficient permissions"))
    }
}
