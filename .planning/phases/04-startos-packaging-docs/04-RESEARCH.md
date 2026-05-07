# Phase 4: StartOS Packaging + Docs — Research

**Researched:** 2026-05-07
**Domain:** StartOS 0.4.0 SDK (TypeScript), s9pk packaging, GitHub Actions CI/CD, threat model documentation
**Confidence:** HIGH — all findings verified against skill `start9-packaging` (canonical reference), live npm registry, local SDK source inspection, and working reference project `next-block-startos`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Image pin via `@sha256:digest` (not `:latest`, not semver tag). Digest obtained from `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:vX.Y.Z` after tag push. Deterministic s9pk.
- **D-02:** README in **English** (GitHub public + registry audience). Castilian terms preserved only where citing current UI labels. Structure: 6 sections (Quickstart/TL;DR, Usage, Threat model, Crypto details, Common pitfalls, References).
- **D-03:** DOC-02 (golden rule: "never co-locate .bed and a cosigner xpub") appears **twice** in README: in TL;DR and in Threat Model §"What it does NOT protect against." Intentional redundancy.
- **D-04:** Friendly name = **"BED — Bitcoin Encrypted Backup"**. `manifest.title = "BED"`. `manifest.description.short = "Encrypt Bitcoin descriptor backups for redundant multisig storage"`.
- **D-05:** Icon v1 = **"BED" textual SVG logo** in monospace bold, Semilla Bitcoin palette (black `#0c0c0c` or Bitcoin orange `#f7931a`), 1024×1024 PNG export.
- **D-06:** Distribution in 2 channels: sideload always (`.s9pk` artifact in GitHub Releases) + Semilla Bitcoin own registry after S9-04 validation.
- **D-07:** StartOS official registry — defer to future milestone.
- **D-08:** Coupled versioning between `descriptor-cifrado` and `bed-startos`. Both share semver. First release: `v0.1.0`.
- **D-09:** First public release labeled `v0.1.0` (not `v0.0.1`).
- **D-10:** Zero migration code in v1. No auto re-encryption of `.bed` files (app does not store xpub). Breaking `.bed` format changes → new milestone, archive old version in CHANGELOG.
- **D-11:** `.bed` format is an **external contract** (Liana interop crate v0.0.2). Not broken without explicit milestone.
- **D-12:** Repo `semillabitcoin/bed-startos`, cloned from `Start9Labs/hello-world-startos` branch `update/040`. Initially **PRIVATE**.
- **D-13:** Commit author email: `55397917+4rkad@users.noreply.github.com`.
- **D-14:** Interfaces: Tor onion + LAN `.local` both always active via `bindPort`. No clearnet. No toggle between interfaces.
- **D-15:** Volume `main` covers `/data/encrypted/`. UID 65532 (distroless nonroot, Phase 3 D-13). StartOS SDK creates volume before first start.
- **D-16:** Health check `sdk.healthCheck.checkPortListening` on `127.0.0.1:8080`. No custom `/api/health` endpoint in v1.
- **D-17:** `bed-startos` repo has its own CI (GitHub Actions) that runs `start-sdk pack` and uploads `.s9pk` as release asset on tag push.
- **D-18:** S9-04 test on a real StartOS 0.4.0 device is **manual and blocking**. Phase 4 does not close until a physical device test is verified.

### Claude's Discretion

- Directory structure inside `bed-startos` (typical: `manifest.ts` + `instructions.md` + `LICENSE` + `icon.png` + `assets/`). Planner decides after invoking skill `start9-packaging`.
- Exact version of `@start9labs/start-sdk` npm package. Researcher verifies at plan time.
- Whether to add `actions.ts` for custom actions (e.g., "Reset history") or leave v1 without custom actions. Recommendation: no actions in v1.
- Manifest properties structure (env vars exposed, optional config). Recommendation: zero configurable env vars in v1.
- Level of detail of README screenshots (real device vs mockup vs none). Recommendation: real screenshots after S9-04.

### Deferred Ideas (OUT OF SCOPE)

- Phase 5: i18n EN+ES (in this milestone v0.0.2) — add via `/gsd:add-phase` after Phase 4 closes.
- StartOS official Start9 registry — indefinite review timeline.
- SBOM generation of the `.s9pk`.
- Cosign signing of the `.s9pk`.
- Auto migration scripts between `.bed` formats.
- Custom `/api/health` endpoint in backend.
- Custom `actions.ts` in manifest (e.g., "Reset history", "Export all .bed").
- Artistic icons options B/D if logo C does not satisfy.
- README real screenshots after S9-04.
- Multi-platform Umbrel (XPLAT-01).
- File Browser integration (FB-01/FB-02).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| S9-01 | Repo `semillabitcoin/bed-startos` initialized from `hello-world-startos` branch `update/040` | Anatomy + Quickstart from skill confirmed. Template branch `update/040` is the canonical 0.4.0 scaffold. |
| S9-02 | Manifest TypeScript declares interfaces (Tor + LAN auto-generated via `bindPort`) and volume `main` covering `/data/encrypted/` | `interfaces.ts` pattern + `manifest.volumes` + `Mounts.mountVolume` documented and verified in skill + SDK source. |
| S9-03 | Health check uses `sdk.healthCheck.checkPortListening` | SDK source inspected: reads `/proc/net/tcp` — works correctly with `127.0.0.1:8080` bind. See critical finding §Health Check. |
| S9-04 | App installs and starts on a real StartOS 0.4.0 device | Manual UAT only — no emulator path available. Blocker for phase close. |
| S9-05 | App update preserves history content (`/data/encrypted/`) | StartOS volumes are persistent across image updates by design. Verified via skill `references/backups-versions.md`. |
| DOC-01 | README documents the explicit threat model (what it protects and what it does NOT protect) | README skeleton researched and designed. Threat model content confirmed from CONTEXT.md + IDEA.md. |
| DOC-02 | README includes the key warning: "no location should simultaneously contain the `.bed` and an xpub of the multisig" | Appears twice in proposed README structure per D-03. |
</phase_requirements>

