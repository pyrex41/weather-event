use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum::http::header::{COOKIE, SET_COOKIE};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "x-csrf-token";

#[derive(Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
}

/// Generate a new CSRF token and return it with a Set-Cookie header
pub async fn generate_csrf_token() -> impl IntoResponse {
    let token = Uuid::new_v4().to_string();

    // Create secure cookie with SameSite=Strict
    // Note: HttpOnly is NOT set so JavaScript can read it for the header
    let cookie = format!(
        "{}={}; Path=/; SameSite=Strict; Secure",
        CSRF_COOKIE_NAME, token
    );

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    tracing::debug!("Generated new CSRF token");

    (
        StatusCode::OK,
        headers,
        Json(CsrfToken { token })
    )
}

/// CSRF validation middleware
/// Validates CSRF tokens for state-changing requests (POST, PATCH, PUT, DELETE)
/// Extracts token from cookie and X-CSRF-Token header and compares them
pub async fn csrf_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method();

    // Only check CSRF for state-changing methods
    if !matches!(method, &Method::POST | &Method::PATCH | &Method::PUT | &Method::DELETE) {
        return Ok(next.run(request).await);
    }

    // Extract token from cookie
    let cookie_token = extract_csrf_from_cookie(&headers);

    // Extract token from header
    let header_token = extract_csrf_from_header(&headers);

    // Both must be present and match
    match (cookie_token, header_token) {
        (Some(cookie), Some(header)) if cookie == header && !cookie.is_empty() => {
            tracing::debug!("CSRF token validated successfully");
            Ok(next.run(request).await)
        }
        (None, _) => {
            tracing::warn!("CSRF validation failed: missing cookie token");
            Err(StatusCode::FORBIDDEN)
        }
        (_, None) => {
            tracing::warn!("CSRF validation failed: missing header token");
            Err(StatusCode::FORBIDDEN)
        }
        _ => {
            tracing::warn!("CSRF validation failed: token mismatch");
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Extract CSRF token from Cookie header
fn extract_csrf_from_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find_map(|cookie| {
                    let cookie = cookie.trim();
                    if cookie.starts_with(CSRF_COOKIE_NAME) {
                        cookie.strip_prefix(&format!("{}=", CSRF_COOKIE_NAME))
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
        })
}

/// Extract CSRF token from X-CSRF-Token header
fn extract_csrf_from_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CSRF_HEADER_NAME)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_csrf_from_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str("csrf_token=test-token-123; other=value").unwrap(),
        );

        let token = extract_csrf_from_cookie(&headers);
        assert_eq!(token, Some("test-token-123".to_string()));
    }

    #[test]
    fn test_extract_csrf_from_cookie_not_found() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str("other=value").unwrap(),
        );

        let token = extract_csrf_from_cookie(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_csrf_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CSRF_HEADER_NAME,
            HeaderValue::from_str("test-token-123").unwrap(),
        );

        let token = extract_csrf_from_header(&headers);
        assert_eq!(token, Some("test-token-123".to_string()));
    }

    #[test]
    fn test_extract_csrf_from_header_not_found() {
        let headers = HeaderMap::new();
        let token = extract_csrf_from_header(&headers);
        assert_eq!(token, None);
    }
}
