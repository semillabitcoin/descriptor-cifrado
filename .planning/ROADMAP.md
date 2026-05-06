# Roadmap: BED Start9 App

## Overview

Four coarse phases take the project from a verified crypto core to a live StartOS s9pk that any holder can sideload. Phase 1 establishes the cryptographic foundation and HTTP API with all security invariants locked in from day one. Phase 2 adds the Svelte SPA and opt-in history mode, completing the full user-visible product. Phase 3 packages everything into a distroless Docker image published to GHCR. Phase 4 wraps the image in a StartOS s9pk, tests on real hardware, and ships documentation.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Crypto Core + HTTP API** - Establish round-trip encryption/decryption, BIP invariants, memory safety, and stable axum endpoints (completed 2026-05-06)
- [ ] **Phase 2: SPA Frontend + History** - Svelte UI for encrypt/decrypt flows, opt-in history mode with list and delete
- [ ] **Phase 3: Docker + GHCR** - Multi-stage distroless image, multi-arch build, public GHCR push with CI audits
- [ ] **Phase 4: StartOS Packaging + Docs** - s9pk manifest, health check, real-device test, threat model documentation

## Phase Details

### Phase 1: Crypto Core + HTTP API
**Goal**: Developers can curl the local axum server to encrypt a descriptor and get back binary, armored, and QR outputs — and decrypt with an xpub — with all security invariants (zeroize, no-unwrap, no-log, BIP wildcard check, exact crate pin) already in place
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, ENC-01, ENC-02, ENC-03, ENC-04, ENC-05, DEC-01, DEC-02, DEC-03, DEC-04, DEC-05, SEC-01, SEC-02, SEC-03, CI-01, CI-02
**Success Criteria** (what must be TRUE):
  1. A curl POST to `/api/encrypt` with a valid `<0;1>/*` descriptor returns three outputs: a downloadable `.bed` binary (base64-encoded), an armored PGP-style block, and a QR PNG; and a POST with a bare xpub descriptor returns HTTP 422 with a descriptive error
  2. A curl POST to `/api/decrypt` with the `.bed` from step 1 and a valid cosigner xpub returns the original descriptor verbatim; using a wrong xpub returns HTTP 422
  3. A CI test sends a known descriptor, captures tracing output, and asserts the descriptor string does not appear in any log line
  4. `cargo audit` and `cargo deny` pass in CI with no vulnerabilities or prohibited licenses; the CI round-trip test (encrypt then decrypt) is green
  5. The server binds on `127.0.0.1:8080`; `ldd` on the binary shows no `libssl` or `native-tls` symbols
**Plans**: 6 plans
Plans:
- [x] 01-01-workspace-skeleton-PLAN.md — Cargo workspace skeleton, crates/{core,server} con pin exacto, deny.toml
- [x] 01-02-github-actions-ci-PLAN.md — GitHub Actions CI con 5 jobs (fmt, clippy, test, audit, deny)
- [x] 01-03-core-validate-zeroize-PLAN.md — CoreError, ZeroizingDescriptor, validate::require_multipath_0_1
- [x] 01-04-core-armored-qr-encrypt-PLAN.md — Armored encoder/decoder, encrypt_descriptor, decrypt_payload, QR generator, round-trip
- [x] 01-05-server-axum-handlers-PLAN.md — AppError, IntoResponse, handlers POST /api/encrypt y /api/decrypt
- [x] 01-06-integration-tests-PLAN.md — Integration tests round_trip + validation + no_leak (SEC-01)
**UI hint**: no

