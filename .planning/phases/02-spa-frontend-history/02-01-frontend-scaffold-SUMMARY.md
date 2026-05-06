---
phase: 02-spa-frontend-history
plan: 01
subsystem: frontend
tags: [frontend, svelte, vite, scaffold, fonts, tokens, ui-spec]
requires:
  - bed-server HTTP API on 127.0.0.1:8080 (used by Vite dev proxy)
provides:
  - frontend/ Svelte 5 + Vite 8 scaffold ready for plans 02-03..06
  - frontend/src/lib/tokens.css — single source of truth for palette/spacing/radii/shadows
  - frontend/src/app.css — @font-face + minimal reset
  - frontend/src/assets/fonts/{Inter,JetBrainsMono}.woff2 — self-hosted variable woff2
affects:
  - .gitignore (root): adds frontend/{node_modules,dist,.vite}
tech-stack:
  added:
    - svelte@5.55.5 (devDependency)
    - vite@8.0.10 (devDependency)
    - "@sveltejs/vite-plugin-svelte@7.1.1 (devDependency)"
  patterns:
    - "Svelte 5 mount() API (no `new App()`)"
    - "Vite assetsInlineLimit callback excluding woff2 from base64 inlining"
    - "Vite assetFileNames routing woff2 → assets/fonts/[name]-[hash].woff2"
    - "@font-face with format('woff2-variations') + format('woff2') fallback"
    - "Light/Dark/Auto theme via [data-theme] + prefers-color-scheme media query"
key-files:
  created:
    - frontend/package.json
    - frontend/package-lock.json
    - frontend/.gitignore
    - frontend/vite.config.js
    - frontend/index.html
    - frontend/src/main.js
    - frontend/src/App.svelte
    - frontend/src/app.css
    - frontend/src/lib/tokens.css
    - frontend/src/assets/fonts/Inter.woff2
    - frontend/src/assets/fonts/JetBrainsMono.woff2
    - frontend/src/assets/fonts/LICENSE-Inter.txt
    - frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt
  modified:
    - .gitignore (root) — added frontend/{node_modules,dist,.vite}
decisions:
  - "JetBrains Mono variable woff2 fetched from `fonts/webfonts/JetBrainsMono[wght].woff2` because plan-specified `fonts/variable/*.woff2` returns 404 upstream (path renamed in repo)"
  - "Vite 8.0.10 / Svelte 5.55.5 / @sveltejs/vite-plugin-svelte 7.1.1 resolved by ^X.0.0 ranges in package.json"
metrics:
  duration_minutes: 3
  tasks_completed: 3
  commits: 3
  files_created: 13
  files_modified: 1
  completed: 2026-05-06
---

# Phase 2 Plan 01: Frontend Scaffold Summary

Svelte 5 + Vite 8 SPA scaffold in `frontend/` with self-hosted Inter + JetBrains Mono variable woff2, full UI-SPEC color/spacing tokens (light/dark/auto), and a build pipeline that produces `frontend/dist/` with zero external URLs (UI-01 verified).

## Outcome

`cd frontend && npm install && npm run build` produces a self-contained `dist/` with:
- `dist/index.html` — 0.41 KB (no external URL)
- `dist/assets/style-*.css` — 4.39 KB (1.36 KB gzipped)
- `dist/assets/index-*.js` — 24.19 KB (9.80 KB gzipped)
- `dist/assets/fonts/Inter-*.woff2` — 352.24 KB
- `dist/assets/fonts/JetBrainsMono-*.woff2` — 113.67 KB

JS+CSS gzipped (excluding fonts) = **11.16 KB**, well under the 50 KB budget. Zero references to `fonts.googleapis.com`, `fonts.gstatic.com`, `cdn.jsdelivr`, or `unpkg.com`. Plans 02-03..06 will build on this scaffold.

## Tooling versions resolved

