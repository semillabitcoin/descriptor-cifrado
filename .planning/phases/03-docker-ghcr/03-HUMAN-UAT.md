---
status: complete
phase: 03-docker-ghcr
source: [03-VERIFICATION.md]
started: 2026-05-06T23:30:00Z
updated: 2026-05-07T01:30:00Z
---

## Current Test

[done — sesión 10 (2026-05-07) cerró validación remota]

## Tests

### 1. GHA workflow execution
expected: Push to main triggers `.github/workflows/docker.yml`. Run shows 3 green jobs: `build-and-push`, `ldd-check`, `make-public` (or `make-public` with `continue-on-error` warning if PAT lacks org-package permission).
result: ✅ PASS — Docker run `25465312951` verde 3/3 (build-and-push multi-arch, ldd-check sin libssl/libcrypto/native-tls, make-public con `continue-on-error: true` por token sin scope `admin:packages` — workflow siguió). CI run `25465312954` verde 5/5 (fmt+clippy+test+audit+deny) tras fixes a48acc9 (cargo fmt) y 5c84e62 (build frontend antes de cargo + bump timeout 40→90min).

### 2. Multi-arch manifest published to GHCR
expected: `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:latest` lists both `linux/amd64` and `linux/arm64` manifest entries.
result: ✅ PASS (vía build logs) — manifest list `sha256:da3c9a1dce91b741d67b47237a20de66fef9a0f54bb77fbacac9ef0446a199e5` con `linux/amd64,linux/arm64` confirmado en logs del job build-and-push. Caveat: `imagetools inspect` desde local falla 403 porque el paquete sigue privado y el token no tiene `read:packages`; validación remota basada en logs es suficiente.

### 3. Unauthenticated pull from GHCR
expected: `docker logout ghcr.io && docker pull ghcr.io/semillabitcoin/descriptor-cifrado:latest` succeeds without credentials (PKG-04). If denied, toggle visibility at `https://github.com/orgs/semillabitcoin/packages/container/descriptor-cifrado/settings`.
result: ⊘ BLOCKED — paquete GHCR sigue privado (step `make-public` exit 0 vía `continue-on-error` pero PATCH API 403 por token sin `admin:packages`). Bloqueo upstream: el repo también es PRIVATE por contaminación del historial git con xpubs reales / paths personales. Plan: sesión separada con `git-filter-repo` para sanitizar historial → flipar repo y paquete a público desde UI → validar PKG-04 entonces. Diferido fuera de Phase 3.

## Summary

total: 3
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 1

## Gaps

- **PKG-04 (pull sin credenciales)** queda bloqueado por privacidad del repo (historial sucio). Validación deferida hasta sesión de sanitización con `git-filter-repo` + flip a público. No bloquea Phase 4 (StartOS packaging puede consumir la imagen privada con `docker login` en el host de build).
