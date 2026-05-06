# Phase 3: Docker + GHCR - Research

**Researched:** 2026-05-06
**Domain:** Docker multi-stage builds, GitHub Actions buildx, GHCR visibility, Rust release profiles
**Confidence:** HIGH (locked decisions verified; unknowns resolved below)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Dockerfile multi-stage 3 stages: `frontend-builder` (node:20-alpine) → `rust-builder` (rust:1-slim) → `runtime` (gcr.io/distroless/cc-debian12:nonroot). EXPOSE 8080. ENTRYPOINT ["/usr/local/bin/bed-server"]. USER nonroot (UID 65532).
- **D-02:** Single Dockerfile, no per-arch Dockerfiles. buildx resuelve $TARGETPLATFORM.
- **D-03:** .dockerignore raíz excluye target/, frontend/node_modules/, frontend/dist/, .planning/, .git/, .github/, *.md excepto LICENSE, **/.DS_Store, **/.env*.
- **D-04:** Multi-arch via docker buildx + QEMU en GHA. Plataformas: linux/amd64,linux/arm64. No usar `cross`. QEMU ~3-5x más lento en arm64, aceptado.
- **D-05:** Build trigger: push main + tags v*.*.* → push a GHCR. PRs → build only (push: false).
- **D-06:** Tags via docker/metadata-action: main → latest + sha-<7>. Tag v0.1.0 → 0.1.0 + 0.1 + 0 + latest. PRs → no tags.
- **D-07:** OCI labels: image.source, image.description, image.licenses, image.revision, image.created.
- **D-08:** [profile.release] en Cargo.toml raíz: strip=true, lto="thin", codegen-units=1, opt-level=3, panic="abort".
- **D-09:** Nuevo workflow .github/workflows/docker.yml, separado de ci.yml.
- **D-10:** 3 jobs en docker.yml: (1) build-and-push, (2) ldd-check, (3) make-public.
- **D-11:** Permissions en docker.yml: contents:read, packages:write.
- **D-12:** Image name: ghcr.io/semillabitcoin/bed-app.
- **D-13:** Distroless variant: gcr.io/distroless/cc-debian12:nonroot. UID 65532 / GID 65532.
- **D-14:** Build args RUST_VERSION (default 1) y NODE_VERSION (default 20). Reproducibilidad via Cargo.lock + package-lock.json + --locked + npm ci.
- **D-15:** Sin SBOM ni cosign en v1.

### Claude's Discretion

- Versión exacta `rust:slim` tag — planner decide tras verificar en Docker Hub al implementar.
- Orden exacto de COPY layers para maximizar cache.
- Si añadir cargo-chef (recomendado si build sin cache >10 min).
- Estructura interna de docker.yml (naming de jobs).
- Si fijar rust:1.83-slim-bookworm o dejar rust:1-slim-bookworm.
- Si añadir docker scout/trivy.

### Deferred Ideas (OUT OF SCOPE)

- SBOM (--sbom=true), cosign signing, docker scout/trivy, cargo-chef (solo si build excede tiempo), provenance attestations, branch images (:dev/:next), PR images publicadas, native arm64 runners, Alpine variants, image size budget tracker.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PKG-01 | Dockerfile multi-stage rust:slim → distroless/cc-debian12 produce imagen ≤25 MB | D-08 release profile (strip+lto+panic=abort) + distroless/cc-debian12 base ~3-5 MB + binario ≤10 MB → total ≤25 MB factible. Ver "Image Size Budget" en Pitfalls. |
| PKG-02 | Imagen multi-arch (amd64 + arm64) publicada en GHCR bajo org semillabitcoin | D-04 buildx+QEMU resuelve ambas arches. Metadata-action v6 + build-push-action v7 son las versiones actuales (Mayo 2026). |
| PKG-03 | CI corre ldd sobre binario y falla si aparece libssl o lib no-distroless | ldd-check job nativo amd64 en ubuntu-latest. Patrón grep script verificado. Distroless cc-debian12 incluye libgcc_s, libc, libm — esos son OK. |
| PKG-04 | Imagen GHCR se marca pública inmediatamente tras primer push | Si el repo `semillabitcoin/<repo>` es público Y el Dockerfile incluye `org.opencontainers.image.source` label ANTES del primer push, el package hereda visibilidad pública automáticamente. Ver sección "GHCR Visibility" — el endpoint API /user/packages/... es para paquetes de usuario, no de org. Se recomienda alternativa de herencia automática. |
</phase_requirements>

