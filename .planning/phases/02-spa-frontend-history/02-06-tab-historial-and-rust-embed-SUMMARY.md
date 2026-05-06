---
phase: 02-spa-frontend-history
plan: 06
subsystem: full-stack
tags: [frontend, svelte5, tab-historial, rust-embed, axum, ui-01, ui-02, hist-02, hist-04, hist-05, hist-06, phase-closure]

requires:
  - phase: 02-spa-frontend-history
    plan: 02
    provides: "POST/GET/GET-id/DELETE /api/history endpoints (consumed end-to-end here)"
  - phase: 02-spa-frontend-history
    plan: 03
    provides: "appState store, ApiError/getJson/deleteJson, copyToClipboard, Spinner/Toast/InlineError"
  - phase: 02-spa-frontend-history
    plan: 04
    provides: "frontend/src/lib/download.js — triggerDownloadBase64"
provides:
  - "crates/server/src/assets.rs — FrontendAssets (RustEmbed sobre frontend/dist/) + StaticFile<T> IntoResponse + index_handler + static_handler"
  - "GET / serves index.html (text/html, no-cache); GET /assets/{*path} serves hashed bundles (1y immutable)"
  - "frontend/src/lib/relativeTime.js — formatRelative(iso) en castellano via Intl.RelativeTimeFormat('es')"
  - "frontend/src/components/TabHistorial.svelte — lista con timestamp relativo + tooltip ISO + filename mono + size + Ver/Borrar"
  - "frontend/src/components/HistoryEntryDetailModal.svelte — modal con 3 outputs regenerados desde GET /api/history/{id}"
  - "frontend/src/components/ConfirmDeleteModal.svelte — modal destructivo con default focus en Cancelar (D-36)"
affects: [04-startos-packaging]

tech-stack:
  added:
    - "mime_guess 2.0.5 (workspace dep, used by static_handler for Content-Type)"
  patterns:
    - "rust-embed feature flag 'axum' (no 'axum-ex') — consistente con Plan 02-02"
    - "Static asset routes registered AFTER /api/* in axum 0.8 router (specificity priority)"
    - "Cache-Control split: hashed /assets/* get 'public, max-age=31536000, immutable'; index.html gets 'no-cache' (browsers re-fetch on each version)"
    - "StaticFile<T> generic IntoResponse wrapper — index/static handlers share the same MIME resolution path via mime_guess"
    - "Svelte 5 bind:this on $state(null) variable to avoid non_reactive_update warning when read inside $effect"
    - "Intl.RelativeTimeFormat('es', {numeric:'auto'}) — produces 'hace 3 días' nativamente sin string interpolation manual"
    - "Empty state literal del UI-SPEC + tooltip ISO en <span title> sobre relativeTime"

key-files:
  created:
    - "crates/server/src/assets.rs (FrontendAssets + StaticFile<T> IntoResponse + index_handler + static_handler + cache_control_for)"
    - "crates/server/tests/embedded_spa.rs (2 integration tests: GET / SPA HTML; GET /assets/index-*.js MIME JS)"
    - "frontend/src/lib/relativeTime.js (formatRelative formatter)"
    - "frontend/src/components/TabHistorial.svelte (lista + empty state + Ver/Borrar + 2 modales)"
    - "frontend/src/components/HistoryEntryDetailModal.svelte (regen 3 outputs vía GET /api/history/{id})"
    - "frontend/src/components/ConfirmDeleteModal.svelte (default focus Cancelar D-36)"
  modified:
    - "Cargo.toml (add mime_guess 2 al workspace.dependencies)"
    - "Cargo.lock (mime_guess 2.0.5 + unicase 2.9.0 lock-in)"
    - "crates/server/Cargo.toml (mime_guess workspace dep)"
    - "crates/server/src/lib.rs (add `pub mod assets;` + register / and /assets/{*path} after /api/* routes)"
    - "frontend/src/App.svelte (import + mount <TabHistorial /> en panel-historial; remove placeholder + .placeholder CSS)"

