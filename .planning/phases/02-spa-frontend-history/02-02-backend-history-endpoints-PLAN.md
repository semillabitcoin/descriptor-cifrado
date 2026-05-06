---
phase: 02-spa-frontend-history
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/server/Cargo.toml
  - crates/server/src/error.rs
  - crates/server/src/state.rs
  - crates/server/src/lib.rs
  - crates/server/src/routes/mod.rs
  - crates/server/src/routes/history.rs
  - crates/server/tests/history_round_trip.rs
  - crates/server/tests/history_no_leak.rs
autonomous: true
requirements: [HIST-02, HIST-03, HIST-04, HIST-05]
must_haves:
  truths:
    - "POST /api/history con bed_b64 válido escribe un archivo en BED_DATA_DIR y retorna {id, timestamp, filename}"
    - "GET /api/history lista entradas via directory scan, ordenadas timestamp desc"
    - "GET /api/history/:id regenera bed_b64 + armored + qr_png_b64 desde el .bed persistido"
    - "DELETE /api/history/:id elimina el archivo y retorna 204; 404 si no existe; 422 si id inválido"
    - "El descriptor en claro NUNCA aparece en archivos bajo BED_DATA_DIR (test grep en CI)"
    - "id con caracteres no-hex o con '..' devuelve 422 HISTORY_INVALID_ID (anti path traversal)"
  artifacts:
    - path: "crates/server/src/routes/history.rs"
      provides: "Cuatro handlers axum: post_history, get_history, get_history_id, delete_history"
      exports: ["post_history", "get_history", "get_history_id", "delete_history"]
    - path: "crates/server/src/state.rs"
      provides: "data_dir() function que resuelve BED_DATA_DIR env var"
      contains: "BED_DATA_DIR"
    - path: "crates/server/src/error.rs"
      provides: "AppError variantes nuevas: HistoryNotFound (404), HistoryWriteFailed (500), HistoryInvalidId (422)"
      contains: "HistoryNotFound"
    - path: "crates/server/tests/history_round_trip.rs"
      provides: "Test integration POST → GET list → GET :id → DELETE round trip"
    - path: "crates/server/tests/history_no_leak.rs"
      provides: "Test que descriptor cleartext no aparece en archivos persistidos (HIST-03)"
  key_links:
    - from: "crates/server/src/lib.rs"
      to: "crates/server/src/routes/history.rs"
      via: "Router::new().route(\"/api/history\", post(...).get(...))"
      pattern: "/api/history"
    - from: "crates/server/src/routes/history.rs"
      to: "crates/server/src/state.rs"
      via: "state::data_dir()"
      pattern: "data_dir\\(\\)"
    - from: "crates/server/src/routes/history.rs"
      to: "crates/core/src/armored.rs (encode_armored)"
      via: "bed_core::encode_armored para regenerar armored desde bytes"
      pattern: "encode_armored"
    - from: "crates/server/src/routes/history.rs"
      to: "crates/core/src/qr.rs (render_qr_png)"
      via: "bed_core::render_qr_png para regenerar QR desde armored"
      pattern: "render_qr_png"
---

<objective>
Añadir los cuatro endpoints HTTP del historial al backend axum existente: `POST /api/history`, `GET /api/history`, `GET /api/history/:id`, `DELETE /api/history/:id`. La persistencia es directory scan de `BED_DATA_DIR` (default `/data/encrypted/`) — sin redb, sin DB. El descriptor en claro NUNCA toca el disco (HIST-03 enforced por design: el endpoint solo acepta `bed_b64` ya cifrado, nunca el descriptor). Anti path traversal por validación estricta del id (`[a-z0-9]{8}`).

Purpose: Cubrir HIST-02 (persistencia opt-in del .bed), HIST-04 (listado) y HIST-05 (delete) — los requisitos backend del modo histórico, ortogonales al frontend que los consume en plan 02-06.
Output: 4 handlers funcionales registrados en el router, integration tests verificando round-trip y no-leak, AppError extendido con tres variantes nuevas, y `BED_DATA_DIR` env var resoluble.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/02-spa-frontend-history/02-CONTEXT.md
@.planning/phases/02-spa-frontend-history/02-RESEARCH.md
@.planning/PROJECT.md
@CLAUDE.md
@Cargo.toml
@crates/server/Cargo.toml
@crates/server/src/lib.rs
@crates/server/src/error.rs
@crates/server/src/state.rs
@crates/server/src/routes/encrypt.rs
@crates/server/src/routes/decrypt.rs
@crates/core/src/lib.rs
@crates/core/src/armored.rs
@crates/core/src/qr.rs

<interfaces>
<!-- Existing exports the executor will consume. Extracted from crates/core/src/lib.rs. -->

From crates/core/src/lib.rs:
```rust
pub use armored::{decode_armored, encode_armored, ArmoredError, ARMOR_BEGIN, ARMOR_END};
pub use qr::{render_qr_png, MAX_QR_BYTES};
pub use error::CoreError;
```

From crates/server/src/error.rs (current — to be extended):
```rust
pub enum AppError {
    MissingMultipathWildcard,
    DescriptorParse,
    XpubMismatch,
    QrTooLarge { size: usize, max: usize },
    Internal,
    BadRequest(String),
}
// Body shape: {"error": {"code": "<UPPER_SNAKE>", "message": "<castellano>"}}
```