---

## Summary

Phase 3 agrega tres artefactos: `Dockerfile` (raíz), `.dockerignore` (raíz), y `.github/workflows/docker.yml`. Además modifica `Cargo.toml` raíz para añadir `[profile.release]`. Todos los otros archivos del proyecto permanecen inalterados.

La investigación confirmó que las versiones de Docker GitHub Actions han avanzado desde los valores asumidos en CONTEXT.md: el ecosistema completo migró a **Node 24 como runtime** en Marzo 2026, resultando en versiones v4 para setup-qemu, setup-buildx y login-action; v6 para metadata-action; y v7 para build-push-action. El planner DEBE usar estas versiones actuales — las v3/v5/v6 mencionadas en CONTEXT.md son una versión atrás.

El mecanismo de visibilidad pública de GHCR para paquetes de **organización** (no usuario) tiene una particularidad crítica: la decisión D-10 propone `gh api -X PATCH /user/packages/container/bed-app/visibility` pero ese endpoint aplica a paquetes de usuario, **no a paquetes de org**. Para un paquete bajo `ghcr.io/semillabitcoin/...`, el endpoint correcto sería `/orgs/semillabitcoin/packages/container/bed-app` — pero la investigación revela que ese endpoint PATCH de visibilidad **no está oficialmente documentado ni confirmado como funcional** para organizaciones. La alternativa confiable: si el repo es público y el `Dockerfile` incluye la label `org.opencontainers.image.source` ANTES del primer push, GHCR hereda automáticamente la visibilidad pública del repo cuando se usa `GITHUB_TOKEN`.

El Rust binary de este proyecto con el release profile D-08 (strip+lto+panic=abort) debería producir un binario de ~5-8 MB basado en el binario actual (5.8 MB sin strip). La imagen distroless/cc-debian12 base es ~3-5 MB comprimida. Total estimado: ~10-15 MB comprimido, bien dentro del límite de 25 MB.

