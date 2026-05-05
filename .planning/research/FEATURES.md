# Feature Research

**Domain:** Bitcoin descriptor encryption/decryption — local-first StartOS web app
**Researched:** 2026-05-05
**Confidence:** HIGH (encryption flow and BIP constraints from primary sources; UX patterns from analogous tools)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

#### Encrypt Flow

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Textarea to paste descriptor | Primary input method for a web tool — no native file association for descriptors | LOW | Must accept multiline miniscript descriptors; strip leading/trailing whitespace before parsing |
| Produce `.bed` binary download | The BIP output format; what the Liana v13 recovery flow and pythcoiner CLI both produce | LOW | Triggered immediately after encryption; browser `Content-Disposition: attachment` |
| Produce armored text output | Analogous to PGP ASCII armor — enables copy/paste transport without binary file handling | LOW | `-----BEGIN BITCOIN ENCRYPTED BACKUP-----` / `-----END BITCOIN ENCRYPTED BACKUP-----` header/footer per BIP draft; base64-encoded payload |
| Produce QR PNG download | Enables paper backup and air-gapped scanning; derived from the armored base64 payload | MEDIUM | QR encodes the armored string (not raw binary); PNG download via canvas or image/png response; only feasible if armored payload fits QR capacity (~2900 bytes at ECC-L) |
| Error display on invalid input | Users will paste wrong formats, raw xpubs, or truncated strings | LOW | Inline error, not an alert box; explain what went wrong specifically |
| Reject descriptors missing `<0;1>/*` derivation | BIP requirement — without this, spending from index 0 exposes the xpub on-chain and breaks encryption | LOW | The `bitcoin-encrypted-backup` crate enforces this; surface the rejection as a user-readable message, not a panic |
| Clear/reset button | After a successful encrypt, user needs to start fresh without reloading page | LOW | Clear textarea and output area |

#### Decrypt Flow

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Upload `.bed` binary file | Primary input for decrypt; Liana v13 uses this as the recovery entry point | LOW | File input `<input type="file">`; accept `.bed` or `*` (users may rename) |
| Paste armored text as decrypt input | Users who stored the armored string (email, note) need to decrypt without a binary file | LOW | Detect `-----BEGIN BITCOIN ENCRYPTED BACKUP-----` header on paste; auto-switch between file and text input modes |
| xpub text input for decryption key | Any cosigner can decrypt with their xpub — this is the 1-of-N property | LOW | `xpub...` or `[fingerprint/path]xpub...` format; accept with or without key origin prefix |
| Display recovered descriptor in cleartext | Primary output of decryption — what the user came for | LOW | Monospace textarea; selectable for copy |
| Copy-to-clipboard button for recovered descriptor | Users need to paste it into Liana, Sparrow, Nunchuk, etc. | LOW | `navigator.clipboard.writeText()`; show "Copied!" feedback |
| Error on wrong xpub | User supplies xpub that is not a cosigner — AEAD authentication tag fails | LOW | ChaCha20-Poly1305 authentication failure; show "Decryption failed: xpub is not a cosigner of this backup" |
| Memory clearing after decrypt | Descriptor in cleartext should not persist in app state beyond session | MEDIUM | Zero the buffer after displaying; do not log to server; no server-side storage of cleartext |

#### General UX

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Tab or section separation (Encrypt / Decrypt) | Two distinct flows in one app; mixing them creates confusion | LOW | Two tabs or two clearly labeled sections on the same page |
| Inline loading state | Rust/axum backend may take 50–200 ms for crypto; user needs feedback | LOW | Spinner or "Encrypting..." text while awaiting response |
| Threat model notice visible in UI | Security-critical tool — users must understand limitations before trusting it | LOW | Brief inline callout (not a modal): "The descriptor passes through server memory during encryption. Do not use on an untrusted StartOS instance." |

