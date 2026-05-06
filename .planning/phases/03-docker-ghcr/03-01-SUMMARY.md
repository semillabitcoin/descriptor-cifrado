---
phase: 03-docker-ghcr
plan: 01
subsystem: infra
tags: [docker, dockerfile, distroless, rust-release-profile, dockerignore]

requires:
  - phase: 02-spa-frontend-history
    provides: frontend/dist/ via rust-embed (Vite build output embedded in binary)
  - phase: 01-crypto-core-http-api
    provides: bed-server binary, workspace Cargo.toml, cargo-deny bans (no openssl/native-tls)

provides:
  - Dockerfile 3-stage build (node:20-alpine → rust:1-slim-bookworm → distroless/cc-debian12:nonroot)
  - .dockerignore with 11 exclusions for minimal build context
  - "[profile.release] in workspace Cargo.toml (strip+lto+panic=abort)"
  - Docker image descriptor-cifrado:dev, 10 MB compressed, PKG-01 fulfilled

affects: [03-02, phase-04-startos-s9pk]

tech-stack:
  added:
    - "Dockerfile (multi-stage, distroless runtime)"
    - ".dockerignore"
    - "Cargo [profile.release] with strip=true, lto=thin, codegen-units=1, opt-level=3, panic=abort"
  patterns:
    - "3-stage Dockerfile: frontend-builder (Node alpine) → rust-builder (rust:slim-bookworm) → runtime (distroless nonroot)"
    - "COPY layer order for cache: workspace config → crate manifests → frontend/dist → source code"
    - "rust:1-slim-bookworm (Debian 12) aligned with distroless/cc-debian12 for glibc compat"
    - "rust-embed path /app/frontend/dist/ aligns with WORKDIR /app + COPY --from=frontend-builder"

key-files:
  created:
    - Dockerfile
    - .dockerignore
  modified:
    - Cargo.toml

key-decisions:
  - "rust:1-slim-bookworm (not rust:1-slim) — Debian 12 alignment with distroless/cc-debian12; rust:1-slim points to Trixie (Debian 13) since Mar 2026 causing glibc incompatibility"
  - "panic=abort in profile.release consistent with workspace lints unwrap_used=deny/expect_used=deny"
  - "COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist matches rust-embed folder path from crates/server/src/assets.rs"
  - "Checkpoint auto-approved: server binds 127.0.0.1:8080 (SEC-02); verified via --network host; SPA HTML served correctly"

patterns-established:
  - "Dockerfile COPY layers ordered for maximal cache reuse: workspace config → crate manifests → frontend dist → crate source"
  - "No HEALTHCHECK in Dockerfile (distroless has no shell; Phase 4 s9pk handles health checks via SDK)"
  - "No USER directive (tag :nonroot inherits UID 65532 automatically)"

requirements-completed: [PKG-01]

duration: 16min
completed: 2026-05-06
---

# Phase 3 Plan 01: Docker Multi-Stage Dockerfile + Release Profile Summary

**Distroless 3-stage Dockerfile (node:20-alpine → rust:1-slim-bookworm → distroless/cc-debian12:nonroot) producing a 10 MB compressed image with strip+lto Rust binary of 3.7 MB; PKG-01 fulfilled**

## Performance

- **Duration:** 16 min
- **Started:** 2026-05-06T20:51:44Z
- **Completed:** 2026-05-06T21:08:00Z
- **Tasks:** 4 (Tasks 1-3 executed; Task 4 checkpoint auto-approved after passing all automated checks)
- **Files modified:** 3

## Accomplishments

- Added `[profile.release]` to workspace Cargo.toml with strip=true, lto=thin, codegen-units=1, opt-level=3, panic=abort — binary shrank from 5.8 MB (Phase 2) to 3.7 MB (36% reduction)
- Created .dockerignore with 11 exclusions (target/, node_modules/, dist/, planning, git, github, DS_Store, env, md except LICENSE)
- Created 3-stage Dockerfile: frontend-builder (node:20-alpine + npm ci + vite build) → rust-builder (rust:1-slim-bookworm + cargo build --release --locked) → runtime (gcr.io/distroless/cc-debian12:nonroot)
- Docker image `descriptor-cifrado:dev` built at 10 MB compressed (26 MB uncompressed) — PKG-01 fulfilled (≤25 MB)
- Container verified: starts with "listening addr=127.0.0.1:8080", SPA HTML served with title "BED — Bitcoin Encrypted Backup", all assets embedded via rust-embed