**Primary recommendation:** Usar versiones actuales de docker/* actions (v4/v6/v7), configurar `org.opencontainers.image.source` label en Dockerfile como mecanismo primario de visibilidad pública, y confirmar que el repo `semillabitcoin/<nombre>` es público antes del primer push.

---

## Standard Stack

### Core (Docker GitHub Actions — versiones verificadas Mayo 2026)

| Action | Version | Purpose | Why Standard |
|--------|---------|---------|--------------|
| `actions/checkout` | `@v4` | Checkout repo | Ya en uso en ci.yml |
| `docker/setup-qemu-action` | **`@v4`** | QEMU para emulación arm64 | Versión actual (Marzo 2026, Node 24). v3 es anterior. |
| `docker/setup-buildx-action` | **`@v4`** | BuildKit multi-arch | Versión actual (Marzo 2026). Elimina inputs deprecados. |
| `docker/login-action` | **`@v4`** | Auth a GHCR con GITHUB_TOKEN | Versión actual (Marzo-Abril 2026). |
| `docker/metadata-action` | **`@v6`** | Tags + OCI labels automáticos | Versión actual (Marzo 2026, Node 24). v5 es anterior. |
| `docker/build-push-action` | **`@v7`** | Build multi-arch + push | Versión actual. v6.19.2 es la última v6; v7.1.0 es latest overall. |

**AVISO AL PLANNER:** CONTEXT.md menciona `@v3`/`@v5`/`@v6` porque fue escrito antes de Marzo 2026. Las versiones actuales correctas son las de la tabla anterior. Usar v7 para build-push-action requiere Actions Runner v2.327.1+; GitHub-hosted runners en ubuntu-latest ya lo cumplen.

### Base Images

| Image | Tag | Purpose | Notes |
|-------|-----|---------|-------|
| `node` | `20-alpine` | Frontend build (stage 1) | D-01 locked. Node 20 LTS. Alpine solo en build, no en runtime. |
| `rust` | `1-slim-bookworm` | Rust build (stage 2) | Recomendado: `rust:1-slim-bookworm` (Debian Bookworm = Debian 12) para alineación con cc-debian12. `rust:1-slim` apunta ahora a Trixie (Debian 13) — diferente glibc. Ver nota abajo. |
| `gcr.io/distroless/cc-debian12` | `:nonroot` | Runtime (stage 3) | D-13 locked. UID 65532 / GID 65532. Contiene: glibc, libgcc_s, libstdc++ mínimo. |

**Nota crítica sobre rust:1-slim vs rust:1-slim-bookworm:** Según Docker Hub (verificado Mayo 2026), `rust:1-slim` ahora apunta a Trixie (Debian 13), mientras que `rust:1-slim-bookworm` apunta a Debian 12 (Bookworm). Como `distroless/cc-debian12` es Debian 12, la alineación correcta para evitar incompatibilidades de glibc es **`rust:1-slim-bookworm`**. El planner DEBE usar `rust:1-slim-bookworm`, no `rust:1-slim`.

### Cargo Release Profile (D-08 — sin cambios existentes)

```toml
[profile.release]
strip = true
lto = "thin"
codegen-units = 1
opt-level = 3
panic = "abort"
```

Verificado: `Cargo.toml` raíz actual NO tiene `[profile.release]`. Esta es una adición nueva. La opción `panic = "abort"` es consistente con los workspace lints `unwrap_used = "deny"` / `expect_used = "deny"` ya presentes.

**Installation (no npm/cargo install needed — todo es infra/config):**
```bash
# Verificar versión actual de rust:1-slim-bookworm en Docker Hub
docker pull rust:1-slim-bookworm
docker run --rm rust:1-slim-bookworm rustc --version
# -> rustc 1.95.0 (a6e9c5c5a 2026-04-22) aprox.

# Verificar disponibilidad de buildx (ya instalado en el entorno)
docker buildx version
# -> github.com/docker/buildx v0.33.0 (verificado en este entorno)
```

---

## Architecture Patterns

### Recommended Project Structure (archivos nuevos/modificados)

```
descriptor-cifrado/
├── Dockerfile                    # NUEVO — 3-stage build
├── .dockerignore                 # NUEVO — excluye target/, node_modules/, etc.
├── Cargo.toml                    # MODIFICADO — añade [profile.release]
└── .github/
    └── workflows/
        ├── ci.yml                # SIN CAMBIOS
        └── docker.yml            # NUEVO — 3 jobs: build-and-push, ldd-check, make-public
```

### Pattern 1: Dockerfile 3-Stage Multi-Arch

**What:** Tres stages separados para frontend build, Rust build, y runtime mínimo.
**When to use:** Siempre — es el D-01 locked.

```dockerfile
# Source: distroless Rust example + D-01 decisions
ARG NODE_VERSION=20
ARG RUST_VERSION=1

# ── Stage 1: Frontend ────────────────────────────────────────────────────
FROM node:${NODE_VERSION}-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build
# Output: /app/frontend/dist/

# ── Stage 2: Rust Build ──────────────────────────────────────────────────
FROM rust:${RUST_VERSION}-slim-bookworm AS rust-builder
WORKDIR /app

# COPY layers ordered for max cache reuse (Claude's Discretion):
# 1. Cargo workspace config (changes rarely)
COPY Cargo.toml Cargo.lock rust-toolchain.toml deny.toml ./
# 2. Crate manifests only (fake-build trick for dep caching)
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml
# NOTE: cargo-chef alternativa si build excede 10 min sin cache

# 3. Frontend dist (rust-embed reads ../../frontend/dist/ from crates/server/)
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# 4. Source code (invalidates most frequently)
COPY crates/ crates/

RUN cargo build --release --locked --bin bed-server

# ── Stage 3: Runtime ─────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=rust-builder /app/target/release/bed-server /usr/local/bin/bed-server
# USER nonroot ya está configurado por el tag :nonroot (UID 65532)
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/bed-server"]
```

**Notas críticas del Dockerfile:**
- La ruta `crates/server/src/assets.rs` usa `#[folder = "../../frontend/dist/"]` con ruta relativa desde `crates/server/`. El Dockerfile debe reproducir esa estructura: `frontend/dist/` debe existir en `/app/frontend/dist/` antes de `cargo build`.
- `rust-toolchain.toml` contiene `channel = "stable"` — no override en Dockerfile, usar el toolchain del repo.
- No añadir `HEALTHCHECK` (distroless no tiene shell; Phase 4 s9pk lo gestiona).

### Pattern 2: docker.yml Workflow

**What:** Workflow GHA separado con 3 jobs para build+push, ldd check, y visibilidad pública.
**When to use:** Siempre — D-09 locked.

```yaml
# .github/workflows/docker.yml
name: Docker

on:
  push:
    branches: [main]
    tags: ['v*.*.*']
  pull_request:
    branches: [main]

permissions:
  contents: read
  packages: write

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v4

      - name: Log in to GHCR
        if: github.event_name != 'pull_request'
        uses: docker/login-action@v4
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v6
        with:
          images: ghcr.io/semillabitcoin/bed-app
          tags: |
            type=ref,event=branch,enable=${{ github.ref == 'refs/heads/main' }},value=latest
            type=sha,format=short,prefix=sha-
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=semver,pattern={{major}}
          flavor: |
            latest=false
          labels: |
            org.opencontainers.image.source=https://github.com/semillabitcoin/bed-app
            org.opencontainers.image.description=BED — Bitcoin Encrypted Backup app for StartOS
            org.opencontainers.image.licenses=MIT

      - name: Build and push
        uses: docker/build-push-action@v7
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: ${{ github.event_name != 'pull_request' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  ldd-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build release binary (native amd64)
        run: cargo build --release --locked --bin bed-server
      - name: ldd check — no libssl or non-distroless libs
        run: |
          echo "=== ldd output ==="
          ldd target/release/bed-server
          if ldd target/release/bed-server | grep -E 'libssl|libcrypto|native-tls|libpq|libmysqlclient|libsqlite3'; then
            echo "ERROR: Binary links to forbidden library"
            exit 1
          fi
          echo "=== ldd check PASSED ==="

  make-public:
    runs-on: ubuntu-latest
    needs: build-and-push
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - name: Make GHCR package public (org endpoint)
        run: |
          gh api -X PATCH \
            /orgs/semillabitcoin/packages/container/bed-app \
            -f visibility=public
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**AVISO CRÍTICO sobre make-public (ver sección GHCR Visibility abajo):** El endpoint PATCH para paquetes de organización no está completamente documentado. Si el repo es público y tiene `org.opencontainers.image.source` en el Dockerfile, la herencia automática puede ser suficiente. El job `make-public` es un safety net pero puede no funcionar sin un PAT con `write:packages` scope. Ver Pitfall #3.

### Pattern 3: Metadata-Action Tags — Comportamiento Esperado

| Evento | Tags generados |
|--------|----------------|
| push a `main` | `ghcr.io/semillabitcoin/bed-app:latest`, `ghcr.io/semillabitcoin/bed-app:sha-a1b2c3d` |
| push tag `v0.1.0` | `0.1.0`, `0.1`, `0`, `latest` |
| PR | Sin tags, build sin push |

**Nota:** `flavor: latest=false` + `type=ref,event=branch,enable=${{ github.ref == 'refs/heads/main' }},value=latest` produce `latest` solo en push a `main`, no en otras ramas. Si no se usa `enable=`, metadata-action v6 genera `latest` también para otras ramas.

### Anti-Patterns to Avoid

- **rust:1-slim sin sufijo:** Desde Marzo 2026 apunta a Trixie (Debian 13). Usar `rust:1-slim-bookworm` para alinear con distroless/cc-debian12 (Debian 12).
- **COPY . . antes de COPY Cargo.toml:** Invalida todo el cache de deps con cada cambio de código fuente.
- **Compilar frontend fuera del Dockerfile:** El `frontend/dist/` no debe copiarse desde el host; debe construirse dentro del stage `frontend-builder`.
- **login-action sin condicional:** Fallaría en PRs donde no hay GITHUB_TOKEN con write:packages. Usar `if: github.event_name != 'pull_request'`.
- **`ldd` sin grep + exit 1:** `ldd` nunca falla solo; el check requiere grep explícito.
- **Usar `type=gha` cache en buildx sin mode=max:** Sin `mode=max`, los stages intermedios (frontend-builder, rust-builder) no se cachean — solo el final.
- **Omitir `--locked` en cargo build del Dockerfile:** Sin `--locked`, el build puede usar versiones de deps distintas al Cargo.lock del repo.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tag generation (latest/sha/semver) | Script bash que parsea git refs | `docker/metadata-action@v6` | Maneja edge cases (tag en PR, versión major/minor/patch semver, sha acortado). |
| Multi-arch build orchestration | Múltiples `docker build` + `docker manifest create` | `docker/build-push-action@v7` con `platforms:` | BuildKit unifica internamente, produce manifest list, gestiona cache por arch. |
| QEMU setup | apt-get install qemu-* | `docker/setup-qemu-action@v4` | Configura binfmt_misc correctamente para buildx. |
| Buildx builder creation | `docker buildx create` manual | `docker/setup-buildx-action@v4` | Crea builder compatible con GHA cache backend y provenance. |
| OCI label injection | Vars hardcodeadas en Dockerfile | `docker/metadata-action@v6` outputs `labels` | Labels dinámicos con sha + timestamp + version correctos por evento. |
| Image size check | Script manual curl al registry | `docker buildx imagetools inspect --raw` + jq | Parseado correcto de manifest lists multi-arch. |

**Key insight:** El ecosistema docker/* actions resuelve correctamente todos los edge cases de multi-arch + caching + tagging. Implementar cualquiera de estos manualmente introduce bugs sutiles (e.g., `latest` en PRs, cache miss por branch restrictions).

---

## GHCR Visibility — Investigación Crítica (PKG-04)

### Contexto del Problema

CONTEXT.md D-10 propone `gh api -X PATCH /user/packages/container/bed-app/visibility` para el step `make-public`. **Este endpoint es para paquetes de usuario, no de organización.**

### Mecanismos Disponibles (en orden de confiabilidad)

**Mecanismo 1: Herencia automática desde repo público (RECOMENDADO)**

Condición: Si el repo `semillabitcoin/<nombre>` es público Y la label `org.opencontainers.image.source` está en el Dockerfile **antes** del primer push, el package de GHCR hereda la visibilidad pública automáticamente cuando se usa `GITHUB_TOKEN`.

Fuente: GitHub Docs "Publishing and installing a package with GitHub Actions" — "The package inherits the visibility and permissions model of the repository where the workflow is run."

Implicación práctica: Si el repo es público (como debe ser para `semillabitcoin/bed-app` dado que es open source), y el Dockerfile incluye la label `org.opencontainers.image.source=https://github.com/semillabitcoin/bed-app`, el primer push resultará en un paquete público sin step adicional.

**Esta es la alternativa más simple y la que el planner DEBE explorar primero.**

**Mecanismo 2: API PATCH para paquetes de organización (INCERTIDUMBRE)**

Endpoint candidato: `PATCH /orgs/semillabitcoin/packages/container/bed-app`

Estado: **No oficialmente documentado para PATCH de visibilidad.** Investigación encontró discussions de community que reportan que este endpoint puede devolver 404 o funcionar sin documentación formal. No se puede garantizar que funcione con solo `GITHUB_TOKEN` — puede requerir un PAT con scope `write:packages` del owner de la org.

**Mecanismo 3: Configuración manual en GitHub UI**

Fallback garantizado: El owner de la org `semillabitcoin` puede ir a `github.com/orgs/semillabitcoin/packages/container/bed-app/settings` y cambiar visibilidad a Public manualmente tras el primer push. **Una vez público, no se puede hacer privado de nuevo (por diseño de GHCR).**

### Recomendación para el Planner

El job `make-public` debe:
1. Intentar `PATCH /orgs/semillabitcoin/packages/container/bed-app` con `GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}`
2. Si falla (código de salida distinto de 0), continuar (`continue-on-error: true`) con un warning visible
3. En Wave 0 del plan, incluir una task manual: "Verificar que el repo `semillabitcoin/<nombre>` es público antes del primer push, y que el Dockerfile incluye `org.opencontainers.image.source` label"

---

## Common Pitfalls

### Pitfall 1: GHA Cache — Cross-Branch Cache Miss en PRs

**What goes wrong:** PRs no pueden acceder al cache de ramas que no sean base branch y default branch. Si el primer build de un PR es un cold start, el build-and-push job tardará 15-25 min (incluye QEMU arm64 emulación).

**Why it happens:** GitHub cache service API v2 restringe acceso por branch por razones de seguridad. Un PR desde `feature/X` solo puede leer cache de `feature/X` y del default branch (`main`).

**How to avoid:** El primer push a `main` establece el cache GHA. Los PRs subsiguientes reutilizarán ese cache base. El primer run siempre será lento — es normal.

**Warning signs:** Build arm64 tardando >20 min en el segundo run de un PR no es normal — revisar si el cache se está usando.

### Pitfall 2: `rust:1-slim` apunta a Trixie (Debian 13) desde Marzo 2026

**What goes wrong:** Si se usa `rust:1-slim` (sin sufijo), el builder está en Debian 13 pero el runtime `distroless/cc-debian12` es Debian 12. Los símbolos glibc compilados en Trixie pueden ser más nuevos que los disponibles en Debian 12, causando error "GLIBC_2.3X not found" al ejecutar el binario en el contenedor runtime.

**Why it happens:** Docker Hub cambió el tag `rust:1-slim` para apuntar a Trixie cuando Trixie se volvió stable.

**How to avoid:** Usar `rust:1-slim-bookworm` explícitamente. Bookworm = Debian 12 = misma base que `distroless/cc-debian12`.

**Warning signs:** Error en `docker run` del contenedor final: `ldd` o el proceso muere con "version 'GLIBC_2.XX' not found".

### Pitfall 3: `make-public` step con endpoint /user/packages/ para paquetes de org

**What goes wrong:** El endpoint `/user/packages/container/bed-app/visibility` aplica a paquetes del usuario autenticado (el GITHUB_TOKEN actúa como `github.actor`). Si el paquete está bajo la org `semillabitcoin`, este endpoint devolverá 404.

**Why it happens:** GHCR distingue entre user-scoped (`ghcr.io/<username>/...`) y org-scoped (`ghcr.io/<org>/...`) packages.

**How to avoid:** Ver sección GHCR Visibility. Usar `/orgs/semillabitcoin/packages/container/bed-app` con `continue-on-error: true`, y confiar en herencia automática de visibilidad si el repo es público.

**Warning signs:** El step `make-public` retorna 404. El paquete sigue siendo privado tras el primer push.

### Pitfall 4: rust-embed ruta relativa — estructura del WORKDIR

**What goes wrong:** `crates/server/src/assets.rs` declara `#[folder = "../../frontend/dist/"]`. Esta ruta es relativa al directorio del crate en compile time. Si `cargo build` se ejecuta desde `/app` (WORKDIR), la ruta resuelve a `/app/frontend/dist/`. Si el COPY del Dockerfile coloca `frontend/dist/` en otra ubicación, el build falla con "folder not found".

**Why it happens:** `rust-embed` resuelve la ruta en compile time relativa al `CARGO_MANIFEST_DIR` del crate.

**How to avoid:** WORKDIR del rust-builder stage = `/app`. Dockerfile debe: `COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist`. La ruta `/app/frontend/dist/` es lo que rust-embed busca desde `crates/server/` (que está en `/app/crates/server/`).

**Warning signs:** Error de compilación en el stage rust-builder: "directory '../../frontend/dist' not found" o "no assets to embed".

### Pitfall 5: Binario excede 25 MB comprimido

**What goes wrong:** Si el release profile no se aplica correctamente (e.g., Cargo.toml malformado), el binario sin strip puede ser ~15-20 MB, empujando la imagen total por encima de 25 MB comprimida.

**Why it happens:** Sin `strip = true`, el binario incluye debug symbols (~2x tamaño). Sin `lto = "thin"`, el código puede ser menos optimizado.

**How to avoid:** Verificar el perfil está en el `[profile.release]` del workspace root Cargo.toml (no en un crate individual). La sección debe ser `[profile.release]` exactamente (no `[profile.release.package.bed-server]`). El binario actual (Phase 2) en modo release es 5.8 MB sin strip — con strip se espera ~3-5 MB.

**Warning signs:** `ls -lh target/release/bed-server` muestra >10 MB antes del COPY al stage runtime.

### Pitfall 6: `ldd` en el ldd-check job — semantica

**What goes wrong:** `ldd target/release/bed-server` siempre retorna exit code 0, incluso cuando el binario tiene libssl. El job no falla sin el grep explícito.

**Why it happens:** `ldd` reporta libs pero no falla si las encuentra. El check de CI debe usar `grep -E 'pattern' && exit 1`.

**How to avoid:** Script del ldd-check job:
```bash
if ldd target/release/bed-server | grep -E 'libssl|libcrypto|native-tls|libpq|libmysqlclient|libsqlite3'; then
  echo "ERROR: Binary links to forbidden library"
  exit 1
fi
```

**Warning signs:** El job ldd-check "pasa" aunque el binario tenga libssl — revisar que el script incluye `exit 1`.

---

## Code Examples

### Dockerfile Completo (Patrón Recomendado)

```dockerfile
# Source: D-01 decisions + distroless official Rust example
# https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile
ARG NODE_VERSION=20
ARG RUST_VERSION=1

FROM node:${NODE_VERSION}-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:${RUST_VERSION}-slim-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY deny.toml ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist
COPY crates/ crates/
RUN cargo build --release --locked --bin bed-server

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=rust-builder /app/target/release/bed-server /usr/local/bin/bed-server
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/bed-server"]
```

### .dockerignore Raíz

```
target/
frontend/node_modules/
frontend/dist/
.planning/
.git/
.github/
**/.DS_Store
**/.env*
**/.env
*.md
!LICENSE
```

### Cargo.toml — Adición [profile.release]

```toml
# Añadir al final de Cargo.toml raíz (workspace)
[profile.release]
strip = true
lto = "thin"
codegen-units = 1
opt-level = 3
panic = "abort"
```

### Verificación de Tamaño de Imagen en CI

```bash
# Tras push a GHCR, inspeccionar tamaño desde manifest (sin pull):
docker buildx imagetools inspect --raw ghcr.io/semillabitcoin/bed-app:sha-<x> | \
  python3 -c "