---

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Syntax validation with human-readable feedback | pythcoiner CLI gives no custom error messages; Liana wraps errors opaquely | MEDIUM | Before encrypting, call a `/validate` endpoint (or validate inline on Rust side) that checks descriptor parse, required derivation path `<0;1>/*`, and checksum. Return specific error: "Missing `<0;1>/*` derivation" vs "Checksum mismatch" vs "Unknown script type" |
| "Test decrypt" round-trip before showing success | Proves the produced `.bed` actually decrypts — prevents user from storing a corrupt backup | MEDIUM | After encrypt, the backend silently decrypts with a randomly chosen key from the descriptor and verifies payload matches; reports "Backup verified" or "Verification failed" in the response. Does NOT expose the xpub used. |
| Drag-and-drop `.bed` file onto decrypt form | Expected UX in 2026; reduces friction for users who open the file manager alongside the browser | LOW | Drop zone on decrypt tab; `dragover`/`drop` events; visual highlight on hover |
| Opt-in history mode with list and delete | Enables users to keep an audit trail of backups produced, without requiring external file management | MEDIUM | Toggle persists preference in a cookie or localStorage; backend stores `.bed` in `/data/encrypted/<timestamp>-<short-id>.bed`; list endpoint returns entries; delete endpoint removes individual files. Descriptor in cleartext NEVER stored. |
| Descriptor checksum display | Lets user verify they pasted the right descriptor before encrypting | LOW | Show the 8-character BIP-380 `#CHECKSUM` suffix; compute in Rust before encrypt; display below the textarea |
| Armored output has one-click copy | Enables sharing armored payload via email or chat without downloading a file | LOW | "Copy armored" button next to the armored textarea; same `navigator.clipboard` pattern |
| QR PNG has high-contrast styling with version hint | QR for descriptors can be large (version 25+); explicit guidance prevents scan failures | LOW | Label the downloaded PNG with the QR version/error correction level; use black-on-white; no logo overlay |
| StartOS-native dark/light theme following system preference | Consistent with StartOS UI; reduces jarring contrast when using alongside other apps | LOW | `prefers-color-scheme` CSS media query; no JS required |

---

### Anti-Features (Deliberately NOT in v1)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Hardware wallet (HWW) xpub fetch for decrypt | Liana v13 does this automatically; users expect it | USB does not reach inside the StartOS container in v0.4.0; implementing USB passthrough requires `devices` crate feature + host configuration outside app scope | User pastes xpub manually from their HWW companion app or Sparrow |
| File Browser integration (open/save `.bed` from StartOS file tree) | Seamless file management without browser downloads | StartOS 0.4.0 File Browser APIs unverified — HTTP API scoping, shared volumes, and `user-files` path are unknown; integrating on unverified APIs risks silent breakage | Browser download/upload works out-of-the-box; defer to v2 after APIs are verified |
| Camera / webcam QR scanner for decrypt input | Mobile-friendly; avoids file transfer | Browser camera access over Tor onion or LAN requires HTTPS (secure context); StartOS LAN may lack valid TLS cert for camera API to work; high complexity for marginal gain | User can paste armored text or upload `.bed` file; defer camera scan to v2 |
| Arbitrary data encryption (non-descriptor payloads) | The underlying crate supports arbitrary bytes | Scope creep; threat model and UX are designed around descriptors; arbitrary encryption requires different validation, different error messages, different user guidance | Stay focused: encrypt only Bitcoin output descriptors |
| Shamir Secret Sharing / k-of-n threshold decrypt | Discussed in Delving Bitcoin thread; conceptually appealing | Does not generalize to miniscript policies per BIP author (Salvatoshi); fundamentally different cryptographic model; requires entirely different UX | The BIP's 1-of-N model (any cosigner can decrypt) already provides redundancy; Shamir is a separate BIP-level concern |
| Multi-user auth / user accounts | Power users might want per-user history | Auth is delegated to StartOS Tor onion + StartOS auth layer; adding app-level auth creates duplicate, potentially weaker security surface | Trust the StartOS auth model; all users of the instance are trusted equally |
| Export to Keystone / Coldcard SD card format | Some users manage air-gapped devices | Different binary format per device; out of scope for v1; would require per-device format research | Download `.bed` binary; user copies to SD card manually |
| Descriptor normalization / round-tripping through Bitcoin Core | Some wallets export descriptors with different whitespace or key origin formats | Normalization can silently change the descriptor (Bitcoin Core collapses key origin paths, losing embedded info per Delving Bitcoin thread); creates unexpected behavior | Validate and encrypt the descriptor as-is; document that pasted descriptor must match the wallet's canonical form |
| Real-time descriptor parsing as user types | IDE-like experience | Sends partial, invalid descriptors to backend on every keystroke; expensive; Rust parse errors on partial input are noisy | Parse on explicit "Encrypt" button press; show validation errors then |

