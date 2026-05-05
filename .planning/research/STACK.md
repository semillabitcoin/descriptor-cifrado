# Stack Research

**Domain:** Local-first Rust web service — Bitcoin descriptor encryption/decryption, StartOS 0.4.0 s9pk app
**Researched:** 2026-05-05
**Confidence:** MEDIUM-HIGH (core Rust/axum stack HIGH; StartOS 0.4.0 packaging MEDIUM — still in beta release cycle)

---

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

**Svelte vs. Vanilla JS decision:** Svelte is recommended over vanilla JS because the UI has meaningful state (file upload, toggle modes, QR display, history list). Vanilla JS becomes error-prone at that scope. Svelte produces bundles under 50 KB for this use case. If the team prefers zero build tooling, vanilla JS with `include_str!` works but requires careful DOM management.

### Persistence (history mode, opt-in)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `redb` | 4.1.0 (Apr 2026) | Key-value metadata store for `.bed` history | Pure Rust, stable 1.0+ release, ACID transactions, MVCC, zero-copy reads. Actively maintained. Stores timestamp + short-id + filename as key; no value needed (the `.bed` file is on disk). Replaces sled, which is effectively abandoned since 2021 and never reached stable. |

**Why not sled:** Abandoned since 2021, never stable, documented memory spikes to 5 GB+ on random access workloads, incompatible API across versions. Do not use.

**Why not rusqlite:** SQLite via FFI (C linkage) complicates the distroless runtime image — needs `libsqlite3.so` or the `bundled` feature which adds build complexity. redb is pure Rust with no system deps.

**Why not the filesystem alone:** A flat directory listing of `/data/encrypted/` is sufficient for v1 without any embedded DB. If the history feature only needs list + delete (no queries), consider deferring redb entirely and scanning the directory. Add redb only when richer metadata queries are needed.

### QR Code Generation

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `qrcode` | 0.14.x | Generate QR PNG from base64-armored payload | Integrates with `image` crate for PNG output; widely used, straightforward API. |
| `image` | 0.25.x | Encode QR matrix to PNG bytes | Required by `qrcode` for PNG rendering; use `ImageOutputFormat::Png` to get bytes for HTTP response. |

**Why not `qrcodegen`:** Development has stalled; does not produce PNG natively (raw pixels only). `qrcode` is the standard choice for PNG output in Rust.

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

---

## Installation

```toml
# Cargo.toml (production)
[dependencies]
axum = "0.8"
tokio = { version = "1.51", features = ["rt-multi-thread", "macros", "io-util"] }
tower-http = { version = "0.6", features = ["fs", "compression-gzip", "compression-zstd", "trace"] }
rust-embed = { version = "8", features = ["axum-ex"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
qrcode = "0.14"
image = { version = "0.25", default-features = false, features = ["png"] }
uuid = { version = "1", features = ["v4"] }
redb = "4"                         # only if history mode needs metadata queries

# bitcoin-encrypted-backup — git dep (not on crates.io)
[dependencies.bitcoin-encrypted-backup]
git = "https://github.com/pythcoiner/encrypted_backup"
rev = "<pin to specific commit hash>"
default-features = false
features = ["miniscript_12_3_5", "rand", "base64"]
# Do NOT enable: "devices", "cli", "tokio" (tokio is only needed by devices)

[dev-dependencies]
tokio = { version = "1.51", features = ["full"] }  # full only for tests
```

```bash
# Frontend (build-time only, not in Docker runtime)
cd frontend
npm install
npm run build   # outputs to frontend/dist/
```

---

## Docker Build Pipeline

```dockerfile
# Stage 1: Rust build
FROM rust:1.80-slim AS builder
WORKDIR /app

# Frontend build (Node only needed here)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
  && apt-get install -y nodejs
COPY frontend/ frontend/
RUN cd frontend && npm ci && npm run build

# Rust build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Stage 2: Distroless runtime
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/bed-startos /bed-startos
EXPOSE 8080
ENTRYPOINT ["/bed-startos"]
```

**Why `distroless/cc-debian12`:** Provides glibc (required for the dynamically-linked Rust binary) with no shell, no package manager, no apt. Attack surface is minimal. Expected image size: ~15–25 MB (Rust binary ~10 MB + distroless base ~5 MB).

