#![allow(clippy::panic, clippy::unwrap_used)]

use zeroize::Zeroizing;

use bed_core::{decrypt_payload, encrypt_descriptor};

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
const FIXTURE_XPUB: &str = include_str!("fixtures/xpub.txt");

/// Normalize descriptor notation: miniscript library parses `h` (hardened marker)
/// and re-serializes as `'`. Both are valid BIP-380 representations; for comparison
/// purposes we normalize both sides to `'` notation and strip checksums.
fn normalize_desc(s: &str) -> String {
    // Strip checksum (#xxxx)
    let without_checksum = match s.rfind('#') {
        Some(i) => &s[..i],
        None => s,
    };
    // Replace any `h` that appears as a hardened marker: preceded by a digit, followed by / or ]
    // Use char-by-char approach for reliability
    let mut out = String::with_capacity(without_checksum.len());
    let chars: Vec<char> = without_checksum.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == 'h' && i > 0 {
            let prev = chars[i - 1];
            let next = if i + 1 < chars.len() { chars[i + 1] } else { '\0' };
            // hardened `h` marker: after digit, before `/` or `]`
            if prev.is_ascii_digit() && (next == '/' || next == ']') {
                out.push('\'');
                i += 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

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

    // miniscript normalizes `h` → `'` on parse+reserialize; compare normalized forms
    assert_eq!(
        normalize_desc(recovered.as_str()),
        normalize_desc(original.as_str()),
        "round-trip mismatch"
    );
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
    // miniscript normalizes `h` → `'` on parse+reserialize; compare normalized forms
    assert_eq!(normalize_desc(recovered.as_str()), normalize_desc(expected));
}

#[test]
fn qr_too_large_returns_error() {
    use bed_core::{render_qr_png, CoreError};
    let big = "x".repeat(3000);
    let err = render_qr_png(&big).unwrap_err();
    assert!(matches!(err, CoreError::QrTooLarge { size: 3000, max: 2900 }));
}
