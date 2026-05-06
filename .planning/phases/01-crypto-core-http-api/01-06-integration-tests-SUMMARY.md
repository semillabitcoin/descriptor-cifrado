---
phase: 01-crypto-core-http-api
plan: 06
subsystem: test
tags: [rust, axum, integration-test, tower-oneshot, tracing, no-leak, sec-01, ci-02]
dependency_graph:
  requires:
    - phase: 01-05-server-axum-handlers
      provides: router(), AppError IntoResponse, POST /api/encrypt, POST /api/decrypt
  provides:
    - End-to-end encrypt+decrypt round-trip via HTTP (CI-02)
    - Descriptor never appears in tracing log output (SEC-01)
    - Invalid descriptor → 422; wrong xpub → 422; bad JSON → 400; missing bed → 400
  affects: [CI job "test", Phase 2 SPA contract stability]
tech-stack:
  added: []
  patterns:
    - tower::ServiceExt::oneshot for in-process HTTP test (D-23, no socket bind)
    - SharedBuf MakeWriter over Arc<Mutex<Vec<u8>>> for tracing capture in no-leak test (D-20)
    - tracing::subscriber::with_default + tokio Runtime::block_on inside sync #[test]
    - "#![allow(clippy::panic)] + #![allow(clippy::unwrap_used)] in test files (workspace lint override)"
    - strip_checksum() helper to normalize BIP-380 descriptor before round-trip comparison
key-files:
  created:
    - crates/server/tests/round_trip.rs (encrypt_then_decrypt_roundtrip + decrypt_with_binary_bed_works)
    - crates/server/tests/no_leak.rs (descriptor_never_appears_in_logs, MakeWriter pattern)
    - crates/server/tests/validation.rs (4 HTTP boundary tests)
    - crates/server/tests/fixtures/desc.txt (2-of-3 multisig descriptor fixture)
    - crates/server/tests/fixtures/xpub.txt (cosigner xpub fixture)
    - crates/server/tests/fixtures/wrong_xpub.txt (unrelated xpub for XPUB_MISMATCH test)
  modified:
    - crates/core/src/{armored,decrypt,encrypt,validate,zeroize}.rs (cargo fmt reformatting only)
    - crates/core/tests/{armored,round_trip,validate}.rs (cargo fmt reformatting only)
    - crates/server/src/{error,routes/decrypt,routes/encrypt}.rs (cargo fmt reformatting only)
key-decisions:
  - "Strip BIP-380 checksum (#xxxxxxxx) from descriptor before round-trip comparison: miniscript re-computes checksum after canonical normalization of sortedmulti key ordering"
  - "#![allow(clippy::panic)] required in test files: workspace lint panic=warn escalated to error by -D warnings in CI clippy job"
  - "cargo fmt --all applied to pre-existing unfmt core/server files: required for cargo fmt --check to pass in CI"
requirements-completed: [CORE-02, CI-02, SEC-01]

duration: 27min
completed: "2026-05-06"
---

# Phase 01 Plan 06: Integration Tests Summary

**3 integration test files (7 tests) covering round-trip end-to-end via HTTP, descriptor-never-in-logs assertion with MakeWriter, and HTTP 422/400 boundary validation — all using tower::ServiceExt::oneshot, zero socket binding, zero external HTTP clients**

## Performance

- **Duration:** ~27 min
- **Started:** 2026-05-06T04:21:00Z
- **Completed:** 2026-05-06T04:48:16Z
- **Tasks:** 2
- **Files created:** 6
- **Files modified:** 9 (fmt only on pre-existing)

## Accomplishments

- `round_trip.rs`: `encrypt_then_decrypt_roundtrip` — POST /api/encrypt with fixture desc → captures armored → POST /api/decrypt with fixture xpub → asserts descriptor matches (normalizing h/apostrophe and stripping BIP-380 checksum); `decrypt_with_binary_bed_works` — verifies bed_b64 (raw base64, no PEM headers) also decrypts correctly
- `validation.rs`: 4 tests — bare xpub → 422 MISSING_MULTIPATH_WILDCARD with castellano message containing `<0;1>/*` and `xpub on-chain`; wrong xpub → 422 XPUB_MISMATCH; malformed JSON → 400; missing bed field → 400
- `no_leak.rs`: `descriptor_never_appears_in_logs` — installs SharedBuf MakeWriter subscriber at TRACE level via `with_default`; runs full encrypt+decrypt round-trip; captures 0 bytes of descriptor or xpub in log output (verified: buffer captured tracing output but descriptor was absent)
- `cargo fmt --all` applied to all workspace files — pre-existing format violations in core/server src files fixed
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0
- `cargo deny check` exits 0 (advisories ok, bans ok, licenses ok, sources ok)
- `ldd target/release/bed-server | grep -E "libssl|native-tls"` → empty (SEC-03 confirmed)

## Workspace Test Results

```
cargo test --workspace --all-features --locked

bed_core (lib): 0 tests
bed_core/tests/armored.rs: 8 passed
bed_core/tests/round_trip.rs: 4 passed
bed_core/tests/validate.rs: 6 passed
bed_server (lib): 0 tests
bed_server/tests/no_leak.rs: 1 passed
bed_server/tests/round_trip.rs: 2 passed
bed_server/tests/validation.rs: 4 passed
doc-tests: 0 tests each

Total: 25 passed, 0 failed
```

## SEC-01 No-Leak Confirmation

The `descriptor_never_appears_in_logs` test ran full encrypt+decrypt with TRACE-level tracing subscriber backed by a SharedBuf (in-memory, zero I/O). The buffer captured tracing output from the axum tower stack and handlers. Post-run assertions:
- `!captured.contains(&descriptor)` → PASSED (descriptor not in buffer)
- `!captured.contains(needle)` (xpub) → PASSED (xpub not in buffer)