| Package                          | Range  | Resolved |
| -------------------------------- | ------ | -------- |
| svelte                           | ^5.0.0 | 5.55.5   |
| vite                             | ^8.0.0 | 8.0.10   |
| @sveltejs/vite-plugin-svelte     | ^7.0.0 | 7.1.1    |

39 packages installed total; package-lock.json committed.

## Font sizes (source assets)

| File                                                | Size (bytes) | Source                                                              |
| --------------------------------------------------- | ------------ | ------------------------------------------------------------------- |
| frontend/src/assets/fonts/Inter.woff2               | 352,240      | https://rsms.me/inter/font-files/InterVariable.woff2                |
| frontend/src/assets/fonts/JetBrainsMono.woff2       | 113,672      | https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/webfonts/JetBrainsMono%5Bwght%5D.woff2 |
| frontend/src/assets/fonts/LICENSE-Inter.txt         | 4,380        | rsms/inter LICENSE.txt (OFL)                                        |
| frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt | 4,399        | JetBrains/JetBrainsMono OFL.txt                                     |

Both woff2 files verified to start with `wOF2` magic bytes (not HTML error pages).

## Reference paths for downstream plans (03–06)

- Tokens: `/workspace/descriptor-cifrado/frontend/src/lib/tokens.css`
- App stylesheet (with @font-face + reset): `/workspace/descriptor-cifrado/frontend/src/app.css`
- App entrypoint (Svelte 5 mount): `/workspace/descriptor-cifrado/frontend/src/main.js`
- App root component (placeholder, to be replaced by plan 02-03): `/workspace/descriptor-cifrado/frontend/src/App.svelte`
- Vite config (with /api → 127.0.0.1:8080 proxy): `/workspace/descriptor-cifrado/frontend/vite.config.js`

## Commits

| Task | Hash    | Message                                                            |
| ---- | ------- | ------------------------------------------------------------------ |
| 1    | 5993097 | feat(02-01): scaffold Svelte 5 + Vite frontend project             |
| 2    | 42bb769 | feat(02-01): add Inter and JetBrains Mono variable woff2 self-hosted |
| 3    | 53c0a62 | feat(02-01): add tokens.css palette + app.css with self-hosted @font-face |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] JetBrains Mono variable woff2 URL changed upstream**
- **Found during:** Task 2
- **Issue:** Plan-specified URL `https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/variable/JetBrainsMono%5Bwght%5D.woff2` returned 404. The release zip (v2.304) ships only `.ttf` variable, not `.woff2`. Listing the repo via the GitHub API showed the variable woff2 lives in `fonts/webfonts/`, not `fonts/variable/`.
- **Fix:** Used `https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/webfonts/JetBrainsMono%5Bwght%5D.woff2`. File downloaded successfully (113,672 bytes), verified with `wOF2` magic bytes, OFL license downloaded separately.
- **Files modified:** `frontend/src/assets/fonts/JetBrainsMono.woff2`
- **Commit:** 42bb769

## Verification (UI-01 zero external URLs)

```text
$ grep -rE "fonts\.googleapis|fonts\.gstatic|cdn\.jsdelivr|unpkg\.com" frontend/dist/
(no matches)

$ grep -E "https?://" frontend/dist/index.html
(no matches)

$ find frontend/dist/assets -name "*.woff2" | wc -l
2
```

## Self-Check: PASSED
- frontend/package.json: FOUND
- frontend/vite.config.js: FOUND
- frontend/index.html: FOUND
- frontend/src/main.js: FOUND
- frontend/src/App.svelte: FOUND
- frontend/src/app.css: FOUND
- frontend/src/lib/tokens.css: FOUND
- frontend/src/assets/fonts/Inter.woff2: FOUND
- frontend/src/assets/fonts/JetBrainsMono.woff2: FOUND
- frontend/src/assets/fonts/LICENSE-Inter.txt: FOUND
- frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt: FOUND
- frontend/dist/index.html: FOUND (post-build)
- commit 5993097: FOUND
- commit 42bb769: FOUND
- commit 53c0a62: FOUND
