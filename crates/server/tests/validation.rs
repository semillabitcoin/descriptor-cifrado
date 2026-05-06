#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
const WRONG_XPUB: &str = include_str!("fixtures/wrong_xpub.txt");

#[tokio::test]
async fn encrypt_with_bare_xpub_returns_422() {
    let app = bed_server::router();
    let bare = "wsh(pk(xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ))";
    let body = json!({ "descriptor": bare }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/encrypt")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("req: {e}"));
    let resp = app
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("oneshot: {e}"));
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap_or_else(|e| panic!("bytes: {e}"));
    let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
    assert_eq!(
        parsed["error"]["code"].as_str(),
        Some("MISSING_MULTIPATH_WILDCARD")
    );
    // Castellano message check (D-09):
    let msg = parsed["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("<0;1>/*"),
        "message must mention <0;1>/*: {msg}"
    );
    assert!(
        msg.contains("xpub on-chain"),
        "message must mention xpub on-chain: {msg}"
    );
}

#[tokio::test]
async fn encrypt_with_malformed_json_returns_400() {
    let app = bed_server::router();
    let req = Request::builder()
        .method("POST")
        .uri("/api/encrypt")
        .header("content-type", "application/json")
        .body(Body::from("{not json"))
        .unwrap_or_else(|e| panic!("req: {e}"));
    let resp = app
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("oneshot: {e}"));
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn decrypt_with_wrong_xpub_returns_422() {
    let app = bed_server::router();

    let body = json!({ "descriptor": FIXTURE_DESC.trim() }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/encrypt")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("req: {e}"));
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("oneshot: {e}"));
    let bytes = to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|e| panic!("bytes: {e}"));
    let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
    let armored = parsed["armored"]
        .as_str()
        .unwrap_or_else(|| panic!("no armored"))
        .to_string();

    let boundary = "----b";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{a}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
        b = boundary,
        a = armored,
        x = WRONG_XPUB.trim(),
    );
    let req = Request::builder()
        .method("POST")
        .uri("/api/decrypt")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("req: {e}"));
    let resp = app
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("oneshot: {e}"));
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap_or_else(|e| panic!("bytes: {e}"));
    let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
    assert_eq!(parsed["error"]["code"].as_str(), Some("XPUB_MISMATCH"));
}

#[tokio::test]
async fn decrypt_missing_bed_field_returns_400() {
    let app = bed_server::router();
    let boundary = "----b";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\nxpub-only\r\n--{b}--\r\n",
        b = boundary
    );
    let req = Request::builder()
        .method("POST")
        .uri("/api/decrypt")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("req: {e}"));
    let resp = app
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("oneshot: {e}"));
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
