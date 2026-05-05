# Architecture Research

**Domain:** Local web service (Rust + axum) wrapping a cryptographic Rust crate, packaged as s9pk for StartOS 0.4.0
**Researched:** 2026-05-05
**Confidence:** HIGH — all crate internals verified from source code at `/tmp/bed-test/encrypted_backup/src/`, StartOS SDK structure verified from local skill at `/workspace/.claude/skills/start9-packaging/`

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    StartOS 0.4.0 s9pk Container                     │
│  (single SubContainer: distroless/cc image, network ns shared)      │
├───────────────────────┬─────────────────────────────────────────────┤
│   SPA Frontend        │       Rust axum server (port 8080)          │
│   (vanilla JS / Svelte│                                              │
│    served as static   │  ┌─────────────────────────────────────┐   │
│    files from /assets)│  │         HTTP Handler Layer           │   │
│                       │  │  POST /api/encrypt                   │   │
│   browser ←→ 8080     │  │  POST /api/decrypt  (multipart)      │   │
│                       │  │  GET  /api/history                   │   │
│                       │  │  DEL  /api/history/:id               │   │
│                       │  │  GET  /                  (SPA HTML)  │   │
│                       │  │  GET  /assets/*          (static)    │   │
│                       │  └──────────────┬──────────────────────┘   │
│                       │                 │                            │
│                       │  ┌──────────────▼──────────────────────┐   │
│                       │  │        Encryption Service            │   │
│                       │  │  validate_descriptor()               │   │
│                       │  │  encrypt()  → Vec<u8> (binary .bed) │   │
│                       │  │  encrypt_armored() → String          │   │
│                       │  │  encrypt_qr()  → PNG bytes           │   │
│                       │  │  decrypt()  → cleartext descriptor   │   │
│                       │  │  [uses bitcoin-encrypted-backup]     │   │
│                       │  └──────────────┬──────────────────────┘   │
│                       │                 │                            │
│                       │  ┌──────────────▼──────────────────────┐   │
│                       │  │        Persistence Layer (opt-in)    │   │
│                       │  │  list_history()                      │   │
│                       │  │  save_bed(id, bytes)                 │   │
│                       │  │  delete_entry(id)                    │   │
│                       │  │  [files: /data/encrypted/*.bed]      │   │
│                       │  │  [metadata: SQLite /data/meta.db]    │   │
│                       │  └─────────────────────────────────────┘   │
├───────────────────────┴─────────────────────────────────────────────┤
│   Volume: main  →  /data/encrypted/*.bed + /data/meta.db            │
│   StartOS SDK:  MultiHost 8080 → Tor onion + LAN hostname           │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| SPA (static HTML/JS) | Render forms for encrypt/decrypt/history; call API; offer download / copy / QR display | HTTP GET/POST to axum |
| HTTP Handler Layer | Parse requests; validate inputs; marshal responses; return errors as JSON | Encryption Service, Persistence Layer |
| Encryption Service | Call `bitcoin-encrypted-backup` crate; validate descriptor BIP format; zeroize cleartext after use; produce binary, armored, QR output | `bitcoin-encrypted-backup` crate (`EncryptedBackup`) |
| Persistence Layer | Manage opt-in history: save `.bed` files, store metadata (id, timestamp, short-id) in SQLite, list and delete entries | Filesystem `/data/encrypted/`, SQLite |
| StartOS glue (`startos/`) | `manifest/index.ts`, `main.ts`, `interfaces.ts`, `backups.ts`, `versions/` — TypeScript SDK wrapper | StartOS 0.4.0 SDK |

---

## Recommended Project Structure

Two-repo layout (separate concerns, mirrors how all StartOS packages work):

```
# Repo 1: Rust service (semillabitcoin/bed-app)
bed-app/
├── Cargo.toml                  # workspace root
├── Dockerfile                  # rust:slim build → distroless/cc runtime
├── crates/
│   ├── core/                   # encryption service logic
│   │   ├── Cargo.toml          # depends on bitcoin-encrypted-backup, zeroize, secrecy
│   │   └── src/
│   │       ├── lib.rs          # pub mod encrypt; pub mod decrypt; pub mod validate;
│   │       ├── encrypt.rs      # encrypt() → (Vec<u8>, String, Vec<u8>)  binary/armored/qr
│   │       ├── decrypt.rs      # decrypt() → SecretBox<String>
│   │       ├── validate.rs     # check_descriptor() — <0;1>/* constraint
│   │       └── zeroize.rs      # ZeroizingDescriptor newtype wrapper
│   └── server/                 # axum HTTP server
│       ├── Cargo.toml          # depends on core, axum, tokio, tower-http, serde, sqlx, qrcode
│       └── src/
│           ├── main.rs         # tokio::main, Router::new(), serve on 0.0.0.0:8080
│           ├── routes/
│           │   ├── mod.rs
│           │   ├── encrypt.rs  # POST /api/encrypt handler
│           │   ├── decrypt.rs  # POST /api/decrypt handler (multipart)
│           │   └── history.rs  # GET/DEL /api/history handlers
│           ├── state.rs        # AppState: DB pool, history toggle
│           ├── persistence/
│           │   ├── mod.rs
│           │   └── history.rs  # SQLite via sqlx; list/save/delete
│           └── error.rs        # AppError → axum IntoResponse
├── frontend/                   # SPA (vanilla JS or Svelte)
│   ├── index.html
│   ├── app.js  (or src/ for Svelte)
│   └── style.css
└── assets/                     # compiled frontend, baked into Docker image

# Repo 2: StartOS packaging (semillabitcoin/bed-startos)
bed-startos/
├── Makefile
├── s9pk.mk
├── package.json
├── icon.svg
├── LICENSE
├── README.md
└── startos/
    ├── index.ts
    ├── sdk.ts
    ├── main.ts                 # SubContainer + Daemons, mountVolume main→/data
    ├── interfaces.ts           # MultiHost 8080 http → Tor + LAN
    ├── dependencies.ts         # no external deps
    ├── backups.ts              # ofVolumes(['main'])
    ├── utils.ts                # const APP_PORT = 8080
    ├── manifest/
    │   ├── index.ts            # id: 'bed-app', volumes: ['main'], images: {app: dockerTag}
    │   └── i18n.ts
    ├── actions/
    │   └── index.ts            # minimal (no actions in v1)
    ├── file-models/
    │   └── (empty v1)
    └── versions/
        └── v1.0.0.1.ts
```

### Structure Rationale

- **`crates/core/` separate from `crates/server/`:** Allows unit testing the encryption logic without spinning up HTTP. The core crate is the trust boundary — all cryptographic operations live here.
- **Two-repo split (bed-app + bed-startos):** Matches all Start9Labs packaging conventions. `bed-app` produces the Docker image pushed to GHCR; `bed-startos` produces the `.s9pk` referencing that tag. This also lets CI test the Rust service independently from packaging.
- **Frontend baked into Docker image:** Served as static files from axum's `tower-http::ServeDir`. No separate container, no CDN. Avoids bind-mount issues (StartOS preserves volumes but volatile container paths reset on update).
- **Single volume `main`:** Maps `/data/` inside the container. Contains `encrypted/*.bed` files and `meta.db`. No need for separate volumes in v1 — history is one logical unit.

---

## Architectural Patterns

### Pattern 1: ZeroizingDescriptor newtype

**What:** Wrap `String` in a newtype that implements `Drop` via the `zeroize` crate, so the cleartext descriptor is automatically overwritten when it goes out of scope.

**When to use:** Immediately upon receiving the descriptor string from the HTTP request body. The cleartext must never exist in a plain `String` beyond the scope of the encryption call.

**Trade-offs:** Requires explicitly calling `.expose_secret()` (from the `secrecy` crate) to read the inner value. This is intentional — it makes cleartext access auditable. The `Zeroize` impl for `String` zeroes the full capacity of the backing buffer but cannot guarantee no copies were made by prior reallocations, so the descriptor must be placed into the `SecretBox` as early as possible in the request lifecycle.

**Implementation:**

```rust
// In crates/core/src/zeroize.rs
use secrecy::{SecretBox, ExposeSecret};
use zeroize::Zeroize;

// Use secrecy::SecretString (= SecretBox<String>) directly:
pub type CleartextDescriptor = secrecy::SecretString;

// In the encrypt handler: receive as Bytes, convert immediately, never store as plain String
pub fn encrypt_descriptor(
    cleartext: secrecy::SecretString,
) -> Result<EncryptResult, EncryptError> {
    let descriptor = parse_descriptor(cleartext.expose_secret())?;
    let result = EncryptedBackup::new()
        .set_payload(&descriptor)?
        .encrypt()?;
    // cleartext drops and zeroizes here
    Ok(build_output(result))
}
```

The axum handler must avoid `String` for the descriptor field. Use `axum::body::Bytes` or `String` only transiently — immediately wrap in `SecretString` before any fallible operation.

### Pattern 2: Three-format output from a single encrypt call

**What:** Call `EncryptedBackup::encrypt()` once to get the raw `Vec<u8>`. Derive armored and QR from that same binary without re-encrypting.

**When to use:** The `POST /api/encrypt` endpoint returns a JSON response with all three representations, or the SPA fetches them individually. Calling encrypt once avoids nonce reuse risk and duplicate CPU work.

**Trade-offs:** Response payload is larger (JSON with base64 fields). Acceptable because descriptors are tiny (<2 KB).

```rust
// In routes/encrypt.rs
pub struct EncryptResponse {
    pub binary_b64: String,     // base64 of raw .bed bytes (for download)
    pub armored: String,        // -----BEGIN BITCOIN ENCRYPTED BACKUP----- block
    pub qr_png_b64: String,     // base64 PNG of the QR code
}

// Derive armored from binary:
fn to_armored(bytes: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!(
        "-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n{}\n-----END BITCOIN ENCRYPTED BACKUP-----",
        textwrap(b64, 64)
    )
}

// QR from armored base64 using `qrcode` crate → render to PNG via `image` crate
fn to_qr_png(bytes: &[u8]) -> Result<Vec<u8>, QrError> { ... }
```

### Pattern 3: Axum multipart for decrypt

**What:** The decrypt endpoint accepts `multipart/form-data` with two fields: `bed_file` (binary or armored text, up to ~50 KB) and `xpub` (text field). The `Multipart` extractor must be the last extractor in the handler signature.

**When to use:** `.bed` upload from disk, or paste of armored text. The `set_encrypted_payload()` method on `EncryptedBackup` auto-detects binary vs base64 armored by checking for the `BIPXXX` magic prefix.

```rust
// In routes/decrypt.rs
pub async fn decrypt_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<DecryptResponse>, AppError> {
    let mut bed_bytes: Option<Vec<u8>> = None;
    let mut xpub_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("bed_file") => bed_bytes = Some(field.bytes().await?.to_vec()),
            Some("xpub")    => xpub_str  = Some(field.text().await?),
            _ => {}
        }
    }
    // Parse xpub → secp256k1::PublicKey, call EncryptedBackup::new()
    //   .set_encrypted_payload(&bed_bytes)?.set_keys(vec![pk]).decrypt()
    // Return cleartext descriptor as plain text in JSON (not stored anywhere)
    ...
}
```

Size limit: set `DefaultBodyLimit` to 512 KB at the router level. A `.bed` for a multisig descriptor will never exceed a few KB, so this is conservative.

### Pattern 4: Opt-in history toggle via AppState

**What:** A boolean `history_enabled` flag lives in `AppState` (shared across handlers via `Arc<RwLock<AppState>>` or an `AtomicBool`). The SPA sends a toggle request to enable/disable. When enabled, the encrypt handler also calls `persistence::save_bed()`.

**When to use:** After delivering the encrypted output to the client. Persistence is a side-effect, not part of the encryption path — failure to persist must not cause the encryption response to fail.

**Trade-offs:** The toggle is in-memory, not persisted across restarts. If the user wants it always-on, a `file-models` config stored in `/data/config.json` would be needed. For v1, in-memory is acceptable and avoids config wizard complexity.

---

## Data Flow

### Encrypt Path

```
User pastes descriptor in SPA
    │
    ▼ POST /api/encrypt  { "descriptor": "wsh(...)", "save_history": bool }
    │
axum handler (routes/encrypt.rs)
    │
    ├─ 1. Receive descriptor as String
    │      Immediately wrap in SecretString (secrecy)
    │
    ├─ 2. validate_descriptor() → check <0;1>/* derivation constraint
    │      Reject with 422 if invalid
    │
    ├─ 3. core::encrypt_descriptor(cleartext: SecretString)
    │      ├─ parse to Descriptor<DescriptorPublicKey>  (miniscript)
    │      ├─ EncryptedBackup::new().set_payload(&descriptor)?.encrypt()
    │      │   → Vec<u8>  (binary .bed, ChaCha20-Poly1305, random nonce via rand feature)
    │      ├─ derive armored String from binary (base64 + PEM headers)
    │      ├─ derive QR PNG from binary (qrcode + image crates)
    │      └─ SecretString drops → zeroize() called on backing buffer
    │
    ├─ 4. If save_history AND history_enabled:
    │      persistence::save_bed(timestamp, short_id, bed_bytes)
    │        → write /data/encrypted/<ts>-<id>.bed
    │        → INSERT INTO history (id, ts, size) in SQLite
    │
    └─ 5. Return JSON { binary_b64, armored, qr_png_b64 }
         SPA offers three download/copy buttons
```

### Decrypt Path

```
User uploads .bed file (or pastes armored) + xpub in SPA
    │
    ▼ POST /api/decrypt  (multipart/form-data: bed_file, xpub)
    │
axum handler (routes/decrypt.rs)
    │
    ├─ 1. Read multipart fields: bed_bytes: Vec<u8>, xpub_str: String
    │
    ├─ 2. Parse xpub_str → DescriptorPublicKey → secp256k1::PublicKey
    │      Reject with 422 if invalid
    │
    ├─ 3. core::decrypt_bed(bed_bytes, public_key)
    │      ├─ EncryptedBackup::new()
    │      │     .set_encrypted_payload(&bed_bytes)?  ← auto-detects binary vs armored
    │      │     .set_keys(vec![pk])
    │      │     .decrypt()?
    │      └─ Returns Decrypted::Descriptor(d) → d.to_string() = cleartext descriptor
    │
    └─ 4. Return JSON { descriptor: String }
         SPA displays cleartext in a textarea for copy
         (cleartext NEVER written to disk or logged)
```

### History Paths

```
GET /api/history
    → persistence::list_history()
    → SELECT id, timestamp, size FROM history ORDER BY ts DESC
    → JSON [{ id, timestamp, size_bytes }]

DELETE /api/history/:id
    → persistence::delete_entry(id)
    → DELETE FROM history WHERE id = ?
    → fs::remove_file("/data/encrypted/<ts>-<id>.bed")
    → 204 No Content
```

---

## Memory Hygiene Strategy

**Crate:** `zeroize` (v1.8+) + `secrecy` (v0.10+)

**Approach:**

1. **Never use `String` for the descriptor in flight.** The axum JSON extractor will deserialize into `String` — that is unavoidable for a single tick. Immediately move it into `secrecy::SecretString` (which wraps `zeroize::Zeroizing<String>`) before any `?` operator can cause an early return.

2. **`EncryptedBackup` struct does not implement `Zeroize`.** The crate's `Payload::Encrypt { payload: Vec<u8> }` field holds the cleartext bytes during the encrypt call. This `Vec<u8>` is stack/heap data that is dropped when `encrypt()` consumes `self`. The bytes may linger in heap memory until the allocator reclaims them. To mitigate: call `zeroize::Zeroize::zeroize()` explicitly on any intermediate `Vec<u8>` that contained cleartext before dropping it, or use `zeroize::Zeroizing<Vec<u8>>` as a wrapper.

3. **Axum request body.** The raw body `Bytes` from the JSON extractor also contains the cleartext. After parsing, call `drop(body_bytes)` explicitly. There is no practical way to zeroize `axum::body::Bytes` (it is a ref-counted pointer). Accept this as a residual risk — it is consistent with the stated threat model: "does not protect against compromise of StartOS during encryption."

4. **No disk writes of cleartext.** The `save_bed` path only sees `Vec<u8>` (the already-encrypted blob). The cleartext descriptor path never touches disk or SQLite.

5. **Logging.** Configure `tracing` to never log request bodies. Use `tower_http::trace::TraceLayer` with a custom `make_span_with` that omits body data.

**Confidence:** MEDIUM — the `secrecy` + `zeroize` strategy is well-established in Rust crypto crates (it is the same approach used by `age`, `sequoia-pgp`, and Bitcoin hardware wallet tooling). The residual risk from axum `Bytes` is real but acknowledged in the threat model.

---

## StartOS 0.4.0 Manifest Considerations

This section is grounded in the local skill at `/workspace/.claude/skills/start9-packaging/`.

### `startos/manifest/index.ts`

```ts
export const manifest = setupManifest({
  id: 'bed-app',
  title: 'Bitcoin Encrypted Backup',
  license: 'MIT',
  packageRepo: 'https://github.com/semillabitcoin/bed-startos',
  upstreamRepo: 'https://github.com/pythcoiner/encrypted_backup',
  marketingUrl: null,
  donationUrl: null,
  docsUrls: [],
  description: { short, long },

  volumes: ['main'],           // /data inside container — .bed files + meta.db

  images: {
    app: {
      source: {
        dockerTag: 'ghcr.io/semillabitcoin/bed-app:1.0.0',
        // built with: rust:slim → distroless/cc, target ~5-10 MB
      },
      arch: ['x86_64', 'aarch64'],
    },
  },

  alerts: { install: null, update: null, uninstall: null,
            restore: null, start: null, stop: null },
  dependencies: {},   // no external deps
})
```

### `startos/interfaces.ts`

Single port, single MultiHost. StartOS auto-generates Tor onion + LAN hostname for HTTP.

```ts
export const setInterfaces = sdk.setupInterfaces(async ({ effects }) => {
  const multi = sdk.MultiHost.of(effects, 'main')
  const origin = await multi.bindPort(APP_PORT, { protocol: 'http' })

  const ui = sdk.createInterface(effects, {
    name: i18n('Web Interface'),
    id: 'webui',
    description: i18n('Encrypt and decrypt Bitcoin descriptors'),
    type: 'ui',
    schemeOverride: null,
    masked: false,
    username: null,
    path: '',
    query: {},
  })

  return [await origin.export([ui])]
})
```

`APP_PORT = 8080` defined in `utils.ts`. There is no clearnet exposure — StartOS will generate Tor `.onion` and `bed-app.local` (mDNS LAN) automatically from this single `bindPort` declaration.

### `startos/main.ts` — Volume Mount and Health Check

```ts
export const main = sdk.setupMain(async ({ effects }) => {
  const mounts = sdk.Mounts.of()
    .mountVolume({
      volumeId: 'main',
      subpath: null,
      mountpoint: '/data',
      readonly: false,
    })

  const sub = await sdk.SubContainer.of(
    effects,
    { imageId: 'app' },
    mounts,
    'bed-app-sub',
  )

  return sdk.Daemons.of(effects).addDaemon('primary', {
    subcontainer: sub,
    exec: {
      command: ['/usr/local/bin/bed-server'],
      env: { DATA_DIR: '/data' },
    },
    ready: {
      display: i18n('Web Interface'),
      fn: () =>
        sdk.healthCheck.checkPortListening(effects, APP_PORT, {
          successMessage: i18n('Service is ready'),
          errorMessage: i18n('Service is not listening'),
        }),
    },
    requires: [],
  })
})
```

`checkPortListening` is the correct health check for a TCP server — it avoids HTTP-level calls that could fail during startup before axum binds. This is the recommended approach for simple services per the skill.

### `startos/backups.ts`

```ts
export const { createBackup, restoreInit } = sdk.Backups.of()
  .addVolume('main')
  .build()
```

This backs up the entire `main` volume (`.bed` history files + `meta.db`). The cleartext descriptor is never in the volume, so backup exposure is only the encrypted blobs — consistent with the threat model.

### GHCR Image Visibility

The Docker image `ghcr.io/semillabitcoin/bed-app` must be made **public** after the first `docker push`. If the package remains private, StartOS will fail to pull the image during install. This is a known pitfall from user memory (`feedback_ghcr_private_default.md`).

---

## Suggested Build Order

Dependencies between components dictate this order:

```
Phase 1 — CLI / crate spike
    ├─ Clone bitcoin-encrypted-backup source (already done: /tmp/bed-test)
    ├─ Write crates/core with encrypt/decrypt wrappers
    ├─ Write a minimal binary (crates/server/src/main.rs) that just calls core functions
    ├─ Verify descriptor round-trip end-to-end
    └─ Verify <0;1>/* validation rejects invalid descriptors
    Rationale: crate API is the unknown. Spike confirms it compiles and works
               before investing in HTTP or frontend.

Phase 2 — axum HTTP backend
    ├─ Add axum, tokio, tower-http to crates/server
    ├─ Implement POST /api/encrypt (JSON in, JSON out)
    ├─ Implement POST /api/decrypt (multipart)
    ├─ Stub history endpoints (return empty list / 204)
    ├─ Serve a placeholder index.html on GET /
    └─ Integration test with curl / httpie
    Rationale: backend must exist before frontend has anything to call.

Phase 3 — SPA frontend
    ├─ Encrypt tab: textarea + three output buttons (download binary, copy armored, download QR)
    ├─ Decrypt tab: file upload + xpub field + display result
    ├─ History tab: list entries, delete button
    └─ Compile/bundle into assets/ baked into Dockerfile
    Rationale: SPA depends on stable API contract. Build backend first.

Phase 4 — Persistence layer
    ├─ Add sqlx + SQLite to crates/server
    ├─ Implement history table schema + migrations
    ├─ Wire save_bed() into encrypt handler
    ├─ Wire list/delete into history handlers
    └─ Test with in-container SQLite file at /data/meta.db
    Rationale: history is opt-in and isolated from core flow.
               Do not block earlier phases on it.

Phase 5 — Dockerfile + GHCR
    ├─ Multi-stage: rust:slim (build) → distroless/cc (runtime)
    ├─ Copy assets/ into image at /assets (baked, not bind-mounted)
    ├─ Push to ghcr.io/semillabitcoin/bed-app
    └─ Make package public immediately after first push
    Rationale: can't package s9pk until image exists and is pullable.

Phase 6 — s9pk packaging (bed-startos repo)
    ├─ scaffold from hello-world-startos
    ├─ Implement manifest, main, interfaces, backups, versions
    ├─ `make arm` / `make x86` / `make universal`
    ├─ Sideload on real StartOS device (start-cli package install -s)
    └─ Smoke test: install → open Tor URL → encrypt → decrypt → verify QR
    Rationale: packaging is the integration layer. Must test on real StartOS,
               not just in docker run locally (StartOS sanitizes mounts differently).

Phase 7 — Hardening
    ├─ Memory zeroize audit (trace cleartext paths)
    ├─ Tracing config to never log request bodies
    ├─ Threat model documentation in README
    └─ Size audit of Docker image (target ≤10 MB compressed)
```

---

## Anti-Patterns

### Anti-Pattern 1: Shelling out to the `beb` CLI binary

**What people do:** `Command::new("beb").arg("encrypt").arg("...").output()`

**Why it's wrong:** Passes the descriptor through process arguments (visible in `/proc/*/cmdline`), requires the `beb` binary compiled with `cli` feature adding unnecessary dependencies, loses typed error handling, and cannot implement zeroize on data that left the process.

**Do this instead:** Import `bitcoin-encrypted-backup` as a library crate dependency and call `EncryptedBackup::new()` directly.

### Anti-Pattern 2: Storing cleartext descriptor in the history or logs

**What people do:** Log the full request body for debugging; accidentally add descriptor to a history metadata table.

**Why it's wrong:** Violates the core security promise. The history schema must only contain: `id`, `timestamp`, `file_size`. Never `descriptor`, never `xpub`.

**Do this instead:** Log request metadata only (endpoint, status code, duration). Use `tracing` with `tower_http::trace::TraceLayer` configured without body logging.

### Anti-Pattern 3: Bind-mounting the frontend from `${APP_DATA_DIR}` at runtime

**What people do:** Serve the SPA from a volume path rather than baking it into the image, trying to allow "live updates" of the UI.

**Why it's wrong:** StartOS volumes are persistent but the container filesystem resets on update. If the SPA is only in the volume, a fresh install before the first mount populates it will serve nothing. See `feedback_umbrel_app_data_preservation.md`.

**Do this instead:** Bake the compiled frontend into the Docker image at build time (`COPY frontend/dist /assets`). Axum serves it via `tower-http::ServeDir`.

### Anti-Pattern 4: Calling `EncryptedBackup::new()` without the `rand` feature

**What goes wrong:** Without the `rand` feature, `encrypt()` requires a caller-supplied `nonce: [u8; 12]`. If the caller supplies a static or predictable nonce, the encryption is broken (nonce reuse with ChaCha20-Poly1305 leaks the keystream).

**Do this instead:** Keep the `rand` feature enabled (it is in `default`). Verify the `Cargo.toml` dependency includes `features = ["rand"]` explicitly. Do not disable `default-features` unless consciously re-enabling `rand`.

### Anti-Pattern 5: Exposing the service on clearnet

**What people do:** Declare a `publicDomain` interface in `interfaces.ts` for convenience.

**Why it's wrong:** The service processes Bitcoin descriptor plaintext. Clearnet exposure adds an attack surface with no benefit — users access it over Tor or LAN only.

**Do this instead:** Use only one `MultiHost` binding on `port 8080, protocol: 'http'`. StartOS generates Tor + LAN automatically. Do not add a second MultiHost for clearnet.

---

## Scaling Considerations

This is a single-user local service on a personal server. Scaling is not a concern. The only resource consideration:

| Concern | At 1 user (typical) | Notes |
|---------|---------------------|-------|
| Memory | <50 MB RSS | axum is lean; zeroize means cleartext does not accumulate |
| Disk (history) | <1 MB for hundreds of `.bed` files | Each `.bed` is <2 KB for a multisig descriptor |
| CPU | Negligible | ChaCha20-Poly1305 on <2 KB input is sub-millisecond |
| Startup time | <1 second | distroless binary, no JVM/runtime |

QR code generation (`qrcode` + `image` crates) is the most CPU-intensive operation and still completes in <10 ms for this payload size.

---

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| SPA ↔ axum | HTTP/JSON + multipart | Same origin (8080), no CORS needed |
| axum handler ↔ core (encryption) | direct Rust function call | In same crate tree; no IPC |
| core ↔ `bitcoin-encrypted-backup` | direct crate import | Library linkage, no subprocess |
| axum handler ↔ persistence | direct Rust function call (async) | sqlx + tokio runtime |
| SQLite ↔ filesystem | standard file I/O at `/data/meta.db` | In `main` volume |

### External Services

None. This service is intentionally air-gapped from external services (no RPC to Bitcoin Core, no network calls after startup).

---

## Sources

- `bitcoin-encrypted-backup` source code: `/tmp/bed-test/encrypted_backup/src/lib.rs` (confirmed crate version 1.0.0, real API)
- StartOS SDK manifest reference: `/workspace/.claude/skills/start9-packaging/references/manifest.md`
- StartOS SDK interfaces reference: `/workspace/.claude/skills/start9-packaging/references/interfaces.md`
- StartOS SDK main/daemons reference: `/workspace/.claude/skills/start9-packaging/references/main-daemons.md`
- StartOS SDK anatomy reference: `/workspace/.claude/skills/start9-packaging/references/anatomy.md`
- `zeroize` crate docs: https://docs.rs/zeroize/latest/zeroize/
- `secrecy` crate docs: https://docs.rs/secrecy/0.10.3/secrecy/
- axum multipart docs: https://docs.rs/axum/latest/axum/extract/multipart/index.html
- User project memory: `/workspace/.claude/projects/-home-anon/memory/project_bed_start9.md`

---
*Architecture research for: BED Start9 App (Rust + axum + bitcoin-encrypted-backup → s9pk)*
*Researched: 2026-05-05*
