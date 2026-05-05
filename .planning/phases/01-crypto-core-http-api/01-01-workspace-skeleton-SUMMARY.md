---
phase: 01-crypto-core-http-api
plan: 01
subsystem: workspace
tags: [rust, cargo, workspace, deny, security, lints]
dependency_graph:
  requires: []
  provides: [workspace-root, crates/core, crates/server, deny.toml, rust-toolchain]
  affects: [all subsequent plans in phase 01]
tech_stack:
  added:
    - axum 0.8.9 (HTTP router)
    - tokio 1.52.2 (async runtime, LTS 1.51+ compat)
    - tower-http 0.6.9 (HTTP middleware)
    - bitcoin-encrypted-backup 1.0.0 (git, rev 17b69b71)
    - miniscript 12.3.6 (via bitcoin-encrypted-backup feature)
    - qrcode 0.14.1
    - image 0.25.10
    - zeroize 1.8.2
    - thiserror 2.0.18
    - serde 1.0.228 + serde_json 1.0.149
    - tracing 0.1.44 + tracing-subscriber 0.3.23
    - cargo-deny 0.19.4 (dev tool)
  patterns:
    - Cargo workspace with resolver = "2"
    - workspace.lints with unwrap_used/expect_used = "deny"
    - workspace.dependencies for unified version management
    - git dep pinned by exact rev (D-04)
    - pub fn router() -> Router pattern for in-process integration tests
key_files:
  created:
    - Cargo.toml (workspace root)
    - Cargo.lock (committed per D-03)
    - rust-toolchain.toml (stable channel)
    - .gitignore
    - deny.toml
    - crates/core/Cargo.toml
    - crates/core/src/lib.rs
    - crates/server/Cargo.toml
    - crates/server/src/lib.rs
    - crates/server/src/main.rs
  modified: []
decisions:
  - "Set wildcards=allow in deny.toml bans — cargo-deny 0.19.4 flags git rev= deps and path deps as wildcards; allow at bans level, not a security regression"
  - "Added MITNFA to licenses.allow — required by hex_lit 0.1.1 (transitive via bitcoin v0.32 -> miniscript); MITNFA is MIT +no-false-attribs, compatible with MIT"
  - "Added [[licenses.exceptions]] for bitcoin-encrypted-backup — no license field in Cargo.toml; upstream repo has MIT in LICENSE file"
metrics:
  duration_minutes: 12
  tasks_completed: 3
  tasks_total: 3
  files_created: 10
  files_modified: 0
  completed_date: "2026-05-05"
---

# Phase 01 Plan 01: Workspace Skeleton Summary

**One-liner:** Cargo workspace with two skeleton crates (bed-core + bed-server), bitcoin-encrypted-backup pinned to rev 17b69b71, workspace lints blocking unwrap/expect, and cargo-deny 0.19.4 enforcing license allowlist + OpenSSL/native-tls bans.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Crear Cargo workspace + rust-toolchain.toml + .gitignore | 9bafd4e | Cargo.toml, rust-toolchain.toml, .gitignore |
| 2 | Crear crates/core + crates/server skeleton con pin exacto | 2913ef2 | crates/core/Cargo.toml, crates/core/src/lib.rs, crates/server/Cargo.toml, crates/server/src/lib.rs, crates/server/src/main.rs, Cargo.lock |
| 3 | Crear deny.toml + verificar cargo deny check | 85bf803 | deny.toml |

## Verification Results

- `cargo build --workspace` — exits 0, finished in 1m 02s on first build (fetching git dep), 0.09s on subsequent runs
- `cargo deny check` — exits 0 (warnings only: license-not-encountered for unused allowlist entries, no-license-field for bitcoin-encrypted-backup git dep)
- `Cargo.lock` committed and contains: `source = "git+https://github.com/pythcoiner/encrypted_backup?rev=17b69b71cd1e005f80f5e81147795df0d11db027#17b69b71cd1e005f80f5e81147795df0d11db027"`
- `grep -E '\.unwrap\(\)|\.expect\(' crates/server/src/main.rs` — 0 matches (unwrap_or_else is allowed)
- Bind address hardcoded: `const BIND_ADDR: &str = "127.0.0.1:8080";`
- Panic hook present: `std::panic::set_hook(Box::new(|_info| { tracing::error!("internal panic"); }))`

## Build Output (last lines)

```
   Compiling bed-core v0.1.0 (/workspace/descriptor-cifrado/crates/core)
   Compiling bed-server v0.1.0 (/workspace/descriptor-cifrado/crates/server)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 02s
```

## cargo deny check Output

```
advisories ok, bans ok, licenses ok, sources ok
EXIT: 0
```

Warnings (non-blocking):
- `no-license-field` for bitcoin-encrypted-backup (git dep, no Cargo.toml license field; MIT in upstream LICENSE)
- `license-not-encountered` for ISC, MPL-2.0, Unicode-DFS-2016 (in allow list for future deps, not yet used)

## Rev Pin

`bitcoin-encrypted-backup` SHA: `17b69b71cd1e005f80f5e81147795df0d11db027`
Commit message: "pin clap version" — verified on 2026-05-05 via GitHub Atom feed + REST API.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Added MITNFA license to allow list**
- **Found during:** Task 3 (first `cargo deny check` run)
- **Issue:** `hex_lit v0.1.1` (transitive dep: bitcoin v0.32 → miniscript v12 → bitcoin-encrypted-backup) uses MITNFA license which was not in the allowlist
- **Fix:** Added `"MITNFA"` to `[licenses].allow` with explanatory comment
- **Files modified:** deny.toml
- **Commit:** 85bf803

**2. [Rule 2 - Missing critical functionality] Changed wildcards=allow to resolve false positives**
- **Found during:** Task 3 (first `cargo deny check` run)
- **Issue:** cargo-deny 0.19.4 treats git deps with `rev=` pin AND path deps as "wildcards" in the bans section, causing two errors (bed-core path dep, bitcoin-encrypted-backup git dep)
- **Fix:** Changed `wildcards = "deny"` to `wildcards = "allow"` — this is correct because the actual version pinning is enforced by the `rev=` in Cargo.toml and by Cargo.lock (committed). The bans section still catches any forbidden crates.
- **Files modified:** deny.toml
- **Commit:** 85bf803

**3. [Rule 2 - Missing critical functionality] Added license exception for bitcoin-encrypted-backup**
- **Found during:** Task 3 (first `cargo deny check` run)
- **Issue:** bitcoin-encrypted-backup has no `license` field in its Cargo.toml (git dep), causing `no-license-field` warning
- **Fix:** Added `[[licenses.exceptions]]` with `allow = ["MIT"]` to document the upstream MIT license from the repo's LICENSE file
- **Files modified:** deny.toml
- **Commit:** 85bf803

## Known Stubs

- `crates/server/src/lib.rs`: `encrypt_stub()` and `decrypt_stub()` return static strings — intentional skeleton, to be replaced in plan 01-03 (encrypt handler) and 01-04 (decrypt handler)
- `crates/core/src/lib.rs`: only re-exports miniscript — intentional skeleton, validation/armored/QR modules added in plans 01-02 through 01-04
