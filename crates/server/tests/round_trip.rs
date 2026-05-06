#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
const FIXTURE_XPUB: &str = include_str!("fixtures/xpub.txt");

#[tokio::test]
async fn encrypt_then_decrypt_roundtrip() {
    let app = bed_server::router();

    let body = json!({ "descriptor": FIXTURE_DESC.trim() }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/encrypt")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("build req: {e}"));
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("encrypt: {e}"));
    assert_eq!(resp.status(), StatusCode::OK, "encrypt status");
    let bytes = to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|e| panic!("body: {e}"));
    let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
    let armored = parsed["armored"]
        .as_str()
        .unwrap_or_else(|| panic!("no armored field: {parsed}"))
        .to_string();
    assert!(parsed["bed_b64"].is_string(), "bed_b64 must be string");
    assert!(
        parsed["qr_png_b64"].is_string(),
        "qr_png_b64 must be string"
    );

    // Multipart with armored bed + xpub
    let boundary = "----testboundary";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{a}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
        b = boundary,
        a = armored,
        x = FIXTURE_XPUB.trim(),
    );
    let req = Request::builder()
        .method("POST")
        .uri("/api/decrypt")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("build req: {e}"));
    let resp = app
        .oneshot(req)
        .await
        .unwrap_or_else(|e| panic!("decrypt: {e}"));
    assert_eq!(resp.status(), StatusCode::OK, "decrypt status");
    let bytes = to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|e| panic!("body: {e}"));
    let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));

    // Normalize h/apostrophe: miniscript re-serializes 48h as 48', both BIP-380 valid.
    // Strip the trailing checksum (#xxxxxxxx) before comparing: miniscript may
    // re-compute it after internal normalization (e.g. canonical key ordering in sortedmulti).
    fn strip_checksum(s: &str) -> &str {
        // BIP-380 checksum is '#' + 8 alphanumeric chars at the very end.
        if let Some(pos) = s.rfind('#') {
            if s.len() - pos == 9 {
                return &s[..pos];
            }
        }
        s
    }
    let got_raw = parsed["descriptor"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    let got = strip_checksum(&got_raw).replace('\'', "h");
    let expected_raw = FIXTURE_DESC.trim().to_string();
    let expected = strip_checksum(&expected_raw).replace('\'', "h");
    assert_eq!(got, expected, "round-trip descriptor mismatch");
}

#[tokio::test]
async fn decrypt_with_binary_bed_works() {
    // Use bed_b64 (raw base64 of binary, no PEM headers) to verify the
    // crate's auto-detect path (set_encrypted_payload base64 branch).
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
    let bed_b64 = parsed["bed_b64"]
        .as_str()
        .unwrap_or_else(|| panic!("no bed_b64"))
        .to_string();

    // Send bed_b64 raw text (not armored) — handler should pass to crate
    // whose set_encrypted_payload decodes base64 automatically.
    let boundary = "----b";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{d}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
        b = boundary,
        d = bed_b64,
        x = FIXTURE_XPUB.trim(),
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
    assert_eq!(resp.status(), StatusCode::OK);
}
