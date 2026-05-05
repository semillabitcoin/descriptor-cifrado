# Phase 1: Crypto Core + HTTP API - Context

**Gathered:** 2026-05-05
**Status:** Ready for planning
**Mode:** `--auto` (Claude auto-selected recommended options; review and revisit before plan if needed)

<domain>
## Phase Boundary

Phase 1 entrega un servidor `axum` local en `127.0.0.1:8080` con dos endpoints estables — `POST /api/encrypt` y `POST /api/decrypt` — que importan la crate `bitcoin-encrypted-backup` directamente y producen / consumen los tres formatos (`.bed` binario base64, armored estilo PGP, QR PNG base64). Todas las invariantes de seguridad (zeroize, no-`unwrap`, no-log de descriptor, validación `<0;1>/*` multipath, panic hook genérico, pin exacto de la crate, sin OpenSSL/native-tls) quedan cerradas en esta fase. CI corre `cargo audit` + `cargo deny` + round-trip + no-leak en cada PR.

**Fuera de Phase 1:** SPA frontend (Phase 2), endpoints de history (`GET/DELETE /api/history` son Phase 2 con HIST-*), Dockerfile multi-stage (Phase 3), s9pk packaging (Phase 4).

</domain>

<decisions>
## Implementation Decisions

### Repo & Workspace
- **D-01:** Repo único `semillabitcoin/bed-app` (Docker en GHCR). El repo `semillabitcoin/bed-startos` (TypeScript wrapper) se inicializa en Phase 4.
- **D-02:** Cargo workspace con dos crates: `crates/core` (cripto puro, unit-testable sin HTTP) + `crates/server` (axum). Razón: separar trust boundary de HTTP layer; tests del core no requieren bindeo a socket.
- **D-03:** Phase 1 incluye repo init: `Cargo.toml` raíz workspace + dos crates skeleton + `Cargo.lock` commiteado + `deny.toml` + `.github/workflows/ci.yml` + `rust-toolchain.toml` (canal `stable` pinneado). **NO** Dockerfile (Phase 3).
- **D-04:** Pinning de la crate `bitcoin-encrypted-backup`: git dep con `rev = "<commit-SHA>"` exacto (no tag, no branch). El SHA exacto se resuelve en `plan-phase` consultando HEAD de `https://github.com/pythcoiner/encrypted_backup` y se documenta en comentario de `Cargo.toml`.

### API Surface
- **D-05:** `POST /api/encrypt` — request `Content-Type: application/json`, body `{"descriptor": "<string>"}`. Response 200 JSON único con los tres outputs: `{"bed_b64": "...", "armored": "-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n...\n-----END...", "qr_png_b64": "..."}`. Razón: matches success criterion #1 ("returns three outputs"), single round-trip, frontend simple.
- **D-06:** `POST /api/decrypt` — request `Content-Type: multipart/form-data`. Campos: `bed` (texto armored OR file binario `.bed`, el server detecta por bytes mágicos / presencia de header) + `xpub` (texto OR file con xpub). Response 200 JSON `{"descriptor": "<cleartext>"}`. Razón: DEC-01 exige multipart; aceptar ambos formatos sin duplicar endpoint.
- **D-07:** Endpoint contract es estable a partir de Phase 1 — Phase 2 solo wirea la SPA contra estas rutas. Cambios al contract requieren bump de version path (`/api/v2/...`).

### Validación BIP
- **D-08:** Validación `<0;1>/*` multipath: iterar `desc.iter_pk()` sobre el `Descriptor<DescriptorPublicKey>` parseado y exigir que cada key tenga derivación multipath con índices `0` y `1` específicamente. Rechazar:
  - `Wildcard::None` (xpub bare, sin `/*`)
  - Wildcard simple `/*` sin multipath
  - Multipath con índices distintos a `<0;1>` (ej. `<2;3>` solo es válido como segundo descriptor en taproot timelock branches; en single-descriptor backup el primario debe ser `<0;1>`)
- **D-09:** Error tipado `AppError::MissingMultipathWildcard` → HTTP 422 con mensaje en castellano: `"El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain."` (UI lo muestra inline — ENC-05).