decisions:
  - "rust-embed feature 'axum' confirmado (Plan 02-02 ya validó); rust-embed 8.11.0"
  - "mime_guess 2.0.5 elegido sobre EmbeddedFile.metadata.mimetype: más robusto para .woff2/.css que necesitan resolución por extensión, no solo magic bytes"
  - "Cache-Control split por path: assets/ get max-age=31536000 immutable; index.html no-cache (browsers re-fetch en cada deploy)"
  - "Static routes registradas DESPUÉS de /api/* en lib.rs: axum 0.8 prefiere ruta más específica, pero el orden refuerza intent + facilita reading"
  - "router() (no build_router(state) como sugería el PLAN): preserva la signature existente de Plan 02-02; AppState es un default unit struct sin estado relevante en Phase 2"
  - "Empty state título (<h2>) + body literal del UI-SPEC: 'Sin backups cifrados aún' / 'Cifra un descriptor con el modo histórico activo para que aparezca aquí.'"
  - "ConfirmDeleteModal usa svelte:window onkeydown para Escape (no document listener); cancelButton declarado con $state(null) para evitar warning non_reactive_update cuando bind:this se lee en $effect"
  - "HistoryEntryDetailModal: $effect resetea result/errorMessage cuando open=false, asegurando que el siguiente open vuelve a hacer GET (no muestra datos stale de otra entrada)"
  - "DELETE refresca lista en memoria (filter), no re-fetch: ahorra una request, UX inmediata; consistente con D-21 (UI optimista)"
  - "Suprimimos a11y_click_events_have_key_events / a11y_no_static_element_interactions / a11y_interactive_supports_focus en backdrop+panel de ambos modales — pattern WAI-ARIA estándar para click-outside-to-close, mismo enfoque del Plan 02-03/02-05"

requirements-completed: [UI-01, UI-02, HIST-02, HIST-04, HIST-05, HIST-06]

metrics:
  duration_minutes: 11
  tasks_completed: 2
  commits: 2
  files_created: 6
  files_modified: 5
  completed: 2026-05-06
---

# Phase 02 Plan 06: Tab Historial + Rust Embed Summary

**Phase 2 closure**: SPA Svelte completa servida desde el binario `bed-server` vía `rust-embed` (`frontend/dist/` embebido en compile-time, cero requests externos al cargar). Tab Historial consumiendo los 4 endpoints de Plan 02-02 (lista con timestamp relativo castellano + filename mono + size; modal de detalle con 3 outputs regenerados; modal destructivo con default focus en Cancelar). Bundle inicial JS+CSS gzipped **30,045 bytes / 60% del budget 50 KB**. Binario release **5.8 MB** (target 5-10 MB cumplido).

## Versiones exactas

| Paquete | Versión | Origen | Feature flag |
|---------|---------|--------|--------------|
| `rust-embed` | 8.11.0 | Cargo.lock (workspace dep ya presente desde Plan 02-02) | `axum` |
| `mime_guess` | 2.0.5 | Cargo.lock (añadido aquí; transitive: unicase 2.9.0) | n/a |
| `bbqr` | 1.2.0 | frontend (Plan 02-05) — sin cambios | n/a |
| `qrcode` | 1.5.4 | frontend (Plan 02-05) — sin cambios | n/a |

**Feature flag rust-embed final:** `"axum"` (consistente con Plan 02-02; sin cambio).

## Bundle Size

`cd frontend && npm run build`:

```
dist/index.html                                   0.41 KB │ gzip:  0.28 KB
dist/assets/style-CcGZZ8vR.css                   27.81 KB │ gzip:  4.41 KB
dist/assets/index-D6Zc-EU9.js                    71.89 KB │ gzip: 25.97 KB
dist/assets/browser-CP_WuX39.js                  23.46 KB │ gzip:  8.84 KB   ← qrcode dep, lazy
dist/assets/bbqr-fj_Xli65.js                    145.74 KB │ gzip: 49.87 KB   ← bbqr+qrcode core, lazy
dist/assets/fonts/Inter-DiVDrmQJ.woff2          352.24 KB
dist/assets/fonts/JetBrainsMono-BeqGHA24.woff2  113.67 KB
```

