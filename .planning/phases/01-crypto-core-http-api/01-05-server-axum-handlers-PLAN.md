---
phase: 01-crypto-core-http-api
plan: 05
type: execute
wave: 3
depends_on: ["01-03", "01-04"]
files_modified:
  - crates/server/src/lib.rs
  - crates/server/src/error.rs
  - crates/server/src/state.rs
  - crates/server/src/routes/mod.rs
  - crates/server/src/routes/encrypt.rs
  - crates/server/src/routes/decrypt.rs
autonomous: true
requirements: [ENC-01, ENC-02, ENC-03, ENC-04, ENC-05, DEC-01, DEC-02, DEC-03, DEC-04, SEC-01, SEC-02, CORE-04, CORE-05]
must_haves:
  truths:
    - "POST /api/encrypt acepta JSON {descriptor} y devuelve {bed_b64, armored, qr_png_b64}"
    - "POST /api/decrypt acepta multipart con bed (binario o armored) + xpub (texto o file)"
    - "Handlers tienen #[tracing::instrument(skip_all)]"
    - "AppError implementa IntoResponse con body shape {error: {code, message}}"
    - "Descriptor envuelto en Zeroizing<String> en la PRIMERA línea del handler tras Json extract"
    - "MissingMultipathWildcard, DescriptorParse, XpubMismatch, QrTooLarge → HTTP 422"
    - "JSON malformado / multipart inválido → HTTP 400"
    - "Internal/Crypto → HTTP 500 con body genérico"
    - "Server file binda a 127.0.0.1:8080 (constante hardcoded de Plan 01)"
  artifacts:
    - path: "crates/server/src/error.rs"
      provides: "AppError enum + IntoResponse impl + From<CoreError>"
      exports: ["AppError"]
    - path: "crates/server/src/routes/encrypt.rs"
      provides: "POST /api/encrypt handler"
      contains: "post_encrypt"
    - path: "crates/server/src/routes/decrypt.rs"
      provides: "POST /api/decrypt handler"
      contains: "post_decrypt"
    - path: "crates/server/src/lib.rs"
      provides: "router() function for testability via oneshot"
      exports: ["router", "AppError"]
  key_links:
    - from: "crates/server/src/routes/encrypt.rs"
      to: "bed_core::encrypt_descriptor"
      via: "&mut Zeroizing<String> pipeline"
      pattern: "encrypt_descriptor\\(&mut"
    - from: "crates/server/src/routes/decrypt.rs"
      to: "bed_core::{decode_armored, decrypt_payload}"
      via: "armored detection + crate decrypt"
      pattern: "decode_armored|decrypt_payload"
    - from: "crates/server/src/error.rs"
      to: "axum::response::IntoResponse"
      via: "impl IntoResponse for AppError"
      pattern: "impl IntoResponse for AppError"
---

<objective>
Implementar la capa HTTP completa: `AppError` con `IntoResponse` (D-16, D-17, ENC-05), router con dos endpoints (`POST /api/encrypt` JSON D-05, `POST /api/decrypt` multipart D-06), handlers con `#[tracing::instrument(skip_all)]` (D-19, SEC-01), Zeroizing en la primera línea (D-10, CORE-04), y mapeo de errores del crate al `AppError`. NOTA al executor: ENC-02 + ENC-05 + DEC-02 + DEC-03 son **API substrate only** — la presentación UI es Phase 2; este plan solo entrega el contrato HTTP estable.

Purpose: Cerrar la API pública estable que la SPA Phase 2 consumirá. Tras este plan, `curl POST /api/encrypt` con JSON funciona end-to-end y todos los errores tipados aparecen como HTTP 422 con mensajes en castellano.
Output: `crates/server` con handlers reales reemplazando los stubs de Plan 01; `cargo build` y `cargo run -p bed-server` arrancan el server en `127.0.0.1:8080`.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/01-crypto-core-http-api/01-CONTEXT.md
@.planning/phases/01-crypto-core-http-api/01-RESEARCH.md
@.planning/research/PITFALLS.md

<interfaces>
From bed-core (created in Plans 03, 04):
```rust
pub fn encrypt_descriptor(cleartext: &mut Zeroizing<String>) -> Result<EncryptOutput, CoreError>;
pub struct EncryptOutput { pub bed_bytes: Vec<u8>, pub armored: String, pub qr_png: Vec<u8> }
pub fn decrypt_payload(bed_bytes: &[u8], xpub_str: &str) -> Result<Zeroizing<String>, CoreError>;
pub fn decode_armored(input: &str) -> Result<Vec<u8>, ArmoredError>;
pub enum CoreError {
    MissingMultipathWildcard,
    DescriptorParse,
    XpubMismatch,
    QrTooLarge { size: usize, max: usize },
    Armored(String),
    Crypto,
}
```

