---
id: 04-05
phase: 04-startos-packaging-docs
plan: 05
title: "Local pack smoke-test, configure CI secret, tag v0.1.0, S9-04 device UAT, S9-05 update test, Semilla registry publish"
type: execute
wave: 3
depends_on: ["04-04"]
files_modified:
  - "/workspace/bed-startos/bed_x86_64.s9pk (build artifact, not committed)"
  - "/workspace/bed-startos/bed_aarch64.s9pk (build artifact, not committed)"
  - ".planning/phases/04-startos-packaging-docs/05-UAT.md"
autonomous: false
model: sonnet
requirements: [S9-04, S9-05]
gap_closure: false

must_haves:
  truths:
    - "Local make clean x86 arm produces both bed_x86_64.s9pk and bed_aarch64.s9pk"
    - "start-cli s9pk inspect on both files shows id 'bed', title 'BED', volume main, digest-pinned image, arch matching the file"
    - "DEV_KEY secret configured on semillabitcoin/bed-startos repo (CI build can run)"
    - "Tag v0.1.0 pushed to bed-startos; release.yml workflow run is success"
    - "Both .s9pk files attached as assets to GitHub Release v0.1.0"
    - "S9-04 manual UAT on real StartOS 0.4.0 device passes: install, healthy, Tor URL works, LAN URL works, encrypt+decrypt round-trip"
    - "S9-05 manual UAT passes: bumped version preserves /data/encrypted/ history files; previous .bed files still listed and decryptable"
    - "(Optional D-06 secondary) Package published to Semilla Bitcoin registry via start-cli registry package add"
    - "ROADMAP.md Phase 4 marked complete; STATE.md updated"
  artifacts:
    - path: ".planning/phases/04-startos-packaging-docs/05-UAT.md"
      provides: "UAT evidence — install output, dashboard health screenshot reference, Tor URL probe, LAN URL probe, round-trip round, update preservation evidence"
      contains: "## UAT-1: Install + Healthy"
  key_links:
    - from: "git tag v0.1.0 on bed-startos"
      to: "GitHub Release v0.1.0 with bed_x86_64.s9pk + bed_aarch64.s9pk"
      via: ".github/workflows/release.yml"
      pattern: "tags:"
    - from: "Real StartOS 0.4.0 device"
      to: "BED app installed, healthy, Tor+LAN reachable"
      via: "start-cli package install -s OR StartOS UI sideload"
      pattern: "package install"
---

<objective>
Build the s9pk locally as a smoke test, configure the CI signing-key secret, tag v0.1.0 on bed-startos to trigger the release workflow, then run the manual UAT on a real StartOS 0.4.0 device (D-18, blocking) to verify install + healthy + Tor URL + LAN URL + round-trip encrypt/decrypt (S9-04), then verify history persistence across an update (S9-05). After UAT passes, optionally publish to the Semilla Bitcoin registry (D-06 secondary channel) and close the phase.

Per D-18, S9-04 is **manual and blocking** — Phase 4 does not close without device evidence. The plan front-loads automated checks (local pack, manifest inspect, CI green) so the manual UAT only runs when the artifact is known-good, minimizing wasted device cycles.

Output: GitHub Release v0.1.0 of bed-startos with both arch s9pk assets, 05-UAT.md with documented evidence of all six UAT checks passing, optional registry publication, ROADMAP+STATE updates.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/04-startos-packaging-docs/04-CONTEXT.md
@.planning/phases/04-startos-packaging-docs/04-RESEARCH.md
@.planning/phases/04-startos-packaging-docs/04-01-SUMMARY.md
@.planning/phases/04-startos-packaging-docs/04-04-SUMMARY.md
@.planning/ROADMAP.md
@.planning/STATE.md
</context>

<tasks>

