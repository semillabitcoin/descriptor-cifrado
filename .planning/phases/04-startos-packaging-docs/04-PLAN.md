---
id: 04-04
phase: 04-startos-packaging-docs
plan: 04
title: "Author bed-startos manifest, main, interfaces, backups, versions, icon, LICENSE, README, CI workflow"
type: execute
wave: 2
depends_on: ["04-01", "04-03"]
files_modified:
  - "/workspace/bed-startos/startos/manifest/index.ts"
  - "/workspace/bed-startos/startos/manifest/i18n.ts"
  - "/workspace/bed-startos/startos/main.ts"
  - "/workspace/bed-startos/startos/interfaces.ts"
  - "/workspace/bed-startos/startos/backups.ts"
  - "/workspace/bed-startos/startos/dependencies.ts"
  - "/workspace/bed-startos/startos/utils.ts"
  - "/workspace/bed-startos/startos/actions/index.ts"
  - "/workspace/bed-startos/startos/versions/index.ts"
  - "/workspace/bed-startos/startos/versions/v0.1.0.1.ts"
  - "/workspace/bed-startos/icon.png"
  - "/workspace/bed-startos/icon.svg"
  - "/workspace/bed-startos/LICENSE"
  - "/workspace/bed-startos/README.md"
  - "/workspace/bed-startos/instructions.md"
  - "/workspace/bed-startos/.github/workflows/release.yml"
autonomous: false
model: opus
requirements: [S9-02, S9-03, S9-05, DOC-01, DOC-02]
gap_closure: false

must_haves:
  truths:
    - "manifest.ts pins ghcr.io/semillabitcoin/descriptor-cifrado@sha256:<digest from 01-DIGEST.txt> (D-01)"
    - "manifest.ts declares title 'BED', id 'bed', volumes ['main'], arch ['x86_64','aarch64'] (D-04, D-15)"
    - "main.ts mounts volume 'main' at /data/encrypted/ via Mounts.mountVolume (D-15)"
    - "main.ts wires sdk.healthCheck.checkPortListening on port 8080 (S9-03, D-16)"
    - "interfaces.ts calls bindPort(8080, { protocol: 'http' }) — single call generates both Tor onion AND LAN .local (D-14)"
    - "backups.ts uses Backups.ofVolumes('main') so /data/encrypted/ is preserved across StartOS backup/restore (S9-05)"
    - "versions/v0.1.0.1.ts declares VersionInfo with version '0.1.0:1' and empty migrations (D-09, D-10)"
    - "icon.png exists at 1024×1024 with BED textual logo (D-05)"
    - "LICENSE exists (MIT)"
    - "bed-startos/README.md covers install instructions, threat model summary, golden rule (DOC-01, DOC-02), and links to descriptor-cifrado/README.md for the deep model"
    - "Golden rule appears verbatim in bed-startos/README.md"
    - ".github/workflows/release.yml triggers on tag push v*.*, runs npm ci + make clean x86 arm, uploads .s9pk artifacts to GitHub Release"
    - "tsc --noEmit passes with zero errors after all task files are written"
  artifacts:
    - path: "/workspace/bed-startos/startos/manifest/index.ts"
      provides: "setupManifest declaration with digest pin, identity, volumes, multi-arch images"
      contains: "@sha256:"
    - path: "/workspace/bed-startos/startos/main.ts"
      provides: "Daemon definition with checkPortListening health check and volume mount"
      contains: "checkPortListening"
    - path: "/workspace/bed-startos/startos/interfaces.ts"
      provides: "bindPort(8080) for Tor + LAN auto-generation"
      contains: "bindPort"
    - path: "/workspace/bed-startos/icon.png"
      provides: "1024×1024 BED textual logo"
    - path: "/workspace/bed-startos/.github/workflows/release.yml"
      provides: "CI that produces .s9pk on tag push"
      contains: "make"
  key_links:
    - from: "manifest.images.main.source.dockerTag"
      to: "ghcr.io/semillabitcoin/descriptor-cifrado@sha256:DIGEST"
      via: "Verbatim copy of contents of 01-DIGEST.txt from Plan 01"
      pattern: "sha256:[a-f0-9]{64}"
    - from: "main.ts Mounts.mountVolume mountpoint"
      to: "/data/encrypted (matches BED_DATA_DIR default in bed-server binary)"
      via: "String literal '/data/encrypted'"
      pattern: "/data/encrypted"
    - from: "release.yml on.push.tags"
      to: "make clean x86 arm + softprops/action-gh-release"
      via: "GitHub Actions workflow"
      pattern: "tags:"
---

<objective>
Author the complete app-specific TypeScript surface of bed-startos plus the supporting assets (icon, LICENSE, README, instructions, CI workflow). After this plan, `make clean x86 arm` in the bed-startos repo produces valid `.s9pk` files that pass `start-cli s9pk inspect`. Plan 05 then handles local pack verification, real-device UAT, and registry publication.

This is the architectural core of Phase 4 — the manifest must correctly declare digest pinning (D-01), interfaces (D-14), volume (D-15), and health check (D-16); main.ts must wire the daemon and health probe correctly; the CI workflow must produce verifiable artifacts on tag push (D-17). Hence `model: opus`.

Output: bed-startos repo at /workspace/bed-startos has working manifest + main + interfaces + backups + versions + icon + LICENSE + README + instructions + release workflow, all committed and pushed to origin/main.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/04-startos-packaging-docs/04-CONTEXT.md
@.planning/phases/04-startos-packaging-docs/04-RESEARCH.md
@.planning/phases/04-startos-packaging-docs/01-DIGEST.txt
@$HOME/.claude/skills/start9-packaging/SKILL.md
</context>

<tasks>

