---
phase: 01-crypto-core-http-api
plan: 04
subsystem: core-crypto
tags: [rust, encryption, armored, qr, bed, bitcoin-encrypted-backup, round-trip]
dependency_graph:
  requires: [01-01-workspace-skeleton, 01-03-core-validate-zeroize]
  provides: [armored-encoder-decoder, qr-png-generator, encrypt-descriptor, decrypt-payload]
  affects: [crates/server encrypt/decrypt handlers in plan 01-05+]
tech_stack:
  added:
    - base64 0.22 (armored encoder/decoder)
    - qrcode 0.14 + image 0.25 (QR PNG generator, ECC-L)
    - bitcoin_encrypted_backup::EncryptedBackup (encrypt + decrypt builder)
    - bitcoin_encrypted_backup::descriptor::dpk_to_pk (xpub → PublicKey conversion)
  patterns:
    - PEM-style armored format with exact BEGIN/END BITCOIN ENCRYPTED BACKUP headers
    - QR ECC-L with 2900-byte cap enforced at render time (typed error)
    - encrypt_descriptor takes &mut Zeroizing<String> to honor D-10 stack-copy avoidance
    - decrypt_payload returns Zeroizing<String> to protect cleartext on heap
    - Cross-implementation interop: wallet.bed from reference impl decrypts correctly
key_files:
  created:
    - crates/core/src/armored.rs
    - crates/core/src/qr.rs
    - crates/core/src/encrypt.rs
    - crates/core/src/decrypt.rs
    - crates/core/tests/armored.rs
    - crates/core/tests/round_trip.rs
    - crates/core/tests/fixtures/wallet.bed
    - crates/core/tests/fixtures/xpub.txt
  modified:
    - crates/core/src/lib.rs (added pub mod armored, decrypt, encrypt, qr + re-exports)
decisions:
  - "Normalize h↔' in round-trip assertions: miniscript re-serializes 48h/0h/0h/2h as 48'/0'/0'/2'; both are BIP-380 valid; comparison normalizes both sides"
  - "Add #![allow(clippy::panic)] to test files: workspace lint blocks panic! even in test binaries; allow at file level for test-only code"
  - "desc.txt fixture already committed by sibling 01-03; wallet.bed + xpub.txt added by this plan"
metrics:
  duration_minutes: 18
  tasks_completed: 2
  tasks_total: 2
  files_created: 8
  files_modified: 1
  completed_date: "2026-05-05"
---

# Phase 01 Plan 04: Core Armored + QR + Encrypt/Decrypt Summary

**One-liner:** PEM-style armored encoder/decoder (64-char line-wrap, BOM/CRLF/indent tolerant), QR PNG generator with typed QrTooLarge error at 2900 bytes, and encrypt_descriptor + decrypt_payload wrappers over bitcoin-encrypted-backup — all verified via cross-implementation round-trip with pythcoiner/bed reference fixtures.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Armored encoder/decoder + tolerance tests (DEC-05) | 4e78c27 | crates/core/src/armored.rs, crates/core/src/lib.rs, crates/core/tests/armored.rs |
| 2 | encrypt/decrypt/qr + round-trip tests + fixtures (CORE-02) | 554b98e | crates/core/src/qr.rs, crates/core/src/encrypt.rs, crates/core/src/decrypt.rs, crates/core/tests/round_trip.rs, crates/core/tests/fixtures/wallet.bed, crates/core/tests/fixtures/xpub.txt |

## Verification Results

- `cargo build -p bed-core` — exits 0
- `cargo test -p bed-core` — exits 0; 18 total tests pass (8 armored + 4 round_trip + 6 validate)
- `cargo clippy -p bed-core --all-targets -- -D warnings` — exits 0
- `grep -E '\.unwrap\(\)|\.expect\(' crates/core/src/armored.rs crates/core/src/encrypt.rs crates/core/src/decrypt.rs crates/core/src/qr.rs | grep -v unwrap_or` — 0 matches
- `diff crates/core/tests/fixtures/wallet.bed /tmp/bed-test/wallet.bed` — exits 0

## Payload Size Measurements