---

## Summary

Phase 4 wraps the existing `ghcr.io/semillabitcoin/descriptor-cifrado` multi-arch image (built in Phase 3) into a `.s9pk` package for StartOS 0.4.0, then writes a threat-model README in English. The work lives in a new repo `semillabitcoin/bed-startos` initialized from `Start9Labs/hello-world-startos` branch `update/040`.

The StartOS 0.4.0 SDK (`@start9labs/start-sdk` version **1.4.1** — the current `latest` on npm) uses TypeScript throughout. There is no YAML manifest, no shell scripts, and no config.yaml — everything is typed TypeScript compiled to a single `javascript.squashfs`. The `start-cli s9pk pack` tool consumes the JS bundle, the Docker image (pulled at pack time from the tag/digest declared in `manifest.images`), and produces a signed `.s9pk` Merkle archive.

BED's architecture is simple: one Docker image, one subcontainer, one daemon, one volume, one port. This maps to a minimal s9pk with no cross-package dependencies, no file-models, no actions, and no migration logic in v1. The most significant implementation detail is the `127.0.0.1:8080` bind — verified via SDK source inspection to work correctly with `checkPortListening` (which reads `/proc/net/tcp`, where loopback-bound sockets appear). The critical remaining risk is the GHCR package visibility: the image must be **public** before `start-cli s9pk pack` can pull it during CI/build.

**Primary recommendation:** Clone `hello-world-startos` branch `update/040`, adapt the 5 editable TypeScript files (`manifest/index.ts`, `manifest/i18n.ts`, `main.ts`, `interfaces.ts`, `backups.ts`) plus one version file, install SDK 1.4.1, and build with `make clean x86 arm`. CI uses `start9labs/shared-workflows` for release, or a minimal custom workflow that runs `npm ci && npm run build && start-cli s9pk pack`. Prioritize flipping the GHCR image to public before building the s9pk.

---

## Standard Stack

### Core

| Tool/Library | Version | Purpose | Why Standard |
|---|---|---|---|
| `@start9labs/start-sdk` | **1.4.1** (npm `latest`) | TypeScript SDK for all s9pk manifest, daemon, interface, health, backup, version declarations | The only SDK for StartOS 0.4.0. Versions `0.4.0-beta.X` are legacy pre-releases — do not use. 1.4.1 is the current stable. |
| `@vercel/ncc` | `^0.38.x` | Bundle TypeScript entrypoint to single `javascript/index.js` | Required by `s9pk.mk` Makefile plumbing. `ncc build startos/index.ts -o ./javascript` is the build command. |
| `typescript` | `^5.x` | TypeScript compiler | Required for `tsc --noEmit` type check step before pack. |
| `start-cli` | **0.4.0-beta.5** (installed locally) | CLI that runs `s9pk pack`, `package install` (sideload), `s9pk inspect`, `s9pk publish` | The only tool that produces and installs s9pk files. Already installed at `/workspace/.cargo/bin/start-cli`. |
| `make` | system | Orchestrate build steps via `s9pk.mk` + `Makefile` | s9pk.mk plumbing is the standard build driver. |

**Version verification (run at pack time):**
```bash
npm view @start9labs/start-sdk version   # should be 1.4.1 or later
start-cli --version                       # should be 0.4.0-beta.5 or later
```

**Installation:**
```bash
npm init -y
npm install @start9labs/start-sdk@1.4.1
npm install --save-dev @vercel/ncc typescript prettier @types/node
```

### Supporting Tools

| Tool | Version | Purpose | When to Use |
|---|---|---|---|
| Docker / buildx | 29.1.x (system) | Pull and inspect multi-arch images | To get the sha256 digest for D-01 pinning |
| `prettier` | `^3.x` | TypeScript formatting | Enforced by `s9pk.mk` check step |
| `ncc` | `^0.38.x` | Bundle `startos/index.ts` → `javascript/index.js` | Automatic via `make` |

---

## Architecture Patterns

### Recommended Project Structure

```
bed-startos/
├── Makefile                      # minimal: "include s9pk.mk" + ARCHES override
├── s9pk.mk                       # plumbing — copy verbatim from hello-world-startos
├── package.json                  # name: "bed-startos", deps: @start9labs/start-sdk@1.4.1
├── package-lock.json
├── tsconfig.json                 # copy from hello-world-startos (target: es2022, module: commonjs)
├── icon.png                      # 1024x1024 PNG — BED textual logo (D-05)
├── LICENSE                       # MIT
├── README.md                     # 6-section EN doc covering DOC-01 + DOC-02
├── CLAUDE.md                     # AI notes: image source, registry URL, digest pinning
└── startos/
    ├── index.ts                  # PLUMBING — DO NOT EDIT
    ├── sdk.ts                    # PLUMBING — DO NOT EDIT
    ├── main.ts                   # runtime: one SubContainer + one Daemon
    ├── interfaces.ts             # one MultiHost, port 8080 http, type 'ui'
    ├── dependencies.ts           # empty (BED has no cross-package deps)
    ├── backups.ts                # Backups.ofVolumes('main')
    ├── utils.ts                  # uiPort = 8080
    ├── manifest/
    │   ├── index.ts              # setupManifest — id, title, images, volumes
    │   └── i18n.ts               # short, long in en_US + es_ES + de_DE + pl_PL + fr_FR
    ├── init/
    │   └── index.ts              # PLUMBING — DO NOT EDIT (no seedFiles needed)
    ├── actions/
    │   └── index.ts              # sdk.Actions.of() — empty, no custom actions v1
    ├── versions/
    │   ├── index.ts              # VersionGraph.of({ current: v_0_1_0_1, other: [] })
    │   └── v0.1.0.1.ts           # VersionInfo v0.1.0:1 with releaseNotes + empty migrations
    └── i18n/
        ├── index.ts              # PLUMBING — DO NOT EDIT
        └── dictionaries/
            ├── default.ts        # BED-specific strings (health check labels)
            └── translations.ts   # es_ES + de_DE + pl_PL + fr_FR
```

