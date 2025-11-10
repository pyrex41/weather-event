use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Standardized API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetails {
                code: code.into(),
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            error: ErrorDetails {
                code: code.into(),
                message: message.into(),
                details: Some(details.into()),
            },
        }
    }

    // Common error constructors
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new(
            "NOT_FOUND",
            format!("{} not found", resource.into()),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    pub fn database_error(details: impl Into<String>) -> Self {
        Self::with_details(
            "DATABASE_ERROR",
            "A database error occurred",
            details,
        )
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn external_api_error(service: impl Into<String>, details: impl Into<String>) -> Self {
        Self::with_details(
            "EXTERNAL_API_ERROR",
            format!("{} API error", service.into()),
            details,
        )
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message)
    }
}

/// Convert ApiError to HTTP response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.error.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
            "CONFLICT" => StatusCode::CONFLICT,
            "EXTERNAL_API_ERROR" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

/// Helper type for Result with ApiError
pub type ApiResult<T> = Result<T, ApiError>;

/// Convert common errors to ApiError
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::not_found("Resource"),
            sqlx::Error::Database(db_err) => {
                tracing::error!("Database error: {:?}", db_err);
                ApiError::database_error(db_err.message())
            }
            _ => {
                tracing::error!("Database error: {:?}", err);
                ApiError::internal_error("Database operation failed")
            }
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::bad_request(format!("Invalid JSON: {}", err))
    }
}

impl From<chrono::ParseError> for ApiError {
    fn from(err: chrono::ParseError) -> Self {
        ApiError::validation_error(format!("Invalid date format: {}", err))
    }
}

impl From<uuid::Error> for ApiError {
    fn from(err: uuid::Error) -> Self {
        ApiError::validation_error(format!("Invalid UUID: {}", err))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        // Log detailed error for server-side debugging
        tracing::error!("Internal error occurred: {:?}", err);
        // Return generic message to client to avoid information disclosure
        ApiError::internal_error("An unexpected internal error occurred")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let error = ApiError::not_found("Booking");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("NOT_FOUND"));
        assert!(json.contains("Booking not found"));
    }

    #[test]
    fn test_error_with_details() {
        let error = ApiError::database_error("Connection timeout");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("DATABASE_ERROR"));
        assert!(json.contains("Connection timeout"));
    }
}
