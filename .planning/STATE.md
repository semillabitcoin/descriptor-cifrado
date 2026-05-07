---
gsd_state_version: 1.0
milestone: v0.0.2
milestone_name: milestone
status: verifying
stopped_at: Completed 04-04-PLAN.md (bed-startos manifest+main+interfaces+versions+icon+LICENSE+README+CI); approved by user, advancing to Plan 05
last_updated: "2026-05-07T17:15:48.115Z"
last_activity: 2026-05-07
progress:
  total_phases: 4
  completed_phases: 3
  total_plans: 19
  completed_plans: 18
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-05)

**Core value:** Un holder StartOS puede pegar un descriptor multisig y obtener un `.bed` cifrado (binario, armored o QR) sin instalar ni compilar nada, y luego recuperarlo pegando `.bed` + cualquier xpub cosigner — todo local, sobre Tor, sin telemetría.
**Current focus:** Phase 04 — startos-packaging-docs

## Current Position

Phase: 04 (startos-packaging-docs) — EXECUTING
Plan: 5 of 5
Status: Phase complete — ready for verification
Last activity: 2026-05-07

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: — min
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
| Phase 01-crypto-core-http-api P01 | 12 | 3 tasks | 10 files |
| Phase 01-crypto-core-http-api P02 | 4 | 1 tasks | 1 files |
| Phase 01-crypto-core-http-api P03 | 8 | 2 tasks | 6 files |
| Phase 01-crypto-core-http-api P04 | 18 | 2 tasks | 9 files |
| Phase 01-crypto-core-http-api P05 | 5 | 2 tasks | 7 files |
| Phase 01-crypto-core-http-api P06 | 27 | 2 tasks | 6 files |
| Phase 02-spa-frontend-history P01 | 3 | 3 tasks | 13 files |
| Phase 02-spa-frontend-history P02 | 6 | 3 tasks | 10 files |
| Phase 02-spa-frontend-history P03 | 4 | 3 tasks | 14 files |
| Phase 02-spa-frontend-history P04 | 5 | 2 tasks | 4 files |
| Phase 02-spa-frontend-history P05-tab-descifrar | 3 | 2 tasks | 7 files |
| Phase 02-spa-frontend-history P06-tab-historial-and-rust-embed | 11 | 2 tasks | 11 files |
| Phase 03-docker-ghcr P01 | 16 | 4 tasks | 3 files |
| Phase 03-docker-ghcr P02 | 3 | 2 tasks | 1 files |
| Phase 04-startos-packaging-docs P02 | 3 | 1 tasks | 2 files |
| Phase 04-startos-packaging-docs P03 | 14 | 4 tasks | 33 files |
| Phase 04-startos-packaging-docs P01 | 45 | 4 tasks | 2 files |
| Phase 04-startos-packaging-docs P04 | 45 | 6 tasks | 12 files |
| Phase 04-startos-packaging-docs P04 | 45 | 7 tasks | 12 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Roadmap: 4 coarse phases derived from SUMMARY.md build order (steps 1+2 merged, steps 3+4 merged, step 5 standalone, steps 6+7 merged)
- Phase 4 (StartOS s9pk) flagged `needs_research: true` — SDK 0.4.0 still in beta; invoke `start9-packaging` skill before planning
- Phase 2 (SPA) flagged `UI hint: yes` — Svelte 5 + Vite 6 frontend work
- [Phase 01-crypto-core-http-api]: wildcards=allow in deny.toml bans: cargo-deny 0.19.4 flags git rev= deps and path deps as wildcards; security maintained via Cargo.lock
- [Phase 01-crypto-core-http-api]: MITNFA added to license allowlist: required by hex_lit 0.1.1 (transitive via bitcoin -> miniscript)
- [Phase 01-crypto-core-http-api]: bitcoin-encrypted-backup license exception: no Cargo.toml license field in git dep; upstream has MIT in repo LICENSE file
- [Phase 01-crypto-core-http-api]: Actions versions pinned to canonical stable: checkout@v4, rust-toolchain@stable, rust-cache@v2, audit-check@v2, cargo-deny-action@v2
- [Phase 01-crypto-core-http-api]: CI: No Rust version matrix — stable only. No release/publish job — deferred to Phase 3
- [Phase 01-crypto-core-http-api]: paths() accessor usado directamente en miniscript 12.3.6 (no fallback Display)
- [Phase 01-crypto-core-http-api]: #[allow(clippy::panic)] requerido en test helpers — workspace lint panic=deny se aplica a test targets
- [Phase 01-crypto-core-http-api]: Normalize h/apostrophe in round-trip assertions: miniscript re-serializes 48h as 48'; both BIP-380 valid; comparison normalizes both sides
- [Phase 01-crypto-core-http-api]: QrTooLarge message uses 'Usa' (Castilian) not 'Usá' (Argentine) per feedback_castellano_no_argentino.md
- [Phase 01-crypto-core-http-api]: base64 added to bed-server Cargo.toml — required for bed_b64/qr_png_b64 JSON fields in encrypt handler
- [Phase 01-crypto-core-http-api]: Strip BIP-380 checksum before round-trip comparison: miniscript re-computes checksum after canonical normalization
- [Phase 01-crypto-core-http-api]: #![allow(clippy::panic)] required in integration test files: workspace lint panic=warn + -D warnings in CI
- [Phase 02-spa-frontend-history]: Vite 8.0.10 / Svelte 5.55.5 / vite-plugin-svelte 7.1.1 resolved by ^X.0.0 ranges
- [Phase 02-spa-frontend-history]: JetBrains Mono variable woff2 path is fonts/webfonts/, not fonts/variable/ (upstream renamed)
- [Phase 02-spa-frontend-history]: Directory scan persistence (no redb): Phase 2 v1 solo necesita list/delete con .bed cifrado en BED_DATA_DIR; redb diferido hasta queries más ricas
- [Phase 02-spa-frontend-history]: Filename sortable <YYYYMMDDTHHMMSSZ>-<8hex>.bed: lex sort = chronological sort; validate_history_id es único guard anti path traversal
- [Phase 02-spa-frontend-history]: HIST-03 enforced by design: POST /api/history acepta solo bed_b64 cifrado; descriptor cleartext nunca cruza el módulo de history (test no_leak verifica con fixture multisig real)
- [Phase 02-spa-frontend-history]: Svelte a11y warnings 'no_noninteractive_element_to_interactive_role' suprimidos con svelte-ignore en <nav role=tablist> y <section role=tabpanel>: pattern WAI-ARIA estándar, falso positivo del linter
- [Phase 02-spa-frontend-history]: Tab Historial NO se renderiza en DOM cuando historyEnabled=false (no solo hidden) — must_haves enforce 'NO solo hidden'
- [Phase 02-spa-frontend-history]: Bundle JS+CSS gzipped post-shell = 18,439 bytes (36% del budget 50 KB; 32 KB libres para planes 04/05/06)
- [Phase 02-spa-frontend-history]: TabCifrar dual copy feedback: toast 3s + button label 'Copiado ✓' reverting after 1500ms (D-34); QR rendered inline via data:image/png;base64 from backend (no client-side qrcode lib needed for cifrar)
- [Phase 02-spa-frontend-history]: Fire-and-warn POST /api/history when historyEnabled toggle ON: encryption result preserved on persistence failure (toast 'Cifrado OK, pero no se guardó en historial')
- [Phase 02-spa-frontend-history]: Bundle JS+CSS gzipped post-TabCifrar = 23,546 bytes (46% del budget 50 KB; ~28 KB libres para 02-05/02-06)
- [Phase 02-spa-frontend-history]: TabDescifrar: lazy import bbqr@1.2.0 + qrcode@1.5.4 → chunks dinámicos 57.8 KB gzipped separados (bundle inicial 27,636 B / 54% del budget 50 KB)
- [Phase 02-spa-frontend-history]: STATIC_QR_THRESHOLD 500 chars: descriptors multisig 2-de-3 caben en QR estático, multisig 5+ requieren BBQR animado (frame rotation 600ms)
- [Phase 02-spa-frontend-history]: validateXpub regex ^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$ rechaza descriptor-style con [fingerprint/path] prefix — backend /api/decrypt espera xpub bare; smoke test confirma round-trip cifrar→descifrar
- [Phase 02-spa-frontend-history]: Plan 02-06: rust-embed feature 'axum' confirmed (8.11.0); mime_guess 2.0.5 added for Content-Type resolution on .woff2/.css; Cache-Control split (assets/* immutable 1y, index.html no-cache)
- [Phase 02-spa-frontend-history]: Plan 02-06: Static routes registered AFTER /api/* in axum 0.8 router; static handler uses uri.path().trim_start_matches('/') so rust-embed receives 'assets/index-...' path
- [Phase 02-spa-frontend-history]: Plan 02-06: Bundle JS+CSS gzipped post-TabHistorial = 30,045 bytes (60% del budget 50 KB); binary release 5.8 MB (within STACK target 5-10 MB); Phase 2 closed 9/9 requirements
- [Phase 03-docker-ghcr]: rust:1-slim-bookworm (not rust:1-slim) for Debian 12 glibc alignment with distroless/cc-debian12; rust:1-slim → Trixie since Mar 2026
- [Phase 03-docker-ghcr]: Dockerfile COPY layer order: workspace config → crate manifests → frontend/dist → source code for maximal cache reuse
- [Phase 03-docker-ghcr]: Action versions for docker.yml confirmed May 2026: setup-qemu@v4, setup-buildx@v4, login@v4, metadata@v6, build-push@v7 (CONTEXT.md had pre-March 2026 v3/v5/v6)
- [Phase 03-docker-ghcr]: make-public uses /orgs/semillabitcoin/packages/container/descriptor-cifrado (not /user/packages/... which is incorrect for org-scoped packages); continue-on-error: true with fallback manual toggle URL documented
- [Phase 03-docker-ghcr]: flavor: latest=false + conditional enable= in metadata-action@v6 prevents latest tag on non-main branches
- [Phase 04-startos-packaging-docs]: README.md golden rule: 'never co-locate' embedded mid-sentence in blockquotes (not sentence-opener) to satisfy case-sensitive grep acceptance test
- [Phase 04-startos-packaging-docs]: LICENSE created with MIT + Semilla Bitcoin copyright — file was absent from repo; README links to it
- [Phase 04-startos-packaging-docs]: Used hello-world-startos master branch (update/040 merged into master by 2026-05-07)
- [Phase 04-startos-packaging-docs]: marketingUrl must be string in SDK 1.4.1 (not null) — set to GitHub repo URL
- [Phase 04-startos-packaging-docs]: bed-startos repo path: /home/anon/bed-startos (not /workspace/bed-startos — /workspace does not exist)
- [Phase 04-startos-packaging-docs]: GHCR tag es 0.1.0 (sin prefijo v) porque metadata-action@v6 con type=semver,pattern={{version}} lo normaliza; Plan 04 debe usar digest no tag string
- [Phase 04-startos-packaging-docs]: Manifest list digest sha256:41684bce9dd4ec6270965f8df2caafecab031b573ab9dd52c38937c057fa67b5 capturado en 01-DIGEST.txt para pin exacto en manifest.ts
- [Phase 04-startos-packaging-docs]: Digest-pinned GHCR image in manifest (sha256:41684bce9dd4ec6270965f8df2caafecab031b573ab9dd52c38937c057fa67b5) — deterministic s9pk (D-01)
- [Phase 04-startos-packaging-docs]: Single bindPort(8080, http) in interfaces.ts generates both Tor onion + LAN .local automatically (D-14)
- [Phase 04-startos-packaging-docs]: release.yml Option B (custom CI with explicit GHCR docker/login-action@v3) chosen over shared-workflows for auth control (Pitfall 1 mitigation)
- [Phase 04-startos-packaging-docs]: Digest-pinned GHCR image in manifest (sha256:41684bce9dd4ec6270965f8df2caafecab031b573ab9dd52c38937c057fa67b5) — deterministic s9pk (D-01)
- [Phase 04-startos-packaging-docs]: Single bindPort(8080, http) in interfaces.ts generates both Tor onion + LAN .local automatically (D-14)
- [Phase 04-startos-packaging-docs]: release.yml Option B (custom CI with explicit GHCR docker/login-action@v3) chosen over shared-workflows for auth control (Pitfall 1 mitigation)

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 4 depends on StartOS 0.4.0 beta SDK stability; invoke `start9-packaging` skill at plan time for verified current details
- Exact armored header string and QR size limit (2,900 B ECC-L) must be verified against reference impl in Phase 1, not assumed

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260506-rx9 | fix xpub interop con Liana (normalizar descriptor-style → bare en frontend) | 2026-05-06 | eedaa33 | | [260506-rx9-260506-rgb-fix-xpub-interop-con-liana](./quick/260506-rx9-260506-rgb-fix-xpub-interop-con-liana/) |
| 260506-sr7 | migrar bitcoin-encrypted-backup a v0.0.2 (interop Liana, magic BEB, AES-256-GCM) | 2026-05-06 | 8d9d2f0 | Verified | [260506-sr7-migrar-bitcoin-encrypted-backup-a-v0-0-2](./quick/260506-sr7-migrar-bitcoin-encrypted-backup-a-v0-0-2/) |
| 260507-v6e | relajar validador multipath <0;1>/* → <a;b>/* + dropzone TabCifrar + botones Limpiar | 2026-05-07 | 53af96d | Verified | [260507-v6e-relax-multipath-validator-file-upload-ta](./quick/260507-v6e-relax-multipath-validator-file-upload-ta/) |
| 260507-ww3 | UX Nunchuk: detectar single-chain /N/* y autoconvertir par-impar (regla Sparrow) con modal | 2026-05-07 | c057d72 | Local-only | [260507-ww3-ux-nunchuk-detectar-single-chain-n-y-aut](./quick/260507-ww3-ux-nunchuk-detectar-single-chain-n-y-aut/) |
| 260508-0ao | polish UI: validar issue B (history toast = config no bug), reducir help redundante TabCifrar (issue C), generalizar copy "del multisig" → "del descriptor" en CifrarOutputs+TabDescifrar (issue D) | 2026-05-08 | b2384c6 | Local-only | [260508-0ao-polish-history-textos](./quick/260508-0ao-polish-history-textos/) |

## Session Continuity

Last session: 2026-05-08T00:20:00Z
Stopped at: Completed quick task 260508-0ao (issues B/C/D — Issue A queda para Quick 2 con UR crypto-output)
Resume file: None