### Memoria & Zeroize
- **D-10:** `Zeroizing<String>` se aplica en el handler **al deserializar el body**, antes de cualquier `?` early-return o llamada a helper. Pasar por `&mut` ref a través de `validate → encrypt`; nunca mover el `String` por valor entre funciones (PITFALLS #4 — moves dejan copias en stack). Tras `encrypt()`, `.zeroize()` explícito y `drop()` inmediato.
- **D-11:** Newtype wrapper `ZeroizingDescriptor` en `crates/core/src/zeroize.rs` que envuelve `Zeroizing<String>` y NO implementa `Clone` ni `Display` ni `Debug`. Razón: imposibilita logging accidental.

### Armored Format
- **D-12:** La crate `bitcoin-encrypted-backup` **NO provee armored** (verificado en `/tmp/bed-test/encrypted_backup/src/`). Implementar el wrapper armored en `crates/core/src/armored.rs`:
  - Encode: `Vec<u8>` binary → base64 con line-wrap cada 64 chars → wrap con `-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n` y `\n-----END BITCOIN ENCRYPTED BACKUP-----\n`
  - Decode: tolerante a whitespace, indentación arbitraria, `\r\n` vs `\n`. Strip headers, strip whitespace, base64 decode → `Vec<u8>`
- **D-13:** El header string exacto se valida con cross-implementation round-trip antes de cerrar la fase (idealmente contra el `.bed` que produce Liana v13 o `pythcoiner/bed`). Si el draft BIP cambia el header, se actualiza con bump menor de la crate `core`.

### QR
- **D-14:** Server-side QR con `qrcode 0.14` + `image 0.25`. ECC level **L** (máxima capacidad). Modo byte sobre el armored completo (incluye headers). PNG output via `ImageOutputFormat::Png`.
- **D-15:** Si `armored.len() > 2900` bytes (capacidad práctica QR ECC-L versión 40 modo byte) → HTTP 422 con `AppError::QrTooLarge` y mensaje: `"El descriptor cifrado excede capacidad QR (X > 2900 bytes). Usá el archivo .bed o el armored copy/paste."`. Phase 1 mide payload real para multisig 2-of-3 típico y ajusta si hace falta.

### Errores
- **D-16:** Single `AppError` enum (`thiserror`) en `crates/server/src/error.rs` con `impl IntoResponse for AppError`. Variantes: `MissingMultipathWildcard`, `DescriptorParse`, `XpubMismatch`, `QrTooLarge`, `Internal`. Mapeo de status:
  - `MissingMultipathWildcard | DescriptorParse | QrTooLarge | XpubMismatch` → 422
  - JSON malformado / multipart inválido → 400 (axum default + extractor errors mapeados)
  - Panic / internal → 500 con body genérico `{"error":{"code":"INTERNAL","message":"internal error"}}` (sin detalles)
- **D-17:** Response body uniforme: `{"error": {"code": "<UPPER_SNAKE>", "message": "<castellano>"}}`. Frontend muestra `message` en UI.

### Tracing & Logging
- **D-18:** `tracing-subscriber::fmt()` con `EnvFilter::from_default_env()` por defecto en `INFO`. Format: `with_target(false)`, `with_level(true)`, JSON solo si `BED_LOG_FORMAT=json` (defer a v2 si no se necesita).
- **D-19:** `TraceLayer::new_for_http()` aplicado SOLO a rutas no-sensibles (Phase 1: ninguna otra ruta). Encrypt/decrypt handlers `#[tracing::instrument(skip_all)]`. Span fields permitidos: `method`, `path`, `status`, `duration_ms`. NUNCA bodies, headers `Authorization`, ni params de query.
- **D-20:** Test no-leak (CI-02) en `crates/server/tests/no_leak.rs`: usa `tracing_subscriber::fmt::TestWriter`, ejecuta encrypt+decrypt con descriptor fixture, asserta que `descriptor_str` no aparece como substring en el buffer capturado.

### Panic Hook
- **D-21:** En `main.rs` startup: `std::panic::set_hook(Box::new(|_info| { tracing::error!("internal panic"); }))`. Descarta `PanicInfo` entero. `RUST_BACKTRACE` nunca se setea en Dockerfile (es problema de Phase 3, pero el hook ya protege).
- **D-22:** Cero `unwrap()` / `expect()` en el path de request. Test CI con `clippy::unwrap_used` + `clippy::expect_used` en lint level `deny` para `crates/server/src/`.

### Test Infra
- **D-23:** Tests integración con `tower::ServiceExt::oneshot` + `axum::body::to_bytes` (in-process, sin sockets, sin port allocation). `tower` ya está en el dep graph vía `tower-http`. NO usar `axum-test` ni `reqwest` (extra dep).
- **D-24:** Round-trip test (CI-02) en `crates/server/tests/round_trip.rs`: descriptor fixture → encrypt → decrypt con xpub correcto → assert equals original. Variante con xpub incorrecto → assert 422.
- **D-25:** Property-based test (validación) en `crates/core/tests/validate.rs`: feedea descriptors sin `<0;1>/*` (bare xpub, single wildcard `/*`, multipath `<2;3>`) → assert error.

### Bind & TLS
- **D-26:** Server bindeado a `127.0.0.1:8080` en `crates/server/src/main.rs`. Constante `const BIND_ADDR: &str = "127.0.0.1:8080";` — NO leer de env en Phase 1 (StartOS rutea externamente, no necesita override).
- **D-27:** `tokio` features mínimos: `rt-multi-thread`, `macros`, `io-util`. Evitar `full`. Evitar `tokio-util` salvo que un test lo requiera.

### CI Workflow
- **D-28:** `.github/workflows/ci.yml` con jobs:
  - `fmt`: `cargo fmt --all -- --check`
  - `clippy`: `cargo clippy --all-targets --all-features -- -D warnings`
  - `test`: `cargo test --all-features --workspace` (incluye round-trip + no-leak + property)
  - `audit`: `cargo audit` (instalado vía action)
  - `deny`: `cargo deny check` con `deny.toml` que rechaza `openssl-sys` y `native-tls`
- **D-29:** Trigger: `pull_request` + `push` a `main`. Runner: `ubuntu-latest`. Jobs en paralelo cuando sea posible. Timeout 15 min.
- **D-30:** `deny.toml` declara: `licenses.allow = [...]` (MIT, Apache-2.0, BSD-{2,3}-Clause, ISC, CC0-1.0, Unicode-DFS-2016, MPL-2.0); `bans.deny = [{name = "openssl-sys"}, {name = "native-tls"}]`; `advisories.vulnerability = "deny"`.

### Claude's Discretion
- Layout exacto de archivos dentro de cada crate (subdirs `routes/`, `handlers/`, etc.) — research sugiere estructura, planner ajusta.
- Names exactos de variantes en `AppError` enum (manteniendo el contract del campo `code`).
- Versions exactas de `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber` (latest patch del minor especificado en STACK.md).
- Si tests de propiedad usan `proptest` o `quickcheck` (planner decide; ambos válidos).
- Estructura interna del armored encoder (helper functions, line-wrap detail).

### Folded Todos
None — `gsd-tools todo match-phase 1` retornó 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning
- `.planning/PROJECT.md` — Vision, constraints, key decisions table
- `.planning/REQUIREMENTS.md` — Phase 1 requirements: CORE-01..05, ENC-01..05, DEC-01..05, SEC-01..03, CI-01, CI-02
- `.planning/ROADMAP.md` §"Phase 1: Crypto Core + HTTP API" — Goal + 5 success criteria
- `.planning/STATE.md` — Current position
- `IDEA.md` — Original brief (referencias a BIP draft, repos, threat model)
- `CLAUDE.md` — Stack lock-in (axum 0.8, tokio 1.51 LTS, tower-http 0.6, bitcoin-encrypted-backup 1.0.0 git dep, etc.) + "What NOT to Use" + version compatibility matrix

### Research artifacts (este proyecto)
- `.planning/research/SUMMARY.md` — Locked stack table, 5 critical pitfalls, build order
- `.planning/research/PITFALLS.md` — 5 pitfalls completos con código de ejemplo (validación wildcard, tracing skip_all, panic hook, zeroize stack moves, persist cleartext)
- `.planning/research/STACK.md` — Versiones exactas, alternatives considered, version compatibility
- `.planning/research/ARCHITECTURE.md` — System overview ASCII, project structure (Cargo workspace), crate internals confirmados desde `/tmp/bed-test/encrypted_backup/src/`
- `.planning/research/FEATURES.md` — Feature scope mapping

### Reference implementation (lectura local)
- `/tmp/bed-test/encrypted_backup/src/lib.rs` — API pública de la crate (`EncryptedBackup`, encrypt/decrypt entry points)
- `/tmp/bed-test/encrypted_backup/src/descriptor.rs` — Tipos de descriptor, validación interna, fixtures de wildcard. **Confirmado:** la crate NO provee armored format — la app lo implementa.
- `/tmp/bed-test/encrypted_backup/src/ll.rs` — Low-level encrypt/decrypt
- `/tmp/bed-test/desc.txt` + `/tmp/bed-test/wallet.bed` + `/tmp/bed-test/key{1,2,3}.txt` + `/tmp/bed-test/xpub.txt` — fixtures funcionales para round-trip cross-implementation

### External specs (web — usar `WebFetch` cuando se necesite)
- BIP draft PR `bitcoin/bips#1951` — https://github.com/bitcoin/bips/pull/1951 — formato armored exacto, header strings, regla `<0;1>/*`
- Hilo Delving Bitcoin — https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607 — racional + threat model
- Crate repo — https://github.com/pythcoiner/encrypted_backup — para resolver el `rev` SHA exacto al pinear (D-04)
- GUI nativa de referencia — https://github.com/pythcoiner/bed — comportamiento UX de referencia (no copiar UI; verificar contract de `.bed`)

### Memoria del usuario aplicable
- `feedback_verificar_no_inventar.md` — toda afirmación técnica contrastada en fuente primaria
- `feedback_no_circles.md` — diagnosticar a fondo antes de proponer fixes
- `feedback_castellano_no_argentino.md` — mensajes de error y docs en castellano (tú/descarga/coge), no argentino
- `feedback_git_noreply_email.md` — usar `55397917+4rkad@users.noreply.github.com` en commits

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Reference impl `/tmp/bed-test/encrypted_backup/`** — fuente primaria para entender API de la crate antes de tocar código. Los archivos `desc.txt`, `wallet.bed`, `xpub.txt`, `key{1,2,3}.txt` son fixtures perfectos para round-trip y cross-implementation tests.
- **Skill `start9-packaging`** — NO se invoca en Phase 1 (es Phase 4); listada solo para no perderla.

### Established Patterns
- **Greenfield repo** — `descriptor-cifrado/` está vacío excepto `IDEA.md` + `CLAUDE.md` + `.planning/`. No hay convenciones de código establecidas; Phase 1 las crea (rustfmt config, clippy lints, error pattern, test pattern).
- **Stack pre-locked en CLAUDE.md** — versiones exactas, "What NOT to Use" como guard rails. Planner debe respetar literalmente.

### Integration Points
- `crates/core` expone API pública que `crates/server` consume — el contract entre ambos es interno al workspace y puede iterar libremente en Phase 1.
- `crates/server` expone HTTP contract público (`/api/encrypt`, `/api/decrypt`) que la SPA de Phase 2 consume — este contract es estable desde fin de Phase 1.

</code_context>

<specifics>
## Specific Ideas

- **Round-trip cross-implementation:** verificar que el `.bed` producido por nuestra app abre con la CLI `beb` de pythcoiner Y viceversa. Fixtures en `/tmp/bed-test/` permiten esto sin compilar nada externo.
- **Mensajes de error en castellano** — específicos por causa: "El descriptor debe incluir `<0;1>/*`", "Xpub no coincide con ningún cosigner del .bed", "El descriptor cifrado excede capacidad QR (X bytes)".
- **Aviso clave del threat model** (DOC-02 es Phase 4 pero la lógica nace aquí): "ninguna ubicación debe contener simultáneamente el `.bed` y una xpub del multisig" — Phase 1 NO lo persiste en docs, pero el wording del aviso ya queda fijado.

</specifics>

<deferred>
## Deferred Ideas

- **Drag-and-drop de archivos** (UX2-01) — Phase 2 / v2.
- **"Test decrypt" round-trip transparente antes de mostrar éxito** (UX2-02) — v2.
- **Display de checksum del descriptor recuperado** (UX2-03) — v2.
- **Mensajes específicos por tipo de error de sintaxis del descriptor** (UX2-04) — v2; Phase 1 mapea solo error genérico `DescriptorParse`.
- **Persistencia del toggle "guardar historial" cruza reinicios** (PERS-01) — v1.x con `file-models`; Phase 2 usa `AtomicBool` in-memory.
- **JSON log format vía env var** (parte de D-18) — defer salvo que CI lo requiera.
- **History endpoints stub en Phase 1** — descartado; Phase 1 sin código de history (HIST-* todos en Phase 2).
- **`config.json` para override de bind addr** — descartado; constante hardcoded en Phase 1, StartOS no lo necesita (D-26).

### Reviewed Todos (not folded)
None — sin todos en backlog.

</deferred>

---

*Phase: 01-crypto-core-http-api*
*Context gathered: 2026-05-05 (--auto mode)*
