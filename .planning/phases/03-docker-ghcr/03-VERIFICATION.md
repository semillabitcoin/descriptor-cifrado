---
phase: 03-docker-ghcr
verified: 2026-05-06T23:30:00Z
status: human_needed
score: 7/10 must-haves verified locally; 3 require remote GHA run
re_verification: false
human_verification:
  - test: "Push to main triggers docker.yml; confirm build-and-push job succeeds"
    expected: "GHA run shows 3 green jobs: build-and-push, ldd-check, make-public (or make-public with continue-on-error warning)"
    why_human: "Cannot run GitHub Actions locally; requires real push + GITHUB_TOKEN + GHCR registry"
  - test: "docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:latest"
    expected: "Output explicitly lists both linux/amd64 and linux/arm64 manifest entries"
    why_human: "Image not yet pushed to GHCR; multi-arch manifest only exists after CI run"
  - test: "docker logout ghcr.io && docker pull ghcr.io/semillabitcoin/descriptor-cifrado:latest (from a machine without docker login)"
    expected: "Pull succeeds without credentials (PKG-04)"
    why_human: "Requires GHCR to have the public image; depends on prior push + package visibility toggle"
---

# Phase 3: Docker + GHCR Verification Report

**Phase Goal:** The service ships as a multi-arch Docker image on GHCR that a StartOS instance can pull without authentication
**Verified:** 2026-05-06T23:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | `[profile.release]` in Cargo.toml with all 5 settings produces a stripped ≤10 MB binary | VERIFIED | Cargo.toml lines 37-43: strip=true, lto="thin", codegen-units=1, opt-level=3, panic="abort"; SUMMARY reports 3.7 MB binary |
| 2 | `docker build` produces a 3-stage distroless image ≤25 MB compressed | VERIFIED | Local image exists (27.4 MB uncompressed / ~10 MB compressed per SUMMARY); Dockerfile has all 3 stages; docker inspect confirms sha256:47f3cc72 |
| 3 | Image uses `gcr.io/distroless/cc-debian12:nonroot` runtime (UID 65532, no shell) | VERIFIED | Dockerfile line 44: `FROM gcr.io/distroless/cc-debian12:nonroot AS runtime`; no HEALTHCHECK or USER directives present |
| 4 | rust-embed path wiring: `COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist` resolves correctly for `#[folder = "../../frontend/dist/"]` | VERIFIED | Dockerfile line 33 matches the COPY path; SUMMARY confirms SPA HTML served with "BED — Bitcoin Encrypted Backup" title |
| 5 | `docker.yml` workflow builds multi-arch (linux/amd64,linux/arm64) and only pushes on non-PR events | VERIFIED | docker.yml line 57: `platforms: linux/amd64,linux/arm64`; line 58: `push: ${{ github.event_name != 'pull_request' }}`; YAML valid |
| 6 | `ldd-check` job exits 1 if binary links libssl/libcrypto/native-tls/libpq/libmysqlclient/libsqlite3 | VERIFIED | docker.yml lines 78-81: grep -E pattern + exit 1; ldd exit code alone is not relied upon (correct) |
| 7 | `make-public` step targets `/orgs/semillabitcoin/packages/container/descriptor-cifrado` with `continue-on-error: true` | VERIFIED | docker.yml lines 87-88, 95-96: correct org endpoint, continue-on-error present |
| 8 | Push to main → CI publishes `ghcr.io/semillabitcoin/descriptor-cifrado:latest` and `:sha-<7chars>` | NEEDS HUMAN | Workflow YAML is correct locally; actual GHCR push requires a live CI run |
| 9 | Multi-arch manifest lists both linux/amd64 and linux/arm64 | NEEDS HUMAN | Cannot inspect GHCR manifest without a prior push; CI has not run yet on main |
| 10 | `docker pull ghcr.io/semillabitcoin/descriptor-cifrado:latest` succeeds without credentials | NEEDS HUMAN | Requires GHCR push + public package (PKG-04 primary: image.source label inheritance; secondary: make-public step) |

**Score:** 7/10 truths locally verified; 3 require a live GHA run against GHCR

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | `[profile.release]` with 5 settings | VERIFIED | All 5 keys present: strip, lto, codegen-units, opt-level, panic |
| `Dockerfile` | 3-stage: frontend-builder (Node alpine) → rust-builder (rust:slim-bookworm) → runtime (distroless nonroot) | VERIFIED | All 3 stages present; distroless base correct; ENTRYPOINT set |
| `.dockerignore` | 11 exclusions per D-03 | VERIFIED | 11 lines including target/, node_modules/, frontend/dist/, .planning/, .git/, .github/, DS_Store, .env*, *.md, !LICENSE |
| `.github/workflows/docker.yml` | 3-job workflow: build-and-push, ldd-check, make-public | VERIFIED | All 3 jobs present; YAML valid; 14 plan acceptance criteria satisfied |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `docker.yml job build-and-push` | `Dockerfile` in root | `context: .` | VERIFIED | Line 56: `context: .` |
| `docker.yml job build-and-push` | `ghcr.io/semillabitcoin/descriptor-cifrado` | `docker/metadata-action@v6 images:` | VERIFIED | Line 39: correct registry+org+repo |
| `docker.yml job ldd-check` | `target/release/bed-server` | `cargo build --release --locked + ldd + grep -E + exit 1` | VERIFIED | Full pattern present; exit 1 on match (not relying on ldd exit code) |
| `docker.yml job make-public` | `GHCR API /orgs/semillabitcoin/packages/container/descriptor-cifrado` | `gh api -X PATCH visibility=public` | VERIFIED | Correct org endpoint (not /user/packages/...) |
| `Dockerfile rust-builder stage` | `frontend/dist/` from stage 1 | `COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist` | VERIFIED | Line 33; resolves rust-embed path correctly from /app/crates/server/ |
| `Dockerfile runtime stage` | `bed-server` binary from rust-builder | `COPY --from=rust-builder /app/target/release/bed-server /usr/local/bin/bed-server` | VERIFIED | Line 45; correct source and destination paths |

