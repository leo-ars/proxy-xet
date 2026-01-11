//! XET Proxy Server - Rust HTTP wrapper for zig-xet CLI
//!
//! This server provides a production-ready HTTP interface to the XET protocol
//! implementation in Zig. It handles HTTP routing, streaming, and error handling
//! while delegating the actual XET protocol work to the Zig CLI.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_util::io::ReaderStream;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

const VERSION: &str = "0.1.0";

#[derive(Clone)]
struct AppState {
    hf_token: String,
    zig_bin_path: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get configuration from environment
    let hf_token = std::env::var("HF_TOKEN").expect("HF_TOKEN environment variable must be set");
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    let zig_bin_path = std::env::var("ZIG_BIN_PATH")
        .unwrap_or_else(|_| "/usr/local/bin/xet-download".to_string());

    let state = Arc::new(AppState {
        hf_token,
        zig_bin_path,
    });

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/download/:owner/:repo/*file", get(download_by_path))
        .route("/download-hash/:hash", get(download_by_hash))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("========================================");
    info!("XET Proxy Server v{}", VERSION);
    info!("========================================");
    info!("Listening on: http://{}", addr);
    info!("");
    info!("Endpoints:");
    info!("  GET /health");
    info!("  GET /download/:owner/:repo/*file");
    info!("  GET /download-hash/:hash");
    info!("");
    info!("Press Ctrl+C to stop");
    info!("========================================");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

/// Root endpoint - returns usage instructions
async fn root() -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>XET Proxy Server</title>
    <style>
        body {{ font-family: system-ui; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        pre {{ background: #f4f4f4; padding: 15px; border-radius: 5px; overflow-x: auto; }}
        code {{ background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }}
        .endpoint {{ margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>XET Protocol HTTP Proxy Server</h1>
    <p>Version: {}</p>
    
    <h2>Endpoints</h2>
    
    <div class="endpoint">
        <h3>Health Check</h3>
        <code>GET /health</code>
        <p>Returns server health status</p>
    </div>
    
    <div class="endpoint">
        <h3>Download by Repository and Path</h3>
        <code>GET /download/:owner/:repo/*file</code>
        <p>Download a file from HuggingFace by repository and file path</p>
        <pre>curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf -o model.gguf</pre>
    </div>
    
    <div class="endpoint">
        <h3>Download by XET Hash</h3>
        <code>GET /download-hash/:hash</code>
        <p>Download a file directly by its XET hash (64 hex characters)</p>
        <pre>curl http://localhost:8080/download-hash/ef62b750... -o model.safetensors</pre>
    </div>
    
    <h2>Examples</h2>
    <pre>
# Download MiMo-7B model
curl http://localhost:8080/download/jedisct1/MiMo-7B-RL-GGUF/MiMo-7B-RL-Q8_0.gguf \\
  -o model.gguf

# Download by hash
curl http://localhost:8080/download-hash/89dbfa4888600b29be17ddee8bdbf9c48999c81cb811964eee6b057d8467f927 \\
  -o model.safetensors

# Check health
curl http://localhost:8080/health
    </pre>
</body>
</html>"#,
        VERSION
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

/// Health check endpoint
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: VERSION,
    })
}

/// Download file by repository path
async fn download_by_path(
    State(state): State<Arc<AppState>>,
    Path((owner, repo, file)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    let repo_id = format!("{}/{}", owner, repo);
    info!("Download request: repo={}, file={}", repo_id, file);

    // First, list files to get the XET hash
    let output = Command::new(&state.zig_bin_path)
        .arg(&repo_id)
        .env("HF_TOKEN", &state.hf_token)
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to execute zig binary: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Zig CLI failed: {}", stderr);
        return Err(AppError::Internal(format!("Failed to list files: {}", stderr)));
    }

    // Parse output to find the file and get its XET hash
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Look for the file in the output
    // Expected format: "filename - size bytes - xetHash: abc123..."
    let mut xet_hash = None;
    for line in stdout.lines() {
        if line.contains(&file) && line.contains("xetHash:") {
            if let Some(hash_part) = line.split("xetHash:").nth(1) {
                xet_hash = Some(hash_part.trim().to_string());
                break;
            }
        }
    }

    let hash = xet_hash.ok_or_else(|| {
        AppError::NotFound(format!("File '{}' not found or not XET-enabled", file))
    })?;

    info!("Found XET hash for {}: {}", file, hash);

    // Now download by hash
    download_by_hash_impl(state, hash).await
}

/// Download file by XET hash
async fn download_by_hash(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Result<Response, AppError> {
    info!("Download by hash: {}", hash);

    // Validate hash format (64 hex characters)
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::BadRequest(
            "Invalid XET hash format (expected 64 hex characters)".to_string(),
        ));
    }

    download_by_hash_impl(state, hash).await
}

/// Internal implementation of hash-based download
async fn download_by_hash_impl(
    state: Arc<AppState>,
    hash: String,
) -> Result<Response, AppError> {
    // Spawn the Zig CLI process to download the file
    // We'll use a temporary repo for the token, but download by hash directly
    let mut child = Command::new(&state.zig_bin_path)
        .arg("jedisct1/MiMo-7B-RL-GGUF") // Temporary repo for token
        .arg(&hash) // Pass hash as second argument
        .env("HF_TOKEN", &state.hf_token)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn zig process: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Internal("Failed to capture stdout".to_string()))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Internal("Failed to capture stderr".to_string()))?;

    // Log stderr in the background
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            info!("zig stderr: {}", line);
        }
    });

    // Create streaming response from stdout
    let stream = ReaderStream::new(stdout);
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.bin\"", &hash[..8]),
        )
        .body(body)
        .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Application error types
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse { error: message });

        (status, body).into_response()
    }
}