import json, sys
data = json.load(sys.stdin)
total = sum(l.get('size', 0) for m in data.get('manifests', []) for l in m.get('layers', []))
print(f'Total layers size: {total / 1024 / 1024:.1f} MB')
" || echo "Use: docker pull + docker image ls para verificar post-build"
```

**Alternativa simple (post-pull local):**
```bash
docker pull ghcr.io/semillabitcoin/bed-app:sha-<x>
docker image ls ghcr.io/semillabitcoin/bed-app --format "{{.Size}}"
# Nota: docker image ls muestra tamaño descomprimido. Para comprimido, usar registry API.
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `docker/setup-qemu-action@v3` | `@v4` | Marzo 2026 | Node 24 runtime requerido; Actions Runner v2.327.1+ |
| `docker/setup-buildx-action@v3` | `@v4` | Marzo 2026 | Elimina inputs deprecados; Node 24 |
| `docker/login-action@v3` | `@v4` | Marzo 2026 | Node 24; fix para scoped Docker Hub cleanup |
| `docker/metadata-action@v5` | `@v6` | Marzo 2026 | Node 24; ESM migration |
| `docker/build-push-action@v6` | `@v7` | Abril 2026 | Node 24; elimina env vars DOCKER_BUILD_NO_SUMMARY y legacy export-build |
| `rust:1-slim` = Debian 12 | `rust:1-slim` = Debian 13 (Trixie) | Cuando Trixie se volvió stable | Usar `rust:1-slim-bookworm` para alineación con distroless |