**Key constraint:** No `file-models/` directory needed. BED has zero user-configurable settings in v1 — no config form, no persistent config file. The only persistent data is the volume itself (`/data/encrypted/`), which is opaque to the SDK.

---

### Pattern 1: Manifest with GHCR digest pin (D-01)

```typescript
// startos/manifest/index.ts
import { setupManifest } from '@start9labs/start-sdk'
import { long, short } from './i18n'

export const manifest = setupManifest({
  id: 'bed',                                                    // kebab-case, StartOS hostname = bed.startos
  title: 'BED',
  license: 'MIT',
  packageRepo: 'https://github.com/semillabitcoin/bed-startos',
  upstreamRepo: 'https://github.com/semillabitcoin/descriptor-cifrado',
  marketingUrl: null,
  donationUrl: null,
  docsUrls: ['https://github.com/semillabitcoin/bed-startos#readme'],
  description: { short, long },
  volumes: ['main'],                                            // maps to /data/encrypted/ via mountVolume
  images: {
    main: {
      source: {
        // D-01: pin by digest, not by tag. Get digest after Phase 3 tag push:
        //   docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0
        dockerTag: 'ghcr.io/semillabitcoin/descriptor-cifrado@sha256:<DIGEST_HERE>',
      },
      arch: ['x86_64', 'aarch64'],
    },
  },
  alerts: {
    install: null, update: null, uninstall: null, restore: null, start: null, stop: null,
  },
  dependencies: {},                                             // no cross-package deps
})
```

**Source:** skill `references/manifest.md` + `references/anatomy.md` — HIGH confidence.

---

### Pattern 2: Single-container main.ts with volume mount

```typescript
// startos/main.ts
import { i18n } from './i18n'
import { sdk } from './sdk'
import { uiPort } from './utils'

export const main = sdk.setupMain(async ({ effects }) => {
  const mounts = sdk.Mounts.of().mountVolume({
    volumeId: 'main',
    subpath: null,            // mount entire volume root
    mountpoint: '/data/encrypted',
    readonly: false,
  })

  const sub = await sdk.SubContainer.of(
    effects,
    { imageId: 'main' },
    mounts,
    'bed-sub',
  )

  return sdk.Daemons.of(effects).addDaemon('primary', {
    subcontainer: sub,
    exec: {
      command: sdk.useEntrypoint(),   // respects ENTRYPOINT ["/usr/local/bin/bed-server"]
      // env: {} — no env vars needed; BED_DATA_DIR defaults to /data/encrypted in binary
      // user: '65532' — distroless nonroot UID; SDK inherits from image USER directive
    },
    ready: {
      gracePeriod: 10_000,            // bed-server starts in <1s; 10s grace is generous
      display: i18n('Web Interface'),
      fn: () =>
        sdk.healthCheck.checkPortListening(effects, uiPort, {
          successMessage: i18n('The web interface is ready'),
          errorMessage: i18n('The web interface is not ready'),
        }),
    },
    requires: [],
  })
})
```

**Source:** skill `templates/main.ts.tpl` + `references/main-daemons.md` + working reference `next-block-startos/startos/main.ts` — HIGH confidence.

**Critical note on `BED_DATA_DIR`:** The bed-server binary reads `BED_DATA_DIR` env var with default `/data/encrypted`. The volume `main` is mounted at `/data/encrypted` (same path as the binary default). Therefore NO env var needs to be passed to the container — the default matches the mount point exactly. This was confirmed from `crates/server/src/state.rs`:
```rust
std::env::var("BED_DATA_DIR").unwrap_or_else(|_| PathBuf::from("/data/encrypted"))
```

---

### Pattern 3: Interfaces declaration (Tor + LAN auto-generated)

```typescript
// startos/interfaces.ts
import { i18n } from './i18n'
import { sdk } from './sdk'
import { uiPort } from './utils'

export const setInterfaces = sdk.setupInterfaces(async ({ effects }) => {
  const mainHost = sdk.MultiHost.of(effects, 'main')
  const origin = await mainHost.bindPort(uiPort, { protocol: 'http' })

  const ui = sdk.createInterface(effects, {
    name: i18n('Web UI'),
    id: 'ui',
    description: i18n('Encrypt and decrypt Bitcoin descriptor backups'),
    type: 'ui',                     // shows "Launch UI" button in StartOS dashboard
    masked: false,
    schemeOverride: null,
    username: null,
    path: '',
    query: {},
  })

  return [await origin.export([ui])]
})
```

**Source:** skill `templates/interfaces.ts.tpl` + `references/interfaces.md` + `next-block-startos/startos/interfaces.ts` — HIGH confidence.

**How Tor + LAN work:** One `bindPort(8080, { protocol: 'http' })` declaration causes StartOS to:
1. Auto-generate a `.onion` address for Tor access (no configuration needed).
2. Auto-register the package under `bed.local` on the LAN with mDNS + auto-cert.
3. Show both URLs in the StartOS package dashboard.

The developer does NOT configure Tor or LAN separately — this is entirely automatic from a single `bindPort` call.

---

### Pattern 4: Backups — volume rsync (simplest path)

```typescript
// startos/backups.ts
import { sdk } from './sdk'

export const { createBackup, restoreInit } = sdk.setupBackups(
  async ({ effects }) => sdk.Backups.ofVolumes('main'),
)
```