<task id="04-01-write-manifest-and-i18n" type="auto">
  <name>Task 1: Write manifest/index.ts (digest pin, identity, volumes) and manifest/i18n.ts (descriptions)</name>
  <read_first>
    - $HOME/.claude/skills/start9-packaging/SKILL.md plus references/manifest.md and references/anatomy.md (canonical reference for setupManifest API surface)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md Pattern 1 (skeleton TypeScript)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt (THE digest to pin — verbatim, single line `sha256:HEX64`)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-01, D-04, D-15
    - /workspace/bed-startos/startos/manifest/index.ts (current placeholder — overwrite, do not append)
    - /workspace/bed-startos/node_modules/@start9labs/start-sdk/package/lib/manifest/index.d.ts (verify the setupManifest function signature against the installed SDK; if RESEARCH.md drifts from SDK 1.4.x typings, the SDK source is canonical)
  </read_first>
  <files>
    - /workspace/bed-startos/startos/manifest/index.ts
    - /workspace/bed-startos/startos/manifest/i18n.ts
  </files>
  <action>
    Step 1. Read digest into a shell variable: `DIGEST=$(cat /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt)` and assert it matches `^sha256:[a-f0-9]{64}$` before proceeding.

    Step 2. Overwrite /workspace/bed-startos/startos/manifest/index.ts with EXACTLY the following content, substituting the literal digest at write time (the file is committed with the literal digest, no env-var indirection):

    ```typescript
    import { setupManifest } from '@start9labs/start-sdk'
    import { long, short } from './i18n'

    export const manifest = setupManifest({
      id: 'bed',
      title: 'BED',
      license: 'MIT',
      packageRepo: 'https://github.com/semillabitcoin/bed-startos',
      upstreamRepo: 'https://github.com/semillabitcoin/descriptor-cifrado',
      marketingUrl: null,
      donationUrl: null,
      docsUrls: ['https://github.com/semillabitcoin/bed-startos#readme'],
      description: { short, long },
      volumes: ['main'],
      images: {
        main: {
          source: {
            dockerTag: 'ghcr.io/semillabitcoin/descriptor-cifrado@DIGEST_PLACEHOLDER',
          },
          arch: ['x86_64', 'aarch64'],
        },
      },
      alerts: { install: null, update: null, uninstall: null, restore: null, start: null, stop: null },
      dependencies: {},
    })
    ```

    Replace `DIGEST_PLACEHOLDER` with the literal value of `$DIGEST` (which already includes the `sha256:` prefix). The result line MUST read exactly:
    `        dockerTag: 'ghcr.io/semillabitcoin/descriptor-cifrado@sha256:HEX64HERE',`

    Step 3. Overwrite /workspace/bed-startos/startos/manifest/i18n.ts with `short` and `long` description objects keyed by the SAME locale set the upstream `startos/i18n/dictionaries/translations.ts` defines (typically en_US, es_ES, de_DE, pl_PL, fr_FR per RESEARCH.md). Verify the locale set first with `grep -E '(en_US|es_ES|de_DE|pl_PL|fr_FR)' /workspace/bed-startos/startos/i18n/dictionaries/translations.ts` — if upstream uses a different set, MIRROR the upstream set exactly (do not invent locales the i18n plumbing does not handle).

    Required exact strings:
    - `short.en_US`: "Encrypt Bitcoin descriptor backups for redundant multisig storage" (D-04)
    - `long.en_US`: a paragraph describing what BED does, mentioning AES-256-GCM, BIP `bitcoin/bips#1951`, Liana interop, AND containing the literal phrase "NEVER store a .bed and a cosigner xpub in the same location" (DOC-02 visibility from the StartOS dashboard).
    - `short.es_ES`: castellano peninsular per feedback_castellano_no_argentino.md ("Cifra backups de descriptors Bitcoin para almacenamiento multisig redundante").
    - `long.es_ES`: equivalent paragraph in castellano with phrase "NUNCA guardes un .bed y una xpub cosigner en el mismo lugar".
    - Other locales: shorter translations are acceptable; minimum content = product summary + golden-rule warning.

    Step 4. Run `cd /workspace/bed-startos && npx tsc --noEmit` and capture errors. Errors confined to `import` references resolving symbols not yet written by Tasks 2-5 are EXPECTED — leave them; the final type check is in Task 7. Errors specific to setupManifest field names (e.g. "Property 'X' does not exist") indicate SDK API drift — resolve by reading `node_modules/@start9labs/start-sdk/package/lib/manifest/setupManifest.d.ts` and adjusting field names. Do NOT silently rename fields without verifying the SDK source.
  </action>
  <verify>
    <automated>grep -qE "dockerTag: 'ghcr.io/semillabitcoin/descriptor-cifrado@sha256:[a-f0-9]{64}'" /workspace/bed-startos/startos/manifest/index.ts &amp;&amp; grep -q "title: 'BED'" /workspace/bed-startos/startos/manifest/index.ts &amp;&amp; grep -q "id: 'bed'" /workspace/bed-startos/startos/manifest/index.ts &amp;&amp; grep -q "volumes: \['main'\]" /workspace/bed-startos/startos/manifest/index.ts &amp;&amp; grep -qE "arch: \['x86_64', ?'aarch64'\]" /workspace/bed-startos/startos/manifest/index.ts</automated>
  </verify>
  <acceptance_criteria>
    - manifest/index.ts contains the literal substring `'ghcr.io/semillabitcoin/descriptor-cifrado@sha256:` followed by 64 lowercase hex chars and a single quote.
    - manifest/index.ts contains `title: 'BED'`, `id: 'bed'`, `volumes: ['main']`, `arch: ['x86_64', 'aarch64']`.
    - manifest/index.ts contains `packageRepo: 'https://github.com/semillabitcoin/bed-startos'`.
    - manifest/index.ts contains `upstreamRepo: 'https://github.com/semillabitcoin/descriptor-cifrado'`.
    - manifest/index.ts contains `dependencies: {}`.
    - manifest/i18n.ts exports `short` and `long` objects with at least the en_US and es_ES keys.
    - `grep 'NEVER store a .bed' /workspace/bed-startos/startos/manifest/i18n.ts` returns at least one match (golden rule visible in StartOS dashboard description).
    - `grep 'NUNCA guardes' /workspace/bed-startos/startos/manifest/i18n.ts` returns at least one match (castellano golden rule).
    - The locale set in i18n.ts matches the locale set in startos/i18n/dictionaries/translations.ts.
  </acceptance_criteria>
  <done>manifest/index.ts and manifest/i18n.ts are valid TypeScript, declare the digest pin, identity, volumes, multi-arch images, and per-locale descriptions including the golden rule.</done>