**Deprecated/outdated:**
- `docker/metadata-action@v5`: Anterior, no recibe nuevas features. Usar v6.
- `/user/packages/container/{name}/visibility` PATCH para paquetes de org: Endpoint incorrecto. Usar `/orgs/{org}/packages/container/{name}` o confiar en herencia automática.

---

## Open Questions

1. **¿Es el repo `semillabitcoin/bed-app` público en GitHub?**
   - What we know: La herencia automática de visibilidad GHCR requiere repo público.
   - What's unclear: El nombre exacto del repo en la org `semillabitcoin` (puede ser `descriptor-cifrado` o `bed-app` o similar).
   - Recommendation: El planner debe incluir en Wave 0 una task de verificación: "Confirmar que el repo es público en GitHub org semillabitcoin". Si no lo es, el mecanismo de herencia automática no aplica y se necesita configuración manual.

2. **¿Funciona PATCH /orgs/semillabitcoin/packages/container/bed-app con GITHUB_TOKEN?**
   - What we know: El endpoint para paquetes de usuario (`/user/packages/...`) funciona. El endpoint de org no está oficialmente documentado para PATCH de visibilidad. Algunos community reports lo marcan como 404.
   - What's unclear: Si el `GITHUB_TOKEN` en un workflow de la org tiene permisos suficientes para PATCH ese endpoint.
   - Recommendation: Incluir `continue-on-error: true` en el step `make-public`. Documentar en plan como "best-effort + fallback manual".