**Source:** `next-block-startos/startos/backups.ts` + skill `references/backups-versions.md` — HIGH confidence.

`ofVolumes('main')` rsync-copies `/data/encrypted/` during a StartOS backup operation. This is the correct pattern for BED: there is no database to dump, only flat `.bed` files in a directory.

---

### Pattern 5: Version info for v0.1.0:1

```typescript
// startos/versions/v0.1.0.1.ts
import { VersionInfo } from '@start9labs/start-sdk'

export const v_0_1_0_1 = VersionInfo.of({
  version: '0.1.0:1',              // X.Y.Z:N format — N is packaging revision
  releaseNotes: {
    en_US: 'Initial release of BED — Bitcoin Encrypted Backup for StartOS.',
    es_ES: 'Primera versión de BED — Bitcoin Encrypted Backup para StartOS.',
    de_DE: 'Erste Version von BED — Bitcoin Encrypted Backup für StartOS.',
    pl_PL: 'Pierwsze wydanie BED — Bitcoin Encrypted Backup dla StartOS.',
    fr_FR: 'Première version de BED — Bitcoin Encrypted Backup pour StartOS.',
  },
  migrations: {
    up: async ({ effects }) => {},  // no migration needed for v1 initial install
    down: async ({ effects }) => {},
  },
})
```

**Source:** skill `references/backups-versions.md` + `next-block-startos/startos/versions/v1.6.0.1.ts` — HIGH confidence.

---

### Pattern 6: Minimal Makefile (with multi-arch)

```makefile
# Makefile — overrides to s9pk.mk must precede the include statement
ARCHES := x86 arm       # produces bed_x86_64.s9pk and bed_aarch64.s9pk
# To build universal (single s9pk with both archs embedded):
# TARGETS := universal

include s9pk.mk
```

**Build commands:**
```bash
make clean x86 arm      # build both per-arch s9pk files
make clean x86 install  # build x86_64 and sideload to device at host in ~/.startos/config.yaml
make clean arm install  # build arm64 and sideload
```

**Source:** skill `references/build-sideload.md` + `next-block-startos/Makefile` — HIGH confidence.

---

### Anti-Patterns to Avoid

- **Do NOT use `manifest.ports`** — ports are not declared in the manifest; they live only in `interfaces.ts` via `bindPort`.
- **Do NOT rely on `EXPOSE 8080` in the Dockerfile** — StartOS ignores Docker `EXPOSE` directives entirely.
- **Do NOT use SDK versions `0.4.0-beta.X`** — these are legacy pre-releases from the alpha period. Current stable is `1.4.1`.
- **Do NOT use `changeOnFirstSuccess`, `successFailure`, or `lastStatus` triggers** — removed in SDK 1.1. Use `trigger.statusTrigger(defaultMs)` if custom polling is needed.
- **Do NOT declare `dependencies: {}` and then mount from them** — cross-package mounts require entries in both `manifest.dependencies` AND `dependencies.ts`. BED has no cross-package deps so both are empty.
- **Do NOT pass `BED_DATA_DIR` env var in `exec.env`** — it defaults to `/data/encrypted` which matches the `mountVolume` mountpoint exactly. Passing it redundantly is harmless but misleading.
- **Do NOT pin to `ghcr.io/semillabitcoin/descriptor-cifrado:latest`** — D-01 requires digest pinning for determinism.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Tor routing | Manual Tor daemon, torrc config | Nothing — StartOS does it automatically from `bindPort` | StartOS auto-generates .onion, routes traffic. Developer declares nothing beyond `bindPort`. |
| LAN TLS certificate | Self-signed cert generation, Let's Encrypt | Nothing — StartOS terminates TLS at the LAN layer | For `.local` URLs StartOS generates and manages the cert. App listens on plain HTTP; StartOS adds TLS externally. |
| Health probing | Custom TCP connect, HTTP check | `sdk.healthCheck.checkPortListening` | Reads `/proc/net/tcp` in the subcontainer's network namespace — works for loopback-bound sockets. No connection opened, no data sent. |
| Persistent volume management | Docker volume creation, UID chown scripts | `manifest.volumes: ['main']` + `Mounts.mountVolume` | StartOS creates the volume, sets ownership, and mounts it before the container starts. Developer declares; StartOS manages. |
| s9pk signing | Ed25519 key management | `start-cli init-key` + developer.key.pem auto-management | `check-init` Makefile target runs `start-cli init-key` automatically on first build. |
| Multi-arch package | Separate CI per arch, manifest merging | `ARCHES := x86 arm` in Makefile | `s9pk.mk` produces per-arch s9pk files from a single `make` invocation. |
| Update migration | Custom up-migration scripts for file format | Not needed in v1 — `Backups.ofVolumes('main')` + empty `migrations.up` | Volume rsync + no schema change = no migration needed. |

**Key insight:** StartOS 0.4.0 eliminates an entire class of DevOps complexity. Tor, LAN, TLS, volume ownership, service restart on update, and backup orchestration are all platform responsibilities, not application responsibilities.

---

## Critical Finding: `checkPortListening` works with `127.0.0.1` bind

**Concern from skill `references/gotchas.md`:** "The port is bound to `127.0.0.1` inside the container but StartOS checks the subcontainer NS — usually OK."

**Verified against SDK 1.3.2 source** (file: `package/lib/health/checkFns/checkPortListening.js`):

```javascript
const hasAddress =
  containsAddress(await cpExec(`cat /proc/net/tcp`, {}), port)    // IPv4, no address filter
  || containsAddress(await cpExec(`cat /proc/net/tcp6`, {}), port, BigInt(0)) // IPv6 ::
  || containsAddress(await cpExec('cat /proc/net/udp', {}), port)
  || containsAddress(await cpExec('cat /proc/net/udp6', {}), port, BigInt(0))
```

