---
phase: 01-crypto-core-http-api
plan: 05
subsystem: api
tags: [rust, axum, http, multipart, zeroize, tracing, error-mapping]
dependency_graph:
  requires:
    - phase: 01-03-core-validate-zeroize
      provides: CoreError, ZeroizingDescriptor
    - phase: 01-04-core-armored-qr-encrypt
      provides: encrypt_descriptor, decrypt_payload, decode_armored signatures
  provides:
    - AppError enum + IntoResponse impl (D-16, D-17)
    - POST /api/encrypt handler (ENC-01..05 API substrate)
    - POST /api/decrypt handler (DEC-01..04 API substrate)
    - router() function for integration test oneshot (D-23)
    - AppState placeholder for Phase 2
  affects: [02-spa-frontend, crates/server/tests]
tech-stack:
  added:
    - base64 0.22 (added to bed-server Cargo.toml for response encoding)
  patterns:
    - AppError enum with IntoResponse mapping CoreError variants to HTTP status codes
    - Zeroizing<String> wrap on FIRST line of encrypt handler before any ? return (D-10)
    - "#[tracing::instrument(skip_all)] on every handler (D-19, SEC-01)"
    - Multipart field loop with named field detection for bed + xpub (D-06)
    - Auto-detect armored (-----BEGIN prefix) vs binary bed bytes in decrypt handler
    - 0 unwrap()/expect() in crates/server/src/ (CORE-05)

key-files:
  created:
    - crates/server/src/error.rs (AppError enum + IntoResponse + From<CoreError>)
    - crates/server/src/state.rs (AppState placeholder for Phase 2)
    - crates/server/src/routes/mod.rs (pub mod encrypt; pub mod decrypt;)
    - crates/server/src/routes/encrypt.rs (POST /api/encrypt handler)
    - crates/server/src/routes/decrypt.rs (POST /api/decrypt handler)
  modified:
    - crates/server/src/lib.rs (modules declared, router() with real handlers, AppError re-exported)
    - crates/server/Cargo.toml (added base64 workspace dep)

key-decisions:
  - "QrTooLarge message uses 'Usa' (Castilian) not 'Usá' (Argentine) per feedback_castellano_no_argentino.md"
  - "base64 added to server Cargo.toml — required for bed_b64 and qr_png_b64 JSON fields"
  - "encrypt handler returns impl IntoResponse (not Result<Json<EncryptResponse>, AppError>) to satisfy axum type inference with Json wrapping"

requirements-completed: [ENC-01, ENC-02, ENC-03, ENC-04, ENC-05, DEC-01, DEC-02, DEC-03, DEC-04, SEC-01, SEC-02, CORE-04, CORE-05]

duration: 5min
completed: "2026-05-05"
---

# Phase 01 Plan 05: Server Axum Handlers Summary

**axum HTTP layer with AppError IntoResponse (422/400/500), POST /api/encrypt returning {bed_b64, armored, qr_png_b64}, and POST /api/decrypt accepting multipart bed (armored or binary) + xpub — all handlers with tracing skip_all and Zeroizing at parse boundary**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-05T22:21:14Z
- **Completed:** 2026-05-05T22:26:00Z
- **Tasks:** 2
- **Files created:** 5
- **Files modified:** 2

## Accomplishments

- AppError enum (6 variants) with IntoResponse producing `{"error":{"code":"UPPER_SNAKE","message":"..."}}` and correct HTTP status codes (422 for validation, 400 for bad request, 500 for internal)
- From<CoreError> for AppError mapping all 6 CoreError variants
- POST /api/encrypt: Zeroizing<String> wrap on first line; bed_core::encrypt_descriptor called; response contains bed_b64 + armored + qr_png_b64
- POST /api/decrypt: multipart field loop detecting bed (armored auto-decoded via decode_armored or raw binary) + xpub; returns {descriptor} in JSON
- router() function in lib.rs for testability via oneshot (D-23)
- 0 unwrap()/expect() in crates/server/src/ — cargo clippy -D warnings exits 0

## Smoke Test Results

```
# Invalid descriptor (bare xpub) → 422 DESCRIPTOR_PARSE
curl -X POST http://127.0.0.1:8080/api/encrypt \
  -H 'Content-Type: application/json' \
  -d '{"descriptor":"xpub661MyMw..."}'
→ HTTP 422: {"error":{"code":"DESCRIPTOR_PARSE","message":"No se pudo parsear el descriptor."}}

# Valid descriptor → 200 with three outputs
curl -X POST http://127.0.0.1:8080/api/encrypt \
  -H 'Content-Type: application/json' \
  -d '{"descriptor":"<fixture desc.txt>"}'
→ HTTP 200: {"bed_b64":"QUJQ...","armored":"-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n...","qr_png_b64":"iVBO..."}

# Missing bed field in decrypt → 400 BAD_REQUEST
curl -X POST http://127.0.0.1:8080/api/decrypt -F "xpub=..."
→ HTTP 400: {"error":{"code":"BAD_REQUEST","message":"solicitud inválida: missing 'bed' field"}}
```