From crates/server/src/lib.rs (current — to be extended):
```rust
pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
}
```

axum 0.8 path param syntax: use `{id}` not `:id` in route patterns (axum 0.8 changed from colon-prefix to brace syntax). Confirm with axum 0.8 docs at compile time. Routes: `/api/history/{id}`.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Añadir AppError variantes + data_dir() helper + dependencias workspace</name>
  <files>Cargo.toml, crates/server/Cargo.toml, crates/server/src/error.rs, crates/server/src/state.rs</files>
  <read_first>
    - Cargo.toml (workspace.dependencies actual)
    - crates/server/Cargo.toml (deps actuales del crate)
    - crates/server/src/error.rs (estructura AppError + IntoResponse + From&lt;CoreError&gt;)
    - crates/server/src/state.rs (placeholder actual, debe extenderse SIN romper signature)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 4 — data_dir, Patrón 10 — AppError variantes)
  </read_first>
  <behavior>
    - `state::data_dir()` retorna `PathBuf` desde env var `BED_DATA_DIR`, default `/data/encrypted`
    - `state::data_dir()` con `BED_DATA_DIR=/tmp/foo` retorna `PathBuf::from("/tmp/foo")`
    - `state::validate_history_id("a3f7b2c1")` retorna true (8 chars hex lowercase)
    - `state::validate_history_id("a3f7b2c")` retorna false (7 chars)
    - `state::validate_history_id("a3f7b2c1x")` retorna false (9 chars)
    - `state::validate_history_id("A3F7B2C1")` retorna false (uppercase)
    - `state::validate_history_id("../etc/p")` retorna false (no hex)
    - `state::validate_history_id("a3f7b2g1")` retorna false (g no es hex)
    - `AppError::HistoryNotFound.into_response()` retorna status 404 con body code "HISTORY_NOT_FOUND"
    - `AppError::HistoryWriteFailed.into_response()` retorna status 500 con code "HISTORY_WRITE_FAILED"
    - `AppError::HistoryInvalidId.into_response()` retorna status 422 con code "HISTORY_INVALID_ID"
  </behavior>
  <action>
1. **Workspace `Cargo.toml`** — añadir a `[workspace.dependencies]` (mantén el orden alfabético existente cuando aplique):

```toml
rust-embed = { version = "8", features = ["axum"] }
uuid = { version = "1", features = ["v4"] }
time = { version = "0.3", default-features = false, features = ["formatting", "macros"] }
```

Notas:
- Feature flag exacto de rust-embed para axum 0.8 puede ser `"axum"` o `"axum-ex"`. Verifica antes de implementar consultando `cargo tree -p rust-embed -e features --no-default-features --features axum 2>/dev/null` después de añadir; si falla, usa `axum-ex`. La fuente del RESEARCH dice `"axum-ex"` pero rust-embed 8.5+ unificó a `"axum"`. Si ambos fallan al compilar, usa `cargo doc -p rust-embed --no-deps --open` para inspeccionar.
- `time` se prefiere sobre `chrono` (sin C deps, más liviano para timestamps ISO).

2. **`crates/server/Cargo.toml`** — añadir bajo `[dependencies]`:

```toml
rust-embed.workspace = true
uuid.workspace = true
time.workspace = true
```

Y EXTENDER el feature set de `tokio` para añadir `fs`:

```toml
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "io-util", "net", "signal", "fs"] }
```

Y añadir `tempfile` a `[dev-dependencies]`:

```toml
tempfile = "3"
```

3. **`crates/server/src/state.rs`** — REEMPLAZA el contenido completo (el placeholder actual es trivial; mantener struct AppState vacía para compatibilidad de signature):

```rust
//! AppState + helpers de configuración runtime para Phase 2 history endpoints.
//!
//! `data_dir()` resuelve el directorio de persistencia del historial desde
//! la env var `BED_DATA_DIR` (default `/data/encrypted`). Permite que los
//! integration tests usen `tempfile::tempdir()` sin colisionar con el path
//! productivo (Trampa 7 del RESEARCH).
//!
//! `validate_history_id()` enforces anti-path-traversal (D-29): id debe ser
//! exactamente 8 caracteres hex lowercase `[a-z0-9]{8}`.

use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct AppState;

/// Resuelve el directorio donde se persisten los `.bed` del historial.
///
/// Default: `/data/encrypted` (StartOS volume `main`).
/// Override: env var `BED_DATA_DIR` (usado en dev y tests).
pub fn data_dir() -> PathBuf {
    std::env::var("BED_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/encrypted"))
}

/// Valida un id de historial: exactamente 8 caracteres hex lowercase.
/// Cualquier otro patrón (uppercase, longitud distinta, caracteres no-hex,
/// path traversal `../`, etc.) retorna false.
pub fn validate_history_id(id: &str) -> bool {
    id.len() == 8
        && id
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    #[test]
    fn validate_history_id_accepts_8_hex_lowercase() {
        assert!(validate_history_id("a3f7b2c1"));
        assert!(validate_history_id("00000000"));
        assert!(validate_history_id("ffffffff"));
        assert!(validate_history_id("0123abcd"));
    }

    #[test]
    fn validate_history_id_rejects_uppercase() {
        assert!(!validate_history_id("A3F7B2C1"));
        assert!(!validate_history_id("a3F7b2c1"));
    }

    #[test]
    fn validate_history_id_rejects_wrong_length() {
        assert!(!validate_history_id(""));
        assert!(!validate_history_id("a3f7b2c"));     // 7
        assert!(!validate_history_id("a3f7b2c1x"));   // 9
        assert!(!validate_history_id("a3f7b2c1a3f7b2c1")); // 16
    }

    #[test]
    fn validate_history_id_rejects_non_hex() {
        assert!(!validate_history_id("a3f7b2g1"));    // g no es hex
        assert!(!validate_history_id("a3f7-2c1"));    // guión
        assert!(!validate_history_id("../etc/p"));    // path traversal
        assert!(!validate_history_id("a3f7 2c1"));    // espacio
    }
}
```

