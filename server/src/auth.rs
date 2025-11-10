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
    tracing::debug!("Auth middleware: checking request to {}", request.uri());

    // Get authorization header
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    tracing::debug!("Auth header present: {}", auth_header.is_some());

    // Check for valid token
    if let Some(token) = auth_header {
        // Validate API key from environment
        if validate_api_key(token) {
            tracing::debug!("Auth successful");
            return Ok(next.run(request).await);
        } else {
            tracing::debug!("Invalid API key provided");
        }
    } else {
        tracing::debug!("No authorization header provided");
    }

    tracing::warn!("Unauthorized access attempt to {}", request.uri());
    Err(StatusCode::UNAUTHORIZED)
}

/// Validate API key from environment
fn validate_api_key(token: &str) -> bool {
    tracing::debug!("Validating token: {}", token);

    if let Some(bearer_token) = token.strip_prefix("Bearer ") {
        tracing::debug!("Bearer token extracted: {}", bearer_token);

        // Check against configured API key
        if let Ok(api_key) = std::env::var("API_KEY") {
            tracing::debug!("API_KEY from env: {}", api_key);
            let valid = bearer_token == api_key;
            tracing::debug!("Token validation result: {}", valid);
            return valid;
        } else {
            tracing::debug!("API_KEY environment variable not set");
        }
    } else {
        tracing::debug!("Token doesn't start with 'Bearer '");
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
