# Phase 1: Crypto Core + HTTP API — Research

**Researched:** 2026-05-05
**Domain:** Rust (axum 0.8 + tokio LTS) HTTP service wrapping `bitcoin-encrypted-backup` crate; descriptor cipher core + binary/armored/QR outputs
**Confidence:** HIGH (crate API verified locally at `/tmp/bed-test/encrypted_backup/src/`; axum/zeroize/cargo-deny patterns from official docs; HEAD SHA cross-verified via GitHub Atom + REST)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Repo & Workspace**
- **D-01:** Repo único `semillabitcoin/bed-app`. Repo `semillabitcoin/bed-startos` se inicializa en Phase 4.
- **D-02:** Cargo workspace con dos crates: `crates/core` (cripto puro, unit-testable sin HTTP) + `crates/server` (axum). Razón: separar trust boundary de HTTP layer.
- **D-03:** Phase 1 incluye repo init: `Cargo.toml` raíz workspace + dos crates skeleton + `Cargo.lock` commiteado + `deny.toml` + `.github/workflows/ci.yml` + `rust-toolchain.toml` (canal `stable` pinneado). **NO** Dockerfile (Phase 3).
- **D-04:** Pinning de la crate `bitcoin-encrypted-backup`: git dep con `rev = "<commit-SHA>"` exacto (no tag, no branch). El SHA exacto se documenta en comentario de `Cargo.toml`.

**API Surface**
- **D-05:** `POST /api/encrypt` — request `Content-Type: application/json`, body `{"descriptor": "<string>"}`. Response 200 JSON único: `{"bed_b64", "armored", "qr_png_b64"}`.
- **D-06:** `POST /api/decrypt` — `Content-Type: multipart/form-data`. Campos: `bed` (texto armored OR file binario `.bed`) + `xpub` (texto OR file). Response 200 JSON `{"descriptor": "<cleartext>"}`.
- **D-07:** Endpoint contract estable a partir de Phase 1; cambios requieren bump de version path (`/api/v2/...`).

**Validación BIP**
- **D-08:** Validación `<0;1>/*` multipath: iterar `desc.iter_pk()` y exigir multipath con índices `0` y `1`. Rechazar `Wildcard::None`, wildcard simple `/*` sin multipath, y multipath con índices ≠ `<0;1>`.
- **D-09:** Error tipado `AppError::MissingMultipathWildcard` → HTTP 422 con mensaje en castellano: *"El descriptor debe incluir derivación `<0;1>/*` en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain."*

**Memoria & Zeroize**
- **D-10:** `Zeroizing<String>` se aplica al deserializar el body, antes de cualquier `?` early-return. Pasar por `&mut` ref a través de `validate → encrypt`. Tras `encrypt()`, `.zeroize()` explícito y `drop()` inmediato.
- **D-11:** Newtype `ZeroizingDescriptor` en `crates/core/src/zeroize.rs` envolviendo `Zeroizing<String>`, sin `Clone`/`Display`/`Debug`.

**Armored Format**
- **D-12:** La crate **NO provee armored**. Implementar wrapper en `crates/core/src/armored.rs`:
  - Encode: `Vec<u8>` → base64 → line-wrap 64 chars → `-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n...\n-----END BITCOIN ENCRYPTED BACKUP-----\n`
  - Decode: tolerante a whitespace, indentación, `\r\n` vs `\n`. Strip headers + whitespace + base64 decode.
- **D-13:** Cross-implementation round-trip antes de cerrar fase (vs `pythcoiner/bed` o Liana v13).

**QR**
- **D-14:** `qrcode 0.14` + `image 0.25`. ECC level **L** (máxima capacidad). Modo byte sobre el armored completo (incluye headers). PNG output.
- **D-15:** Si `armored.len() > 2900` bytes → HTTP 422 con `AppError::QrTooLarge`.

**Errores**
- **D-16:** Single `AppError` enum (`thiserror`) en `crates/server/src/error.rs` con `impl IntoResponse`. Variantes: `MissingMultipathWildcard`, `DescriptorParse`, `XpubMismatch`, `QrTooLarge`, `Internal`. Status:
  - `MissingMultipathWildcard | DescriptorParse | QrTooLarge | XpubMismatch` → 422
  - JSON malformado / multipart inválido → 400
  - Panic / internal → 500 con body genérico `{"error":{"code":"INTERNAL","message":"internal error"}}`
- **D-17:** Response body uniforme: `{"error": {"code": "<UPPER_SNAKE>", "message": "<castellano>"}}`.

**Tracing & Logging**
- **D-18:** `tracing-subscriber::fmt()` con `EnvFilter::from_default_env()` por defecto en `INFO`. Format: `with_target(false)`, `with_level(true)`. JSON solo si `BED_LOG_FORMAT=json` (defer si no hace falta).
- **D-19:** `TraceLayer::new_for_http()` SOLO a rutas no-sensibles. Encrypt/decrypt handlers `#[tracing::instrument(skip_all)]`. Span fields permitidos: `method`, `path`, `status`, `duration_ms`. NUNCA bodies / `Authorization` / params.
- **D-20:** Test no-leak (CI-02) en `crates/server/tests/no_leak.rs`: `tracing_subscriber::fmt::TestWriter` + buffer compartido; assert que `descriptor_str` no aparece como substring.

**Panic Hook**
- **D-21:** En `main.rs`: `std::panic::set_hook(Box::new(|_info| { tracing::error!("internal panic"); }))`. Descarta `PanicInfo` entero. `RUST_BACKTRACE` nunca se setea.
- **D-22:** Cero `unwrap()`/`expect()` en path de request. CI con `clippy::unwrap_used` + `clippy::expect_used` en `deny` para `crates/server/src/`.

**Test Infra**
- **D-23:** `tower::ServiceExt::oneshot` + `axum::body::to_bytes` (in-process). NO `axum-test`, NO `reqwest`.
- **D-24:** Round-trip test (CI-02) en `crates/server/tests/round_trip.rs`: descriptor fixture → encrypt → decrypt con xpub correcto → assert equals. Variante con xpub incorrecto → 422.
- **D-25:** Property-based test en `crates/core/tests/validate.rs`: bare xpub, single wildcard `/*`, multipath `<2;3>` → assert error.

**Bind & TLS**
- **D-26:** `127.0.0.1:8080`. Constante `const BIND_ADDR: &str = "127.0.0.1:8080";`. NO leer de env en Phase 1.
- **D-27:** `tokio` features mínimos: `rt-multi-thread`, `macros`, `io-util`. Evitar `full`. Evitar `tokio-util` salvo que un test lo requiera.

**CI Workflow**
- **D-28:** `.github/workflows/ci.yml` con jobs: `fmt`, `clippy`, `test`, `audit`, `deny`.
- **D-29:** Trigger: `pull_request` + `push` a `main`. Runner: `ubuntu-latest`. Paralelo cuando posible. Timeout 15 min.
- **D-30:** `deny.toml` declara: `licenses.allow = [MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, CC0-1.0, Unicode-DFS-2016, MPL-2.0]`; `bans.deny = [{name="openssl-sys"}, {name="native-tls"}]`; `advisories.vulnerability = "deny"`.

### Claude's Discretion
- Layout exacto de archivos dentro de cada crate (subdirs `routes/`, `handlers/`, etc.).
- Names exactos de variantes en `AppError` enum (manteniendo contract del campo `code`).
- Versions exactas de `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber` (latest patch del minor especificado).
- `proptest` o `quickcheck` para property tests.
- Estructura interna del armored encoder.

