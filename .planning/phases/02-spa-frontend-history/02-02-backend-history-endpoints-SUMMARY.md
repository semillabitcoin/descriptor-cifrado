---
phase: 02-spa-frontend-history
plan: 02
subsystem: api
tags: [axum, history, persistence, base64, uuid, time, hist-02, hist-03, hist-04, hist-05]

requires:
  - phase: 01-crypto-core-http-api
    provides: bed_core::encode_armored, bed_core::render_qr_png, AppError envelope, /api/encrypt + /api/decrypt
provides:
  - "POST /api/history endpoint (persiste .bed cifrado en BED_DATA_DIR)"
  - "GET /api/history endpoint (directory scan, ordenado timestamp desc)"
  - "GET /api/history/{id} endpoint (regenera bed_b64 + armored + qr_png_b64)"
  - "DELETE /api/history/{id} endpoint (204 / 404 / 422)"
  - "AppError variants HistoryNotFound, HistoryWriteFailed, HistoryInvalidId"
  - "state::data_dir() resuelve BED_DATA_DIR env var"
  - "state::validate_history_id() anti path traversal"
  - "Filename format <YYYYMMDDTHHMMSSZ>-<8hex>.bed (sortable)"
affects: [02-06-tab-historial-and-rust-embed, 04-startos-packaging]

tech-stack:
  added: [rust-embed 8.11.0, uuid 1.23.1, time 0.3.47, tempfile 3.27.0, serial_test 3.4.0]
  patterns:
    - "Directory scan persistence (no DB): timestamp-prefixed filenames son sortable lexicograficamente"
    - "Anti path traversal: validate_history_id como guard único en endpoints id-aware"
    - "HIST-03 by design: POST acepta solo bed_b64 cifrado, descriptor cleartext nunca cruza el módulo"
    - "BED_DATA_DIR env var permite tempfile en tests sin colisionar con /data/encrypted"
    - "serial_test::serial en integration tests para evitar race condition en env var"

key-files:
  created:
    - "crates/server/src/routes/history.rs (4 handlers + filename helpers + parse_filename)"
    - "crates/server/tests/history_round_trip.rs (4 integration tests)"
    - "crates/server/tests/history_no_leak.rs (HIST-03 enforcement test)"
  modified:
    - "Cargo.toml (workspace deps: rust-embed, uuid, time; tokio fs feature)"
    - "crates/server/Cargo.toml (deps + dev-deps tempfile/serial_test)"
    - "crates/server/src/error.rs (3 variants nuevas con status codes 404/500/422)"
    - "crates/server/src/state.rs (data_dir + validate_history_id + 5 unit tests)"
    - "crates/server/src/lib.rs (router con 6 rutas; axum 0.8 {id} syntax)"
    - "crates/server/src/routes/mod.rs (pub mod history)"

key-decisions:
  - "Directory scan en lugar de redb: Phase 2 v1 solo necesita list/delete; redb diferido hasta queries más ricas"
  - "rust-embed feature flag: 'axum' (no 'axum-ex') - validado en cargo build con rust-embed 8.11.0"
  - "bed_core::render_qr_png recibe &str (armored), no &[u8] - confirmado leyendo crates/core/src/qr.rs"
  - "axum 0.8 path syntax {id} (no :id) - Cargo.lock muestra axum 0.8.9"
  - "tempfile::tempdir() + serial_test en integration tests evita race en BED_DATA_DIR env var"
  - "Inline test modules requieren #![allow(clippy::unwrap_used + expect_used)] adicional al panic allow"

patterns-established:
  - "Filename parser <YYYYMMDDTHHMMSSZ>-<8hex>.bed: validate_history_id + length checks + digit checks"
  - "find_file_by_id: directory scan con suffix match + parse_filename validation"
  - "AppError::HistoryWriteFailed para fs::write fail, AppError::Internal para fs::read fail (post-find)"

requirements-completed: [HIST-02, HIST-03, HIST-04, HIST-05]

duration: 6min
completed: 2026-05-06
---

# Phase 02 Plan 02: Backend History Endpoints Summary

**4 endpoints axum HTTP (POST/GET/GET-id/DELETE) sobre directory scan de BED_DATA_DIR con filename sortable + anti path traversal + HIST-03 enforced by design**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-05-06T13:17:30Z
- **Completed:** 2026-05-06T13:23:00Z (aprox)
- **Tasks:** 3 (todas TDD: RED → GREEN combinado por brevedad de unit tests)
- **Files modified:** 10 (3 creados, 7 editados)
- **Tests añadidos:** 12 (5 lib state + 2 lib history + 4 round_trip + 1 no_leak)
- **Tests totales bed-server:** 19 verdes