</task>

<task id="04-02-write-utils-interfaces-main" type="auto">
  <name>Task 2: Write utils.ts, interfaces.ts (Tor+LAN bindPort), main.ts (daemon + health check + volume mount)</name>
  <read_first>
    - $HOME/.claude/skills/start9-packaging/SKILL.md (sections on bindPort, MultiHost, createInterface, Mounts, Daemons, healthCheck)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md Pattern 2 (main.ts skeleton) and Pattern 3 (interfaces.ts skeleton)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-14 (Tor+LAN both active, no toggle), D-15 (volume main → /data/encrypted), D-16 (checkPortListening on 127.0.0.1:8080)
    - /workspace/descriptor-cifrado/crates/server/src/main.rs (confirms bind on 127.0.0.1:8080)
    - /workspace/descriptor-cifrado/crates/server/src/state.rs (confirms BED_DATA_DIR default '/data/encrypted')
    - /workspace/descriptor-cifrado/Dockerfile (confirms ENTRYPOINT `/usr/local/bin/bed-server`, USER 65532)
    - /workspace/bed-startos/startos/sdk.ts (PLUMBING — read to know what `sdk` re-exports; do NOT edit)
  </read_first>
  <files>
    - /workspace/bed-startos/startos/utils.ts
    - /workspace/bed-startos/startos/interfaces.ts
    - /workspace/bed-startos/startos/main.ts
  </files>
  <action>
    Step 1. Write /workspace/bed-startos/startos/utils.ts with:
    ```typescript
    export const uiPort = 8080
    ```

    Step 2. Write /workspace/bed-startos/startos/interfaces.ts EXACTLY (RESEARCH.md Pattern 3, verified against next-block-startos):
    ```typescript
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
        type: 'ui',
        masked: false,
        schemeOverride: null,
        username: null,
        path: '',
        query: {},
      })

      return [await origin.export([ui])]
    })
    ```
    The single `bindPort(uiPort, { protocol: 'http' })` call generates BOTH the Tor onion address AND the LAN `.local` URL automatically (RESEARCH.md Pattern 3 §"How Tor + LAN work"). NO clearnet bind. NO secondary bindPort calls.

    Step 3. Write /workspace/bed-startos/startos/main.ts EXACTLY (RESEARCH.md Pattern 2 verified against next-block-startos):
    ```typescript
    import { i18n } from './i18n'
    import { sdk } from './sdk'
    import { uiPort } from './utils'

    export const main = sdk.setupMain(async ({ effects }) => {
      const mounts = sdk.Mounts.of().mountVolume({
        volumeId: 'main',
        subpath: null,
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
          command: sdk.useEntrypoint(),
        },
        ready: {
          gracePeriod: 10_000,
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
    Critical points:
    - `mountpoint: '/data/encrypted'` MATCHES the bed-server binary's `BED_DATA_DIR` default (verified from crates/server/src/state.rs). NO env var needed in `exec.env` (RESEARCH.md Anti-Pattern: do not pass redundant BED_DATA_DIR).
    - `command: sdk.useEntrypoint()` respects the Dockerfile ENTRYPOINT. NO custom `command` array — the binary is already entrypointed.
    - `subpath: null` mounts the entire volume root at `/data/encrypted`.
    - `gracePeriod: 10_000` (10s) — bed-server starts in <1s; 10s is generous (RESEARCH.md Pattern 2).
    - `checkPortListening(effects, 8080, {...})` reads /proc/net/tcp inside subcontainer NS — VERIFIED to detect 127.0.0.1 binds (RESEARCH.md §"Critical Finding: checkPortListening works with 127.0.0.1 bind"). Do NOT change the bed-server bind.

    Step 4. Run `cd /workspace/bed-startos && npx tsc --noEmit`. After Tasks 1+2 complete, the only remaining TS errors should be from Tasks 3-5 (backups, versions, dictionaries) which are written next.
  </action>
  <verify>
    <automated>grep -q "bindPort(uiPort, { protocol: 'http' })" /workspace/bed-startos/startos/interfaces.ts &amp;&amp; grep -q "checkPortListening(effects, uiPort" /workspace/bed-startos/startos/main.ts &amp;&amp; grep -q "mountpoint: '/data/encrypted'" /workspace/bed-startos/startos/main.ts &amp;&amp; grep -q "volumeId: 'main'" /workspace/bed-startos/startos/main.ts &amp;&amp; grep -q "useEntrypoint()" /workspace/bed-startos/startos/main.ts &amp;&amp; grep -q "uiPort = 8080" /workspace/bed-startos/startos/utils.ts</automated>
  </verify>
  <acceptance_criteria>
    - utils.ts exports `uiPort = 8080`.
    - interfaces.ts contains `bindPort(uiPort, { protocol: 'http' })` (single call).
    - interfaces.ts contains `type: 'ui'`.
    - main.ts contains `volumeId: 'main'` AND `mountpoint: '/data/encrypted'` AND `subpath: null` AND `readonly: false`.
    - main.ts contains `checkPortListening(effects, uiPort` (S9-03, D-16).
    - main.ts contains `command: sdk.useEntrypoint()`.
    - main.ts does NOT contain any `BED_DATA_DIR` string (anti-pattern: redundant env var).
    - main.ts does NOT contain `0.0.0.0` (the Rust binary binds 127.0.0.1, no override).
  </acceptance_criteria>
  <done>utils.ts, interfaces.ts, main.ts wire the daemon, volume mount, health check, and Tor+LAN interfaces correctly per RESEARCH.md verified patterns.</done>
</task>

<task id="04-03-write-backups-deps-actions-versions" type="auto">
  <name>Task 3: Write backups.ts, dependencies.ts, actions/index.ts, versions/index.ts, versions/v0.1.0.1.ts</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md Pattern 4 (backups.ts) and Pattern 5 (versions/v0.1.0.1.ts)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-09 (v0.1.0 first release), D-10 (zero migration code), S9-05 (volume preserved across update)
    - /workspace/bed-startos/node_modules/@start9labs/start-sdk/package/lib/version/VersionInfo.d.ts (verify VersionInfo.of signature against installed SDK)
  </read_first>
  <files>
    - /workspace/bed-startos/startos/backups.ts
    - /workspace/bed-startos/startos/dependencies.ts
    - /workspace/bed-startos/startos/actions/index.ts
    - /workspace/bed-startos/startos/versions/index.ts
    - /workspace/bed-startos/startos/versions/v0.1.0.1.ts
  </files>
  <action>
    Step 1. Write /workspace/bed-startos/startos/backups.ts EXACTLY (RESEARCH.md Pattern 4):
    ```typescript
    import { sdk } from './sdk'

    export const { createBackup, restoreInit } = sdk.setupBackups(
      async ({ effects }) => sdk.Backups.ofVolumes('main'),
    )
    ```
    `Backups.ofVolumes('main')` rsyncs /data/encrypted/ during StartOS backup/restore (S9-05). Plus, StartOS volumes are persistent across image updates by design (RESEARCH.md confirms — VolumeID 'main' survives version bumps).

    Step 2. Write /workspace/bed-startos/startos/dependencies.ts:
    ```typescript
    import { sdk } from './sdk'
    export const dependencies = sdk.setupDependencies(async ({ effects }) => ({}))
    ```
    BED has zero cross-package dependencies (no Bitcoin Core, no Electrs).

    Step 3. Write /workspace/bed-startos/startos/actions/index.ts:
    ```typescript
    import { sdk } from '../sdk'
    export const actions = sdk.Actions.of()
    ```
    Empty actions per CONTEXT.md "Recommendation: no actions in v1". No Reset History, no Export. Existing BED UI handles all user-driven operations.

    Step 4. Write /workspace/bed-startos/startos/versions/v0.1.0.1.ts EXACTLY (RESEARCH.md Pattern 5):
    ```typescript
    import { VersionInfo } from '@start9labs/start-sdk'

    export const v_0_1_0_1 = VersionInfo.of({
      version: '0.1.0:1',
      releaseNotes: {
        en_US: 'Initial release of BED — Bitcoin Encrypted Backup for StartOS.',
        es_ES: 'Primera versión de BED — Bitcoin Encrypted Backup para StartOS.',
        de_DE: 'Erste Version von BED — Bitcoin Encrypted Backup für StartOS.',
        pl_PL: 'Pierwsze wydanie BED — Bitcoin Encrypted Backup dla StartOS.',
        fr_FR: 'Première version de BED — Bitcoin Encrypted Backup pour StartOS.',
      },
      migrations: {
        up: async ({ effects }) => {},
        down: async ({ effects }) => {},
      },
    })
    ```
    Empty migrations match D-10 (zero migration code in v1). Locale set MUST match the i18n.ts locale set from Task 1.

    Step 5. Write /workspace/bed-startos/startos/versions/index.ts (VersionGraph):
    ```typescript
    import { VersionGraph } from '@start9labs/start-sdk'
    import { v_0_1_0_1 } from './v0.1.0.1'

    export const versions = VersionGraph.of(v_0_1_0_1)
    ```
    If the installed SDK exposes `VersionGraph.of({ current, other })` shape (RESEARCH.md Pattern 5 hint), use:
    ```typescript
    export const versions = VersionGraph.of({ current: v_0_1_0_1, other: [] })
    ```
    Verify against the installed SDK's `VersionGraph.d.ts` BEFORE writing — pick the form the type signature accepts.

    Step 6. Run `cd /workspace/bed-startos && npx tsc --noEmit`. At this point, the only remaining errors should be from i18n dictionary additions (Task 4).
  </action>
  <verify>
    <automated>grep -q "Backups.ofVolumes('main')" /workspace/bed-startos/startos/backups.ts &amp;&amp; grep -q "version: '0.1.0:1'" /workspace/bed-startos/startos/versions/v0.1.0.1.ts &amp;&amp; grep -q "migrations:" /workspace/bed-startos/startos/versions/v0.1.0.1.ts &amp;&amp; grep -q "v_0_1_0_1" /workspace/bed-startos/startos/versions/index.ts &amp;&amp; grep -q "Actions.of()" /workspace/bed-startos/startos/actions/index.ts</automated>
  </verify>
  <acceptance_criteria>
    - backups.ts contains `Backups.ofVolumes('main')`.
    - dependencies.ts exports `dependencies` (empty deps).
    - actions/index.ts contains `Actions.of()` and is otherwise empty.
    - versions/v0.1.0.1.ts contains `version: '0.1.0:1'` and `migrations:` block with empty up/down.
    - versions/index.ts imports `v_0_1_0_1` and constructs a VersionGraph.
    - The releaseNotes locale set in v0.1.0.1.ts matches the locale set in manifest/i18n.ts (Task 1).
  </acceptance_criteria>
  <done>Backups, dependencies, actions, versions are all declared per StartOS 0.4.0 SDK requirements with v0.1.0:1 as the initial version.</done>
</task>

<task id="04-04-i18n-dictionary-and-typecheck" type="auto">
  <name>Task 4: Add BED-specific strings to i18n dictionaries and run final tsc --noEmit</name>
  <read_first>
    - /workspace/bed-startos/startos/i18n/dictionaries/default.ts (current placeholder strings — see what hello-world used as keys)
    - /workspace/bed-startos/startos/i18n/dictionaries/translations.ts (translations file structure)
    - /workspace/bed-startos/startos/i18n/index.ts (PLUMBING — read to understand i18n() helper signature; do NOT edit)
  </read_first>
  <files>
    - /workspace/bed-startos/startos/i18n/dictionaries/default.ts
    - /workspace/bed-startos/startos/i18n/dictionaries/translations.ts
  </files>
  <action>
    Step 1. List the i18n() string keys used in interfaces.ts and main.ts. From Task 2 these are:
    - `'Web UI'`
    - `'Encrypt and decrypt Bitcoin descriptor backups'`
    - `'Web Interface'`
    - `'The web interface is ready'`
    - `'The web interface is not ready'`

    Step 2. Open /workspace/bed-startos/startos/i18n/dictionaries/default.ts. Add (or replace placeholder hello-world strings with) the BED-specific strings. The exact dict shape comes from the upstream file — preserve its export name and key structure. Each key is a string the i18n() helper looks up.

    Example shape (verify against upstream first — if upstream uses a different export shape, MIRROR upstream):
    ```typescript
    export const default_dict = {
      'Web UI': 'Web UI',
      'Encrypt and decrypt Bitcoin descriptor backups': 'Encrypt and decrypt Bitcoin descriptor backups',
      'Web Interface': 'Web Interface',
      'The web interface is ready': 'The web interface is ready',
      'The web interface is not ready': 'The web interface is not ready',
    }
    ```

    Step 3. Open /workspace/bed-startos/startos/i18n/dictionaries/translations.ts. For each non-default locale (es_ES, de_DE, pl_PL, fr_FR — verified set), add translations for the same keys. Castellano peninsular (no argentino):
    ```
    'Web UI': 'Interfaz web'
    'Encrypt and decrypt Bitcoin descriptor backups': 'Cifra y descifra backups de descriptors de Bitcoin'
    'Web Interface': 'Interfaz web'
    'The web interface is ready': 'La interfaz web está lista'
    'The web interface is not ready': 'La interfaz web no está lista'
    ```
    For other locales, machine-quality translations are acceptable; the strings appear only in the StartOS dashboard health status display.

    Step 4. DELETE any stale hello-world dict entries (e.g. `'Hello World'`, `'Hello, World'`, `'Charset'`) — they are template residue. Run `grep -i 'hello' /workspace/bed-startos/startos/i18n/dictionaries/*.ts` after the edit; result MUST be empty.

    Step 5. Run final type check from /workspace/bed-startos:
    ```
    cd /workspace/bed-startos && npx tsc --noEmit 2>&1 | tee /tmp/bed-startos-final-tsc.log
    ```
    The exit code MUST be 0 and the log MUST be empty (or contain only informational notes — no errors). If errors remain:
    - Errors about missing module exports → re-run grep to find the symbol and verify Tasks 1-3 wrote it correctly.
    - Errors about VersionGraph.of signature → check `node_modules/@start9labs/start-sdk/package/lib/version/VersionGraph.d.ts` and use the form the SDK accepts.
    - Errors about manifest field names → check `setupManifest.d.ts`.
    DO NOT silence errors with `// @ts-ignore` or `any` casts. If the SDK API genuinely diverges from RESEARCH.md, update the code to match the SDK source — do not hide drift.

    Step 6. Run prettier check (s9pk.mk plumbing requires it):
    ```
    cd /workspace/bed-startos && npx prettier --check 'startos/**/*.ts'
    ```
    If formatting violations: `npx prettier --write 'startos/**/*.ts'`. Re-run check.
  </action>
  <verify>
    <automated>cd /workspace/bed-startos &amp;&amp; npx tsc --noEmit &amp;&amp; ! grep -irq 'hello' startos/i18n/dictionaries/*.ts</automated>
  </verify>
  <acceptance_criteria>
    - `cd /workspace/bed-startos && npx tsc --noEmit` exits 0 with no error output.
    - `cd /workspace/bed-startos && npx prettier --check 'startos/**/*.ts'` exits 0.
    - All five i18n keys used in interfaces.ts/main.ts ('Web UI', 'Encrypt and decrypt Bitcoin descriptor backups', 'Web Interface', 'The web interface is ready', 'The web interface is not ready') exist in default.ts.
    - Each key in default.ts has a corresponding entry in every locale of translations.ts.
    - `grep -i 'hello' /workspace/bed-startos/startos/i18n/dictionaries/` returns no matches.
  </acceptance_criteria>
  <done>The full TypeScript surface compiles with zero errors. All i18n strings used by BED resolve. Hello-world residue is purged from dictionaries.</done>
</task>

<task id="04-05-icon-license-instructions" type="auto">
  <name>Task 5: Generate icon (1024×1024 BED textual SVG → PNG), write LICENSE (MIT), write instructions.md</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-05 (BED textual logo, monospace bold, Semilla Bitcoin palette, 1024×1024 PNG)
    - /workspace/descriptor-cifrado/frontend/public/fonts/ (verify what monospace font the frontend uses self-hosted — likely JetBrains Mono per Phase 2 STATE; align icon font for visual consistency)
  </read_first>
  <files>
    - /workspace/bed-startos/icon.svg
    - /workspace/bed-startos/icon.png
    - /workspace/bed-startos/LICENSE
    - /workspace/bed-startos/instructions.md
  </files>
  <action>
    Step 1. Create /workspace/bed-startos/icon.svg with a 1024×1024 viewBox showing the text "BED" centered, monospace bold, on a Bitcoin-orange or near-black background. Two acceptable variants — pick orange-on-black for best dashboard contrast (Bitcoin orange `#f7931a` text on `#0c0c0c` background):
    ```xml
    &lt;?xml version="1.0" encoding="UTF-8"?&gt;
    &lt;svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024"&gt;
      &lt;rect width="1024" height="1024" fill="#0c0c0c"/&gt;
      &lt;text
        x="512"
        y="600"
        fill="#f7931a"
        font-family="JetBrains Mono, Menlo, Consolas, monospace"
        font-weight="800"
        font-size="380"
        text-anchor="middle"&gt;BED&lt;/text&gt;
    &lt;/svg&gt;
    ```
    The text x/y is tuned for visual centering; the monospace font ascender pushes y above the geometric center.

    Step 2. Convert SVG → PNG at 1024×1024. Use rsvg-convert (preferred — preserves font metrics):
    ```
    rsvg-convert -w 1024 -h 1024 -o /workspace/bed-startos/icon.png /workspace/bed-startos/icon.svg
    ```
    If rsvg-convert is not installed, fall back to inkscape:
    ```
    inkscape /workspace/bed-startos/icon.svg --export-type=png --export-filename=/workspace/bed-startos/icon.png -w 1024 -h 1024
    ```
    If neither is installed, use ImageMagick:
    ```
    magick -background '#0c0c0c' -density 1024 /workspace/bed-startos/icon.svg -resize 1024x1024 /workspace/bed-startos/icon.png
    ```
    Verify dimensions:
    ```
    file /workspace/bed-startos/icon.png  # must report PNG image data, 1024 x 1024
    ```
    Verify size: `stat -c '%s' /workspace/bed-startos/icon.png` should be < 200 KB (a textual logo on solid bg compresses well).

    Step 3. Write /workspace/bed-startos/LICENSE with the standard MIT text. Copyright line: `Copyright (c) 2026 Semilla Bitcoin (4rkad &lt;55397917+4rkad@users.noreply.github.com&gt;)`. Use the standard MIT body verbatim (https://opensource.org/license/mit). Year 2026 per currentDate.

    Step 4. Write /workspace/bed-startos/instructions.md (StartOS shows this on package page after install). 30-60 lines, English (D-02 README language), markdown. Required sections:
    - **Quickstart** — open Tor URL or LAN URL from dashboard, paste descriptor, encrypt, download .bed.
    - **Golden rule** — verbatim "Never store a `.bed` file and a cosigner xpub of the same multisig in the same location" (DOC-02 visibility on the StartOS package page).
    - **Threat model summary** — 2-3 sentences pointing to the full README.
    - **Where files are stored** — /data/encrypted/ when history toggle is ON (default OFF). StartOS backups via the platform's Backup feature include this directory automatically.
    - **Updating** — "Updates preserve files saved in /data/encrypted/. To verify, save a .bed before update, then check the Historial tab after."
    - **Reporting issues** — link to https://github.com/semillabitcoin/bed-startos/issues for s9pk wrapper and https://github.com/semillabitcoin/descriptor-cifrado/issues for the underlying tool.
    - Include a link to the project README: `https://github.com/semillabitcoin/descriptor-cifrado#threat-model`.

    Step 5. Visual verification of icon — convert PNG to ASCII or open in a viewer. Since this is autonomous-written (then reviewed in Task 7 checkpoint), produce a base64 thumbnail in the task output for human inspection at the checkpoint:
    ```
    magick /workspace/bed-startos/icon.png -resize 64x64 /tmp/icon-thumb.png
    base64 /tmp/icon-thumb.png | head -c 200
    ```
    Append the base64 prefix to the SUMMARY.md so the user can spot-check at the Task 7 checkpoint.
  </action>
  <verify>
    <automated>test -f /workspace/bed-startos/icon.png &amp;&amp; file /workspace/bed-startos/icon.png | grep -q '1024 x 1024' &amp;&amp; test -f /workspace/bed-startos/LICENSE &amp;&amp; grep -q 'MIT License\|Permission is hereby granted' /workspace/bed-startos/LICENSE &amp;&amp; test -f /workspace/bed-startos/instructions.md &amp;&amp; grep -q 'Never store a' /workspace/bed-startos/instructions.md</automated>
  </verify>
  <acceptance_criteria>
    - /workspace/bed-startos/icon.svg exists with viewBox 1024×1024 and contains the literal text "BED".
    - /workspace/bed-startos/icon.png exists with dimensions exactly 1024×1024 (verified via `file`).
    - /workspace/bed-startos/icon.png file size is between 1 KB and 200 KB (sanity bound).
    - /workspace/bed-startos/LICENSE exists, contains "MIT License" or the canonical "Permission is hereby granted, free of charge" sentence, contains "Copyright (c) 2026" and "Semilla Bitcoin".
    - /workspace/bed-startos/instructions.md exists with at least 30 lines and contains the literal phrase "Never store a `.bed`" or "never co-locate".
    - instructions.md contains a link to `descriptor-cifrado#threat-model` (deep README anchor).
  </acceptance_criteria>
  <done>Icon, LICENSE, and instructions are present per D-05 and StartOS package conventions. The icon is 1024×1024 BED textual logo. The license is MIT. The instructions surface the golden rule on the StartOS package page.</done>
</task>

<task id="04-06-write-readme-and-ci-workflow" type="auto">
  <name>Task 6: Write bed-startos README.md, .github/workflows/release.yml (CI build + GitHub Release)</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Build Pipeline" (Option A shared-workflows + Option B custom — choose Option B for explicit control + auth handling)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Common Pitfalls" Pitfall 1 (GHCR auth) and Pitfall 8 (developer.key.pem secret)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-17 (own CI), D-13 (noreply commits)
    - /workspace/descriptor-cifrado/.github/workflows/docker.yml (existing project workflow conventions: action versions @v4/@v5/@v6, setup steps order)
    - /workspace/descriptor-cifrado/README.md (the README written by Plan 02 — link from bed-startos README to it)
  </read_first>
  <files>
    - /workspace/bed-startos/README.md (overwrite the stub from Plan 03)
    - /workspace/bed-startos/.github/workflows/release.yml
  </files>
  <action>
    Step 1. Write /workspace/bed-startos/README.md (English, ~80-120 lines). Required sections:

    1. **Title + tagline + badges**: "# BED — Bitcoin Encrypted Backup (StartOS)"
    2. **TL;DR** with the golden-rule callout (verbatim "Never store a `.bed` file and a cosigner xpub of the same multisig in the same location" — DOC-02 first occurrence).
    3. **Install** with two paths:
       - Sideload: download `bed_x86_64.s9pk` or `bed_aarch64.s9pk` from latest GitHub Release, then `start-cli package install -s bed_<arch>.s9pk` (or via StartOS UI "Sideload package").
       - Registry: when published to the Semilla Bitcoin registry, install via that registry URL in StartOS settings.
    4. **Usage**: brief — open Tor or LAN URL → paste descriptor → encrypt. Link to the project README usage section: `[Full usage docs](https://github.com/semillabitcoin/descriptor-cifrado#usage)`.
    5. **Threat model summary**: 1 paragraph with the golden-rule callout REPEATED (DOC-02 second occurrence). Link to deep model: `[Full threat model](https://github.com/semillabitcoin/descriptor-cifrado#threat-model)`.
    6. **What's in the s9pk**:
       - Single subcontainer running `ghcr.io/semillabitcoin/descriptor-cifrado` (digest-pinned).
       - Volume `main` mounts at `/data/encrypted/`; preserved across updates and included in StartOS backups.
       - Health check via `checkPortListening` on port 8080.
       - Tor + LAN interfaces auto-generated by StartOS.
    7. **Building from source**: `npm ci && make clean x86 arm`. Note: requires `start-cli` and `~/.startos/developer.key.pem`.
    8. **Reporting issues**: link to issues, separately for wrapper vs underlying tool.
    9. **License**: MIT, see LICENSE.

    Step 2. Verify the golden rule appears at LEAST twice (TL;DR + Threat model summary):
    ```
    grep -c 'never co-locate\|Never store a' /workspace/bed-startos/README.md
    ```
    MUST be ≥ 2.

    Step 3. Create /workspace/bed-startos/.github/workflows/release.yml using Option B (custom minimal CI from RESEARCH.md). Use Option B over Option A because it gives explicit control over the GHCR auth step (Pitfall 1) which the shared workflow may not handle for non-Start9 image sources:

    ```yaml
    name: Release

    on:
      push:
        tags:
          - 'v*.*.*'
      workflow_dispatch:

    permissions:
      contents: write    # for softprops/action-gh-release

    jobs:
      build:
        runs-on: ubuntu-latest
        strategy:
          fail-fast: false
          matrix:
            arch:
              - { name: x86, target: x86_64 }
              - { name: arm, target: aarch64 }
        steps:
          - uses: actions/checkout@v4
            with:
              fetch-depth: 0

          - name: Setup developer key
            run: |
              mkdir -p ~/.startos
              printf '%s' '${{ secrets.DEV_KEY }}' > ~/.startos/developer.key.pem
              chmod 600 ~/.startos/developer.key.pem

          - name: Setup Node.js
            uses: actions/setup-node@v4
            with:
              node-version: '20'
              cache: npm

          - name: Install start-cli
            run: |
              curl -L -o /usr/local/bin/start-cli \
                https://github.com/Start9Labs/start-os/releases/download/v0.4.0-beta.5/start-cli-x86_64
              chmod +x /usr/local/bin/start-cli
              start-cli --version

          - name: Login to GHCR (read:packages for image pull)
            uses: docker/login-action@v3
            with:
              registry: ghcr.io
              username: ${{ github.repository_owner }}
              password: ${{ secrets.GITHUB_TOKEN }}

          - name: Install npm deps
            run: npm ci

          - name: Build s9pk (${{ matrix.arch.target }})
            run: make clean ${{ matrix.arch.name }}

          - name: Verify s9pk artifact
            run: |
              ls -la *.s9pk
              start-cli s9pk inspect bed_${{ matrix.arch.target }}.s9pk manifest

          - name: Upload artifact
            uses: actions/upload-artifact@v4
            with:
              name: bed-${{ matrix.arch.target }}-s9pk
              path: bed_${{ matrix.arch.target }}.s9pk
              if-no-files-found: error
              retention-days: 30

      release:
        runs-on: ubuntu-latest
        needs: [build]
        if: startsWith(github.ref, 'refs/tags/v')
        steps:
          - uses: actions/download-artifact@v4
            with:
              path: artifacts
              merge-multiple: true

          - name: Create GitHub Release
            uses: softprops/action-gh-release@v2
            with:
              files: artifacts/*.s9pk
              fail_on_unmatched_files: true
              draft: false
              prerelease: false
              generate_release_notes: true
    ```

    Critical points:
    - `on.push.tags: ['v*.*.*']` triggers on every semver tag. (Plan 05 will push `v0.1.0`.)
    - The GHCR login step uses `secrets.GITHUB_TOKEN` which by default has `read:packages` for the same org's packages. If the descriptor-cifrado package is private at CI time, this auth lets the pack succeed (Pitfall 1 mitigation). After Plan 01 flipped the package public, this step is harmless redundancy — it does NOT fail.
    - `start-cli` is downloaded from the StartOS releases. Pin to `v0.4.0-beta.5` (matches the `start-cli` already at `/workspace/.cargo/bin/start-cli` per RESEARCH.md Environment Availability table). When a newer start-cli releases, bump this URL — cosmetic.
    - The `developer.key.pem` is loaded from `secrets.DEV_KEY`. Plan 05 stores this secret. The workflow will FAIL on first tag push until the secret is configured (expected — Plan 05 has a checkpoint for this).
    - The matrix produces two parallel builds. The `release` job depends on both via `needs: [build]` and uploads both `.s9pk` files to the same GitHub Release.
    - Only triggers on tags `v*.*.*` (avoid main pushes triggering releases). `workflow_dispatch` allows manual re-runs from the Actions UI for debugging.

    Step 4. Verify YAML syntax with a parser before commit:
    ```
    npx --yes js-yaml /workspace/bed-startos/.github/workflows/release.yml > /dev/null
    ```
    OR:
    ```
    python3 -c "import yaml,sys; yaml.safe_load(open('/workspace/bed-startos/.github/workflows/release.yml'))"
    ```
    Either MUST exit 0.

    Step 5. Stage and commit all Plan 04 work in a single commit (Tasks 1-6 collectively):
    ```
    cd /workspace/bed-startos
    git add -A
    git -c user.email="55397917+4rkad@users.noreply.github.com" \
      commit -m "feat(bed-startos): manifest, main, interfaces, backups, versions, icon, LICENSE, README, CI

    - Pin GHCR image to ghcr.io/semillabitcoin/descriptor-cifrado@<digest>
    - Volume main → /data/encrypted; checkPortListening on 8080
    - Single bindPort generates Tor + LAN interfaces (D-14)
    - VersionInfo v0.1.0:1 with empty migrations (D-10)
    - Icon: BED textual logo 1024x1024
    - README covers install, threat model summary, golden rule (DOC-01, DOC-02)
    - CI: matrix build x86+arm on tag push v*.*.* + GitHub Release upload"
    git push origin main
    ```
  </action>
  <verify>
    <automated>test -f /workspace/bed-startos/README.md &amp;&amp; grep -cE 'never co-locate|Never store a' /workspace/bed-startos/README.md | awk '$1 >= 2 {ok=1} END {exit !ok}' &amp;&amp; test -f /workspace/bed-startos/.github/workflows/release.yml &amp;&amp; python3 -c "import yaml,sys; yaml.safe_load(open('/workspace/bed-startos/.github/workflows/release.yml'))" &amp;&amp; grep -q "make clean" /workspace/bed-startos/.github/workflows/release.yml &amp;&amp; grep -qE "tags:" /workspace/bed-startos/.github/workflows/release.yml</automated>
  </verify>
  <acceptance_criteria>
    - bed-startos/README.md exists with at least 80 lines.
    - bed-startos/README.md contains the literal phrase "Never store a `.bed`" OR "never co-locate" — at least TWICE (DOC-02 redundancy).
    - bed-startos/README.md contains a link to `https://github.com/semillabitcoin/descriptor-cifrado#threat-model`.
    - bed-startos/README.md contains a link to `https://github.com/semillabitcoin/descriptor-cifrado#usage`.
    - .github/workflows/release.yml is valid YAML (parser exits 0).
    - release.yml `on.push.tags` contains `'v*.*.*'`.
    - release.yml has a `matrix.arch` with two entries (x86_64 + aarch64).
    - release.yml runs `make clean ${{ matrix.arch.name }}`.
    - release.yml has a `release` job that uses `softprops/action-gh-release@v2` with `files: artifacts/*.s9pk`.
    - release.yml has a `Login to GHCR` step using `docker/login-action@v3`.
    - All Plan 04 work is committed via the noreply email; `cd /workspace/bed-startos && git log -1 --format='%ae'` outputs `55397917+4rkad@users.noreply.github.com`.
    - The commit is pushed to origin/main.
  </acceptance_criteria>
  <done>bed-startos has a complete TypeScript manifest surface, icon, LICENSE, README with golden-rule redundancy, instructions for the dashboard, and a CI workflow that produces signed multi-arch s9pks on tag push. The repo is ready for Plan 05 to do a local pack smoke-test and then trigger a real tag release.</done>
</task>

<task id="04-07-checkpoint-pre-pack" type="checkpoint:human-verify" gate="blocking">
  <name>Task 7: Pre-pack verify — TS surface, icon, README, CI workflow ready</name>
  <action>Human runs the seven verification commands listed under how-to-verify (tsc check, digest grep, icon dimensions, golden-rule count, YAML parse, visual icon review, git log email). Approving releases Plan 05 (Wave 3) for local pack + tag.</action>
  <what-built>
    - Full bed-startos TypeScript surface: manifest, i18n, main, interfaces, backups, dependencies, actions, versions, dictionaries
    - tsc --noEmit passes with zero errors
    - Icon (1024×1024 BED logo), LICENSE (MIT), instructions.md (with golden rule), README.md (with golden rule TWICE)
    - .github/workflows/release.yml (matrix x86+arm, GHCR auth, GitHub Release upload)
    - All committed and pushed to private semillabitcoin/bed-startos main
  </what-built>
  <how-to-verify>
    Run these checks and paste output:
    1. `cd /workspace/bed-startos && npx tsc --noEmit && echo OK` — must print OK.
    2. `grep "dockerTag:" /workspace/bed-startos/startos/manifest/index.ts` — must show `ghcr.io/semillabitcoin/descriptor-cifrado@sha256:` followed by 64 hex chars matching the contents of /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt.
    3. `file /workspace/bed-startos/icon.png` — must report `1024 x 1024`.
    4. `grep -c -E 'never co-locate|Never store a' /workspace/bed-startos/README.md` — must output 2 or higher.
    5. `python3 -c "import yaml; yaml.safe_load(open('/workspace/bed-startos/.github/workflows/release.yml'))"` — must exit 0.
    6. Open `/workspace/bed-startos/icon.png` in an image viewer and confirm the "BED" textual logo is legible at thumbnail size (this is the only manual visual check — D-05 says "if the preview no convince fallback to opción D"). If not legible / not visually acceptable, describe the issue and we tweak the SVG before Plan 05.
    7. `cd /workspace/bed-startos && git log --format='%ae %s' | head -3` — all commits must show the noreply email.

    If all seven pass AND the icon is acceptable, Plan 05 can run `make clean x86 arm` locally as the first smoke test before tagging v0.1.0.
  </how-to-verify>
  <resume-signal>Type "approved" to release Plan 05, or describe icon tweaks / TS errors / YAML issues to fix.</resume-signal>
</task>

</tasks>

<verification>
- All TypeScript files compile (tsc --noEmit exits 0).
- Manifest pins exact digest from 01-DIGEST.txt.
- main.ts mounts /data/encrypted, runs checkPortListening on 8080.
- interfaces.ts uses single bindPort for Tor+LAN.
- backups.ts preserves volume across updates.
- v0.1.0:1 declared with empty migrations.
- Icon 1024×1024, LICENSE MIT, instructions+README contain golden rule.
- release.yml is valid YAML with matrix x86+arm and GitHub Release upload.
- All commits use the noreply email.
</verification>

<success_criteria>
A subsequent agent can `cd /workspace/bed-startos && make clean x86 arm` and produce two valid s9pk files. `start-cli s9pk inspect bed_x86_64.s9pk manifest` returns a JSON object with `id: "bed"`, `title: "BED"`, the digest-pinned image source, and the volume declaration. Tagging `v0.1.0` triggers the CI to produce both arch s9pks as GitHub Release assets.
</success_criteria>

<output>
After completion, create `.planning/phases/04-startos-packaging-docs/04-04-SUMMARY.md` recording:
- File inventory of bed-startos/startos/ (tree)
- The exact digest pinned in manifest/index.ts (verbatim)
- Locale set used in i18n (e.g., `en_US, es_ES, de_DE, pl_PL, fr_FR`)
- Icon dimensions and file size
- README golden-rule occurrence count
- release.yml matrix arch list
- Commit SHA pushed to origin/main
- Base64 thumbnail of icon (200 chars) for traceability
</output>