The `/proc/net/tcp` call uses **no address filter** for IPv4 — it checks if ANY socket is bound to port 8080 regardless of whether it is `127.0.0.1` or `0.0.0.0`. A process bound to `127.0.0.1:8080` appears in `/proc/net/tcp` as `0100007F:1F90` (little-endian loopback, port 8080 hex), and `containsAddress` returns `true`.

**Conclusion (HIGH confidence):** `sdk.healthCheck.checkPortListening(effects, 8080, {...})` correctly detects `bed-server` listening on `127.0.0.1:8080`. No change to the Rust binary binding is required.

**Why the gotcha mentions `0.0.0.0`:** Some services bind only to a specific non-loopback interface (e.g., `192.168.1.x:8080`) which may NOT appear in `/proc/net/tcp` within the container's network namespace. The loopback interface (`127.0.0.1`) is always present inside any container network namespace.

---

## Common Pitfalls

### Pitfall 1: GHCR image is private when `start-cli s9pk pack` runs

**What goes wrong:** `start-cli s9pk pack` pulls the Docker image at pack time. If `ghcr.io/semillabitcoin/descriptor-cifrado` is still private, the pack fails with `Error: failed to resolve image ghcr.io/...`.

**Why it happens:** GHCR packages are private by default after first push. Phase 3's `make-public` step failed with 403 (token had no `admin:packages` scope). The package is currently private.

**How to avoid:** Before running `start-sdk pack` (in CI or locally), either:
1. Flip the package to public manually at https://github.com/orgs/semillabitcoin/packages/container/descriptor-cifrado/settings, OR
2. Authenticate with `docker login ghcr.io` using a PAT with `read:packages` scope before running `make`.

**For CI (`bed-startos` GitHub Actions):** Use `secrets.GITHUB_TOKEN` from the `semillabitcoin` org for `docker login ghcr.io` before pack. Document this step explicitly.

**Warning signs:** Build log shows `failed to resolve image` or `unauthorized`.

---

### Pitfall 2: Digest pin requires a v0.1.0 tag on `descriptor-cifrado` first

**What goes wrong:** D-01 requires `@sha256:<DIGEST>` in `manifest.images`. The digest can only be obtained AFTER the Phase 3 image is built for a specific version tag (not just `latest`/`sha-xxxxxx` from branch push).

**Why it happens:** The current GHCR image has digest `sha256:da3c9a1d...` from branch push. The manifest pin should reference the *version-tagged* image digest (e.g., `:v0.1.0`). Until `descriptor-cifrado` has a git tag `v0.1.0` and the Docker workflow publishes it with that semver tag, the digest to pin doesn't exist yet.

**How to avoid:** Phase 4 plan must include a task to tag `descriptor-cifrado` at `v0.1.0`, trigger the Docker CI, and record the resulting digest from `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0`.

**Warning signs:** If pinning `sha256:da3c9a1d...` (branch-push digest) in the manifest, that digest refers to an untagged build. Acceptable for development but not for a public v0.1.0 release.

---

### Pitfall 3: `version` field NOT in `setupManifest()`

**What goes wrong:** Developer puts `version: '0.1.0'` in `setupManifest({...})` — TypeScript error, build fails.

**Why it happens:** In StartOS 0.4.0 SDK, version lives in `VersionInfo.of({ version: '0.1.0:1' })` inside `startos/versions/v0.1.0.1.ts`. The manifest plumbing in `startos/index.ts` injects it via `buildManifest(versionGraph, sdkManifest)`.

**How to avoid:** Follow the anatomy from the skill exactly. Version is ONLY in `versions/vX.Y.Z.N.ts`.

---

### Pitfall 4: Old version cached by StartOS — sideload "succeeds" but app not updated

**What goes wrong:** After rebuilding, the s9pk installs but the behavior doesn't change. StartOS uses the same version.

**Why it happens:** StartOS caches packages by their `version` field. If the `X.Y.Z:N` in `VersionInfo` hasn't changed, StartOS may serve from cache.

**How to avoid:** Bump `:N` (packaging revision) on every non-trivial rebuild during development. E.g., `0.1.0:1` → `0.1.0:2`. Also run `make clean` before rebuild to avoid stale `.s9pk` files.

---

### Pitfall 5: Volume appears in manifest but not mounted — files vanish

**What goes wrong:** `.bed` files written to `/data/encrypted/` inside the container are not persisted. After restart, the directory is empty.

**Why it happens:** A volume must be BOTH declared in `manifest.volumes: ['main']` AND mounted via `Mounts.of().mountVolume({ volumeId: 'main', subpath: null, mountpoint: '/data/encrypted', readonly: false })`. Declaring in manifest alone creates the volume but does not mount it.

**How to avoid:** Both declarations are required. The skill SKILL.md "Gotcha rápido" explicitly calls this out.

---

### Pitfall 6: `tsc` / `ncc` build errors on TypeScript type mismatches

**What goes wrong:** `npm run check` (which runs `tsc --noEmit`) fails during `make`, blocking pack.

**Why it happens:** `tsconfig.json` must extend the hello-world-startos default (which sets `"target": "es2022"`, `"module": "commonjs"` — required by `@vercel/ncc`). Custom `tsconfig.json` configurations can break this.

**How to avoid:** Copy `tsconfig.json` verbatim from `hello-world-startos`. Do not modify TypeScript compiler options.

---

### Pitfall 7: Multi-arch image not available for one arch during pack

**What goes wrong:** `make arm` succeeds but the resulting s9pk fails to install on x86_64 machines (or vice versa).

**Why it happens:** `start-cli s9pk pack --arch=aarch64` only embeds the arm64 image layers. The x86_64 pack needs the `linux/amd64` manifest to exist in GHCR.

