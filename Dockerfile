# syntax=docker/dockerfile:1.7
# BED Start9 — Bitcoin Encrypted Backup
# Multi-stage build: frontend (Node) -> rust-builder (rust:slim-bookworm) -> distroless runtime

ARG NODE_VERSION=20
ARG RUST_VERSION=1

# ── Stage 1: Frontend (Vite + Svelte 5) ─────────────────────────────────
FROM node:${NODE_VERSION}-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build
# Output: /app/frontend/dist/

# ── Stage 2: Rust workspace build ───────────────────────────────────────
# Pin a -bookworm (Debian 12) para alineación con distroless/cc-debian12.
# rust:1-slim apunta a Debian 13 (Trixie) desde Mar 2026 — incompatible glibc.
FROM rust:${RUST_VERSION}-slim-bookworm AS rust-builder
WORKDIR /app

# 1. Workspace config (cambia poco, maximiza cache)
COPY Cargo.toml Cargo.lock rust-toolchain.toml deny.toml ./

# 2. Crate manifests (necesarios para que cargo resuelva el workspace)
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

# 3. Frontend dist desde stage 1 — rust-embed lo lee en compile-time
#    crates/server/src/assets.rs: #[folder = "../../frontend/dist/"]
#    relativo a /app/crates/server/ → resuelve a /app/frontend/dist/
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# 4. Source code (cambia más frecuente, último para cache reuse)
COPY crates/ crates/

# --locked: respeta Cargo.lock exacto (CORE-01 pin v0.0.2)
RUN cargo build --release --locked --bin bed-server

# ── Stage 3: Distroless runtime ─────────────────────────────────────────
# cc-debian12 incluye glibc + libgcc_s + libstdc++ mínimo. Sin shell, sin
# package manager. Corre como UID 0 dentro del container — necesario para
# escribir en el volume `main` montado por StartOS en /data/encrypted/
# (SDK 1.4.1 no expone ownership en mountVolume; el sandbox real lo provee
# StartOS via cgroups + netns, no el UID interno).
FROM gcr.io/distroless/cc-debian12 AS runtime
COPY --from=rust-builder /app/target/release/bed-server /usr/local/bin/bed-server

# Documentación; el bind real lo hace el binario en 0.0.0.0:8080 — el proxy StartOS vive fuera del netns del contenedor (SEC-02)
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/bed-server"]