4. **`crates/server/src/error.rs`** — AÑADIR tres variantes al enum (mantén las cinco existentes intactas), AÑADIR sus matches en `IntoResponse`:

Editar el `enum AppError`:

```rust
    #[error("Entrada de historial no encontrada.")]
    HistoryNotFound,

    #[error("No se pudo escribir en el historial.")]
    HistoryWriteFailed,

    #[error("ID de historial inválido.")]
    HistoryInvalidId,
```

Editar el match de `IntoResponse for AppError`:

```rust
            AppError::HistoryNotFound => (StatusCode::NOT_FOUND, "HISTORY_NOT_FOUND"),
            AppError::HistoryWriteFailed => (StatusCode::INTERNAL_SERVER_ERROR, "HISTORY_WRITE_FAILED"),
            AppError::HistoryInvalidId => (StatusCode::UNPROCESSABLE_ENTITY, "HISTORY_INVALID_ID"),
```

5. Verificar compilación: `cargo build -p bed-server`. Verificar tests del módulo state: `cargo test -p bed-server state::tests --lib`.

NO añadir telemetry / logging que filtre nombres de archivos del historial (los nombres son safe — solo timestamp + short-id, sin descriptor). Pero NO loguear bodies (el `bed_b64` en POST /api/history es opaco pero por consistencia con SEC-01: `#[tracing::instrument(skip_all)]` en todos los handlers).
NO usar `unwrap()` ni `expect()` en código no-test (workspace lints lo prohíben).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado &amp;&amp; cargo build -p bed-server 2>&amp;1 | tail -5 &amp;&amp; cargo test -p bed-server state --lib 2>&amp;1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `grep "rust-embed" /workspace/descriptor-cifrado/Cargo.toml` encuentra match en workspace.dependencies
    - `grep '"v4"' /workspace/descriptor-cifrado/Cargo.toml` encuentra match (uuid feature)
    - `grep "uuid.workspace = true" /workspace/descriptor-cifrado/crates/server/Cargo.toml` encuentra match
    - `grep "rust-embed.workspace = true" /workspace/descriptor-cifrado/crates/server/Cargo.toml` encuentra match
    - `grep '"fs"' /workspace/descriptor-cifrado/crates/server/Cargo.toml` encuentra match (tokio fs feature)
    - `grep "tempfile" /workspace/descriptor-cifrado/crates/server/Cargo.toml` encuentra match en dev-dependencies
    - `grep "HistoryNotFound" /workspace/descriptor-cifrado/crates/server/src/error.rs` encuentra match
    - `grep "HistoryWriteFailed" /workspace/descriptor-cifrado/crates/server/src/error.rs` encuentra match
    - `grep "HistoryInvalidId" /workspace/descriptor-cifrado/crates/server/src/error.rs` encuentra match
    - `grep "HISTORY_NOT_FOUND" /workspace/descriptor-cifrado/crates/server/src/error.rs` encuentra match
    - `grep "HISTORY_WRITE_FAILED" /workspace/descriptor-cifrado/crates/server/src/error.rs` encuentra match
    - `grep "HISTORY_INVALID_ID" /workspace/descriptor-cifrado/crates/server/src/error.rs` encuentra match
    - `grep "BED_DATA_DIR" /workspace/descriptor-cifrado/crates/server/src/state.rs` encuentra match
    - `grep "pub fn data_dir" /workspace/descriptor-cifrado/crates/server/src/state.rs` encuentra match
    - `grep "pub fn validate_history_id" /workspace/descriptor-cifrado/crates/server/src/state.rs` encuentra match
    - `cd /workspace/descriptor-cifrado && cargo build -p bed-server` exit code 0
    - `cd /workspace/descriptor-cifrado && cargo test -p bed-server state --lib` exit code 0 con mínimo 4 tests pasando
  </acceptance_criteria>
  <done>workspace.dependencies declara rust-embed, uuid, time; bed-server depende de ellos + tokio fs feature + tempfile dev-dep; AppError tiene 3 variantes nuevas con sus status codes; state::data_dir() y state::validate_history_id() implementadas y testeadas; cargo build verde.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implementar 4 handlers history.rs (POST, GET list, GET :id, DELETE :id)</name>
  <files>crates/server/src/routes/mod.rs, crates/server/src/routes/history.rs, crates/server/src/lib.rs</files>
  <read_first>
    - crates/server/src/routes/encrypt.rs (patrón handler axum + response shape)
    - crates/server/src/routes/decrypt.rs (multipart pattern y response shape de descifrado)
    - crates/server/src/lib.rs (forma actual del router — debes añadir 4 routes)
    - crates/server/src/error.rs (AppError + From&lt;CoreError&gt;)
    - crates/server/src/state.rs (data_dir, validate_history_id que acabas de crear)
    - crates/core/src/armored.rs (signature de encode_armored — debes invocar con bytes)
    - crates/core/src/qr.rs (signature de render_qr_png — debes invocar con armored)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 4 — directory scan, formato filename)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-26, D-27, D-28, D-29 — contracts exactos)
  </read_first>
  <behavior>
    - **POST /api/history** con body `{"bed_b64": "<valid-base64-of-.bed-bytes>"}`:
      - Decodifica base64; si falla → 422 BadRequest "bed_b64 no es base64 válido"
      - Genera id = primeros 8 chars hex de `Uuid::new_v4().simple().to_string()`
      - Genera timestamp_compact: `format!("{}T{}Z", date YYYYMMDD, time HHMMSS)` UTC (formato sortable, sin guiones ni colons)
      - Genera timestamp_iso: `2026-05-06T11:55:37Z` (ISO-8601 UTC, para response)
      - Escribe bytes en `data_dir().join(format!("{compact}-{id}.bed"))`
      - Si write falla → 500 HistoryWriteFailed
      - Retorna 200 JSON: `{"id": "<id>", "timestamp": "<iso>", "filename": "<compact>-<id>.bed"}`
    - **GET /api/history**: directory scan de `data_dir()`, parsea filenames `<compact>-<id>.bed`, retorna `{"entries": [{id, timestamp_iso, filename, size_bytes}, ...]}` ordenado timestamp desc. Vacío → `{"entries": []}`. Si data_dir no existe → `{"entries": []}` (no es error — modo histórico nunca usado).
    - **GET /api/history/{id}**: 422 si !validate_history_id(id); busca archivo `*-{id}.bed` en data_dir; 404 si no existe; lee bytes; retorna `{"bed_b64": "...", "armored": "...", "qr_png_b64": "..."}` (mismo shape que /api/encrypt). Si encode_armored o render_qr_png fallan → 500.
    - **DELETE /api/history/{id}**: 422 si !validate_history_id(id); 404 si no existe; remove_file; 204 No Content.
    - **HIST-03 enforced by design**: ningún endpoint acepta el descriptor cleartext. POST solo recibe `bed_b64` ya cifrado. Por lo tanto el cleartext jamás cruza este módulo.
  </behavior>
  <action>
