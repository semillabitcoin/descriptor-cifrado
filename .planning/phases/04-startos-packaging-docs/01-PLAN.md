---
id: 04-01
phase: 04-startos-packaging-docs
plan: 01
title: "Tag descriptor-cifrado v0.1.0, flip GHCR package public, capture image digest"
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/phases/04-startos-packaging-docs/01-DIGEST.txt
autonomous: false
model: sonnet
requirements: [S9-02]
gap_closure: false

must_haves:
  truths:
    - "Repo descriptor-cifrado has a git tag v0.1.0 pushed to origin"
    - "GitHub Actions docker.yml workflow built and published the multi-arch image at ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0"
    - "GHCR package descriptor-cifrado is publicly visible (anonymous docker pull works)"
    - "The exact sha256 digest of the v0.1.0 multi-arch manifest is captured in 01-DIGEST.txt for downstream consumption by Plan 04"
  artifacts:
    - path: ".planning/phases/04-startos-packaging-docs/01-DIGEST.txt"
      provides: "Verbatim sha256:<64hex> digest of ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 manifest list"
      contains: "sha256:"
  key_links:
    - from: "git tag v0.1.0"
      to: "GHCR :v0.1.0 image + digest"
      via: ".github/workflows/docker.yml on tag push"
      pattern: "tags:\\s*v\\*"
    - from: "01-DIGEST.txt"
      to: "Plan 04 manifest.images.main.source.dockerTag"
      via: "Manual copy at plan 04 execution time"
      pattern: "sha256:[a-f0-9]{64}"
---

<objective>
Tag the descriptor-cifrado repo at v0.1.0 (D-09: first public release), trigger the existing Phase 3 docker.yml workflow which produces the multi-arch GHCR image, ensure the GHCR package is publicly visible (PKG-04 carry-over — currently private per Phase 3 history), and capture the exact sha256 digest of the resulting v0.1.0 multi-arch manifest list. The digest is the critical input for Plan 04's manifest.ts (D-01 digest pin).

Purpose: Without a tagged build, there is no version-tagged digest to pin. Without a public GHCR package, the bed-startos `start-cli s9pk pack` step in Plan 05 fails with `failed to resolve image` (Pitfall 1 in RESEARCH.md).

Output: git tag v0.1.0 pushed, GHCR :v0.1.0 image public, .planning/phases/04-startos-packaging-docs/01-DIGEST.txt containing the verbatim digest line.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/04-startos-packaging-docs/04-CONTEXT.md
@.planning/phases/04-startos-packaging-docs/04-RESEARCH.md
@.planning/phases/03-docker-ghcr/03-02-PLAN.md
@.github/workflows/docker.yml
</context>

<tasks>

