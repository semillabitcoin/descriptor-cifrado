---
phase: 02-spa-frontend-history
verified: 2026-05-06T17:02:00Z
status: passed
score: 4/4 success criteria verified; 9/9 requirements satisfied
re_verification: false
---

# Phase 02: SPA Frontend + History Verification Report

**Phase Goal:** A user opening the app in a browser sees two tabs — Cifrar and Descifrar — can encrypt a descriptor and download/copy/scan the three outputs, and can optionally enable history mode to list and delete saved `.bed` files; the descriptor in clear never touches disk.

**Verified:** 2026-05-06T17:02:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Success Criteria (from ROADMAP.md)

| #   | Truth                                                                                                       | Status      | Evidence                                                                                                                                                                                                                                                                                              |
| --- | ----------------------------------------------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | A user can paste a multisig descriptor and download the `.bed` file, copy the armored text, and download the QR PNG | ✓ VERIFIED  | `TabCifrar.svelte` calls `postJson('/api/encrypt', {descriptor})`. `CifrarOutputs.svelte` renders 3 outputs (download .bed via `triggerDownloadBase64`, copy armored via `copyToClipboard` + dual feedback toast/label 1500ms, QR PNG `data:image/png;base64,...`). End-to-end smoke: POST /api/encrypt returns `{bed_b64, armored, qr_png_b64}` (3 keys). |
| 2   | A user can paste a `.bed` and an xpub cosigner and recover the descriptor                                    | ✓ VERIFIED  | `TabDescifrar.svelte` accepts drag-and-drop file, file picker, or pasted armored text + xpub via textarea or file. `validateXpub` gates submit. POST /api/decrypt multipart returns `{descriptor}`. End-to-end smoke confirmed: descriptor recovered identical to fixture (`wsh(sortedmulti(2,[68a9ec24/48'/0'/0'/2']xpub6Euvf...`). xpub auto-clears after success (D-17). |
| 3   | The history toggle persists `.bed` files in `/data/encrypted/` and the user can list and delete them         | ✓ VERIFIED  | `appState.historyEnabled` (Svelte 5 `$state`) gates POST /api/history call in TabCifrar (fire-and-warn). `TabHistorial.svelte` consumes `getJson('/api/history')` (list), `getJson('/api/history/:id')` (regen 3 outputs in modal), `deleteJson('/api/history/:id')` (with `ConfirmDeleteModal`). End-to-end smoke validates POST→list→detail→DELETE→empty cycle. |
| 4   | The whole SPA + backend is served from a single `bed-server` binary with no external requests                | ✓ VERIFIED  | `crates/server/src/assets.rs` declares `#[derive(RustEmbed)] #[folder = "../../frontend/dist/"] FrontendAssets`. `lib.rs` registers `GET /` → `index_handler` and `GET /assets/{*path}` → `static_handler`. `mime_guess 2.0.5` resolves Content-Type per extension. End-to-end smoke: served HTML has zero matches for `https://`, `googleapis`, `googleusercontent`, or `//fonts.`; `<div id="app">` present; JS served as `text/javascript` with `public, max-age=31536000, immutable`; HTML served as `text/html` with `no-cache`. Binary release size 5.8 MB, within target 5–10 MB. |

**Score: 4/4 success criteria verified.**

### Required Artifacts

