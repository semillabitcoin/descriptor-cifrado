---
id: 04-02
phase: 04-startos-packaging-docs
plan: 02
title: "Write descriptor-cifrado README in English with threat model and golden rule"
type: execute
wave: 1
depends_on: []
files_modified:
  - README.md
autonomous: true
model: sonnet
requirements: [DOC-01, DOC-02]
gap_closure: false

must_haves:
  truths:
    - "descriptor-cifrado/README.md exists at repo root"
    - "README contains an explicit Threat Model section listing what BED protects and what it does NOT protect against (DOC-01)"
    - "The golden rule 'never co-locate a .bed file and a cosigner xpub' appears verbatim TWICE — once in TL;DR and once in Threat Model (D-03)"
    - "README is in English (D-02), references AES-256-GCM, magic BEB, BIP PR 1951, Liana interop, and crate v0.0.2"
  artifacts:
    - path: "README.md"
      provides: "Project README — entry point for github.com/semillabitcoin/descriptor-cifrado visitors"
      contains: "## Threat Model"
      min_lines: 80
  key_links:
    - from: "TL;DR"
      to: "Golden rule callout"
      via: "blockquote with 'never co-locate' phrase"
      pattern: "never co-locate"
    - from: "Threat Model section"
      to: "Golden rule callout (repeated)"
      via: "blockquote inside §What BED does NOT protect against"
      pattern: "never co-locate"
---

<objective>
Author the descriptor-cifrado/README.md (English, 6 sections per D-02) covering DOC-01 (explicit threat model) and DOC-02 (golden rule appears twice per D-03). This README is the canonical project documentation — it lives in the BACKEND repo, NOT the bed-startos wrapper. The bed-startos repo will get its own README in Plan 04 covering install instructions; this one covers the underlying tool, its crypto, and its threat model in depth.

Purpose: The descriptor-cifrado repo currently has no README.md. Without it, the GitHub project page is bare and the threat model lives only in `IDEA.md` and `.planning/`. DOC-01 and DOC-02 are explicit Phase 4 requirements; without them the phase cannot close.

Output: descriptor-cifrado/README.md with TL;DR, Usage, Threat Model, Crypto Details, Common Pitfalls, References. The bed-startos README in Plan 04 will link here for the deep technical detail.
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
@IDEA.md
</context>

<tasks>

