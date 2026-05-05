---
phase: 01-crypto-core-http-api
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - Cargo.lock
  - rust-toolchain.toml
  - .gitignore
  - deny.toml
  - crates/core/Cargo.toml
  - crates/core/src/lib.rs
  - crates/server/Cargo.toml
  - crates/server/src/lib.rs
  - crates/server/src/main.rs
autonomous: true
requirements: [CORE-01, CORE-05, SEC-03]
must_haves:
  truths:
    - "cargo build --workspace compiles cleanly with no warnings"
    - "cargo deny check passes (no openssl-sys, no native-tls, allowed licenses only)"
    - "Workspace lints deny unwrap_used and expect_used in crates/server"
    - "Crate bitcoin-encrypted-backup pinned to rev 17b69b71cd1e005f80f5e81147795df0d11db027"
  artifacts:
    - path: "Cargo.toml"
      provides: "Workspace manifest with members crates/core + crates/server, workspace.dependencies, workspace.lints"
      contains: "[workspace]"
    - path: "crates/core/Cargo.toml"
      provides: "Core crate manifest pinning bitcoin-encrypted-backup to exact rev"
      contains: 'rev = "17b69b71cd1e005f80f5e81147795df0d11db027"'
    - path: "crates/server/Cargo.toml"
      provides: "Server crate manifest with axum 0.8, tokio 1.51 LTS, tower-http 0.6"
      contains: "axum.workspace = true"
    - path: "rust-toolchain.toml"
      provides: "Stable toolchain with rustfmt + clippy components"
      contains: 'channel = "stable"'
    - path: "deny.toml"
      provides: "License allowlist + bans for openssl-sys/native-tls/async-hwi"
      contains: '{ name = "openssl-sys" }'
    - path: ".gitignore"
      provides: "Standard Rust gitignore (target/, .DS_Store)"
      contains: "target"
  key_links:
    - from: "crates/server/Cargo.toml"
      to: "crates/core"
      via: "path dependency bed-core"
      pattern: 'bed-core = \{ path = "../core" \}'
    - from: "Cargo.toml workspace.lints.clippy"
      to: "crates/server/Cargo.toml [lints]"
      via: "workspace = true reference"
      pattern: "workspace = true"
---

<objective>
Inicializa el Cargo workspace para `bed-app` con dos crates skeleton (`crates/core`, `crates/server`), pin exacto de la crate `bitcoin-encrypted-backup`, `deny.toml` con bans y license allowlist, `rust-toolchain.toml` en stable, lints `unwrap_used`/`expect_used = "deny"` en workspace y `.gitignore`.

Purpose: Sentar las bases de seguridad y reproducibilidad — pin exacto de la crate (D-04), prohibición de OpenSSL/native-tls (D-30, SEC-03), y lints que bloquean `unwrap()` (D-22, CORE-05) — antes de que cualquier otro plan toque código.
Output: Workspace compila vacío (`cargo build --workspace`) y `cargo deny check` pasa.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/01-crypto-core-http-api/01-CONTEXT.md
@.planning/phases/01-crypto-core-http-api/01-RESEARCH.md
@CLAUDE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Crear Cargo workspace + rust-toolchain.toml + .gitignore</name>
  <files>Cargo.toml, rust-toolchain.toml, .gitignore</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (sección "Workspace skeleton" — copia literal del workspace Cargo.toml)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-02, D-03, D-22, D-27)
    - CLAUDE.md (Version Compatibility table)
  </read_first>
  <action>
    Crear `Cargo.toml` en raíz (workspace root) con el contenido EXACTO del bloque "Workspace skeleton" de `01-RESEARCH.md`:
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
    print_stdout = "warn"
    print_stderr = "warn"

    [workspace.dependencies]
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

    Crear `rust-toolchain.toml`:
    ```toml
    [toolchain]
    channel = "stable"
    components = ["rustfmt", "clippy"]
    profile = "minimal"
    ```

    Crear `.gitignore`:
    ```
    /target
    **/*.rs.bk
    Cargo.lock.bak
    .DS_Store
    .idea/
    .vscode/
    ```
    NOTA: NO ignorar `Cargo.lock` (D-03 exige commitearlo).
  </action>
  <verify>
    <automated>test -f Cargo.toml && test -f rust-toolchain.toml && test -f .gitignore && grep -q 'unwrap_used = "deny"' Cargo.toml && grep -q 'channel = "stable"' rust-toolchain.toml</automated>
  </verify>
  <acceptance_criteria>
    - `Cargo.toml` existe y `grep -c '\[workspace\]' Cargo.toml` == 1
    - `Cargo.toml` contiene literal `unwrap_used = "deny"`
    - `Cargo.toml` contiene literal `expect_used = "deny"`
    - `Cargo.toml` contiene literal `members = ["crates/core", "crates/server"]`
    - `Cargo.toml` contiene literal `resolver = "2"`
    - `rust-toolchain.toml` contiene literal `channel = "stable"`
    - `.gitignore` contiene literal `/target`
    - `.gitignore` NO contiene la línea `Cargo.lock` (debe versionarse)
  </acceptance_criteria>
  <done>Workspace root files commiteables; ejecutar `cargo metadata --no-deps` no falla por sintaxis (aún sin miembros — eso lo añade Task 2).</done>
