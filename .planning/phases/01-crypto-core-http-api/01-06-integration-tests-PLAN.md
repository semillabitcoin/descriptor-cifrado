---
phase: 01-crypto-core-http-api
plan: 06
type: execute
wave: 4
depends_on: ["01-05"]
files_modified:
  - crates/server/tests/round_trip.rs
  - crates/server/tests/no_leak.rs
  - crates/server/tests/validation.rs
  - crates/server/tests/fixtures/desc.txt
  - crates/server/tests/fixtures/xpub.txt
  - crates/server/tests/fixtures/wrong_xpub.txt
autonomous: true
requirements: [CORE-02, CI-02, SEC-01]
must_haves:
  truths:
    - "Round-trip test envía descriptor a /api/encrypt, captura armored, lo envía a /api/decrypt con xpub correcta y assertea que el descriptor recuperado == original"
    - "no_leak test asserta que el descriptor cleartext NO aparece en logs capturados durante encrypt+decrypt"
    - "validation test asserta que bare xpub → 422 MISSING_MULTIPATH_WILDCARD; xpub incorrecta → 422 XPUB_MISMATCH"
    - "Tests usan tower::ServiceExt::oneshot + axum::body::to_bytes (no sockets, no axum-test, no reqwest)"
  artifacts:
    - path: "crates/server/tests/round_trip.rs"
      provides: "End-to-end encrypt+decrypt vía HTTP layer"
      contains: "encrypt_then_decrypt_roundtrip"
    - path: "crates/server/tests/no_leak.rs"
      provides: "Descriptor never appears in captured tracing output"
      contains: "descriptor_never_appears_in_logs"
    - path: "crates/server/tests/validation.rs"
      provides: "Wildcard validation HTTP boundary tests"
      contains: "bare_xpub_returns_422"
  key_links:
    - from: "crates/server/tests/round_trip.rs"
      to: "bed_server::router()"
      via: "tower::ServiceExt::oneshot"
      pattern: "oneshot"
    - from: "crates/server/tests/no_leak.rs"
      to: "tracing::subscriber::with_default + MakeWriter shared buffer"
      via: "captured stdout substring search"
      pattern: "MakeWriter"
---

<objective>
Cerrar las invariantes de Phase 1 con tests de integración: round-trip end-to-end (CI-02), no-leak en logs (SEC-01, CI-02), y validación de descriptor inválido (CORE-03 boundary). Todos los tests usan `tower::ServiceExt::oneshot` (D-23) — sin bind de socket, sin `axum-test`, sin `reqwest`. El test de no-leak usa `MakeWriter` con buffer compartido (D-20) porque `TestWriter` no expone los bytes capturados al test.

Purpose: Que CI valide en cada PR que (a) encriptar y descifrar funciona end-to-end vía HTTP, (b) el descriptor NUNCA aparece en logs, y (c) los descriptors inválidos producen error tipado HTTP 422 con el mensaje en castellano correcto.
Output: 3 archivos de test pasando; CI job `test` verde.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/01-crypto-core-http-api/01-CONTEXT.md
@.planning/phases/01-crypto-core-http-api/01-RESEARCH.md
@/tmp/bed-test/desc.txt
@/tmp/bed-test/xpub.txt

<interfaces>
From bed-server (created in Plan 05):
```rust
pub fn router() -> Router;
```

Patterns from RESEARCH.md:
- §"Test infra: tower::ServiceExt::oneshot for round-trip" — full body of round_trip.rs
- §"Test infra: no-leak via TestWriter" — full body of no_leak.rs (uses MakeWriter shared buffer)

