---
phase: 02-spa-frontend-history
plan: 06
type: execute
wave: 4
depends_on: [02-02, 02-03, 02-04, 02-05]
files_modified:
  - frontend/src/App.svelte
  - frontend/src/components/TabHistorial.svelte
  - frontend/src/components/HistoryEntryDetailModal.svelte
  - frontend/src/components/ConfirmDeleteModal.svelte
  - frontend/src/lib/relativeTime.js
  - crates/server/Cargo.toml
  - crates/server/src/assets.rs
  - crates/server/src/lib.rs
  - crates/server/tests/embedded_spa.rs
autonomous: true
requirements: [UI-01, UI-02, HIST-02, HIST-04, HIST-05, HIST-06]
must_haves:
  truths:
    - "GET / sobre el binario `bed-server` corriendo devuelve 200 con Content-Type text/html y body que contiene `<div id=\"app\">` y un `<script type=\"module\" src=\"/assets/index-...js\">` embebido"
    - "GET /assets/<hashed-name>.js y .css devuelven los archivos del build de Vite con Content-Type correcto"
    - "GET /assets/<hashed-name>.woff2 (Inter, JetBrains Mono) devuelven 200 con Content-Type font/woff2"
    - "Tab Historial visible solo cuando `appState.historyEnabled === true` (D-20) y muestra GET /api/history en una lista con timestamp relativo + short-id + botones Ver/Borrar"
    - "Click en 'Ver' abre modal con los 3 outputs regenerados desde el .bed persistido (descarga .bed, copia armored, descarga PNG QR)"
    - "Click en 'Borrar' abre modal de confirmación con foco default en 'Cancelar' (D-36); confirmar dispara DELETE /api/history/{id} y la entrada desaparece de la lista + toast 'Entrada borrada'"
    - "Empty state cuando lista vacía: 'Sin backups cifrados aún' + 'Cifra un descriptor con el modo histórico activo para que aparezca aquí.' (D-24, UI-SPEC §Empty States)"
    - "El binario Rust embebe `frontend/dist/` vía rust-embed; cero requests externos al cargar la SPA (verificable con grep en index.html servido)"
  artifacts:
    - path: "crates/server/src/assets.rs"
      provides: "FrontendAssets (RustEmbed sobre frontend/dist/) + StaticFile<T> IntoResponse + index_handler + static_handler"
    - path: "crates/server/src/lib.rs"
      provides: "Registra rutas estáticas (GET /, GET /assets/*path) ANTES de las rutas /api/* en el Router"
    - path: "crates/server/tests/embedded_spa.rs"
      provides: "Test de integración: levanta Router, hace GET /, verifica Content-Type text/html y presencia de `<div id=\"app\">`; GET /assets/index-*.js verifica 200 con Content-Type application/javascript"
    - path: "frontend/src/components/TabHistorial.svelte"
      provides: "Lista de entradas, empty state, botones Ver/Borrar, toasts"
    - path: "frontend/src/components/HistoryEntryDetailModal.svelte"
      provides: "Modal con los 3 outputs regenerados desde GET /api/history/{id}"
    - path: "frontend/src/components/ConfirmDeleteModal.svelte"
      provides: "Modal de confirmación destructiva (foco default Cancelar, D-36)"
    - path: "frontend/src/lib/relativeTime.js"
      provides: "formatRelative(isoTimestamp) → 'hace 3 días' / 'hace 5 minutos' / 'ahora'"
      exports: ["formatRelative"]
  key_links:
    - from: "crates/server/src/lib.rs"
      to: "crates/server/src/assets.rs"
      via: "router.route('/', get(assets::index_handler)).route('/assets/{*path}', get(assets::static_handler))"
      pattern: "assets::"
    - from: "crates/server/src/assets.rs"
      to: "frontend/dist/"
      via: "#[derive(RustEmbed)] #[folder = \"../../frontend/dist/\"]"
      pattern: "#\\[folder"
    - from: "frontend/src/components/TabHistorial.svelte"
      to: "frontend/src/lib/api.js"
      via: "getJson('/api/history') / deleteJson('/api/history/{id}')"
      pattern: "/api/history"
    - from: "frontend/src/components/HistoryEntryDetailModal.svelte"
      to: "frontend/src/lib/api.js"
      via: "getJson('/api/history/{id}') para regenerar bed_b64+armored+qr_png_b64"
      pattern: "/api/history/"
    - from: "frontend/src/App.svelte"
      to: "frontend/src/components/TabHistorial.svelte"
      via: "import + montaje condicional cuando appState.historyEnabled"
      pattern: "TabHistorial"
---

<objective>
Cerrar Phase 2 con dos integraciones:

1. **Frontend**: implementar Tab Historial (lista + detalle + borrado con modal de confirmación) que consume los 4 endpoints creados en Plan 02-02. Tab solo visible si `appState.historyEnabled === true` (D-20).
2. **Backend**: integrar el SPA compilado (`frontend/dist/`) en el binario Rust vía `rust-embed`. Servir `GET /` con `index.html` y `GET /assets/{*path}` con los assets hashados (JS, CSS, woff2). Rutas estáticas se registran ANTES de las rutas `/api/*` en el Router para que las primeras tengan prioridad sobre coincidencias parciales.
3. **Test de integración**: verificar que el binario sirve la SPA correctamente (UI-01: cero requests externos, todo embebido).

Purpose: Cubrir UI-01 (SPA servida desde el binario sin CDN), HIST-02/HIST-04/HIST-05/HIST-06 (UI completa de historial con persistencia ya hecha en backend Plan 02-02). Tras este plan, Phase 2 queda cerrada — `cargo run -p bed-server` arranca el binario y `http://127.0.0.1:8080` sirve la SPA completa.