From axum 0.8 (verified):
- `axum::extract::{Json, Multipart}` — extractors
- `axum::response::{IntoResponse, Response}` — handler return
- `axum::http::StatusCode` — status codes
- `Multipart::next_field()` returns `Result<Option<Field>, MultipartError>`
- `Field::name()` returns `Option<&str>`
- `Field::bytes()` consumes field returning `Result<Bytes, MultipartError>`
- `Field::text()` consumes field returning `Result<String, MultipartError>`

Patterns from RESEARCH.md:
- §"Pattern 1: Zeroizing at the parse boundary" — encrypt handler full body
- §"Pattern 5: AppError with IntoResponse" — full AppError + impl
- §"Pattern 6: Multipart for decrypt" — decrypt handler full body
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: AppError + IntoResponse + state.rs + routes/mod.rs</name>
  <files>crates/server/src/error.rs, crates/server/src/state.rs, crates/server/src/routes/mod.rs</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Pattern 5: AppError with IntoResponse" — bloque verbatim)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-16, D-17)
    - crates/core/src/error.rs (CoreError variants para mapping)
  </read_first>
  <behavior>
    - AppError variants: MissingMultipathWildcard, DescriptorParse, XpubMismatch, QrTooLarge {size, max}, Internal, BadRequest(String)
    - IntoResponse mapping: MissingMultipathWildcard|DescriptorParse|XpubMismatch|QrTooLarge → 422; BadRequest → 400; Internal → 500
    - Response body: {"error": {"code": "<UPPER_SNAKE>", "message": "<castellano>"}}
    - From<CoreError> maps every CoreError variant to a AppError
  </behavior>
  <action>
    Crear `crates/server/src/error.rs` con el contenido de RESEARCH.md §"Pattern 5: AppError with IntoResponse", mapeando CoreError correctamente:

    ```rust
    //! AppError — single error type for the HTTP layer (D-16, D-17).
    //!
    //! Variants map to status codes:
    //!   MissingMultipathWildcard | DescriptorParse | XpubMismatch | QrTooLarge → 422
    //!   BadRequest(_) → 400
    //!   Internal → 500
    //!
    //! Response body shape (D-17):
    //!   {"error": {"code": "<UPPER_SNAKE>", "message": "<castellano>"}}

    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };
    use serde::Serialize;
    use thiserror::Error;

    use bed_core::CoreError;

    #[derive(Error, Debug)]
    pub enum AppError {
        #[error("El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain.")]
        MissingMultipathWildcard,

        #[error("No se pudo parsear el descriptor.")]
        DescriptorParse,

        #[error("La xpub proporcionada no descifra este .bed (no es un cosigner válido).")]
        XpubMismatch,

        #[error("El descriptor cifrado excede capacidad QR ({size} > {max} bytes). Usá el archivo .bed o el armored.")]
        QrTooLarge { size: usize, max: usize },

        #[error("internal error")]
        Internal,

        #[error("solicitud inválida: {0}")]
        BadRequest(String),
    }

    #[derive(Serialize)]
    struct ErrorBody {
        code: &'static str,
        message: String,
    }

    #[derive(Serialize)]
    struct ErrorEnvelope {
        error: ErrorBody,
    }

    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            let (status, code): (StatusCode, &'static str) = match &self {
                AppError::MissingMultipathWildcard => (StatusCode::UNPROCESSABLE_ENTITY, "MISSING_MULTIPATH_WILDCARD"),
                AppError::DescriptorParse => (StatusCode::UNPROCESSABLE_ENTITY, "DESCRIPTOR_PARSE"),
                AppError::XpubMismatch => (StatusCode::UNPROCESSABLE_ENTITY, "XPUB_MISMATCH"),
                AppError::QrTooLarge { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "QR_TOO_LARGE"),
                AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
                AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL"),
            };
            let body = ErrorEnvelope {
                error: ErrorBody {
                    code,
                    message: self.to_string(),
                },
            };
            (status, Json(body)).into_response()
        }
    }

    impl From<CoreError> for AppError {
        fn from(e: CoreError) -> Self {
            match e {
                CoreError::MissingMultipathWildcard => AppError::MissingMultipathWildcard,
                CoreError::DescriptorParse => AppError::DescriptorParse,
                CoreError::XpubMismatch => AppError::XpubMismatch,
                CoreError::QrTooLarge { size, max } => AppError::QrTooLarge { size, max },
                CoreError::Armored(msg) => AppError::BadRequest(msg),
                CoreError::Crypto => AppError::Internal,
            }
        }
    }
    ```

    Crear `crates/server/src/state.rs` (placeholder vacío para Phase 2):
    ```rust
    //! AppState — placeholder for shared state. Phase 1 has no shared state;
    //! this exists so Phase 2's history toggle can be added without restructuring
    //! the router signature.

    #[derive(Clone, Default)]
    pub struct AppState;
    ```

    Crear `crates/server/src/routes/mod.rs`:
    ```rust
    pub mod decrypt;
    pub mod encrypt;
    ```
  </action>
  <verify>
    <automated>cargo build -p bed-server 2>&1 | tail -5 && grep -q 'impl IntoResponse for AppError' crates/server/src/error.rs && grep -q 'MISSING_MULTIPATH_WILDCARD' crates/server/src/error.rs</automated>
  </verify>
  <acceptance_criteria>
    - `cargo build -p bed-server` exits 0 (handlers stub aún en `lib.rs` desde Plan 01; routes/mod.rs declara módulos pero los archivos los crea Task 2 — comentar `pub mod routes;` temporalmente o ejecutar Task 2 inmediatamente)
    - `crates/server/src/error.rs` contiene literal `MISSING_MULTIPATH_WILDCARD`
    - `crates/server/src/error.rs` contiene literal `DESCRIPTOR_PARSE`
    - `crates/server/src/error.rs` contiene literal `XPUB_MISMATCH`
    - `crates/server/src/error.rs` contiene literal `QR_TOO_LARGE`
    - `crates/server/src/error.rs` contiene literal `INTERNAL`
    - `crates/server/src/error.rs` contiene literal `BAD_REQUEST`
    - `crates/server/src/error.rs` contiene literal `StatusCode::UNPROCESSABLE_ENTITY`
    - `crates/server/src/error.rs` contiene `impl From<CoreError> for AppError`
    - Mensaje EXACTO: `"El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain."`
  </acceptance_criteria>
  <done>AppError completo + IntoResponse impl + mapping desde CoreError; mensajes en castellano EXACTOS; status codes correctos.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implementar handlers /api/encrypt y /api/decrypt + actualizar router</name>
  <files>crates/server/src/routes/encrypt.rs, crates/server/src/routes/decrypt.rs, crates/server/src/lib.rs</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Pattern 1: Zeroizing at the parse boundary" + §"Pattern 6: Multipart for decrypt")
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-05, D-06, D-10, D-19)
    - .planning/research/PITFALLS.md (Pitfall 2 + 4 — tracing skip_all + zeroize at boundary)
    - crates/server/src/lib.rs (estado actual con stubs)
  </read_first>
  <behavior>
    - POST /api/encrypt con `{"descriptor": "<0;1>/* fixture"}` → 200 + `{bed_b64, armored, qr_png_b64}`
    - POST /api/encrypt con bare xpub → 422 + `{"error":{"code":"MISSING_MULTIPATH_WILDCARD", ...}}`
    - POST /api/encrypt con JSON malformado → 400 (axum default JsonRejection)
    - POST /api/decrypt multipart con `bed` + `xpub` → 200 + `{"descriptor": "..."}`
    - POST /api/decrypt con `bed` armored (texto) → 200 (decode_armored unwraps PEM)
    - POST /api/decrypt con `bed` binario (`BIPXXX...`) → 200 (auto-detect en crate)
    - POST /api/decrypt con xpub incorrecta → 422 XPUB_MISMATCH
    - POST /api/decrypt sin field `bed` → 400 BAD_REQUEST
    - Tracing: ningún log line contiene el descriptor cleartext
  </behavior>
  <action>
    Crear `crates/server/src/routes/encrypt.rs`:
    ```rust
    //! POST /api/encrypt — JSON in, JSON out (D-05). Three outputs in one response:
    //! bed_b64 (base64 of binary), armored (PEM string), qr_png_b64 (base64 of PNG).

    use axum::{extract::Json, response::IntoResponse};
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde::{Deserialize, Serialize};
    use zeroize::{Zeroize, Zeroizing};

    use crate::AppError;

    #[derive(Deserialize)]
    pub struct EncryptRequest {
        pub descriptor: String,
    }

    #[derive(Serialize)]
    pub struct EncryptResponse {
        pub bed_b64: String,
        pub armored: String,
        pub qr_png_b64: String,
    }

    /// Encrypt handler. The `descriptor` field is moved into `Zeroizing<String>`
    /// on the FIRST line (D-10, PITFALLS #4) before any `?` early-return.
    #[tracing::instrument(skip_all)]
    pub async fn post_encrypt(
        Json(req): Json<EncryptRequest>,
    ) -> Result<Json<EncryptResponse>, AppError> {
        // STEP 1 (D-10): wrap immediately; req.descriptor is moved INTO Zeroizing
        // on this line. Any subsequent access is via &mut, never by value.
        let mut cleartext: Zeroizing<String> = Zeroizing::new(req.descriptor);

        let out = bed_core::encrypt_descriptor(&mut cleartext)?;

        // Defense-in-depth: explicit zeroize + drop before serialization.
        cleartext.zeroize();
        drop(cleartext);

        let response = EncryptResponse {
            bed_b64: STANDARD.encode(&out.bed_bytes),
            armored: out.armored,
            qr_png_b64: STANDARD.encode(&out.qr_png),
        };
        Ok(Json(response))
    }
    ```

    Crear `crates/server/src/routes/decrypt.rs` con el contenido de RESEARCH.md §"Pattern 6: Multipart for decrypt":
    ```rust
    //! POST /api/decrypt — multipart in, JSON out (D-06).
    //!
    //! Fields:
    //!   bed:  text armored OR binary .bed file (auto-detected by leading bytes)
    //!   xpub: text xpub OR file containing xpub (multipart treats both as bytes)
    //!
    //! NOTE: ENC-02/ENC-05/DEC-02/DEC-03 are API substrate only — UI presentation
    //! is Phase 2. This handler accepts both pasted-text and uploaded-file via the
    //! same multipart contract.

    use axum::{extract::Multipart, Json};
    use serde::Serialize;
    use zeroize::{Zeroize, Zeroizing};

    use crate::AppError;

    #[derive(Serialize)]
    pub struct DecryptResponse {
        pub descriptor: String,
    }

    #[tracing::instrument(skip_all)]
    pub async fn post_decrypt(mut form: Multipart) -> Result<Json<DecryptResponse>, AppError> {
        let mut bed: Option<Vec<u8>> = None;
        let mut xpub: Option<String> = None;

        while let Some(field) = form
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            let name = field.name().unwrap_or("").to_string();
            match name.as_str() {
                "bed" => {
                    let raw = field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?;
                    bed = Some(raw.to_vec());
                }
                "xpub" => {
                    let text = field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?;
                    xpub = Some(text.trim().to_string());
                }
                _ => { /* ignore unknown fields */ }
            }
        }

        let bed_bytes =
            bed.ok_or_else(|| AppError::BadRequest("missing 'bed' field".to_string()))?;
        let xpub_str =
            xpub.ok_or_else(|| AppError::BadRequest("missing 'xpub' field".to_string()))?;

        // Auto-detect armored: if bytes start with "-----BEGIN", strip PEM headers
        // via decode_armored. Otherwise pass raw bytes to crate (which auto-detects
        // binary "BIPXXX" magic vs raw base64).
        let payload: Vec<u8> = if bed_bytes.starts_with(b"-----BEGIN") {
            let text = std::str::from_utf8(&bed_bytes)
                .map_err(|_| AppError::BadRequest("invalid utf-8 in armored".to_string()))?;
            bed_core::decode_armored(text)
                .map_err(|e| AppError::BadRequest(format!("armored: {e}")))?
        } else {
            bed_bytes
        };

        let mut cleartext: Zeroizing<String> = bed_core::decrypt_payload(&payload, &xpub_str)?;

        // Snapshot the cleartext to a String for JSON serialization. This is the
        // documented residual exposure (RESEARCH.md note in §Pattern 6): once it
        // crosses the JSON boundary it cannot be zeroized in serde's intermediate
        // buffer. We zeroize the source Zeroizing immediately after the clone.
        let descriptor = cleartext.as_str().to_string();
        cleartext.zeroize();
        drop(cleartext);

        Ok(Json(DecryptResponse { descriptor }))
    }
    ```

    Actualizar `crates/server/src/lib.rs` (reemplaza stubs de Plan 01):
    ```rust
    //! bed-server — axum HTTP layer for bed-core.

    use axum::{routing::post, Router};

    pub mod error;
    pub mod routes;
    pub mod state;

    pub use error::AppError;

    /// Build the router. Used by main.rs (binds socket) and integration tests
    /// (oneshot via tower::ServiceExt — D-23).
    pub fn router() -> Router {
        Router::new()
            .route("/api/encrypt", post(routes::encrypt::post_encrypt))
            .route("/api/decrypt", post(routes::decrypt::post_decrypt))
            .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
        // No TraceLayer in Phase 1: there are no non-sensitive routes (D-19).
        // Phase 2 will add it on history endpoints.
    }
    ```

    NOTA al executor: Plan 01 dejó `encrypt_stub` y `decrypt_stub` async functions en `lib.rs` — eliminarlos. El nuevo `lib.rs` no los referencia.
  </action>
  <verify>
    <automated>cargo build -p bed-server 2>&1 | tail -5 && cargo clippy -p bed-server --all-targets -- -D warnings 2>&1 | tail -5</automated>
  </verify>
  <acceptance_criteria>
    - `cargo build -p bed-server` exits 0
    - `cargo clippy -p bed-server --all-targets -- -D warnings` exits 0
    - `crates/server/src/routes/encrypt.rs` contiene literal `#[tracing::instrument(skip_all)]`
    - `crates/server/src/routes/decrypt.rs` contiene literal `#[tracing::instrument(skip_all)]`
    - `crates/server/src/routes/encrypt.rs` PRIMERA línea de función contiene `Zeroizing::new(req.descriptor)` (verificable con `grep -A2 'async fn post_encrypt' crates/server/src/routes/encrypt.rs | grep 'Zeroizing::new'`)
    - `crates/server/src/routes/decrypt.rs` contiene literal `decode_armored` y `decrypt_payload`
    - `crates/server/src/lib.rs` contiene literal `.route("/api/encrypt", post(`
    - `crates/server/src/lib.rs` contiene literal `.route("/api/decrypt", post(`
    - `crates/server/src/lib.rs` contiene literal `DefaultBodyLimit::max(512 * 1024)`
    - `grep -E '\.unwrap\(\)|\.expect\(' crates/server/src/ -r | grep -vE 'unwrap_or|tests/' | wc -l` == 0
    - `cargo run -p bed-server &` arranca y responde a `curl -s 127.0.0.1:8080/api/encrypt -X POST -H 'content-type: application/json' -d '{"descriptor":"invalid"}'` con HTTP 422 (probar manualmente o en SUMMARY)
  </acceptance_criteria>
  <done>Server crate completo: handlers reales, error mapping, router con /api/encrypt + /api/decrypt; sin unwrap en path de request; tracing skip_all aplicado.</done>