3. **¿Cuánto tarda el build arm64 QEMU en el runner ubuntu-latest sin cache?**
   - What we know: QEMU emulación es ~3-5x más lenta. Este proyecto tiene pocas deps (no redb, no SQLite) y un binario de ~6 MB. Cold build amd64 estimado ~3-5 min con cache deps. Con QEMU arm64: ~10-20 min cold start.
   - What's unclear: Si el timeout de 30 min default del job es suficiente para el primer cold build.
   - Recommendation: Añadir `timeout-minutes: 40` al job `build-and-push`. Con GHA cache activo, los rebuilds deberían ser <10 min.

4. **¿El fake-build trick para deps funciona con el workspace multi-crate?**
   - What we know: El patrón cargo-chef resuelve esto perfectamente. El fake-build trick (crear src/lib.rs vacíos) puede funcionar en workspaces pero es frágil.
   - What's unclear: Si vale la pena la complejidad del fake-build sin cargo-chef para este workspace.
   - Recommendation: Para v1, usar COPY layers sin fake-build trick. Si el build sin cache excede 10 min, añadir cargo-chef en un follow-up. El planner puede dejar un comment TODO en el Dockerfile.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Docker + BuildKit | Dockerfile build local | ✓ | 29.1.3 | — |
| `docker buildx` | Multi-arch build local | ✓ | v0.33.0 | — |
| Node.js | Frontend build (stage 1) | ✓ | v20.20.1 | — |
| `cargo` / `rustc` | Rust build + ldd-check | ✓ | 1.93.0 (stable) | — |
| `gh` CLI | make-public step (local test) | ✓ | 2.88.1 | — |
| GHCR registry | Image push | ✓ (vía CI) | — | — |
| GitHub Actions runner | CI execution | ✓ (ubuntu-latest en GHA) | — | — |
| QEMU (en CI) | arm64 emulation | ✓ (setup-qemu-action@v4 instala) | — | — |

