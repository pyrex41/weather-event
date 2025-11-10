use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Simple authentication middleware
/// In production, replace with JWT validation or proper session management
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get authorization header
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    // Check for valid token
    if let Some(token) = auth_header {
        // Validate API key from environment
        if validate_api_key(token) {
            return Ok(next.run(request).await);
        }
    }

    tracing::warn!("Unauthorized access attempt");
    Err(StatusCode::UNAUTHORIZED)
}

/// Validate API key from environment
fn validate_api_key(token: &str) -> bool {
    if let Some(bearer_token) = token.strip_prefix("Bearer ") {
        // Check against configured API key
        if let Ok(api_key) = std::env::var("API_KEY") {
            return bearer_token == api_key;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key() {
        std::env::set_var("API_KEY", "test-secret-key");
        assert!(validate_api_key("Bearer test-secret-key"));
        assert!(!validate_api_key("Bearer wrong-key"));
        assert!(!validate_api_key("test-secret-key")); // Missing Bearer prefix
    }
}