1. **`crates/server/src/routes/mod.rs`** — añadir `pub mod history;` (mantén `pub mod encrypt;` y `pub mod decrypt;` que ya existen).

2. **`crates/server/src/routes/history.rs`** — crear con esta estructura. Implementa los 4 handlers + helpers locales. **NO uses unwrap/expect** (workspace lint lo deniega).

```rust
//! Handlers para el modo histórico opt-in (Phase 2).
//!
//! Endpoints (D-26..D-29):
//!   POST   /api/history          — persiste un .bed
//!   GET    /api/history          — lista entradas (directory scan)
//!   GET    /api/history/{id}     — regenera bed/armored/qr desde el .bed persistido
//!   DELETE /api/history/{id}     — borra una entrada
//!
//! HIST-03 (no leak): el endpoint POST acepta SOLO `bed_b64` ya cifrado, jamás
//! el descriptor en claro. El módulo no tiene ninguna ruta de código que
//! reciba o escriba descriptors cleartext.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};
use tokio::fs;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::{data_dir, validate_history_id},
};

// === Request/Response shapes ===

#[derive(Deserialize)]
pub struct PostHistoryRequest {
    pub bed_b64: String,
}

#[derive(Serialize)]
pub struct PostHistoryResponse {
    pub id: String,
    pub timestamp: String,
    pub filename: String,
}

#[derive(Serialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub filename: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
pub struct ListHistoryResponse {
    pub entries: Vec<HistoryEntry>,
}

#[derive(Serialize)]
pub struct GetHistoryIdResponse {
    pub bed_b64: String,
    pub armored: String,
    pub qr_png_b64: String,
}

// === Filename format helpers ===

const FILENAME_COMPACT: &[FormatItem<'_>] =
    format_description!("[year][month][day]T[hour][minute][second]Z");
const FILENAME_ISO: &[FormatItem<'_>] = format_description!(
    "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
);

fn now_compact() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(FILENAME_COMPACT)
        .map_err(|_| AppError::Internal)
}

fn now_iso() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(FILENAME_ISO)
        .map_err(|_| AppError::Internal)
}

/// Parsea `20260506T115537Z-a3f7b2c1.bed` → (timestamp_iso, id).
/// Retorna None si el filename no matchea el formato esperado.
fn parse_filename(name: &str) -> Option<(String, String)> {
    // Formato: 20260506T115537Z-XXXXXXXX.bed (16 chars timestamp + "-" + 8 hex + ".bed")
    let stripped = name.strip_suffix(".bed")?;
    let (compact, dash_id) = stripped.split_at(stripped.len().checked_sub(9)?);
    if !dash_id.starts_with('-') {
        return None;
    }
    let id = &dash_id[1..];
    if !validate_history_id(id) {
        return None;
    }
    if compact.len() != 16 {
        return None;
    }
    // Convertir 20260506T115537Z → 2026-05-06T11:55:37Z
    if compact.as_bytes().get(8) != Some(&b'T')
        || compact.as_bytes().last() != Some(&b'Z')
    {
        return None;
    }
    let date = &compact[0..8];
    let time = &compact[9..15];
    // YYYYMMDD → YYYY-MM-DD
    let iso = format!(
        "{}-{}-{}T{}:{}:{}Z",
        &date[0..4],
        &date[4..6],
        &date[6..8],
        &time[0..2],
        &time[2..4],
        &time[4..6],
    );
    Some((iso, id.to_string()))
}

fn make_filename(compact: &str, id: &str) -> String {
    format!("{compact}-{id}.bed")
}

fn full_path(filename: &str) -> PathBuf {
    data_dir().join(filename)
}

async fn find_file_by_id(id: &str) -> Result<Option<PathBuf>, AppError> {
    let dir = data_dir();
    let mut rd = match fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(AppError::Internal),
    };
    let suffix = format!("-{id}.bed");
    while let Some(entry) = rd.next_entry().await.map_err(|_| AppError::Internal)? {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        if s.ends_with(&suffix) && parse_filename(s).is_some() {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

// === Handlers ===

/// `POST /api/history` — persiste un .bed cifrado.
#[tracing::instrument(skip_all)]
pub async fn post_history(
    Json(req): Json<PostHistoryRequest>,
) -> Result<Json<PostHistoryResponse>, AppError> {
    let bytes = B64
        .decode(req.bed_b64.as_bytes())
        .map_err(|_| AppError::BadRequest("bed_b64 no es base64 válido".to_string()))?;
    if bytes.is_empty() {
        return Err(AppError::BadRequest(
            "bed_b64 está vacío".to_string(),
        ));
    }
    let id: String = Uuid::new_v4().simple().to_string()[..8].to_string();
    let compact = now_compact()?;
    let iso = now_iso()?;
    let filename = make_filename(&compact, &id);
    let path = full_path(&filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|_| AppError::HistoryWriteFailed)?;
    }
    fs::write(&path, &bytes)
        .await
        .map_err(|_| AppError::HistoryWriteFailed)?;
    Ok(Json(PostHistoryResponse {
        id,
        timestamp: iso,
        filename,
    }))
}

/// `GET /api/history` — lista entradas via directory scan.
#[tracing::instrument(skip_all)]
pub async fn get_history() -> Result<Json<ListHistoryResponse>, AppError> {
    let dir = data_dir();
    let mut rd = match fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Json(ListHistoryResponse { entries: vec![] }));
        }
        Err(_) => return Err(AppError::Internal),
    };
    let mut entries = Vec::new();
    while let Some(entry) = rd.next_entry().await.map_err(|_| AppError::Internal)? {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        let Some((timestamp, id)) = parse_filename(s) else {
            continue;
        };
        let size_bytes = entry
            .metadata()
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        entries.push(HistoryEntry {
            id,
            timestamp,
            filename: s.to_string(),
            size_bytes,
        });
    }
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(Json(ListHistoryResponse { entries }))
}

/// `GET /api/history/{id}` — regenera bed_b64 + armored + qr_png_b64.
#[tracing::instrument(skip_all)]
pub async fn get_history_id(
    Path(id): Path<String>,
) -> Result<Json<GetHistoryIdResponse>, AppError> {
    if !validate_history_id(&id) {
        return Err(AppError::HistoryInvalidId);
    }
    let path = find_file_by_id(&id).await?.ok_or(AppError::HistoryNotFound)?;
    let bytes = fs::read(&path).await.map_err(|_| AppError::Internal)?;
    let bed_b64 = B64.encode(&bytes);
    let armored = bed_core::encode_armored(&bytes);
    let qr_png = bed_core::render_qr_png(armored.as_bytes())?;
    let qr_png_b64 = B64.encode(&qr_png);
    Ok(Json(GetHistoryIdResponse {
        bed_b64,
        armored,
        qr_png_b64,
    }))
}

/// `DELETE /api/history/{id}` — borra una entrada.
#[tracing::instrument(skip_all)]
pub async fn delete_history(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if !validate_history_id(&id) {
        return Err(AppError::HistoryInvalidId);
    }
    let path = find_file_by_id(&id).await?.ok_or(AppError::HistoryNotFound)?;
    fs::remove_file(&path)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    #[test]
    fn parse_filename_round_trip() {
        let id = "a3f7b2c1";
        let compact = "20260506T115537Z";
        let name = make_filename(compact, id);
        assert_eq!(name, "20260506T115537Z-a3f7b2c1.bed");
        let (iso, parsed_id) = parse_filename(&name).unwrap();
        assert_eq!(iso, "2026-05-06T11:55:37Z");
        assert_eq!(parsed_id, id);
    }

    #[test]
    fn parse_filename_rejects_bad_format() {
        assert!(parse_filename("not-a-bed-file.txt").is_none());
        assert!(parse_filename("garbage.bed").is_none());
        assert!(parse_filename("20260506T115537Z-BADID0.bed").is_none()); // BADID0 too short
        assert!(parse_filename("20260506T115537Z-A3F7B2C1.bed").is_none()); // uppercase
    }
}
```