**Missing dependencies with no fallback:** Ninguna — todas disponibles.

**Nota:** El build local multi-arch requiere `docker buildx create --use` si el builder actual no tiene soporte multi-platform. El GHA setup-buildx-action lo hace automáticamente.

---

## Sources

### Primary (HIGH confidence)
- Docker Hub oficial `hub.docker.com/_/rust/` — verificado Mayo 2026: `rust:1-slim` = Trixie, `rust:1-slim-bookworm` = Debian 12
- `github.com/docker/build-push-action/releases` — v7.1.0 es latest (Abril 2026), v6.19.2 es última v6
- `github.com/docker/metadata-action/releases` — v6.0.0 es latest (Marzo 2026)
- `github.com/docker/setup-qemu-action/releases` — v4.0.0 es latest (Marzo 2026)
- `github.com/docker/setup-buildx-action/releases` — v4.0.0 es latest (Marzo 2026)
- `github.com/docker/login-action/releases` — v4.1.0 es latest (Abril 2026)
- `docs.docker.com/build/ci/github-actions/multi-platform/` — workflow YAML multi-platform verificado
- `docs.docker.com/build/ci/github-actions/cache/` — sintaxis `type=gha,mode=max` verificada
- `github.com/GoogleContainerTools/distroless/examples/rust/Dockerfile` — patrón oficial Rust distroless
- `.planning/phases/03-docker-ghcr/03-CONTEXT.md` — decisiones D-01..D-15 locked
- `crates/server/src/assets.rs` — `#[folder = "../../frontend/dist/"]` ruta verificada localmente
- `Cargo.toml` raíz — confirmado: sin `[profile.release]` existente