<task id="01-01-tag-and-trigger-build" type="auto">
  <name>Task 1: Tag v0.1.0, push, watch docker.yml workflow succeed</name>
  <read_first>
    - /workspace/descriptor-cifrado/.github/workflows/docker.yml (current trigger config — confirm it builds on tag push `v*`)
    - /workspace/descriptor-cifrado/.planning/phases/03-docker-ghcr/03-02-PLAN.md (history of make-public step — confirms Phase 3 left package PRIVATE per RESEARCH.md Pitfall 1)
    - /workspace/descriptor-cifrado/Cargo.toml (workspace version field — verify it is or will be 0.1.0 before tagging)
  </read_first>
  <files>
    - (no file modifications in repo — git tag operation only)
  </files>
  <action>
    1. From the `descriptor-cifrado` repo root, verify clean working tree: `git status` must show "nothing to commit, working tree clean". If not, abort and report.
    2. Verify the current commit on `main` matches origin/main: `git fetch origin && git log origin/main..HEAD` must be empty AND `git log HEAD..origin/main` must be empty.
    3. Inspect `Cargo.toml` workspace `[workspace.package]` block. The `version` field MUST be `0.1.0` before tagging. If currently `0.0.x` or anything else, STOP and ask the user — bumping the workspace version is a separate commit decision, not done implicitly here. (Per D-09 the public release is v0.1.0; do not silently fast-forward this.)
    4. Inspect `.github/workflows/docker.yml`. The `on.push.tags` field MUST include `v*` or `v*.*.*` so the tag push triggers the workflow. If absent, STOP and report — the tag is meaningless without the trigger.
    5. Confirm the metadata-action step in docker.yml emits a tag matching `v0.1.0` (look for `type=semver,pattern={{version}}` or `type=ref,event=tag` — Phase 3 STATE log line confirms `flavor: latest=false + conditional enable=`).
    6. Create annotated tag: `git tag -a v0.1.0 -m "BED v0.1.0 — first public release (descriptor-cifrado image)"`. Commits and tags MUST use the noreply email per D-13 / feedback_git_noreply_email.md. Verify with `git config user.email` returns `55397917+4rkad@users.noreply.github.com` BEFORE running the tag command (the tag's tagger email comes from this config). If wrong, set it explicitly: `git -c user.email="55397917+4rkad@users.noreply.github.com" tag -a v0.1.0 -m "..."`.
    7. Push the tag: `git push origin v0.1.0`.
    8. Watch the docker.yml workflow run: `gh run list --workflow=docker.yml --limit=1 --json databaseId,status,conclusion,headBranch,event` (event should be `push`, headBranch `v0.1.0`). Wait until `conclusion=success` (use `gh run watch <run-id>`).
    9. If the workflow's `make-public` job fails (continue-on-error or hard fail), proceed — Task 2 will handle the public flip manually. If the multi-arch build job fails, STOP and surface the error.
  </action>
  <verify>
    <automated>git tag --list v0.1.0 | grep -q '^v0.1.0$' &amp;&amp; gh run list --workflow=docker.yml --limit=1 --json conclusion --jq '.[0].conclusion' | grep -q '^success$'</automated>
  </verify>
  <acceptance_criteria>
    - `git tag --list v0.1.0` outputs the literal string `v0.1.0`.
    - `git ls-remote --tags origin v0.1.0` returns a non-empty result (tag exists on origin).
    - `gh run list --workflow=docker.yml --limit=1 --json conclusion --jq '.[0].conclusion'` outputs `success` for the run triggered by the v0.1.0 tag push (not a previous main-branch run).
    - `git for-each-ref refs/tags/v0.1.0 --format='%(taggeremail)'` outputs `&lt;55397917+4rkad@users.noreply.github.com&gt;` (D-13 enforcement).
  </acceptance_criteria>
  <done>The v0.1.0 tag exists on origin, the docker.yml workflow ran to success, and the multi-arch image v0.1.0 is published to GHCR (visibility status confirmed in Task 2).</done>
</task>

<task id="01-02-verify-and-flip-ghcr-public" type="auto">
  <name>Task 2: Verify GHCR package public — flip via gh API if needed</name>
  <read_first>
    - /workspace/descriptor-cifrado/.github/workflows/docker.yml (existing make-public job — see whether it succeeded; STATE.md notes Phase 3's make-public used /orgs/semillabitcoin/packages/container/descriptor-cifrado endpoint)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md (Pitfall 1: GHCR package private when start-cli s9pk pack runs)
  </read_first>
  <files>
    - (no repo file modifications — GitHub API operation only)
  </files>
  <action>
    1. Probe public visibility WITHOUT credentials. Run in a clean shell (no docker login active):
       ```
       docker logout ghcr.io 2>/dev/null
       docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0
       ```
       If exit code is 0 and output contains `"manifests":` with `"architecture": "amd64"` AND `"architecture": "arm64"`, the package is already public AND multi-arch — skip to step 4.
    2. If step 1 fails with `unauthorized` or `denied`, the package is still private. Flip via gh API:
       ```
       gh api -X PATCH /orgs/semillabitcoin/packages/container/descriptor-cifrado \
         -f visibility=public
       ```
       This requires a token with `admin:packages` scope on the `semillabitcoin` org. If `gh auth status` shows the active token lacks that scope, run `gh auth refresh -s admin:packages -h github.com` first.
    3. If the gh API call fails (403 or 404), fall back to manual UI flip: open https://github.com/orgs/semillabitcoin/packages/container/descriptor-cifrado/settings → "Change visibility" → Public → confirm. This is acceptable per Phase 3 STATE log "continue-on-error: true with fallback manual toggle URL documented".
    4. Re-probe anonymously: `docker logout ghcr.io && docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0`. MUST succeed with both arch entries.
    5. Probe the registry HTTP endpoint anonymously to double-check (no docker daemon involved):
       ```
       curl -sS -o /dev/null -w '%{http_code}\n' \
         https://ghcr.io/v2/semillabitcoin/descriptor-cifrado/manifests/v0.1.0 \
         -H 'Accept: application/vnd.oci.image.index.v1+json'
       ```
       Expected: HTTP 200 (or 401 with `WWW-Authenticate` containing only `repository:semillabitcoin/descriptor-cifrado:pull` — anonymous pull is auth-bearer-token-allowed even on public packages; the no-credentials docker manifest inspect is the authoritative signal).
  </action>
  <verify>
    <automated>docker logout ghcr.io 2>/dev/null; docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 | grep -q '"architecture": "amd64"' &amp;&amp; docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 | grep -q '"architecture": "arm64"'</automated>
  </verify>
  <acceptance_criteria>
    - `docker logout ghcr.io` followed by `docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0` exits 0.
    - The output contains the literal string `"architecture": "amd64"` AND `"architecture": "arm64"` (multi-arch confirmed).
    - `gh api /orgs/semillabitcoin/packages/container/descriptor-cifrado --jq '.visibility'` outputs `public`.
  </acceptance_criteria>
  <done>The GHCR package is publicly readable, multi-arch manifest is confirmed, and `start-cli s9pk pack` in Plan 05 will be able to pull the image without auth.</done>
</task>

<task id="01-03-capture-digest" type="auto">
  <name>Task 3: Capture sha256 digest of v0.1.0 multi-arch manifest into 01-DIGEST.txt</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md (Pattern 1: manifest.images.main.source.dockerTag = ghcr.io/.../descriptor-cifrado@sha256:&lt;DIGEST_HERE&gt;)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-01 (digest pin requirement)
  </read_first>
  <files>
    - .planning/phases/04-startos-packaging-docs/01-DIGEST.txt
  </files>
  <action>
    1. Get the multi-arch manifest list digest (this is the digest Plan 04 will pin — it covers BOTH amd64 and arm64 in a single descriptor):
       ```
       docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 --raw \
         | sha256sum | awk '{print "sha256:" $1}'
       ```
       Alternative (more direct, uses HEAD on the registry — no docker daemon needed):
       ```
       docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 \
         | grep '^Digest:' | awk '{print $2}'
       ```
       BOTH methods MUST return identical output (a string of form `sha256:<64 lowercase hex chars>`). If they differ, surface the discrepancy — do not pick one silently.
    2. Write the result verbatim to `/workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt`. The file content MUST be exactly one line: `sha256:<64hex>` followed by a single trailing newline. No prefix, no explanation, no markdown — Plan 04 will literally `cat` this file.
    3. Sanity-check: `cat .planning/phases/04-startos-packaging-docs/01-DIGEST.txt | wc -c` MUST output `72` (sha256: prefix is 7 chars + 64 hex chars + 1 newline = 72). If different length, surface the file content and STOP.
    4. Also record the per-arch digests as a side-comment for traceability. Append to a SECOND file `01-DIGEST-PER-ARCH.txt` (informational only, not consumed downstream):
       ```
       docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 \
         > .planning/phases/04-startos-packaging-docs/01-DIGEST-PER-ARCH.txt
       ```
  </action>
  <verify>
    <automated>test -f /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt &amp;&amp; grep -qE '^sha256:[a-f0-9]{64}$' /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt</automated>
  </verify>
  <acceptance_criteria>
    - File `/workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt` exists.
    - The file content matches regex `^sha256:[a-f0-9]{64}$` (one line, exact format).
    - `wc -c` on the file outputs `72`.
    - The digest in the file matches the `Digest:` line from `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0`.
    - File `/workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST-PER-ARCH.txt` exists for traceability and contains both `linux/amd64` and `linux/arm64` lines.
  </acceptance_criteria>
  <done>The exact digest Plan 04 will pin is recorded in a single, parseable file. Downstream plans can `cat 01-DIGEST.txt` to inject the digest into manifest.ts without manual transcription.</done>
</task>

<task id="01-04-checkpoint-verify-pre-handoff" type="checkpoint:human-verify" gate="blocking">
  <name>Task 4: Pre-handoff verify — tag, GHCR public, digest captured</name>
  <action>Human verifies that v0.1.0 was tagged, GHCR is public, and the multi-arch digest is captured to 01-DIGEST.txt. The four shell commands listed under how-to-verify are run by the human; their outputs determine pass/fail. No autonomous code is run in this task — it is the gate that releases Wave 2.</action>
  <what-built>
    - git tag v0.1.0 created and pushed (annotated, noreply email)
    - docker.yml workflow built and published ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 (multi-arch amd64+arm64)
    - GHCR package flipped to public (anonymous docker pull verified)
    - `.planning/phases/04-startos-packaging-docs/01-DIGEST.txt` contains the v0.1.0 multi-arch manifest digest
  </what-built>
  <how-to-verify>
    Run these four commands and paste output back:
    1. `git tag --list v0.1.0` — must output `v0.1.0`
    2. `gh api /orgs/semillabitcoin/packages/container/descriptor-cifrado --jq '.visibility'` — must output `public`
    3. `cat /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt` — must output `sha256:&lt;64hex&gt;` exactly
    4. `docker logout ghcr.io && docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 | grep architecture` — must show both `amd64` and `arm64`

    If all four pass, this checkpoint is the contract Plan 04 depends on. Approving here means: the digest is locked in; Plan 04 will pin THIS exact digest in manifest.ts.
  </how-to-verify>
  <resume-signal>Type "approved" to release Wave 2, or describe any failures.</resume-signal>
</task>

</tasks>

<verification>
- v0.1.0 tag exists on origin.
- docker.yml CI green for v0.1.0 tag run.
- GHCR package descriptor-cifrado is public.
- 01-DIGEST.txt contains a valid `sha256:<64hex>` line.
- Anonymous `docker manifest inspect` succeeds and shows multi-arch.
</verification>

<success_criteria>
Plan 04's manifest.ts can use `cat 01-DIGEST.txt` to construct `dockerTag: "ghcr.io/semillabitcoin/descriptor-cifrado@$(cat .../01-DIGEST.txt)"` without any manual transcription. Plan 05's `start-cli s9pk pack` can pull the image anonymously inside CI runners.
</success_criteria>

<output>
After completion, create `.planning/phases/04-startos-packaging-docs/04-01-SUMMARY.md` recording:
- The v0.1.0 tag SHA
- The captured digest (verbatim)
- Whether GHCR public flip was automatic (workflow) or manual (gh API / UI)
- Multi-arch manifest entries observed
</output>
