#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! HIST-03: el descriptor en claro NUNCA aparece en archivos persistidos
//! bajo `BED_DATA_DIR`. Reusa el fixture de Phase 1 (descriptor multisig
//! 2-of-3 con derivación <0;1>/* válido contra miniscript 12.3.5).

use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use bed_server::router;
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");

#[tokio::test]
#[serial]
async fn descriptor_cleartext_never_persisted_in_history_dir() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("BED_DATA_DIR", tmp.path());
    let app = router();
    let descriptor = FIXTURE_DESC.trim().to_string();

    // 1. Encrypt el descriptor — no persiste; retorna outputs.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/encrypt")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "descriptor": descriptor })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "encrypt failed — descriptor inválido?"
    );
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    let bed_b64 = v["bed_b64"].as_str().unwrap().to_string();

    // 2. POST /api/history con el bed_b64 → escribe archivo en tmp.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/history")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "bed_b64": bed_b64, "label": "leak test" }))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Para cada archivo en tmp, leer bytes y verificar que NINGÚN substring
    //    significativo del descriptor en claro aparece (HIST-03). Substrings
    //    derivados del fixture real:
    //      - function name "wsh(sortedmulti"
    //      - 3 fingerprints: 68a9ec24, f91be7a4, e3a2b8a8
    //      - 3 xpub fragmentos largos
    //      - checksum #rzf36yej
    //      - multipath wildcard <0;1>/*
    let needles: &[&[u8]] = &[
        b"wsh(sortedmulti",
        b"xpub6PLACEHOLDER2xxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        b"xpub6DhWabCb9j5vGn3VU2YJ9kNgk258AoBcQWHQPstRY2",
        b"xpub6DtCKqADjbNNP3Yg8TC88yqqpZfepov4feSKD8HrJG",
        b"68a9ec24",
        b"f91be7a4",
        b"e3a2b8a8",
        b"#rzf36yej",
        b"<0;1>/*",
    ];

    let mut rd = tokio::fs::read_dir(tmp.path()).await.unwrap();
    let mut file_count = 0;
    while let Some(entry) = rd.next_entry().await.unwrap() {
        file_count += 1;
        let path = entry.path();
        let bytes = tokio::fs::read(&path).await.unwrap();
        for needle in needles {
            assert!(
                !bytes.windows(needle.len()).any(|w| w == *needle),
                "HIST-03 violado: descriptor cleartext detectado en {} (substring: {:?})",
                path.display(),
                std::str::from_utf8(needle).unwrap_or("<binary>"),
            );
        }
    }
    assert!(file_count >= 1, "ningún archivo escrito en BED_DATA_DIR");
}