</task>

</tasks>

<verification>
- `cargo build -p bed-server` exits 0
- `cargo clippy -p bed-server --all-targets -- -D warnings` exits 0
- Sin `unwrap()`/`expect()` en `crates/server/src/` (excepto tests, que están en `crates/server/tests/`)
- Handlers tienen `#[tracing::instrument(skip_all)]`
- Zeroizing aplicado en la primera línea del handler encrypt
- Bind a 127.0.0.1:8080 (constante de Plan 01, sin cambios aquí)
</verification>

<success_criteria>
- ENC-01: POST /api/encrypt JSON con tres outputs
- ENC-03: armored field con headers exactos (vía bed_core)
- ENC-04: qr_png_b64 field; QR_TOO_LARGE error tipado si excede
- ENC-05 (API substrate only): error JSON estructurado con `code` UPPER_SNAKE
- DEC-01: POST /api/decrypt multipart funcional
- DEC-02 + DEC-03 (API substrate only): bed acepta texto armored o binario; xpub acepta texto o file
- DEC-04: descriptor returned vía JSON, nunca persistido a disco
- SEC-01: handlers tienen `skip_all`; no body-logging middleware
- CORE-04: Zeroizing wrap at parse boundary
- CORE-05: sin unwrap() en `crates/server/src/`
</success_criteria>

<output>
Tras completar, crear `.planning/phases/01-crypto-core-http-api/01-05-server-axum-handlers-SUMMARY.md` documentando:
- Endpoints registrados con sus métodos
- Resultado de un curl real al server local: encrypt OK + decrypt OK + descriptor inválido → 422
- Output de `cargo clippy --workspace -- -D warnings`
- Confirmación de zeroize en primera línea del handler encrypt (línea exacta)
</output>
