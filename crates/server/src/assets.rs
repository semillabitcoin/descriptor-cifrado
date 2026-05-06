//! Embed del SPA compilado por Vite. Usa rust-embed para incluir
//! `frontend/dist/` en el binario en compile-time.
//!
//! Routes:
//! - GET /            → index.html
//! - GET /assets/{*path} → cualquier asset hashado (JS, CSS, woff2)
//!
//! UI-01: cero requests externos al cargar la SPA. Los assets se sirven desde
//! el binario directamente.

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../frontend/dist/"]
pub struct FrontendAssets;

/// Wrapper response que sirve un archivo embebido con su Content-Type correcto.
pub struct StaticFile<T>(pub T);

impl<T: AsRef<str>> IntoResponse for StaticFile<T> {
    fn into_response(self) -> Response {
        let path = self.0.as_ref();
        match FrontendAssets::get(path) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    // Cache largo para assets hashados; index.html sin cache largo.
                    .header(header::CACHE_CONTROL, cache_control_for(path))
                    .body(Body::from(content.data.into_owned()))
                    .unwrap_or_else(|_| {
                        (StatusCode::INTERNAL_SERVER_ERROR, "asset error").into_response()
                    })
            }
            None => (StatusCode::NOT_FOUND, "404").into_response(),
        }
    }
}

fn cache_control_for(path: &str) -> &'static str {
    // Hashed assets: cache largo. index.html: sin cache para que el browser pille nuevo build.
    if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}

pub async fn index_handler() -> impl IntoResponse {
    StaticFile("index.html")
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    // uri.path() llega como "/assets/index-abc123.js"; rust-embed espera "assets/index-abc123.js".
    let path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}