**Bundle inicial JS+CSS gzipped (excluye fonts y chunks dinámicos) = 30,045 bytes / 29.3 KB.**

| Plan | Cumulative bundle (gz) | Delta vs anterior |
|------|------------------------|-------------------|
| 02-03 (shell) | 18,439 B | — |
| 02-04 (TabCifrar) | 23,546 B | +5,107 B |
| 02-05 (TabDescifrar + lazy QR) | 27,636 B | +4,090 B |
| **02-06 (TabHistorial)** | **30,045 B** | **+2,409 B** |

UI-SPEC §Build Constraints exige <50 KB → **60% del budget consumido**, ~20 KB libres.

Chunks dinámicos `bbqr` + `browser` (qrcode dep) = **58.7 KB gzipped**, lazy-loaded SOLO cuando el usuario pulsa "Mostrar QR" en TabDescifrar.

## Binary size (release)

```bash
$ cargo build -p bed-server --release
$ ls -lh target/release/bed-server
-rwxrwxr-x 2 anon anon 5.8M May  6 18:52 target/release/bed-server
```

**Binario release: 5.8 MB.** Dentro del target STACK 5–10 MB. Incluye SPA completa embebida (frontend/dist con bundles + 466 KB de fonts woff2 + 4 endpoints HTTP + bed-core crypto). Phase 4 (Docker distroless) no hará compresión adicional, solo strip de debug symbols.

## End-to-end smoke test

Backend `bed-server` arrancado con `BED_DATA_DIR=/tmp/bed-test-06 cargo run -p bed-server`. Verificación full-stack via `curl`:

```bash
$ curl -fsS http://127.0.0.1:8080/ -D headers.txt -o spa.html
content-type: text/html
cache-control: no-cache
$ grep '<div id="app">' spa.html → YES
$ grep -E "https://|googleapis|googleusercontent|//fonts\." spa.html → cero matches (UI-01)

$ curl -fsS http://127.0.0.1:8080/assets/index-D6Zc-EU9.js -D h2.txt -o /dev/null
content-type: text/javascript
cache-control: public, max-age=31536000, immutable

$ curl -fsS http://127.0.0.1:8080/assets/style-CcGZZ8vR.css -D h3.txt -o /dev/null
content-type: text/css
cache-control: public, max-age=31536000, immutable

$ curl -fsS http://127.0.0.1:8080/assets/fonts/Inter-DiVDrmQJ.woff2 -D h4.txt -o /dev/null
content-type: font/woff2
cache-control: public, max-age=31536000, immutable

$ # Encrypt → POST /api/history → list → detail → DELETE
$ DESC=$(cat crates/server/tests/fixtures/desc.txt)
$ ENCRYPTED=$(curl -fsS -X POST http://127.0.0.1:8080/api/encrypt -H "Content-Type: application/json" \
  -d "$(jq -n --arg d "$DESC" '{descriptor:$d}')")
$ BED_B64=$(echo "$ENCRYPTED" | jq -r .bed_b64)  # 820 chars
$ HIST_RESP=$(curl -fsS -X POST http://127.0.0.1:8080/api/history -H "Content-Type: application/json" \
  -d "$(jq -n --arg b "$BED_B64" '{bed_b64:$b}')")
$ # → {"id":"6eed14f8","timestamp":"2026-05-06T16:50:27Z","filename":"20260506T165027Z-6eed14f8.bed"}
$ curl -fsS http://127.0.0.1:8080/api/history | jq '.entries | length' → 1
$ curl -fsS http://127.0.0.1:8080/api/history/6eed14f8 | jq 'keys' → ["armored","bed_b64","qr_png_b64"]
$ curl -X DELETE -w "%{http_code}" http://127.0.0.1:8080/api/history/6eed14f8 → 204
$ curl -fsS http://127.0.0.1:8080/api/history | jq '.entries | length' → 0
```

