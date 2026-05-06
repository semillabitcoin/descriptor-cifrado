//! bed-server — axum HTTP layer for bed-core.

use axum::{
    routing::{delete, get, post},
    Router,
};

pub mod error;
pub mod routes;
pub mod state;

pub use error::AppError;

/// Build the router. Used by main.rs (binds socket) and integration tests
/// (oneshot via tower::ServiceExt — D-23).
///
/// Phase 2: añade rutas de historial. NOTA: las rutas de assets (rust-embed)
/// se añaden en el plan 02-06 al integrar el frontend embebido.
pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        .route("/api/history", post(routes::history::post_history))
        .route("/api/history", get(routes::history::get_history))
        .route("/api/history/{id}", get(routes::history::get_history_id))
        .route("/api/history/{id}", delete(routes::history::delete_history))
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
    // No TraceLayer in Phase 1: there are no non-sensitive routes (D-19).
    // Phase 2 will add it on history endpoints.
}