CRITICAL — verifica antes de compilar:
- La signature exacta de `bed_core::encode_armored`. El RESEARCH dice "encode_armored(&bytes)" pero confirma con `crates/core/src/armored.rs`. Si la firma es `encode_armored(bytes: &[u8]) -> String`, la llamada anterior es correcta. Si retorna `Result<String, _>`, ajusta con `?` y el `From<CoreError> for AppError` ya existente convertirá errores.
- La signature exacta de `bed_core::render_qr_png`. Verifica que recibe `&[u8]` y retorna `Result<Vec<u8>, CoreError>`. Si recibe `&str` ajusta. El RESEARCH usa `render_qr_png(&armored)` mientras yo escribí `armored.as_bytes()` — usa lo que matche el código actual.

3. **`crates/server/src/lib.rs`** — extender el router. Reemplaza el `pub fn router()` con:

```rust
//! bed-server — axum HTTP layer for bed-core.

use axum::{
    routing::{delete, get, post},
    Router,
};

pub mod error;
pub mod routes;
pub mod state;

pub use error::AppError;

/// Build the router. Used by main.rs (binds socket) and integration tests
/// (oneshot via tower::ServiceExt — D-23).
///
/// Phase 2: añade rutas de historial. NOTA: las rutas de assets (rust-embed)
/// se añaden en el plan 02-06 al integrar el frontend embebido.
pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        .route("/api/history", post(routes::history::post_history))
        .route("/api/history", get(routes::history::get_history))
        .route("/api/history/{id}", get(routes::history::get_history_id))
        .route("/api/history/{id}", delete(routes::history::delete_history))
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
}
```

