# Phase 3: Docker + GHCR - Context

**Gathered:** 2026-05-06
**Status:** Ready for planning (auto-mode discussion)

<domain>
## Phase Boundary

Phase 3 empaqueta el binario `bed-server` (workspace Rust) + frontend Svelte 5 ya construido, en una imagen Docker multi-stage que termina en `gcr.io/distroless/cc-debian12:nonroot`, multi-arch (linux/amd64 + linux/arm64), publicada bajo `ghcr.io/semillabitcoin/bed-app` con visibilidad pública desde el primer push. Añade un workflow de CI separado que (a) compila la imagen multi-arch con `docker buildx`, (b) la pushea a GHCR en eventos disparadores, (c) corre un check `ldd` sobre el binario release nativo amd64 que falla si aparece `libssl`/`native-tls`/lib no-distroless, y (d) marca el package GHCR como público vía API tras el primer push.

**Fuera de Phase 3:**
- Manifest s9pk de StartOS, health checks Start9 SDK, install en device real (Phase 4 — S9-01..05).
- README threat-model docs (DOC-01/DOC-02 → Phase 4).
- Cualquier cambio al código Rust o frontend (Phase 1 + 2 estables).
- Release semver formal (`v0.1.0`); el tagging soporta semver pero el bump de versión es independiente y manual.
- Helm/Kubernetes/Compose orchestration (sólo imagen Docker en GHCR, consumida después por el wrapper s9pk).

</domain>

<decisions>
## Implementation Decisions

### Dockerfile Architecture
- **D-01:** Dockerfile multi-stage con **3 stages** en `Dockerfile` raíz del repo:
  1. `frontend-builder` — base `node:20-alpine`. Copia `frontend/package.json` + `frontend/package-lock.json`, ejecuta `npm ci`, copia `frontend/src/`, `frontend/vite.config.js`, etc., ejecuta `npm run build`. Output: `/app/frontend/dist/`.
  2. `rust-builder` — base `rust:1-slim` (pin a versión major estable, sin Alpine). Copia `Cargo.toml`, `Cargo.lock`, `crates/`, `rust-toolchain.toml`, `deny.toml`. Copia `--from=frontend-builder /app/frontend/dist /app/frontend/dist`. Ejecuta `cargo build --release --locked --bin bed-server` con `--target $TARGETARCH-equivalent` cuando aplique (ver D-04).
  3. `runtime` — base `gcr.io/distroless/cc-debian12:nonroot`. Copia el binario release. `USER nonroot` (UID 65532) heredado del tag. `EXPOSE 8080`. `ENTRYPOINT ["/usr/local/bin/bed-server"]`.
- **D-02:** Single Dockerfile, **no Dockerfile separado por arquitectura** — `buildx` resuelve `$TARGETPLATFORM` y `$BUILDPLATFORM` en build args para hacer cross-compile-friendly stages cuando sea posible.
- **D-03:** `.dockerignore` raíz excluye: `target/`, `frontend/node_modules/`, `frontend/dist/` (se rebuilden dentro del Dockerfile), `.planning/`, `.git/`, `.github/` (en builds locales), `*.md` excepto `LICENSE`, `**/.DS_Store`, `**/.env*`. Reduce build context y evita invalidaciones de cache espurias.

### Multi-Arch Build Strategy
- **D-04:** Multi-arch via **`docker buildx` + QEMU emulation** en GitHub Actions (`docker/setup-qemu-action@v3` + `docker/setup-buildx-action@v3`). Plataformas: `linux/amd64,linux/arm64`. **No usar `cross`**: añade toolchain-config friction; `buildx + QEMU` es el patrón estándar GHA. Tradeoff aceptado: compilación arm64 emulada es ~3-5x más lenta que nativa, pero el proyecto es pequeño y CI no es bottleneck crítico (ver D-09 caching para amortización).
- **D-05:** Build trigger en push a `main` y push de tags `v*.*.*`; PRs solo **validan build** (no push, `push: false`). Razón: evita publicar imágenes de PRs no merged + asegura que main siempre tiene `:latest` actualizado.

### Tagging Strategy
- **D-06:** Tags producidos via `docker/metadata-action@v5`:
  - Push a `main` → `latest` + `sha-<7chars>` (ej. `sha-a1b2c3d`).
  - Push de tag git `v0.1.0` → `0.1.0` + `0.1` + `0` + `latest`.
  - PRs → no tags (no se pushea).