---

## Feature Dependencies

```
[Armored output]
    └──enables──> [QR PNG output]  (QR encodes the armored base64 string)
    └──enables──> [One-click copy armored]

[Descriptor validation]
    └──required by──> [Encrypt flow]  (reject before calling crate)
    └──enables──> [Checksum display]

[Armored text paste on decrypt input]
    └──requires──> [Auto-detect format]  (binary file vs armored text; detect by header presence)

["Test decrypt" round-trip]
    └──requires──> [Encrypt flow completes successfully]
    └──uses──> [Decrypt logic internally]  (same Rust function; not a second HTTP call from frontend)

[Opt-in history mode]
    └──requires──> [Backend /data/encrypted/ volume mounted]
    └──enables──> [List history endpoint]
    └──enables──> [Delete history entry endpoint]

[Memory clearing after decrypt]
    └──requires──> [Decrypt flow produces cleartext]  (clear after response is sent to client)
```

### Dependency Notes

- **Armored output enables QR:** The QR encodes the armored base64 string (not raw binary), so armored output must be produced first. If the armored string exceeds ~2900 bytes (QR ECC-L capacity), the backend should return an error rather than produce an unscannable QR.
- **Validation required before encryption:** The `bitcoin-encrypted-backup` crate panics or returns an error on malformed descriptors; the HTTP layer must catch these and return structured JSON errors, not HTTP 500.
- **"Test decrypt" uses internal decrypt:** The round-trip verification is done server-side within the same request handler — the frontend receives a `verified: true/false` flag in the encrypt response. No separate client round-trip.
- **History mode conflicts with ephemeral default:** The default is ephemeral (no persistence). The toggle is opt-in; if off, the `/data/encrypted/` directory is never written. The two modes are mutually exclusive for a given session.

---

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed for the StartOS holder to replace the pythcoiner CLI.

- [ ] **Paste descriptor, download `.bed` binary** — the core encrypt action; replaces `beb encrypt -f descriptor.txt`
- [ ] **Armored text output with one-click copy** — enables paper/email transport without binary file handling
- [ ] **QR PNG download** — enables air-gapped paper backup
- [ ] **Reject descriptors missing `<0;1>/*` derivation** — BIP requirement; prevents producing a non-compliant backup
- [ ] **Upload `.bed` or paste armored + xpub → display recovered descriptor** — the full decrypt flow
- [ ] **Copy-to-clipboard for recovered descriptor** — essential for pasting into Liana/Sparrow after recovery
- [ ] **Clear error messages (encrypt and decrypt)** — wrong format, wrong xpub, malformed backup
- [ ] **Threat model callout in UI** — security tool requires informed users
- [ ] **Opt-in history mode (toggle, list, delete)** — enables redundant backup management without external file manager

### Add After Validation (v1.x)

Features to add once core flows are stable and tested on real StartOS hardware.

- [ ] **Drag-and-drop `.bed` onto decrypt form** — trigger: users report friction with file picker; easy to add after initial ship
- [ ] **"Test decrypt" round-trip verification** — trigger: user reports uncertainty about backup validity; medium complexity, high trust value
- [ ] **Descriptor checksum display** — trigger: users paste wrong descriptor version; low complexity enhancement
- [ ] **Syntax validation with specific error messages** — trigger: support requests about "why did encryption fail?"; requires careful Rust error mapping

### Future Consideration (v2+)

Features to defer until v1 is validated on real hardware.