### Deferred Ideas (OUT OF SCOPE)
- Drag-and-drop archivos (UX2-01) — Phase 2 / v2.
- "Test decrypt" round-trip transparente (UX2-02) — v2.
- Display de checksum del descriptor recuperado (UX2-03) — v2.
- Mensajes específicos por tipo de error de sintaxis del descriptor (UX2-04) — v2.
- Persistencia del toggle "guardar historial" (PERS-01) — v1.x con `file-models`.
- JSON log format vía env var (parte de D-18) — defer salvo que CI lo requiera.
- History endpoints stub en Phase 1 — descartado.
- `config.json` para override de bind addr — descartado.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CORE-01 | Crate `bitcoin-encrypted-backup` con features `miniscript_12_3_5`, `rand`, `base64`, sin `devices`/`cli`/`tokio`, pinneada a commit/rev exacto | §Standard Stack (Cargo.toml + verified rev SHA `17b69b71cd1e005f80f5e81147795df0d11db027`) |
| CORE-02 | Round-trip determinista (encrypt → decrypt con xpub válida) cubierto por test automatizado | §Crate API Surface — `EncryptedBackup::new().set_payload().encrypt()` → `set_encrypted_payload().set_keys().decrypt()`; fixtures en `/tmp/bed-test/` |
| CORE-03 | Capa core valida derivación `<0;1>/*` y rechaza descriptors inválidos con error tipado | §Wildcard Validation — `descriptor.for_each_key()` + `DescriptorPublicKey::MultiXPub.derivation_paths` matching |
| CORE-04 | Descriptor envuelto en `Zeroizing<String>` desde parse y zeroizado tras operación | §Zeroize Pattern — `Zeroizing<String>` con `&mut` ref a través de pipeline |
| CORE-05 | Sin `unwrap()`/`expect()` en path de request; panic hook genérico | §Pitfalls #3 — `clippy::unwrap_used` deny + `set_hook` discarding `PanicInfo` |
| ENC-01 | `POST /api/encrypt` (JSON) acepta descriptor y devuelve los tres formatos | §axum Patterns — `Json<EncryptRequest>` extractor + `Json<EncryptResponse>` con tres campos |
| ENC-02 | Salida binaria `.bed` descargable | §Three-format output — `bed_b64` field (base64 del `Vec<u8>` raw) |
| ENC-03 | Salida armored estilo PGP con cabeceras `BEGIN/END BITCOIN ENCRYPTED BACKUP` | §Armored Format Pattern (D-12, custom impl en `crates/core/src/armored.rs`) |
| ENC-04 | Salida QR PNG; si excede capacidad QR ECC-L (~2,900 B) error descriptivo | §QR Sizing Math — `QrCode::with_version(data, Version::Normal(40), EcLevel::L)` máx 2,953 B byte mode |
| ENC-05 | UI muestra errores de validación inline y específicos | §Error Pattern (`AppError` con códigos UPPER_SNAKE) |
| DEC-01 | `POST /api/decrypt` (multipart) acepta `.bed` (binario o armored) + xpub | §axum Multipart Pattern — `Multipart::next_field()` loop |
| DEC-02 | Aceptar pegado armored o subida binario indistintamente | §Auto-detect — crate's `set_encrypted_payload()` detecta magic `BIPXXX` o base64 |
| DEC-03 | Aceptar xpub pegado o archivo | §Multipart fields — text + bytes branches sobre `field.content_type()`/`field.file_name()` |
| DEC-04 | Descriptor recuperado mostrado con copy-clipboard y nunca persistido | §Decrypt path — `Decrypted::Descriptor` → `to_string()` → JSON response, sin disk write |
| DEC-05 | Parser tolera espacios/indentación en armored pegado | §Armored Decoder — strip whitespace antes de base64 decode (verificado pattern) |
| SEC-01 | TraceLayer `skip_all` en handlers sensibles; test no-leak | §Tracing Pattern — `#[tracing::instrument(skip_all)]` + `TestWriter` capture |
| SEC-02 | Servidor binda en `127.0.0.1:8080` | §Bind Constant — `TcpListener::bind("127.0.0.1:8080")` |
| SEC-03 | `rustls` en todo lugar; `cargo deny` rechaza `openssl-sys`/`native-tls` | §deny.toml template — bans entries |
| CI-01 | Pipeline corre `cargo audit` + `cargo deny` y falla en vulnerabilidades/licencias | §GitHub Actions YAML — jobs `audit` + `deny` |
| CI-02 | Pipeline corre round-trip + no-leak | §GitHub Actions YAML — job `test` ejecuta `cargo test --all-features --workspace` |
</phase_requirements>

## Summary

Phase 1 wraps the verified `bitcoin-encrypted-backup` crate (HEAD `17b69b71cd1e005f80f5e81147795df0d11db027` on `master`, version `1.0.0`) inside an axum 0.8 service exposing two endpoints. The crate has been read end-to-end at `/tmp/bed-test/encrypted_backup/src/` so the public API is fully known: `EncryptedBackup::new() → set_payload(&Descriptor) → encrypt() → Vec<u8>` for the encrypt path; `EncryptedBackup::new() → set_encrypted_payload(&[u8]) → set_keys(Vec<PublicKey>) → decrypt() → Decrypted::Descriptor` for the decrypt path. The crate auto-detects binary vs. base64-encoded payloads via the `BIPXXX` magic prefix at `set_encrypted_payload`, which simplifies the multipart handler.

Three Phase 1 problem areas need careful planning beyond what the crate provides:

