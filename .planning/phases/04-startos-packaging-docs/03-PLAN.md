---
id: 04-03
phase: 04-startos-packaging-docs
plan: 03
title: "Bootstrap bed-startos repo from hello-world-startos branch update/040"
type: execute
wave: 1
depends_on: []
files_modified:
  - "../bed-startos/ (new sibling repo — full directory)"
autonomous: false
model: sonnet
requirements: [S9-01]
gap_closure: false

must_haves:
  truths:
    - "Local sibling directory ../bed-startos exists with the hello-world-startos branch update/040 scaffold cloned in"
    - "GitHub repo semillabitcoin/bed-startos exists, is PRIVATE (D-12), and has the bootstrap commit pushed to main"
    - "package.json declares @start9labs/start-sdk@1.4.1 (RESEARCH.md confirmed npm latest)"
    - "All clones-of-template strings (id 'hello-world', titles, descriptions, repo URLs) are scrubbed and replaced with bed-startos identity"
    - "The first commit author email is 55397917+4rkad@users.noreply.github.com (D-13)"
  artifacts:
    - path: "../bed-startos/package.json"
      provides: "npm package definition for bed-startos with start-sdk@1.4.1 dependency"
      contains: "@start9labs/start-sdk"
    - path: "../bed-startos/Makefile"
      provides: "Build orchestration; ARCHES := x86 arm; includes s9pk.mk"
      contains: "ARCHES"
    - path: "../bed-startos/s9pk.mk"
      provides: "Copy-verbatim plumbing from hello-world-startos branch update/040"
      min_lines: 20
    - path: "../bed-startos/startos/index.ts"
      provides: "Plumbing entrypoint — DO NOT EDIT (per RESEARCH.md)"
      min_lines: 5
    - path: "../bed-startos/.git"
      provides: "Initialized git repo with first commit"
  key_links:
    - from: "Local ../bed-startos"
      to: "github.com/semillabitcoin/bed-startos (PRIVATE)"
      via: "git remote add origin + git push -u"
      pattern: "github.com[:/]semillabitcoin/bed-startos"
---

<objective>
Bootstrap the bed-startos repo as a sibling directory of descriptor-cifrado (path: `/workspace/bed-startos`), scaffolded from `Start9Labs/hello-world-startos` branch `update/040` per S9-01 and D-12. Scrub all hello-world references, install SDK 1.4.1, initialize git history with the noreply email (D-13), create the GitHub repo as PRIVATE on the `semillabitcoin` org, and push the first commit. Subsequent plans (Plan 04) edit manifest/main/interfaces/README/icon/LICENSE/CI inside this scaffold; this plan ONLY produces the empty-but-functional skeleton.

Purpose: Phase 4 lives in a separate repo (D-12). All Wave 2 manifest/icon/CI work must be done against an existing scaffolded checkout. Splitting this off as a Wave 1 plan lets it run in parallel with the descriptor-cifrado tag work (Plan 01) and the README work (Plan 02).

Output: Functioning local sibling repo `/workspace/bed-startos`, private remote on GitHub, package.json + node_modules ready for `make` (verified by `npm run check` succeeding even with the placeholder hello-world content — Plan 04 then overwrites the per-app TypeScript files).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/04-startos-packaging-docs/04-CONTEXT.md
@.planning/phases/04-startos-packaging-docs/04-RESEARCH.md
@$HOME/.claude/skills/start9-packaging/SKILL.md
</context>

<tasks>