## Task Commits

1. **Task 1: AppError + IntoResponse + state.rs + routes/mod.rs** - `567445d` (feat)
2. **Task 2: Handlers /api/encrypt + /api/decrypt + router update** - `6e002a0` (feat)

## Files Created/Modified

- `crates/server/src/error.rs` - AppError enum with 6 variants, IntoResponse with status codes, From<CoreError>
- `crates/server/src/state.rs` - AppState placeholder for Phase 2 shared state
- `crates/server/src/routes/mod.rs` - Module declarations for encrypt + decrypt
- `crates/server/src/routes/encrypt.rs` - POST /api/encrypt with Zeroizing at parse boundary, tracing skip_all
- `crates/server/src/routes/decrypt.rs` - POST /api/decrypt multipart with armored auto-detection, tracing skip_all
- `crates/server/src/lib.rs` - modules declared, router() with real handlers replacing stubs
- `crates/server/Cargo.toml` - base64 workspace dep added

## Decisions Made

- QrTooLarge message uses "Usa" (Castilian) not "Usá" (Argentine) per feedback_castellano_no_argentino.md — the plan erroneously had "Usá"
- base64 added to server Cargo.toml — not previously listed though it's in workspace deps; needed for encoding bed bytes and qr png to JSON strings
- encrypt handler signature uses `-> Result<impl IntoResponse, AppError>` to preserve type inference compatibility with axum

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing context] Castilian correction: "Usa" instead of "Usá" in QrTooLarge message**
- **Found during:** Task 1 (writing error.rs)
- **Issue:** The plan body specified "Usá el archivo .bed" (Argentine voseo). CLAUDE.md feedback says "castellano, no argentino" — tú/descarga/usa, never vos/usá.
- **Fix:** Used "Usa el archivo .bed o el armored." (Castilian imperative)
- **Files modified:** crates/server/src/error.rs
- **Verification:** Message in error.rs uses "Usa" not "Usá"
- **Committed in:** 567445d (Task 1 commit)

**2. [Rule 3 - Blocking] Added base64 to server Cargo.toml**
- **Found during:** Task 2 (writing encrypt.rs with base64::engine::general_purpose::STANDARD)
- **Issue:** base64 is in workspace deps but not listed in crates/server/Cargo.toml; compile error would occur without it
- **Fix:** Added `base64.workspace = true` to [dependencies] in crates/server/Cargo.toml
- **Files modified:** crates/server/Cargo.toml
- **Verification:** cargo build -p bed-server exits 0
- **Committed in:** 6e002a0 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 convention fix, 1 blocking missing dep)
**Impact on plan:** Both auto-fixes essential for correctness. No scope creep.

## Verification Commands

```
cargo build --workspace         → exits 0
cargo build -p bed-server       → exits 0
cargo clippy -p bed-server --all-targets -- -D warnings → exits 0 (no warnings)
grep -E '\.unwrap\(\)|\.expect\(' crates/server/src/ -r | grep -vE 'unwrap_or|tests/' | wc -l → 0
grep -q '#\[tracing::instrument(skip_all)\]' crates/server/src/routes/encrypt.rs → found
grep -q '#\[tracing::instrument(skip_all)\]' crates/server/src/routes/decrypt.rs → found
```

## Zeroize First Line Confirmation

File: `crates/server/src/routes/encrypt.rs`

```rust
#[tracing::instrument(skip_all)]
pub async fn post_encrypt(
    Json(req): Json<EncryptRequest>,
) -> Result<impl IntoResponse, AppError> {
    // STEP 1 (D-10): wrap immediately; req.descriptor is moved INTO Zeroizing
    // on this line. Any subsequent access is via &mut, never by value.
    let mut cleartext: Zeroizing<String> = Zeroizing::new(req.descriptor);
    // ^^^ THIS IS THE FIRST EXECUTABLE LINE — before any ? early-return
```

## Known Stubs

`crates/server/src/state.rs` — `AppState` is an empty struct (intentional Phase 1 placeholder; Phase 2 adds history toggle). This does not block the plan goal — the router compiles and works correctly without state.

## Next Phase Readiness

- POST /api/encrypt and POST /api/decrypt are fully functional via `curl`
- router() function is testable via tower::ServiceExt::oneshot (D-23) — ready for Plan 01-06 integration tests
- AppError with typed variants ready for Phase 2 SPA to display inline error messages
- Bind address 127.0.0.1:8080 unchanged (from Plan 01); StartOS routes externally

---
*Phase: 01-crypto-core-http-api*
*Completed: 2026-05-05*