### Phase 2: SPA Frontend + History
**Goal**: A user opening the app in a browser sees two tabs — Cifrar and Descifrar — can encrypt a descriptor and download/copy/scan the three outputs, and can optionally enable history mode to list and delete saved `.bed` files; the descriptor in clear never touches disk
**Depends on**: Phase 1
**Requirements**: UI-01, UI-02, UI-03, HIST-01, HIST-02, HIST-03, HIST-04, HIST-05, HIST-06
**Success Criteria** (what must be TRUE):
  1. User pastes a descriptor in the Cifrar tab, clicks "Cifrar", and receives three outputs: a `.bed` download button, an armored block with a "Copiar" button, and a QR PNG download button — all within the same page, no CDN requests
  2. User switches to Descifrar, uploads a `.bed` or pastes armored text, enters an xpub, clicks "Descifrar", and sees the recovered descriptor with a "Copiar" button; the descriptor never appears in server logs or saved files
  3. User enables the history toggle, performs an encrypt, sees the entry appear in the history list, and can delete it; after a container restart the history files persist but no plaintext descriptor can be found in `/data/encrypted/` by grep
  4. The SPA is served entirely from the binary (no filesystem mount needed); no external font, script, or stylesheet URLs appear in the HTML
**Plans**: 6 plans
Plans:
- [x] 02-01-frontend-scaffold-PLAN.md — package.json + Vite + tokens.css + fonts self-hosted (Inter + JetBrains Mono)
- [x] 02-02-backend-history-endpoints-PLAN.md — 4 endpoints history (POST/GET/GET-id/DELETE) + tests (round-trip + no-leak HIST-03)
- [x] 02-03-shell-and-shared-components-PLAN.md — stores Svelte 5 + lib (api/clipboard) + 8 componentes compartidos + App.svelte shell
- [x] 02-04-tab-cifrar-PLAN.md — TabCifrar.svelte + CifrarOutputs (3 outputs simultáneos) + history fire-and-warn
- [x] 02-05-tab-descifrar-PLAN.md — TabDescifrar.svelte + drop-zone + DescifrarOutputs + AnimatedQrModal (BBQR lazy)
- [ ] 02-06-tab-historial-and-rust-embed-PLAN.md — TabHistorial + 2 modales + rust-embed wiring + test embedded_spa
**UI hint**: yes

### Phase 3: Docker + GHCR
**Goal**: The service ships as a multi-arch Docker image on GHCR that a StartOS instance can pull without authentication
**Depends on**: Phase 2
**Requirements**: PKG-01, PKG-02, PKG-03, PKG-04
**Success Criteria** (what must be TRUE):
  1. `docker pull ghcr.io/semillabitcoin/bed-app:latest` succeeds without any GitHub credentials from a fresh machine
  2. `docker inspect` reports both `linux/amd64` and `linux/arm64` manifest entries for the image
  3. CI `ldd` check on the release binary exits 0 only if no `libssl` or non-distroless lib appears; the final image is at or below 25 MB compressed
**Plans**: TBD
**UI hint**: no

### Phase 4: StartOS Packaging + Docs
**Goal**: Any holder with a StartOS 0.4.0 device can sideload the `.s9pk`, open the Tor or LAN URL, encrypt a real descriptor, and recover it — and the README documents exactly what the app protects and what it does not
**Depends on**: Phase 3
**Requirements**: S9-01, S9-02, S9-03, S9-04, S9-05, DOC-01, DOC-02
**Success Criteria** (what must be TRUE):
  1. `start-sdk pack` produces a valid `bed-startos.s9pk` that installs on a real StartOS 0.4.0 device via `start-cli package install`; the app appears as healthy in the StartOS dashboard
  2. After install, the Tor onion URL and the LAN `.local` URL both load the SPA in a browser; the full encrypt-then-decrypt round-trip works via both interfaces
  3. After a simulated update (new image version pushed, app updated via StartOS), all `.bed` files saved before the update are still present and decryptable
  4. The README contains an explicit threat model section stating what the app protects (descriptor privacy, xpub distribution) and what it does NOT protect (compromise of StartOS during active encrypt session, loss of all xpubs simultaneously)
**Plans**: TBD
**needs_research**: true
**UI hint**: no

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Crypto Core + HTTP API | 6/6 | Complete   | 2026-05-06 |
| 2. SPA Frontend + History | 0/6 | Not started | - |
| 3. Docker + GHCR | 0/? | Not started | - |
| 4. StartOS Packaging + Docs | 0/? | Not started | - |