- **D-07:** OCI labels obligatorias inyectadas via `metadata-action`:
  - `org.opencontainers.image.source = https://github.com/semillabitcoin/<repo>`
  - `org.opencontainers.image.description = "BED — Bitcoin Encrypted Backup app for StartOS"`
  - `org.opencontainers.image.licenses = MIT`
  - `org.opencontainers.image.revision = <sha>`
  - `org.opencontainers.image.created = <timestamp>` (auto)
  - GHCR usa `image.source` para link del package a este repo; sin esto el package queda huérfano en GHCR UI.

### Release Profile (binary size)
- **D-08:** Añadir a `Cargo.toml` raíz un perfil `[profile.release]`:
  ```toml
  [profile.release]
  strip = true            # quita debuginfo del binario
  lto = "thin"            # link-time optimization, balance speed/size
  codegen-units = 1       # mejor optimización a costa de compile time
  opt-level = 3           # default ya, explícito por claridad
  panic = "abort"         # ahorra ~100-200 KB del unwind machinery
  ```
  Target: binario release ≤10 MB → imagen total ≤25 MB compressed (PKG-01). `panic = "abort"` consistent con doctrine no-unwrap/no-panic en request path.

### CI Workflow Split
- **D-09:** Nuevo workflow `.github/workflows/docker.yml` separado de `ci.yml` (que queda Rust-only). Razones: matriz de jobs distinta (buildx + QEMU + GHCR auth solo en docker.yml), triggers distintos (PRs+main+tags vs solo PRs+main), permisos distintos (`packages: write` solo en docker.yml).
- **D-10:** Jobs en `docker.yml`:
  1. **`build-and-push`** — corre en push main / push tag / PR. Steps:
     - `actions/checkout@v4`
     - `docker/setup-qemu-action@v3` (multi-arch)
     - `docker/setup-buildx-action@v3`
     - `docker/login-action@v3` (GHCR, solo si no es PR — usa `${{ secrets.GITHUB_TOKEN }}`)
     - `docker/metadata-action@v5` (computa tags + labels)
     - `docker/build-push-action@v6` con:
       - `platforms: linux/amd64,linux/arm64`
       - `push: ${{ github.event_name != 'pull_request' }}`
       - `cache-from: type=gha`
       - `cache-to: type=gha,mode=max`
       - `tags`/`labels` desde metadata-action
  2. **`ldd-check`** — corre en cada PR + push main. Compila release amd64 nativo en runner GHA estándar, ejecuta `ldd target/release/bed-server` y `nm` chequeando símbolos. Falla si aparece cualquiera de: `libssl`, `libcrypto`, `native-tls`, `libpq`, `libmysqlclient`, `libsqlite3`. PKG-03 cumplido.
  3. **`make-public`** — corre solo en push a `main`, después de `build-and-push`, condicional en `success()`. Step usa `gh api -X PATCH /user/packages/container/bed-app/visibility` con body `{"visibility":"public"}` para forzar visibilidad pública. Idempotente (toggle de no-op si ya pública). Necesario por `feedback_ghcr_private_default.md` — paquetes GHCR son privados por default y rompen pull sin auth (PKG-04).
- **D-11:** Workflow permissions explícitos en `docker.yml`: `permissions: { contents: read, packages: write }`. Sin `id-token`/`OIDC` en v1 (auth con `GITHUB_TOKEN` es suficiente).

### Image Identity
- **D-12:** Image name: **`ghcr.io/semillabitcoin/bed-app`**. Coincide exactamente con ROADMAP success criteria #1 (`docker pull ghcr.io/semillabitcoin/bed-app:latest`). No usar variantes (`bed`, `bed-server`, `bed-startos`).
- **D-13:** Distroless variant: **`gcr.io/distroless/cc-debian12:nonroot`**. UID 65532 / GID 65532. El binario `bed-server` bindea a `127.0.0.1:8080` (Phase 1 SEC-02), por lo cual no requiere capabilities privilegiadas. Defensa en profundidad sin coste operacional.

### Build Context & Reproducibility
- **D-14:** Build args explícitos:
  - `RUST_VERSION` (default `1`, se puede pin a `1.83` etc. en Dockerfile via `ARG RUST_VERSION=1`).
  - `NODE_VERSION` (default `20`).
  - Sin args adicionales en v1; reproducibilidad garantizada por `Cargo.lock` + `package-lock.json` checked in + `--locked` en cargo build + `npm ci` (no `npm install`).
