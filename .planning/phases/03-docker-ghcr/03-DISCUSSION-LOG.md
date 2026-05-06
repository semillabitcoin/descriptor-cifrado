# Phase 3: Docker + GHCR - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-06
**Phase:** 03-docker-ghcr
**Mode:** `--auto` (Claude auto-selected recommended option for every gray area)
**Areas discussed:** Build pipeline, Frontend build location, Tagging strategy, CI workflow split, Cache backend, GHCR visibility, ldd enforcement, Release profile, Distroless variant, Image name

---

## Build pipeline (multi-arch strategy)

| Option | Description | Selected |
|--------|-------------|----------|
| `docker buildx + QEMU` | Multi-arch en CI con emulación; sin toolchain extra; arm64 más lento | ✓ |
| `cross` | Toolchain Rust cross-compile nativo; más rápido pero requiere setup adicional | |
| Native arm64 runners | GHA arm64 runners (recientes); más simple pero coste/disponibilidad variable | |

**User's choice:** auto → buildx + QEMU
**Notes:** Patrón estándar GHA, suficiente para proyecto pequeño. Defer a runners nativos cuando GA + cost-effective.

---

## Frontend build location

| Option | Description | Selected |
|--------|-------------|----------|
| Stage Node en Dockerfile | Reproducible single-source; `npm ci` dentro Dockerfile | ✓ |
| Build pre-Docker en CI | Más rápido pero depende de toolchain CI; menos reproducible local | |

**User's choice:** auto → Node stage en Dockerfile
**Notes:** Reproducibilidad y `docker build .` self-contained.

---

## Tagging strategy

| Option | Description | Selected |
|--------|-------------|----------|
| `latest` + `sha-<short>` + semver | metadata-action genera; main → latest+sha; tag git → semver | ✓ |
| Solo `latest` | Simple pero sin trazabilidad de commits | |
| Solo semver | Requiere tag git para cada release; bloquea pull continuo de main | |

**User's choice:** auto → multi-tag con metadata-action
**Notes:** Patrón industria estándar; cubre dev (sha) + release (semver) + estable (latest).

---

## CI workflow split

| Option | Description | Selected |
|--------|-------------|----------|
| Nuevo `.github/workflows/docker.yml` | Separa concerns; ci.yml queda Rust-only | ✓ |
| Añadir jobs a `ci.yml` | Single workflow file; más acoplamiento | |

**User's choice:** auto → docker.yml separado
**Notes:** Permisos (`packages: write`) y triggers distintos justifican separación.

---

## Cache backend

| Option | Description | Selected |
|--------|-------------|----------|
| `type=gha` | GHA cache backend buildx; estándar y sin infra extra | ✓ |
| `type=registry` | Cache layer en propio GHCR; reutilizable cross-job pero overhead | |
| Sin cache | Build cold cada vez; descartado por tiempo CI | |

**User's choice:** auto → type=gha
**Notes:** Default razonable, soporta `mode=max` para layer cache completo.

---

## GHCR visibility (PKG-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Step automatizado `gh api` | Idempotente; toggle público tras primer push | ✓ |
| Toggle manual UI | Frágil, olvido = deploy falla | |

**User's choice:** auto → step automatizado
**Notes:** Memoria `feedback_ghcr_private_default.md` confirma necesidad.

---

## ldd enforcement (PKG-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Job separado `ldd-check` en docker.yml | Aislado, fácil debugging, build native amd64 | ✓ |
| Step dentro Docker build | Acoplado al build multi-arch; difícil leer logs | |
| Script post-deploy | Tarde, no bloquea push | |

**User's choice:** auto → job separado
**Notes:** Falla si aparece libssl/libcrypto/native-tls/libpq/libmysqlclient/libsqlite3.

---

## Release profile (binary size)

| Option | Description | Selected |
|--------|-------------|----------|
| strip+lto+codegen-units=1+panic=abort | Optimización agresiva; alineado con doctrina no-panic | ✓ |
| Solo `strip = true` | Menos optimización; binario más grande | |
| Default release profile | Sin optimizaciones extras; binario ~15-20 MB | |

**User's choice:** auto → optimización completa
**Notes:** Target binario ≤10 MB, imagen ≤25 MB compressed (PKG-01).

---

## Distroless variant

| Option | Description | Selected |
|--------|-------------|----------|
| `cc-debian12:nonroot` | UID 65532; defensa en profundidad; sin coste operacional | ✓ |
| `cc-debian12` | Root user; simple pero menos seguro | |
| `cc-debian12:debug` | Incluye busybox; útil debugging pero attack surface | |

**User's choice:** auto → :nonroot
**Notes:** bind 127.0.0.1:8080 no requiere capabilities privilegiadas.

---

## Image name

| Option | Description | Selected |
|--------|-------------|----------|
| `ghcr.io/semillabitcoin/bed-app` | Match exacto con ROADMAP success criteria #1 | ✓ |
| `ghcr.io/semillabitcoin/bed` | Más corto pero no documentado | |
| `ghcr.io/semillabitcoin/bed-server` | Refleja binario pero confunde con app completa | |

**User's choice:** auto → bed-app
**Notes:** ROADMAP lo nombra explícitamente.

---

## Claude's Discretion

- Versión exacta `rust:slim` tag (planner verifica current al implementar)
- Orden de COPY layers en Dockerfile (cache optimization)
- Si añadir `cargo-chef` (recomendado solo si build >10 min)
- Estructura interna de `docker.yml` (un job vs múltiples)
- Si añadir `docker scout`/`trivy` scan (defer)

## Deferred Ideas

- SBOM generation
- cosign signing
- Vulnerability scan en CI
- cargo-chef para cache deps
- Provenance attestations
- Branch-specific tags
- PR preview deploys
- Native arm64 runners
- Container size tracker en PR comment
</content>
</invoke>