**Why not Alpine/musl:** `distroless/cc` avoids musl incompatibilities. The `bitcoin-encrypted-backup` crate uses `chacha20poly1305` which is pure Rust, so no C crypto libs are needed — glibc link is only for the Rust std itself.

**Multi-arch:** Build with `docker buildx build --platform linux/amd64,linux/arm64`. Use `cross` crate to cross-compile ARM target on x86 CI.

---

## StartOS 0.4.0 Packaging

**Status:** StartOS 0.4.0 is in beta as of May 2026. The packaging workflow is confirmed but some details may shift before stable release. Confidence: MEDIUM.

**SDK:** TypeScript SDK (`start-sdk`). The packaging layer is TypeScript, not Rust. The Rust binary is the container workload; TypeScript files in `startos/` define manifest, interfaces, health checks, and actions for StartOS.

**Key project structure:**
```
bed-startos/
├── Dockerfile
├── icon.svg                    # max 40 KiB
├── LICENSE
├── Makefile
├── s9pk.mk                     # boilerplate from hello-world-startos
├── package.json
├── tsconfig.json
├── assets/
│   └── instructions.md
└── startos/
    ├── manifest/
    │   ├── index.ts            # setupManifest() call
    │   └── i18n.ts             # localized strings
    ├── interfaces.ts           # Tor onion + LAN port binding
    ├── main.ts                 # daemon health check loop
    ├── backups.ts
    ├── init/
    └── index.ts
```

**Manifest key fields (TypeScript, not YAML):**
```typescript
// startos/manifest/index.ts
setupManifest({
  id: "bed-startos",
  title: { en: "Bitcoin Encrypted Backup" },
  license: "MIT",
  wrapperRepo: "https://github.com/semillabitcoin/bed-startos",
  upstreamRepo: "https://github.com/pythcoiner/encrypted_backup",
  // ...
  volumes: ["main"],
  images: {
    main: {
      source: { dockerTag: "ghcr.io/semillabitcoin/bed-startos:latest" },
      arch: ["x86_64", "aarch64"],
    },
  },
})
```

**Interfaces — Tor + LAN:**
- Tor: StartOS auto-generates the onion address; developer just declares the interface with `protocol: 'http'` and internal port 8080.
- LAN: Declare `type: 'ui'` interface on port 8080 with `ssl: false` (StartOS handles TLS termination at the LAN layer for `.local` URLs).

**Build command:** `start-sdk pack` then `start-sdk verify s9pk bed-startos.s9pk`

**Version format (Exver):** `1.0.0:0` — upstream semver `:` wrapper semver. Example: `1.0.0:1` for a packaging fix without upstream changes.

**Reference:** Fork `Start9Labs/hello-world-startos` as the scaffold. The `update/040` branch tracks the 0.4.0 format. Do NOT use the `master` branch which targets 0.3.x.

**GHCR:** Push image to `ghcr.io/semillabitcoin/bed-startos`. Make the package public immediately after first push or StartOS fails to pull. (Confirmed constraint from project memory.)

---

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

---

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

---

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

---

## Feature Flags: `bitcoin-encrypted-backup`

This is critical to get right. The crate's default features (`miniscript_latest`, `rand`, `base64`) are all acceptable. However:

```toml
[dependencies.bitcoin-encrypted-backup]
git = "https://github.com/pythcoiner/encrypted_backup"
default-features = false
features = [
  "miniscript_12_3_5",  # = miniscript_latest; pins to 0.12.3.5
  "rand",               # required for nonce generation
  "base64",             # required for armored output format
  # "miniscript_12_0"   # DO NOT enable alongside 12_3_5 — they conflict
  # "devices"           # DO NOT enable — USB HW wallets don't reach StartOS containers
  # "cli"               # DO NOT enable — pulls clap, exposes CLI entry points
  # "tokio"             # DO NOT enable separately — only used by devices feature
]
```

Do NOT add a separate `miniscript` dependency to `Cargo.toml`. The crate manages its own miniscript version internally via Cargo rename (`mscript_12_3_5`). Adding an independent `miniscript` dep risks Cargo feature unification errors.

---

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

---

*Stack research for: BED Start9 App — Bitcoin Encrypted Backup s9pk service*
*Researched: 2026-05-05*