1. **Multipath wildcard validation.** The crate accepts any descriptor with at least one key (rejecting only NUMS); it does not enforce the `<0;1>/*` BIP rule. The validation must be implemented in `crates/core` by walking `descriptor.for_each_key(...)` and confirming each key is `DescriptorPublicKey::MultiXPub` with `derivation_paths` matching exactly `[0/*, 1/*]` (after the origin path, applying the `Wildcard::Unhardened` suffix).
2. **Armored wrapper.** The crate exposes `encrypt_base64()` (single-line base64 string) but no PEM-style armored output. Phase 1 implements `crates/core/src/armored.rs` with encoder (line-wrap 64 chars + `-----BEGIN/END BITCOIN ENCRYPTED BACKUP-----`) and tolerant decoder (strip BOM, normalize line endings, accept `\r\n`).
3. **Memory hygiene at the parse boundary.** Standard `Json<T>` extractor produces `String` first; the descriptor must move into `Zeroizing<String>` before any `?` operator and pass by `&mut Zeroizing<String>` through validate → encrypt to avoid stack-copy leaks (PITFALLS #4).

**Primary recommendation:** Build `crates/core` first as a self-contained library exercising the full crypto path with the `/tmp/bed-test/` fixtures (descriptor `wsh(sortedmulti(2,...))` 458 bytes, `wallet.bed` 614 bytes, three xpub keys), then add `crates/server` axum layer. This matches D-02 and isolates the trust boundary from HTTP concerns.

## Standard Stack

### Core (workspace dependencies — pinned in Cargo.toml at workspace root)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | `0.8` | HTTP router, JSON/multipart extractors, `IntoResponse` trait | Locked in CLAUDE.md + STACK.md; co-maintained with tokio |
| `tokio` | `1.51` (LTS) | Async runtime; features `rt-multi-thread`, `macros`, `io-util` only | Locked in STACK.md; supported until Mar 2027 |
| `tower` | `0.5` | `ServiceExt::oneshot` for in-process tests (D-23) | Already in dep graph via `tower-http`; only `util` feature needed for tests |
| `tower-http` | `0.6` | `TraceLayer` for non-sensitive routes (D-19) | Locked in STACK.md |
| `bitcoin-encrypted-backup` | `1.0.0` git, `rev = "17b69b71cd1e005f80f5e81147795df0d11db027"` | BIP-XXX cipher core | Verified HEAD via Atom feed + REST API; see CARGO.toml below |
| `serde` | `1` (latest patch, currently `1.0.228+`) | Derive on request/response structs | Standard |
| `serde_json` | `1` (latest patch) | JSON encode/decode | Standard |
| `thiserror` | `2` | `AppError` enum with `Display`/`Error` derive (D-16) | Locked |
| `tracing` | `0.1` | Structured logging | Standard |
| `tracing-subscriber` | `0.3` | `EnvFilter` + `fmt::TestWriter` for tests | features: `env-filter`, `fmt` |
| `qrcode` | `0.14` | QR PNG generation; `EcLevel::L`, `Version::Normal(40)` | Locked |
| `image` | `0.25` (`default-features = false`, `features = ["png"]`) | PNG encoding for QR; `Luma<u8>` rendering | Locked |
| `base64` | `0.22` | Encoding/decoding for armored format and `bed_b64` field | Already transitive dep of crate |
| `zeroize` | `1.8` | `Zeroizing<String>` wrapper + `Zeroize` trait | Pure Rust; no system deps |

### Crate dependency block (verified verbatim from `/tmp/bed-test/encrypted_backup/Cargo.toml` and HEAD SHA)

```toml
# In crates/core/Cargo.toml
[dependencies.bitcoin-encrypted-backup]
git = "https://github.com/pythcoiner/encrypted_backup"
# Pinned to HEAD on master as of 2026-05-05 (verified via GitHub Atom + REST):
#   commit "pin clap version" — 17b69b71cd1e005f80f5e81147795df0d11db027
rev = "17b69b71cd1e005f80f5e81147795df0d11db027"
default-features = false
features = ["miniscript_12_3_5", "rand", "base64"]
# DO NOT enable: "devices" (USB doesn't reach StartOS), "cli" (pulls clap), "tokio" (only for devices)

# Re-export miniscript types from the crate (the crate exposes `pub use mscript_12_3_5 as miniscript`):
# use bitcoin_encrypted_backup::miniscript::{Descriptor, DescriptorPublicKey};
# This avoids any independent `miniscript` dep in our Cargo.toml that would risk version unification errors.
```

### Workspace skeleton (Cargo.toml at repo root)

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/server"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["semillabitcoin"]

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "warn"
# Apply per-crate via [lints] in each Cargo.toml — see below.

[workspace.dependencies]
# Pin minor versions here, override in member crates if needed.
axum = { version = "0.8", default-features = false, features = ["json", "multipart", "tokio", "http1"] }
tokio = { version = "1.51", default-features = false, features = ["rt-multi-thread", "macros", "io-util", "net", "signal"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", default-features = false, features = ["trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", default-features = false, features = ["env-filter", "fmt"] }
qrcode = { version = "0.14", default-features = false, features = ["image"] }
image = { version = "0.25", default-features = false, features = ["png"] }
base64 = "0.22"
zeroize = { version = "1.8", features = ["derive"] }
```

### Per-crate `Cargo.toml`

```toml
# crates/core/Cargo.toml
[package]
name = "bed-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
zeroize.workspace = true
thiserror.workspace = true
base64.workspace = true
qrcode.workspace = true
image.workspace = true

[dependencies.bitcoin-encrypted-backup]
git = "https://github.com/pythcoiner/encrypted_backup"
rev = "17b69b71cd1e005f80f5e81147795df0d11db027"
default-features = false
features = ["miniscript_12_3_5", "rand", "base64"]

[lints]
workspace = true
```

```toml
# crates/server/Cargo.toml
[package]
name = "bed-server"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
bed-core = { path = "../core" }
axum.workspace = true
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "io-util", "net", "signal"] }
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
zeroize.workspace = true

[dev-dependencies]
tower.workspace = true
tracing-subscriber = { workspace = true, features = ["env-filter", "fmt"] }

[lints]
workspace = true
```

### Toolchain pin

```toml
# rust-toolchain.toml (workspace root)
[toolchain]
channel = "stable"  # current stable as of 2026-05; CI uses dtolnay/rust-toolchain action with this file
components = ["rustfmt", "clippy"]
profile = "minimal"
```

## Crate API Surface (verified at `/tmp/bed-test/encrypted_backup/src/lib.rs`)

The crate exposes a builder-pattern struct `EncryptedBackup`. All methods take `self` by value and return `Self` or `Result<Self, Error>` — chain them.

**Public types** (re-exported via `pub use`):
- `EncryptedBackup` — main builder
- `Decrypted` enum: `Descriptor(Descriptor<DescriptorPublicKey>)`, `Policy`, `Labels`, `WalletBackup(Vec<u8>)`, `Raw(Vec<u8>)`
- `Content` enum (in `ll`): `None`, `Bip380`, `Bip388`, `Bip329`, `BIP(u16)`, `Proprietary(Vec<u8>)`, `Unknown` — `is_known()` returns true for the four named BIPs
- `Encryption` enum: `Undefined`, `ChaCha20Poly1305`, `Unknown`
- `Version` enum: `V0`, `V1`, `Unknown`
- `Error` enum: `Ll(ll::Error)`, `Utf8`, `Descriptor`, `NotImplemented`, `UnknownContent`, `EncryptionUndefined`, `InvalidVersion`, `WrongPayload`, `UnknownVersion`, `NoKey`, `WrongKey`, `DescriptorHasNoKeys`, `Base64`, `String(Box<String>)`
- `ToPayload` trait — implemented for `Vec<u8>` (Content::Unknown) and `Descriptor<DescriptorPublicKey>` (Content::Bip380)
- Re-export of `miniscript`: `pub use mscript_12_3_5 as miniscript;` — use as `bitcoin_encrypted_backup::miniscript::{Descriptor, DescriptorPublicKey, ...}`

**Encrypt path (verified, with `rand` feature enabled the nonce arg is omitted):**

```rust
use bitcoin_encrypted_backup::{
    miniscript::{Descriptor, DescriptorPublicKey},
    EncryptedBackup,
};
use std::str::FromStr;

let desc: Descriptor<DescriptorPublicKey> = Descriptor::from_str(descr_str)?;
let bed_bytes: Vec<u8> = EncryptedBackup::new()
    .set_payload(&desc)?       // sets payload + content=Bip380 + derivation_paths + keys (all from descriptor)
    .encrypt()?;               // ChaCha20-Poly1305, random nonce via OsRng (`rand` feature)
```

`set_payload` populates derivation paths and keys from the descriptor automatically (it calls `descr_to_dpks` internally and filters NUMS keys). No need to call `set_keys` for encryption.

**Decrypt path (verified, including base64 auto-detect):**

```rust
use bitcoin_encrypted_backup::{miniscript::bitcoin::secp256k1, EncryptedBackup, Decrypted};

// xpub_str is the user-supplied xpub from the multipart form.
// Parse to DescriptorPublicKey, then to secp256k1::PublicKey via dpk_to_pk:
let dpk = DescriptorPublicKey::from_str(xpub_str)?;
let pk: secp256k1::PublicKey = dpk_to_pk(&dpk);  // helper from crate's descriptor module

let restored: Decrypted = EncryptedBackup::new()
    .set_encrypted_payload(&bed_bytes)?  // auto-detects binary "BIPXXX" magic vs RFC4648 base64 string
    .set_keys(vec![pk])
    .decrypt()?;

let descriptor: Descriptor<DescriptorPublicKey> = match restored {
    Decrypted::Descriptor(d) => d,
    _ => return Err(AppError::WrongContent),
};
let cleartext: String = descriptor.to_string();  // ToString impl produces canonical descriptor with checksum
```

Note: `dpk_to_pk` is `pub` in `descriptor.rs` and accessible as `bitcoin_encrypted_backup::descriptor::dpk_to_pk`.

**Auto-detect at `set_encrypted_payload`** (from source line 219-239 of `lib.rs`):
```rust
if bytes.starts_with(ll::MAGIC.as_bytes()) {  // "BIPXXX" — 6 bytes
    return self.set_encrypted_payload_binary(bytes);
}
// else: try base64 decode (UTF-8 first) — handles armored payload AND raw base64
```

This means the multipart `bed` field can be passed as raw bytes regardless of whether the user uploaded a binary `.bed` (starts with `BIPXXX`) or pasted/uploaded armored text. **However**, it does NOT strip PEM-style headers — Phase 1's armored decoder must strip `-----BEGIN ... -----` / `-----END ... -----` and any whitespace **before** passing bytes to the crate.

## Wildcard Validation Pattern

The crate does NOT enforce `<0;1>/*`. Source verification: `set_payload` calls `descr_to_dpks(descriptor)` which only filters NUMS keys (BIP341 unspendable internal). Any descriptor that parses and has ≥1 non-NUMS key proceeds to encrypt.

The validation lives in `crates/core/src/validate.rs`. Use `for_each_key` (the `ForEachKey` trait is imported in `descriptor.rs` line 12) to walk every `DescriptorPublicKey`. The required shape:

| Key variant | Multipath `<0;1>/*`? | Action |
|-------------|---------------------|--------|
| `DescriptorPublicKey::Single(_)` | No (no derivation) | REJECT |
| `DescriptorPublicKey::XPub(k)` with `wildcard: Wildcard::None` | No (bare xpub) | REJECT |
| `DescriptorPublicKey::XPub(k)` with `wildcard: Wildcard::Unhardened`, single path | Single `/*`, not multipath | REJECT |
| `DescriptorPublicKey::MultiXPub(k)` where `k.derivation_paths.paths()` ≠ `[0/*, 1/*]` (e.g. `<2;3>`) | Wrong indices | REJECT |
| `DescriptorPublicKey::MultiXPub(k)` where `k.derivation_paths.paths() == [0/*, 1/*]` AND `wildcard == Wildcard::Unhardened` | YES | ACCEPT |

```rust
// crates/core/src/validate.rs
use bitcoin_encrypted_backup::miniscript::{
    descriptor::Wildcard, Descriptor, DescriptorPublicKey, ForEachKey,
};
use bitcoin_encrypted_backup::miniscript::bitcoin::bip32::{ChildNumber, DerivationPath};

pub fn require_multipath_0_1(desc: &Descriptor<DescriptorPublicKey>) -> Result<(), CoreError> {
    let mut all_ok = true;
    desc.for_each_key(|k| {
        let ok = match k {
            DescriptorPublicKey::MultiXPub(mx) => {
                if mx.wildcard != Wildcard::Unhardened {
                    false
                } else {
                    let paths: Vec<&DerivationPath> = mx.derivation_paths.paths().iter().collect();
                    paths.len() == 2
                        && paths[0].to_string() == "0"
                        && paths[1].to_string() == "1"
                    // The internal derivation paths in DescriptorMultiXKey are the
                    // *trailing* paths after the xkey; the wildcard is encoded
                    // separately. So `<0;1>/*` parses as
                    // derivation_paths = [DerivationPath("0"), DerivationPath("1")]
                    // wildcard = Wildcard::Unhardened.
                }
            }
            _ => false,
        };
        if !ok { all_ok = false; }
        true  // continue iteration regardless
    });
    if all_ok { Ok(()) } else { Err(CoreError::MissingMultipathWildcard) }
}
```

**Verification approach for the planner:** the `descriptor.rs` reference impl reads `dpk_to_deriv_path` against `DescriptorMultiXKey { derivation_paths, wildcard, ... }` — the field names are stable. Use `mx.derivation_paths.paths()` (returns `&[DerivationPath]`) per the miniscript `DerivPaths::new(...)` constructor visible at `descriptor.rs` line 178, 240, 250. If the field accessor differs in 12.3.5, fall back to iterating via `Display`: `mx.to_string()` contains the literal `<0;1>/*` substring after the xpub, which is robust as a final check.

**Property test inputs** (D-25):
- `wsh(pk(xpub.../0/*))` — Single wildcard, not multipath → reject
- `wsh(pk(xpub...))` — bare xpub no wildcard → reject
- `wsh(pk(xpub.../<2;3>/*))` — multipath but wrong indices → reject
- `wsh(sortedmulti(2, xpub1.../<0;1>/*, xpub2.../<2;3>/*))` — one good one bad → reject
- `wsh(sortedmulti(2, xpub1.../<0;1>/*, xpub2.../<0;1>/*, xpub3.../<0;1>/*))` — fixture from `/tmp/bed-test/desc.txt` → accept

## Architecture Patterns

### Recommended Crate Layout

```
crates/
├── core/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs            # pub use of: encrypt, decrypt, validate, armored, error
│   │   ├── encrypt.rs        # encrypt_descriptor(&mut Zeroizing<String>) -> Result<EncryptOutput, CoreError>
│   │   ├── decrypt.rs        # decrypt_payload(&[u8], &str xpub) -> Result<Zeroizing<String>, CoreError>
│   │   ├── validate.rs       # require_multipath_0_1(&Descriptor<DescriptorPublicKey>)
│   │   ├── armored.rs        # encode/decode PEM-style block
│   │   ├── qr.rs             # render_qr_png(&str armored) -> Result<Vec<u8>, CoreError>
│   │   ├── zeroize.rs        # ZeroizingDescriptor newtype (no Clone/Display/Debug)
│   │   └── error.rs          # CoreError (thiserror) — internal to core; mapped to AppError in server
│   └── tests/
│       ├── round_trip.rs     # encrypt+decrypt fixture round-trip
│       ├── validate.rs       # property tests (D-25)
│       ├── armored.rs        # armored encode/decode + tolerance
│       └── fixtures/
│           ├── desc.txt      # copy of /tmp/bed-test/desc.txt (multipath 2-of-3)
│           ├── wallet.bed    # copy of /tmp/bed-test/wallet.bed
│           └── xpub.txt      # copy of /tmp/bed-test/xpub.txt
└── server/
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs           # tokio::main, panic hook, EnvFilter, Router::new(), bind 127.0.0.1:8080
    │   ├── lib.rs            # pub fn router() -> Router  (for tests via oneshot)
    │   ├── routes/
    │   │   ├── mod.rs
    │   │   ├── encrypt.rs    # POST /api/encrypt
    │   │   └── decrypt.rs    # POST /api/decrypt
    │   ├── error.rs          # AppError (thiserror) + impl IntoResponse
    │   └── state.rs          # AppState (empty for Phase 1; placeholder for Phase 2)
    └── tests/
        ├── round_trip.rs     # POST /api/encrypt → POST /api/decrypt
        ├── no_leak.rs        # TestWriter capture asserts descriptor not in logs
        └── validation.rs     # 422 for bare xpub, wrong xpub, etc.
```

The `crates/server/src/lib.rs` exposing `pub fn router() -> Router` is critical for D-23: tests construct the router via this function and use `tower::ServiceExt::oneshot` against it without binding a socket.

### Pattern 1: Zeroizing at the parse boundary (PITFALLS #4)

**What:** Wrap the descriptor in `Zeroizing<String>` immediately at the JSON deserialization point, before any `?` early-return. Pass through the validate→encrypt pipeline by `&mut`, never by value.

**When to use:** Every code path that touches cleartext descriptor bytes.

**Example:**
```rust
// crates/server/src/routes/encrypt.rs
use axum::{extract::Json, response::IntoResponse};
use serde::Deserialize;
use zeroize::Zeroizing;

#[derive(Deserialize)]
pub struct EncryptRequest {
    pub descriptor: String,
}

#[tracing::instrument(skip_all)]
pub async fn post_encrypt(
    Json(req): Json<EncryptRequest>,
) -> Result<axum::Json<EncryptResponse>, AppError> {
    // STEP 1: wrap immediately — `req.descriptor` is moved INTO Zeroizing on the next line.
    // Any subsequent move of `cleartext` is &mut, never by value.
    let mut cleartext: Zeroizing<String> = Zeroizing::new(req.descriptor);
    // (req.descriptor's original String backing buffer is moved; the EncryptRequest
    // struct drops with the String now empty. No leftover stack copy.)

    let output = bed_core::encrypt::encrypt_descriptor(&mut cleartext)?;
    // After this line, encrypt_descriptor has parsed & built the .bed; cleartext
    // is still valid but the planner can choose to call .zeroize() explicitly here
    // and drop, OR rely on Drop at scope end (both safe per zeroize docs).

    cleartext.zeroize();  // explicit, defense-in-depth
    drop(cleartext);

    Ok(axum::Json(output))
}
```

The `crates/core` API:
```rust
pub fn encrypt_descriptor(cleartext: &mut Zeroizing<String>) -> Result<EncryptOutput, CoreError>;
pub fn decrypt_payload(bed_bytes: &[u8], xpub: &str) -> Result<Zeroizing<String>, CoreError>;
```

### Pattern 2: Armored encoder/decoder

**Encoder (verified format from BIP draft + OpenPGP convention):**

```rust
// crates/core/src/armored.rs
use base64::{engine::general_purpose::STANDARD, Engine as _};

pub const ARMOR_BEGIN: &str = "-----BEGIN BITCOIN ENCRYPTED BACKUP-----";
pub const ARMOR_END: &str = "-----END BITCOIN ENCRYPTED BACKUP-----";
const LINE_WIDTH: usize = 64;

pub fn encode_armored(bed_bytes: &[u8]) -> String {
    let b64 = STANDARD.encode(bed_bytes);
    let mut out = String::with_capacity(b64.len() + 128);
    out.push_str(ARMOR_BEGIN);
    out.push('\n');
    for chunk in b64.as_bytes().chunks(LINE_WIDTH) {
        out.push_str(std::str::from_utf8(chunk).expect("base64 is ASCII"));
        out.push('\n');
    }
    out.push_str(ARMOR_END);
    out.push('\n');
    out
}

pub fn decode_armored(input: &str) -> Result<Vec<u8>, ArmoredError> {
    // Strip BOM if present
    let s = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let mut payload_lines = Vec::new();
    let mut in_block = false;
    for raw_line in s.lines() {
        let line = raw_line.trim();  // tolerates leading indent + trailing spaces; \r already stripped by .lines()
        if line.is_empty() { continue; }
        if line.starts_with("-----BEGIN") {
            // Accept the canonical header verbatim, but be permissive on whitespace
            if line == ARMOR_BEGIN { in_block = true; continue; }
            return Err(ArmoredError::WrongHeader);
        }
        if line.starts_with("-----END") {
            if line == ARMOR_END { break; }
            return Err(ArmoredError::WrongFooter);
        }
        if in_block { payload_lines.push(line); }
    }
    if payload_lines.is_empty() { return Err(ArmoredError::EmptyPayload); }
    let joined: String = payload_lines.concat();
    let bytes = STANDARD.decode(joined.as_bytes()).map_err(|_| ArmoredError::Base64)?;
    Ok(bytes)
}

#[derive(thiserror::Error, Debug)]
pub enum ArmoredError {
    #[error("missing or wrong BEGIN header")]
    WrongHeader,
    #[error("missing or wrong END footer")]
    WrongFooter,
    #[error("empty payload between headers")]
    EmptyPayload,
    #[error("invalid base64 payload")]
    Base64,
}
```

`std::str::lines()` handles both `\n` and `\r\n` per std docs (verified). The `.trim()` per line absorbs any indentation and trailing whitespace, satisfying DEC-05.

### Pattern 3: QR PNG generation

```rust
// crates/core/src/qr.rs
use image::Luma;
use qrcode::{EcLevel, QrCode, Version};

pub const MAX_QR_BYTES: usize = 2900;  // safe ceiling under 2,953 ECC-L V40 byte mode

pub fn render_qr_png(armored: &str) -> Result<Vec<u8>, CoreError> {
    if armored.len() > MAX_QR_BYTES {
        return Err(CoreError::QrTooLarge { size: armored.len(), max: MAX_QR_BYTES });
    }
    // Auto-pick smallest version that fits; ECC-L for max capacity:
    let code = QrCode::with_error_correction_level(armored.as_bytes(), EcLevel::L)
        .map_err(|_| CoreError::QrEncode)?;
    let img = code.render::<Luma<u8>>().min_dimensions(256, 256).build();
    let mut buf = std::io::Cursor::new(Vec::<u8>::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|_| CoreError::QrEncode)?;
    Ok(buf.into_inner())
}
```

**QR sizing math** (verified from qrcode docs and QR spec):
- Version 40, ECC-L, **byte mode** capacity: 2,953 bytes (raw)
- Version 40, ECC-L, **alphanumeric mode** capacity: 4,296 chars
- Armored payload is base64 + headers — base64 chars `[A-Za-z0-9+/=]` are NOT alphanumeric per QR spec (alphanumeric = `0-9 A-Z $%*+-./: `, no lowercase, no `+/=`), so byte mode applies.
- Typical 2-of-3 multisig descriptor cleartext: ~458 bytes (verified from `/tmp/bed-test/desc.txt`).
- Encrypted `.bed` binary for that: 614 bytes (verified from `/tmp/bed-test/wallet.bed`).
- Base64 of 614 bytes: ⌈614/3⌉×4 = 820 chars.
- Armored adds: `BEGIN` line (40+1) + `END` line (38+1) + line-wraps (820/64 ≈ 13 newlines) + 2 outer newlines ≈ **~895 bytes total**.
- Safely under 2,900 cap. The `QrTooLarge` branch only triggers for unusually large multisig (e.g., 7-of-15 with long miniscript expressions).

### Pattern 4: axum Router with `lib.rs` for testability

```rust
// crates/server/src/lib.rs
use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::new()
        .route("/api/encrypt", post(routes::encrypt::post_encrypt))
        .route("/api/decrypt", post(routes::decrypt::post_decrypt))
        // Apply default body limit at the Router level; multipart fields can override:
        .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

mod routes;
mod error;
mod state;
pub use error::AppError;
```

```rust
// crates/server/src/main.rs
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

const BIND_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .with_level(true)
        .init();

    std::panic::set_hook(Box::new(|_info| {
        tracing::error!("internal panic");
    }));

    let listener = TcpListener::bind(BIND_ADDR).await?;
    tracing::info!(addr = %BIND_ADDR, "bed-server listening");
    axum::serve(listener, bed_server::router()).await?;
    Ok(())
}
```

### Pattern 5: AppError with `IntoResponse` (D-16 + D-17)

```rust
// crates/server/src/error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain.")]
    MissingMultipathWildcard,
    #[error("No se pudo parsear el descriptor.")]
    DescriptorParse,
    #[error("La xpub proporcionada no descifra este .bed (no es un cosigner válido).")]
    XpubMismatch,
    #[error("El descriptor cifrado excede capacidad QR ({size} > {max} bytes). Usá el archivo .bed o el armored.")]
    QrTooLarge { size: usize, max: usize },
    #[error("internal error")]
    Internal,
    #[error("solicitud inválida: {0}")]
    BadRequest(String),
}

#[derive(Serialize)]
struct ErrorBody { code: &'static str, message: String }

#[derive(Serialize)]
struct ErrorEnvelope { error: ErrorBody }

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code): (StatusCode, &'static str) = match &self {
            AppError::MissingMultipathWildcard => (StatusCode::UNPROCESSABLE_ENTITY, "MISSING_MULTIPATH_WILDCARD"),
            AppError::DescriptorParse           => (StatusCode::UNPROCESSABLE_ENTITY, "DESCRIPTOR_PARSE"),
            AppError::XpubMismatch              => (StatusCode::UNPROCESSABLE_ENTITY, "XPUB_MISMATCH"),
            AppError::QrTooLarge { .. }         => (StatusCode::UNPROCESSABLE_ENTITY, "QR_TOO_LARGE"),
            AppError::BadRequest(_)             => (StatusCode::BAD_REQUEST,           "BAD_REQUEST"),
            AppError::Internal                  => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL"),
        };
        let body = ErrorEnvelope { error: ErrorBody { code, message: self.to_string() } };
        (status, Json(body)).into_response()
    }
}

// Map crate's Error → AppError. The crate's WrongKey is the xpub-mismatch case.
impl From<bitcoin_encrypted_backup::Error> for AppError {
    fn from(e: bitcoin_encrypted_backup::Error) -> Self {
        use bitcoin_encrypted_backup::Error as E;
        match e {
            E::WrongKey | E::NoKey | E::DescriptorHasNoKeys => AppError::XpubMismatch,
            E::Descriptor | E::Utf8 => AppError::DescriptorParse,
            _ => AppError::Internal,
        }
    }
}
```

For axum 0.8's built-in extractor errors (malformed JSON → `JsonRejection`, multipart errors → `MultipartRejection`), wire them via the typed extractor — both implement `IntoResponse` returning 400 by default, satisfying D-16's "JSON malformado / multipart inválido → 400" rule without extra code.

### Pattern 6: Multipart for decrypt (D-06, axum 0.8 verified)

```rust
// crates/server/src/routes/decrypt.rs
use axum::{extract::Multipart, Json};
use zeroize::Zeroizing;

#[tracing::instrument(skip_all)]
pub async fn post_decrypt(mut form: Multipart) -> Result<Json<DecryptResponse>, AppError> {
    let mut bed: Option<Vec<u8>> = None;
    let mut xpub: Option<String> = None;

    while let Some(field) = form.next_field().await
        .map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "bed"  => {
                let raw = field.bytes().await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                bed = Some(raw.to_vec());
            }
            "xpub" => {
                // text() consumes the field; for both inline-text and uploaded-file,
                // multipart treats them the same — bytes interpreted as UTF-8.
                let text = field.text().await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                xpub = Some(text.trim().to_string());
            }
            _ => { /* ignore unknown fields */ }
        }
    }
    let bed_bytes = bed.ok_or_else(|| AppError::BadRequest("missing 'bed' field".into()))?;
    let xpub_str  = xpub.ok_or_else(|| AppError::BadRequest("missing 'xpub' field".into()))?;

    // Detect armored vs binary BEFORE handing to crate's auto-detect:
    // (the crate handles raw base64 but NOT PEM headers — see D-12)
    let payload: Vec<u8> = if bed_bytes.starts_with(b"-----BEGIN") {
        let text = std::str::from_utf8(&bed_bytes).map_err(|_| AppError::BadRequest("invalid utf-8 in armored".into()))?;
        bed_core::armored::decode_armored(text)?
    } else {
        bed_bytes
    };

    let mut cleartext: Zeroizing<String> = bed_core::decrypt::decrypt_payload(&payload, &xpub_str)?;
    let response = DecryptResponse { descriptor: cleartext.clone() };
    cleartext.zeroize();
    drop(cleartext);
    Ok(Json(response))
    // Note: response.descriptor is a String containing cleartext; it exits via JSON serialization.
    // This is the documented residual risk in the threat model — there is no way to keep it
    // zeroized through serde_json::to_writer + body framing.
}
```

### Anti-Patterns to Avoid (CONTEXT-locked + research-confirmed)

- **`unwrap()` / `expect()` in handler path** — enforced by `clippy::unwrap_used = "deny"` workspace lint (D-22).
- **`#[tracing::instrument]` without `skip_all`** — leaks descriptor argument by default (PITFALLS #2).
- **Move descriptor by value across function boundaries** — leaves stack copies (PITFALLS #4); pass by `&mut Zeroizing<String>`.
- **Bind to `0.0.0.0`** — bypasses StartOS routing (PITFALLS #12); use `127.0.0.1:8080` constant.
- **Deserialize `bed` field into `String`** — corrupts binary payloads with non-UTF-8 bytes; use `Vec<u8>` via `.bytes()`.
- **Build a `Multipart` extractor + a JSON body extractor on the same handler** — incompatible (axum reads body once); use one or the other.
- **Custom `axum-test` or `reqwest` for tests** — D-23 forbids; `tower::ServiceExt::oneshot` + `axum::body::to_bytes` is in-process.
- **`miniscript` as a separate Cargo dep** — version unification break; use `bitcoin_encrypted_backup::miniscript::*` re-export.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ChaCha20-Poly1305 + key derivation | Custom AEAD | `bitcoin-encrypted-backup` 1.0.0 (pinned rev) | BIP-XXX semantics, NUMS handling, individual_secrets — gets test vectors and audit by upstream |
| Descriptor parsing | Hand-roll regex | `Descriptor::<DescriptorPublicKey>::from_str` (re-exported by crate) | Miniscript 12.3.5 grammar, checksum verification, multipath syntax |
| QR PNG | DIY bit matrix | `qrcode 0.14` + `image 0.25` | ECC level handling, version selection, byte/alphanumeric mode auto-pick |
| Base64 line-wrapping | Hand-roll loop with comparison | `base64::engine::general_purpose::STANDARD` + `chunks(64)` | Standard alphabet, padding, validated by crate |
| Zeroize on String drop | Manual `.fill(0)` + `Drop` impl | `Zeroizing<String>` from `zeroize 1.8` | `ZeroizeOnDrop` marker, panic-safe, integration with `secrecy` if needed later |
| `IntoResponse` body framing | Custom `Response::builder` | `(StatusCode, Json<T>)` tuple impl | axum 0.8 stdlib pattern; handles `Content-Type` and content-length |
| Multipart parsing | Manual boundary regex | `axum::extract::Multipart` (multer 3 under the hood) | Streaming, RFC 7578-compliant, integrated with axum body limits |
| In-process integration tests | Bind socket on random port | `tower::ServiceExt::oneshot(router, request)` + `axum::body::to_bytes` | No port allocation, no race conditions, works in `cargo test --no-run` checks |
| License/dep audits | Custom CI script | `cargo deny check` + `cargo audit` | RustSec advisory DB, SPDX 3.25 license parsing |

**Key insight:** Phase 1's surface area is small enough that almost everything is "use the crate." The only hand-rolled pieces are: the armored encoder/decoder (D-12, ~50 LOC), the wildcard validator (~30 LOC), the `AppError` enum (~60 LOC), and the panic hook (3 LOC). Everything else is glue.

## Common Pitfalls (Phase 1 scope)

(Sourced from `.planning/research/PITFALLS.md` — Phase 1 must verify each.)

### Pitfall 1: Descriptor without `<0;1>/*` accepted (PITFALLS #1)
**What goes wrong:** Crate accepts any non-NUMS descriptor; a `wsh(pk(xpub.../0/*))` (single wildcard) silently encrypts.
**How to avoid:** `validate::require_multipath_0_1()` runs BEFORE `EncryptedBackup::new().set_payload(&desc)`.
**Verification:** Property test in `crates/core/tests/validate.rs` (D-25) feeding 5 invalid shapes + 1 valid fixture.

### Pitfall 2: Cleartext in tracing logs (PITFALLS #2 / SEC-01)
**What goes wrong:** A future contributor adds `tracing::info!(?req)` or omits `skip_all`.
**How to avoid:** `#[tracing::instrument(skip_all)]` on every handler; no body-logging middleware on `/api/*`. CI test (D-20) asserts descriptor string never appears in `TestWriter`-captured output across encrypt + decrypt.

### Pitfall 3: Panic backtrace leaking locals (PITFALLS #3 / CORE-05)
**What goes wrong:** `unwrap()` on malformed input panics; default Rust hook prints stack frames.
**How to avoid:** Workspace lint `clippy::unwrap_used = "deny"` + custom panic hook (D-21) discarding `PanicInfo`. CI runs `cargo clippy --all-targets -- -D warnings`.

### Pitfall 4: Stack copies of descriptor (PITFALLS #4 / CORE-04)
**What goes wrong:** `Zeroizing<String>` applied late or moved by value through helpers leaves bytes at earlier stack addresses.
**How to avoid:** Wrap on the very first line of the handler; pass `&mut Zeroizing<String>` through `validate::*` and `encrypt::*`. Code review checklist item.

### Pitfall 5: Armored format header mismatch (PITFALLS #10)
**What goes wrong:** Different case, BOM, or line endings reject cross-implementation `.bed`.
**How to avoid:** Single `const ARMOR_BEGIN` in `armored.rs`; decoder normalizes via `.lines()` + `.trim()`; cross-implementation round-trip test against `/tmp/bed-test/wallet.bed` and `pythcoiner/bed` output.

### Pitfall 6: Round-trip not tested (PITFALLS #11 / CI-02)
**What goes wrong:** Unit-tested encrypt + unit-tested decrypt don't combine end-to-end.
**How to avoid:** `crates/server/tests/round_trip.rs` runs `oneshot(POST /api/encrypt) → oneshot(POST /api/decrypt) → assert_eq!(input_descriptor, output_descriptor)`.

### Pitfall 7: Tor binding on `0.0.0.0` (PITFALLS #12 / SEC-02)
**How to avoid:** Hardcode `const BIND_ADDR: &str = "127.0.0.1:8080";` (D-26). No env var override in Phase 1.

### Pitfall 8: Crate format breakage (PITFALLS #14 / CORE-01)
**How to avoid:** `rev = "17b69b71cd1e005f80f5e81147795df0d11db027"` exact pin (D-04). `Cargo.lock` committed. CI uses `--locked`.

### Pitfall 9: `devices` feature pulled in transitively (PITFALLS #9)
**How to avoid:** `default-features = false` + only `["miniscript_12_3_5", "rand", "base64"]` (CORE-01). `cargo deny` declares `bans.deny = [{name="async-hwi"}]` as defense in depth.

## Code Examples

### Cargo.toml workspace lints (`clippy::unwrap_used` deny)

```toml
# Cargo.toml at workspace root
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "warn"
print_stdout = "warn"
print_stderr = "warn"

# Each crate references this:
# [lints]
# workspace = true
```

### `deny.toml` template (D-30)

```toml
# deny.toml at workspace root
[graph]
all-features = true

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
yanked = "deny"
ignore = []

# As of cargo-deny 0.16+: the schema uses a single setting; older docs reference "vulnerability"
# but the current key is unified — `cargo deny check advisories` deny by default with this setup.

[licenses]
confidence-threshold = 0.93
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "CC0-1.0",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "MPL-2.0",
    "Zlib",
]
exceptions = []

[bans]
multiple-versions = "warn"
wildcards = "deny"
deny = [
    { name = "openssl" },
    { name = "openssl-sys" },
    { name = "native-tls" },
    { name = "async-hwi" },     # defense-in-depth: ensures crate's `devices` feature stays disabled
]

[sources]
unknown-registry = "deny"
unknown-git = "allow"  # required for bitcoin-encrypted-backup git dep
allow-git = [
    "https://github.com/pythcoiner/encrypted_backup",
]
```

**Note:** `Unicode-3.0` added because newer crates (e.g., `unicode-ident`) migrated from `Unicode-DFS-2016` in 2024. Verify dep tree with `cargo deny list -l crate` after first build to catch missing licenses.

### GitHub Actions CI (D-28, D-29)

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  fmt:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features --workspace -- -D warnings

  test:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features --workspace --locked

  audit:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  deny:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with:
          command: check
          # checks: advisories bans licenses sources (default)
```

`Swatinem/rust-cache@v2` is the standard cargo cache action. `dtolnay/rust-toolchain@stable` is the standard toolchain installer (it reads `rust-toolchain.toml` automatically).

### Test infra: `tower::ServiceExt::oneshot` for round-trip (D-23, D-24)

```rust
// crates/server/tests/round_trip.rs
use axum::{body::{to_bytes, Body}, http::{Request, StatusCode}};
use serde_json::{json, Value};
use tower::ServiceExt;

const FIXTURE_DESC: &str = include_str!("../../core/tests/fixtures/desc.txt");
const FIXTURE_XPUB: &str = include_str!("../../core/tests/fixtures/xpub.txt");

#[tokio::test]
async fn encrypt_then_decrypt_roundtrip() {
    let app = bed_server::router();

    // POST /api/encrypt
    let body = json!({ "descriptor": FIXTURE_DESC.trim() }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/encrypt")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    let armored = parsed["armored"].as_str().unwrap().to_string();

    // POST /api/decrypt with armored + xpub via multipart
    let boundary = "----testboundary";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"bed\"\r\n\r\n{a}\r\n--{b}\r\nContent-Disposition: form-data; name=\"xpub\"\r\n\r\n{x}\r\n--{b}--\r\n",
        b = boundary, a = armored, x = FIXTURE_XPUB.trim()
    );
    let req = Request::builder()
        .method("POST")
        .uri("/api/decrypt")
        .header("content-type", format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["descriptor"].as_str().unwrap().trim(), FIXTURE_DESC.trim());
}

#[tokio::test]
async fn decrypt_with_wrong_xpub_returns_422() {
    // similar pattern, swap xpub for an unrelated xpub fixture; assert StatusCode::UNPROCESSABLE_ENTITY
}
```

### Test infra: no-leak via `TestWriter` (D-20 / SEC-01 / CI-02)

```rust
// crates/server/tests/no_leak.rs
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::{self, MakeWriter};

#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Vec<u8>>>);
impl<'a> MakeWriter<'a> for SharedBuf {
    type Writer = SharedWriter;
    fn make_writer(&'a self) -> Self::Writer { SharedWriter(self.0.clone()) }
}
struct SharedWriter(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[tokio::test]
async fn descriptor_never_appears_in_logs() {
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let sub = fmt::Subscriber::builder()
        .with_writer(SharedBuf(buf.clone()))
        .with_max_level(tracing::Level::TRACE)
        .finish();

    let descriptor = include_str!("../../core/tests/fixtures/desc.txt").trim().to_string();
    tracing::subscriber::with_default(sub, || {
        // Block-on inside subscriber default scope to capture all spans.
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let app = bed_server::router();
            // run the same encrypt+decrypt round-trip as in round_trip.rs
            // ...
        });
    });

    let captured = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let needle = "xpub6PLACEHOLDER2xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    assert!(!captured.contains(needle), "descriptor leaked into logs:\n{}", captured);
    // Also assert the full descriptor string never appears:
    assert!(!captured.contains(&descriptor), "full descriptor leaked into logs");
}
```

The pattern uses a `MakeWriter` impl over a shared buffer because `TestWriter` writes to stdout (verified at docs.rs/tracing-subscriber); `cargo test` capture works for stdout but doesn't expose captured bytes back to the test code. The shared buffer pattern is the canonical workaround documented in the tracing-subscriber issue tracker.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `axum 0.7` `axum::Server::bind()` | `axum 0.8` `axum::serve(TcpListener, Router)` | axum 0.7→0.8 (Nov 2024) | Use `tokio::net::TcpListener` + `axum::serve`; the `axum::Server` builder is gone |
| Sync `tracing::subscriber::set_global_default(...)` then go | `tracing_subscriber::fmt().with_env_filter(...).init()` | tracing-subscriber 0.3 stable | Stays current; combine with `EnvFilter::try_from_default_env()` for `RUST_LOG` support |
| `rusqlite` for any KV/metadata | Skip embedded DB in Phase 1 (history is Phase 2) | STACK.md decision | No DB code in Phase 1 |
| `actix-web` for HTTP | `axum 0.8` | Locked in CLAUDE.md | Don't reach for actix-web examples — they don't translate |
| `secrecy::SecretString` (preview suggestion) | `Zeroizing<String>` directly (`zeroize 1.8`) | CONTEXT D-10/D-11 | Wrap returns `&mut Zeroizing<String>`; `secrecy` is unnecessary indirection given the newtype `ZeroizingDescriptor` (no Clone/Display/Debug) |
| `tracing::instrument` default capture | Always `skip_all` on sensitive handlers | Project rule | Mandatory pattern in `crates/server/src/routes/*` |
| Body-logging middleware globally | TraceLayer only on non-sensitive routes; encrypt/decrypt opt-out | D-19 | Phase 1 has no non-sensitive routes — TraceLayer applies to none in Phase 1, becomes relevant when stub history endpoints appear in Phase 2 |

**Deprecated/outdated:**
- `axum::extract::ContentLengthLimit` — gone in 0.8; use `DefaultBodyLimit` layer.
- `tokio = { features = ["full"] }` in production — bloats binary; use `["rt-multi-thread", "macros", "io-util", "net", "signal"]`.
- `secrecy` 0.8 / SecretBox API — 0.10 is current; but Phase 1 doesn't need it (Zeroizing is sufficient per D-10/D-11).
- `image::ImageOutputFormat::Png` (deprecated in image 0.25) — use `image::ImageFormat::Png` with `write_to(&mut Cursor, ImageFormat::Png)`.
- `qrcodegen` crate — stalled; do not use.

## Open Questions

1. **Exact field accessor for `DescriptorMultiXKey::derivation_paths`**
   - What we know: The crate's reference impl reads `mx.derivation_paths` as a `DerivPaths` and the test fixtures (`descriptor.rs` line 178, 240, 250) construct via `DerivPaths::new(vec![DerivationPath::from_str("0").unwrap()])`. For multipath `<0;1>/*`, expect `vec![DerivationPath("0"), DerivationPath("1")]` with `wildcard: Wildcard::Unhardened`.
   - What's unclear: Whether `mx.derivation_paths.paths()` is the public accessor in miniscript 12.3.5 vs `mx.derivation_paths.iter()` vs deref. Direct field access on a `pub` struct member is also possible.
   - **Recommendation:** Planner adds a Wave 0 task: write a 5-line throwaway in `crates/core/src/validate.rs` with `mx.derivation_paths` and let the compiler error message reveal the accessor. If `paths()` doesn't exist, fall back to `Display`-based check: `mx.to_string().contains("<0;1>/*")` is robust because miniscript serializes multipath in canonical form.

2. **Whether `DefaultBodyLimit::max(512 * 1024)` is enough for armored decrypt**
   - What we know: Typical 2-of-3 multisig encrypted payload is ~600 bytes binary, ~900 bytes armored. 512 KB is a 500× safety margin.
   - What's unclear: Edge cases — large miniscript timelocks with 7+ keys could push armored to 4-5 KB but still nowhere near 512 KB.
   - **Recommendation:** Keep 512 KB; document in code comment as "well above worst-case armored multisig descriptor."

3. **`DerivPaths.paths()` returns `&[DerivationPath]` vs an iterator**
   - **Recommendation:** Same as (1) — let the compiler tell. Both APIs satisfy the validation logic.

4. **Whether the BIP draft requires `<0;1>` exactly or allows `<0;1;2;3>` for Liana taproot timelock branches**
   - What we know: D-08 explicitly rejects `<2;3>` as the primary descriptor for backup. The `descr_1()` test fixture in the crate uses `wsh(or_d(pk(.../<0;1>/*),and_v(v:pkh(.../<2;3>/*),older(...))))` — meaning a single descriptor can mix `<0;1>` and `<2;3>` keys for taproot branches.
   - What's unclear: Whether the validator should reject if ANY key uses non-`<0;1>` derivation, or only require that ALL keys use proper multipath of any 2-element form, or specifically that ALL primary keys use `<0;1>`.
   - **Recommendation:** Honor D-08 strictly — every key must have multipath `<0;1>/*`. This rejects `descr_1()` from the crate's tests. If users have a taproot timelock descriptor that mixes, they need to handle it as a future feature (not in Phase 1 scope per D-08).

## Sources

### Primary (HIGH confidence — verified locally or against official docs)

- **Crate source `/tmp/bed-test/encrypted_backup/src/lib.rs` and `descriptor.rs`** — full `EncryptedBackup` API, `set_payload`/`encrypt`/`set_encrypted_payload`/`set_keys`/`decrypt`, auto-detect at `set_encrypted_payload` (line 219-239), NUMS filter at `descr_to_dpks`, public `dpk_to_pk` helper.
- **Crate `/tmp/bed-test/encrypted_backup/Cargo.toml`** — feature flags: `default = ["miniscript_latest", "rand", "base64"]`; `miniscript_latest = ["miniscript_12_3_5"]`. Confirms `default-features = false, features = ["miniscript_12_3_5", "rand", "base64"]` is the safe pin.
- **GitHub Atom + REST `/repos/pythcoiner/encrypted_backup`** — HEAD on `master` = `17b69b71cd1e005f80f5e81147795df0d11db027`, message "pin clap version" — verified 2026-05-05.
- **`/tmp/bed-test/desc.txt` + `wallet.bed` + `xpub.txt`** — round-trip fixtures; multipath `<0;1>/*` 2-of-3 sortedmulti, 458/614/143 bytes. These ARE the test fixtures for `crates/core/tests/`.
- **CLAUDE.md** — Stack lock-in (axum 0.8, tokio 1.51 LTS, tower-http 0.6, no openssl/native-tls), "What NOT to Use," noreply email, castellano language.
- **`.planning/research/STACK.md`** — Cargo workspace deps with version pins, feature flags annotation.
- **`.planning/research/PITFALLS.md`** — 14 cataloged pitfalls; Phase 1 addresses #1-5, #9-11, #14 directly.
- **`.planning/research/ARCHITECTURE.md`** — System ASCII diagram, project structure, encrypt/decrypt data flow, anti-patterns (5 of them) verified against same crate source.
- **docs.rs/axum/0.8.8** — `IntoResponse` impl pattern, `Multipart::next_field()` API, `Json<T>` extractor (`Content-Type: application/json` automatic).
- **docs.rs/zeroize/1.8.1** — `Zeroizing<String>` semantics, `Drop`/`ZeroizeOnDrop` markers, `&mut` pass-through via `DerefMut`.
- **docs.rs/qrcode/0.14** — `EcLevel::L`, `Version::Normal(40)`, `2,953 bytes` byte mode capacity at V40 ECC-L; `render::<Luma<u8>>().build()` → `image::ImageBuffer`.
- **docs.rs/tracing-subscriber/0.3 `fmt::TestWriter`** — captures via `libtest`'s stdout interception; combined with `MakeWriter` impl for in-test buffer access.
- **cargo-deny official docs (`docs.rs/cargo-deny`)** — `[licenses]`/`[bans]`/`[advisories]`/`[sources]` schema.
- **GitHub Actions: `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `rustsec/audit-check@v2`, `EmbarkStudios/cargo-deny-action@v2`** — current canonical actions.

### Secondary (MEDIUM confidence — derived from official sources but unverified at exact version)

- **QR alphanumeric mode chars set** — `[0-9 A-Z $%*+\-./: ]` per QR spec ISO/IEC 18004 — confirms base64 is byte-mode (lowercase `a-z`, `+`, `/`, `=` not in alphanumeric set). Standard fact, source: QR spec.
- **`std::str::lines()` handles `\r\n`** — Rust std docs confirm trailing `\r\n` is stripped along with `\n`.
- **Crate `dpk_to_pk` accessibility** — `pub fn dpk_to_pk(...)` is at top of `descriptor.rs` (line 17); module-public via `pub mod descriptor` in `lib.rs`. Available as `bitcoin_encrypted_backup::descriptor::dpk_to_pk`.

### Tertiary (LOW confidence — needs verification at compile time)

- **`DescriptorMultiXKey::derivation_paths` accessor** — exact field name vs `paths()` method (see Open Question #1).
- **Exact `image 0.25` PNG write API** — `image::ImageFormat::Png` (current) replaced `ImageOutputFormat::Png` (0.24); confirm at compile time when implementing `qr.rs`.

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — versions locked in CLAUDE.md/STACK.md and crate features verified against `/tmp/bed-test/encrypted_backup/Cargo.toml`.
- Architecture (workspace + crate split): HIGH — D-02 explicit, ARCHITECTURE.md exhaustive.
- Crate API Surface: HIGH — read line-by-line from local source.
- Wildcard Validation: MEDIUM-HIGH — strategy is clear; exact accessor for `derivation_paths` needs compile-time confirmation (Open Q1).
- Armored Format: MEDIUM — header string is canonical, but cross-implementation round-trip vs `pythcoiner/bed` GUI not verified before fixing the const (D-13 explicit gate).
- Pitfalls: HIGH — sourced from PITFALLS.md, all preventions concrete.
- CI Workflow: HIGH — actions are canonical and stable.
- Test Infra: HIGH — `oneshot` + `to_bytes` + `TestWriter+MakeWriter` are documented patterns.

**Research date:** 2026-05-05
**Valid until:** 2026-06-05 (30 days for stable Rust ecosystem; HEAD SHA may advance — D-04 captures the snapshot)
