---
phase: 04-startos-packaging-docs
plan: 02
subsystem: docs
tags: [readme, threat-model, bitcoin, bed, aes-256-gcm, bip-1951, liana]

# Dependency graph
requires:
  - phase: 01-crypto-core-http-api
    provides: Verified crypto properties (AES-256-GCM, magic BEB, crate v0.0.2 rev cd7ee382, multipath wildcard requirement)
  - phase: 02-spa-frontend-history
    provides: Verified UX details (QR ECC-L limit, history mode opt-in, armored header text)
provides:
  - "descriptor-cifrado/README.md — canonical English documentation with threat model and golden rule"
  - "descriptor-cifrado/LICENSE — MIT with Semilla Bitcoin copyright"
affects:
  - 04-startos-packaging-docs (plan 04 bed-startos README can link to #threat-model and #crypto-details anchors)
  - github.com/semillabitcoin/descriptor-cifrado public page

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "README structure: TL;DR → Usage → Threat Model (3 sub-sections) → Crypto Details → Common Pitfalls → References"
    - "Golden rule appears twice: once in TL;DR blockquote, once in Threat Model blockquote — grep-verifiable"

key-files:
  created:
    - README.md
    - LICENSE
  modified: []

key-decisions:
  - "Golden rule phrasing: 'never co-locate' (lowercase) embedded mid-sentence in both blockquotes to satisfy case-sensitive grep test"
  - "LICENSE created with Semilla Bitcoin copyright — README linked to it and file was missing from repo"
  - "No screenshots section in Usage v1 — textual step-by-step per RESEARCH.md deferral note"

patterns-established:
  - "bed-startos README (plan 04) can safely link to #threat-model and #crypto-details anchors from this file"

requirements-completed: [DOC-01, DOC-02]

# Metrics
duration: 3min
completed: 2026-05-07
---

# Phase 4 Plan 02: descriptor-cifrado README Summary

**English README with explicit threat model (3 sub-sections), golden rule 'never co-locate' appearing twice, AES-256-GCM / BIP PR #1951 / crate v0.0.2 crypto details, and MIT LICENSE**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-07T16:06:32Z
- **Completed:** 2026-05-07T16:09:29Z
- **Tasks:** 1 of 1
- **Files modified:** 2 (README.md created, LICENSE created)

## Accomplishments

- Created `README.md` at repo root — 161 lines, 6 sections, English
- Threat Model section with 3 sub-sections satisfying DOC-01: What BED protects / What BED does NOT protect against / Model assumptions
- Golden rule "never co-locate" appears twice (lines 22 and 76) — `grep -c 'never co-locate' README.md` returns `2` (DOC-02 / D-03)
- Crypto Details table: AES-256-GCM, magic BEB, crate pinned at v0.0.2 rev cd7ee382, BIP PR #1951, Liana v13+ interop
- MIT LICENSE created (file was absent; README links to it)

## Task Commits

1. **Task 1: Write README.md (6 sections, English, DOC-01 + DOC-02)** - `d66839b` (docs)

## Files Created/Modified

- `README.md` — 161-line canonical project documentation in English
- `LICENSE` — MIT license with Semilla Bitcoin copyright

## Decisions Made

- **Golden rule phrasing:** The acceptance criteria uses `grep -c 'never co-locate'` (lowercase, case-sensitive). The plan's literal blockquote text started with capital "Never" (sentence opener). Fixed by embedding "never co-locate" mid-sentence in each blockquote: "The cardinal rule of BED — never co-locate..." and "...requires that you never co-locate...". Both blockquotes remain clear and idiomatic English.
- **LICENSE:** File did not exist at repo root. Plan required creating it if missing. Created with standard MIT text, "Semilla Bitcoin" as copyright holder, year 2026.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed golden rule grep test — lowercase 'never co-locate' required**
- **Found during:** Task 1 (Write README.md) — post-write verification
- **Issue:** Plan's literal blockquote text used "Never co-locate" (capital N at sentence start); the acceptance criteria `grep -c 'never co-locate'` is case-sensitive and requires lowercase `never`. The plan's own literal text would have failed its own verification test.
- **Fix:** Restructured both golden rule blockquotes to embed "never co-locate" mid-sentence (not as sentence opener), satisfying the case-sensitive grep while preserving clear English phrasing.
- **Files modified:** README.md
- **Verification:** `grep -c 'never co-locate' README.md` → `2`
- **Committed in:** d66839b

---

**Total deviations:** 1 auto-fixed (Rule 1 — literal text in plan would have failed its own acceptance test)
**Impact on plan:** Essential for DOC-02 compliance. No scope creep.

## Issues Encountered

None beyond the golden rule phrasing fix above.

## User Setup Required

None.

## Known Stubs

None. README references all real, verified technical details. No placeholder text.

## Next Phase Readiness

- Plan 04 (bed-startos README) can safely link to `#threat-model` and `#crypto-details` anchors from this README.
- `github.com/semillabitcoin/descriptor-cifrado` will show a complete README on next push.

## Self-Check: PASSED

- FOUND: README.md (161 lines)
- FOUND: LICENSE
- FOUND: commit d66839b
- `grep -c 'never co-locate' README.md` → 2
- All acceptance criteria verified

---
*Phase: 04-startos-packaging-docs*
*Completed: 2026-05-07*