<task id="05-01-local-pack-smoke-test" type="auto">
  <name>Task 1: Local make clean x86 arm + start-cli s9pk inspect both artifacts</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Test / Verification Strategy" §"What CAN be automated"
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md Pitfall 7 (multi-arch image must exist; Plan 01 confirmed both archs present)
    - /workspace/bed-startos/Makefile (current ARCHES line)
  </read_first>
  <files>
    - (build artifacts only — bed_x86_64.s9pk and bed_aarch64.s9pk, NOT committed)
  </files>
  <action>
    Step 1. Verify environment is ready:
    ```
    which start-cli                        # /workspace/.cargo/bin/start-cli
    start-cli --version                    # 0.4.0-beta.5
    ls ~/.startos/developer.key.pem        # must exist (RESEARCH.md confirms present)
    cd /workspace/bed-startos && ls node_modules/@start9labs/start-sdk/package.json
    ```
    All four MUST succeed. If developer.key.pem is missing, STOP — start-cli will fail. The key was used for the Semilla registry per RESEARCH.md.

    Step 2. Verify GHCR image is pullable (Plan 01 should have flipped public; sanity-check):
    ```
    docker logout ghcr.io
    docker manifest inspect ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0 | grep architecture
    ```
    MUST show both amd64 and arm64. If not, surface and STOP — the local pack will fail to pull. (Plan 01's checkpoint should have caught this; this is defensive.)

    Step 3. Pre-pull the multi-arch image to local docker cache (start-cli pulls under the hood, but pre-pulling surfaces auth issues earlier and speeds the next step):
    ```
    docker pull --platform linux/amd64 ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0
    docker pull --platform linux/arm64 ghcr.io/semillabitcoin/descriptor-cifrado:v0.1.0
    ```

    Step 4. Build x86_64:
    ```
    cd /workspace/bed-startos
    make clean x86 2>&1 | tee /tmp/bed-pack-x86.log
    ```
    Expected: `bed_x86_64.s9pk` in repo root, exit 0. If `npm run check` fails: re-run from Plan 04 Task 4 (TS errors must already have been resolved). If `start-cli s9pk pack` fails with `failed to resolve image`: re-check public visibility (Pitfall 1).

    Step 5. Build aarch64:
    ```
    make clean arm 2>&1 | tee /tmp/bed-pack-arm.log
    ```
    Note: `make clean` between builds wipes the JS bundle. The two builds are sequential, not parallel — that's fine for local smoke-test. CI matrix runs them in parallel.

    Step 6. Inspect both artifacts:
    ```
    cd /workspace/bed-startos
    start-cli s9pk inspect bed_x86_64.s9pk manifest > /tmp/bed-x86-manifest.json
    start-cli s9pk inspect bed_aarch64.s9pk manifest > /tmp/bed-arm-manifest.json
    ```
    Both files MUST contain `"id": "bed"`, `"title": "BED"`, `"volumes": ["main"]`, and the digest-pinned image source. Verify with `jq`:
    ```
    jq -r '.id, .title, .volumes[], .images.main.source.dockerTag' /tmp/bed-x86-manifest.json
    ```
    Expected output:
    ```
    bed
    BED
    main
    ghcr.io/semillabitcoin/descriptor-cifrado@sha256:<digest>
    ```
    The digest MUST equal the contents of /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/01-DIGEST.txt.

    Step 7. Confirm s9pk file sizes are reasonable (sanity bound):
    ```
    ls -lh bed_*.s9pk
    ```
    Each file should be in the 5-50 MB range (the descriptor-cifrado image is 5.8 MB compressed per Phase 2 STATE; the s9pk wraps the layer plus manifest plus signature). If a file is < 1 MB or > 100 MB, surface as anomaly.
  </action>
  <verify>
    <automated>cd /workspace/bed-startos &amp;&amp; test -f bed_x86_64.s9pk &amp;&amp; test -f bed_aarch64.s9pk &amp;&amp; start-cli s9pk inspect bed_x86_64.s9pk manifest | jq -r '.id' | grep -q '^bed$' &amp;&amp; start-cli s9pk inspect bed_x86_64.s9pk manifest | jq -r '.images.main.source.dockerTag' | grep -qE '@sha256:[a-f0-9]{64}$'</automated>
  </verify>
  <acceptance_criteria>
    - /workspace/bed-startos/bed_x86_64.s9pk exists, size between 1 MB and 100 MB.
    - /workspace/bed-startos/bed_aarch64.s9pk exists, size between 1 MB and 100 MB.
    - `start-cli s9pk inspect bed_x86_64.s9pk manifest | jq -r '.id'` outputs `bed`.
    - `start-cli s9pk inspect bed_x86_64.s9pk manifest | jq -r '.title'` outputs `BED`.
    - `start-cli s9pk inspect bed_x86_64.s9pk manifest | jq -r '.volumes[]'` outputs `main`.
    - `start-cli s9pk inspect bed_x86_64.s9pk manifest | jq -r '.images.main.source.dockerTag'` outputs a string containing `@sha256:<64hex>` matching 01-DIGEST.txt verbatim.
    - Same checks pass for bed_aarch64.s9pk.
    - `make clean x86` and `make clean arm` exit codes are 0.
  </acceptance_criteria>
  <done>Local pack works for both archs. Manifest declarations are correctly embedded in the s9pk. Plan 05 next steps can confidently push the tag knowing CI will produce equivalent artifacts.</done>
</task>

<task id="05-02-configure-ci-secret-and-tag-release" type="checkpoint:decision" gate="blocking">
  <name>Task 2: Decide release path — CI build (Option A) vs manual upload (Option B)</name>
  <action>Decision checkpoint — the human chooses between two release paths defined in the options below. The selected option drives Task 3 execution.</action>
  <decision>How to provision the DEV_KEY secret and trigger the v0.1.0 release</decision>
  <context>
    The CI workflow (.github/workflows/release.yml) needs `secrets.DEV_KEY` containing the contents of `~/.startos/developer.key.pem` to sign the s9pk. Without this secret, the CI build fails on `start-cli s9pk pack`.

    There are two paths to ship v0.1.0:

    Option A — Configure secret + tag on bed-startos (CI builds release):
    1. Configure secret: `gh secret set DEV_KEY --repo semillabitcoin/bed-startos < ~/.startos/developer.key.pem`
    2. Tag and push: `cd /workspace/bed-startos && git tag -a v0.1.0 -m "BED v0.1.0 — first public release" && git push origin v0.1.0`
    3. Watch CI: `gh run watch --repo semillabitcoin/bed-startos`
    4. Verify GitHub Release contains both .s9pk assets.

    Option B — Skip CI for v0.1.0; upload locally-built artifacts manually:
    1. Tag locally: `cd /workspace/bed-startos && git tag -a v0.1.0 -m "BED v0.1.0 — first public release" && git push origin v0.1.0`
    2. Create release manually with local artifacts: `gh release create v0.1.0 bed_x86_64.s9pk bed_aarch64.s9pk --repo semillabitcoin/bed-startos --title "v0.1.0" --notes-file /tmp/release-notes.md`
    3. Skip CI entirely for this release (CI workflow runs but may fail on the tag — that's a known no-op since the artifacts are already uploaded).

    Option A is more reproducible (CI is the canonical build path, anyone can re-trigger). Option B is faster (no secret rotation concerns) and matches the "shippeable now" prioritization.

    Per CONTEXT.md "Claude's Discretion" and the v1 priority on shipping fast, the recommended choice is Option A — establish the CI as the canonical build path now while everything is fresh, even if it costs ~15 minutes of secret config + first run debugging.
  </context>
  <options>
    <option id="option-a">
      <name>CI builds release (configure DEV_KEY secret, tag, watch CI)</name>
      <pros>
        - Reproducible: any future tag re-triggers identical build
        - Future contributors don't need a local signing key
        - Establishes the canonical build path early
      </pros>
      <cons>
        - Requires DEV_KEY secret rotation discipline
        - First CI run may surface auth or arch issues that local build missed
      </cons>
    </option>
    <option id="option-b">
      <name>Manual upload of locally-built artifacts (skip CI for v0.1.0)</name>
      <pros>
        - Faster: no secret config, no CI debugging
        - Local pack already verified in Task 1
      </pros>
      <cons>
        - Future tags either need a one-off manual upload (toil) or DEV_KEY config later anyway
        - Mixed signal: CI workflow exists but isn't the canonical build path until later
      </cons>
    </option>
  </options>
  <resume-signal>Select: option-a, option-b, or describe a third path.</resume-signal>
</task>

<task id="05-03-execute-chosen-release-path" type="auto">
  <name>Task 3: Execute the release path chosen in Task 2 (provision secret, tag v0.1.0, verify release)</name>
  <read_first>
    - .planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-09 (first release v0.1.0), D-13 (noreply tag email)
    - /workspace/.startos/developer.key.pem (signing key — not contents, just confirm it exists)
  </read_first>
  <files>
    - (no repo file changes — git tag operation + GitHub Release creation)
  </files>
  <action>
    IF Option A selected in Task 2:

    Step A1. Configure DEV_KEY secret:
    ```
    gh secret set DEV_KEY --repo semillabitcoin/bed-startos < ~/.startos/developer.key.pem
    gh secret list --repo semillabitcoin/bed-startos | grep -q '^DEV_KEY'
    ```
    Verify the secret is configured. The contents are not retrievable after set — that's expected.

    Step A2. Verify Cargo.toml workspace version on descriptor-cifrado is `0.1.0` (Plan 01 ensured this) AND bed-startos has no separate version file (only versions/v0.1.0.1.ts is the SDK version).

    Step A3. Tag and push from bed-startos:
    ```
    cd /workspace/bed-startos
    git config user.email "55397917+4rkad@users.noreply.github.com"
    git tag -a v0.1.0 -m "BED v0.1.0 — first public release

    s9pk wrapper for ghcr.io/semillabitcoin/descriptor-cifrado@<digest from manifest>.

    Sideload via 'start-cli package install -s bed_<arch>.s9pk' or via the
    StartOS UI Sideload feature."
    git push origin v0.1.0
    ```
    Verify the tag is pushed:
    ```
    git ls-remote --tags origin v0.1.0
    ```

    Step A4. Watch the CI run:
    ```
    sleep 5
    RUN_ID=$(gh run list --repo semillabitcoin/bed-startos --workflow=release.yml --limit=1 --json databaseId --jq '.[0].databaseId')
    gh run watch $RUN_ID --repo semillabitcoin/bed-startos
    ```
    Wait for completion. If the run fails:
    - Auth error in GHCR step → ensure GITHUB_TOKEN has `read:packages` (org default)
    - DEV_KEY parse error → verify the secret was set with the FULL pem file including `-----BEGIN` and `-----END` lines and a trailing newline
    - Build matrix error → re-check Task 4 of Plan 04 (tsc must pass)
    Surface the failure log; do not retry blindly.

    Step A5. Verify the GitHub Release was created with both s9pks:
    ```
    gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '.assets[].name'
    ```
    Output MUST include `bed_x86_64.s9pk` AND `bed_aarch64.s9pk`.

    IF Option B selected in Task 2:

    Step B1. Tag locally:
    ```
    cd /workspace/bed-startos
    git config user.email "55397917+4rkad@users.noreply.github.com"
    git tag -a v0.1.0 -m "BED v0.1.0 — first public release"
    git push origin v0.1.0
    ```

    Step B2. Create release notes file at /tmp/release-notes.md (4-6 lines; mention the digest pinned, the install command, link to README).

    Step B3. Create release with local artifacts:
    ```
    cd /workspace/bed-startos
    gh release create v0.1.0 \
      --repo semillabitcoin/bed-startos \
      --title "BED v0.1.0" \
      --notes-file /tmp/release-notes.md \
      bed_x86_64.s9pk bed_aarch64.s9pk
    ```
    Verify:
    ```
    gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '.assets[].name'
    ```

    BOTH paths converge on: GitHub Release v0.1.0 with both s9pk assets visible.
  </action>
  <verify>
    <automated>gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '.assets[].name' 2>/dev/null | tr '\n' ' ' | grep -q 'bed_x86_64.s9pk' &amp;&amp; gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '.assets[].name' 2>/dev/null | grep -q 'bed_aarch64.s9pk'</automated>
  </verify>
  <acceptance_criteria>
    - `git ls-remote --tags origin v0.1.0` (in bed-startos) returns a non-empty result.
    - `gh release view v0.1.0 --repo semillabitcoin/bed-startos --json tagName --jq '.tagName'` outputs `v0.1.0`.
    - `gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '[.assets[].name] | sort | join(",")'` outputs `bed_aarch64.s9pk,bed_x86_64.s9pk` (both, sorted).
    - The asset sizes match the local build sizes from Task 1 (within 1 KB) — this confirms the artifacts in the release are equivalent to what was tested locally.
    - If Option A: `gh run list --repo semillabitcoin/bed-startos --workflow=release.yml --limit=1 --json conclusion --jq '.[0].conclusion'` outputs `success`.
    - The tag's tagger email is `55397917+4rkad@users.noreply.github.com` (verified via `git for-each-ref refs/tags/v0.1.0 --format='%(taggeremail)'`).
  </acceptance_criteria>
  <done>v0.1.0 is tagged on bed-startos and a GitHub Release with both arch s9pks is published. The artifacts are ready for sideload to a real StartOS device in Task 4.</done>
</task>

<task id="05-04-uat-real-device" type="checkpoint:human-action" gate="blocking">
  <name>Task 4: Manual blocking UAT on real StartOS device (S9-04)</name>
  <action>Human performs the full UAT script on a real StartOS 0.4.0 device (D-18: no emulator). The how-to-verify section embeds the 05-UAT.md template with six sections (Install+Healthy, Tor URL, LAN URL, Round-trip encrypt+decrypt, History persistence across update, Threat-model accuracy). Each section must be completed with concrete evidence pasted in. Phase 4 does not close until 05-UAT.md is filled and approved.</action>
  <what-built>
    - GitHub Release v0.1.0 of bed-startos with bed_x86_64.s9pk and bed_aarch64.s9pk assets
    - Locally-verified manifest (id=bed, title=BED, digest-pinned image)
    - Health check uses checkPortListening on 8080
    - Volume main mounts at /data/encrypted/
  </what-built>
  <how-to-verify>
    **D-18: This is the manual blocking UAT on a REAL StartOS 0.4.0 device. No emulator exists. Phase 4 does not close until this passes.**

    **Prerequisite:** Open /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/05-UAT.md (create it if it does not exist — content below) and fill it in as you go. Each section MUST end with concrete evidence (command output paste, dashboard screenshot reference, .onion URL probed, etc.) — not just a checkmark.

    Create 05-UAT.md with this template, then fill each section as you work:

    ```markdown
    # Phase 4 UAT — Real StartOS 0.4.0 Device

    Tested: <YYYY-MM-DD>
    Device hostname: <e.g., embassy.local>
    StartOS version: <output of /system/info/version on dashboard>
    Architecture: <x86_64 or aarch64>
    Tested by: <user>

    ## UAT-1: Install + Healthy (S9-04)

    **Command:**
    ```
    # If using start-cli sideload:
    start-cli -h <device-host>.local package install -s bed_<arch>.s9pk

    # OR via StartOS UI:
    # Settings → Sideload → upload bed_<arch>.s9pk
    ```

    **Expected:** App appears in dashboard within 30-60s with status "Running" and health "Healthy".

    **Evidence (paste output):**
    - `start-cli` output (or screenshot reference of UI install progress)
    - Dashboard package list line for BED (status + health)
    - Logs button output first 50 lines (look for "Listening on 127.0.0.1:8080")
    - Health check passing (✓ green check next to BED in dashboard)

    Status: <PASS / FAIL — describe>

    ## UAT-2: Tor URL works + round-trip encrypt/decrypt (S9-04)

    **Steps:**
    1. From StartOS dashboard, copy the Tor .onion URL for BED.
    2. Open Tor Browser → paste URL → page loads.
    3. Tab "Cifrar": paste this real 2-of-3 multisig descriptor (with `<0;1>/*`):
       ```
       wsh(sortedmulti(2,
         [<fp1>/<path1>]xpub.../<0;1>/*,
         [<fp2>/<path2>]xpub.../<0;1>/*,
         [<fp3>/<path3>]xpub.../<0;1>/*
       ))#<checksum>
       ```
       (Use a real test descriptor — a fresh one for UAT, not a production wallet.)
    4. Click "Cifrar". Verify three outputs appear: .bed download, armored block, QR.
    5. Switch to "Descifrar" tab. Upload the .bed and paste one of the cosigner xpubs.
    6. Click "Descifrar". The recovered descriptor MUST match the original byte-for-byte (modulo BIP-380 checksum normalization).

    **Evidence:**
    - .onion URL prefix (first 8 chars OK for log; not the full URL)
    - Screenshot reference of three outputs visible
    - Round-trip: paste of "original descriptor" line + "recovered descriptor" line + a `diff` showing they are equal (or the explicit normalization path)

    Status: <PASS / FAIL>

    ## UAT-3: LAN URL works + same round-trip (S9-04)

    **Steps:**
    1. From StartOS dashboard, copy the LAN URL (likely `https://bed.<host>.local`).
    2. Open in a regular browser on the same LAN. Browser may warn about cert; accept.
    3. Repeat the encrypt+decrypt round-trip from UAT-2.

    **Evidence:**
    - LAN URL probe: `curl -k -sS -o /dev/null -w '%{http_code}\n' https://bed.<host>.local/`  → expect 200.
    - Round-trip evidence as in UAT-2.

    Status: <PASS / FAIL>

    ## UAT-4: History persistence baseline (S9-05 setup)

    **Steps:**
    1. In the BED UI, enable the history toggle.
    2. Encrypt the same descriptor as UAT-2 again (history mode ON).
    3. Switch to Historial tab → verify the .bed entry appears.
    4. Note the entry filename (e.g. `20260507T120000Z-abc12345.bed`).

    **Evidence:**
    - Filename of the history entry created
    - Screenshot reference of Historial tab

    Status: <PASS / FAIL>

    ## UAT-5: Update preserves history (S9-05)

    **Steps:**
    1. From the dev workstation, edit /workspace/bed-startos/startos/versions/v0.1.0.1.ts → bump version string from `'0.1.0:1'` to `'0.1.0:2'`. Add a v0.1.0.2.ts file mirroring the original.
    2. Update versions/index.ts to use v_0_1_0_2 as current; v_0_1_0_1 in `other`.
    3. `make clean x86` (or `arm` matching device).
    4. Sideload the new s9pk: `start-cli package install -s bed_<arch>.s9pk` (StartOS detects update — no uninstall needed).
    5. After update completes (status returns to Running + Healthy), open Historial tab.
    6. Verify the .bed entry from UAT-4 is STILL listed.
    7. Click decrypt on it (or download → re-decrypt) and verify the original descriptor is recovered.

    **Evidence:**
    - Output of update install command
    - Filename of UAT-4 entry STILL visible in Historial tab post-update (paste filename)
    - Decryption succeeds: paste recovered descriptor + diff vs UAT-4 original
    - File on the device persists at /data/encrypted/<filename>.bed (verify via StartOS Files browser if available, or via Logs after a redeployment)

    Status: <PASS / FAIL>

    Note: this UAT also revalidates D-10 (no migration code needed for patch bumps — same crypto, same volume schema).

    ## UAT-6: Threat model README accuracy review (DOC-01)

    **Steps:**
    1. Read /workspace/descriptor-cifrado/README.md §"Threat Model" carefully.
    2. Read /workspace/bed-startos/README.md §"Threat model summary" carefully.
    3. Confirm the golden rule appears at LEAST twice in each.
    4. Confirm what BED protects vs does NOT protect is stated honestly given what was just tested in UAT-2/3.

    **Evidence:**
    - "Read and accurate" or list any inaccuracies to fix before phase close.

    Status: <PASS / FAIL>

    ## Summary

    All UAT-1..6 pass: <YES / NO — describe failures>
    Phase 4 closure recommendation: <APPROVE / FIX issues + retest>
    ```

    Run UAT-1 through UAT-6 on a real device. Paste back the completed 05-UAT.md.

    **If any UAT fails:** Phase 4 does NOT close. Surface the failure to the planner for revision (likely re-running Plan 04 Task X to fix manifest/main/etc.).
  </how-to-verify>
  <resume-signal>Paste back the completed 05-UAT.md with all 6 sections filled and Status fields populated. If all PASS, type "approved" to release Task 5 (registry publish + close).</resume-signal>
</task>

<task id="05-05-publish-to-semilla-registry" type="auto">
  <name>Task 5: (Optional D-06 secondary) Publish bed s9pk to Semilla Bitcoin registry</name>
  <read_first>
    - $HOME/.claude/projects/-home-anon/memory/project_startos_registry.md (registry URL, CLI command, per-arch gitHash requirement)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md D-06 (Semilla registry as secondary channel post-S9-04)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"Build Pipeline" final sentence on registry CLI
  </read_first>
  <files>
    - (no local file changes — registry RPC operation only)
  </files>
  <action>
    Step 1. Confirm UAT-1..6 all PASS in Task 4. If any UAT failed, SKIP this task entirely — do not publish a broken artifact to the user-facing registry.

    Step 2. Read the registry URL and credentials from the project memory note. Per the memory, the registry CLI command is:
    ```
    start-cli -r <REGISTRY_URL> registry package add <FILE> [--url <PUBLIC_URL>]
    ```
    where REGISTRY_URL is the Semilla Bitcoin registry RPC endpoint (typically `https://tienda.privacidadbitcoin.com/rpc/v0` per RESEARCH.md) and PUBLIC_URL points to the GitHub Release asset.

    Step 3. Get the public URLs of the GitHub Release assets:
    ```
    X86_URL=$(gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '.assets[] | select(.name=="bed_x86_64.s9pk") | .url')
    ARM_URL=$(gh release view v0.1.0 --repo semillabitcoin/bed-startos --json assets --jq '.assets[] | select(.name=="bed_aarch64.s9pk") | .url')
    ```
    Both should be public download URLs of the form `https://github.com/semillabitcoin/bed-startos/releases/download/v0.1.0/bed_<arch>.s9pk`.

    Step 4. Publish each arch separately (per-arch package add per the memory's gitHash requirement):
    ```
    cd /workspace/bed-startos
    start-cli -r <REGISTRY_URL> registry package add bed_x86_64.s9pk --url "$X86_URL"
    start-cli -r <REGISTRY_URL> registry package add bed_aarch64.s9pk --url "$ARM_URL"
    ```
    Each command MAY require interactive confirmation or auth. Both .s9pk files MUST share the same gitHash (the bed-startos commit SHA at v0.1.0 tag) — start-cli enforces this; if it complains, surface and STOP.

    Step 5. Verify the package is listed in the registry:
    ```
    start-cli -r <REGISTRY_URL> registry package list | grep -i 'bed'
    ```
    Should show both arch entries.

    Step 6. If publication fails for any reason (auth, schema mismatch, network), this is NON-BLOCKING for phase close — D-06 lists registry as secondary channel and sideload as primary. Document the failure in 04-05-SUMMARY.md and proceed.
  </action>
  <verify>
    <automated>true  # Manual verification — registry RPC may not have a clean grep target; check via project memory commands</automated>
  </verify>
  <acceptance_criteria>
    - If UAT passed in Task 4: at least one attempt was made to publish to the Semilla Bitcoin registry.
    - Either both `start-cli registry package add` commands succeeded AND `registry package list` shows both arch entries, OR a failure is documented in the SUMMARY explaining why publication was skipped or failed (D-06 secondary channel — non-blocking).
    - The two .s9pk files in the registry (if published) share the same gitHash (verified by start-cli's per-arch consistency check).
  </acceptance_criteria>
  <done>The Semilla Bitcoin registry has bed v0.1.0 listed (or a documented reason why not). Sideload via GitHub Release is the primary install path either way.</done>
</task>

<task id="05-06-close-phase" type="auto">
  <name>Task 6: Update ROADMAP.md, STATE.md, REQUIREMENTS.md to mark Phase 4 complete</name>
  <read_first>
    - /workspace/descriptor-cifrado/.planning/ROADMAP.md (Phase 4 entry to mark complete)
    - /workspace/descriptor-cifrado/.planning/STATE.md (Current Position section to update)
    - /workspace/descriptor-cifrado/.planning/REQUIREMENTS.md (Traceability table — S9-01..S9-05, DOC-01, DOC-02 to mark Complete)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/05-UAT.md (UAT evidence — must show PASS for all 6 sections before this task runs)
  </read_first>
  <files>
    - /workspace/descriptor-cifrado/.planning/ROADMAP.md
    - /workspace/descriptor-cifrado/.planning/STATE.md
    - /workspace/descriptor-cifrado/.planning/REQUIREMENTS.md
  </files>
  <action>
    Step 1. Confirm all UAT sections in 05-UAT.md show "Status: PASS". If any FAIL, STOP — phase does not close.

    Step 2. Update ROADMAP.md:
    - Phase 4 list entry (line ~18): change `- [ ] **Phase 4: StartOS Packaging + Docs** ...` → `- [x] **Phase 4: StartOS Packaging + Docs** - ... (completed YYYY-MM-DD)`
    - Phase 4 details block (line ~75): no goal change. Update Plans count: `**Plans:** 5 plans` (matches actual). Replace `**Plans**: TBD` line with the actual 5-plan list using `[x]` markers.
    - Progress table (line ~93): update Phase 4 row → `5/5 | Complete | YYYY-MM-DD`.

    Step 3. Update STATE.md:
    - Line 6 `stopped_at:` → "Phase 4 complete; v0.1.0 released; UAT passed"
    - Line 7 `last_updated:` → current ISO timestamp
    - Line 11 `completed_phases: 4` (was 3)
    - Line 13 `completed_plans: 19` (14 from prior phases + 5 from Phase 4)
    - Line 14 `percent: 100`
    - "Current Position" block: Phase: Done; Plan: All complete; Status: v0.1.0 shipped.
    - Append "Decisions affecting current work" with relevant Phase 4 entries (digest pin, manifest patterns, UAT evidence path).

    Step 4. Update REQUIREMENTS.md Traceability table (line ~149-155):
    - S9-01 | Phase 4 | Complete
    - S9-02 | Phase 4 | Complete
    - S9-03 | Phase 4 | Complete
    - S9-04 | Phase 4 | Complete
    - S9-05 | Phase 4 | Complete
    - DOC-01 | Phase 4 | Complete
    - DOC-02 | Phase 4 | Complete
    Update header coverage line if present: "Mapped to phases: 40/40, all Complete".

    Step 5. Stage and commit ALL three files in a single phase-close commit. The commit must use the noreply email and be in the descriptor-cifrado repo (NOT bed-startos):
    ```
    cd /workspace/descriptor-cifrado
    git add .planning/ROADMAP.md .planning/STATE.md .planning/REQUIREMENTS.md \
            .planning/phases/04-startos-packaging-docs/05-UAT.md
    git -c user.email="55397917+4rkad@users.noreply.github.com" \
      commit -m "docs(planning): close Phase 4 — StartOS Packaging + Docs (v0.1.0 shipped)

    - bed-startos v0.1.0 released with bed_x86_64.s9pk and bed_aarch64.s9pk
    - Real-device UAT passed (S9-04) on StartOS 0.4.0
    - History preservation across update verified (S9-05)
    - Threat model documented in both repo READMEs (DOC-01, DOC-02)
    - 7 requirements (S9-01..S9-05, DOC-01, DOC-02) marked Complete in REQUIREMENTS.md"
    git push origin main
    ```

    Step 6. Note follow-ups for the user (do NOT auto-create — user may have shifted priorities):
    - Phase 5 (i18n EN+ES) — defer to /gsd:add-phase per CONTEXT.md `<deferred>`
    - Real screenshots for both READMEs (currently text-only) — defer to a follow-up patch release (v0.1.1)
    - Flip bed-startos to public after sanitizing both repos' git histories per CONTEXT.md D-12
    - Consider updating bed-startos description to a marketing-friendly tagline once registry feedback comes in
  </action>
  <verify>
    <automated>grep -q '\[x\] \*\*Phase 4:' /workspace/descriptor-cifrado/.planning/ROADMAP.md &amp;&amp; grep -q '^| S9-04 | Phase 4 | Complete' /workspace/descriptor-cifrado/.planning/REQUIREMENTS.md &amp;&amp; grep -q '^| DOC-02 | Phase 4 | Complete' /workspace/descriptor-cifrado/.planning/REQUIREMENTS.md &amp;&amp; grep -q 'completed_phases: 4' /workspace/descriptor-cifrado/.planning/STATE.md</automated>
  </verify>
  <acceptance_criteria>
    - ROADMAP.md Phase 4 entry shows `- [x]` (checked) and a completion date.
    - ROADMAP.md Phase 4 Plans list shows 5 entries, each `[x]`.
    - STATE.md `completed_phases: 4` and `completed_plans: 19`.
    - REQUIREMENTS.md Traceability table shows all 7 Phase 4 requirements (S9-01..S9-05, DOC-01, DOC-02) as Complete.
    - 05-UAT.md exists with PASS in all 6 sections.
    - The phase-close commit is on origin/main of descriptor-cifrado with the noreply email.
  </acceptance_criteria>
  <done>Phase 4 is closed. The project state files reflect that v0.1.0 shipped, UAT passed, and all 7 requirements are met. The user can now run `/gsd:add-phase` to start Phase 5 (i18n) or wrap the milestone.</done>
</task>

</tasks>

<verification>
- Local pack produces both arch s9pks; manifest inspect confirms identity + digest.
- v0.1.0 tag pushed; GitHub Release v0.1.0 has both s9pk assets.
- 05-UAT.md shows PASS for all 6 sections (S9-04 + S9-05 verified on real device).
- Semilla Bitcoin registry has bed v0.1.0 (or a documented reason why not).
- ROADMAP, STATE, REQUIREMENTS updated and committed; Phase 4 marked complete.
</verification>

<success_criteria>
A holder running StartOS 0.4.0 can visit https://github.com/semillabitcoin/bed-startos/releases/tag/v0.1.0, download the s9pk for their architecture, sideload via `start-cli package install -s` or the StartOS UI, and have a working BED app within minutes — verified end-to-end on real hardware. Plans are reproducible: tagging v0.1.1 with bumped digest will produce the next release via the same CI workflow.
</success_criteria>

<output>
After completion, create `.planning/phases/04-startos-packaging-docs/04-05-SUMMARY.md` recording:
- Local pack file sizes (x86 + arm)
- The chosen release path (Option A or B from Task 2)
- The CI run conclusion + run URL (if Option A)
- The GitHub Release URL
- Per-section UAT outcomes (linked to 05-UAT.md)
- Registry publish outcome (success / skipped / failed with reason)
- Final ROADMAP/STATE/REQUIREMENTS commit SHA
- Open follow-ups for the user
</output>
