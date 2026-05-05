# Phase 1: Crypto Core + HTTP API - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Phase:** 01-crypto-core-http-api
**Mode:** `--auto` (Claude auto-selected recommended option for each gray area)
**Areas discussed:** API surface, Decrypt input format, BIP wildcard validation, Error response shape, Armored format implementation, QR strategy, Tracing/logging, Panic hook, Crate pinning, Test infrastructure, CI workflow, Repo init scope, Bind address

---

## API Surface — `/api/encrypt`

| Option | Description | Selected |
|--------|-------------|----------|
| Single JSON endpoint with all 3 outputs | `POST /api/encrypt` returns `{bed_b64, armored, qr_png_b64}` in one response | ✓ |
| Three separate endpoints | `/api/encrypt/bed`, `/api/encrypt/armored`, `/api/encrypt/qr` | |
| Content-negotiation single endpoint | `Accept` header determines output format | |

**Auto-selected:** Single JSON endpoint with all 3 outputs.
**Notes:** Matches success criterion #1 ("returns three outputs"); single round-trip; frontend simpler. Chosen over content-negotiation because the SPA needs all three simultaneously to render download buttons + armored copy + QR display.

---

## API Surface — `/api/decrypt`

| Option | Description | Selected |
|--------|-------------|----------|
| Multipart with mixed text/file fields | `bed` field accepts armored text OR binary file; `xpub` accepts text OR file | ✓ |
| JSON body with base64 fields | `{bed_b64, xpub}` — client encodes binary | |
| Two separate endpoints | `/api/decrypt/binary` (multipart) + `/api/decrypt/armored` (JSON) | |

**Auto-selected:** Multipart with mixed text/file fields.
**Notes:** REQ DEC-01 specifies multipart. Single endpoint avoids duplication; server detects format by header presence / magic bytes.

---

## BIP Wildcard Validation Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Strict multipath `<0;1>/*` only | Reject `Wildcard::None`, simple `/*`, and multipath with non-`<0;1>` indices on primary descriptor | ✓ |
| Any wildcard accepted | Just check `Wildcard::None` like PITFALLS pseudocode | |
| Defer to crate | Trust `bitcoin-encrypted-backup` to enforce | |

**Auto-selected:** Strict multipath `<0;1>/*` only.
**Notes:** PITFALLS #1 documents that the crate does NOT enforce; BIP draft requires multipath specifically. Anything weaker silently breaks privacy.

---

## Error Response Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Uniform JSON `{error: {code, message}}` with thiserror enum | Single `AppError` mapped to status codes via `IntoResponse` | ✓ |
| RFC 7807 Problem Details | Standard `application/problem+json` | |
| Plain text errors | `String` body with status code | |

**Auto-selected:** Uniform JSON `{error: {code, message}}`.
**Notes:** Predictable for SPA, typed at code level, no extra deps. RFC 7807 viable but adds verbosity not justified here.

---

## Armored Format Implementation

| Option | Description | Selected |
|--------|-------------|----------|
| Implement in `crates/core/src/armored.rs` | App owns format: BEGIN/END headers + base64 + line-wrap | ✓ |
| Use crate's armored API | (Disqualified — crate doesn't provide it) | |
| Defer armored to Phase 2 | (Violates ENC-03 success criterion) | |

**Auto-selected:** Implement in `crates/core`.
**Notes:** Verified by reading `/tmp/bed-test/encrypted_backup/src/`. The crate is binary-only. Header string must be cross-validated against BIP draft + reference GUI before Phase 1 closes.

---

## QR Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Server-side `qrcode 0.14` + `image 0.25`, ECC-L | PNG output via `ImageOutputFormat::Png`; reject if armored > 2900 B with 422 | ✓ |
| Client-side QR generation | SPA generates QR from base64 in browser | |
| Server-side ECC-M | Better error correction, less capacity | |

**Auto-selected:** Server-side ECC-L with size guard.
**Notes:** Server-side guarantees consistency. ECC-L gives max capacity; multisig 2-of-3 typical payload measured during implementation.

---

## Tracing / Logging Policy