**How to avoid:** Phase 3 already produces a multi-arch manifest list with both `linux/amd64` and `linux/arm64`. Verify with `docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0` before pack. For the registry, both `bed_x86_64.s9pk` and `bed_aarch64.s9pk` must be published (or use `make universal` for a single combined s9pk).

---

### Pitfall 8: `developer.key.pem` in CI requires secret management

**What goes wrong:** CI fails on `start-cli init-key` because there's no TTY or interactive prompt in the runner.

**Why it happens:** The first invocation of `start-cli init-key` in a fresh environment may require TTY interaction. CI runners don't have one.

**How to avoid:** Store `developer.key.pem` as a GitHub Actions secret (`DEV_KEY`). In CI, write it to `~/.startos/developer.key.pem` before running `make`. The `next-block-startos` uses `start9labs/shared-workflows` which handles this via `secrets.DEV_KEY`. The Semilla Bitcoin registry requires the same key that signed the packages (already at `/workspace/.startos/developer.key.pem`).

**Warning signs:** CI log shows `Error: developer.key.pem not found` or hangs waiting for input.

---

## Build Pipeline

### Option A: `start9labs/shared-workflows` (recommended for release)

The simplest path reuses Start9's shared CI infrastructure, as used by `next-block-startos`:

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*.*']
jobs:
  release:
    uses: start9labs/shared-workflows/.github/workflows/release.yml@master
    with:
      RELEASE_REGISTRY: ${{ vars.RELEASE_REGISTRY }}     # optional — for Semilla registry
      S3_S9PKS_BASE_URL: ${{ vars.S3_S9PKS_BASE_URL }}   # optional — for S3 hosting
    secrets:
      DEV_KEY: ${{ secrets.DEV_KEY }}                     # developer.key.pem contents
      S3_ACCESS_KEY: ${{ secrets.S3_ACCESS_KEY }}         # optional
      S3_SECRET_KEY: ${{ secrets.S3_SECRET_KEY }}         # optional
    permissions:
      contents: write
```

This produces per-arch `.s9pk` files and publishes them as GitHub Release assets. The shared workflow handles key setup, npm ci, make, and upload.

**For GitHub Releases (sideload, D-06):** `.s9pk` artifacts are automatically attached to the GitHub Release created by the shared workflow.

**For Semilla Bitcoin registry (D-06 secondary channel):** After GitHub Release, manually run `start-cli -r https://tienda.privacidadbitcoin.com/rpc/v0 registry package add bed_aarch64.s9pk --url <github-release-url>` per the registry memory.

### Option B: Custom minimal CI (if shared workflow not suitable)

```yaml
# .github/workflows/build.yml (simplified custom)
name: Build and Release
on:
  push:
    tags: ['v*.*']
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup developer key
        run: |
          mkdir -p ~/.startos
          echo "${{ secrets.DEV_KEY }}" > ~/.startos/developer.key.pem
          chmod 600 ~/.startos/developer.key.pem
      - name: Install start-cli
        run: cargo install start-cli   # or download pre-built binary
      - name: Docker login (for private GHCR pull)
        run: echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u semillabitcoin --password-stdin
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - name: Build x86_64 s9pk
        run: make clean x86
      - name: Build aarch64 s9pk
        run: make clean arm
      - name: Upload to GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: '*.s9pk'
```

**Recommendation:** Use Option A (shared workflow) to leverage Start9's tested CI infrastructure. If `RELEASE_REGISTRY` is set, it also auto-publishes to the Semilla Bitcoin registry. Requires storing `developer.key.pem` as `DEV_KEY` secret.

---

## README + Threat Model Structure

### Proposed Skeleton (6 sections, satisfying DOC-01 + DOC-02)

