---
phase: 04-startos-packaging-docs
plan: "04"
title: "Author bed-startos manifest, main, interfaces, backups, versions, icon, LICENSE, README, CI workflow"
subsystem: startos-packaging
tags: [startos, s9pk, manifest, typescript, ci, github-actions, icon, readme]
dependency_graph:
  requires:
    - phase: 04-startos-packaging-docs
      plan: "01"
      provides: "GHCR image public at sha256:41684bce9dd4ec6270965f8df2caafecab031b573ab9dd52c38937c057fa67b5"
    - phase: 04-startos-packaging-docs
      plan: "03"
      provides: "bed-startos scaffold at /home/anon/bed-startos with SDK 1.4.1"
  provides:
    - "bed-startos complete TypeScript surface: manifest, i18n, main, interfaces, backups, dependencies, actions, versions"
    - "manifest pins GHCR image by digest (D-01)"
    - "icon 1024x1024 BED textual logo"
    - "LICENSE MIT 2026 Semilla Bitcoin"
    - "instructions.md (StartOS package page)"
    - "README.md with threat model and golden rule x2 (DOC-01, DOC-02)"
    - ".github/workflows/release.yml — matrix x86+arm CI on tag push"
  affects: [04-05-local-pack-uat]
tech-stack:
  added: []
  patterns:
    - "Digest-pinned GHCR image in manifest.images (not tag-based, D-01)"
    - "Single bindPort(8080, http) generates both Tor + LAN (D-14)"
    - "Mounts.mountVolume main → /data/encrypted matches BED_DATA_DIR default (D-15)"
    - "checkPortListening on 8080 detects 127.0.0.1 bind via /proc/net/tcp (D-16)"
    - "VersionGraph.of({ current: v_0_1_0_1, other: [] }) for initial version"
    - "i18n dict uses numeric indices keyed in default.ts, locale strings in translations.ts"
key-files:
  created:
    - /home/anon/bed-startos/startos/versions/v0.1.0.1.ts
    - /home/anon/bed-startos/icon.png
    - /home/anon/bed-startos/instructions.md
  modified:
    - /home/anon/bed-startos/startos/manifest/index.ts
    - /home/anon/bed-startos/startos/manifest/i18n.ts
    - /home/anon/bed-startos/startos/main.ts
    - /home/anon/bed-startos/startos/interfaces.ts
    - /home/anon/bed-startos/startos/utils.ts
    - /home/anon/bed-startos/startos/versions/index.ts
    - /home/anon/bed-startos/startos/i18n/dictionaries/default.ts
    - /home/anon/bed-startos/startos/i18n/dictionaries/translations.ts
    - /home/anon/bed-startos/icon.svg
    - /home/anon/bed-startos/LICENSE
    - /home/anon/bed-startos/README.md
    - /home/anon/bed-startos/.github/workflows/release.yml
key-decisions:
  - "Plan 03 already wrote correct backups.ts (Backups.ofVolumes), dependencies.ts, and actions/index.ts — no re-write needed"
  - "uiPort was 80 (template default) — corrected to 8080 (Rule 1 bug fix)"
  - "interfaces.ts host renamed from 'ui-multi' to 'main' per RESEARCH.md Pattern 3"
  - "Removed stale 'Starting BED!' i18n key since main.ts no longer calls console.info"
  - "Old hello-world version v2.0.0_4.ts kept (not deleted) — only VersionGraph.of updated to point to v_0_1_0_1"
  - "Icon PNG generated with Python PIL using DejaVu Sans Mono Bold (rsvg-convert/inkscape not available)"
  - "release.yml: Option B custom CI (explicit GHCR auth step) — gives control over Pitfall 1 (private GHCR pull)"
  - "prettier --write applied after writing manifest files; reformatted dockerTag to 2-line style (value on next line)"
requirements-completed: [S9-02, S9-03, S9-05, DOC-01, DOC-02]
duration: 45
completed: "2026-05-07"
---

# Phase 4 Plan 04: Author bed-startos TypeScript Surface, Assets, and CI Workflow

**bed-startos manifest TypeScript surface complete: digest-pinned GHCR image, Tor+LAN via bindPort, volume main→/data/encrypted, checkPortListening health check, VersionInfo v0.1.0:1, icon 1024x1024, MIT LICENSE, instructions.md with golden rule, README with golden rule x2, custom CI release.yml producing multi-arch s9pks on v*.*.* tag push.**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-07T16:46:00Z
- **Completed:** 2026-05-07T18:00:00Z
- **Tasks:** 6 auto + 1 checkpoint
- **Files modified:** 12

## Accomplishments