</task>

<task type="auto">
  <name>Task 2: Crear crates/core + crates/server skeleton con pin exacto de la crate</name>
  <files>crates/core/Cargo.toml, crates/core/src/lib.rs, crates/server/Cargo.toml, crates/server/src/lib.rs, crates/server/src/main.rs, Cargo.lock</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (secciones "Per-crate Cargo.toml" + "Pattern 4: axum Router with lib.rs for testability" — bloques verbatim)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-04, D-21, D-26)
    - /tmp/bed-test/encrypted_backup/src/lib.rs (verifica nombre del módulo `descriptor` y re-export `pub use mscript_12_3_5 as miniscript;`)
  </read_first>
  <action>
    Crear `crates/core/Cargo.toml` (D-04 — pin EXACTO):
    ```toml
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
    # Pinned to HEAD on master as of 2026-05-05 (verified via GitHub Atom + REST):
    #   commit "pin clap version" — 17b69b71cd1e005f80f5e81147795df0d11db027
    rev = "17b69b71cd1e005f80f5e81147795df0d11db027"
    default-features = false
    features = ["miniscript_12_3_5", "rand", "base64"]
    # DO NOT enable: "devices" (USB doesn't reach StartOS), "cli" (pulls clap), "tokio"

    [lints]
    workspace = true
    ```

    Crear `crates/core/src/lib.rs` (skeleton, módulos serán llenados en planes 03/04):
    ```rust
    //! bed-core — pure Bitcoin Encrypted Backup logic (validation, encrypt/decrypt wrapper,
    //! armored encoder/decoder, QR generation). No HTTP layer.
    //!
    //! Re-exports the `bitcoin_encrypted_backup::miniscript` types so consumers don't add
    //! a separate `miniscript` dep (would risk version unification break).

    pub use bitcoin_encrypted_backup::miniscript;
    ```

    Crear `crates/server/Cargo.toml`:
    ```toml
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

    Crear `crates/server/src/lib.rs` (router skeleton — handlers stub):
    ```rust
    //! bed-server — axum HTTP layer for bed-core.
    //!
    //! Exposes `pub fn router() -> Router` so integration tests can use
    //! `tower::ServiceExt::oneshot` without binding a socket (D-23).

    use axum::{routing::post, Router};

    pub fn router() -> Router {
        Router::new()
            .route("/api/encrypt", post(encrypt_stub))
            .route("/api/decrypt", post(decrypt_stub))
            .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
    }

    async fn encrypt_stub() -> &'static str { "encrypt stub" }
    async fn decrypt_stub() -> &'static str { "decrypt stub" }
    ```

    Crear `crates/server/src/main.rs` (D-21, D-26 — bind hardcoded + panic hook):
    ```rust
    use std::error::Error;
    use tokio::net::TcpListener;
    use tracing_subscriber::EnvFilter;

    const BIND_ADDR: &str = "127.0.0.1:8080";

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn Error>> {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
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
    NOTA al executor: `unwrap_or_else` NO es `unwrap()` — está permitido bajo el lint `unwrap_used`. Verificar que clippy pase.

    Ejecutar `cargo build --workspace --locked` (genera `Cargo.lock`). Si la primera resolución falla (no hay `--locked` aún), correr `cargo build --workspace`, después confirmar que pasa con `--locked` en runs subsiguientes.
  </action>
  <verify>
    <automated>cargo build --workspace 2>&1 | tail -5 && test -f Cargo.lock && grep -q 'rev = "17b69b71cd1e005f80f5e81147795df0d11db027"' crates/core/Cargo.toml && grep -q 'const BIND_ADDR: &str = "127.0.0.1:8080";' crates/server/src/main.rs</automated>
  </verify>
  <acceptance_criteria>
    - `cargo build --workspace` exits 0
    - `Cargo.lock` existe y contiene la entry `bitcoin-encrypted-backup` con `source = "git+https://github.com/pythcoiner/encrypted_backup?rev=17b69b71cd1e005f80f5e81147795df0d11db027#17b69b71..."`
    - `grep -c 'rev = "17b69b71cd1e005f80f5e81147795df0d11db027"' crates/core/Cargo.toml` == 1
    - `grep -c 'features = \["miniscript_12_3_5", "rand", "base64"\]' crates/core/Cargo.toml` == 1
    - `grep -c '"devices"\|"cli"' crates/core/Cargo.toml` == 0 (las features prohibidas NO aparecen)
    - `grep -c 'const BIND_ADDR: &str = "127.0.0.1:8080";' crates/server/src/main.rs` == 1
    - `grep -c 'std::panic::set_hook' crates/server/src/main.rs` == 1
    - `grep -E '\bunwrap\(\)|\.expect\(' crates/server/src/main.rs crates/server/src/lib.rs | grep -v 'unwrap_or_else' | wc -l` == 0
  </acceptance_criteria>
  <done>Workspace compila; pin exacto de la crate; bind y panic hook ya en `main.rs`; router skeleton listo para handlers reales en planes posteriores.</done>
</task>

<task type="auto">
  <name>Task 3: Crear deny.toml + verificar cargo deny check</name>
  <files>deny.toml</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (sección "deny.toml template" — copia literal)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-30)
    - .planning/research/PITFALLS.md (Pitfall 8 + 9 — distroless TLS, devices feature)
  </read_first>
  <action>
    Crear `deny.toml` en raíz con el contenido EXACTO del bloque "deny.toml template (D-30)" de `01-RESEARCH.md`:
    ```toml
    [graph]
    all-features = true

    [advisories]
    db-path = "~/.cargo/advisory-db"
    db-urls = ["https://github.com/rustsec/advisory-db"]
    yanked = "deny"
    ignore = []

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
        { name = "async-hwi" },
    ]

    [sources]
    unknown-registry = "deny"
    unknown-git = "allow"
    allow-git = [
        "https://github.com/pythcoiner/encrypted_backup",
    ]
    ```

    Instalar cargo-deny si no está disponible: `cargo install --locked cargo-deny` (skip si ya está). Ejecutar `cargo deny check` y verificar que pasa. Si falla por una licencia faltante (e.g., una transitive dep usa `OpenSSL` SPDX o similar), añadir SOLAMENTE la licencia específica al `allow` con un comentario `# added because crate X requires it`. NO añadir crates al deny-list ni quitar bans existentes.
  </action>
  <verify>
    <automated>cargo deny check 2>&1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `deny.toml` contiene literal `{ name = "openssl-sys" }`
    - `deny.toml` contiene literal `{ name = "native-tls" }`
    - `deny.toml` contiene literal `{ name = "async-hwi" }`
    - `deny.toml` contiene literal `"https://github.com/pythcoiner/encrypted_backup"`
    - `cargo deny check` exits 0
    - `cargo deny check bans 2>&1 | grep -E "openssl-sys|native-tls"` retorna 0 matches (esos crates NO están en el dep tree — el ban no se gatilla, solo defiende contra futuras adiciones)
  </acceptance_criteria>
  <done>`cargo deny check` pasa; bans en su sitio; license allowlist cubre todas las deps actuales.</done>