## Accomplishments

- 4 handlers axum compilados, registrados en router, y testeados end-to-end via tower::ServiceExt::oneshot
- HIST-03 verificado por test que cifra el fixture multisig 2-of-3 de Phase 1, persiste el .bed, y aserta que ningún substring del descriptor cleartext (xpubs, fingerprints, checksum, function name, multipath) aparece en archivos del data_dir
- Anti path traversal robusto: validate_history_id rechaza uppercase, longitud incorrecta, no-hex, `../`, espacios — 5 patrones probados
- AppError extendido sin romper variantes existentes; envelope JSON consistente con Phase 1 (D-17)
- BED_DATA_DIR env var permite swap a tempfile en tests sin tocar /data/encrypted

## Task Commits

Cada task fue commiteado atómicamente (--no-verify por wave-1 paralelo con 02-01):

1. **Task 1: AppError variantes + data_dir() helper + dependencias workspace** — `42cd161` (feat)
2. **Task 2: 4 handlers history.rs (POST, GET list, GET id, DELETE id) + router** — `fe4b884` (feat)
3. **Task 3: Integration tests round-trip + no-leak HIST-03** — `9bfb3cb` (test)

## Files Created/Modified

**Creados:**
- `crates/server/src/routes/history.rs` — 4 handlers + parse_filename + find_file_by_id + 2 unit tests
- `crates/server/tests/history_round_trip.rs` — 4 integration tests con #[serial]
- `crates/server/tests/history_no_leak.rs` — 1 test HIST-03 con descriptor real

**Modificados:**
- `Cargo.toml` — workspace.dependencies + tokio fs feature
- `crates/server/Cargo.toml` — 3 deps nuevas + 2 dev-deps
- `crates/server/src/error.rs` — 3 variants AppError
- `crates/server/src/state.rs` — data_dir + validate_history_id + 5 unit tests
- `crates/server/src/lib.rs` — router con 6 rutas (4 history nuevas)
- `crates/server/src/routes/mod.rs` — pub mod history

## Decisions Made

- **Feature flag rust-embed:** "axum" (validado por compilación exitosa contra rust-embed 8.11.0). El comentario de la PLAN sobre posible "axum-ex" no aplicó.
- **Signature de bed_core::render_qr_png:** recibe `&str` (no `&[u8]`); decisión tomada inspeccionando crates/core/src/qr.rs antes de implementar handler.
- **axum 0.8 path syntax:** confirmado `{id}` (no `:id`) leyendo Cargo.lock — version 0.8.9 instalada.
- **serial_test añadido como dev-dep:** los 5 integration tests setean BED_DATA_DIR globalmente; sin serial habría race condition.
- **Inline test allows:** `#![allow(clippy::unwrap_used)]` y `#![allow(clippy::expect_used)]` adicionales en mod tests (workspace lints aplican incluso a #[cfg(test)]).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Workspace tokio dep necesitaba feature `fs` actualizada también, no solo crate-level**
- **Found during:** Task 1 (build inicial)
- **Issue:** El plan instruía añadir `fs` al feature set del crate server, pero el server crate hereda de workspace.dependencies.tokio. Si solo se modifica el crate-level features sin el workspace-level, cargo no compone correctamente y `tokio::fs` no resuelve.
- **Fix:** Añadí `fs` al features array en workspace.dependencies.tokio en Cargo.toml raíz también.
- **Files modified:** Cargo.toml (workspace), crates/server/Cargo.toml
- **Verification:** `cargo build -p bed-server` exit 0; `tokio::fs::read_dir` resuelve sin errores.
- **Committed in:** `42cd161` (Task 1 commit)

**2. [Rule 1 - Bug] Inline test usa `expect()`, deniega por workspace lint**
- **Found during:** Task 2 (cargo clippy --tests fallaba)
- **Issue:** El módulo `mod tests` dentro de routes/history.rs usaba `.expect("parse should succeed")`. Workspace lints en `[workspace.lints.clippy]` declaran `expect_used = "deny"` y aplican incluso al test target.
- **Fix:** Añadí `#![allow(clippy::unwrap_used)]` y `#![allow(clippy::expect_used)]` al inicio del módulo de tests, junto al `#![allow(clippy::panic)]` ya presente.
- **Files modified:** crates/server/src/routes/history.rs
- **Verification:** `cargo clippy -p bed-server --all-targets -- -D warnings` exit 0.
- **Committed in:** `fe4b884` (Task 2 commit)

**3. [Rule 1 - Bug] Clippy needless_borrows_for_generic_args en URI builders**
- **Found during:** Task 3 (cargo clippy --all-targets tras añadir integration tests)
- **Issue:** `&format!("/api/history/{id}")` triggers Clippy's needless_borrows_for_generic_args lint en Rust 1.93.
- **Fix:** Removido el `&` redundante; `.uri()` acepta `String` directamente.
- **Files modified:** crates/server/tests/history_round_trip.rs
- **Verification:** Clippy verde, todos los tests pasan post-fix.
- **Committed in:** `9bfb3cb` (Task 3 commit)

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 bugs)
**Impact on plan:** Todos los auto-fixes son ajustes mecánicos al entorno (workspace tokio features, lint allows, clippy versión). Sin scope creep, ningún cambio funcional al diseño.

