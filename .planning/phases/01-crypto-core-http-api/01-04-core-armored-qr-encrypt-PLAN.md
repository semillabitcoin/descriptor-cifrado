---
phase: 01-crypto-core-http-api
plan: 04
type: execute
wave: 2
depends_on: ["01-01"]
files_modified:
  - crates/core/src/lib.rs
  - crates/core/src/armored.rs
  - crates/core/src/qr.rs
  - crates/core/src/encrypt.rs
  - crates/core/src/decrypt.rs
  - crates/core/tests/armored.rs
  - crates/core/tests/round_trip.rs
  - crates/core/tests/fixtures/wallet.bed
  - crates/core/tests/fixtures/xpub.txt
autonomous: true
requirements: [CORE-01, CORE-02, ENC-03, ENC-04, DEC-05]
must_haves:
  truths:
    - "encode_armored produce headers exactos `-----BEGIN BITCOIN ENCRYPTED BACKUP-----` + line-wrap 64 chars"
    - "decode_armored tolera espacios, indentación, BOM, \\r\\n vs \\n (DEC-05)"
    - "render_qr_png devuelve PNG bytes para armored ≤ 2900 B; falla con CoreError::QrTooLarge si mayor"
    - "encrypt_descriptor + decrypt_payload round-trip produce el mismo descriptor (CORE-02)"
    - "Crate bitcoin_encrypted_backup importable y EncryptedBackup builder usable"
  artifacts:
    - path: "crates/core/src/armored.rs"
      provides: "Encoder + decoder PEM-style con tolerancia"
      exports: ["encode_armored", "decode_armored", "ARMOR_BEGIN", "ARMOR_END"]
    - path: "crates/core/src/qr.rs"
      provides: "QR PNG generator con cap 2900 B"
      exports: ["render_qr_png", "MAX_QR_BYTES"]
    - path: "crates/core/src/encrypt.rs"
      provides: "encrypt_descriptor wrapper sobre crate"
      exports: ["encrypt_descriptor", "EncryptOutput"]
    - path: "crates/core/src/decrypt.rs"
      provides: "decrypt_payload wrapper sobre crate"
      exports: ["decrypt_payload"]
    - path: "crates/core/tests/round_trip.rs"
      provides: "Test integración encrypt+decrypt con fixture"
      contains: "fn round_trip"
  key_links:
    - from: "crates/core/src/encrypt.rs"
      to: "bitcoin_encrypted_backup::EncryptedBackup"
      via: "EncryptedBackup::new().set_payload(&desc)?.encrypt()?"
      pattern: "EncryptedBackup::new"
    - from: "crates/core/src/decrypt.rs"
      to: "bitcoin_encrypted_backup::Decrypted"
      via: "set_encrypted_payload + set_keys + decrypt"
      pattern: "Decrypted::Descriptor"
    - from: "crates/core/src/armored.rs"
      to: "base64::engine::general_purpose::STANDARD"
      via: "encode/decode"
      pattern: "STANDARD"
---

<objective>
Implementar el wrapper alrededor de `bitcoin-encrypted-backup` (`encrypt_descriptor` y `decrypt_payload`), el encoder/decoder armored estilo PGP (D-12), y el generador QR PNG con cap 2900 B (D-14, D-15). Tests de armored con tolerancia a whitespace/CRLF (DEC-05) + round-trip determinista usando los fixtures de `/tmp/bed-test/`.

Purpose: Cerrar las primitivas core de cifrado/decifrado y los formatos de salida (armored + QR) — todo lo que el server consumirá, en una capa puramente unit-testable sin HTTP.
Output: `crates/core` completo; `cargo test -p bed-core` pasa todos los tests, incluyendo el round-trip de fixture real.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/01-crypto-core-http-api/01-CONTEXT.md
@.planning/phases/01-crypto-core-http-api/01-RESEARCH.md
@/tmp/bed-test/encrypted_backup/src/lib.rs
@/tmp/bed-test/encrypted_backup/src/descriptor.rs
@/tmp/bed-test/desc.txt
@/tmp/bed-test/wallet.bed
@/tmp/bed-test/xpub.txt
@/tmp/bed-test/key1.txt

<interfaces>
From bitcoin_encrypted_backup (verified at /tmp/bed-test/encrypted_backup/src/lib.rs):

