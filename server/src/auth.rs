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

    if let Some(token) = auth_header {
        tracing::debug!("Auth header value: '{}'", token);
        // Validate API key from environment
        if validate_api_key(token) {
            tracing::debug!("Auth successful");
            return Ok(next.run(request).await);
        } else {
            tracing::debug!("Invalid API key provided: '{}'", token);
        }
    } else {
        tracing::debug!("No authorization header provided");
    }

    tracing::warn!("Unauthorized access attempt to {}", request.uri());
    Err(StatusCode::UNAUTHORIZED)
}

/// Validate API key from environment
fn validate_api_key(token: &str) -> bool {
    tracing::debug!("Validating token: '{}'", token);

    if let Some(bearer_token) = token.strip_prefix("Bearer ") {
        tracing::debug!("Extracted bearer token: '{}'", bearer_token);

        // For development, accept the test key
        let expected_key = "test-secure-api-key-12345";
        tracing::debug!("Expected API key: '{}'", expected_key);
        let valid = bearer_token == expected_key;
        tracing::debug!("Token validation result: {}", valid);
        return valid;
    } else {
        tracing::debug!("Token does not start with 'Bearer '");
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