## Issues Encountered

- **Race condition en env var BED_DATA_DIR entre tests integración:** mitigado preventivamente con `serial_test::serial` (PLAN ya lo anticipó). Sin esta protección, los 5 tests podrían ejecutarse en paralelo y mutar el env globalmente.
- **Fixture descriptor reutilizado:** Phase 1 ya tiene `tests/fixtures/desc.txt` con multisig 2-of-3 válido. El test no_leak lo importa con `include_str!` evitando hardcodear un descriptor potencialmente inválido.
- **Parallel commit con 02-01:** la rama master vio commits intercalados de wave-1 (02-01 y 02-02). `git log` muestra el orden cronológico real: 42cd161 (02-02 task 1) → 53c0a62 (02-01 fonts) → fe4b884 (02-02 task 2) → 0a4e859 (02-01 summary) → 9bfb3cb (02-02 task 3). Sin conflicto: archivos disjuntos (frontend/ vs crates/server/).

## User Setup Required

Ninguna. La env var `BED_DATA_DIR` tiene default `/data/encrypted` que coincidirá con el volume `main` de StartOS en Phase 4. En desarrollo local los tests usan tempdirs.

## Next Phase Readiness

- Plan 02-06 (tab-historial + rust-embed) puede consumir los 4 endpoints directamente vía fetch
- Contracts response shapes coinciden con UI-SPEC: PostHistoryResponse {id, timestamp, filename}, ListHistoryResponse {entries: [{id, timestamp, filename, size_bytes}]}, GetHistoryIdResponse {bed_b64, armored, qr_png_b64}
- HIST-03 ya enforced backend; el plan 02-06 solo necesita repetir el guard de "no enviar descriptor cleartext al servidor" en el cliente JS

---

## Output Spec Compliance

Confirmaciones requeridas por el `<output>` del plan:

- **Versiones exactas:**
  - rust-embed 8.11.0 ([feature: "axum"])
  - uuid 1.23.1 ([feature: "v4"])
  - time 0.3.47 ([features: "formatting", "macros"])
  - tempfile 3.27.0 (dev-dep)
  - serial_test 3.4.0 (dev-dep, añadido)
- **Feature flag rust-embed elegido:** `"axum"` (NO "axum-ex"). Compila limpio en rust-embed 8.11.0.
- **bed_core signatures usadas:**
  - `encode_armored(bytes: &[u8]) -> String` ✓
  - `render_qr_png(armored: &str) -> Result<Vec<u8>, CoreError>` ✓ (NO `&[u8]` como sugería el RESEARCH)
- **axum 0.8 path syntax:** `/api/history/{id}` (braces). Cargo.lock confirma axum 0.8.9.
- **Conteo de tests pasando:**
  - Lib state: 5 ✓
  - Lib routes::history: 2 ✓
  - Integration history_round_trip: 4 ✓
  - Integration history_no_leak: 1 ✓
  - Total nuevo: 12 tests
  - Total bed-server: 19 verdes (incluye Phase 1 intactos: no_leak 1, round_trip 2, validation 4)

---
*Phase: 02-spa-frontend-history*
*Completed: 2026-05-06*

## Self-Check: PASSED

- All 3 created files exist on disk
- All 3 task commits present in git log (42cd161, fe4b884, 9bfb3cb)
- 19 tests pass (cargo test -p bed-server -- --test-threads=1)
- Clippy verde (cargo clippy -p bed-server --all-targets -- -D warnings)
