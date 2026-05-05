# Research Summary — BED Start9 App

Single-binary Rust web service (axum 0.8 + tokio LTS 1.51) wrapping the `bitcoin-encrypted-backup` crate (BIP draft #1951) to encrypt/decrypt Bitcoin multisig descriptors. Packaged as s9pk for StartOS 0.4.0, exposed via Tor onion + LAN, with a Svelte 5 SPA baked into the binary via `rust-embed`.

## Stack — Locked Decisions

| Area | Choice | Why |
|------|--------|-----|
| HTTP | `axum` 0.8 + `tokio` LTS 1.51 + `tower-http` 0.6 | Standard, co-maintained, best-in-class |
| Crypto crate | `bitcoin-encrypted-backup` 1.0.0 as **git dep with pinned `rev`** | Not on crates.io; draft BIP, no migration path |
| Crate features | `miniscript_12_3_5`, `rand`, `base64` | NEVER `devices`, `cli`, `tokio` |
| QR | `qrcode` 0.14 + `image` 0.25 | Pure Rust, links cleanly under distroless |
| Memory hygiene | `secrecy` 0.10 + `zeroize` 1.8 | Wrap cleartext at parse boundary |
| SPA framework | Svelte 5 + Vite 6 (NOT SvelteKit) | <50 KB bundle; vanilla JS becomes brittle at this scope |
| Asset embed | `rust-embed` 8.x with `axum-ex` feature | Bakes SPA into binary, avoids bind-mount anti-pattern |
| Persistence v1 | Directory scan of `/data/encrypted/` | NO embedded DB — sled is dead, SQLite breaks distroless |
| TLS / network | `rustls` everywhere; bind `127.0.0.1:8080` | No `openssl`/`native-tls`; StartOS handles external routing |
| Build image | `rust:slim` → `distroless/cc-debian12` | Glibc, no shell, ~15–25 MB final |
| StartOS SDK | TypeScript `start-sdk` from `hello-world-startos` `update/040` branch | 0.4.0 SDK is TypeScript, not YAML; `master` targets 0.3.x |

## Repo Layout

- Cargo workspace: `crates/core` (pure crypto, unit-testable without HTTP) + `crates/server` (axum)
- Two-repo split: `semillabitcoin/bed-app` (Docker on GHCR) + `semillabitcoin/bed-startos` (TypeScript packaging)

## Resolved Conflicts

| Conflict | Resolution |
|----------|------------|
| ARCHITECTURE.md used SQLite/sqlx; STACK.md banned it | Directory scan in v1, no embedded DB |
| ARCHITECTURE.md bound `0.0.0.0:8080`; PITFALLS.md required loopback | Bind `127.0.0.1:8080`; StartOS routes externally |

## Features — v1 Scope (locked)

**Encrypt:** paste descriptor → `.bed` binary download + armored copy + QR PNG download. **Decrypt:** upload `.bed` (binary or armored) + xpub → cleartext descriptor + copy-to-clipboard. **Validation:** reject descriptors missing `<0;1>/*`. **History:** opt-in toggle, list, delete, never persists cleartext.

Defer to v1.x: drag-and-drop, "test decrypt" round-trip, descriptor checksum display, specific syntax errors. Defer to v2: File Browser integration. Locked out: HWW USB (no USB in container), Shamir SSS, arbitrary data encryption, camera QR scan.

## Top 5 Pitfalls (Must Shape Phase 1)

1. **Descriptor without `<0;1>/*` accepted silently** — crate does NOT enforce this BIP rule. Handler must iterate `desc.iter_pk()` and check `Wildcard::None`. Property-test: bare-xpub → 422.
2. **Cleartext in tracing logs** — `#[tracing::instrument(skip_all)]` on encrypt/decrypt handlers; no body-logging middleware on sensitive routes. CI: assert known descriptor never appears in captured traces.
3. **Panic backtrace leaks locals** — no `unwrap()/expect()` in request path; install custom panic hook emitting generic "internal error".
4. **Zeroize applied too late** — wrap as `SecretString` AT body parse point, before any `?` early-return. Pass by `&mut` ref; never move through function boundaries before zeroing.
5. **Draft BIP crate breakage** — pin `= "1.0.0"` (exact, no `^`); never `cargo update` without running round-trip test against prior `.bed`s.

## Packaging Pitfalls (Phase 6)

- GHCR package left private after first push → silent install failure. Verify with unauthenticated `curl -I` before real-device test.
- distroless missing libs → use `rustls` everywhere; CI `ldd` check asserting no `libssl`.
- `/data/encrypted/` must live inside the `main` volume declared in the StartOS manifest (Dockerfile `VOLUME` is NOT enough).
- Frontend assets baked into image, NOT bind-mounted from `${APP_DATA_DIR}` (would disappear on updates).

## Build Order (Roadmap-Ready)

1. **Core Crate Spike + Crypto Logic** — confirm round-trip, establish security invariants (zeroize, panic hook, no `unwrap`, exact pin, `<0;1>/*` check). Trust boundary of the app.
2. **axum HTTP Backend** — stable API: POST `/api/encrypt` (JSON), POST `/api/decrypt` (multipart), stubbed history. TraceLayer configured `skip_all`. Curl integration tests.
3. **SPA Frontend** — Svelte 5 UI for encrypt + decrypt + history. Baked into Docker image. Depends on Phase 2's API contract being stable.
4. **Persistence (History Mode)** — directory scan of `/data/encrypted/`. Toggle as `AtomicBool` in `AppState`. CI grep test asserting saved `.bed` does not contain descriptor string. File Browser explicitly locked out.
5. **Docker + GHCR** — multi-stage, multi-arch, `ldd` audit, image set public immediately after first push.
6. **StartOS s9pk Packaging** — `bed-startos` repo from `update/040` scaffold; `checkPortListening` health check; `127.0.0.1` binding; volume in manifest. Test on real device, not `docker run`. **Invoke `start9-packaging` skill when planning this phase.**
7. **Hardening** — zeroize path audit, log config verification, threat model README, `cargo-audit` + `cargo-deny` in CI, image size audit.

## Research Flags

- **Phase 6** needs research-before-planning: StartOS 0.4.0 still in beta; SDK details may shift.
- Phases 1–5, 7: standard patterns, no extra research needed.

## Open Questions (Resolve in Implementation, Not Roadmap)

- Exact armored header string and CRC presence — derive from reference impl source in Phase 1, not from memory. Cross-implementation round-trip test mandatory.
- History toggle persistence across container restarts — in-memory in v1; defer `file-models` config to v1.x.
- Real `.bed` payload size for typical 2-of-3 multisig vs QR ECC-L 2,900-byte capacity — measure in Phase 1; if too large, return descriptive error rather than unscannable QR.

## Confidence

| Dimension | Level |
|-----------|-------|
| Stack (core) | HIGH |
| Stack (StartOS 0.4.0 beta) | MEDIUM |
| Features | HIGH |
| Architecture | HIGH (crate source verified locally at `/tmp/bed-test/`) |
| Pitfalls (security/memory) | HIGH |
| Pitfalls (BIP format) | MEDIUM |

**Overall: MEDIUM-HIGH**
