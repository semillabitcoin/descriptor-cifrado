---
phase: 04-startos-packaging-docs
plan: "03"
title: "Bootstrap bed-startos repo from hello-world-startos"
subsystem: startos-packaging
tags: [startos, s9pk, scaffold, github, sdk, typescript]
dependency_graph:
  requires: []
  provides: [bed-startos-scaffold, bed-startos-github-repo]
  affects: [04-04-manifest-main-interfaces, 04-05-ci-release]
tech_stack:
  added:
    - "@start9labs/start-sdk@1.4.1"
    - "@vercel/ncc@^0.38.4"
    - "typescript@^5.9.3"
    - "prettier@^3.6.2"
  patterns:
    - hello-world-startos template scaffold (cloned from master, update/040 merged)
    - noreply email for all git commits
    - private GitHub repo on semillabitcoin org
key_files:
  created:
    - /home/anon/bed-startos/ (full new sibling repo)
    - /home/anon/bed-startos/package.json
    - /home/anon/bed-startos/startos/manifest/index.ts
    - /home/anon/bed-startos/startos/manifest/i18n.ts
    - /home/anon/bed-startos/startos/main.ts
    - /home/anon/bed-startos/startos/interfaces.ts
    - /home/anon/bed-startos/startos/i18n/dictionaries/default.ts
    - /home/anon/bed-startos/startos/i18n/dictionaries/translations.ts
    - /home/anon/bed-startos/README.md (stub)
    - /home/anon/bed-startos/CLAUDE.md
  modified: []
decisions:
  - "Used hello-world-startos master branch (update/040 was merged into master by 2026-05-07)"
  - "marketingUrl must be string in SDK 1.4.1 (not null) — set to GitHub repo URL"
  - "image key renamed to 'main' (not 'hello-world') matching RESEARCH.md Pattern 1"
  - "mountpoint set to /data/encrypted (matching BED_DATA_DIR default in bed-server binary)"
metrics:
  duration: 14
  completed: "2026-05-07T16:20:54Z"
  tasks: 4
  files: 33
---

# Phase 4 Plan 03: Bootstrap bed-startos repo from hello-world-startos

**One-liner:** Scaffolded `semillabitcoin/bed-startos` PRIVATE repo from `hello-world-startos` master with SDK 1.4.1, scrubbed all hello-world identity, and pushed 2 commits to GitHub.

## Summary

Plan 03 created the `bed-startos` sibling repo at `/home/anon/bed-startos` (sibling of `descriptor-cifrado`), cloned from `Start9Labs/hello-world-startos` master branch (which contains the `update/040` code post-merge). All hello-world identity strings were scrubbed from app-editable files. `@start9labs/start-sdk` was pinned to `1.4.1` (latest on npm). The GitHub repo `semillabitcoin/bed-startos` was created as PRIVATE per D-12, and both commits were pushed to main. Plan 04 (Wave 2) can now edit the manifest, write main/interfaces/backups content, and wire up CI without infrastructure setup.

## /workspace/bed-startos Directory Tree (top 2 levels)

Note: the actual path on this machine is `/home/anon/bed-startos` (sibling of `/home/anon/descriptor-cifrado`). The plan references `/workspace/bed-startos` but `/workspace` does not exist on this machine.

```
/home/anon/bed-startos/
├── assets/
│   └── README.md
├── CLAUDE.md
├── CONTRIBUTING.md
├── .dockerignore
├── icon.svg
├── LICENSE
├── Makefile                   # ARCHES := x86 arm; include s9pk.mk
├── package.json               # name: bed-startos; @start9labs/start-sdk@1.4.1
├── package-lock.json
├── README.md                  # stub — Plan 04 writes full README
├── s9pk.mk                    # plumbing (132 lines, copy verbatim)
├── startos/
│   ├── actions/
│   ├── backups.ts
│   ├── dependencies.ts
│   ├── fileModels/
│   ├── i18n/
│   │   └── dictionaries/
│   │       ├── default.ts     # scrubbed Hello World strings → BED
│   │       └── translations.ts
│   ├── index.ts               # PLUMBING — DO NOT EDIT
│   ├── init/
│   ├── interfaces.ts          # scrubbed description
│   ├── main.ts                # imageId: 'main', mountpoint: '/data/encrypted'
│   ├── manifest/
│   │   ├── i18n.ts            # placeholder strings for Plan 04
│   │   └── index.ts           # id: 'bed', packageRepo/upstreamRepo set
│   ├── sdk.ts                 # PLUMBING — DO NOT EDIT
│   ├── utils.ts
│   └── versions/
└── tsconfig.json              # PLUMBING — DO NOT EDIT
```

