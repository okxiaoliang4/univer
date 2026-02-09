//! Request logging middleware for axum
//!
//! Records request latency, method, path, and status code at INFO level.
//! Useful for identifying performance bottlenecks in production.

use axum::{
    body::Body,
    extract::{Request, MatchedPath},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::info;

/// Middleware to log HTTP request details and latency
///
/// Logs:
/// - HTTP method and path
/// - Response status code
/// - Request duration in milliseconds
/// - Matched route pattern (e.g., "/api/documents/:doc_id/changeset")
///
/// Example log output:
/// ```text
/// INFO POST /api/documents/abc-123/changeset -> 200 OK (125.3ms) [route=/api/documents/:doc_id/changeset]
/// ```
pub async fn log_request(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Extract matched route pattern (e.g., "/api/documents/:doc_id/changeset")
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string());

    let response = next.run(req).await;
    let status = response.status();
    let elapsed = start.elapsed();

    info!(
        "{} {} -> {} ({:.1}ms) [route={}]",
        method,
        uri.path(),
        status.as_u16(),
        elapsed.as_secs_f64() * 1000.0,
        matched_path
    );

    response
}