- **D-15:** Sin SBOM ni firmas (cosign) en v1. Defer a follow-up — la prioridad v1 es shippeable a StartOS, no compliance avanzada. Si Start9 lo exige luego, se añade.

### Claude's Discretion
- Versión exacta `rust:slim` tag (`rust:1-slim` vs `rust:1.83-slim`) — planner decide tras verificar imagen current en Docker Hub al momento de implementar.
- Orden exacto de COPY layers en Dockerfile para maximizar cache reuse (típicamente: COPY Cargo.toml + Cargo.lock + crates/*/Cargo.toml para "fake build" antes que código fuente).
- Si añadir `cargo-chef` para mejorar cache de deps Rust — recomendado si build no-cached >10 min, opcional v1.
- Estructura interna de `docker.yml` (un job vs steps en build-and-push job), naming convention de jobs.
- Si fijar `image: rust:1.83-slim-bookworm` (versión OS) o dejar default — recomendación: dejar default Debian 12 para alineación con `cc-debian12` runtime.
- Si añadir step `docker scout` o `trivy` para vulnerability scan de la imagen — útil pero no requerido por PKG-* requirements; defer si no agrega tiempo significativo.

### Folded Todos
None — `gsd-tools todo match-phase 3` retornó 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning
- `.planning/PROJECT.md` — Constraints (`rust:slim` → `distroless/cc`, target 5–10 MB, `semillabitcoin` org, GHCR público, email noreply)
- `.planning/REQUIREMENTS.md` §"Packaging — Docker / GHCR" — PKG-01..04 (image ≤25 MB, multi-arch, ldd no libssl, GHCR público)
- `.planning/ROADMAP.md` §"Phase 3: Docker + GHCR" — Goal + 3 success criteria
- `.planning/STATE.md` — Position actual (Phase 3 not started)
- `.planning/phases/01-crypto-core-http-api/01-CONTEXT.md` — Bind 127.0.0.1:8080 (D-13 distroless nonroot compatible), bans cargo-deny openssl-sys/native-tls/async-hwi (refuerza D-08, ldd check D-10)
- `.planning/phases/02-spa-frontend-history/02-CONTEXT.md` — Frontend dir `frontend/`, Vite output `frontend/dist/`, rust-embed lee `../../frontend/dist/` (D-39..43 Phase 2 → confirma layout que Phase 3 Dockerfile copia)
- `IDEA.md` — Brief original
- `CLAUDE.md` §"Docker Build Pipeline" + §"Recommended Stack" + §"What NOT to Use" — stack lock-in, distroless rationale, no openssl/native-tls

### Research artifacts
- `.planning/research/STACK.md` — distroless/cc-debian12 versión + rust:slim como build base + multi-arch consideraciones
- `.planning/research/PITFALLS.md` — pitfalls de distroless (sin shell, sin debug), cross-compile arm64
- `.planning/research/ARCHITECTURE.md` — overview general

### Existing CI / repo files (lectura local)
- `.github/workflows/ci.yml` — Pattern actual de jobs (fmt/clippy/test/audit/deny) — Phase 3 NO modifica este archivo, añade `docker.yml` paralelo
- `Cargo.toml` (raíz) — Workspace + lints + dependencies. Phase 3 añade `[profile.release]` (D-08)
- `rust-toolchain.toml` — `channel = "stable"`, profile minimal — respeta esto en Dockerfile (no override)
- `deny.toml` — bans openssl-sys/native-tls/async-hwi (refuerza D-10 ldd-check job)
- `frontend/package.json` + `frontend/vite.config.js` — referencia para Dockerfile frontend stage (D-01)
- `crates/server/src/main.rs` — bind addr `127.0.0.1:8080` (confirma D-13 nonroot OK)

### External specs (web — usar `WebFetch` cuando se necesite verificar versiones / patrones actuales)
- distroless cc-debian12 — https://github.com/GoogleContainerTools/distroless/tree/main/cc — variantes `:nonroot`, contenido exacto
- distroless usage — https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile — ejemplo Rust oficial
- docker buildx multi-platform — https://docs.docker.com/build/building/multi-platform/ — patrón QEMU
- docker/build-push-action v6 — https://github.com/docker/build-push-action — inputs actuales (`platforms`, `cache-from/to`)
- docker/metadata-action v5 — https://github.com/docker/metadata-action — sintaxis tags + labels
- docker/setup-qemu-action v3 — https://github.com/docker/setup-qemu-action
- docker/setup-buildx-action v3 — https://github.com/docker/setup-buildx-action
- docker/login-action v3 — https://github.com/docker/login-action — auth con `GITHUB_TOKEN`
- GHCR docs — https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry — auth, visibility API
- GHCR visibility API — https://docs.github.com/en/rest/packages/packages — endpoint `PATCH /user/packages/container/{package_name}/visibility` (D-10 make-public)
- cargo profile.release — https://doc.rust-lang.org/cargo/reference/profiles.html — strip/lto/codegen-units/panic semántica oficial
- OCI image labels — https://github.com/opencontainers/image-spec/blob/main/annotations.md — keys estándar (D-07)
- cargo-chef (opcional) — https://github.com/LukeMathWalker/cargo-chef — receta para cache-friendly Docker builds

### Memoria del usuario aplicable
- `feedback_ghcr_private_default.md` — GHCR packages privadas por default; obligatorio togglear pública tras primer push o Umbrel/Start9 fallan al pull. Razón directa de D-10 step `make-public`.
- `feedback_test_before_push.md` — tests locales no detectan sanitización ni conflictos de puertos; Phase 3 incluye explicit `ldd-check` job en CI (D-10) y la imagen final debe testearse `docker run` antes de declarar el plan completo.
- `feedback_git_noreply_email.md` — usar `55397917+4rkad@users.noreply.github.com` en commits.
- `feedback_castellano_no_argentino.md` — strings/labels (poco aplicable aquí, sólo OCI labels en inglés estándar).
- `feedback_verificar_no_inventar.md` — versiones de actions y bases distroless deben verificarse current en research/planning, no asumir tags exactos.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`frontend/dist/`** (output de Phase 2) — Phase 3 lo regenera dentro del Dockerfile via `frontend-builder` stage. No se commitea al repo (asumido).
- **`crates/server/` binary `bed-server`** — único binario release que el runtime stage copia.
- **`.github/workflows/ci.yml`** — Pattern de Actions versions (`@v4`, `@v2`, `@stable`) y estructura de jobs sirve de referencia para `docker.yml`.
- **`deny.toml`** — bans openssl-sys/native-tls/async-hwi son la razón estructural por la que `ldd` debería pasar; D-10 `ldd-check` job verifica esto en runtime real.

### Established Patterns
- **Versión pinning de Actions** — Phase 1 fijó canonical stable (`checkout@v4`, `Swatinem/rust-cache@v2`, `EmbarkStudios/cargo-deny-action@v2`). Phase 3 sigue el patrón con `docker/*-action@v3`/`@v5`/`@v6` actuales.
- **Workspace `--locked`** — Phase 1 usa `cargo test --workspace --locked` en CI. Dockerfile rust-builder stage hereda con `cargo build --release --locked --bin bed-server`.
- **rust-embed lee `../../frontend/dist/`** desde `crates/server/` (Phase 2 D-40). Dockerfile debe replicar esa estructura: copiar `frontend/dist/` al path correcto antes del `cargo build`.
- **Lints workspace `unwrap_used = "deny"`/`expect_used = "deny"`/`panic = "warn"`** (Phase 1) — alineado con `panic = "abort"` en release profile (D-08).

### Integration Points
- **Repo organización** — `semillabitcoin/<repo>`. El nombre exacto del repo determina `image.source` label. Si el repo se llama `bed-app` ya no hace falta sobreescribir; si es `descriptor-cifrado` o similar, el GHCR `bed-app` package no coincide con repo name pero `image.source` label hace el link.
- **Volume `/data/encrypted/`** — Phase 2 lo usa via env `BED_DATA_DIR` con default `/data/encrypted/`. Dockerfile no necesita `VOLUME` declaration (Phase 4 s9pk lo gestiona), pero el binario debe poder escribir ahí cuando StartOS monte el volume — distroless `:nonroot` necesita que el directorio sea propiedad de UID 65532. Phase 4 manifest gestiona ownership; Phase 3 sólo asegura que el binario respeta `BED_DATA_DIR`.
- **`EXPOSE 8080`** declarativo — orientativo para herramientas; el bind real lo hace el binario en `127.0.0.1:8080`. StartOS gestiona el routing externo.
- **GHCR auth** — `GITHUB_TOKEN` con scope `packages: write` es suficiente; no requiere PAT manual.

</code_context>

<specifics>
## Specific Ideas

- **Dockerfile en raíz del repo** (no `docker/Dockerfile`); convención más común para que `docker build .` funcione sin flags. `.dockerignore` también raíz.
- **Verificar tamaño imagen tras primer build** — pre-condición para mergear: `docker buildx imagetools inspect ghcr.io/semillabitcoin/bed-app:sha-<x>` reporta `linux/amd64` y `linux/arm64`, ambas ≤25 MB compressed (success criterion 3 + PKG-01).
- **Test manual end-to-end pre-merge**: tras primer push, hacer `docker pull ghcr.io/semillabitcoin/bed-app:sha-<x>` desde máquina limpia (sin `docker login`), `docker run -p 8080:8080 -v $(pwd)/data:/data/encrypted ghcr.io/semillabitcoin/bed-app:sha-<x>`, y comprobar que SPA carga en `http://localhost:8080` y un round-trip cifrar→descifrar funciona contra esa imagen. Cumple `feedback_test_before_push.md` para Phase 3.
- **`make-public` step idempotente** — el endpoint API acepta toggle a público incluso si ya lo es; no falla en runs subsiguientes. Útil porque el primer push tras crear el package es donde el toggle es crítico, y los siguientes son no-op.
- **`ldd` exit code semántica** — `ldd` no falla por sí solo cuando encuentra libs; el job debe `grep -E 'libssl|libcrypto|native-tls|libpq'` y fallar via `if grep -E ... ; then exit 1; fi`. Distroless `cc-debian12` incluye `libgcc_s`, `libc`, `libm` — esos son OK.
- **`nm --dynamic --defined-only target/release/bed-server`** complementa `ldd` para detectar symbol-level surprises (ej. símbolos OpenSSL inlined estáticamente). Opcional v1; recomendable.
- **`cargo-chef` defer** — si el build CI tarda >10 min sin cache, planner puede añadir patrón cargo-chef en rust-builder stage (separar deps de código). Mantener simple v1 si el build inicial sin chef ya es <8 min con buildx cache.
- **`rust:1-slim` vs `rust:1.83-slim` pin** — recomendación: usar `rust:1-slim` (major version) para evitar churn semanal; rebuild semanal o mensual del CI ya rota a versiones recientes vía base image refresh.
- **Sin `HEALTHCHECK` en Dockerfile** — Phase 4 (S9-03) define `sdk.healthCheck.checkPortListening` desde el manifest s9pk; añadir `HEALTHCHECK` Dockerfile sería redundante y requiere shell (ausente en distroless).

</specifics>

<deferred>
## Deferred Ideas

- **SBOM generation** (`docker buildx --sbom=true`) — útil para compliance pero no requerido por PKG-* ni Start9. Defer a follow-up si la audiencia (Start9 / usuario fiscal) lo pide.
- **cosign signing de imágenes** — firma criptográfica de la imagen GHCR. Defer; requeriría keypair management y un policy de verificación que excede scope v1.
- **`docker scout` / `trivy` vulnerability scan en CI** — añade tiempo CI; útil pero no en lista de requisitos. Defer si build empieza a tardar.
- **`cargo-chef` para acelerar cache** — solo si build CI excede tiempo aceptable (~10 min); medir antes de añadir complejidad.
- **Provenance attestations** — buildx `--provenance=true`, similar a SBOM. Defer.
- **Branch images (e.g. `:dev`, `:next`)** — sólo `latest` + `sha-<x>` + semver en v1; branch-specific tags si se establece cadencia de release branches.
- **PR images publicadas** — actualmente PRs solo build-validate sin push. Si en el futuro se quiere "PR preview deploy", añadir tag `pr-<num>` con TTL. Defer.
- **Native arm64 runners** — GitHub anunció runners arm64 nativos; cuando estén GA y baratos, migrar de QEMU a runners nativos para acortar build time. Defer hasta GA + cost-effective.
- **Multi-distro variants** (Alpine para usuarios que prefieran musl) — no en scope, distroless ganó la decisión de stack (CLAUDE.md "What NOT to Use"). Defer indefinidamente.
- **Container image size budget tracker** — script que compara tamaño imagen build vs límite 25 MB y reporta delta en PR comment. Nice-to-have, defer.

### Reviewed Todos (not folded)
None — `gsd-tools todo match-phase 3` retornó 0 matches.

</deferred>

---

*Phase: 03-docker-ghcr*
*Context gathered: 2026-05-06 (auto-mode)*
</content>
</invoke>