## Pinned SDK Version

`@start9labs/start-sdk@1.4.1` (exact pin, no caret) — verified against npm registry 2026-05-07.

## GitHub Repo

- **URL:** https://github.com/semillabitcoin/bed-startos
- **Visibility:** private (D-12)
- **Default branch:** main

## HEAD SHA of main on origin

`6167e49767667d401998622d25f8ec23e91ac2f9`

## Number of Commits in Bootstrap

2 commits:

| Hash | Message |
|------|---------|
| `020985a` | `init(bed-startos): bootstrap from hello-world-startos branch update/040` |
| `6167e49` | `deps(bed-startos): pin @start9labs/start-sdk@1.4.1 and resolve lockfile` |

Both commits use author email `55397917+4rkad@users.noreply.github.com` (D-13).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Branch update/040 not found — used master (which contains same code post-merge)**
- **Found during:** Task 1 (clone step)
- **Issue:** `git clone --branch update/040 https://github.com/Start9Labs/hello-world-startos.git` failed with "Remote branch update/040 not found". Only `master` branch exists on the upstream.
- **Investigation:** `git ls-remote --heads` shows only `refs/heads/master`. Inspection of master shows SDK 1.3.3 (1.x = StartOS 0.4.0 SDK), same TypeScript structure, same `ARCHES := x86 arm` Makefile, same `startos/` layout as expected from `update/040`. Conclusion: `update/040` was merged into master.
- **Fix:** Cloned from `master` branch. The plan's RESEARCH.md warning "master is StartOS 0.3.x" was outdated — master IS now the 0.4.0 template.
- **Files modified:** N/A (clone source)
- **Commit:** `020985a`

**2. [Rule 1 - Bug] `marketingUrl: null` type error — SDK 1.4.1 requires `string`**
- **Found during:** Task 2 (type check step)
- **Issue:** `tsc --noEmit` reported `Type 'null' is not assignable to type 'string'` for `marketingUrl`. SDK type definition: `marketingUrl: string` (non-nullable), `donationUrl: string | null`.
- **Fix:** Set `marketingUrl: 'https://github.com/semillabitcoin/bed-startos'`. The RESEARCH.md Pattern 1 had `marketingUrl: null` which is incorrect for SDK 1.4.1.
- **Files modified:** `startos/manifest/index.ts`
- **Commit:** `6167e49`

### Checkpoint Auto-Approval

Task 4 (`checkpoint:human-verify`) was auto-approved per `auto_advance: true` config. All 5 verification commands passed:
1. `ls /home/anon/bed-startos | sort` — includes Makefile, package.json, s9pk.mk, startos, tsconfig.json
2. SDK version is `1.4.1`
3. GitHub repo is `private` with default_branch `main`
4. 2 commits, both with noreply email `55397917+4rkad@users.noreply.github.com`
5. No `hello-world|Hello World` matches in package.json or startos/manifest/

## Known Stubs

The following placeholders are intentional — Plan 04 will replace them:

| File | Stub | Reason |
|------|------|--------|
| `startos/manifest/index.ts` | `dockerTag: 'ghcr.io/semillabitcoin/descriptor-cifrado@sha256:PLACEHOLDER_DIGEST_SET_BY_PLAN_04'` | Digest requires v0.1.0 tag on descriptor-cifrado (Plan 01 must run first) |
| `startos/manifest/i18n.ts` | `short`/`long` = "BED — placeholder, edited by Plan 04" | Final D-04 identity strings written in Plan 04 |
| `README.md` | Single stub line | Full 6-section README written in Plan 04 |

These stubs do NOT prevent the plan's goal (scaffold creation and GitHub push) from being achieved. Plan 04 resolves all stubs.

## Self-Check: PASSED

All artifacts verified:
- `/home/anon/bed-startos/.git` — exists
- `/home/anon/bed-startos/package.json` — exists, `@start9labs/start-sdk@1.4.1`
- `/home/anon/bed-startos/Makefile` — exists, `ARCHES := x86 arm`
- `/home/anon/bed-startos/s9pk.mk` — exists, 132 lines
- `/home/anon/bed-startos/startos/index.ts` — exists (plumbing preserved)
- Commit `020985a` — exists in git log
- Commit `6167e49` — exists in git log
- `semillabitcoin/bed-startos` — exists on GitHub, visibility: private
- SUMMARY.md — created at `.planning/phases/04-startos-packaging-docs/04-03-SUMMARY.md`
