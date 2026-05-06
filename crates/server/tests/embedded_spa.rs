//! Verifica que rust-embed sirve la SPA correctamente.
//! Test fundamental para UI-01 (SPA servida desde el binario sin requests externos).

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use bed_server::router;
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn get_root_returns_spa_html() {
    let app = router();

    let resp = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .expect("oneshot");

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .expect("content-type")
        .to_str()
        .unwrap();
    assert!(ct.contains("text/html"), "expected text/html, got {ct}");

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("body");
    let body = std::str::from_utf8(&bytes).expect("utf8");

    // Marcas mínimas de la SPA Svelte
    assert!(body.contains("<div id=\"app\">"), "missing root mount point");
    // Vite emite hashed assets bajo /assets/
    assert!(
        body.contains("/assets/index-"),
        "missing /assets/index- reference"
    );
    // Cero referencias externas (UI-01)
    assert!(
        !body.contains("https://"),
        "external https:// reference detected in index.html"
    );
    assert!(
        !body.contains("//fonts."),
        "external font URL detected"
    );
    assert!(
        !body.contains("googleapis"),
        "googleapis reference leaked"
    );
    assert!(
        !body.contains("googleusercontent"),
        "google reference leaked"
    );
}

#[tokio::test]
#[serial]
async fn get_assets_returns_200() {
    let app = router();

    // Primero leer index.html para obtener el nombre del JS hasheado
    let resp = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .expect("oneshot");
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body = std::str::from_utf8(&body_bytes).unwrap();

    // Encontrar el primer /assets/index-*.js referenciado
    let needle = "/assets/index-";
    let start = body.find(needle).expect("no /assets/index- in HTML");
    let rest = &body[start..];
    let end = rest.find('"').expect("no closing quote");
    let asset_path = &rest[..end];
    assert!(
        asset_path.ends_with(".js"),
        "expected .js, got {asset_path}"
    );

    let resp2 = app
        .oneshot(
            Request::builder()
                .uri(asset_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(resp2.status(), StatusCode::OK);
    let ct = resp2
        .headers()
        .get("content-type")
        .expect("content-type")
        .to_str()
        .unwrap();
    assert!(
        ct.contains("javascript") || ct.contains("ecmascript"),
        "expected js MIME, got {ct}"
    );
}