## Exact Metrics

- **Binary size after strip+lto:** 3,814,312 bytes (3.7 MB) — was 5.8 MB without strip
- **Image compressed size:** 11,140,333 bytes (10 MB) — PKG-01 limit is 26,214,400 bytes (25 MB)
- **Image uncompressed (local docker):** 27.4 MB
- **Rust version in rust:1-slim-bookworm:** rustc 1.95.0 (59807616e 2026-04-14)
- **Workspace tests:** 27 tests, all passed (cargo test --workspace --locked)

## Task Commits

1. **Task 1: Añadir [profile.release] al workspace Cargo.toml** - `696367e` (chore)
2. **Task 2: Crear .dockerignore en raíz** - `6911820` (chore)
3. **Task 3: Crear Dockerfile multi-stage en raíz** - `407a772` (feat)
4. **Task 4: Checkpoint auto-approved** - all automated checks passed

## Files Created/Modified

- `Cargo.toml` — Added [profile.release] block with 5 settings (strip, lto, codegen-units, opt-level, panic)
- `Dockerfile` — 3-stage multi-stage build (frontend-builder + rust-builder + runtime distroless)
- `.dockerignore` — 11 exclusions to minimize build context and avoid cache-busting spurious invalidations

## Decisions Made

- Used `rust:1-slim-bookworm` (Debian 12) not `rust:1-slim` (now Trixie/Debian 13 since Mar 2026) to align glibc with distroless/cc-debian12 — critical to prevent GLIBC_2.3X not found errors at runtime
- COPY layer order in rust-builder stage optimized for cache reuse: Cargo.toml+Cargo.lock+rust-toolchain.toml+deny.toml → crate manifests → frontend/dist → crates source
- Checkpoint (Task 4) auto-approved per auto_mode instructions after all automated checks passed: image exists, compressed size 10 MB < 25 MB, container logs show "listening addr=127.0.0.1:8080", SPA HTML served

## Deviations from Plan

None — plan executed exactly as written. The server's 127.0.0.1:8080 bind (SEC-02) required using `--network host` for verification (not `-p 18080:8080`), but this is expected behavior documented in Phase 1 and aligns with StartOS's routing model.

## Issues Encountered

- The server binds to 127.0.0.1:8080 (SEC-02) which is not reachable from the host when using `-p 18080:8080` (Docker NAT maps to container 0.0.0.0, not 127.0.0.1). Verification used `--network host` mode to confirm the container works correctly. This is correct and expected per the SEC-02 design; StartOS routes to the container's 127.0.0.1 internally.

## Known Stubs

None — all functionality is wired. The image builds and serves the fully functional SPA.

## User Setup Required

None — no external service configuration required. Docker image is available locally as `descriptor-cifrado:dev`.

## Next Phase Readiness

- Plan 03-02 ready: add GitHub Actions `docker.yml` workflow with multi-arch build (linux/amd64 + linux/arm64 via QEMU), GHCR push, ldd-check job, and make-public step
- The Dockerfile is tested and functional for native amd64; buildx will use same Dockerfile for arm64 cross-compilation in CI
- rust:1-slim-bookworm and distroless/cc-debian12 alignment confirmed — no glibc issues expected

## Self-Check: PASSED

- Cargo.toml: FOUND
- Dockerfile: FOUND
- .dockerignore: FOUND
- 03-01-SUMMARY.md: FOUND
- Commit 696367e (profile.release): FOUND
- Commit 6911820 (.dockerignore): FOUND
- Commit 407a772 (Dockerfile): FOUND
- Docker image descriptor-cifrado:dev: EXISTS (10 MB compressed)

---
*Phase: 03-docker-ghcr*
*Completed: 2026-05-06*