### Secondary (MEDIUM confidence)
- GitHub Docs "Publishing and installing a package with GitHub Actions" — herencia de visibilidad con GITHUB_TOKEN verificada conceptualmente; mecanismo `org.opencontainers.image.source` no explícitamente probado para orgs
- WebSearch + distroless issue #1795 — glibc/libgcc_s.so en cc-debian12 con rust:bookworm confirmado
- WebSearch cargo-chef — benchmarks 5x speedup en proyectos >500 deps; este proyecto tiene ~30 deps directos, menos crítico

### Tertiary (LOW confidence)
- GHCR PATCH `/orgs/{org}/packages/container/{name}` visibility endpoint — community discussions indican que puede funcionar pero no está oficialmente documentado; tratar como best-effort con fallback manual
- QEMU arm64 build time estimate (~10-20 min cold) — basado en factor 3-5x + estimación de build amd64 de este proyecto; sin benchmark específico del proyecto

---

## Metadata

**Confidence breakdown:**
- Standard stack (Actions versions): HIGH — verificado en releases pages de cada repo, Mayo 2026
- Architecture (Dockerfile pattern): HIGH — basado en distroless official Rust example + CONTEXT.md decisions
- Distroless/cc-debian12 libs: MEDIUM — confirmado conceptualmente (glibc + libgcc_s), sin inspección directa de imagen
- GHCR visibility (org endpoint): LOW — endpoint no oficialmente documentado para PATCH; herencia automática es HIGH si repo es público
- QEMU build times: LOW — estimación, sin benchmark del proyecto específico

**Research date:** 2026-05-06
**Valid until:** 2026-06-06 (Docker actions ecosystem — estable; GHCR API — puede cambiar)
