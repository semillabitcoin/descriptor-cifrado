//! bed-server — axum HTTP layer for bed-core.
//!
//! Exposes `pub fn router() -> Router` so integration tests can use
//! `tower::ServiceExt::oneshot` without binding a socket (D-23).

use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(encrypt_stub))
        .route("/api/decrypt", post(decrypt_stub))
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
}

async fn encrypt_stub() -> &'static str { "encrypt stub" }
async fn decrypt_stub() -> &'static str { "decrypt stub" }
