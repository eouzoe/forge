//! Axum route handlers for the Forge gateway API.

use std::{sync::Arc, time::Instant};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{error::GatewayError, pool::SandboxPool};

// ── Shared state ─────────────────────────────────────────────────────────────

type Pool = Arc<SandboxPool>;

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateSandboxBody {
    pub runtime: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSandboxResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ShellBody {
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteBody {
    pub code: String,
    pub runtime: String,
}

/// Result returned by both `/shell` and `/execute` endpoints.
#[derive(Debug, Serialize)]
pub struct ShellResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u128,
}

// ── Router ────────────────────────────────────────────────────────────────────

/// Build the application router with the given sandbox pool.
pub fn create_router(pool: Pool) -> Router {
    Router::new()
        .route("/v1/sandbox", post(create_sandbox))
        .route("/v1/sandbox/{id}/shell", post(shell_command))
        .route("/v1/sandbox/{id}/execute", post(execute_code))
        .route("/v1/sandbox/{id}", delete(destroy_sandbox))
        .route("/health", get(health))
        .with_state(pool)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /health` — liveness probe.
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// `POST /v1/sandbox` — register a new sandbox and return its ID.
///
/// # Errors
/// Returns [`GatewayError::InvalidRequest`] if the runtime is not `"node"` or `"python"`.
pub async fn create_sandbox(
    State(pool): State<Pool>,
    Json(body): Json<CreateSandboxBody>,
) -> Result<impl IntoResponse, GatewayError> {
    if body.runtime != "node" && body.runtime != "python" {
        return Err(GatewayError::InvalidRequest(format!(
            "unsupported runtime '{}'; expected 'node' or 'python'",
            body.runtime
        )));
    }
    let id = pool.create(body.runtime);
    Ok((StatusCode::CREATED, Json(CreateSandboxResponse { id })))
}

/// `DELETE /v1/sandbox/:id` — destroy a sandbox.
///
/// # Errors
/// Returns [`GatewayError::SandboxNotFound`] if the ID is not registered.
pub async fn destroy_sandbox(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, GatewayError> {
    if !pool.remove(id) {
        return Err(GatewayError::SandboxNotFound(id));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /v1/sandbox/:id/shell` — run a shell command inside the sandbox.
///
/// # Errors
/// Returns [`GatewayError::SandboxNotFound`] if the ID is not registered, or
/// [`GatewayError::InvalidRequest`] if the shell process cannot be spawned.
///
/// # MVP note
/// Commands are executed locally via `sh -c`. Firecracker isolation is not
/// yet wired up; this is intentional for the MVP stage.
pub async fn shell_command(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(body): Json<ShellBody>,
) -> Result<impl IntoResponse, GatewayError> {
    if !pool.contains(id) {
        return Err(GatewayError::SandboxNotFound(id));
    }
    let result = run_shell(&body.command).await?;
    Ok(Json(result))
}

/// `POST /v1/sandbox/:id/execute` — run code in the sandbox runtime.
///
/// # Errors
/// Returns [`GatewayError::SandboxNotFound`] if the ID is not registered, or
/// [`GatewayError::InvalidRequest`] if the runtime is unsupported or the
/// process cannot be spawned.
///
/// # MVP note
/// Code is executed locally. Firecracker isolation is not yet wired up.
pub async fn execute_code(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(body): Json<ExecuteBody>,
) -> Result<impl IntoResponse, GatewayError> {
    if !pool.contains(id) {
        return Err(GatewayError::SandboxNotFound(id));
    }
    let result = run_code(&body.runtime, &body.code).await?;
    Ok(Json(result))
}

// ── Execution helpers ─────────────────────────────────────────────────────────

async fn run_shell(command: &str) -> Result<ShellResult, GatewayError> {
    let start = Instant::now();
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .await
        .map_err(|e| GatewayError::InvalidRequest(format!("failed to spawn shell: {e}")))?;

    Ok(ShellResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
        execution_time_ms: start.elapsed().as_millis(),
    })
}

async fn run_code(runtime: &str, code: &str) -> Result<ShellResult, GatewayError> {
    let (bin, flag) = match runtime {
        "node" => ("node", "-e"),
        "python" => ("python3", "-c"),
        other => {
            return Err(GatewayError::InvalidRequest(format!(
                "unsupported runtime '{other}'"
            )))
        }
    };

    let start = Instant::now();
    let output = tokio::process::Command::new(bin)
        .arg(flag)
        .arg(code)
        .output()
        .await
        .map_err(|e| GatewayError::InvalidRequest(format!("failed to spawn {bin}: {e}")))?;

    Ok(ShellResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
        execution_time_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_pool() -> Pool {
        Arc::new(SandboxPool::new())
    }

    #[tokio::test]
    async fn health_response_format_returns_ok_with_status_field() {
        let app = create_router(test_pool());
        let req = match Request::builder().uri("/health").body(Body::empty()) {
            Ok(r) => r,
            Err(e) => panic!("failed to build request: {e}"),
        };
        let resp = match app.oneshot(req).await {
            Ok(r) => r,
            Err(e) => panic!("handler error: {e}"),
        };
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = match axum::body::to_bytes(resp.into_body(), 1024).await {
            Ok(b) => b,
            Err(e) => panic!("failed to read body: {e}"),
        };
        let body: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => panic!("invalid JSON: {e}"),
        };
        assert_eq!(body["status"], "ok");
    }

    #[test]
    fn shell_result_serialization_includes_all_fields() {
        let result = ShellResult {
            success: true,
            stdout: "hello\n".to_owned(),
            stderr: String::new(),
            exit_code: 0,
            execution_time_ms: 42,
        };
        let json = match serde_json::to_string(&result) {
            Ok(s) => s,
            Err(e) => panic!("serialization failed: {e}"),
        };
        assert!(json.contains("\"success\":true"), "missing success field");
        assert!(json.contains("\"stdout\""), "missing stdout field");
        assert!(json.contains("\"exit_code\":0"), "missing exit_code field");
        assert!(json.contains("\"execution_time_ms\":42"), "missing execution_time_ms field");
    }
}