- **wallet.bed size:** 614 bytes (reference impl fixture)
- **Armored size:** ~913 bytes (40-char BEGIN header + 820-char base64 body in 13 lines of 64 + 38-char END header + newlines)
- **QR capacity check:** 913 bytes << 2900 byte cap — fixture fits comfortably in a single QR ECC-L code
- **QR PNG size:** approximately 3-8 KB depending on content complexity (256×256 minimum dimension)

## Tests Passed

**armored.rs (8 tests):**
- `round_trip_identity` — encode_armored → decode_armored == identity
- `headers_present_exactly` — BEGIN/END headers with exact string constants
- `line_wrap_64_chars` — no line exceeds 64 chars in base64 body
- `tolerates_crlf` — \r\n line endings decoded correctly
- `tolerates_indentation_and_trailing_spaces` — trimmed before decode
- `tolerates_bom` — BOM stripped before processing
- `rejects_wrong_header` — BEGIN PGP → WrongHeader error
- `rejects_empty_payload` — empty block → EmptyPayload error

**round_trip.rs (4 tests):**
- `round_trip_fixture` — encrypt_descriptor → decrypt_payload with fixture xpub → same descriptor (normalized)
- `decrypt_with_wrong_xpub_fails` — unrelated xpub → Err()
- `cross_implementation_decrypt_with_reference_bed` — wallet.bed from pythcoiner/bed decrypts correctly via our decrypt_payload
- `qr_too_large_returns_error` — 3000-char input → CoreError::QrTooLarge { size: 3000, max: 2900 }

## Cross-Implementation Confirmation (D-13)

`crates/core/tests/fixtures/wallet.bed` was produced by the reference implementation at `/tmp/bed-test/wallet.bed`. Our `decrypt_payload` successfully decrypts it using the matching xpub from `xpub.txt` and recovers the descriptor from `desc.txt` (after normalizing `h`/`'` hardened markers). This confirms wire-format compatibility with pythcoiner/bed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Round-trip test assertions normalize h↔' hardened notation**
- **Found during:** Task 2 (first test run)
- **Issue:** The miniscript library normalizes `h` hardened marker (e.g. `48h`) to apostrophe (`48'`) when parsing and re-serializing descriptors. Both `desc.txt` (uses `h`) and the decrypted output (uses `'`) represent the same valid BIP-380 descriptor, but string equality fails.
- **Fix:** Added `normalize_desc()` helper in `round_trip.rs` that strips checksums and replaces `h` markers (preceded by digit, followed by `/` or `]`) with `'` before comparison.
- **Files modified:** crates/core/tests/round_trip.rs
- **Commit:** 554b98e

**2. [Rule 2 - Missing critical functionality] Added #![allow(clippy::panic)] to test files**
- **Found during:** Task 2 (cargo clippy -D warnings run)
- **Issue:** Workspace lint `clippy::panic = "warn"` elevated to `-D warnings` also triggers on `panic!` calls inside test binaries. The plan's test code uses `panic!` via `unwrap_or_else(|e| panic!(...))` which is idiomatic for test failure messages.
- **Fix:** Added `#![allow(clippy::panic)]` at crate root of both test files; added `#![allow(clippy::unwrap_used)]` to round_trip.rs for `.unwrap_err()` usage.
- **Files modified:** crates/core/tests/armored.rs, crates/core/tests/round_trip.rs
- **Commit:** 554b98e

**3. [Rule 3 - Blocking issue] Created qr.rs/encrypt.rs/decrypt.rs before armored tests could run**
- **Found during:** Task 1 (compilation of armored test binary)
- **Issue:** lib.rs declared `pub mod encrypt; pub mod decrypt; pub mod qr;` which the compiler requires to exist even when only compiling the armored test. The modules didn't exist yet.
- **Fix:** Created stub-equivalent qr.rs, encrypt.rs, decrypt.rs as part of Task 1 compilation unblock; fully implemented in Task 2.
- **Files modified:** crates/core/src/qr.rs, crates/core/src/encrypt.rs, crates/core/src/decrypt.rs
- **Commit:** 4e78c27 (Task 1 already had the file declared; Task 2 filled in full implementation)

## Known Stubs

None — all modules are fully implemented.

The `validate::require_multipath_0_1` function referenced in `encrypt.rs` was initially a stub in the sibling plan (01-03) but was implemented by the sibling agent before this plan's Task 2 ran. Full cross-plan integration works.
