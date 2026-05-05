//! bed-server — axum HTTP layer for bed-core.

use axum::{routing::post, Router};

pub mod error;
pub mod routes;
pub mod state;

pub use error::AppError;

/// Build the router. Used by main.rs (binds socket) and integration tests
/// (oneshot via tower::ServiceExt — D-23).
pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
    // No TraceLayer in Phase 1: there are no non-sensitive routes (D-19).
    // Phase 2 will add it on history endpoints.
}