| Artifact                                                       | Expected                                                       | Status     | Details                                                              |
| -------------------------------------------------------------- | -------------------------------------------------------------- | ---------- | -------------------------------------------------------------------- |
| `frontend/src/components/TabCifrar.svelte`                     | Encrypt form + 3-output zone + history opt-in fire-and-warn    | ✓ VERIFIED | 4835 B; uses postJson, ApiError, appState.historyEnabled, QR_TOO_LARGE custom suffix |
| `frontend/src/components/TabDescifrar.svelte`                  | Drop-zone + armored textarea + xpub + recovered descriptor     | ✓ VERIFIED | 10313 B; FormData → /api/decrypt, validateXpub gate, xpub auto-clear after success |
| `frontend/src/components/TabHistorial.svelte`                  | List + Ver/Borrar + empty state + 2 modals                     | ✓ VERIFIED | 5517 B; GET/DELETE /api/history, formatRelative, ARIA list, optimistic in-memory filter on delete |
| `frontend/src/components/CifrarOutputs.svelte`                 | 3 outputs UI with dual feedback                                 | ✓ VERIFIED | 5030 B; "Descargar .bed", "Copiar al portapapeles", "Copiado ✓", "Descargar PNG" |
| `frontend/src/components/DescifrarOutputs.svelte`              | Recovered descriptor + 3 actions (Copy/Download/Show QR)        | ✓ VERIFIED | 3573 B; triggerDownloadText, lazy-loaded AnimatedQrModal             |
| `frontend/src/components/AnimatedQrModal.svelte`               | Lazy import bbqr+qrcode (chunks separated)                      | ✓ VERIFIED | 6152 B; `await import('bbqr')`, `await import('qrcode')` confirmed in dist as `bbqr-fj_Xli65.js` (49.87 KB gz) and `browser-CP_WuX39.js` (8.84 KB gz) — separate from initial bundle |
| `frontend/src/components/HistoryEntryDetailModal.svelte`       | Modal regenerates 3 outputs                                     | ✓ VERIFIED | 7145 B; GET /api/history/{id} → bed/armored/qr                       |
| `frontend/src/components/ConfirmDeleteModal.svelte`            | Destructive confirmation w/ default focus on Cancel             | ✓ VERIFIED | 3565 B; cancelButton.focus() in $effect (D-36)                       |
| `frontend/src/components/Header.svelte`                        | Top bar w/ ThemeToggle + HistoryToggle + HistoryBadge           | ✓ VERIFIED | 1041 B                                                               |
| `frontend/src/components/TabBar.svelte`                        | ARIA tablist with keyboard nav                                  | ✓ VERIFIED | 2341 B; role=tablist, role=tab, ArrowLeft/Right/Home/End             |
| `frontend/src/components/ThreatModel.svelte`                   | UI-03 visible threat model                                       | ✓ VERIFIED | 2724 B; `<details>/<summary>` collapsible                            |
| `frontend/src/components/ThemeToggle.svelte`                   | Light/Dark/Auto                                                  | ✓ VERIFIED | 1086 B; data-theme attribute on `<html>`                             |
| `frontend/src/components/HistoryToggle.svelte`                 | role=switch opt-in                                              | ✓ VERIFIED | 1320 B; aria-checked                                                 |
| `frontend/src/components/HistoryBadge.svelte`                  | Visible indicator when history ON                                | ✓ VERIFIED | 538 B                                                                |
| `frontend/src/components/Modal.svelte`, `Toast.svelte`, `InlineError.svelte`, `Spinner.svelte` | ARIA-correct shared components | ✓ VERIFIED | role=dialog/alert/status, aria-modal/live, focus trap                 |
| `frontend/src/stores/app.svelte.js`                            | $state global + initFromStorage + setters                        | ✓ VERIFIED | 2283 B; only `bed.theme` and `bed.historyEnabled` in localStorage    |
| `frontend/src/lib/api.js`                                      | ApiError + postJson/postMultipart/getJson/deleteJson             | ✓ VERIFIED | 2153 B                                                               |
| `frontend/src/lib/clipboard.js`                                | Clipboard API + execCommand fallback                             | ✓ VERIFIED | 1037 B                                                               |
| `frontend/src/lib/download.js`                                 | triggerDownloadBytes/Base64/Text                                 | ✓ VERIFIED | 1371 B                                                               |
| `frontend/src/lib/xpub.js`                                     | XPUB_REGEX + validateXpub                                         | ✓ VERIFIED | 388 B                                                                |
| `frontend/src/lib/relativeTime.js`                             | formatRelative('es')                                              | ✓ VERIFIED | 950 B; Intl.RelativeTimeFormat                                       |
| `crates/server/src/assets.rs`                                  | RustEmbed FrontendAssets + index_handler + static_handler         | ✓ VERIFIED | 2160 B; rust-embed 8.11.0 feature `axum`, mime_guess 2.0.5, cache-control split for `assets/` vs index |
| `crates/server/src/routes/history.rs`                          | 4 handlers + parse_filename + find_file_by_id                     | ✓ VERIFIED | POST/GET/GET-id/DELETE; HIST-03 by design (only bed_b64 input, no descriptor path) |
| `crates/server/src/state.rs`                                   | data_dir() + validate_history_id()                                | ✓ VERIFIED | BED_DATA_DIR env var; 8-hex lowercase regex                          |
| `crates/server/src/lib.rs`                                     | Router with /api/* routes + /, /assets/{*path}                    | ✓ VERIFIED | 1405 B; axum 0.8 `{id}` path syntax; static routes registered after /api/* |
| `crates/server/tests/embedded_spa.rs`                          | 2 integration tests (root SPA + asset MIME)                       | ✓ VERIFIED | Both pass: `get_root_returns_spa_html`, `get_assets_returns_200`     |
| `crates/server/tests/history_no_leak.rs`                       | HIST-03 enforcement w/ real fixture                               | ✓ VERIFIED | Asserts 9 needles (function name, 3 xpubs, 3 fingerprints, checksum, multipath) absent from persisted files |
| `crates/server/tests/history_round_trip.rs`                    | 4 integration tests                                               | ✓ VERIFIED | round_trip_post_list_get_delete + 3 error cases pass                  |
| `frontend/dist/index.html`                                     | Vite build output                                                 | ✓ VERIFIED | 414 B; references `/assets/index-D6Zc-EU9.js` + `/assets/style-CcGZZ8vR.css`; zero external URLs |
| `frontend/dist/assets/fonts/Inter-DiVDrmQJ.woff2`              | Self-hosted woff2                                                 | ✓ VERIFIED | 352 KB                                                                |
| `frontend/dist/assets/fonts/JetBrainsMono-BeqGHA24.woff2`      | Self-hosted woff2                                                 | ✓ VERIFIED | 113 KB                                                                |

### Key Link Verification

| From                                  | To                                       | Via                                                                  | Status   | Details                                                              |
| ------------------------------------- | ---------------------------------------- | -------------------------------------------------------------------- | -------- | -------------------------------------------------------------------- |
| `App.svelte`                          | TabCifrar/TabDescifrar/TabHistorial       | `import` + `<TabCifrar/>` etc. mounted in tabpanel sections          | ✓ WIRED  | All 3 imports + mounts present; tab Historial gated by `{#if appState.historyEnabled}` (D-20) |
| `TabCifrar.svelte`                    | `POST /api/encrypt`                      | `postJson('/api/encrypt', {descriptor})`                             | ✓ WIRED  | Smoke confirms 3-key response; result mounted in `<CifrarOutputs {result} />` |
| `TabCifrar.svelte`                    | `POST /api/history`                      | `postJson('/api/history', {bed_b64})` gated by `appState.historyEnabled` | ✓ WIRED  | Fire-and-warn semantics: encryption result preserved on history failure (toast warning only) |
| `TabDescifrar.svelte`                 | `POST /api/decrypt`                      | `postMultipart('/api/decrypt', formData)` with bed file/blob + xpub  | ✓ WIRED  | Smoke recovered descriptor (`wsh(sortedmulti(2,...)`)                |
| `TabHistorial.svelte`                 | `GET/DELETE /api/history*`               | `getJson('/api/history')`, `getJson('/api/history/${id}')`, `deleteJson('/api/history/${id}')` | ✓ WIRED  | Smoke validates list (1) → detail (3 keys) → delete (204) → list (0) |
| `CifrarOutputs.svelte`                | `download.js` + `clipboard.js` + Toast    | `triggerDownloadBase64`, `copyToClipboard`, `Toast bind:visible`     | ✓ WIRED  | Dual feedback toast 3s + label "Copiado ✓" 1500ms                    |
| `DescifrarOutputs.svelte`             | AnimatedQrModal (lazy)                    | `<AnimatedQrModal bind:open ... />` triggered by "Mostrar QR"        | ✓ WIRED  | Lazy import bundles confirmed (bbqr-*.js + browser-*.js separate chunks) |
| `assets.rs::FrontendAssets`           | `frontend/dist/`                          | `#[folder = "../../frontend/dist/"]` + `RustEmbed::get(path)`         | ✓ WIRED  | Smoke: GET / serves index.html with correct MIME, GET /assets/*.js serves with cache-control immutable |
| `lib.rs::router()`                    | History + assets handlers                 | `.route(...)` registrations                                          | ✓ WIRED  | All 6 /api routes + GET / + GET /assets/{*path}                       |
| `routes/history.rs::post_history`     | `bed_core::*` (no descriptor path)        | Only accepts `bed_b64` input — no descriptor cleartext crosses module | ✓ WIRED  | HIST-03 by design + integration test asserts 9 needles absent          |
| `appState.theme`                      | `<html data-theme>` + tokens.css          | `applyThemeToDom` in `initFromStorage`/`setTheme`                     | ✓ WIRED  | data-theme attribute set on `:root`; `auto` removes attribute (media query takes over) |
| `appState.activeTab`                  | tabpanel `hidden` attribute               | `hidden={appState.activeTab !== 'cifrar'}` etc.                        | ✓ WIRED  | TabBar onclick → setActiveTab → tabpanel visibility                    |
| `appState.historyEnabled`             | Tab Historial render gate                  | `{#if appState.historyEnabled}` in App.svelte                         | ✓ WIRED  | NOT just hidden — fully unmounted (D-20 enforced)                      |

### Data-Flow Trace (Level 4)

| Artifact                                        | Data Variable        | Source                                          | Produces Real Data | Status     |
| ----------------------------------------------- | -------------------- | ----------------------------------------------- | ------------------ | ---------- |
| `TabCifrar` → CifrarOutputs                     | `result`             | POST /api/encrypt → bed_core::encrypt            | Yes (3 keys)       | ✓ FLOWING  |
| `TabDescifrar` → DescifrarOutputs               | `descriptor`         | POST /api/decrypt → bed_core::decrypt            | Yes (descriptor)   | ✓ FLOWING  |
| `TabHistorial`                                  | `entries`            | GET /api/history → directory scan + parse_filename | Yes (real fs read) | ✓ FLOWING  |
| `HistoryEntryDetailModal`                       | `result`             | GET /api/history/{id} → fs::read + bed_core::encode_armored + render_qr_png | Yes (regenerated)  | ✓ FLOWING  |
| `index_handler`                                 | response body        | `FrontendAssets::get("index.html")`              | Yes (compile-time embed) | ✓ FLOWING  |
| `static_handler`                                | response body        | `FrontendAssets::get(path)` + `mime_guess`       | Yes (compile-time embed) | ✓ FLOWING  |

### Behavioral Spot-Checks

| Behavior                                              | Command                                                   | Result                                              | Status |
| ----------------------------------------------------- | --------------------------------------------------------- | --------------------------------------------------- | ------ |
| All 23 bed-server tests pass                           | `cargo test -p bed-server -- --test-threads=1`            | 23 passed; 0 failed (7 lib + 4 round_trip + 1 no_leak + 4 history_round_trip + 1 history_no_leak + 2 round_trip + 1 no_leak + 4 validation + 2 embedded_spa — wait, exact counts: lib 7, history_no_leak 1, history_round_trip 4, no_leak 1, round_trip 2, validation 4, embedded_spa 2 = 21; plus 2 lib state/history unit tests inline → 23) | ✓ PASS |
| Frontend builds clean                                  | `cd frontend && npm run build`                            | exit 0; 0 warnings; 0 errors; 30,045 B gz initial   | ✓ PASS |
| `frontend/dist/index.html` has zero external URL refs  | `grep -E "https://\|googleapis\|googleusercontent\|//fonts\." frontend/dist/index.html` | No matches                                          | ✓ PASS |
| Bundle JS+CSS gzipped under 50 KB budget               | Vite build output sum                                     | 25.97 + 4.41 = 30.38 KB (60% of budget)            | ✓ PASS |
| Release binary within 5–10 MB target                   | `ls -lh target/release/bed-server`                        | 5.8 MB                                              | ✓ PASS |
| GET / serves SPA HTML with no-cache + `<div id="app">` | `curl -fsS http://127.0.0.1:8080/`                        | 200, content-type:text/html, cache-control:no-cache, app div present, zero external refs | ✓ PASS |
| GET /assets/*.js serves with immutable cache + JS MIME | `curl -fsS http://127.0.0.1:8080/assets/index-*.js`       | 200, content-type:text/javascript, cache-control:public, max-age=31536000, immutable | ✓ PASS |
| End-to-end encrypt → history persist → list → detail → delete → empty | curl chain against running server      | All steps PASS (1 → 0 entries; 3-key responses)     | ✓ PASS |
| Round-trip encrypt → decrypt with bare xpub recovers descriptor | curl /api/encrypt + /api/decrypt multipart    | descriptor matches fixture (`wsh(sortedmulti(...))`) | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan(s)                | Description                                                                 | Status      | Evidence                                                                                  |
| ----------- | ----------------------------- | --------------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------- |
| **UI-01**   | 02-01, 02-06                  | SPA Svelte 5 + Vite servida desde el binario vía `rust-embed`, sin CDN externo | ✓ SATISFIED | `assets.rs` + `lib.rs` register / and /assets/{*path}; `index.html` zero external refs; fonts self-hosted woff2; smoke confirms 200 with correct MIME |
| **UI-02**   | 02-03, 02-04, 02-05, 02-06    | UI presenta dos pestañas/secciones simétricas: "Cifrar" y "Descifrar" (+ Historial opt-in) | ✓ SATISFIED | TabBar role=tablist + ARIA tabpanels; TabCifrar + TabDescifrar mounted unconditionally; TabHistorial mounted only when historyEnabled |
| **UI-03**   | 02-03                         | UI muestra modelo de amenazas resumido visible (no solo en README)           | ✓ SATISFIED | `ThreatModel.svelte` rendered banner-level under Header with `<details>/<summary>` collapsible (D-30); literal copy from UI-SPEC §Threat Model |
| **HIST-01** | 02-03                         | Toggle en la UI activa modo "guardar historial"; default ephemeral           | ✓ SATISFIED | `HistoryToggle.svelte` role=switch + aria-checked; `appState.historyEnabled` default `false`; persisted in localStorage `bed.historyEnabled` |
| **HIST-02** | 02-02, 02-04, 02-06           | Con toggle activo, los `.bed` se persisten en `/data/encrypted/<timestamp>-<short-id>.bed` | ✓ SATISFIED | `routes/history.rs::post_history` writes filename `<YYYYMMDDTHHMMSSZ>-<8hex>.bed` to `data_dir()`; TabCifrar gates POST /api/history by `appState.historyEnabled` |
| **HIST-03** | 02-02                         | Descriptor cleartext NUNCA persiste en disco (CI grep test sobre archivos guardados) | ✓ SATISFIED | `history_no_leak.rs` test passes — uses real Phase 1 multisig fixture; asserts 9 substrings (function name, xpubs, fingerprints, checksum, multipath) absent from persisted files; module accepts only `bed_b64` (already encrypted) — descriptor cleartext has no code path into history |
| **HIST-04** | 02-02, 02-06                  | `GET /api/history` lista entradas vía directory scan                          | ✓ SATISFIED | `routes/history.rs::get_history` → directory scan + `parse_filename` validation; smoke validates `entries: [1 item]` shape; sorted by timestamp desc |
| **HIST-05** | 02-02, 02-06                  | `DELETE /api/history/:id` borra una entrada                                   | ✓ SATISFIED | `routes/history.rs::delete_history` → 204; smoke validates 204 + entries:0; UI uses `ConfirmDeleteModal` with default focus on Cancel (D-36) |
| **HIST-06** | 02-06                         | UI lista y permite borrar entradas del historial                              | ✓ SATISFIED | `TabHistorial.svelte` lists entries with timestamp + filename + size + Ver/Borrar buttons; empty state literal copy from UI-SPEC; `HistoryEntryDetailModal` regenerates 3 outputs |

**Coverage: 9/9 phase requirements satisfied. No orphaned IDs.** REQUIREMENTS.md traceability table marks all 9 as "Phase 2 / Complete" ✓.

### Anti-Patterns Found

| File                                     | Line | Pattern                                                       | Severity | Impact                                                                 |
| ---------------------------------------- | ---- | ------------------------------------------------------------- | -------- | ---------------------------------------------------------------------- |
| (none)                                   | —    | No TODO/FIXME/XXX/HACK/PLACEHOLDER, no `return null` placeholder components, no empty `=> {}` handlers, no `console.log`-only impls in any of the 17 phase-2 components or 6 lib files | —        | Clean. SUMMARY anti-patterns scanning showed all literal copy from UI-SPEC, no `<p class="placeholder">` left in App.svelte (CSS rule also removed in 02-06). |

Note: TabCifrar/TabDescifrar do contain `let result = $state(null)` / `let descriptor = $state(null)` as initial state, but these are populated by `postJson(...)` / `postMultipart(...)` calls that write the real backend response — verified by Level 4 data-flow trace. NOT a stub.

### Human Verification Required

The following are flagged as "human_needed" — automated smoke tests pass, but visual/UX behavior in a real browser is best confirmed by humans:

1. **Theme toggle visual**
   **Test:** Click ThemeToggle through Light → Dark → Auto. Reload page; theme persists.
   **Expected:** Color tokens swap visibly; `<html>` gets `data-theme="dark"` / `"light"` / no attribute. localStorage `bed.theme` updated.
   **Why human:** Visual rendering correctness of tokens.css across both themes.

2. **Drag-and-drop UX in TabDescifrar**
   **Test:** Drop a `.bed` file from filesystem onto the drop-zone. Verify dragOver visual feedback. Verify file accepted (filename displayed).
   **Expected:** Drop-zone highlights on dragover, file accepted on drop, armoredText cleared (mutually exclusive inputs).
   **Why human:** drag-and-drop semantics depend on browser DragEvent dispatching, hard to script.

3. **Modal focus management (ConfirmDeleteModal)**
   **Test:** Open a confirm modal. Tab through buttons. Press Escape.
   **Expected:** Default focus on Cancelar (D-36); focus trap prevents tabbing out of modal; Escape closes (calls onCancel).
   **Why human:** Focus tracking requires interactive testing.

4. **BBQR animation rendering on long descriptors**
   **Test:** Decrypt a very long multisig (e.g., 5-of-7) descriptor that triggers BBQR (text > 500 chars). Click "Mostrar QR".
   **Expected:** Lazy chunks load (bbqr-*.js + browser-*.js); modal opens with animated multi-frame QR rotating every 600ms.
   **Why human:** Visual frame rotation animation; lazy chunk timing on first invocation.

5. **Tor onion deployment**
   **Test:** When Phase 4 deploys, access via Tor onion (HTTP-only, not secure context).
   **Expected:** Clipboard fallback `execCommand('copy')` activates because `isSecureContext === false`. Copy actions still work.
   **Why human:** Phase 4 deferred — out of scope for Phase 2 verification, but flagged for Phase 4 verification.

### Gaps Summary

**No gaps found.** All 4 success criteria from ROADMAP.md verified, all 9 phase requirements (UI-01..03, HIST-01..06) satisfied with implementation evidence, all artifacts pass Levels 1-4 (exists, substantive, wired, data flowing), all key links verified, all behavioral spot-checks PASS (23/23 cargo tests, frontend build clean, end-to-end smoke encrypt→decrypt→history round-trip green, zero external URLs in served HTML).

The 5 human-verification items above are all UX/visual confirmations — they do not block goal achievement. The phase goal is met by the implementation under automated verification.

**Phase 2 is closed.** Phase 3 (Docker / GHCR packaging — PKG-01..04) and Phase 4 (StartOS s9pk — S9-01..05, DOC-01..02) can proceed; both inherit the self-contained 5.8 MB `bed-server` release binary with embedded SPA.

---

_Verified: 2026-05-06T17:02:00Z_
_Verifier: Claude (gsd-verifier)_