- manifest/index.ts pins `ghcr.io/semillabitcoin/descriptor-cifrado@sha256:41684bce9dd4ec6270965f8df2caafecab031b573ab9dd52c38937c057fa67b5` (D-01)
- main.ts: volume `main` → `/data/encrypted`, `useEntrypoint()`, `checkPortListening` on port 8080 (D-15, D-16)
- interfaces.ts: single `bindPort(8080, { protocol: 'http' })` generates both Tor onion + LAN .local (D-14)
- versions/v0.1.0.1.ts: `VersionInfo.of({ version: '0.1.0:1' })` with empty migrations (D-10)
- i18n dictionaries: 5 BED-specific keys in 5 locales (en_US, es_ES, de_DE, pl_PL, fr_FR)
- icon.png: 1024x1024 BED textual logo (#f7931a on #0c0c0c) — 14.9 KB
- README.md: 120 lines, golden rule appears TWICE (DOC-02), links to full threat model and usage docs
- release.yml: custom CI with matrix x86+arm, GHCR auth, `make clean $arch`, `start-cli s9pk inspect` verify, GitHub Release upload via softprops/action-gh-release@v2
- `tsc --noEmit` passes with zero errors; `prettier --check` passes

## bed-startos/startos/ File Inventory

```
startos/
├── actions/index.ts          — sdk.Actions.of() (empty, no custom actions v1)
├── backups.ts                — Backups.ofVolumes('main') (S9-05)
├── dependencies.ts           — setupDependencies (empty, no cross-package deps)
├── i18n/
│   ├── dictionaries/
│   │   ├── default.ts        — 5 BED keys with numeric indices (0-4)
│   │   └── translations.ts   — es_ES, de_DE, pl_PL, fr_FR translations
│   └── index.ts              — PLUMBING (setupI18n, do not edit)
├── index.ts                  — PLUMBING (entrypoint, do not edit)
├── init/index.ts             — PLUMBING (do not edit)
├── interfaces.ts             — MultiHost 'main', bindPort(8080, http), createInterface 'ui'
├── main.ts                   — Mounts.mountVolume main→/data/encrypted, SubContainer, Daemons + health
├── manifest/
│   ├── i18n.ts               — short/long in 5 locales, golden rule in en_US + es_ES
│   └── index.ts              — setupManifest: id=bed, digest pin, volumes, arch, empty deps
├── sdk.ts                    — PLUMBING (do not edit)
├── utils.ts                  — uiPort = 8080
└── versions/
    ├── index.ts              — VersionGraph.of({ current: v_0_1_0_1, other: [] })
    ├── v0.1.0.1.ts           — VersionInfo v0.1.0:1, releaseNotes 5 locales, empty migrations
    └── v2.0.0_4.ts           — hello-world template version (kept, not referenced)
```

## Task Commits

Each task was committed atomically (all in bed-startos repo):

1. **Task 1: manifest/index.ts + manifest/i18n.ts** — `931f5c5` (feat)
2. **Task 2: utils.ts (uiPort=8080) + interfaces.ts + main.ts** — `87f2d5f` (feat)
3. **Task 3: versions/v0.1.0.1.ts + versions/index.ts** — `9663059` (feat)
4. **Task 4: i18n dictionaries + prettier** — `004301d` (feat)
5. **Task 5: icon.svg + icon.png + LICENSE + instructions.md** — `0b558d8` (feat)
6. **Task 6: README.md + .github/workflows/release.yml** — `f4f64a8` (feat)
7. **Task 7: checkpoint:human-verify** — paused (this SUMMARY is the pre-checkpoint artifact)

All commits pushed to `origin/main`. All use email `55397917+4rkad@users.noreply.github.com`.

## Key Manifest Facts

- **Digest pinned:** `sha256:41684bce9dd4ec6270965f8df2caafecab031b573ab9dd52c38937c057fa67b5`
  (verbatim from `/home/anon/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt`)
- **Locale set:** `en_US, es_ES, de_DE, pl_PL, fr_FR` (matches translations.ts)
- **Icon:** 1024x1024 PNG, 14,919 bytes, DejaVu Sans Mono Bold, #f7931a on #0c0c0c
- **README golden-rule occurrences:** 2 (`grep -cE 'never co-locate|Never store a' README.md` → 2)
- **release.yml matrix arches:** `x86_64` + `aarch64`

## Icon Base64 Thumbnail (64x64 preview, first 200 chars)

`iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAGbklEQVR4nO2ZXahcVxXH/2vvs8/nnXuvaYw1RqTGloCmqZaIWAQDFaXE1FraomJV0Ad90Rd9qdSPqvRVpFqkUBqrDUoRaUtpWozcxqbBGj8CqTGmuSnJTULu3NyZzD1zzpy91/LhzM39mntn8gGHwPw5`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] uiPort was 80 in utils.ts (template default) — corrected to 8080**
- **Found during:** Task 2 (writing utils.ts)
- **Issue:** Plan 03 left `uiPort = 80` from the hello-world template. BED binds to 8080.
- **Fix:** Changed `export const uiPort = 80` to `export const uiPort = 8080` in utils.ts.
- **Files modified:** `startos/utils.ts`
- **Committed in:** `87f2d5f`

