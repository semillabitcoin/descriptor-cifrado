# Changelog

All notable changes to BED — Bitcoin Encrypted Backup.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The Docker image is published per release as
`ghcr.io/semillabitcoin/descriptor-cifrado:<version>` (multi-arch
amd64 + arm64). Each release pins an immutable `@sha256:...` digest
referenced by the [`bed-startos`](https://github.com/semillabitcoin/bed-startos)
s9pk wrapper.

---

## [0.3.0] — 2026-05-09

### Added
- **Liana JSON passthrough**: BED now encrypts and decrypts wallet
  exports from Liana producción (JSON format with nested descriptor),
  preserving metadata. Round-trip is byte-identical. Compatible interop
  with `.bed` files produced by Liana 0.0.2.
- **Sparrow BIP329 JSONL passthrough**: BED also encrypts and decrypts
  exports from Sparrow Wallet (multi-line JSONL with labels per
  [BIP-329](https://github.com/bitcoin/bips/blob/master/bip-0329.mediawiki)),
  preserving labels intact. The decrypt panel visually distinguishes
  Sparrow exports from classic / Liana payloads.
- **QR graceful fallback**: when an encrypted descriptor exceeds the QR
  code size limit (~2,900 bytes ECC-L), the page no longer aborts.
  Binary and ASCII-armored downloads remain available; only the QR
  panel degrades cleanly to a "QR not available — payload too large"
  state.
- **File picker accepts more formats**: the "select file" dialog in the
  Encrypt tab now accepts `.txt`, `.descriptor`, `.json`, and `.jsonl`
  (drag-and-drop already supported these).

### Changed
- Removed the placeholder `Mi multisig 3 de 5` from the Name field —
  the helper text already explains the field's purpose.
- Form label `Descriptor multisig` simplified to `Descriptor` (the form
  accepts any descriptor with the BIP-380 multipath wildcard, not only
  multisig).

### Internal
- `EncryptOutput.qr_png: Vec<u8>` → `Option<Vec<u8>>` (None on
  `QrTooLarge`). Server JSON `qr_png_b64` becomes optional. Frontend
  conditionally renders the QR panel.
- `crates/core/src/sparrow.rs`: new module with `parse_sparrow_jsonl`
  and `compose_descriptor_from_sparrow` for BIP-329 detection and
  composition.
- Test suite: 34 bed-core + 28 bed-server tests, all green.

### Image digest
`ghcr.io/semillabitcoin/descriptor-cifrado@sha256:1ded4e601c079b3c2d9da8c99461feef542f2f464216a8d7a670288fb8e2c7ae`

---

## [0.2.0] — 2026-05-08

### Added
- Descriptor name is now required when encrypting. Download filename
  is `<name>.bed` (no timestamp); history list shows the name as
  primary identifier with the on-disk filename as subtitle.
- History tab refreshes automatically after a successful encryption.

### Notes
- Allowed charset for the name: letters, numbers, spaces, dashes and
  underscores (max 80 chars).
- v0.1.5 was skipped — its history-refresh fix shipped as part of this
  release.

### Image digest
`ghcr.io/semillabitcoin/descriptor-cifrado@sha256:a2b61674bc890299becfab8915e960b648679f58db5d920c26ae7c3bfdc20117`

---

## [0.1.0–0.1.4]

Initial public releases. v0.1.4 was the first usable release after
fixing the container UID issue that prevented history persistence.
See git tag history for per-release details.