<task id="03-01-clone-and-scrub" type="auto">
  <name>Task 1: Clone hello-world-startos branch update/040 to ../bed-startos and scrub template identity</name>
  <read_first>
    - $HOME/.claude/skills/start9-packaging/SKILL.md (canonical reference for s9pk anatomy — confirms what files to scrub vs what to leave verbatim)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Recommended Project Structure" (file inventory: which files are PLUMBING DO NOT EDIT vs which are app-editable)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-12, D-13 (private repo, noreply email)
  </read_first>
  <files>
    - /workspace/bed-startos/ (full new directory, sibling of descriptor-cifrado)
  </files>
  <action>
    1. Verify the sibling parent directory `/workspace/` is writable. Confirm `/workspace/bed-startos` does NOT already exist:
       ```
       test ! -e /workspace/bed-startos
       ```
       If it exists with content, STOP and ask the user whether to overwrite or use a suffix (e.g., `bed-startos-new`).
    2. Clone the canonical 0.4.0 template:
       ```
       git clone --branch update/040 --depth 1 \
         https://github.com/Start9Labs/hello-world-startos.git \
         /workspace/bed-startos
       ```
       If `--branch update/040` fails with `Remote branch update/040 not found`, surface this immediately — RESEARCH.md confirms the branch exists, so a failure means the upstream renamed it. Do NOT silently fall back to master (master is StartOS 0.3.x per RESEARCH.md "State of the Art").
    3. Inside `/workspace/bed-startos`, remove the upstream `.git` directory and re-initialize:
       ```
       cd /workspace/bed-startos
       rm -rf .git
       git init -b main
       git config user.email "55397917+4rkad@users.noreply.github.com"
       git config user.name "4rkad"
       ```
    4. Identify the upstream identity strings to scrub. Run:
       ```
       grep -rn 'hello-world\|hello_world\|HelloWorld\|Hello World' /workspace/bed-startos --include='*.ts' --include='*.json' --include='*.md' --include='Makefile' --include='*.mk' 2>/dev/null
       ```
       Expected matches (per RESEARCH.md and skill): `package.json` `name`, `manifest/index.ts` `id`/`title`/`packageRepo`/`upstreamRepo`/`docsUrls`, `manifest/i18n.ts` `short`/`long`, `README.md`, possibly `actions/` and `versions/` filenames.
    5. Apply the following replacements VERBATIM. These are placeholders for Plan 04 to overwrite — Plan 03 only produces a working scaffold:
       - `package.json` `"name": "hello-world-startos"` → `"name": "bed-startos"`
       - `package.json` `"description"` → `"BED — Bitcoin Encrypted Backup s9pk wrapper"`
       - `manifest/index.ts` `id: 'hello-world'` → `id: 'bed'`
       - `manifest/index.ts` `title: 'Hello World'` → `title: 'BED'`
       - `manifest/index.ts` `packageRepo: '...'` → `packageRepo: 'https://github.com/semillabitcoin/bed-startos'`
       - `manifest/index.ts` `upstreamRepo: '...'` → `upstreamRepo: 'https://github.com/semillabitcoin/descriptor-cifrado'`
       - `manifest/index.ts` `docsUrls: [...]` → `docsUrls: ['https://github.com/semillabitcoin/bed-startos#readme']`
       - `manifest/index.ts` `license: 'MIT'` (leave or set to `'MIT'`)
       - `manifest/i18n.ts` `short`/`long` keys → placeholder strings `"BED — placeholder, edited by Plan 04"` (Plan 04 will write the final D-04 strings)
       - DELETE the upstream `README.md` (Plan 04 writes a new one in Wave 2). Leave a stub with one line: `# BED — Bitcoin Encrypted Backup (s9pk wrapper). README written in Plan 04.`
    6. DO NOT modify these files (RESEARCH.md "PLUMBING — DO NOT EDIT"):
       - `startos/index.ts`, `startos/sdk.ts`, `startos/init/index.ts`, `startos/i18n/index.ts`
       - `s9pk.mk` (copy verbatim — never edit)
       - `tsconfig.json` (copy verbatim — Pitfall 6 in RESEARCH.md)
    7. Verify Makefile has `ARCHES := x86 arm` (per Pattern 6 in RESEARCH.md). If the upstream Makefile uses different arch names or a different format, edit the Makefile to:
       ```
       ARCHES := x86 arm
       include s9pk.mk
       ```
    8. After scrubbing, re-run the grep from step 4. Output MUST be empty for `hello-world` / `Hello World` matches in `*.ts`, `*.json`, `Makefile` (it's OK for `s9pk.mk` to retain template comments; do not edit s9pk.mk).
    9. Stage everything and create the first commit:
       ```
       git add -A
       git -c user.email="55397917+4rkad@users.noreply.github.com" \
         commit -m "init(bed-startos): bootstrap from hello-world-startos branch update/040

       Cloned Start9Labs/hello-world-startos@update/040 (StartOS 0.4.0
       canonical template). Scrubbed hello-world identity strings; replaced
       with bed/BED placeholders. Plumbing files (startos/index.ts,
       startos/sdk.ts, s9pk.mk, tsconfig.json) untouched per skill canonical
       reference.

       Plan 04 in Wave 2 will overwrite manifest/i18n.ts strings, write
       main.ts/interfaces.ts/backups.ts content, generate the icon, write
       LICENSE + README, and add the CI workflow."
       ```
    10. Verify the commit author email is the noreply identity:
        ```
        git log -1 --format='%ae' | grep -q '^55397917+4rkad@users.noreply.github.com$'
        ```
        If wrong, amend with the correct env-overridden email (do NOT change git config globally).
  </action>
  <verify>
    <automated>test -d /workspace/bed-startos/.git &amp;&amp; test -f /workspace/bed-startos/package.json &amp;&amp; test -f /workspace/bed-startos/Makefile &amp;&amp; test -f /workspace/bed-startos/s9pk.mk &amp;&amp; test -f /workspace/bed-startos/startos/index.ts &amp;&amp; ! grep -rIq 'hello-world\|Hello World' /workspace/bed-startos/package.json /workspace/bed-startos/startos/manifest/ &amp;&amp; cd /workspace/bed-startos &amp;&amp; git log -1 --format='%ae' | grep -q '^55397917+4rkad@users.noreply.github.com$'</automated>
  </verify>
  <acceptance_criteria>
    - Directory `/workspace/bed-startos` exists.
    - `/workspace/bed-startos/.git` exists (initialized git repo).
    - `/workspace/bed-startos/package.json` exists and contains `"name": "bed-startos"`.
    - `/workspace/bed-startos/package.json` contains `"@start9labs/start-sdk"` (any version — Task 2 pins to 1.4.1).
    - `/workspace/bed-startos/Makefile` exists and contains `ARCHES := x86 arm` and `include s9pk.mk`.
    - `/workspace/bed-startos/s9pk.mk` exists with at least 20 lines.
    - `/workspace/bed-startos/startos/index.ts`, `startos/sdk.ts`, `tsconfig.json` exist (plumbing files preserved).
    - `grep -rI 'hello-world\|Hello World' /workspace/bed-startos/package.json /workspace/bed-startos/startos/manifest/` returns no matches (scrub complete in app-editable surfaces).
    - `cd /workspace/bed-startos && git log -1 --format='%ae'` outputs `55397917+4rkad@users.noreply.github.com`.
    - `cd /workspace/bed-startos && git log --oneline | wc -l` outputs `1` (single bootstrap commit).
  </acceptance_criteria>
  <done>Scrubbed hello-world-startos scaffold lives at /workspace/bed-startos with a single bootstrap commit. Plan 04 can edit manifest/main/interfaces/etc. without dealing with template identity collisions.</done>
</task>

<task id="03-02-pin-sdk-and-install" type="auto">
  <name>Task 2: Pin @start9labs/start-sdk@1.4.1 and run npm ci to populate node_modules</name>
  <read_first>
    - /workspace/bed-startos/package.json (current SDK version pin from upstream — likely an older minor; verify before bumping)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Standard Stack" (npm view @start9labs/start-sdk version → 1.4.1 confirmed live 2026-05-07)
  </read_first>
  <files>
    - /workspace/bed-startos/package.json
    - /workspace/bed-startos/package-lock.json
    - /workspace/bed-startos/node_modules/ (generated)
  </files>
  <action>
    1. From `/workspace/bed-startos`, verify the live npm registry version BEFORE pinning:
       ```
       npm view @start9labs/start-sdk version
       ```
       Expected output per RESEARCH.md: `1.4.1`. If npm reports a higher minor (1.4.2+) within the same minor line, prefer the higher version (RESEARCH.md valid-until is 2026-06-07; bumps within 1.4.x are safe). If a different MAJOR (2.x+) appears, STOP and surface — that may break the manifest API.
    2. Bump `package.json` `dependencies."@start9labs/start-sdk"` to the version from step 1 (string literal, no caret prefix to keep the pin reproducible — caret is acceptable since the SDK follows semver, but RESEARCH.md prefers explicit `1.4.1`):
       ```
       cd /workspace/bed-startos
       npm install --save-exact @start9labs/start-sdk@&lt;version-from-step-1&gt;
       ```
    3. Verify dev-deps recommended by RESEARCH.md exist (these may already be in the upstream package.json — only add if missing):
       - `@vercel/ncc` (RESEARCH.md: `^0.38.x`)
       - `typescript` (`^5.x`)
       - `prettier` (`^3.x`)
       - `@types/node`
       Run `npm install --save-dev --save-exact` for any that are missing.
    4. Run `npm ci` to fully resolve and lock:
       ```
       cd /workspace/bed-startos
       rm -rf node_modules package-lock.json
       npm install
       ```
       Use `npm install` (not `npm ci`) on the first run to generate the lockfile from scratch. After this, switch convention to `npm ci`.
    5. Run the type check that `make` would run, to confirm the placeholder scaffold still compiles:
       ```
       cd /workspace/bed-startos
       npm run check 2>&1 | tee /tmp/bed-startos-tsc.log
       ```
       If `npm run check` does not exist as a script (depends on hello-world-startos package.json), run `npx tsc --noEmit` directly.
       The check is allowed to FAIL with content errors (the placeholder strings may not satisfy SDK 1.4.1 typings if upstream pinned an older version) but MUST NOT fail with `Cannot find module '@start9labs/start-sdk'` — that signals a broken install.
    6. Stage and commit:
       ```
       cd /workspace/bed-startos
       git add package.json package-lock.json
       # do not commit node_modules — confirm .gitignore excludes it
       grep -q '^node_modules' .gitignore || echo "node_modules/" >> .gitignore
       git add .gitignore
       git -c user.email="55397917+4rkad@users.noreply.github.com" \
         commit -m "deps(bed-startos): pin @start9labs/start-sdk@&lt;version&gt; and resolve lockfile"
       ```
  </action>
  <verify>
    <automated>cd /workspace/bed-startos &amp;&amp; grep -q '@start9labs/start-sdk' package.json &amp;&amp; test -f package-lock.json &amp;&amp; test -d node_modules/@start9labs/start-sdk</automated>
  </verify>
  <acceptance_criteria>
    - `cat /workspace/bed-startos/package.json | jq -r '.dependencies."@start9labs/start-sdk"'` outputs a non-null version string matching `^1\.[0-9]+\.[0-9]+$` (≥ 1.4.1, < 2.0.0).
    - `/workspace/bed-startos/package-lock.json` exists and is non-empty.
    - `/workspace/bed-startos/node_modules/@start9labs/start-sdk/` directory exists.
    - `cat /workspace/bed-startos/.gitignore | grep -q '^node_modules'` succeeds (lockfile committed, modules excluded).
    - `cd /workspace/bed-startos && git log --oneline | wc -l` outputs `2` (bootstrap + sdk-pin commits).
  </acceptance_criteria>
  <done>SDK 1.4.1 (or newer 1.4.x) is pinned, lockfile resolved, node_modules populated. Plan 04 can write manifest.ts code that imports from `@start9labs/start-sdk` without npm-install steps.</done>
</task>

<task id="03-03-create-private-github-repo-and-push" type="auto">
  <name>Task 3: Create private semillabitcoin/bed-startos GitHub repo and push main</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-12 (PRIVATE initially; flip to public when both repos sanitized)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Common Pitfalls" Pitfall 1 (CI auth — bed-startos GHA may need GHCR read:packages PAT; not a Plan 03 concern but documents why repo exists in same org)
  </read_first>
  <files>
    - (no local file modifications — GitHub remote operation only)
  </files>
  <action>
    1. Verify gh CLI is authenticated to an account with create-repo rights on the `semillabitcoin` org:
       ```
       gh auth status
       gh api /orgs/semillabitcoin --jq '.login' | grep -q '^semillabitcoin$'
       ```
    2. Check whether the repo already exists (idempotency):
       ```
       gh api /repos/semillabitcoin/bed-startos --jq '.full_name' 2>/dev/null
       ```
       If the response is `semillabitcoin/bed-startos`, skip to step 4. If `Not Found`, proceed to step 3.
    3. Create the repo as PRIVATE (D-12). Use the `gh` CLI:
       ```
       gh repo create semillabitcoin/bed-startos \
         --private \
         --description "StartOS 0.4.0 s9pk wrapper for BED — Bitcoin Encrypted Backup. Image source: ghcr.io/semillabitcoin/descriptor-cifrado." \
         --homepage "https://github.com/semillabitcoin/descriptor-cifrado" \
         --disable-issues=false \
         --disable-wiki=true
       ```
       Confirm visibility:
       ```
       gh api /repos/semillabitcoin/bed-startos --jq '.visibility'
       ```
       MUST output `private`.
    4. Add the remote and push the bootstrap commits:
       ```
       cd /workspace/bed-startos
       git remote add origin git@github.com:semillabitcoin/bed-startos.git \
         || git remote set-url origin git@github.com:semillabitcoin/bed-startos.git
       git branch -M main
       git push -u origin main
       ```
       If SSH push fails (no key on this account), fall back to HTTPS:
       ```
       git remote set-url origin https://github.com/semillabitcoin/bed-startos.git
       git push -u origin main
       ```
    5. Verify the push succeeded:
       ```
       gh api /repos/semillabitcoin/bed-startos/commits/main --jq '.sha'
       ```
       Output is the local HEAD commit SHA. Verify with:
       ```
       cd /workspace/bed-startos && git rev-parse HEAD
       ```
       The two SHAs MUST match.
  </action>
  <verify>
    <automated>gh api /repos/semillabitcoin/bed-startos --jq '.visibility' | grep -q '^private$' &amp;&amp; cd /workspace/bed-startos &amp;&amp; git ls-remote origin main | awk '{print $1}' | grep -q "$(git rev-parse HEAD)"</automated>
  </verify>
  <acceptance_criteria>
    - `gh api /repos/semillabitcoin/bed-startos --jq '.full_name'` outputs `semillabitcoin/bed-startos`.
    - `gh api /repos/semillabitcoin/bed-startos --jq '.visibility'` outputs `private` (D-12).
    - `cd /workspace/bed-startos && git ls-remote origin main` returns the same SHA as `git rev-parse HEAD` (push verified).
    - `cd /workspace/bed-startos && git log --oneline origin/main` shows at least 2 commits (bootstrap + sdk-pin).
  </acceptance_criteria>
  <done>The bed-startos GitHub repo exists, is private, and has the local bootstrap commits pushed to main. Plan 04 can edit files, commit, and push to the same remote without infrastructure setup.</done>
</task>

<task id="03-04-checkpoint-confirm-scaffold" type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: Confirm scaffold — local repo + SDK + private GitHub repo</name>
  <action>Human runs the five verification commands listed under how-to-verify and confirms the bed-startos scaffold is ready for Plan 04 to author the manifest surface. The gate releases Plan 04 (Wave 2).</action>
  <what-built>
    - Local sibling repo /workspace/bed-startos cloned from hello-world-startos branch update/040
    - hello-world identity strings scrubbed in app-editable files (manifest, package.json)
    - SDK pinned to @start9labs/start-sdk@1.4.1 (or newer 1.4.x)
    - node_modules populated, type check baseline run
    - GitHub repo semillabitcoin/bed-startos created PRIVATE, main branch pushed
  </what-built>
  <how-to-verify>
    Run these five commands and paste output back:
    1. `ls /workspace/bed-startos | sort` — must include `Makefile`, `package.json`, `s9pk.mk`, `startos`, `tsconfig.json`
    2. `cat /workspace/bed-startos/package.json | jq -r '.dependencies."@start9labs/start-sdk"'` — must output `1.4.1` (or newer 1.4.x)
    3. `gh api /repos/semillabitcoin/bed-startos --jq '{visibility, default_branch}'` — must show `"private"` and `"main"`
    4. `cd /workspace/bed-startos && git log --oneline` — must show 2+ commits, all with the noreply email (verify via `git log --format='%ae'`)
    5. `grep -rI 'hello-world\|Hello World' /workspace/bed-startos/package.json /workspace/bed-startos/startos/manifest/ 2>&1` — must return zero matches

    If all five pass, Plan 04 has a working scaffold to edit. Approving means Wave 2 can start.
  </how-to-verify>
  <resume-signal>Type "approved" to release Plan 04 (Wave 2), or describe any failures.</resume-signal>
</task>

</tasks>

<verification>
- /workspace/bed-startos exists with correct files.
- @start9labs/start-sdk 1.4.x pinned, node_modules populated.
- semillabitcoin/bed-startos GitHub repo PRIVATE, main pushed.
- All commit author emails are the noreply identity.
- No hello-world strings in app-editable files.
</verification>

<success_criteria>
A subsequent agent can `cd /workspace/bed-startos && make clean x86` and the build will fail ONLY because the placeholder strings are not yet the final D-04 BED identity (Plan 04's job). The scaffold itself is well-formed: no broken imports, no missing files, no upstream identity leaks.
</success_criteria>

<output>
After completion, create `.planning/phases/04-startos-packaging-docs/04-03-SUMMARY.md` recording:
- /workspace/bed-startos directory tree (top 2 levels)
- Pinned SDK version (exact string)
- GitHub repo URL + visibility
- HEAD SHA of main on origin
- Number of commits in the bootstrap
</output>