This is enforced by `#[tracing::instrument(skip_all)]` on every handler (D-19) and the `ZeroizingDescriptor` newtype that omits `Debug`/`Display` (D-11).

## SEC-03 Confirmation

```
ldd target/release/bed-server | grep -E "libssl|native-tls"
(empty — no match)
```

Binary links only: libgcc_s, libc, libm. No OpenSSL, no native-tls. StartOS handles TLS at the network layer.

## Phase 1 Requirements Closed

| Requirement | Description | Closed in Plan |
|-------------|-------------|----------------|
| CORE-01 | Cargo workspace with core + server crates | 01-01 |
| CORE-02 | Round-trip deterministic encrypt/decrypt | 01-04, 01-06 |
| CORE-03 | <0;1>/* multipath validation | 01-03 |
| CORE-04 | Zeroizing<String> at handler boundary | 01-05 |
| CORE-05 | Zero unwrap()/expect() in server/src/ | 01-05 |
| ENC-01 | POST /api/encrypt JSON endpoint | 01-05 |
| ENC-02 | Returns bed_b64 binary | 01-05 |
| ENC-03 | Returns armored PEM string | 01-04, 01-05 |
| ENC-04 | Returns qr_png_b64 PNG | 01-04, 01-05 |
| ENC-05 | Invalid descriptor → 422 MISSING_MULTIPATH_WILDCARD | 01-05, 01-06 |
| DEC-01 | POST /api/decrypt multipart endpoint | 01-05 |
| DEC-02 | Accepts armored bed | 01-05 |
| DEC-03 | Accepts binary bed | 01-05, 01-06 |
| DEC-04 | Wrong xpub → 422 XPUB_MISMATCH | 01-05, 01-06 |
| SEC-01 | Descriptor never in tracing logs | 01-05, 01-06 |
| SEC-02 | Panic hook logs only "internal panic" | 01-05 (main.rs) |
| SEC-03 | No OpenSSL/native-tls in binary | 01-01, 01-06 (verified) |
| CI-01 | CI workflow (fmt/clippy/test/audit/deny) | 01-02 |
| CI-02 | CI runs round-trip + no-leak in test job | 01-06 |

## Task Commits

1. **Task 1: round_trip.rs + validation.rs + fixtures** - `a950b51` (test)
2. **Task 2: no_leak.rs + cargo fmt all** - `b30fa9c` (test)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Strip BIP-380 checksum before round-trip comparison**
- **Found during:** Task 1 first test run
- **Issue:** `encrypt_then_decrypt_roundtrip` failed because miniscript re-computes the checksum after internal normalization of the descriptor (checksum changed from `#rzf36yej` to `#da2y4klw`). The content was identical; only the checksum differed.
- **Fix:** Added `strip_checksum()` helper inside the test that strips `#XXXXXXXX` suffix (8 alphanumeric chars after `#`) before comparing. Normalization of `'`→`h` was already planned (decision in Plan 05).
- **Files modified:** `crates/server/tests/round_trip.rs`
- **Commit:** `a950b51`

**2. [Rule 1 - Bug] Added `#![allow(clippy::panic)]` and `#![allow(clippy::unwrap_used)]` to test files**
- **Found during:** Task 2 (`cargo clippy --workspace --all-targets --all-features -- -D warnings`)
- **Issue:** Workspace lint `clippy::panic = "warn"` combined with `-D warnings` in CI clippy job causes `panic!()` calls inside `unwrap_or_else(|e| panic!(...))` closures in test files to fail clippy. The plan noted this pattern but didn't pre-add the allow attributes.
- **Fix:** Added `#![allow(clippy::panic)]` + `#![allow(clippy::unwrap_used)]` at crate level in all three test files. Consistent with Pattern established in Plan 04 (D-22 note about test helpers).
- **Files modified:** `crates/server/tests/{round_trip,validation,no_leak}.rs`
- **Commit:** `b30fa9c`

**3. [Rule 3 - Blocking] cargo fmt --all applied to pre-existing source files**
- **Found during:** Task 2 (`cargo fmt --all -- --check` acceptance criterion)
- **Issue:** `cargo fmt --check` failed because existing files in `crates/core/src/` and `crates/server/src/` had formatting violations (single-line if, import grouping, etc.) from prior plans. These were pre-existing, not introduced by Plan 06.
- **Fix:** Applied `cargo fmt --all` which fixed format in 9 pre-existing files. No behavior change.
- **Files modified:** `crates/core/src/{armored,decrypt,encrypt,validate,zeroize}.rs`, `crates/core/tests/{armored,round_trip,validate}.rs`, `crates/server/src/{error,routes/decrypt,routes/encrypt}.rs`
- **Commit:** `b30fa9c`

## Known Stubs

None — all tests wire real HTTP handlers via `bed_server::router()`. No mocked data or hardcoded responses. The `AppState` stub in `state.rs` was documented in Plan 05 SUMMARY and does not affect test correctness.

## Self-Check: PASSED

Files verified present:
- FOUND: crates/server/tests/round_trip.rs
- FOUND: crates/server/tests/no_leak.rs
- FOUND: crates/server/tests/validation.rs
- FOUND: crates/server/tests/fixtures/desc.txt
- FOUND: crates/server/tests/fixtures/xpub.txt
- FOUND: crates/server/tests/fixtures/wrong_xpub.txt

Commits verified:
- FOUND commit: a950b51 (Task 1)
- FOUND commit: b30fa9c (Task 2)

---
*Phase: 01-crypto-core-http-api*
*Completed: 2026-05-06*
