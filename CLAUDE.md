<!-- GSD:project-start source:PROJECT.md -->
## Project

**BED Start9 App**

App s9pk para StartOS 0.4.0 que cifra y descifra descriptors de Bitcoin siguiendo el draft BIP "Bitcoin Encrypted Backup" (PR `bitcoin/bips#1951`, autor pythcoiner / Wizardsardine). Permite a holders multisig distribuir backups del descriptor con redundancia masiva sin sacrificar privacidad: el `.bed` cifrado solo se descifra con una xpub cosigner.

**Core Value:** Un holder StartOS puede pegar un descriptor multisig y obtener un `.bed` cifrado (binario, armored o QR) sin instalar ni compilar nada, y luego recuperar ese descriptor pegando el `.bed` + cualquier xpub cosigner — todo local, sobre Tor, sin telemetría.

### Constraints

- **Tech stack**: Rust + axum + tokio — importar la crate `bitcoin-encrypted-backup` directamente (NO shellear la CLI `beb`)
- **Tech stack**: Frontend SPA mínima vanilla JS o Svelte servida desde el mismo backend — sin CDN externo, sin telemetría, sin fonts remotas
- **Compatibilidad**: miniscript v0.12.x (la crate soporta features `miniscript_12_0` y `miniscript_12_3_5`)
- **BIP**: descriptors deben usar derivación `<0;1>/*`; sin esto, gastar desde dirección 0 expone la xpub on-chain y rompe el cifrado
- **Plataforma**: solo StartOS 0.4.0 — invocar skill `start9-packaging` cuando llegue empaquetado
- **Imagen**: build con `rust:slim`, runtime con `distroless/cc`, target ~5–10 MB
- **Acceso de red**: Tor onion + LAN, no clearnet
- **Persistencia**: descriptor en claro NUNCA persiste — solo `.bed` cifrado en `/data/encrypted/` (modo opt-in)
- **Repo**: organización `semillabitcoin` (preferencia del usuario, no PRs/forks externos)
- **GHCR**: hacer paquetes públicos tras primer push o el deploy falla
- **Git**: usar email noreply `55397917+4rkad@users.noreply.github.com`
- **Idioma**: comunicación en castellano (no argentino)
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `axum` | 0.8.x (0.8.8 as of Jan 2026) | HTTP router and handler framework | Maintained by the Tokio team, 191M+ crates.io downloads, zero unsafe code, macro-free routing, composable with Tower middleware. Standard choice for Rust HTTP services in 2026. |
| `tokio` | 1.52.x (LTS 1.51.x until Mar 2027) | Async runtime | The only viable async runtime for axum; use LTS 1.51.x for stability if pinning a minor. Feature `rt-multi-thread` + `macros` is sufficient (avoid `full` in production to reduce compile surface). |
| `tower-http` | 0.6.x (0.6.8 latest) | HTTP middleware: ServeDir, compression, tracing | Provides `ServeDir` for dev-mode file serving, `CompressionLayer` (gzip/zstd), and `TraceLayer`. Co-maintained with axum; version tracks axum 0.8.x. |
| `bitcoin-encrypted-backup` | 1.0.0 (git dep — not on crates.io) | BIP-1951 encrypt/decrypt logic | Import as git dependency; use features `miniscript_12_3_5` + `rand` + `base64`; disable `devices` and `cli`. This is the only dependency that implements the BIP draft. |
| `rust-embed` | 8.x (8.8.0 latest) | Embed compiled frontend assets into binary | Embeds HTML/CSS/JS at compile time; dev mode reads from filesystem. Official axum integration example exists (`--features axum-ex`). Produces truly self-contained binary — no filesystem assets needed at runtime. |
### Frontend
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Svelte (plain, no SvelteKit) | 5.x | SPA for encrypt/decrypt UI | Compile-time framework — produces plain HTML/CSS/JS with zero runtime framework overhead. Node required only at build time, not in the Docker image. SvelteKit adds routing/SSR complexity this app doesn't need. Svelte's reactive stores handle the single-page state machine (paste → encrypt → download) with minimal boilerplate. Alternative: vanilla JS (see below). |
| Vite | 6.x | Dev server + bundle for Svelte | Standard bundler for Svelte 5; outputs to `frontend/dist/` which `rust-embed` ingests. No Node in the runtime image. |
### Persistence (history mode, opt-in)
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `redb` | 4.1.0 (Apr 2026) | Key-value metadata store for `.bed` history | Pure Rust, stable 1.0+ release, ACID transactions, MVCC, zero-copy reads. Actively maintained. Stores timestamp + short-id + filename as key; no value needed (the `.bed` file is on disk). Replaces sled, which is effectively abandoned since 2021 and never reached stable. |
### QR Code Generation
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `qrcode` | 0.14.x | Generate QR PNG from base64-armored payload | Integrates with `image` crate for PNG output; widely used, straightforward API. |
| `image` | 0.25.x | Encode QR matrix to PNG bytes | Required by `qrcode` for PNG rendering; use `ImageOutputFormat::Png` to get bytes for HTTP response. |
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` | 1.0 | Serialize/deserialize request/response structs | All JSON API endpoints |
| `serde_json` | 1.0 | JSON encoding/decoding | All JSON API endpoints |
| `tracing` | 0.1 | Structured logging | Replace `println!`; integrates with `tower-http`'s `TraceLayer` |
| `tracing-subscriber` | 0.3 | Log formatting/output | `fmt()` subscriber for container stdout |
| `thiserror` | 2.x | Custom error types | Typed API errors; maps to HTTP status codes cleanly |
| `base64` | 0.22 | Encode/decode armored payloads | Already a transitive dep of `bitcoin-encrypted-backup`; re-export or use directly |
| `tokio-util` | 0.7 | Streaming body helpers | Only if streaming large file uploads; not needed for typical descriptor sizes |
| `uuid` | 1.x | Generate short-id for `.bed` history entries | Use `v4()` feature; truncate to 8 chars for filename component |
### Development Tools
| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo-watch` | Rebuild on file change during dev | `cargo watch -x run` — pairs with Vite's HMR for local dev |
| `cargo-audit` | Dependency vulnerability scanning | Run in CI before each release |
| `cargo-deny` | License and duplicate dep checking | Enforce no GPL transitive deps leaking into a Bitcoin privacy tool |
| `cross` | Cross-compile to `aarch64-unknown-linux-gnu` | Required for StartOS on ARM (Raspberry Pi / Purism) without native hardware |
| Docker BuildKit | Multi-arch image build | `docker buildx build --platform linux/amd64,linux/arm64` |
## Installation
# Cargo.toml (production)
# bitcoin-encrypted-backup — git dep (not on crates.io)
# Do NOT enable: "devices", "cli", "tokio" (tokio is only needed by devices)
# Frontend (build-time only, not in Docker runtime)
## Docker Build Pipeline
# Stage 1: Rust build
# Frontend build (Node only needed here)
# Rust build
# Stage 2: Distroless runtime
## StartOS 0.4.0 Packaging
- Tor: StartOS auto-generates the onion address; developer just declares the interface with `protocol: 'http'` and internal port 8080.
- LAN: Declare `type: 'ui'` interface on port 8080 with `ssl: false` (StartOS handles TLS termination at the LAN layer for `.local` URLs).
## Alternatives Considered
| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| `axum` 0.8 | `actix-web` 4 | actix-web uses unsafe internally; axum is maintained by same team as tokio and integrates via Tower; either works but axum is more idiomatic for this stack |
| `axum` 0.8 | `warp` | warp is stagnant (0.4.1, Aug 2025); less ergonomic API; no active development comparable to axum |
| `redb` 4 | `sled` 0.34 | sled is abandoned, never stable, memory-unsafe under load — avoid categorically |
| `redb` 4 | `rusqlite` 0.38 | rusqlite pulls in C FFI (SQLite); complicates distroless build; redb is pure Rust with no system deps |
| `redb` 4 | directory scan only | For v1 with only list/delete, a directory scan of `/data/encrypted/` is sufficient; defer redb until richer queries are needed |
| `qrcode` 0.14 | `qrcodegen` 1.3 | qrcodegen development stalled; raw pixel output only (no PNG without extra wiring); `qrcode` + `image` is the standard path |
| Svelte (plain) | SvelteKit | SvelteKit is designed for JS/TS backends and SSR; fighting it for a Rust backend adds friction; plain Svelte compiles to the same output without framework complexity |
| Svelte (plain) | vanilla JS | Valid for extremely simple UI; Svelte recommended here because history list, mode toggles, and QR display benefit from reactive state management |
| `distroless/cc-debian12` | `alpine` (musl) | musl introduces subtle incompatibilities with some Rust crates; distroless/cc provides glibc with smaller attack surface than Alpine's shell-based image |
| `rust-embed` 8 | `tower-http` ServeDir | ServeDir reads from filesystem at runtime — requires mounting assets into container; rust-embed bakes assets into the binary, producing a truly self-contained image |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Shell the `beb` CLI binary | Creates process exec attack surface, typed error handling is impossible, output parsing is brittle, version mismatch risks | Import `bitcoin-encrypted-backup` as a Rust crate dep |
| `sled` (any version) | Abandoned since 2021, never stable, documented 5 GB+ memory spikes, incompatible on-disk format across versions | `redb` 4.x or directory scan |
| `devices` feature of `bitcoin-encrypted-backup` | USB hardware wallet access doesn't reach StartOS containers; enables unnecessary async-hwi dependency | Omit feature entirely |
| `cli` feature of `bitcoin-encrypted-backup` | Pulls in `clap` and re-exports CLI entry points not needed in a library usage | Omit feature entirely |
| CDN-served fonts or scripts | App serves descriptors over Tor; any external fetch breaks privacy model and fails without internet | Bundle all assets via rust-embed; no external URLs in HTML |
| `axum-extra` crates | Pulls additional deps for features (typed headers, multipart, etc.) that are either not needed or easily implemented inline | Use axum core + tower-http only |
| `SvelteKit` | Designed for JS backends + SSR; requires disabling SSR and fighting adapter-static config for Rust backend | Plain Svelte 5 + Vite |
| `openssl` / `native-tls` | TLS termination is handled by StartOS at the network layer; including OpenSSL in the binary adds ~2 MB and a large C dependency surface | No TLS in the Rust binary; bind to plain HTTP on 8080 |
| `full` feature of tokio in production | Compiles all tokio subsystems (process, signal, net, time, etc.) unnecessarily | `rt-multi-thread` + `macros` + `io-util` only |
| Alpine base image | musl libc has known incompatibilities with some crates; shell in image is unnecessary attack surface | `gcr.io/distroless/cc-debian12` |
## Version Compatibility
| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `axum` 0.8.x | `tower-http` 0.6.x | Must match; axum 0.8 requires tower-http 0.6 |
| `axum` 0.8.x | `tokio` 1.x | Any tokio 1.x works; pin to LTS 1.51.x |
| `bitcoin-encrypted-backup` 1.0.0 | `miniscript` 0.12.3.5 (via feature) | Use feature `miniscript_12_3_5` (= `miniscript_latest`); do NOT mix with a separate `miniscript` dep at a different version or Cargo will unify incorrectly |
| `qrcode` 0.14.x | `image` 0.25.x | `qrcode` default features require `image`; pin `image` to 0.25 to match documented working combination |
| `rust-embed` 8.x | `axum` 0.8.x | rust-embed `axum-ex` feature targets axum 0.8; verify feature flag when bumping either |
| `redb` 4.x | No system deps | Pure Rust; compatible with distroless runtime |
| StartOS SDK TypeScript | Node 20+ | `package.json` in startos/ wrapper requires Node at build time; not in runtime image |
## Feature Flags: `bitcoin-encrypted-backup`
## Sources
- axum 0.8.8 — [crates.io/crates/axum](https://crates.io/crates/axum), [github.com/tokio-rs/axum](https://github.com/tokio-rs/axum) — HIGH confidence
- tokio 1.52.1 / LTS 1.51.x — [tokio.rs](https://tokio.rs), [docs.rs/crate/tokio/latest](https://docs.rs/crate/tokio/latest/source/README.md) — HIGH confidence
- tower-http 0.6.8 — [docs.rs/crate/tower-http/latest](https://docs.rs/crate/tower-http/latest) — HIGH confidence
- bitcoin-encrypted-backup 1.0.0, feature flags — [github.com/pythcoiner/encrypted_backup/blob/master/Cargo.toml](https://github.com/pythcoiner/encrypted_backup/blob/master/Cargo.toml) (fetched directly) — HIGH confidence
- rust-embed 8.8.0 — [crates.io/crates/rust-embed](https://crates.io/crates/rust-embed) — HIGH confidence
- redb 4.1.0 — [github.com/cberner/redb](https://github.com/cberner/redb), [redb.org](https://www.redb.org/) — HIGH confidence
- sled abandonment — multiple Rust forum threads and ecosystem surveys 2024-2025 — HIGH confidence
- qrcode 0.14 + image 0.25 — [crates.io/crates/qrcode](https://crates.io/crates/qrcode), [rust.code-maven.com](https://rust.code-maven.com/create-qrcode) — MEDIUM confidence (version pin derived from working example, verify on crates.io)
- distroless/cc-debian12 for Rust — [github.com/GoogleContainerTools/distroless](https://github.com/GoogleContainerTools/distroless), [oneuptime.com blog Jan 2026](https://oneuptime.com/blog/post/2026-01-07-rust-minimal-docker-images/view) — HIGH confidence
- Svelte vs SvelteKit for Rust backends — [github.com/sveltejs/kit/discussions/7526](https://github.com/sveltejs/kit/discussions/7526), Rust forum threads — MEDIUM confidence
- StartOS 0.4.0 packaging structure — [github.com/Start9Labs/ai-service-packaging](https://github.com/Start9Labs/ai-service-packaging), [github.com/Start9Labs/hello-world-startos](https://github.com/Start9Labs/hello-world-startos), [Start9Labs/start-os v0.4.0-beta release](https://github.com/Start9Labs/start-os/releases/tag/v0.4.0-beta.0) — MEDIUM confidence (beta; invoke `start9-packaging` skill at packaging phase for verified details)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
