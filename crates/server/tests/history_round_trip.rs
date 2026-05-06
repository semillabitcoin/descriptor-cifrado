#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use bed_server::router;
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;

fn set_data_dir(path: &std::path::Path) {
    std::env::set_var("BED_DATA_DIR", path);
}

async fn json_body(resp: axum::response::Response) -> (StatusCode, Value) {
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, v)
}

#[tokio::test]
#[serial]
async fn round_trip_post_list_get_delete() {
    let tmp = tempfile::tempdir().unwrap();
    set_data_dir(tmp.path());

    let app = router();
    let fake_bed_bytes = b"FAKE_BED_BYTES_FOR_ROUND_TRIP_TEST_OPAQUE_BLOB";
    let bed_b64 = B64.encode(fake_bed_bytes);

    // POST /api/history
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/history")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "bed_b64": bed_b64 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let (st, body) = json_body(resp).await;
    assert_eq!(st, StatusCode::OK);
    let id = body["id"].as_str().unwrap().to_string();
    assert_eq!(id.len(), 8);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));

    // GET /api/history list
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (st, body) = json_body(resp).await;
    assert_eq!(st, StatusCode::OK);
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"].as_str().unwrap(), id);

    // GET /api/history/{id}
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/history/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (st, body) = json_body(resp).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["bed_b64"].as_str().unwrap(), bed_b64);
    assert!(!body["armored"].as_str().unwrap().is_empty());
    assert!(!body["qr_png_b64"].as_str().unwrap().is_empty());

    // DELETE /api/history/{id}
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/history/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // GET again → 404
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/history/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (st, body) = json_body(resp).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"].as_str().unwrap(), "HISTORY_NOT_FOUND");
}

#[tokio::test]
#[serial]
async fn invalid_id_returns_422() {
    let tmp = tempfile::tempdir().unwrap();
    set_data_dir(tmp.path());
    let app = router();

    for bad in &["xx", "AAAAAAAA", "a3f7b2c", "a3f7b2c1x", "a3f7b2g1"] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/history/{bad}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let (st, body) = json_body(resp).await;
        assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY, "id={bad}");
        assert_eq!(body["error"]["code"].as_str().unwrap(), "HISTORY_INVALID_ID");
    }
}

#[tokio::test]
#[serial]
async fn empty_dir_returns_empty_entries() {
    let tmp = tempfile::tempdir().unwrap();
    set_data_dir(tmp.path());
    let app = router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (st, body) = json_body(resp).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["entries"].as_array().unwrap().len(), 0);
}

#[tokio::test]
#[serial]
async fn invalid_base64_returns_400() {
    let tmp = tempfile::tempdir().unwrap();
    set_data_dir(tmp.path());
    let app = router();
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/history")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "bed_b64": "not-valid-base64!!!" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
