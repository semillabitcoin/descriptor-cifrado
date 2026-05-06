---
status: partial
phase: 03-docker-ghcr
source: [03-VERIFICATION.md]
started: 2026-05-06T23:30:00Z
updated: 2026-05-06T23:30:00Z
---

## Current Test

[awaiting human testing — requires `git push origin main` to trigger GHA workflow]

## Tests

### 1. GHA workflow execution
expected: Push to main triggers `.github/workflows/docker.yml`. Run shows 3 green jobs: `build-and-push`, `ldd-check`, `make-public` (or `make-public` with `continue-on-error` warning if PAT lacks org-package permission).
result: [pending]

### 2. Multi-arch manifest published to GHCR
expected: `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:latest` lists both `linux/amd64` and `linux/arm64` manifest entries.
result: [pending]

### 3. Unauthenticated pull from GHCR
expected: `docker logout ghcr.io && docker pull ghcr.io/semillabitcoin/descriptor-cifrado:latest` succeeds without credentials (PKG-04). If denied, toggle visibility at `https://github.com/orgs/semillabitcoin/packages/container/descriptor-cifrado/settings`.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