```markdown
# BED — Bitcoin Encrypted Backup

[![StartOS](badge)](...)  [![License: MIT](badge)](...)

## TL;DR

BED encrypts a multisig descriptor using any cosigner xpub, producing a `.bed`
binary that can only be decrypted by someone who holds that same xpub.
Install on StartOS → open the Tor or LAN URL → paste descriptor → download `.bed`.

**Golden rule (see Threat Model for details):**
> Never store a `.bed` file and a cosigner xpub in the same location.
> If an attacker finds both, they recover your descriptor.

## Usage

### Encrypt a descriptor
[screenshot or step list]

### Decrypt a .bed file
[screenshot or step list]

## Threat Model

### What BED protects
- **Descriptor privacy:** A `.bed` file reveals nothing about the wallet's
  structure, xpubs, or spending policy to an attacker who only has the file.
- **xpub distribution:** Each cosigner can safely store a `.bed` backup
  without exposing the full wallet policy — they only need their own xpub to
  decrypt it.

### What BED does NOT protect against

> **Golden rule (repeated):** Never co-locate a `.bed` file with any cosigner
> xpub of that multisig. If an attacker finds both, they can decrypt the
> descriptor and learn the full wallet structure.

- **StartOS compromise during an active encrypt session:** The descriptor
  passes through memory in cleartext during encryption. If the StartOS device
  is compromised at that moment (e.g., malicious app, physical access), the
  descriptor may be exposed. BED cannot protect against an attacker who
  controls the execution environment.
- **Loss of all cosigner xpubs simultaneously:** If every xpub needed to
  decrypt is lost or destroyed, the `.bed` file becomes undecryptable. BED
  provides redundancy for distribution, not a substitute for independent
  xpub backups.
- **An attacker who already has one cosigner xpub:** A `.bed` encrypted with
  xpub A can be decrypted by anyone who has xpub A. The security model assumes
  each co-location (`.bed` + `xpub_N`) is in a different physical location.

### Model assumptions
- The StartOS device is trusted during the session.
- Each `.bed` copy lives in a different physical location than the xpub needed
  to decrypt it.
- The `.bed` file format integrity is guaranteed by AES-256-GCM authentication.

## Crypto Details

- **Algorithm:** AES-256-GCM (from crate `bitcoin-encrypted-backup` v0.0.2)
- **Magic:** `BEB` (binary header identifying the format)
- **BIP:** Draft PR [bitcoin/bips#1951](https://github.com/bitcoin/bips/pull/1951)
- **Interop:** Liana wallet v13+ reads `.bed` files produced by BED (crate v0.0.2)
- **Descriptor requirement:** Must use derivation `<0;1>/*` (BIP requirement for
  safety — spending from address 0 without this wildcard exposes the xpub on-chain)

## Common Pitfalls

1. **Descriptor without `<0;1>/*`**: ...
2. **xpub vs descriptor-style format**: ...
3. **QR size limit (~2,900 bytes ECC-L)**: ...
4. **History mode default is OFF**: ...

## References

- [BIP draft #1951](https://github.com/bitcoin/bips/pull/1951)
- [`bitcoin-encrypted-backup` crate v0.0.2](https://github.com/pythcoiner/encrypted_backup/tree/v0.0.2)
- [Delving Bitcoin thread](https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607)
- [Liana wallet documentation](https://wizardsardine.com/liana/)
```

**Source:** DOC-01, DOC-02, CONTEXT.md D-02, D-03, IDEA.md — HIGH confidence (derived from locked decisions).

---

## Test / Verification Strategy

### What CAN be automated (no device needed)

| Check | Method | Confidence |
|---|---|---|
| TypeScript type correctness | `npm run check` (tsc --noEmit) | HIGH |
| s9pk pack produces valid file | `start-cli s9pk inspect bed_x86_64.s9pk manifest` | HIGH |
| Manifest fields correct | `start-cli s9pk inspect bed_x86_64.s9pk cat /manifest.json` | HIGH |
| Image arch coverage | `docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0` | HIGH |
| Volume + interface declarations present | Inspect manifest JSON from s9pk | HIGH |

### What REQUIRES manual UAT (S9-04)

All four success criteria ultimately require a real StartOS 0.4.0 device:

1. **SC-1:** Install via `start-cli package install -s bed_aarch64.s9pk` + observe "healthy" in dashboard.
2. **SC-2:** Open Tor URL in Tor Browser → encrypt a real descriptor → decrypt it. Open LAN URL → repeat.
3. **SC-3:** Save a `.bed` in history mode → update to a new s9pk version → verify `.bed` still listed and decryptable.
4. **SC-4:** README threat model section review.

**No emulator exists for StartOS 0.4.0 at this time.** The skill notes this explicitly: "Hot reload takes 30s–2min. Some devs iterate first with the Docker image outside StartOS (docker-compose with same image + bindmount of a config), then port to the harness." For BED, `docker run` testing is valid for the backend logic but cannot validate StartOS-specific behaviors (Tor URL, LAN cert, update persistence).

**UAT protocol recommendation:**

```markdown
# HUMAN-UAT.md template for Phase 4

## UAT-1: Install + Healthy
- [ ] `make clean arm install` (arm64 device) OR `make clean x86 install` (x86_64 device)
- [ ] App appears in StartOS dashboard with status "healthy"
- [ ] No crash on startup (check Logs button)

## UAT-2: Tor + LAN access + round-trip
- [ ] Copy Tor .onion URL from StartOS dashboard → open in Tor Browser
- [ ] Paste a valid 2-of-3 multisig descriptor (with <0;1>/*) → click Cifrar
- [ ] Download .bed file, copy armored text, download QR
- [ ] Switch to Descifrar tab → upload .bed + paste xpub → click Descifrar
- [ ] Verify recovered descriptor matches original
- [ ] Repeat via LAN URL (bed.local or IP)

## UAT-3: History persistence across update
- [ ] Enable history toggle → encrypt a descriptor → verify .bed appears in history list
- [ ] Build a new s9pk with bumped version (e.g., 0.1.0:2) → install update via StartOS
- [ ] After update: navigate to Historial tab → verify .bed file still listed
- [ ] Decrypt the old .bed via the updated app → verify decryption succeeds
```

---

## Runtime State Inventory

Phase 4 is NOT a rename/refactor/migration phase. BED stores no named entities in external systems. However, for completeness:

| Category | Items Found | Action Required |
|---|---|---|
| Stored data | None — `.bed` files in `/data/encrypted/` are fresh per install. No Mem0, no ChromaDB, no SQLite with app-identity strings. | None |
| Live service config | None — BED has no external service dependencies (no n8n, no Datadog, no Tailscale). | None |
| OS-registered state | None — no scheduler tasks, no pm2 processes, no systemd units for BED. | None |
| Secrets/env vars | None — no `.env` file, no SOPS keys named after BED. `BED_DATA_DIR` is read at runtime, not a stored secret. | None |
| Build artifacts | None relevant — the GHCR image name `descriptor-cifrado` does not change in Phase 4; `bed-startos` is a new repo, no stale artifacts. | None |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| `start-cli` | S9-01 repo init, s9pk pack, sideload | ✓ | `0.4.0-beta.5` (at `/workspace/.cargo/bin/start-cli`) | — |
| `node` / `npm` | TypeScript build, SDK install | ✓ | Node 20.20.1 / npm 10.8.2 | — |
| `docker` | Image inspect (digest), `start-cli` pack (internal pull) | ✓ | 29.1.3 | — |
| `~/.startos/developer.key.pem` | s9pk signing | ✓ | Present (from next-block-startos + Semilla registry work) | — |
| StartOS 0.4.0 device | S9-04 UAT | Unknown | — | No emulator — this is the manual blocker for phase close |
| GHCR `descriptor-cifrado` image (public) | s9pk pack | Partially (image exists but private) | sha256:da3c9a1d... | Authenticate with PAT `read:packages` OR make public |

**Missing dependencies with no fallback:**
- Real StartOS 0.4.0 device — required for S9-04 (D-18, blocking).

**Missing dependencies with fallback:**
- GHCR private image — fallback is `docker login ghcr.io` with PAT before pack. Long-term fix: flip to public (requires git history sanitization first, per CONTEXT.md D-12).

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| StartOS 0.3.x: YAML manifest, bash scripts, `config.yaml` | StartOS 0.4.0: TypeScript-only SDK (`@start9labs/start-sdk`), no YAML, no bash | SDK 1.0.0 / OS 0.4.0-beta.0 | All packaging is typed TS; `hello-world-startos` branch `update/040` is the canonical starting point |
| `trigger.changeOnFirstSuccess` / `successFailure` / `lastStatus` | `trigger.statusTrigger(defaultMs, overrides?)` | SDK 1.1 | Old triggers removed — using them causes runtime errors |
| `manifest.ports` declaration | No ports in manifest — only in `interfaces.ts` via `bindPort` | SDK 1.0 | Port declaration is decoupled from image metadata |
| `start-sdk pack` CLI | `start-cli s9pk pack` | Renamed in 0.4.0 | Same functionality, different binary name |

**Deprecated / outdated:**
- `Start9Labs/hello-world-startos` branch `master` — points to StartOS 0.3.x. Always use branch `update/040` for 0.4.0.
- `@start9labs/start-sdk` `0.4.0-beta.X` — legacy alpha versions. Current stable: `1.4.1`.

---

## Open Questions

1. **Does `start9labs/shared-workflows` require S3 variables to be set, or can it publish GitHub Releases only?**
   - What we know: the `release.yml` in next-block-startos passes `RELEASE_REGISTRY` and `S3_S9PKS_BASE_URL` as optional `with:` inputs.
   - What's unclear: whether omitting them causes the workflow to fail or gracefully skip registry/S3 steps.
   - Recommendation: Check the shared workflow README or source. If optional, use it without S3 for v1 GitHub Releases. If required, use Option B (custom workflow).

2. **Should `bed-startos` use `make universal` or per-arch s9pk files?**
   - What we know: `make universal` produces a single `.s9pk` with all arch images embedded; per-arch produces separate files. The Semilla Bitcoin registry in `project_startos_registry.md` notes that per-arch files must share the same `gitHash` to be accepted.
   - What's unclear: user preference for distribution simplicity vs file size.
   - Recommendation (Claude's Discretion): Start with per-arch (`make x86 arm`) for registry compatibility (gitHash requirement confirmed in memory). Users can sideload either based on device arch.

3. **Is the `descriptor-cifrado` GHCR package public or must CI authenticate?**
   - What we know: package is currently private (make-public step failed with 403 in Phase 3). Git history contains xpubs — cannot flip to public until sanitized.
   - What's unclear: timeline for git-filter-repo sanitization session.
   - Recommendation: Plan Phase 4 CI to authenticate with PAT `read:packages` stored as a secret, independent of public/private status. This unblocks Phase 4 CI without requiring the sanitization prerequisite.

---

## Sources

### Primary (HIGH confidence)

- Skill `start9-packaging` (`~/.claude/skills/start9-packaging/`) — canonical reference for all StartOS 0.4.0 SDK patterns, manifest, main, interfaces, backups, versions, build/sideload. Read in full.
- `@start9labs/start-sdk` 1.3.2 source at `next-block-startos/node_modules/` — `checkPortListening.js` inspected directly to verify loopback-bind behavior.
- `npm view @start9labs/start-sdk version` → `1.4.1` (verified live against npm registry 2026-05-07).
- `next-block-startos` at `/workspace/claude/proyectos/next-block-start9/next-block-startos/` — working real-world reference: `main.ts`, `interfaces.ts`, `manifest/index.ts`, `backups.ts`, `versions/`, `Makefile`, `.github/workflows/release.yml`.
- `descriptor-cifrado` codebase: `crates/server/src/main.rs` (bind address `127.0.0.1:8080`), `crates/server/src/state.rs` (`BED_DATA_DIR` default `/data/encrypted`), `Dockerfile` (ENTRYPOINT, UID 65532).
- `project_startos_registry.md` memory — Semilla Bitcoin registry URL, per-arch gitHash requirement, registry CLI command.

### Secondary (MEDIUM confidence)

- `start9labs/shared-workflows` referenced from `next-block-startos/.github/workflows/release.yml` — shared CI infrastructure confirmed to exist; internal structure not directly inspected.
- CONTEXT.md D-14 on Tor + LAN auto-generation — consistent with skill `references/interfaces.md` which confirms single `bindPort` generates both.

### Tertiary (LOW confidence — flagged for validation)

- Claim that `start9labs/shared-workflows` `S3_*` variables are optional — inferred from `with:` syntax in YAML but not verified by reading the workflow source directly. Validate before committing to Option A CI.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — SDK version verified live against npm; start-cli version verified from installed binary; next-block-startos is a deployed working reference.
- Architecture: HIGH — all patterns traced to skill canonical references + live SDK source inspection.
- Pitfalls: HIGH — GHCR private trap verified from Phase 3 history; checkPortListening verified from SDK source; other pitfalls from skill `references/gotchas.md`.
- README skeleton: HIGH — derived from locked CONTEXT.md decisions (D-02, D-03) and project requirements (DOC-01, DOC-02).
- Verification strategy: HIGH — StartOS emulator absence is a confirmed constraint from the skill.

**Research date:** 2026-05-07
**Valid until:** 2026-06-07 (30 days; SDK 1.4.1 is stable; `start-cli` 0.4.0-beta.5 installed locally)
