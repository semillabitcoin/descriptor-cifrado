//! bed-server — axum HTTP layer for bed-core.

use axum::{
    routing::{delete, get, post},
    Router,
};

pub mod assets;
pub mod error;
pub mod routes;
pub mod state;

pub use error::AppError;

/// Build the router. Used by main.rs (binds socket) and integration tests
/// (oneshot via tower::ServiceExt — D-23).
///
/// Phase 2 final: añade rutas estáticas (GET /, GET /assets/{*path}) servidas
/// vía rust-embed sobre `frontend/dist/`. Las rutas /api/* se registran primero
/// (especificidad axum 0.8); las rutas estáticas cierran el router.
pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        .route("/api/history", post(routes::history::post_history))
        .route("/api/history", get(routes::history::get_history))
        .route("/api/history/{id}", get(routes::history::get_history_id))
        .route("/api/history/{id}", delete(routes::history::delete_history))
        // Static SPA assets (rust-embed) — UI-01: cero requests externos.
        .route("/", get(assets::index_handler))
        .route("/assets/{*path}", get(assets::static_handler))
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
    // No TraceLayer in Phase 1: there are no non-sensitive routes (D-19).
    // Phase 2 will add it on history endpoints.
}
