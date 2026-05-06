#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

//! SEC-01 / CI-02: ensure no handler logs the descriptor cleartext.
//!
//! Pattern: install a tracing subscriber whose writer is a shared in-memory
//! buffer (MakeWriter impl). Run the full encrypt+decrypt round-trip while
//! that subscriber is the default. Read the buffer afterwards and assert
//! the descriptor (and xpub) substring never appears.

use std::sync::{Arc, Mutex};

use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use tracing_subscriber::fmt::{self, MakeWriter};

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
const FIXTURE_XPUB: &str = include_str!("fixtures/xpub.txt");

#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl<'a> MakeWriter<'a> for SharedBuf {
    type Writer = SharedWriter;
    fn make_writer(&'a self) -> Self::Writer {
        SharedWriter(self.0.clone())
    }
}

struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut g) = self.0.lock() {
            g.extend_from_slice(buf);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn descriptor_never_appears_in_logs() {
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let make = SharedBuf(buf.clone());
    let sub = fmt::Subscriber::builder()
        .with_writer(make)
        .with_max_level(tracing::Level::TRACE)
        .finish();

    let descriptor = FIXTURE_DESC.trim().to_string();
    let xpub = FIXTURE_XPUB.trim().to_string();

    tracing::subscriber::with_default(sub, || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|e| panic!("rt build: {e}"));
        rt.block_on(async {
            let app = bed_server::router();

            let body = json!({ "descriptor": descriptor }).to_string();
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
                .unwrap_or_else(|e| panic!("body: {e}"));
            let parsed: Value =
                serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
            let armored = parsed["armored"].as_str().unwrap_or("").to_string();

            let boundary = "----b";
            let body = format!(
                "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{a}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
                b = boundary,
                a = armored,
                x = xpub,
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
            let _resp = app
                .oneshot(req)
                .await
                .unwrap_or_else(|e| panic!("oneshot: {e}"));
        });
    });

    let captured =
        String::from_utf8_lossy(&buf.lock().unwrap_or_else(|e| panic!("lock: {e}")).clone())
            .to_string();

    assert!(
        !captured.contains(&descriptor),
        "descriptor leaked into logs:\n{captured}"
    );
    // Also assert no substring of the xpub (key part of the descriptor) leaked.
    let needle: &str = xpub.trim();
    assert!(
        !captured.contains(needle),
        "xpub leaked into logs:\n{captured}"
    );
}