<task id="02-01-write-readme" type="auto">
  <name>Task 1: Write README.md (6 sections, English, DOC-01 + DOC-02 redundancy)</name>
  <read_first>
    - /workspace/descriptor-cifrado/IDEA.md (original threat model brief — language to draw on)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-CONTEXT.md (D-02, D-03, D-11 — README structure, golden rule placement, .bed format as external contract)
    - /workspace/descriptor-cifrado/.planning/phases/04-startos-packaging-docs/04-RESEARCH.md §"README + Threat Model Structure" (full 6-section skeleton — use as the literal scaffold)
    - /workspace/descriptor-cifrado/Cargo.toml (verify crate version pin to bitcoin-encrypted-backup v0.0.2 rev cd7ee382 for the Crypto Details section)
    - /workspace/descriptor-cifrado/.planning/phases/01-crypto-core-http-api/01-CONTEXT.md (BIP wildcard `&lt;0;1&gt;/*` requirement, QR ECC-L 2900 byte limit)
    - /workspace/descriptor-cifrado/.planning/phases/02-spa-frontend-history/02-CONTEXT.md (history mode opt-in default OFF, `BED_DATA_DIR=/data/encrypted/`)
  </read_first>
  <files>
    - /workspace/descriptor-cifrado/README.md
  </files>
  <action>
    Create `/workspace/descriptor-cifrado/README.md` with the following structure. Use the EXACT section headings shown — Plan 04's bed-startos README will link to specific anchors (e.g., `#threat-model`).

    ```markdown
    # BED — Bitcoin Encrypted Backup

    [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

    A small Rust + Svelte service that encrypts a Bitcoin multisig descriptor
    using any cosigner xpub, producing a `.bed` file (binary, ASCII-armored, or
    QR PNG) that can only be decrypted by someone holding that same xpub.

    Implements the draft BIP "Bitcoin Encrypted Backup" (PR
    [bitcoin/bips#1951](https://github.com/bitcoin/bips/pull/1951)) and is
    interoperable with [Liana](https://wizardsardine.com/liana/) wallet v13+ via
    the [`bitcoin-encrypted-backup`](https://github.com/pythcoiner/encrypted_backup)
    crate v0.0.2.

    ## TL;DR

    1. Install on StartOS as the [BED s9pk](https://github.com/semillabitcoin/bed-startos).
    2. Open the Tor onion or LAN `.local` URL the StartOS dashboard provides.
    3. Paste your descriptor → download the `.bed` (or armored block, or QR).
    4. Distribute one `.bed` copy per cosigner location.

    > **Golden rule:** Never store a `.bed` file and a cosigner xpub of the same
    > multisig in the same location. If an attacker finds both, they can decrypt
    > the descriptor and learn your full wallet structure. See [Threat
    > Model](#threat-model) for the complete model.

    ## Usage

    ### Encrypt a descriptor

    1. Open the **Cifrar** tab.
    2. Paste a multisig descriptor that uses the multipath wildcard
       `<0;1>/*`. Descriptors without this wildcard are rejected — see
       [Common Pitfalls](#common-pitfalls).
    3. Click **Cifrar**. The page returns three outputs simultaneously:
       - A binary `.bed` file (download).
       - An ASCII-armored block with `-----BEGIN BITCOIN ENCRYPTED BACKUP-----`
         headers, copyable to clipboard.
       - A QR PNG of the armored payload (download), if the payload fits within
         QR ECC-L capacity (~2,900 bytes).
    4. Optionally enable the **history toggle**. With history enabled the `.bed`
       is also saved to `/data/encrypted/<YYYYMMDDTHHMMSSZ>-<8hex>.bed` for
       later listing and deletion. Default is OFF — the cleartext descriptor
       and the `.bed` are returned to the browser and immediately forgotten.

    ### Decrypt a `.bed` file

    1. Open the **Descifrar** tab.
    2. Either paste the armored block or upload the binary `.bed` file.
    3. Either paste the cosigner xpub (bare, e.g. `xpub6...`) or upload a file
       containing it.
    4. Click **Descifrar**. The recovered descriptor appears with a
       Copy-to-clipboard button. The descriptor is never persisted on disk.

    ## Threat Model

    ### What BED protects

    - **Descriptor privacy.** A `.bed` file reveals nothing about the wallet's
      structure, xpubs, derivation paths, or spending policy to an attacker
      who only has the file. AES-256-GCM authenticated encryption guarantees
      that any tampering is detected on decryption.
    - **xpub distribution.** Each cosigner can safely store a `.bed` backup
      without exposing the full wallet policy — they only need their own xpub
      to decrypt it. This enables redundant distribution of the descriptor
      (the only piece needed to fully recover a multisig wallet) without
      coupling that distribution to the privacy of the policy.
    - **Cleartext-on-disk leakage.** The cleartext descriptor never touches
      disk. It is wrapped in `secrecy::SecretString` from parse through
      encryption, zeroized after the operation, and excluded from logs by an
      explicit tracing skip-all guard. A CI test grep-asserts the descriptor
      string does not appear in any log output.

    ### What BED does NOT protect against

    > **Golden rule (repeated):** Never co-locate a `.bed` file with any
    > cosigner xpub of the same multisig. If an attacker finds both, they can
    > decrypt the `.bed` and learn the full wallet structure. The security
    > model assumes each `.bed` copy lives in a different physical location
    > than the xpub needed to decrypt it.

    - **Compromise of the StartOS device during an active encrypt session.**
      The descriptor passes through the device's memory in cleartext during
      encryption. If the device is compromised at that moment (malicious app,
      physical access, kernel-level attacker) the descriptor may be exposed.
      BED cannot protect against an attacker who controls the execution
      environment.
    - **Loss of all cosigner xpubs simultaneously.** If every xpub needed to
      decrypt is lost or destroyed, the `.bed` becomes undecryptable. BED
      provides redundancy for *distribution*, not a substitute for independent
      xpub backups.
    - **An attacker who already holds one cosigner xpub.** A `.bed` encrypted
      with xpub A can be decrypted by anyone holding xpub A. The model is
      sound only when each co-location (`.bed` + `xpub_N`) is in a different
      physical location.
    - **Side-channel and traffic analysis.** BED runs locally on StartOS over
      Tor or LAN. It does not defend against advanced traffic-analysis
      adversaries who already control the local network.
    - **Hardware wallet support.** USB device support is intentionally
      omitted (StartOS 0.4.0 does not pass USB into containers). Use a
      software wallet to extract the descriptor; sign with hardware after.

    ### Model assumptions

    - The StartOS device is trusted during the session.
    - Each `.bed` copy lives in a different physical location than any xpub
      that could decrypt it.
    - The `.bed` file format integrity is guaranteed by AES-256-GCM
      authentication — any tampering is detected.
    - The cosigner xpub used to encrypt is not also the only one that
      decrypts; in a 2-of-3 multisig you typically encrypt one `.bed` per
      cosigner using a *different* cosigner's xpub each time.

    ## Crypto Details

    | Property | Value |
    |---|---|
    | Encryption | AES-256-GCM |
    | Magic header | `BEB` (binary identifier) |
    | KDF | Per BIP draft (xpub-derived; see PR for details) |
    | Spec | [bitcoin/bips PR #1951](https://github.com/bitcoin/bips/pull/1951) — draft |
    | Implementation | [`bitcoin-encrypted-backup`](https://github.com/pythcoiner/encrypted_backup) crate, pinned at `v0.0.2` (rev `cd7ee382bf5ca0798d4f81697e2f9efb5e32fe40`) |
    | Interop | [Liana](https://wizardsardine.com/liana/) wallet v13+ reads `.bed` files produced here, and BED reads `.bed` files produced by Liana |
    | Descriptor requirement | Must use the BIP-380 multipath wildcard `<0;1>/*`. Without it, spending from address 0 exposes the xpub on-chain and breaks the encryption model. |

    The `.bed` format is an **external contract**: BED will not break it
    without a new milestone. If a future BIP revision changes the format
    (e.g. ChaCha20-Poly1305 instead of AES-GCM), the old version is
    archived in CHANGELOG and a separate "BED v0.x archive" branch is kept
    available for decrypting legacy files.

    ## Common Pitfalls

    1. **Descriptor without `<0;1>/*`.** BED rejects single-path
       descriptors with a typed validation error. Convert your descriptor
       to multipath form before encrypting.
    2. **xpub vs descriptor-style format.** The Decrypt tab expects a *bare*
       xpub (e.g. `xpub6FHa3...`). Descriptor-style xpubs with a
       `[fingerprint/path]` prefix or a `/*` suffix are rejected. Strip the
       prefix and suffix before pasting.
    3. **QR size limit (~2,900 bytes ECC-L).** Multisig descriptors with
       five or more cosigners may produce armored payloads that exceed the
       QR ECC-L capacity. BED returns a descriptive error in this case
       instead of an unreadable QR — use the binary or armored output
       and rely on cold-storage paper for QR-bound transport.
    4. **History mode default is OFF.** The history toggle is opt-in. If
       you cifrar without enabling it, the `.bed` is returned to the
       browser and the server forgets it immediately. Enable the toggle
       explicitly if you want the file persisted to `/data/encrypted/`.

    ## References

    - BIP draft PR [bitcoin/bips#1951](https://github.com/bitcoin/bips/pull/1951)
    - Crate [`bitcoin-encrypted-backup`](https://github.com/pythcoiner/encrypted_backup) (tag `v0.0.2`)
    - [Delving Bitcoin thread](https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607)
    - [Liana wallet documentation](https://wizardsardine.com/liana/)
    - StartOS s9pk wrapper: [`semillabitcoin/bed-startos`](https://github.com/semillabitcoin/bed-startos)

    ## License

    MIT. See [LICENSE](LICENSE).
    ```

    Implementation notes:
    - Use the literal text above. Do not paraphrase the golden-rule blockquotes — they are fixed strings the acceptance test grep-validates.
    - The phrase "never co-locate" MUST appear at least twice in the file (once in TL;DR, once in §What BED does NOT protect against). The grep test counts occurrences.
    - Do NOT add a screenshots subsection to Usage in v1 — RESEARCH.md recommends real screenshots after S9-04, deferred to a follow-up. A textual step-by-step is sufficient.
    - The repo URL `https://github.com/semillabitcoin/bed-startos` is referenced as the install path. Plan 04 creates that repo; until then the link points to the future location, which is fine — the README is for the v0.1.0 release after Plan 04 completes.
    - Cargo.toml has no LICENSE file at repo root currently. If `cat /workspace/descriptor-cifrado/LICENSE` returns "no such file", create LICENSE with the standard MIT text using the noreply email and "Semilla Bitcoin" as the copyright holder. The README links to it; missing LICENSE is a broken link.
  </action>
  <verify>
    <automated>test -f /workspace/descriptor-cifrado/README.md &amp;&amp; grep -c 'never co-locate' /workspace/descriptor-cifrado/README.md | grep -qE '^([2-9]|[1-9][0-9]+)$' &amp;&amp; grep -q '^## Threat Model' /workspace/descriptor-cifrado/README.md &amp;&amp; grep -q 'AES-256-GCM' /workspace/descriptor-cifrado/README.md &amp;&amp; grep -q 'bitcoin/bips#1951\|bitcoin/bips/pull/1951' /workspace/descriptor-cifrado/README.md</automated>
  </verify>
  <acceptance_criteria>
    - File `/workspace/descriptor-cifrado/README.md` exists.
    - `grep -c 'never co-locate' README.md` outputs `2` or higher (DOC-02 redundancy per D-03).
    - File contains literal heading `## Threat Model` (DOC-01).
    - File contains sub-headings `### What BED protects` AND `### What BED does NOT protect against` AND `### Model assumptions`.
    - File contains the literal string `AES-256-GCM`.
    - File contains a link to `https://github.com/bitcoin/bips/pull/1951` (BIP draft).
    - File contains the literal string `cd7ee382` or `v0.0.2` referencing the crate pin.
    - File contains the literal string `<0;1>/*` (descriptor multipath wildcard).
    - File contains the heading `## Common Pitfalls`.
    - `wc -l README.md` outputs at least 80 lines.
    - If `LICENSE` did not exist before this task, it now exists at repo root with a standard MIT license text containing "Semilla Bitcoin" or the noreply identity.
  </acceptance_criteria>
  <done>The descriptor-cifrado repo has a publishable README in English satisfying DOC-01 (explicit threat model with three sub-sections) and DOC-02 (golden rule appears at least twice). The bed-startos README in Plan 04 can safely link to `#threat-model` and `#crypto-details` anchors.</done>
</task>

</tasks>

<verification>
- README.md exists at descriptor-cifrado repo root.
- Threat Model section is explicit and structured (3 sub-sections).
- Golden rule appears at least twice.
- Crypto Details lists AES-256-GCM, BIP PR 1951, crate v0.0.2.
- LICENSE file exists (created if missing).
</verification>

<success_criteria>
A reader visiting github.com/semillabitcoin/descriptor-cifrado after v0.1.0 release sees a README that explains what BED is, how to use it, what it protects against, what it does NOT protect against, the crypto details, common pitfalls, and external references. The golden rule is impossible to miss (twice in the doc).
</success_criteria>

<output>
After completion, create `.planning/phases/04-startos-packaging-docs/04-02-SUMMARY.md` recording:
- README.md line count and section list
- LICENSE file status (existed / created)
- Verification of golden-rule occurrence count
</output>