**2. [Rule 1 - Bug] interfaces.ts host was 'ui-multi' — renamed to 'main' per RESEARCH.md Pattern 3**
- **Found during:** Task 2
- **Issue:** Plan 03 used host ID `'ui-multi'` in `MultiHost.of(effects, 'ui-multi')`. RESEARCH.md Pattern 3 specifies `'main'`.
- **Fix:** Rewrote interfaces.ts to use `sdk.MultiHost.of(effects, 'main')` and simplified the export pattern.
- **Files modified:** `startos/interfaces.ts`
- **Committed in:** `87f2d5f`

**3. [Rule 1 - Bug] Stale 'Starting BED!' i18n key removed — main.ts no longer calls console.info**
- **Found during:** Task 4 (i18n dictionary cleanup)
- **Issue:** default.ts had index 0 = `'Starting BED!'` from the hello-world template logging call. The new main.ts does not call `i18n('Starting BED!')`. Leaving it caused a dangling entry.
- **Fix:** Removed the key, renumbered remaining 5 keys (0-4), updated translations.ts indices accordingly.
- **Files modified:** `startos/i18n/dictionaries/default.ts`, `startos/i18n/dictionaries/translations.ts`
- **Committed in:** `004301d`

**4. [Rule 2 - Auto] Icon PNG generated via Python PIL (rsvg-convert/inkscape unavailable)**
- **Found during:** Task 5
- **Issue:** Neither `rsvg-convert` nor `inkscape` nor `magick` were available in the environment.
- **Fix:** Used `python3 PIL` (available) to create the 1024x1024 PNG directly with DejaVu Sans Mono Bold, #f7931a text on #0c0c0c background.
- **Files modified:** `icon.png`
- **Committed in:** `0b558d8`

**5. [Rule 1 - Bug] prettier reformatted dockerTag to 2-line style**
- **Found during:** Task 4 (prettier --write applied to all startos/**/*.ts)
- **Issue:** The single-line `dockerTag: 'ghcr.io/...@sha256:...'` exceeded prettier's printWidth limit and was reformatted to 2 lines (key on one line, value on next).
- **Impact:** The plan's automated grep pattern `grep -E "dockerTag: 'ghcr.io/..."` no longer matches a single line. The digest value is still present and correctly pinned — only the line layout changed.
- **Files modified:** `startos/manifest/index.ts` (via prettier)
- **Committed in:** `004301d`

---

**Total deviations:** 5 auto-fixed (3 Rule 1 bugs, 1 Rule 2 missing fallback, 1 cosmetic prettier)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Known Stubs

None. All plan deliverables are complete with real content. The old hello-world version file `v2.0.0_4.ts` remains in `versions/` but is no longer referenced by `VersionGraph` — it is inert template residue (not a stub affecting functionality).

## Next Phase Readiness

Plan 05 can now:
1. Run `cd /home/anon/bed-startos && make clean x86 arm` — should produce `bed_x86_64.s9pk` and `bed_aarch64.s9pk`
2. Run `start-cli s9pk inspect bed_x86_64.s9pk manifest` — should return manifest JSON with `id: "bed"`, digest-pinned image, volume declaration
3. Sideload to a real StartOS 0.4.0 device for UAT (S9-04 — manual, blocking)
4. Push tag `v0.1.0` to trigger the release CI

Remaining concern: GHCR image must be accessible during `make` (either public, or `docker login ghcr.io` before `make`). Wave context confirms the image is public per Plan 01.

## Self-Check: PASSED

Verified:
- `/home/anon/bed-startos/startos/manifest/index.ts` — exists, contains sha256 digest
- `/home/anon/bed-startos/startos/main.ts` — exists, contains checkPortListening + /data/encrypted
- `/home/anon/bed-startos/startos/interfaces.ts` — exists, contains bindPort
- `/home/anon/bed-startos/startos/versions/v0.1.0.1.ts` — exists, version '0.1.0:1'
- `/home/anon/bed-startos/icon.png` — exists, 1024x1024
- `/home/anon/bed-startos/instructions.md` — exists, golden rule present
- `/home/anon/bed-startos/README.md` — exists, 120 lines, golden rule x2
- `/home/anon/bed-startos/.github/workflows/release.yml` — exists, valid YAML
- Commits 931f5c5, 87f2d5f, 9663059, 004301d, 0b558d8, f4f64a8 — all in git log
- All commits use email `55397917+4rkad@users.noreply.github.com`
- `tsc --noEmit` → exit 0 (zero errors)
- `prettier --check 'startos/**/*.ts'` → exit 0