Output: assets.rs + Cargo.toml extendido + lib.rs router actualizado + test embedded_spa.rs; TabHistorial.svelte + 2 modales + relativeTime.js helper; App.svelte montando TabHistorial.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/02-spa-frontend-history/02-CONTEXT.md
@.planning/phases/02-spa-frontend-history/02-RESEARCH.md
@.planning/phases/02-spa-frontend-history/02-UI-SPEC.md
@.planning/phases/02-spa-frontend-history/02-02-SUMMARY.md
@.planning/phases/02-spa-frontend-history/02-03-SUMMARY.md
@frontend/src/App.svelte
@frontend/src/lib/tokens.css
@frontend/src/lib/api.js
@frontend/src/lib/clipboard.js
@frontend/src/lib/download.js
@frontend/src/components/Modal.svelte
@frontend/src/components/Toast.svelte
@frontend/src/components/InlineError.svelte
@frontend/src/components/Spinner.svelte
@crates/server/src/lib.rs
@crates/server/src/state.rs
@Cargo.toml

<interfaces>
<!-- Backend exports (Plan 02-02) -->

From crates/server/src/lib.rs (existing):
```rust
pub fn build_router(state: AppState) -> Router; // Phase 1 + Plan 02-02 history routes
```

From Plan 02-02, ya registradas en `build_router`:
- `POST /api/encrypt`, `POST /api/decrypt` (Phase 1)
- `POST /api/history`, `GET /api/history`, `GET /api/history/{id}`, `DELETE /api/history/{id}` (Plan 02-02)

Backend contract (Plan 02-02) — `GET /api/history`:
- Response 200: `{ "entries": [{ "id": "<8 hex>", "timestamp": "<ISO>", "filename": "...", "size_bytes": 123 }, ...] }`
- Empty: `{ "entries": [] }`

Backend contract (Plan 02-02) — `GET /api/history/{id}`:
- Response 200: `{ "bed_b64": "...", "armored": "...", "qr_png_b64": "..." }` (mismo shape que `/api/encrypt`)
- 404 si no existe; 422 si id invalid format

Backend contract (Plan 02-02) — `DELETE /api/history/{id}`:
- Response 204 No Content si OK
- 404 si no existe; 422 si id invalid format

<!-- Frontend exports (Plan 02-03 + 02-04) consumidos aquí -->

From frontend/src/lib/api.js:
```javascript
export async function getJson(url): any;
export async function deleteJson(url): any;
```

From frontend/src/lib/download.js (Plan 02-04):
```javascript
export function triggerDownloadBase64(b64, filename, mime): void;
```

From frontend/src/lib/clipboard.js (Plan 02-03):
```javascript
export async function copyToClipboard(text): Promise<boolean>;
```

From frontend/src/stores/app.svelte.js (Plan 02-03):
```javascript
export const appState; // { activeTab, theme, historyEnabled }
```

From frontend/src/components/Modal.svelte (Plan 02-03):
```svelte
<!-- Props: open (bindable), title, onClose, primaryLabel, primaryAction, primaryVariant, secondaryLabel -->
<!-- Foco default en Cancel (D-36); Escape cierra; aria-modal=true; role=dialog -->
```

<!-- rust-embed feature flag a verificar -->

CONTEXT.md / STACK indica `rust-embed = "8"` con feature `axum` o `axum-ex` (la 8.5+ unificó a `axum`).
La verificación exacta debe hacerse al añadir la dep — el patrón canónico es:

```toml
rust-embed = { version = "8", features = ["axum"] }
# o si esa feature no existe en la versión instalada, probar:
# rust-embed = { version = "8", features = ["axum-ex"] }
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="false">
  <name>Task 1: rust-embed wiring (assets.rs + Cargo.toml + lib.rs router)</name>
  <files>crates/server/Cargo.toml, crates/server/src/assets.rs, crates/server/src/lib.rs, Cargo.toml</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-39 carpeta frontend top-level; D-40 rust-embed folder; D-41 hashed filenames; D-42 dev mode proxy NO afecta este plan; D-43 fonts en assets/fonts/)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (§rust-embed + axum integration, ejemplo de StaticFile<T> + IntoResponse + index/static handlers; §Trampa 6 — feature flag axum vs axum-ex)
    - Cargo.toml (workspace deps — rust-embed se añade a [workspace.dependencies] si no existe)
    - crates/server/Cargo.toml (deps actuales)
    - crates/server/src/lib.rs (orden actual de rutas; verificar dónde se nestea /api)
    - frontend/dist/ (debe existir tras Plan 02-01..05; si no, crear con `cd frontend && npm run build` antes)
  </read_first>
  <action>
1. **Pre-check**: Asegurar que `frontend/dist/` existe (necesario para que `rust-embed` pueda compilar):

```bash
cd /workspace/descriptor-cifrado/frontend && npm install && npm run build
test -f dist/index.html || { echo "ERROR: frontend/dist/index.html no existe"; exit 1; }
```

2. **Añadir `rust-embed` a `[workspace.dependencies]` en `Cargo.toml` raíz** (si no está). Probar primero feature `axum`; si el build falla con "feature not found", caer a `axum-ex`:

```toml
[workspace.dependencies]
# ... resto ...
rust-embed = { version = "8", features = ["axum"] }
mime_guess = "2"
```

NOTA: `mime_guess` es necesario para devolver el `Content-Type` correcto en el static_handler (rust-embed expone el `mimetype` en `EmbeddedFile.metadata.mimetype`, pero mime_guess es más robusto cuando el filename trae extensión .woff2/.css).

3. **Añadir las deps al `crates/server/Cargo.toml`**:

```toml
[dependencies]
# ... existentes ...
rust-embed = { workspace = true }
mime_guess = { workspace = true }
```

4. **Crear `crates/server/src/assets.rs`** — embed + handlers. Patrón canónico verificado en RESEARCH §rust-embed + axum:

```rust
//! Embed del SPA compilado por Vite. Usa rust-embed para incluir
//! `frontend/dist/` en el binario en compile-time.
//!
//! Routes:
//! - GET /            → index.html
//! - GET /assets/{*path} → cualquier asset hashado (JS, CSS, woff2)

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
                    // Cache largo para assets hashados; index.html sin cache largo (devolveremos a index_handler).
                    .header(header::CACHE_CONTROL, cache_control_for(path))
                    .body(Body::from(content.data.into_owned()))
                    .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "asset error").into_response())
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
    let path = uri.path().trim_start_matches('/');
    StaticFile(path.to_string())
}
```

5. **Editar `crates/server/src/lib.rs`** para registrar las rutas estáticas. Añadir el módulo y registrar las rutas. CRÍTICO: registrar `/api/*` PRIMERO con `nest` o como rutas individuales, luego añadir `/` y `/assets/{*path}` al final, porque axum 0.8 hace match más específico ganador, pero queremos garantizar que `/api/history/abc12345` no matchee accidentalmente con un asset path. Usar `axum::routing::get` para los handlers nuevos:

```rust
mod assets;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // API routes (Phase 1 + Plan 02-02) — registradas primero, axum 0.8 las prefiere por especificidad
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        .route("/api/history", post(routes::history::post_history).get(routes::history::get_history))
        .route("/api/history/{id}", get(routes::history::get_history_id).delete(routes::history::delete_history))
        // Static assets (SPA) — al final; los hashed paths viven bajo /assets/*
        .route("/", get(assets::index_handler))
        .route("/assets/{*path}", get(assets::static_handler))
        .with_state(state)
        .layer(/* tower-http TraceLayer existente */)
}
```

NO uses axum 0.7 syntax `:id` o `*path` — son `{id}` y `{*path}` en axum 0.8.
NO añadas un fallback genérico que sirva index.html para cualquier ruta no-API — nuestra SPA usa routing state-based interno (D-06), no client-side router con URLs. Solo `/` y `/assets/*` son válidas como rutas estáticas.

6. **Crear `crates/server/tests/embedded_spa.rs`** — test de integración que verifica el embed funciona end-to-end:

```rust
//! Verifica que rust-embed sirve la SPA correctamente.
//! Test fundamental para UI-01 (SPA servida desde el binario sin requests externos).

use axum::{body::Body, http::{Request, StatusCode}};
use bed_server::{build_router, state::AppState};
use serial_test::serial;
use tempfile::TempDir;
use tower::ServiceExt;

fn fresh_state() -> (AppState, TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    // SAFETY (test only): set env var so AppState::data_dir() points at tmp.
    unsafe { std::env::set_var("BED_DATA_DIR", tmp.path()); }
    (AppState::default(), tmp)
}

#[tokio::test]
#[serial]
async fn get_root_returns_spa_html() {
    let (state, _tmp) = fresh_state();
    let app = build_router(state);

    let resp = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .expect("oneshot");

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp.headers().get("content-type").expect("content-type").to_str().unwrap();
    assert!(ct.contains("text/html"), "expected text/html, got {ct}");

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.expect("body");
    let body = std::str::from_utf8(&bytes).expect("utf8");

    // Marcas mínimas de la SPA Svelte
    assert!(body.contains("<div id=\"app\">"), "missing root mount point");
    // Vite emite hashed assets bajo /assets/
    assert!(body.contains("/assets/index-"), "missing /assets/index- reference");
    // Cero referencias externas (UI-01)
    assert!(!body.contains("https://"), "external https:// reference detected in index.html");
    assert!(!body.contains("//fonts."), "external font URL detected");
    assert!(!body.contains("googleapis"), "googleapis reference leaked");
    assert!(!body.contains("googleusercontent"), "google reference leaked");
}

#[tokio::test]
#[serial]
async fn get_assets_returns_200() {
    let (state, _tmp) = fresh_state();
    let app = build_router(state.clone());

    // Primero leer index.html para obtener el nombre del JS hasheado
    let resp = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .expect("oneshot");
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let body = std::str::from_utf8(&body_bytes).unwrap();

    // Encontrar el primer /assets/index-*.js referenciado
    let needle = "/assets/index-";
    let start = body.find(needle).expect("no /assets/index- in HTML");
    let rest = &body[start..];
    let end = rest.find('"').expect("no closing quote");
    let asset_path = &rest[..end];
    assert!(asset_path.ends_with(".js"), "expected .js, got {asset_path}");

    let resp2 = app
        .oneshot(Request::builder().uri(asset_path).body(Body::empty()).unwrap())
        .await
        .expect("oneshot");
    assert_eq!(resp2.status(), StatusCode::OK);
    let ct = resp2.headers().get("content-type").expect("content-type").to_str().unwrap();
    assert!(ct.contains("javascript") || ct.contains("ecmascript"), "expected js MIME, got {ct}");
}
```

7. Verificar build + tests:

```bash
cd /workspace/descriptor-cifrado
cargo build -p bed-server 2>&1 | tail -5
cargo test -p bed-server --test embedded_spa -- --test-threads=1 2>&1 | tail -10
```

Si `cargo build` falla con "unresolved feature `axum`", cambiar a `features = ["axum-ex"]` en ambos `Cargo.toml`.
NO commitees `frontend/dist/` al repo — añade a `.gitignore` si no está. rust-embed lo embebe en compile-time pero el repo solo guarda el source.
NO añadas una ruta fallback genérica que devuelva index.html — la SPA no usa client-side routing con URLs (D-06).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado &amp;&amp; (test -f frontend/dist/index.html || (cd frontend &amp;&amp; npm install &amp;&amp; npm run build)) &amp;&amp; cargo build -p bed-server 2>&amp;1 | tail -5 &amp;&amp; cargo test -p bed-server --test embedded_spa -- --test-threads=1 2>&amp;1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `grep "rust-embed" /workspace/descriptor-cifrado/Cargo.toml` encuentra match (workspace dep)
    - `grep "mime_guess" /workspace/descriptor-cifrado/Cargo.toml` encuentra match
    - `grep "rust-embed" /workspace/descriptor-cifrado/crates/server/Cargo.toml` encuentra match
    - `grep "#\\[derive(RustEmbed)\\]" /workspace/descriptor-cifrado/crates/server/src/assets.rs` encuentra match
    - `grep "folder = \"../../frontend/dist/\"" /workspace/descriptor-cifrado/crates/server/src/assets.rs` encuentra match
    - `grep "pub async fn index_handler" /workspace/descriptor-cifrado/crates/server/src/assets.rs` encuentra match
    - `grep "pub async fn static_handler" /workspace/descriptor-cifrado/crates/server/src/assets.rs` encuentra match
    - `grep "mod assets" /workspace/descriptor-cifrado/crates/server/src/lib.rs` encuentra match
    - `grep '"/assets/{\\*path}"' /workspace/descriptor-cifrado/crates/server/src/lib.rs` encuentra match (axum 0.8 syntax)
    - `! grep '"/assets/\\*path"' /workspace/descriptor-cifrado/crates/server/src/lib.rs` (no axum 0.7 syntax)
    - `! grep "/api/history/:id" /workspace/descriptor-cifrado/crates/server/src/lib.rs` (no axum 0.7 syntax — debe ser `{id}`)
    - `cargo build -p bed-server` exit code 0
    - `cargo test -p bed-server --test embedded_spa -- --test-threads=1` exit code 0 (ambos tests pasan)
    - `cargo clippy -p bed-server -- -D warnings` exit code 0
  </acceptance_criteria>
  <done>FrontendAssets embebido; rutas / y /assets/{*path} registradas en el Router; tests embedded_spa.rs verifican que GET / devuelve la SPA con `<div id="app">` y que GET /assets/index-*.js devuelve 200 con MIME JS; cero referencias externas en el index.html servido (UI-01).</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: TabHistorial.svelte + 2 modales + relativeTime helper + integración con App.svelte</name>
  <files>frontend/src/lib/relativeTime.js, frontend/src/components/HistoryEntryDetailModal.svelte, frontend/src/components/ConfirmDeleteModal.svelte, frontend/src/components/TabHistorial.svelte, frontend/src/App.svelte</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-20 Tab condicional; D-21 lista con timestamp relativo + tooltip ISO; D-22 Ver regenera 3 outputs; D-23 modal confirmación delete; D-24 empty state; D-36 default focus Cancel)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§History Entry Row layout; §Empty States; §Destructive Confirmation; §Toast Messages "Entrada borrada")
    - frontend/src/lib/api.js (getJson + deleteJson + ApiError)
    - frontend/src/lib/download.js (triggerDownloadBase64 — Plan 02-04)
    - frontend/src/lib/clipboard.js (copyToClipboard)
    - frontend/src/components/Modal.svelte (props del modal compartido — Plan 02-03)
    - frontend/src/stores/app.svelte.js (appState.historyEnabled)
    - frontend/src/App.svelte (placeholder Tab Historial a reemplazar)
  </read_first>
  <action>
1. **`frontend/src/lib/relativeTime.js`** — formatter castellano (no argentino):

```javascript
// Convierte ISO timestamp a "hace N días" / "hace N horas" / etc. en castellano.
// Source: Intl.RelativeTimeFormat con locale 'es' — produce "hace 3 días" nativamente.

const RTF = new Intl.RelativeTimeFormat('es', { numeric: 'auto' });

export function formatRelative(isoTimestamp) {
  const then = new Date(isoTimestamp);
  if (isNaN(then.getTime())) return isoTimestamp;
  const now = new Date();
  const diffSec = Math.round((then.getTime() - now.getTime()) / 1000);
  const absSec = Math.abs(diffSec);

  if (absSec < 60) return RTF.format(diffSec, 'second');
  if (absSec < 3600) return RTF.format(Math.round(diffSec / 60), 'minute');
  if (absSec < 86400) return RTF.format(Math.round(diffSec / 3600), 'hour');
  if (absSec < 2592000) return RTF.format(Math.round(diffSec / 86400), 'day');
  if (absSec < 31536000) return RTF.format(Math.round(diffSec / 2592000), 'month');
  return RTF.format(Math.round(diffSec / 31536000), 'year');
}
```

2. **`frontend/src/components/HistoryEntryDetailModal.svelte`** — modal con los 3 outputs regenerados (D-22). Usa el patrón del CifrarOutputs pero recibe los datos de `GET /api/history/{id}`:

```svelte
<script>
  import { getJson, ApiError } from '../lib/api.js';
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadBase64 } from '../lib/download.js';
  import Spinner from './Spinner.svelte';
  import Toast from './Toast.svelte';

  let { open = $bindable(false), entryId = '', filename = '' } = $props();

  let loading = $state(false);
  let errorMessage = $state('');
  let result = $state(null); // { bed_b64, armored, qr_png_b64 }
  let copyLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');

  $effect(() => {
    if (open && entryId && !result) {
      void loadDetail();
    }
    if (!open) {
      result = null;
      errorMessage = '';
      copyLabel = 'Copiar al portapapeles';
      clearTimeout(copyResetTimer);
    }
  });

  async function loadDetail() {
    loading = true;
    errorMessage = '';
    try {
      result = await getJson(`/api/history/${entryId}`);
    } catch (e) {
      if (e instanceof ApiError) {
        if (e.status === 404) {
          errorMessage = 'Esta entrada ya no existe en el historial.';
        } else {
          errorMessage = e.message;
        }
      } else {
        errorMessage = 'No se pudo conectar al servidor local.';
      }
    } finally {
      loading = false;
    }
  }

  function showToast(msg) {
    toastMessage = msg;
    toastVisible = true;
  }

  function downloadBed() {
    if (!result) return;
    triggerDownloadBase64(result.bed_b64, filename || `${entryId}.bed`, 'application/octet-stream');
  }

  function downloadQrPng() {
    if (!result) return;
    const base = (filename || `${entryId}.bed`).replace(/\.bed$/i, '');
    triggerDownloadBase64(result.qr_png_b64, `${base}.png`, 'image/png');
  }

  async function handleCopyArmored() {
    if (!result) return;
    const ok = await copyToClipboard(result.armored);
    if (ok) {
      copyLabel = 'Copiado ✓';
      showToast('Copiado al portapapeles');
      clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => { copyLabel = 'Copiar al portapapeles'; }, 1500);
    } else {
      showToast('No se pudo copiar al portapapeles');
    }
  }

  function handleClose() { open = false; }
</script>

{#if open}
  <div class="backdrop" onclick={handleClose} role="presentation">
    <div
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="detail-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="detail-title" class="title">Backup cifrado</h2>
      <p class="subtitle">{filename}</p>

      {#if loading}
        <p class="hint"><Spinner /> Cargando…</p>
      {:else if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {:else if result}
        <div class="output">
          <div class="row">
            <span class="label">Archivo .bed</span>
            <button type="button" class="btn btn-primary" onclick={downloadBed}>Descargar .bed</button>
          </div>
        </div>

        <div class="output">
          <div class="row">
            <span class="label">Texto armored</span>
            <button type="button" class="btn btn-secondary" onclick={handleCopyArmored}>{copyLabel}</button>
          </div>
          <pre class="armored">{result.armored}</pre>
        </div>

        <div class="output">
          <div class="row">
            <span class="label">Código QR (PNG)</span>
            <button type="button" class="btn btn-secondary" onclick={downloadQrPng}>Descargar PNG</button>
          </div>
          <figure class="qr">
            <img src="data:image/png;base64,{result.qr_png_b64}" alt="Código QR del backup cifrado" width="180" height="180" />
          </figure>
        </div>
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-secondary" onclick={handleClose}>Cerrar</button>
      </div>
    </div>
  </div>
{/if}

<Toast bind:visible={toastVisible} message={toastMessage} />

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(2px); z-index: 9000; display: flex; align-items: center; justify-content: center; padding: var(--space-md); }
  .panel { background: var(--color-surface-raised); border-radius: var(--radius-card); padding: var(--space-lg); max-width: 480px; width: 100%; max-height: 90vh; overflow-y: auto; box-shadow: var(--shadow-modal); }
  .title { margin: 0 0 var(--space-xs) 0; font-size: var(--font-size-heading); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .subtitle { margin: 0 0 var(--space-md) 0; font-family: var(--font-mono); font-size: var(--font-size-mono); color: var(--color-text-secondary); }
  .output { margin-bottom: var(--space-md); }
  .row { display: flex; justify-content: space-between; align-items: center; gap: var(--space-md); margin-bottom: var(--space-sm); flex-wrap: wrap; }
  .label { font-size: var(--font-size-label); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .armored { font-family: var(--font-mono); font-size: var(--font-size-mono); line-height: var(--line-height-mono); background: var(--color-surface-sunken); border: 1px solid var(--color-border); border-radius: var(--radius-input); padding: var(--space-md); white-space: pre-wrap; word-break: break-all; max-height: 160px; overflow-y: auto; margin: 0; }
  .qr { margin: 0; display: flex; justify-content: center; background: white; padding: var(--space-md); border-radius: var(--radius-input); border: 1px solid var(--color-border); }
  .hint { font-size: var(--font-size-label); color: var(--color-text-secondary); display: flex; align-items: center; gap: var(--space-sm); }
  .error { margin: 0 0 var(--space-md) 0; font-size: var(--font-size-label); color: var(--color-warning-text); background: var(--color-warning-bg); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-input); border-left: 4px solid var(--color-warning-border); }
  .actions { display: flex; justify-content: flex-end; gap: var(--space-sm); margin-top: var(--space-md); }
  .btn { min-height: var(--touch-target); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-button); font-size: var(--font-size-label); cursor: pointer; transition: background-color var(--transition-color); }
  .btn-primary { background: var(--color-accent); color: var(--color-accent-fg); border: 0; }
  .btn-primary:hover { background: var(--color-accent-hover); }
  .btn-secondary { background: transparent; color: var(--color-text-primary); border: 1px solid var(--color-border); }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
</style>
```

3. **`frontend/src/components/ConfirmDeleteModal.svelte`** — modal de confirmación destructiva. Foco default en Cancel (D-36). Copy literal del UI-SPEC §Destructive Confirmation:

```svelte
<script>
  let { open = $bindable(false), entry = null, onConfirm = () => {} } = $props();

  let cancelButton;
  let loading = $state(false);

  $effect(() => {
    if (open && cancelButton) {
      // Default focus en Cancelar (D-36) tras un microtask para que el DOM esté pintado.
      queueMicrotask(() => cancelButton.focus());
    }
  });

  function handleKeydown(e) {
    if (e.key === 'Escape' && !loading) {
      open = false;
    }
  }

  async function handleConfirm() {
    loading = true;
    try {
      await onConfirm();
      open = false;
    } finally {
      loading = false;
    }
  }

  function handleCancel() {
    if (!loading) open = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && entry}
  <div class="backdrop" onclick={handleCancel} role="presentation">
    <div
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="confirm-title" class="title">Borrar backup cifrado</h2>
      <p class="body">
        ¿Eliminar el backup <code class="entry-id">{entry.timestamp} · {entry.id}</code>?
        Esta acción no se puede deshacer.
      </p>
      <div class="actions">
        <button bind:this={cancelButton} type="button" class="btn btn-secondary" onclick={handleCancel} disabled={loading}>Cancelar</button>
        <button type="button" class="btn btn-destructive" onclick={handleConfirm} disabled={loading}>
          {loading ? 'Borrando…' : 'Borrar'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(2px); z-index: 9000; display: flex; align-items: center; justify-content: center; padding: var(--space-md); }
  .panel { background: var(--color-surface-raised); border-radius: var(--radius-card); padding: var(--space-lg); max-width: 400px; width: 100%; box-shadow: var(--shadow-modal); }
  .title { margin: 0 0 var(--space-md) 0; font-size: var(--font-size-heading); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .body { margin: 0 0 var(--space-lg) 0; font-size: var(--font-size-body); color: var(--color-text-primary); line-height: var(--line-height-body); }
  .entry-id { font-family: var(--font-mono); font-size: var(--font-size-mono); background: var(--color-surface-sunken); padding: 1px 4px; border-radius: 4px; }
  .actions { display: flex; justify-content: flex-end; gap: var(--space-sm); }
  .btn { min-height: var(--touch-target); min-width: var(--touch-target); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-button); font-size: var(--font-size-label); cursor: pointer; transition: background-color var(--transition-color); }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary { background: transparent; color: var(--color-text-primary); border: 1px solid var(--color-border); }
  .btn-secondary:hover:not(:disabled) { background: var(--color-surface-sunken); }
  .btn-destructive { background: var(--color-destructive); color: var(--color-destructive-fg); border: 0; }
  .btn-destructive:hover:not(:disabled) { background: var(--color-destructive-hover); }
</style>
```

4. **`frontend/src/components/TabHistorial.svelte`** — la tab principal. Carga la lista al montar, refresca tras delete, gestiona los modales:

```svelte
<script>
  import { getJson, deleteJson, ApiError } from '../lib/api.js';
  import { formatRelative } from '../lib/relativeTime.js';
  import HistoryEntryDetailModal from './HistoryEntryDetailModal.svelte';
  import ConfirmDeleteModal from './ConfirmDeleteModal.svelte';
  import InlineError from './InlineError.svelte';
  import Spinner from './Spinner.svelte';
  import Toast from './Toast.svelte';

  let entries = $state([]);
  let loading = $state(true);
  let errorVisible = $state(false);
  let errorMessage = $state('');

  let detailOpen = $state(false);
  let detailEntryId = $state('');
  let detailFilename = $state('');

  let confirmOpen = $state(false);
  let confirmEntry = $state(null);

  let toastVisible = $state(false);
  let toastMessage = $state('');

  $effect(() => { void loadList(); });

  async function loadList() {
    loading = true;
    errorVisible = false;
    errorMessage = '';
    try {
      const resp = await getJson('/api/history');
      entries = (resp.entries ?? []).slice().sort((a, b) => b.timestamp.localeCompare(a.timestamp));
    } catch (e) {
      errorMessage = e instanceof ApiError ? e.message : 'No se pudo conectar al servidor local.';
      errorVisible = true;
    } finally {
      loading = false;
    }
  }

  function openDetail(entry) {
    detailEntryId = entry.id;
    detailFilename = entry.filename;
    detailOpen = true;
  }

  function openConfirm(entry) {
    confirmEntry = entry;
    confirmOpen = true;
  }

  async function handleDelete() {
    if (!confirmEntry) return;
    const id = confirmEntry.id;
    try {
      await deleteJson(`/api/history/${id}`);
      entries = entries.filter((e) => e.id !== id);
      toastMessage = 'Entrada borrada';
      toastVisible = true;
    } catch (e) {
      errorMessage = e instanceof ApiError ? e.message : 'No se pudo borrar la entrada.';
      errorVisible = true;
    }
  }
</script>

<div class="tab-historial">
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  {#if loading}
    <p class="loading"><Spinner /> Cargando…</p>
  {:else if entries.length === 0}
    <div class="empty">
      <h2 class="empty-title">Sin backups cifrados aún</h2>
      <p class="empty-body">Cifra un descriptor con el modo histórico activo para que aparezca aquí.</p>
    </div>
  {:else}
    <ul class="entries" aria-label="Lista de backups cifrados">
      {#each entries as entry (entry.id)}
        <li class="entry">
          <div class="info">
            <span class="when" title={entry.timestamp}>{formatRelative(entry.timestamp)}</span>
            <span class="filename">{entry.filename}</span>
            <span class="size">{Math.round(entry.size_bytes / 1024 * 10) / 10} KB</span>
          </div>
          <div class="actions">
            <button type="button" class="btn btn-secondary" onclick={() => openDetail(entry)}>Ver</button>
            <button type="button" class="btn btn-destructive" onclick={() => openConfirm(entry)}>Borrar</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<HistoryEntryDetailModal bind:open={detailOpen} entryId={detailEntryId} filename={detailFilename} />
<ConfirmDeleteModal bind:open={confirmOpen} entry={confirmEntry} onConfirm={handleDelete} />
<Toast bind:visible={toastVisible} message={toastMessage} />

<style>
  .tab-historial { display: flex; flex-direction: column; gap: var(--space-md); }
  .loading { font-size: var(--font-size-label); color: var(--color-text-secondary); display: flex; align-items: center; gap: var(--space-sm); }
  .empty { text-align: center; padding: var(--space-2xl) var(--space-md); }
  .empty-title { margin: 0 0 var(--space-sm) 0; font-size: var(--font-size-heading); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .empty-body { margin: 0; font-size: var(--font-size-body); color: var(--color-text-secondary); line-height: var(--line-height-body); }
  .entries { list-style: none; padding: 0; margin: 0; background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: var(--radius-card); overflow: hidden; }
  .entry { display: flex; align-items: center; justify-content: space-between; padding: var(--space-sm-plus) var(--space-md); min-height: 56px; border-bottom: 1px solid var(--color-border); flex-wrap: wrap; gap: var(--space-md); }
  .entry:last-child { border-bottom: 0; }
  .info { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .when { font-size: var(--font-size-label); color: var(--color-text-primary); }
  .filename { font-family: var(--font-mono); font-size: var(--font-size-mono); color: var(--color-text-secondary); word-break: break-all; }
  .size { font-size: var(--font-size-label); color: var(--color-text-secondary); }
  .actions { display: flex; gap: var(--space-sm); }
  .btn { min-height: var(--touch-target); min-width: var(--touch-target); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-button); font-size: var(--font-size-label); cursor: pointer; transition: background-color var(--transition-color); }
  .btn-secondary { background: transparent; color: var(--color-text-primary); border: 1px solid var(--color-border); }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
  .btn-destructive { background: transparent; color: var(--color-destructive); border: 1px solid var(--color-destructive); }
  .btn-destructive:hover { background: var(--color-destructive); color: var(--color-destructive-fg); }
</style>
```

5. **`frontend/src/App.svelte`** — REEMPLAZAR el placeholder de la tab Historial. Importante: el tabpanel solo se renderiza en el DOM si `appState.historyEnabled === true` (D-20). El TabBar (Plan 02-03) ya gating la pestaña; aquí también gating el panel:

Buscar el placeholder actual:
```svelte
  {#if appState.historyEnabled}
    <section
      role="tabpanel"
      id="panel-historial"
      aria-labelledby="tab-historial"
      class="panel"
      hidden={appState.activeTab !== 'historial'}
    >
      <!-- Plan 02-06 monta TabHistorial aquí -->
      <p class="placeholder">Tab Historial — pendiente de plan 02-06.</p>
    </section>
  {/if}
```

Reemplazar por:
```svelte
  {#if appState.historyEnabled}
    <section
      role="tabpanel"
      id="panel-historial"
      aria-labelledby="tab-historial"
      class="panel"
      hidden={appState.activeTab !== 'historial'}
    >
      <TabHistorial />
    </section>
  {/if}
```

Y añadir el import:
```svelte
import TabHistorial from './components/TabHistorial.svelte';
```

6. Verificar build + bundle size + smoke test:

```bash
cd /workspace/descriptor-cifrado/frontend && npm run build 2>&1 | tail -8
# Bundle inicial (sin chunk dinámico de bbqr)
INITIAL=$(for f in dist/assets/index-*.js dist/assets/index-*.css; do [ -f "$f" ] && gzip -c "$f" | wc -c; done | awk '{s+=$1} END {print s}')
echo "Bundle inicial JS+CSS gzipped: $INITIAL bytes"
[ "$INITIAL" -lt 51200 ] || { echo "Bundle excede 50 KB"; exit 1; }

# Smoke test full-stack:
cd /workspace/descriptor-cifrado
mkdir -p /tmp/bed-test
BED_DATA_DIR=/tmp/bed-test cargo run -p bed-server &
SERVER_PID=$!
sleep 3
# Verificar que el binario sirve la SPA:
curl -fsS http://127.0.0.1:8080/ | grep -q '<div id="app">' && echo "SPA OK"
# Cifrar un descriptor y guardarlo en historial:
# (usar el descriptor de los tests Phase 1 si está disponible)
curl -fsS http://127.0.0.1:8080/api/history | head -c 200
kill $SERVER_PID
```

NO actives Tab Historial en el TabBar manualmente — el `appState.historyEnabled` ya viene del Plan 02-03 y se persiste en localStorage `bed.historyEnabled` (default false). El usuario activa el toggle del header para ver la tab.
NO añadas un botón "refrescar" — `$effect` se ejecuta cuando se monta el componente (al activar la tab), eso basta. Tras delete recargamos en memoria, no via re-fetch.
NO loguees el id ni el filename en console — security default.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -8 &amp;&amp; INITIAL=$(for f in dist/assets/index-*.js dist/assets/index-*.css; do [ -f "$f" ] &amp;&amp; gzip -c "$f" | wc -c; done | awk '{s+=$1} END {print s}') &amp;&amp; echo "Bundle inicial: $INITIAL bytes" &amp;&amp; [ "$INITIAL" -lt 51200 ] &amp;&amp; cd /workspace/descriptor-cifrado &amp;&amp; cargo build -p bed-server 2>&amp;1 | tail -5 &amp;&amp; cargo test -p bed-server --test embedded_spa -- --test-threads=1 2>&amp;1 | tail -5</automated>
  </verify>
  <acceptance_criteria>
    - `grep "import TabHistorial" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `grep "<TabHistorial" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match
    - `! grep "pendiente de plan 02-06" /workspace/descriptor-cifrado/frontend/src/App.svelte` (placeholder reemplazado)
    - `grep "appState.historyEnabled" /workspace/descriptor-cifrado/frontend/src/App.svelte` encuentra match (D-20 gate)
    - `grep "/api/history" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match (GET list)
    - `grep "deleteJson" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match (DELETE)
    - `grep "Sin backups cifrados aún" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match (empty state literal)
    - `grep "Cifra un descriptor con el modo histórico activo" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match
    - `grep "Entrada borrada" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match (toast literal)
    - `grep "formatRelative" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match (D-21)
    - `grep "title={entry.timestamp}" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` encuentra match (tooltip ISO D-21)
    - `grep "Borrar backup cifrado" /workspace/descriptor-cifrado/frontend/src/components/ConfirmDeleteModal.svelte` encuentra match (D-23 modal title literal)
    - `grep "Esta acción no se puede deshacer" /workspace/descriptor-cifrado/frontend/src/components/ConfirmDeleteModal.svelte` encuentra match
    - `grep "cancelButton.focus" /workspace/descriptor-cifrado/frontend/src/components/ConfirmDeleteModal.svelte` encuentra match (D-36 default focus)
    - `grep "role=\"dialog\"" /workspace/descriptor-cifrado/frontend/src/components/ConfirmDeleteModal.svelte` encuentra match
    - `grep "role=\"dialog\"" /workspace/descriptor-cifrado/frontend/src/components/HistoryEntryDetailModal.svelte` encuentra match
    - `grep "Descargar .bed" /workspace/descriptor-cifrado/frontend/src/components/HistoryEntryDetailModal.svelte` encuentra match
    - `grep "Descargar PNG" /workspace/descriptor-cifrado/frontend/src/components/HistoryEntryDetailModal.svelte` encuentra match
    - `grep "Copiar al portapapeles" /workspace/descriptor-cifrado/frontend/src/components/HistoryEntryDetailModal.svelte` encuentra match
    - `grep "data:image/png;base64" /workspace/descriptor-cifrado/frontend/src/components/HistoryEntryDetailModal.svelte` encuentra match
    - `grep "export function formatRelative" /workspace/descriptor-cifrado/frontend/src/lib/relativeTime.js` encuentra match
    - `grep "Intl.RelativeTimeFormat" /workspace/descriptor-cifrado/frontend/src/lib/relativeTime.js` encuentra match
    - `grep "'es'" /workspace/descriptor-cifrado/frontend/src/lib/relativeTime.js` encuentra match (locale castellano)
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/TabHistorial.svelte` (no hex hardcoded)
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/HistoryEntryDetailModal.svelte`
    - `! grep -E "#[0-9A-Fa-f]{6}" /workspace/descriptor-cifrado/frontend/src/components/ConfirmDeleteModal.svelte`
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
    - Bundle inicial JS+CSS gzipped <51200 bytes
    - `cargo build -p bed-server` exit code 0
    - `cargo test -p bed-server --test embedded_spa -- --test-threads=1` exit code 0
  </acceptance_criteria>
  <done>TabHistorial monta lista con timestamps relativos + filename mono + size + Ver/Borrar; HistoryEntryDetailModal regenera 3 outputs vía GET /api/history/{id}; ConfirmDeleteModal con default focus en Cancelar dispara DELETE /api/history/{id} y refresca lista; empty state literal del UI-SPEC; App.svelte gating del tabpanel por appState.historyEnabled; build verde; tests embedded_spa siguen pasando.</done>
</task>

</tasks>

<verification>
- `cargo build -p bed-server` exit code 0; `cargo test -p bed-server` todos los tests pasan (round-trip, no-leak, history, embedded_spa)
- `cargo run -p bed-server` arranca y `curl http://127.0.0.1:8080/` devuelve la SPA HTML con `<div id="app">` (UI-01)
- En el navegador, activar toggle "Modo histórico" → la pestaña Historial aparece; sin entradas muestra empty state literal del UI-SPEC
- Cifrar un descriptor con toggle ON → entrada aparece en Historial; click "Ver" abre modal con 3 outputs regenerados; click "Borrar" abre modal de confirmación con foco en Cancelar; confirmar borra y muestra toast "Entrada borrada"
- DevTools Network panel: cero requests externos al cargar `/` (todo embebido vía rust-embed); fonts woff2 sirven con MIME font/woff2 desde /assets/
- Bundle inicial JS+CSS gzipped <50 KB (con todos los componentes + bbqr+qrcode en chunk dinámico separado)
</verification>

<success_criteria>
- UI-01: SPA servida desde el binario vía rust-embed; cero CDN; cero fonts remotas; index.html y assets hashados sirven correctamente con MIME types correctos
- UI-02 (parte Historial): tab funcional, gated por toggle (D-20)
- HIST-02 (parte UI): la entrada aparece en la lista tras cifrar con toggle ON (Plan 02-04 dispara POST, Plan 02-02 persiste, este plan lista)
- HIST-04: GET /api/history se consume y se renderiza con timestamp relativo + tooltip ISO + filename mono (D-21)
- HIST-05: DELETE /api/history/{id} con confirmación destructiva (D-23, D-36)
- HIST-06: UI lista y permite borrar entradas — empty state literal del UI-SPEC cuando vacío (D-24)
- Phase 2 cierra: 9 requirements (UI-01..03, HIST-01..06) cubiertos entre los 6 plans
</success_criteria>

<output>
After completion, create `.planning/phases/02-spa-frontend-history/02-06-SUMMARY.md` describing:
- Versión exacta de rust-embed instalada y feature flag final usado (`axum` o `axum-ex`)
- Tamaño bundle inicial JS+CSS gzipped tras añadir TabHistorial
- Tamaño total del binario `bed-server` en release (`cargo build -p bed-server --release && ls -lh target/release/bed-server`)
- Confirmación del flujo end-to-end smoke-tested: cargar SPA → activar toggle → cifrar → ver historial → ver detalle → borrar entrada
- Tests añadidos (embedded_spa.rs) y resultado
- Confirmación de cero requests externos en el HTML servido (`curl http://127.0.0.1:8080/ | grep -E "https://|googleapis|googleusercontent" || echo "OK: cero externos"`)
- Phase 2 closure: 9/9 requirements cubiertos (matriz por plan)
</output>
</content>
</invoke>