NOTA axum 0.8: el path syntax es `/api/history/{id}` (braces), NO `/api/history/:id` (colons — sintaxis axum 0.7). Verifica que la versión instalada en Cargo.lock es 0.8.x antes de elegir; el workspace declara `axum = "0.8"`.

4. Compila y corre los unit tests internos: `cargo build -p bed-server && cargo test -p bed-server routes::history --lib`.

NO usar `unwrap()` en código no-test (workspace lint deny). NO loguear el body (skip_all). NO emitir el descriptor por ningún path — POST solo recibe `bed_b64`. NO permitir que id contenga path traversal — la validación de validate_history_id es el único guard.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado &amp;&amp; cargo build -p bed-server 2>&amp;1 | tail -5 &amp;&amp; cargo test -p bed-server routes::history --lib 2>&amp;1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `grep "pub mod history" /workspace/descriptor-cifrado/crates/server/src/routes/mod.rs` encuentra match
    - `grep "pub async fn post_history" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra match
    - `grep "pub async fn get_history" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra match
    - `grep "pub async fn get_history_id" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra match
    - `grep "pub async fn delete_history" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra match
    - `grep "tracing::instrument(skip_all)" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra al menos 4 matches (uno por handler — SEC-01)
    - `grep "validate_history_id" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra al menos 2 matches (get_id + delete_id)
    - `grep "encode_armored" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra match
    - `grep "render_qr_png" /workspace/descriptor-cifrado/crates/server/src/routes/history.rs` encuentra match
    - `grep "/api/history" /workspace/descriptor-cifrado/crates/server/src/lib.rs` encuentra al menos 4 matches (POST+GET list + GET id + DELETE id)
    - `grep "{id}" /workspace/descriptor-cifrado/crates/server/src/lib.rs` encuentra match (axum 0.8 syntax)
    - `! grep "/api/history/:id" /workspace/descriptor-cifrado/crates/server/src/lib.rs` (no usa axum 0.7 syntax)
    - `cd /workspace/descriptor-cifrado && cargo build -p bed-server` exit code 0
    - `cd /workspace/descriptor-cifrado && cargo test -p bed-server routes::history --lib` exit code 0
  </acceptance_criteria>
  <done>4 handlers axum compilados y registrados en el router; unit tests del módulo (parse_filename round-trip + rechazos de formato malo) verdes.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Integration tests — round-trip POST/GET/DELETE + no-leak HIST-03</name>
  <files>crates/server/tests/history_round_trip.rs, crates/server/tests/history_no_leak.rs</files>
  <read_first>
    - crates/server/tests (cualquier integration test ya existente de Phase 1 — replicar el patrón oneshot)
    - crates/server/src/routes/encrypt.rs (para conocer cómo invocar POST /api/encrypt en el test no-leak — el test usa el flujo real)
    - crates/server/src/routes/history.rs (handlers que se testean)
    - crates/server/src/state.rs (env var BED_DATA_DIR — los tests la setean a tempdir)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Trampa 7 — env var en tests)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (HIST-03 test extendido a frontend → este test es backend pero refuerza HIST-03)
  </read_first>
  <behavior>
    **`history_round_trip.rs`:**
    - test_post_then_list_then_get_then_delete:
      - Setea BED_DATA_DIR a tempfile::tempdir()
      - POST /api/history con bed_b64 = base64(b"FAKE_BED_BYTES_NOT_ENCRYPTED_DESCRIPTOR")
      - Asserta status 200, response tiene id (8 chars hex), timestamp (formato ISO), filename
      - GET /api/history asserta entries.len() == 1 con id matching
      - GET /api/history/{id} asserta status 200, response tiene bed_b64 == original, armored no vacío, qr_png_b64 no vacío
      - DELETE /api/history/{id} asserta status 204
      - GET /api/history/{id} asserta status 404 HISTORY_NOT_FOUND
    - test_invalid_id_returns_422:
      - GET /api/history/xx asserta 422 HISTORY_INVALID_ID
      - GET /api/history/AAAAAAAA asserta 422 (uppercase)
      - DELETE /api/history/.. asserta 422
    - test_empty_dir_returns_empty_entries:
      - Setea BED_DATA_DIR a tempdir vacío
      - GET /api/history asserta {"entries": []}
    - test_invalid_base64_post_returns_422:
      - POST /api/history con bed_b64 = "not-valid-base64!!!" asserta 422 BAD_REQUEST

    **`history_no_leak.rs`** (HIST-03 — el descriptor en claro NUNCA aparece en archivos persistidos):
    - test_descriptor_cleartext_never_persisted:
      - Setea BED_DATA_DIR a tempdir
      - Define descriptor en claro: `let descriptor = "wsh(multi(2,[deadbeef/48h/0h/0h/2h]xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz/<0;1>/*,[deadbeef/48h/0h/0h/2h]xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ/<0;1>/*))#abc12345"`
      - Llama el flujo real: POST /api/encrypt con `{"descriptor": ...}` (xpub válida; necesitas leer crates/core/tests para encontrar un descriptor de ejemplo válido que pase validate::require_multipath_0_1; idealmente reutilizar el mismo del test round-trip de Phase 1)
      - Recibe la respuesta `{bed_b64, armored, qr_png_b64}`
      - POST /api/history con `{"bed_b64": <bed_b64>}` → recibe filename
      - Hace `tokio::fs::read_dir(BED_DATA_DIR)` y para cada archivo:
        - Lee bytes
        - Asserta que NINGÚN substring del descriptor de longitud >= 30 caracteres aparece en los bytes (grep equivalente)
        - Específicamente: ningún xpub completo (89 chars) aparece, ningún substring "wsh(multi" aparece, ningún checksum #abc12345 aparece
      - Mensaje del fallo si encuentra match: "HIST-03 violado: descriptor en claro detectado en archivo X"

    Si el descriptor exacto del test de Phase 1 ya está en algún test helper, reúsalo. Si no, define uno mínimo válido inline.
  </behavior>
  <action>
1. **`crates/server/tests/history_round_trip.rs`** — integration test usando `tower::ServiceExt::oneshot` (mismo pattern que Phase 1). Puntos clave:

```rust
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use bed_server::router;
use serde_json::{json, Value};
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
                .uri(&format!("/api/history/{id}"))
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
                .uri(&format!("/api/history/{id}"))
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
                .uri(&format!("/api/history/{id}"))
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
async fn invalid_id_returns_422() {
    let tmp = tempfile::tempdir().unwrap();
    set_data_dir(tmp.path());
    let app = router();

    for bad in &["xx", "AAAAAAAA", "a3f7b2c", "a3f7b2c1x", "a3f7b2g1"] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/history/{bad}"))
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
async fn invalid_base64_returns_422() {
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
```

**IMPORTANT** — Los tests en el mismo proceso Tokio comparten env vars; usa serial execution si hay race. Si dos tests setean BED_DATA_DIR concurrentemente, el último gana. Mitigación: usa `serial_test` crate (añade a dev-deps si necesario) o ejecuta `cargo test -- --test-threads=1` en CI. Verifica primero si hay race; si sí, añade `#[serial_test::serial]` a cada test.

Si añades `serial_test`:
```toml
# crates/server/Cargo.toml [dev-dependencies]
serial_test = "3"
```

Y marca cada `#[tokio::test]` con `#[serial_test::serial]` antes.

2. **`crates/server/tests/history_no_leak.rs`** — HIST-03 enforced:

```rust
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use bed_server::router;
use serde_json::{json, Value};
use tower::ServiceExt;

// Descriptor multisig 2-of-2 válido con derivación <0;1>/* (cumple require_multipath_0_1).
// IMPORTANTE: si Phase 1 ya tiene un test fixture similar, reúsalo. Si no, este string
// vivirá solo aquí — confirmar que parsea contra miniscript 12.3.5 y la validate_descriptor
// de bed_core. Si falla parsing en runtime, ajustar a uno conocido como válido.
const TEST_DESCRIPTOR: &str = "wsh(sortedmulti(2,\
[deadbeef/48h/0h/0h/2h]xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz/<0;1>/*,\
[cafef00d/48h/0h/0h/2h]xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ/<0;1>/*))#yvpsmzh3";

#[tokio::test]
async fn descriptor_cleartext_never_persisted_in_history_dir() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("BED_DATA_DIR", tmp.path());
    let app = router();

    // 1. Encrypt el descriptor — esto NO persiste, solo retorna los 3 outputs
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/encrypt")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "descriptor": TEST_DESCRIPTOR })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "encrypt failed — descriptor inválido?");
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    let bed_b64 = v["bed_b64"].as_str().unwrap().to_string();

    // 2. POST /api/history con el bed_b64 → escribe archivo en tmp
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
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Para cada archivo en tmp, leer bytes y verificar que NINGÚN substring
    //    significativo del descriptor en claro aparece (HIST-03).
    let needles: &[&[u8]] = &[
        b"wsh(sortedmulti",                                  // function name
        b"xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZ",     // xpub fragment 1
        b"xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53Q",     // xpub fragment 2
        b"deadbeef",                                          // fingerprint 1
        b"cafef00d",                                          // fingerprint 2
        b"#yvpsmzh3",                                         // checksum
        b"<0;1>/*",                                           // multipath wildcard
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
```

NOTA: el descriptor de prueba debe parsear contra miniscript 12.3.5. Si Phase 1 tiene tests con un descriptor conocido válido (busca con `grep -r "xpub6" /workspace/descriptor-cifrado/crates/`), reutiliza el mismo string para evitar riesgo de invalidez.

3. Compila y corre los tests: `cargo test -p bed-server --test history_round_trip --test history_no_leak`. Si los tests fallan por race en env var, añade `serial_test` y rerun.

NO hardcodees rutas absolutas a `/data/encrypted` en los tests — siempre tempdir.
NO comitas el directorio tmpdir (tempfile lo limpia automáticamente al drop).
NO inhabilites el test no_leak por flaky — si falla, el código tiene un bug de seguridad real.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado &amp;&amp; cargo test -p bed-server --test history_round_trip --test history_no_leak -- --test-threads=1 2>&amp;1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `test -f /workspace/descriptor-cifrado/crates/server/tests/history_round_trip.rs` retorna 0
    - `test -f /workspace/descriptor-cifrado/crates/server/tests/history_no_leak.rs` retorna 0
    - `grep "tempfile::tempdir" /workspace/descriptor-cifrado/crates/server/tests/history_round_trip.rs` encuentra match
    - `grep "BED_DATA_DIR" /workspace/descriptor-cifrado/crates/server/tests/history_round_trip.rs` encuentra match
    - `grep "HISTORY_INVALID_ID" /workspace/descriptor-cifrado/crates/server/tests/history_round_trip.rs` encuentra match
    - `grep "HISTORY_NOT_FOUND" /workspace/descriptor-cifrado/crates/server/tests/history_round_trip.rs` encuentra match
    - `grep "HIST-03" /workspace/descriptor-cifrado/crates/server/tests/history_no_leak.rs` encuentra match
    - `grep "wsh(sortedmulti\|wsh(multi" /workspace/descriptor-cifrado/crates/server/tests/history_no_leak.rs` encuentra match
    - `cd /workspace/descriptor-cifrado && cargo test -p bed-server --test history_round_trip -- --test-threads=1` exit code 0
    - `cd /workspace/descriptor-cifrado && cargo test -p bed-server --test history_no_leak -- --test-threads=1` exit code 0
    - El output de los tests muestra >=4 tests pasando en history_round_trip y >=1 en history_no_leak
  </acceptance_criteria>
  <done>history_round_trip cubre POST/GET/DELETE/422/404; history_no_leak cifra un descriptor real, lo persiste, y verifica que NINGÚN substring del cleartext aparece en archivos del data_dir; ambos integration tests verdes.</done>
</task>

</tasks>

<verification>
- `cargo build -p bed-server` exit code 0
- `cargo test -p bed-server` exit code 0 con todos los tests verdes (lib + integration round_trip + no_leak + Phase 1 tests intactos)
- `cargo clippy -p bed-server -- -D warnings` exit code 0 (workspace lints respetados)
- 4 endpoints registrados en lib.rs router; `grep "/api/history" crates/server/src/lib.rs` muestra 4 declaraciones
- AppError extendido sin romper variantes existentes
</verification>

<success_criteria>
- HIST-02: POST /api/history persiste un .bed con filename `<compact-timestamp>-<8-hex-id>.bed` en data_dir()
- HIST-04: GET /api/history retorna lista ordenada timestamp desc via directory scan; vacío retorna `{"entries": []}`
- HIST-05: DELETE /api/history/{id} elimina el archivo y retorna 204; 404 si no existe; 422 si id inválido (anti path traversal)
- HIST-03: integration test demuestra que el descriptor cleartext NO aparece en ningún archivo bajo BED_DATA_DIR tras encrypt + POST history
- AppError extendido con HistoryNotFound (404), HistoryWriteFailed (500), HistoryInvalidId (422)
- BED_DATA_DIR env var configurable (default /data/encrypted)
</success_criteria>

<output>
After completion, create `.planning/phases/02-spa-frontend-history/02-02-SUMMARY.md` describing:
- Versiones exactas resueltas: rust-embed, uuid, time, tempfile, serial_test (si añadido)
- Feature flag elegido para rust-embed: `axum` o `axum-ex`
- Confirmación de la signature de bed_core::encode_armored y render_qr_png usada
- Path syntax de axum 0.8 para path params (`{id}` vs `:id`)
- Conteo de tests pasando: lib + round_trip + no_leak
</output>