### Data-Flow Trace (Level 4)

Not applicable for this phase — all artifacts are infrastructure (Dockerfile, GHA workflow, Cargo configuration), not dynamic data-rendering components.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Local image exists and is correct size | `docker image inspect descriptor-cifrado:dev --format '{{.Id}} {{.Size}}'` | sha256:47f3cc72, 27439498 bytes (27.4 MB uncompressed) | PASS |
| YAML parses without errors | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/docker.yml'))"` | exit 0, "YAML OK" | PASS |
| Cargo.toml profile.release has all 5 settings | grep checks | All 5 keys match | PASS |
| Dockerfile has correct base images | grep checks | distroless/cc-debian12:nonroot, rust:slim-bookworm, node:alpine | PASS |
| ldd-check exits 1 on libssl (not relying on ldd exit code) | Code review | `grep -E '...' | then exit 1` pattern confirmed | PASS |
| Real GHCR push + multi-arch pull | Requires live GHA run | Not run yet | SKIP — needs human |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| PKG-01 | 03-01-PLAN.md | Dockerfile multi-stage rust:slim → distroless/cc-debian12 ≤25 MB | SATISFIED | Local image 10 MB compressed (27.4 MB uncompressed); Dockerfile structure verified |
| PKG-02 | 03-02-PLAN.md | Multi-arch (amd64 + arm64) published in GHCR under semillabitcoin org | PARTIALLY SATISFIED | Workflow configured correctly; actual push pending human verification |
| PKG-03 | 03-02-PLAN.md | CI `ldd` check fails if libssl or non-distroless lib appears | SATISFIED locally | ldd-check job with correct grep + exit 1 pattern present in docker.yml |
| PKG-04 | 03-02-PLAN.md | GHCR image marked public immediately after first push | PARTIALLY SATISFIED | Triple mechanism implemented: image.source label + make-public step + fallback docs; actual public pull not verified |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | — | No stubs, placeholders, or empty implementations found | — | — |

**Notes:**
- No `TODO`/`FIXME`/`PLACEHOLDER` comments in any Phase 3 artifact
- No empty handlers or return stubs
- The local Docker image `descriptor-cifrado:dev` exists and is substantive (27.4 MB, confirmed functional in SUMMARY)
- `ldd-check` correctly uses `grep -E ... | then exit 1` (not relying on `ldd` exit code which is always 0) — this is the correct pattern per Plan 03-02 Pitfall 6

### Human Verification Required

#### 1. GitHub Actions Docker Workflow Run

**Test:** Push the Phase 3 commits to `main` on `semillabitcoin/descriptor-cifrado` (or open a PR first for build-only validation, then merge to main)
**Expected:** All 3 jobs complete — `build-and-push` green (multi-arch image published), `ldd-check` green (no libssl found), `make-public` green or warning (continue-on-error)
**Why human:** Cannot run GitHub Actions locally; requires GITHUB_TOKEN + real GHCR registry access

Pre-flight before push:
```bash
# Confirm repo is public (required for PKG-04 auto-inheritance)
gh repo view semillabitcoin/descriptor-cifrado --json visibility,name | jq
# Confirm git email is noreply
git log -1 --format='%ae'
# Expected: 55397917+4rkad@users.noreply.github.com
```

#### 2. Multi-Arch Manifest Inspection (PKG-02 / Success Criterion 2)

**Test:** After first push to main: `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:latest`
**Expected:** Output lists explicit entries for `linux/amd64` AND `linux/arm64`
**Why human:** Image does not yet exist on GHCR; requires the GHA run from check 1

#### 3. Unauthenticated Pull (PKG-04 / Success Criterion 1)

**Test:**
```bash
docker logout ghcr.io 2>/dev/null || true
docker pull ghcr.io/semillabitcoin/descriptor-cifrado:latest
```
**Expected:** Pull succeeds without credentials
**Why human:** Requires GHCR push + package visibility (either auto-inherited from public repo or triggered by make-public step); if "denied: requested resource is not accessible", toggle manually at `https://github.com/orgs/semillabitcoin/packages/container/descriptor-cifrado/settings`

### Gaps Summary

No gaps. All locally-verifiable artifacts are correct and complete:

- `Cargo.toml` has all 5 `[profile.release]` settings exactly as specified
- `Dockerfile` implements the full 3-stage build with correct base images, COPY layers in cache-optimal order, and rust-embed path alignment
- `.dockerignore` has all 11 exclusions
- `.github/workflows/docker.yml` passes all 14 acceptance criteria from Plan 03-02: 3 jobs, correct action versions (qemu@v4, buildx@v4, login@v4, metadata@v6, build-push@v7), multi-arch platforms, login PR guard, conditional latest tag, sha- prefix, 3 semver patterns, ldd-check with exit 1, make-public with org endpoint and continue-on-error, OCI image.source label for PKG-04 primary mechanism
- Local Docker image `descriptor-cifrado:dev` exists (sha256:47f3cc72, ~10 MB compressed) and was verified functional in Plan 03-01 SUMMARY

The 3 remaining items (GHA run, multi-arch manifest inspection, unauthenticated pull) are exclusively remote-validation items that require a real push to main. These are not gaps in the implementation — they are the expected post-push validation steps documented in Plan 03-02 Task 2 checkpoint.

---

_Verified: 2026-05-06T23:30:00Z_
_Verifier: Claude (gsd-verifier)_
