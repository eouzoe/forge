//! Error types for the gateway crate.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

/// Errors that can occur during gateway request handling.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GatewayError {
    /// An error propagated from the executor layer.
    #[error("executor error: {0}")]
    Executor(#[from] forge_executor::ExecutorError),

    /// The requested sandbox ID does not exist in the pool.
    #[error("sandbox not found: {0}")]
    SandboxNotFound(Uuid),

    /// The request body is malformed or contains invalid values.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let status = match &self {
            GatewayError::Executor(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::SandboxNotFound(_) => StatusCode::NOT_FOUND,
            GatewayError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        };
        (status, Json(json!({"error": self.to_string()}))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn gateway_error_status_codes_map_correctly() {
        let not_found = GatewayError::SandboxNotFound(Uuid::nil());
        let resp = not_found.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let bad_req = GatewayError::InvalidRequest("missing field".to_owned());
        let resp = bad_req.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