</task>

</tasks>

<verification>
- `cargo build --workspace` exits 0
- `cargo deny check` exits 0
- `Cargo.lock` commiteado, contiene rev pin exacto
- `grep -E '\.unwrap\(\)|\.expect\(' crates/server/src/main.rs` retorna 0 matches (excluyendo `unwrap_or_else`)
- Bind hardcoded a `127.0.0.1:8080`
- Panic hook descartando `PanicInfo`
</verification>

<success_criteria>
- Workspace compila vacío (handlers stub) sin warnings
- `cargo deny check` pasa
- Pin exacto de `bitcoin-encrypted-backup` por rev
- Lints `unwrap_used`/`expect_used = "deny"` aplicados
- `deny.toml` rechaza `openssl-sys`, `native-tls`, `async-hwi`
- `BIND_ADDR = "127.0.0.1:8080"` y panic hook ya en `main.rs`
</success_criteria>

<output>
Tras completar, crear `.planning/phases/01-crypto-core-http-api/01-01-workspace-skeleton-SUMMARY.md` documentando:
- Archivos creados (paths exactos)
- Versión de cargo-deny usada
- Output de `cargo build --workspace` (líneas de fin)
- Output de `cargo deny check` (resumen)
- Sha del rev pinneado: `17b69b71cd1e005f80f5e81147795df0d11db027`
</output>