Test imports needed:
```rust
use axum::{body::{to_bytes, Body}, http::{Request, StatusCode}};
use serde_json::{json, Value};
use tower::ServiceExt;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: round_trip.rs + validation.rs + fixtures</name>
  <files>crates/server/tests/round_trip.rs, crates/server/tests/validation.rs, crates/server/tests/fixtures/desc.txt, crates/server/tests/fixtures/xpub.txt, crates/server/tests/fixtures/wrong_xpub.txt</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Test infra: tower::ServiceExt::oneshot for round-trip")
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-23, D-24)
    - /tmp/bed-test/desc.txt + /tmp/bed-test/xpub.txt
  </read_first>
  <behavior>
    - encrypt_then_decrypt_roundtrip: POST encrypt fixture → captura armored → POST decrypt con xpub correcta → assertEq al original
    - encrypt_with_bare_xpub_returns_422: POST encrypt con `wsh(pk(xpub))` → 422 + body code "MISSING_MULTIPATH_WILDCARD"
    - decrypt_with_wrong_xpub_returns_422: POST decrypt con xpub aleatoria → 422 + body code "XPUB_MISMATCH"
    - decrypt_missing_bed_field_returns_400: POST decrypt sin field `bed` → 400 BAD_REQUEST
    - decrypt_with_binary_bed_works: POST decrypt con bed_bytes binarios (raw, no armored) → 200
  </behavior>
  <action>
    Copiar fixtures: `cp /tmp/bed-test/desc.txt crates/server/tests/fixtures/desc.txt && cp /tmp/bed-test/xpub.txt crates/server/tests/fixtures/xpub.txt`. Crear `wrong_xpub.txt` con un xpub aleatorio no relacionado:
    ```
    xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ
    ```
    NOTA al executor: si ese xpub colisiona con la fixture (improbable), generar uno con `bitcoin-cli` o usar un xpub público bien conocido distinto.

    Crear `crates/server/tests/round_trip.rs` con el contenido de RESEARCH.md §"Test infra: tower::ServiceExt::oneshot for round-trip":

    ```rust
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
        let resp = app.clone().oneshot(req).await.unwrap_or_else(|e| panic!("encrypt: {e}"));
        assert_eq!(resp.status(), StatusCode::OK, "encrypt status");
        let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap_or_else(|e| panic!("body: {e}"));
        let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
        let armored = parsed["armored"]
            .as_str()
            .unwrap_or_else(|| panic!("no armored field: {parsed}"))
            .to_string();
        assert!(parsed["bed_b64"].is_string(), "bed_b64 must be string");
        assert!(parsed["qr_png_b64"].is_string(), "qr_png_b64 must be string");

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
            .header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap_or_else(|e| panic!("build req: {e}"));
        let resp = app.oneshot(req).await.unwrap_or_else(|e| panic!("decrypt: {e}"));
        assert_eq!(resp.status(), StatusCode::OK, "decrypt status");
        let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap_or_else(|e| panic!("body: {e}"));
        let parsed: Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
        assert_eq!(
            parsed["descriptor"].as_str().unwrap_or("").trim(),
            FIXTURE_DESC.trim(),
            "round-trip descriptor mismatch"
        );
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
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        let bed_b64 = parsed["bed_b64"].as_str().unwrap().to_string();

        // Send bed_b64 raw text (not armored) — handler should pass to crate
        // whose set_encrypted_payload decodes base64 automatically.
        let boundary = "----b";
        let body = format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{d}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
            b = boundary, d = bed_b64, x = FIXTURE_XPUB.trim(),
        );
        let req = Request::builder()
            .method("POST")
            .uri("/api/decrypt")
            .header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
    ```

    Crear `crates/server/tests/validation.rs`:
    ```rust
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
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["error"]["code"].as_str(), Some("MISSING_MULTIPATH_WILDCARD"));
        // Castellano message check (D-09):
        let msg = parsed["error"]["message"].as_str().unwrap_or("");
        assert!(msg.contains("<0;1>/*"), "message must mention <0;1>/*: {msg}");
        assert!(msg.contains("xpub on-chain"), "message must mention xpub on-chain: {msg}");
    }

    #[tokio::test]
    async fn encrypt_with_malformed_json_returns_400() {
        let app = bed_server::router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/encrypt")
            .header("content-type", "application/json")
            .body(Body::from("{not json"))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
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
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        let armored = parsed["armored"].as_str().unwrap().to_string();

        let boundary = "----b";
        let body = format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{a}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
            b = boundary, a = armored, x = WRONG_XPUB.trim(),
        );
        let req = Request::builder()
            .method("POST")
            .uri("/api/decrypt")
            .header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
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
            .header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    ```

    NOTA al executor: los `unwrap()` en archivos `crates/server/tests/*.rs` están permitidos porque el lint `unwrap_used` se aplica solo a paths de request (`crates/server/src/`); en tests son idiomáticos y aceptables. Si clippy lo bloquea por aplicación de lint a workspace, sustituir por `unwrap_or_else(|e| panic!(...))` patrón usado en Plan 04.
  </action>
  <verify>
    <automated>cargo test -p bed-server --test round_trip --test validation 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p bed-server --test round_trip` exits 0 (ambos tests pasan)
    - `cargo test -p bed-server --test validation` exits 0 (4 tests pasan)
    - `crates/server/tests/round_trip.rs` contiene literal `tower::ServiceExt`
    - `crates/server/tests/round_trip.rs` contiene literal `oneshot`
    - `crates/server/tests/round_trip.rs` contiene literal `axum::body::to_bytes`
    - NO hay `axum-test` en `crates/server/Cargo.toml` (`grep -c axum-test crates/server/Cargo.toml` == 0)
    - NO hay `reqwest` en `crates/server/Cargo.toml` (`grep -c reqwest crates/server/Cargo.toml` == 0)
    - Fixture diff: `diff crates/server/tests/fixtures/desc.txt /tmp/bed-test/desc.txt` exits 0
  </acceptance_criteria>
  <done>Round-trip end-to-end + validación de descriptor inválido + xpub incorrecta tests pasando; sin axum-test ni reqwest.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: no_leak.rs (SEC-01 / CI-02) — descriptor never appears in tracing output</name>
  <files>crates/server/tests/no_leak.rs</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Test infra: no-leak via TestWriter" — copia el patrón MakeWriter)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-20)
    - .planning/research/PITFALLS.md (Pitfall 2)
  </read_first>
  <behavior>
    - Test ejecuta encrypt+decrypt round-trip dentro de un scope con tracing subscriber custom (MakeWriter sobre buffer compartido)
    - Tras el round-trip, el buffer es leído como UTF-8 string
    - assert que el descriptor cleartext NO aparece como substring en el buffer
    - assert que el xpub NO aparece como substring en el buffer
  </behavior>
  <action>
    Crear `crates/server/tests/no_leak.rs` con el patrón de RESEARCH.md §"Test infra: no-leak via TestWriter":

    ```rust
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
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
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
                let resp = app.clone().oneshot(req).await.unwrap_or_else(|e| panic!("oneshot: {e}"));
                let bytes = to_bytes(resp.into_body(), 1024 * 1024)
                    .await
                    .unwrap_or_else(|e| panic!("body: {e}"));
                let parsed: Value =
                    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json: {e}"));
                let armored = parsed["armored"].as_str().unwrap_or("").to_string();

                let boundary = "----b";
                let body = format!(
                    "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{a}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
                    b = boundary, a = armored, x = xpub,
                );
                let req = Request::builder()
                    .method("POST")
                    .uri("/api/decrypt")
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .body(Body::from(body))
                    .unwrap_or_else(|e| panic!("req: {e}"));
                let _resp = app.oneshot(req).await.unwrap_or_else(|e| panic!("oneshot: {e}"));
            });
        });

        let captured = String::from_utf8_lossy(
            &buf.lock().unwrap_or_else(|e| panic!("lock: {e}")).clone(),
        )
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
    ```
  </action>
  <verify>
    <automated>cargo test -p bed-server --test no_leak 2>&1 | tail -10 && cargo test --workspace --all-features --locked 2>&1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p bed-server --test no_leak` exits 0
    - `cargo test --workspace --all-features --locked` exits 0 (TODO el suite del workspace pasa)
    - `crates/server/tests/no_leak.rs` contiene literal `MakeWriter`
    - `crates/server/tests/no_leak.rs` contiene literal `with_default`
    - `crates/server/tests/no_leak.rs` contiene literal `assert!(!captured.contains(`
    - `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0
    - `cargo fmt --all -- --check` exits 0
    - `cargo deny check` exits 0
  </acceptance_criteria>
  <done>no-leak test pasa: descriptor + xpub NUNCA aparecen en el buffer de tracing capturado durante encrypt+decrypt round-trip.</done>
</task>

</tasks>

<verification>
- `cargo test --workspace --all-features --locked` exits 0
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo deny check` exits 0
- Round-trip vía HTTP funciona; descriptor inválido → 422; xpub incorrecta → 422; no descriptor en logs
</verification>

<success_criteria>
- CORE-02: round-trip determinista vía HTTP layer
- CI-02: pipeline corre round-trip + no-leak (jobs verdes)
- SEC-01: descriptor cleartext nunca aparece en tracing output
- Toda la fase verifica end-to-end: `cargo test --workspace --locked` exit 0 + bind 127.0.0.1:8080 + `ldd target/release/bed-server` SIN libssl/native-tls (verificable manualmente o en SUMMARY)
</success_criteria>

<output>
Tras completar, crear `.planning/phases/01-crypto-core-http-api/01-06-integration-tests-SUMMARY.md` documentando:
- Lista completa de tests del workspace y su estado
- Output de `cargo test --workspace --all-features --locked` (resumen)
- Output de `ldd target/release/bed-server | grep -E "libssl|native-tls"` (debe ser vacío — confirma SEC-03)
- Confirmación de que el descriptor NO aparece en el buffer capturado (cuántos bytes capturó el subscriber)
- Lista de los 20 requirement IDs cerrados en Phase 1 con plan de origen
</output>
