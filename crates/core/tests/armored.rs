#![allow(clippy::panic)]

use bed_core::armored::{decode_armored, encode_armored, ArmoredError, ARMOR_BEGIN, ARMOR_END};

const PAYLOAD: &[u8] =
    b"the quick brown fox jumps over the lazy dog 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ";

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
        if line.starts_with("-----") {
            continue;
        }
        assert!(
            line.len() <= 64,
            "line too long: {} chars: {line}",
            line.len()
        );
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
    assert!(matches!(
        decode_armored(bad),
        Err(ArmoredError::WrongHeader)
    ));
}

#[test]
fn rejects_empty_payload() {
    let empty = format!("{ARMOR_BEGIN}\n{ARMOR_END}\n");
    assert!(matches!(
        decode_armored(&empty),
        Err(ArmoredError::EmptyPayload)
    ));
}