- [ ] **File Browser integration** — trigger: StartOS 0.4.0 File Browser API documented and stable
- [ ] **Camera/webcam QR scan for decrypt** — trigger: mobile usage confirmed; HTTPS cert situation resolved
- [ ] **Arbitrary data encryption** — trigger: users explicitly request non-descriptor use cases

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Paste descriptor → `.bed` download | HIGH | LOW | P1 |
| Armored text output | HIGH | LOW | P1 |
| QR PNG download | HIGH | MEDIUM | P1 |
| `<0;1>/*` validation + clear error | HIGH | LOW | P1 |
| Upload `.bed` + xpub → descriptor | HIGH | LOW | P1 |
| Copy recovered descriptor | HIGH | LOW | P1 |
| Opt-in history (toggle, list, delete) | MEDIUM | MEDIUM | P1 |
| Threat model callout | HIGH | LOW | P1 |
| Drag-and-drop on decrypt | MEDIUM | LOW | P2 |
| "Test decrypt" round-trip | HIGH | MEDIUM | P2 |
| Descriptor checksum display | LOW | LOW | P2 |
| Specific syntax error messages | MEDIUM | MEDIUM | P2 |
| File Browser integration | MEDIUM | HIGH | P3 |
| Camera QR scan | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | pythcoiner CLI (`beb`) | pythcoiner GUI (`bed`) | Liana v13 | This App |
|---------|------------------------|------------------------|-----------|----------|
| Encrypt descriptor | Yes (`encrypt` subcommand) | Yes (GUI) | Yes (wallet creation) | Yes |
| Decrypt with xpub | Yes (`-k` flag) | Yes (GUI) | Yes (hardware wallet auto-fetch) | Yes (manual xpub paste) |
| Binary `.bed` output | Yes | Yes | Yes | Yes |
| Armored text output | Unclear (not documented) | Unclear | No | Yes |
| QR PNG output | No | Unclear | No | Yes |
| Hardware wallet xpub fetch | Yes (with `devices` feature) | Likely yes | Yes (auto) | No (v1 scope limit) |
| Descriptor validation error messages | Minimal (Rust unwrap) | Unknown | Embedded in wizard | Specific messages |
| History / audit log | No | No | No | Yes (opt-in) |
| "Test decrypt" verification | No | No | No | Yes (v1.x) |
| Web interface / zero-install | No | No | No (desktop app) | Yes (StartOS local web) |
| Local-only / no telemetry | Yes | Yes | Yes | Yes |
| Tor access | No | No | No | Yes |

---

## Sources

- BIP draft PR #1951 discussion: https://github.com/bitcoin/bips/pull/1951 — encryption scheme, ChaCha20-Poly1305, key derivation, `<0;1>/*` requirement (HIGH confidence)
- Delving Bitcoin thread: https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607 — feature requests, edge cases, design rationale from community (HIGH confidence)
- Liana v13 release: https://wizardsardine.com/blog/liana-13.0-release/ — `.bed` default, recovery flow, hardware wallet auto-fetch, plaintext descriptor still copyable (HIGH confidence)
- pythcoiner CLI repo: https://github.com/pythcoiner/encrypted_backup — `encrypt`/`decrypt` subcommands, `-f`/`-k`/`-o` flags, device feature, no custom error messages (HIGH confidence, direct repo inspection)
- pythcoiner GUI repo: https://github.com/pythcoiner/bed — Qt6 + Rust, demo GIF referenced but content undescribed in README; features inferred from project scope (MEDIUM confidence)
- OpenPGP ASCII armor spec: https://openpgp.dev/book/armor.html — armored format UX patterns, copy-paste whitespace sensitivity (HIGH confidence)
- BIP-380 descriptor checksum: https://en.bitcoin.it/wiki/BIP_0380 — 8-character checksum format, optional but recommended (HIGH confidence)
- Miniscript Studio: https://adys.dev/miniscript — web tool UX patterns for descriptor/miniscript validation (MEDIUM confidence, analogous tool)
- StartOS 0.4.0 overview: https://github.com/Start9Labs/start-os — platform constraints, no USB passthrough to containers (MEDIUM confidence)

---

*Feature research for: BED Start9 App — Bitcoin Encrypted Backup descriptor tool*
*Researched: 2026-05-05*
