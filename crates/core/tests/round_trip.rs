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
            let next = if i + 1 < chars.len() {
                chars[i + 1]
            } else {
                '\0'
            };
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
    let out = encrypt_descriptor(&mut cleartext).unwrap_or_else(|e| panic!("encrypt failed: {e}"));

    // Sanity: bed_bytes empiezan con magic "BEB" (bitcoin-encrypted-backup v0.0.2).
    assert!(
        out.bed_bytes.starts_with(b"BEB"),
        "bed debe empezar con magic BEB"
    );

    // Sanity: armored has the right headers
    assert!(out
        .armored
        .starts_with("-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n"));
    assert!(out
        .armored
        .ends_with("-----END BITCOIN ENCRYPTED BACKUP-----\n"));

    // Sanity: qr_png is a real PNG (fixture descriptor is small — must be Some)
    assert!(
        out.qr_png
            .as_ref()
            .is_some_and(|qr| qr.starts_with(b"\x89PNG")),
        "qr_png must be Some and start with PNG magic"
    );

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
    let out = encrypt_descriptor(&mut cleartext).unwrap_or_else(|e| panic!("encrypt failed: {e}"));

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
    assert!(matches!(
        err,
        CoreError::QrTooLarge {
            size: 3000,
            max: 2900
        }
    ));
}

#[test]
fn encrypt_large_payload_omits_qr() {
    // JSON Liana sintético (>2900 bytes) — el descriptor en sí cabe, pero el JSON
    // completo no. Verifica que qr_png == None y que bed_bytes + armored siguen válidos.
    let descriptor = FIXTURE_DESC.trim();
    let big_payload = format!(
        r#"{{"version":0,"network":"bitcoin","name":"big","accounts":[{{"descriptor":"{}","labels":null,"transactions":[],"psbts":[],"coins":{{}}}}],"padding":"{}","proprietary":{{}}}}"#,
        descriptor,
        "x".repeat(3500)
    );
    let mut cleartext = Zeroizing::new(big_payload);
    let out = encrypt_descriptor(&mut cleartext).unwrap_or_else(|e| panic!("encrypt failed: {e}"));

    assert!(
        out.bed_bytes.starts_with(b"BEB"),
        "bed debe empezar con BEB"
    );
    assert!(out
        .armored
        .starts_with("-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n"));
    assert!(
        out.qr_png.is_none(),
        "qr_png debe ser None cuando el armored excede MAX_QR_BYTES"
    );
}