| Option | Description | Selected |
|--------|-------------|----------|
| `EnvFilter::from_default_env()` + `skip_all` on sensitive handlers + minimal `TraceLayer` span | INFO default, no bodies, no descriptors in spans | ✓ |
| Verbose `TraceLayer::new_for_http()` everywhere | Full request/response logging | |
| No tracing layer, only `println!` | (Violates SEC-01) | |

**Auto-selected:** `EnvFilter` + `skip_all` + minimal span fields.
**Notes:** PITFALLS #2. Test no-leak captures TestWriter buffer and asserts descriptor string absent.

---

## Panic Hook

| Option | Description | Selected |
|--------|-------------|----------|
| `set_hook` discarding `PanicInfo`, log generic "internal panic" | No backtrace, no locals leaked | ✓ |
| Default Rust panic handler | (Leaks locals via backtrace) | |
| Log full panic with payload | (Same leak risk) | |

**Auto-selected:** Discard `PanicInfo`.
**Notes:** PITFALLS #3. `RUST_BACKTRACE` never set in container env.

---

## Crate Pinning Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Git dep with exact `rev = "<sha>"` | Resolve HEAD SHA in plan-phase, document in Cargo.toml comment | ✓ |
| Git dep with `tag = "v1.0.0"` | (Tags can be moved upstream) | |
| Git dep with `branch = "master"` | (Auto-updates, breaks reproducibility) | |
| Vendor the crate | (Maintenance burden, drift) | |

**Auto-selected:** Exact `rev`.
**Notes:** Draft BIP — protect against silent upstream changes. SHA resolved when planner queries the upstream repo.

---

## Test Infrastructure

| Option | Description | Selected |
|--------|-------------|----------|
| `tower::ServiceExt::oneshot` + `axum::body::to_bytes` | In-process, no sockets, no flaky ports | ✓ |
| `axum-test` crate | Higher-level helpers but extra dep | |
| `reqwest` against bound localhost server | Simulates real HTTP, but socket dance | |

**Auto-selected:** `oneshot` pattern.
**Notes:** `tower` already in dep graph via `tower-http`. Zero additional dependencies.

---

## CI Workflow

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Actions `.github/workflows/ci.yml` with fmt/clippy/test/audit/deny jobs | Standard for Rust projects, free tier sufficient | ✓ |
| Single monolithic test job | Less parallel, slower feedback | |
| Self-hosted runner | (Premature; no need for arm64 in Phase 1) | |

**Auto-selected:** GitHub Actions multi-job.
**Notes:** REQ CI-01 + CI-02. `ubuntu-latest` runner. Multi-arch CI deferred to Phase 3.

---

## Repo Init Scope (what Phase 1 creates)

| Option | Description | Selected |
|--------|-------------|----------|
| Workspace + 2 crates + CI + deny.toml + rust-toolchain.toml; NO Dockerfile | Phase 1 sets foundations only | ✓ |
| Include Dockerfile too | Phase 3 already owns this | |
| Minimal: just one crate | Defer workspace split until needed | |

**Auto-selected:** Full workspace skeleton, no Dockerfile.
**Notes:** Workspace split is research-locked (D-02). Dockerfile is Phase 3 scope.

---

## Bind Address Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Hardcoded constant `127.0.0.1:8080` | StartOS routes externally; no override needed | ✓ |
| Env var `BED_BIND_ADDR` | Flexibility for dev (overkill in Phase 1) | |
| Config file `config.json` | (file-models is Phase 4 territory) | |

**Auto-selected:** Hardcoded constant.
**Notes:** REQ SEC-02. Phase 1 minimalism — no env machinery yet.

---

## Claude's Discretion

Areas explicitly delegated to planner / executor:
- Internal layout of subdirs within each crate (`routes/`, `handlers/`, etc.)
- Exact variant names for `AppError` (preserve `code` field contract)
- Latest patch versions of `serde`, `thiserror`, `tracing`, etc.
- Whether to use `proptest` or `quickcheck` for property-based validation tests
- Internal helpers for armored encoder (line-wrap implementation detail)

## Deferred Ideas

- Drag-and-drop, "test decrypt", checksum display, descriptor syntax error specificity → v2
- File-models config persistence → v1.x
- JSON log format env var → v2 if needed
- History endpoints stubs → not in Phase 1 (Phase 2 owns HIST-*)
- Bind addr env override → not in v1