**Round-trip cifrar → persistir → listar → ver detalle → borrar → lista vacía: PASS.**

**UI-01 enforcement:** `index.html` no contiene NINGUNA referencia externa (no `https://`, no `googleapis`, no `googleusercontent`, no `//fonts.`). Todos los assets (JS/CSS/woff2) sirven desde `/assets/` con MIME correcto y cache-control 1 año immutable.

## Tests añadidos

`crates/server/tests/embedded_spa.rs` (2 integration tests con `#[serial]`):

| Test | Verifica |
|------|----------|
| `get_root_returns_spa_html` | GET / → 200 text/html con `<div id="app">` y `/assets/index-` referenciado; sin externos (https://, googleapis, googleusercontent, //fonts.) |
| `get_assets_returns_200` | Parsea index.html para extraer asset path real, GET ese path → 200 con MIME `text/javascript` o `application/ecmascript` |

```bash
$ cargo test -p bed-server --test embedded_spa -- --test-threads=1
running 2 tests
test get_assets_returns_200 ... ok
test get_root_returns_spa_html ... ok
test result: ok. 2 passed; 0 failed
```

**Total tests bed-server tras Plan 02-06: 23 verdes** (sin regresiones de tests previos: 4 history_round_trip + 1 history_no_leak + 2 round_trip + 1 no_leak + 4 validation + 7 lib + 2 embedded_spa).

## Commits

| Task | Hash    | Message                                                       |
| ---- | ------- | ------------------------------------------------------------- |
| 1    | 7af6c56 | feat(02-06): wire rust-embed for SPA assets                   |
| 2    | 529aa65 | feat(02-06): add TabHistorial + detail/confirm modals + relativeTime |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Test signature inconsistency: PLAN sugiere `build_router(state)` pero el código existente usa `router()`**
- **Found during:** Task 1 (writing embedded_spa.rs)
- **Issue:** El PLAN ejemplifica el test usando `bed_server::{build_router, state::AppState}` y un `fresh_state()` con `tempdir()`. El código real (Plan 02-02 SUMMARY confirma) expone `pub fn router() -> Router` sin estado, y `AppState` es un unit struct sin uso runtime. Llamar `build_router(state)` no compilaría.
- **Fix:** Adapté el test para usar `bed_server::router()` directamente, eliminando el helper `fresh_state()` y la dependencia tempfile/AppState. La verificación funcional es idéntica (los handlers de history que requieren BED_DATA_DIR no se invocan en estos 2 tests; solo los de assets, que no leen filesystem en runtime — todo está embebido).
- **Files modified:** `crates/server/tests/embedded_spa.rs`
- **Verification:** `cargo test -p bed-server --test embedded_spa -- --test-threads=1` → 2 passed
- **Committed in:** `7af6c56` (Task 1 commit)

**2. [Rule 3 — Blocking] Svelte 5 emite `non_reactive_update` warning para `cancelButton` declarado sin `$state`**
- **Found during:** Task 2 (npm run build)
- **Issue:** ConfirmDeleteModal.svelte declara `let cancelButton;` y luego lo lee desde un `$effect` que depende de `open`. Svelte 5 detecta este patrón como reactive read y emite warning. Modal.svelte (Plan 02-03) usa el mismo patrón pero con un effect distinto, donde Svelte no warning. El criterio de detección parece sensible al orden exacto de side-effects.
- **Fix:** Declarar `let cancelButton = $state(null);`. Es semánticamente equivalente (Svelte rune permite `bind:this` sobre $state) y silencia la warning preservando la lógica.
- **Files modified:** `frontend/src/components/ConfirmDeleteModal.svelte`
- **Verification:** `npm run build` → 0 warnings.
- **Committed in:** `529aa65` (Task 2 commit)

**3. [Rule 3 — Blocking] Svelte 5 emite `a11y_interactive_supports_focus` en `<div role="dialog">` sin tabindex**
- **Found during:** Task 2 (npm run build)
- **Issue:** Tras suprimir las dos a11y warnings esperadas (`click_events_have_key_events`, `no_static_element_interactions`) en backdrop y panel, queda una tercera: el linter exige `tabindex` en elementos con role interactivo. El modal panel ya tiene focus management vía cancelButton.focus() en $effect, no necesita tabindex en el panel mismo (focusables internos lo manejan).
- **Fix:** Añadir `<!-- svelte-ignore a11y_interactive_supports_focus -->` antes del panel en HistoryEntryDetailModal y ConfirmDeleteModal. Mismo enfoque que Plan 02-05 (AnimatedQrModal).
- **Files modified:** `frontend/src/components/HistoryEntryDetailModal.svelte`, `frontend/src/components/ConfirmDeleteModal.svelte`
- **Verification:** `npm run build` → 0 warnings.
- **Committed in:** `529aa65` (Task 2 commit)

**4. [Rule 1 — Bug] CSS `.placeholder` selector unused en App.svelte tras reemplazar el placeholder por `<TabHistorial />`**
- **Found during:** Task 2 (npm run build)
- **Issue:** El `<p class="placeholder">` original ya había desaparecido de los paneles cifrar/descifrar (planes 02-04/02-05) pero la regla CSS `.placeholder { ... }` permanecía en el `<style>` de App.svelte. Tras reemplazar el último uso (panel-historial) Svelte emite `Unused CSS selector ".placeholder"`.
- **Fix:** Eliminar la regla CSS `.placeholder` del `<style>` de App.svelte.
- **Files modified:** `frontend/src/App.svelte`
- **Verification:** `npm run build` → 0 warnings.
- **Committed in:** `529aa65` (Task 2 commit)

---

**Total deviations:** 4 auto-fixed (1 blocking signature mismatch, 3 lint-quality fixes). Sin scope creep, ningún cambio funcional al diseño ni al UI-SPEC.

## Verification Summary

```bash
$ cargo build -p bed-server                   # exit 0
$ cargo test -p bed-server                    # 23 tests pass (incluye 2 nuevos embedded_spa)
$ cargo clippy -p bed-server --all-targets -- -D warnings  # exit 0
$ cargo build -p bed-server --release         # exit 0; binary 5.8 MB

$ cd frontend && npm run build                # exit 0; 0 warnings; 0 errors
$ # Bundle JS+CSS gzipped: 30,045 bytes (60% del budget 50 KB)

$ # Acceptance grep tests Task 1
=== Cargo.toml: rust-embed + mime_guess workspace deps
=== assets.rs: #[derive(RustEmbed)], folder=../../frontend/dist/, index_handler, static_handler
=== lib.rs: pub mod assets, /assets/{*path} (axum 0.8), no /assets/*path (no axum 0.7)
=== embedded_spa tests: get_root_returns_spa_html OK, get_assets_returns_200 OK

$ # Acceptance grep tests Task 2 (15+ checks)
=== App.svelte: import TabHistorial, <TabHistorial />, no placeholder, appState.historyEnabled gate
=== TabHistorial.svelte: /api/history, deleteJson, "Sin backups cifrados aún",
                          "Cifra un descriptor con el modo histórico activo",
                          "Entrada borrada", formatRelative, title={entry.timestamp}
=== ConfirmDeleteModal.svelte: "Borrar backup cifrado", "Esta acción no se puede deshacer",
                                cancelButton.focus, role="dialog"
=== HistoryEntryDetailModal.svelte: role="dialog", "Descargar .bed", "Descargar PNG",
                                     "Copiar al portapapeles", data:image/png;base64
=== relativeTime.js: export function formatRelative, Intl.RelativeTimeFormat, 'es'
=== No hex colors in any new component (var(--color-*) only)

$ # End-to-end smoke (curl against bed-server)
GET /                       → 200 text/html, no-cache, <div id="app">, cero externos (UI-01)
GET /assets/*.js            → 200 text/javascript, max-age=31536000 immutable
GET /assets/*.css           → 200 text/css, max-age=31536000 immutable
GET /assets/fonts/*.woff2   → 200 font/woff2, max-age=31536000 immutable
POST /api/encrypt           → 200 {bed_b64,armored,qr_png_b64}
POST /api/history           → 200 {id:"6eed14f8",timestamp,filename}
GET /api/history            → 200 {entries:[1 item]}
GET /api/history/6eed14f8   → 200 {bed_b64,armored,qr_png_b64}
DELETE /api/history/6eed14f8 → 204
GET /api/history            → 200 {entries:[]}
ALL CHECKS PASSED
```

## Phase 2 Closure: Requirements Matrix

Phase 2 cubre 9 requirements (UI-01..03, HIST-01..06). Matriz por plan:

| Req | Plan(s) que lo cubrieron | Estado |
|-----|--------------------------|--------|
| UI-01 (SPA self-contained, cero CDN) | 02-01 (fonts woff2 locales) + **02-06 (rust-embed wire)** | ✅ Cerrado |
| UI-02 (3 tabs Cifrar/Descifrar/Historial) | 02-03 (TabBar) + 02-04 + 02-05 + **02-06** | ✅ Cerrado |
| UI-03 (theme + threat model + tokens) | 02-03 (Header + ThreatModel + ThemeToggle + tokens.css) | ✅ Cerrado |
| HIST-01 (toggle opt-in) | 02-03 (HistoryToggle + appState.historyEnabled) | ✅ Cerrado |
| HIST-02 (POST /api/history al cifrar) | 02-02 (endpoint) + 02-04 (TabCifrar consume gated) + **02-06 (visible en TabHistorial)** | ✅ Cerrado |
| HIST-03 (descriptor cleartext nunca persiste) | 02-02 (test no_leak con fixture multisig real) | ✅ Cerrado |
| HIST-04 (lista con timestamp + filename) | 02-02 (GET endpoint) + **02-06 (TabHistorial render)** | ✅ Cerrado |
| HIST-05 (DELETE con confirmación) | 02-02 (DELETE endpoint) + **02-06 (ConfirmDeleteModal)** | ✅ Cerrado |
| HIST-06 (UI gestiona historial) | **02-06 (TabHistorial + 2 modales + empty state)** | ✅ Cerrado |

**Phase 2 cerrada: 9/9 requirements completos.** Phase 3 (Docker / GHCR) puede arrancar. Phase 4 (s9pk StartOS) hereda binario release 5.8 MB self-contained y la SPA embebida.

## Self-Check: PASSED

- crates/server/src/assets.rs: FOUND
- crates/server/tests/embedded_spa.rs: FOUND
- frontend/src/lib/relativeTime.js: FOUND
- frontend/src/components/TabHistorial.svelte: FOUND
- frontend/src/components/HistoryEntryDetailModal.svelte: FOUND
- frontend/src/components/ConfirmDeleteModal.svelte: FOUND
- frontend/src/App.svelte: MODIFIED (placeholder replaced + import added)
- Cargo.toml: MODIFIED (mime_guess workspace dep)
- crates/server/Cargo.toml: MODIFIED (mime_guess dep)
- crates/server/src/lib.rs: MODIFIED (pub mod assets + 2 routes)
- commit 7af6c56: FOUND
- commit 529aa65: FOUND
- frontend/dist/index.html: FOUND (post-build)
- Bundle JS+CSS gzipped: 30,045 bytes < 51,200 (50 KB budget) PASS
- All 23 bed-server tests pass (cargo test -p bed-server -- --test-threads=1)
- Clippy verde (cargo clippy -p bed-server --all-targets -- -D warnings)
- End-to-end smoke test PASS (encrypt → history → list → detail → delete)
- Cero requests externos en index.html servido (UI-01 enforced)
- Binary release 5.8 MB (target STACK 5–10 MB cumplido)