```rust
// Encrypt path (with `rand` feature, nonce arg omitted)
let bed_bytes: Vec<u8> = EncryptedBackup::new()
    .set_payload(&desc)?       // populates content+derivation_paths+keys from descriptor
    .encrypt()?;               // ChaCha20-Poly1305

// Decrypt path
let dpk = DescriptorPublicKey::from_str(xpub_str)?;
let pk = bitcoin_encrypted_backup::descriptor::dpk_to_pk(&dpk);
let restored: Decrypted = EncryptedBackup::new()
    .set_encrypted_payload(&bed_bytes)?  // auto-detects binary "BIPXXX" magic vs base64
    .set_keys(vec![pk])
    .decrypt()?;
let cleartext: String = match restored {
    Decrypted::Descriptor(d) => d.to_string(),
    _ => return Err(CoreError::Crypto),
};
```

Patterns from RESEARCH.md:
- §"Pattern 2: Armored encoder/decoder" — full encode/decode body
- §"Pattern 3: QR PNG generation" — full render_qr_png body using image::ImageFormat::Png (NOT ImageOutputFormat — deprecated)
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implementar armored encoder/decoder + tests de tolerancia (DEC-05)</name>
  <files>crates/core/src/armored.rs, crates/core/tests/armored.rs, crates/core/src/lib.rs</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Pattern 2: Armored encoder/decoder" — bloque verbatim)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-12, D-13, DEC-05)
  </read_first>
  <behavior>
    - encode_armored(b"hello world") devuelve string que comienza con `-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n`, termina con `-----END BITCOIN ENCRYPTED BACKUP-----\n`
    - encode_armored line-wrapping: payloads >64 chars base64 tienen \n cada 64 chars
    - decode_armored ∘ encode_armored == identidad
    - decode_armored tolera: leading spaces (`  -----BEGIN...`), trailing spaces, indented payload lines, BOM (`\u{FEFF}`), CRLF line endings
    - decode_armored rechaza: header con texto extra (`-----BEGIN PGP MESSAGE-----`), payload vacío, base64 inválido
  </behavior>
  <action>
    Crear `crates/core/src/armored.rs` con el contenido EXACTO de RESEARCH.md §"Pattern 2: Armored encoder/decoder":

    ```rust
    //! PEM-style armored encoder/decoder for .bed binary payloads (D-12).
    //!
    //! The bitcoin-encrypted-backup crate provides only base64 (single line via
    //! `encrypt_base64`); it does NOT provide PEM headers. This module wraps
    //! base64 with `-----BEGIN/END BITCOIN ENCRYPTED BACKUP-----` and line-wrap.

    use base64::{engine::general_purpose::STANDARD, Engine as _};

    pub const ARMOR_BEGIN: &str = "-----BEGIN BITCOIN ENCRYPTED BACKUP-----";
    pub const ARMOR_END: &str = "-----END BITCOIN ENCRYPTED BACKUP-----";
    const LINE_WIDTH: usize = 64;

    pub fn encode_armored(bed_bytes: &[u8]) -> String {
        let b64 = STANDARD.encode(bed_bytes);
        let mut out = String::with_capacity(b64.len() + 128);
        out.push_str(ARMOR_BEGIN);
        out.push('\n');
        for chunk in b64.as_bytes().chunks(LINE_WIDTH) {
            // base64 alphabet is pure ASCII → from_utf8 cannot fail; use a defensive
            // fallback that returns empty rather than panic (lint forbids unwrap/expect).
            let s = std::str::from_utf8(chunk).unwrap_or("");
            out.push_str(s);
            out.push('\n');
        }
        out.push_str(ARMOR_END);
        out.push('\n');
        out
    }

    #[derive(thiserror::Error, Debug)]
    pub enum ArmoredError {
        #[error("missing or wrong BEGIN header")]
        WrongHeader,
        #[error("missing or wrong END footer")]
        WrongFooter,
        #[error("empty payload between headers")]
        EmptyPayload,
        #[error("invalid base64 payload")]
        Base64,
    }

    impl From<ArmoredError> for crate::CoreError {
        fn from(e: ArmoredError) -> Self {
            crate::CoreError::Armored(e.to_string())
        }
    }

    pub fn decode_armored(input: &str) -> Result<Vec<u8>, ArmoredError> {
        // Strip BOM if present
        let s = input.strip_prefix('\u{FEFF}').unwrap_or(input);
        let mut payload_lines: Vec<&str> = Vec::new();
        let mut in_block = false;
        for raw_line in s.lines() {
            let line = raw_line.trim();
            if line.is_empty() { continue; }
            if line.starts_with("-----BEGIN") {
                if line == ARMOR_BEGIN {
                    in_block = true;
                    continue;
                }
                return Err(ArmoredError::WrongHeader);
            }
            if line.starts_with("-----END") {
                if line == ARMOR_END {
                    break;
                }
                return Err(ArmoredError::WrongFooter);
            }
            if in_block {
                payload_lines.push(line);
            }
        }
        if payload_lines.is_empty() {
            return Err(ArmoredError::EmptyPayload);
        }
        let joined: String = payload_lines.concat();
        STANDARD
            .decode(joined.as_bytes())
            .map_err(|_| ArmoredError::Base64)
    }
    ```

    NOTA: el `unwrap_or("")` no es `.unwrap()` y NO es rechazado por el lint `unwrap_used`.

    Actualizar `crates/core/src/lib.rs` para exportar:
    ```rust
    pub mod armored;
    pub use armored::{encode_armored, decode_armored, ArmoredError, ARMOR_BEGIN, ARMOR_END};
    ```

    Crear `crates/core/tests/armored.rs`:
    ```rust
    use bed_core::armored::{decode_armored, encode_armored, ArmoredError, ARMOR_BEGIN, ARMOR_END};

    const PAYLOAD: &[u8] = b"the quick brown fox jumps over the lazy dog 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    #[test]
    fn round_trip_identity() {
        let armored = encode_armored(PAYLOAD);
        let decoded = decode_armored(&armored).unwrap_or_else(|e| panic!("decode failed: {e}"));
        assert_eq!(decoded, PAYLOAD);
    }

    #[test]
    fn headers_present_exactly() {
        let armored = encode_armored(PAYLOAD);
        assert!(armored.starts_with(&format!("{ARMOR_BEGIN}\n")));
        assert!(armored.ends_with(&format!("{ARMOR_END}\n")));
    }

    #[test]
    fn line_wrap_64_chars() {
        // Use a payload large enough to trigger wrap (>48 bytes = >64 base64 chars)
        let big = vec![0xABu8; 200];
        let armored = encode_armored(&big);
        for line in armored.lines() {
            if line.starts_with("-----") { continue; }
            assert!(line.len() <= 64, "line too long: {} chars: {line}", line.len());
        }
    }

    #[test]
    fn tolerates_crlf() {
        let armored = encode_armored(PAYLOAD);
        let crlf = armored.replace('\n', "\r\n");
        let decoded = decode_armored(&crlf).unwrap_or_else(|e| panic!("crlf decode: {e}"));
        assert_eq!(decoded, PAYLOAD);
    }

    #[test]
    fn tolerates_indentation_and_trailing_spaces() {
        let armored = encode_armored(PAYLOAD);
        let mut indented = String::new();
        for line in armored.lines() {
            indented.push_str("    ");
            indented.push_str(line);
            indented.push_str("   \n"); // trailing spaces
        }
        let decoded = decode_armored(&indented).unwrap_or_else(|e| panic!("indent decode: {e}"));
        assert_eq!(decoded, PAYLOAD);
    }

    #[test]
    fn tolerates_bom() {
        let armored = encode_armored(PAYLOAD);
        let with_bom = format!("\u{FEFF}{armored}");
        let decoded = decode_armored(&with_bom).unwrap_or_else(|e| panic!("bom decode: {e}"));
        assert_eq!(decoded, PAYLOAD);
    }

    #[test]
    fn rejects_wrong_header() {
        let bad = "-----BEGIN PGP MESSAGE-----\nQUJD\n-----END PGP MESSAGE-----\n";
        assert!(matches!(decode_armored(bad), Err(ArmoredError::WrongHeader)));
    }

    #[test]
    fn rejects_empty_payload() {
        let empty = format!("{ARMOR_BEGIN}\n{ARMOR_END}\n");
        assert!(matches!(decode_armored(&empty), Err(ArmoredError::EmptyPayload)));
    }
    ```
  </action>
  <verify>
    <automated>cargo test -p bed-core --test armored 2>&1 | tail -15</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p bed-core --test armored` exits 0
    - 8 tests pass
    - `crates/core/src/armored.rs` contiene literal `pub const ARMOR_BEGIN: &str = "-----BEGIN BITCOIN ENCRYPTED BACKUP-----";`
    - `crates/core/src/armored.rs` contiene literal `pub const ARMOR_END: &str = "-----END BITCOIN ENCRYPTED BACKUP-----";`
    - `crates/core/src/armored.rs` contiene literal `LINE_WIDTH: usize = 64`
    - `grep -E '\.unwrap\(\)|\.expect\(' crates/core/src/armored.rs | grep -v unwrap_or` retorna 0 matches
  </acceptance_criteria>
  <done>Armored encoder/decoder con tolerancia full (DEC-05) y headers exactos (ENC-03).</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implementar encrypt_descriptor + decrypt_payload + qr.rs + round-trip test (CORE-02)</name>
  <files>crates/core/src/encrypt.rs, crates/core/src/decrypt.rs, crates/core/src/qr.rs, crates/core/tests/round_trip.rs, crates/core/tests/fixtures/wallet.bed, crates/core/tests/fixtures/xpub.txt, crates/core/src/lib.rs</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Crate API Surface" + §"Pattern 3: QR PNG generation")
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-10, D-14, D-15)
    - /tmp/bed-test/encrypted_backup/src/lib.rs (verificar `EncryptedBackup` builder API + `dpk_to_pk` location)
    - /tmp/bed-test/encrypted_backup/src/descriptor.rs (confirmar `pub fn dpk_to_pk`)
  </read_first>
  <behavior>
    - encrypt_descriptor con cleartext válido del fixture devuelve EncryptOutput {bed_bytes: Vec<u8>, armored: String, qr_png: Vec<u8>}
    - decrypt_payload con bed_bytes + xpub válida del fixture devuelve cleartext igual al input
    - decrypt_payload con xpub incorrecta → Err(CoreError::XpubMismatch)
    - render_qr_png con armored ≤ 2900 B → Ok(Vec<u8>) PNG bytes que comienzan con magic PNG `\x89PNG`
    - render_qr_png con armored > 2900 B → Err(CoreError::QrTooLarge { size, max: 2900 })
  </behavior>
  <action>
    Copiar fixtures: `cp /tmp/bed-test/wallet.bed crates/core/tests/fixtures/wallet.bed && cp /tmp/bed-test/xpub.txt crates/core/tests/fixtures/xpub.txt`

    Crear `crates/core/src/qr.rs`:
    ```rust
    //! QR PNG generator (D-14, D-15). ECC-L for max capacity; 2900-byte cap.

    use image::Luma;
    use qrcode::{EcLevel, QrCode};

    use crate::CoreError;

    pub const MAX_QR_BYTES: usize = 2900;

    pub fn render_qr_png(armored: &str) -> Result<Vec<u8>, CoreError> {
        if armored.len() > MAX_QR_BYTES {
            return Err(CoreError::QrTooLarge {
                size: armored.len(),
                max: MAX_QR_BYTES,
            });
        }
        let code = QrCode::with_error_correction_level(armored.as_bytes(), EcLevel::L)
            .map_err(|_| CoreError::Crypto)?;
        let img = code.render::<Luma<u8>>().min_dimensions(256, 256).build();
        let mut buf = std::io::Cursor::new(Vec::<u8>::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|_| CoreError::Crypto)?;
        Ok(buf.into_inner())
    }
    ```

    Crear `crates/core/src/encrypt.rs`:
    ```rust
    //! Encrypt path: descriptor cleartext → .bed binary + armored + QR PNG.
    //!
    //! Receives `&mut Zeroizing<String>` to honor D-10 (no value moves of cleartext).

    use std::str::FromStr;

    use bitcoin_encrypted_backup::{
        miniscript::{Descriptor, DescriptorPublicKey},
        EncryptedBackup,
    };
    use zeroize::Zeroizing;

    use crate::{armored::encode_armored, qr::render_qr_png, validate::require_multipath_0_1, CoreError};

    /// Output triple from a single encrypt call: binary `.bed`, armored PEM string,
    /// and QR PNG bytes. Server serializes these into one JSON response (D-05).
    pub struct EncryptOutput {
        pub bed_bytes: Vec<u8>,
        pub armored: String,
        pub qr_png: Vec<u8>,
    }

    /// Encrypt a cleartext descriptor. Validates `<0;1>/*` first, then calls the
    /// crate to produce binary, wraps to armored, generates QR.
    pub fn encrypt_descriptor(
        cleartext: &mut Zeroizing<String>,
    ) -> Result<EncryptOutput, CoreError> {
        let desc: Descriptor<DescriptorPublicKey> =
            Descriptor::from_str(cleartext.as_str()).map_err(|_| CoreError::DescriptorParse)?;

        require_multipath_0_1(&desc)?;

        let bed_bytes: Vec<u8> = EncryptedBackup::new()
            .set_payload(&desc)?
            .encrypt()?;

        let armored = encode_armored(&bed_bytes);
        let qr_png = render_qr_png(&armored)?;

        Ok(EncryptOutput { bed_bytes, armored, qr_png })
    }
    ```

    Crear `crates/core/src/decrypt.rs`:
    ```rust
    //! Decrypt path: .bed bytes + xpub → cleartext descriptor wrapped in Zeroizing.
    //!
    //! The crate's `set_encrypted_payload` auto-detects binary "BIPXXX" magic vs
    //! base64 — but does NOT strip PEM-style headers. The server is responsible
    //! for stripping headers via `armored::decode_armored` BEFORE calling this.

    use std::str::FromStr;

    use bitcoin_encrypted_backup::{
        descriptor::dpk_to_pk,
        miniscript::DescriptorPublicKey,
        Decrypted, EncryptedBackup,
    };
    use zeroize::Zeroizing;

    use crate::CoreError;

    /// Decrypt a .bed payload (binary or raw base64) using a single xpub. The xpub
    /// is parsed as a DescriptorPublicKey and converted to secp256k1::PublicKey via
    /// the crate's `dpk_to_pk` helper.
    pub fn decrypt_payload(
        bed_bytes: &[u8],
        xpub_str: &str,
    ) -> Result<Zeroizing<String>, CoreError> {
        let dpk =
            DescriptorPublicKey::from_str(xpub_str.trim()).map_err(|_| CoreError::DescriptorParse)?;
        let pk = dpk_to_pk(&dpk);

        let restored: Decrypted = EncryptedBackup::new()
            .set_encrypted_payload(bed_bytes)?
            .set_keys(vec![pk])
            .decrypt()?;

        match restored {
            Decrypted::Descriptor(d) => Ok(Zeroizing::new(d.to_string())),
            _ => Err(CoreError::Crypto),
        }
    }
    ```

    Actualizar `crates/core/src/lib.rs` para exportar todo:
    ```rust
    pub use bitcoin_encrypted_backup::miniscript;

    pub mod armored;
    pub mod decrypt;
    pub mod encrypt;
    pub mod error;
    pub mod qr;
    pub mod validate;
    pub mod zeroize;

    pub use armored::{decode_armored, encode_armored, ArmoredError, ARMOR_BEGIN, ARMOR_END};
    pub use decrypt::decrypt_payload;
    pub use encrypt::{encrypt_descriptor, EncryptOutput};
    pub use error::CoreError;
    pub use qr::{render_qr_png, MAX_QR_BYTES};
    pub use zeroize::ZeroizingDescriptor;
    ```

    Crear `crates/core/tests/round_trip.rs`:
    ```rust
    use zeroize::Zeroizing;

    use bed_core::{decrypt_payload, encrypt_descriptor};

    const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
    const FIXTURE_XPUB: &str = include_str!("fixtures/xpub.txt");

    #[test]
    fn round_trip_fixture() {
        let original = FIXTURE_DESC.trim().to_string();
        let mut cleartext = Zeroizing::new(original.clone());
        let out = encrypt_descriptor(&mut cleartext)
            .unwrap_or_else(|e| panic!("encrypt failed: {e}"));

        // Sanity: bed_bytes start with "BIPXXX" magic (verified at /tmp/bed-test/encrypted_backup/src/lib.rs)
        assert!(out.bed_bytes.starts_with(b"BIPXXX"), "bed must start with BIPXXX magic");

        // Sanity: armored has the right headers
        assert!(out.armored.starts_with("-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n"));
        assert!(out.armored.ends_with("-----END BITCOIN ENCRYPTED BACKUP-----\n"));

        // Sanity: qr_png is a real PNG
        assert!(out.qr_png.starts_with(b"\x89PNG"), "qr_png must be PNG");

        let recovered = decrypt_payload(&out.bed_bytes, FIXTURE_XPUB.trim())
            .unwrap_or_else(|e| panic!("decrypt failed: {e}"));
        assert_eq!(recovered.as_str(), original.as_str(), "round-trip mismatch");
    }

    #[test]
    fn decrypt_with_wrong_xpub_fails() {
        let original = FIXTURE_DESC.trim().to_string();
        let mut cleartext = Zeroizing::new(original);
        let out = encrypt_descriptor(&mut cleartext)
            .unwrap_or_else(|e| panic!("encrypt failed: {e}"));

        // Use an unrelated xpub — must not decrypt
        let wrong = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
        let result = decrypt_payload(&out.bed_bytes, wrong);
        assert!(result.is_err(), "wrong xpub must not decrypt");
    }

    #[test]
    fn cross_implementation_decrypt_with_reference_bed() {
        // Decrypt /tmp/bed-test/wallet.bed (produced by reference impl) with
        // /tmp/bed-test/xpub.txt to confirm cross-implementation interop (D-13).
        let bed_bytes: &[u8] = include_bytes!("fixtures/wallet.bed");
        let xpub = include_str!("fixtures/xpub.txt").trim();
        let recovered = decrypt_payload(bed_bytes, xpub)
            .unwrap_or_else(|e| panic!("cross-impl decrypt failed: {e}"));
        let expected = include_str!("fixtures/desc.txt").trim();
        assert_eq!(recovered.as_str(), expected);
    }

    #[test]
    fn qr_too_large_returns_error() {
        use bed_core::{render_qr_png, CoreError};
        let big = "x".repeat(3000);
        let err = render_qr_png(&big).unwrap_err();
        assert!(matches!(err, CoreError::QrTooLarge { size: 3000, max: 2900 }));
    }
    ```
  </action>
  <verify>
    <automated>cargo test -p bed-core 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p bed-core` exits 0 (todos los tests del crate core pasan: validate + armored + round_trip)
    - `crates/core/tests/round_trip.rs` test `round_trip_fixture` pasa
    - `crates/core/tests/round_trip.rs` test `cross_implementation_decrypt_with_reference_bed` pasa
    - `crates/core/tests/round_trip.rs` test `qr_too_large_returns_error` pasa
    - `crates/core/src/encrypt.rs` contiene literal `EncryptedBackup::new()`
    - `crates/core/src/encrypt.rs` contiene literal `require_multipath_0_1`
    - `crates/core/src/decrypt.rs` contiene literal `dpk_to_pk`
    - `crates/core/src/qr.rs` contiene literal `MAX_QR_BYTES: usize = 2900`
    - `grep -E '\.unwrap\(\)|\.expect\(' crates/core/src/encrypt.rs crates/core/src/decrypt.rs crates/core/src/qr.rs | grep -v unwrap_or` retorna 0 matches
    - `cargo clippy -p bed-core --all-targets -- -D warnings` exits 0
    - Fixtures: `diff crates/core/tests/fixtures/wallet.bed /tmp/bed-test/wallet.bed` exits 0
  </acceptance_criteria>
  <done>Core crate completo: validate + zeroize + armored + qr + encrypt + decrypt; round-trip determinista verificado contra fixture real; cross-impl decrypt funciona con `/tmp/bed-test/wallet.bed`.</done>
</task>

</tasks>

<verification>
- `cargo build -p bed-core` exits 0
- `cargo test -p bed-core` exits 0
- `cargo clippy -p bed-core --all-targets -- -D warnings` exits 0
- Round-trip determinista: encrypt → decrypt → mismo descriptor
- Cross-impl: `/tmp/bed-test/wallet.bed` decifra con la xpub fixture
- QR cap 2900 B enforce con error tipado
</verification>

<success_criteria>
- CORE-01: crate pinneada importable y usable
- CORE-02: round-trip test automatizado verde
- ENC-03: armored format exacto (BEGIN/END headers, line-wrap 64)
- ENC-04: QR PNG con cap 2900 B; error tipado si excede
- DEC-05: decoder armored tolera whitespace/CRLF/BOM/indentación
</success_criteria>

<output>
Tras completar, crear `.planning/phases/01-crypto-core-http-api/01-04-core-armored-qr-encrypt-SUMMARY.md` documentando:
- Tamaño en bytes del armored producido para la fixture (medir con `out.armored.len()`)
- Tamaño en bytes del QR PNG producido
- Confirmación de que `wallet.bed` cross-impl decifra al `desc.txt` original (sha256sum match)
- Lista de tests pasados con sus nombres
</output>
