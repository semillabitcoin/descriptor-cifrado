//! Tests de integración para passthrough JSON Liana en encrypt_descriptor / decrypt_payload.
//! Usan las funciones públicas de bed-core (no las de bitcoin_encrypted_backup directamente).
//! Oráculo: los 3 tests de round_trip_liana_blob.rs validan el patrón subyacente.

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use bed_core::{decrypt_payload, encrypt_descriptor, CoreError};
use zeroize::Zeroizing;

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
const FIXTURE_XPUB: &str = include_str!("fixtures/xpub.txt");

fn make_liana_json(descriptor: &str) -> String {
    format!(
        r#"{{"version":0,"network":"bitcoin","name":"test","accounts":[{{"descriptor":"{}","labels":null,"transactions":[],"psbts":[],"coins":{{}}}}],"proprietary":{{}}}}"#,
        descriptor
    )
}

#[test]
fn test_json_round_trip() {
    let json = make_liana_json(FIXTURE_DESC.trim());
    let mut cleartext = Zeroizing::new(json.clone());
    let out = encrypt_descriptor(&mut cleartext).expect("encrypt JSON Liana");

    assert!(out.bed_bytes.starts_with(b"BEB"), "magic BEB header");

    let recovered =
        decrypt_payload(&out.bed_bytes, FIXTURE_XPUB.trim()).expect("decrypt con xpub fixture");

    assert!(
        recovered.contains("\"accounts\""),
        "JSON shape preserved tras round-trip"
    );
    assert!(
        recovered.contains(FIXTURE_DESC.trim()),
        "descriptor original preservado dentro del JSON"
    );
    assert_eq!(recovered.as_str(), json, "round-trip byte-idéntico");
}

#[test]
fn test_json_malformed() {
    let mut cleartext = Zeroizing::new("{broken json".to_string());
    let err = encrypt_descriptor(&mut cleartext)
        .err()
        .expect("debe fallar con JSON malformado");
    assert!(
        matches!(err, CoreError::DescriptorParse),
        "esperado DescriptorParse, obtenido: {:?}",
        err
    );
}

#[test]
fn test_json_empty_accounts() {
    let json = r#"{"version":0,"network":"bitcoin","name":"test","accounts":[],"proprietary":{}}"#
        .to_string();
    let mut cleartext = Zeroizing::new(json);
    let err = encrypt_descriptor(&mut cleartext)
        .err()
        .expect("debe fallar con accounts vacío");
    assert!(
        matches!(err, CoreError::DescriptorParse),
        "esperado DescriptorParse, obtenido: {:?}",
        err
    );
}

#[test]
fn test_json_descriptor_no_multipath() {
    // Descriptor sin multipath (single wildcard /0/*) — debe rechazarse con MissingMultipathWildcard
    let no_multipath =
        "wpkh([68a9ec24/84h/0h/0h]xpub6Euvf9GFqnGhkLLeonsGfmpNSdz2oBwPMzn8tW8FJKxtfvrQFJrbQ1vp7iP8rbK9GXAG7RQK5D4dHvFRjqyawXnYPairzBM6Pnqqd2TTVMZ/0/*)";
    let json = make_liana_json(no_multipath);
    let mut cleartext = Zeroizing::new(json);
    let err = encrypt_descriptor(&mut cleartext)
        .err()
        .expect("debe fallar con descriptor sin multipath");
    assert!(
        matches!(err, CoreError::MissingMultipathWildcard),
        "esperado MissingMultipathWildcard, obtenido: {:?}",
        err
    );
}

#[test]
fn test_classic_descriptor_no_regression() {
    // Ruta clásica: descriptor plano (no JSON) — debe seguir funcionando
    let mut cleartext = Zeroizing::new(FIXTURE_DESC.trim().to_string());
    let out = encrypt_descriptor(&mut cleartext).expect("encrypt descriptor clásico");

    assert!(out.bed_bytes.starts_with(b"BEB"), "magic BEB header");

    let recovered =
        decrypt_payload(&out.bed_bytes, FIXTURE_XPUB.trim()).expect("decrypt descriptor clásico");

    // miniscript puede normalizar h→' o viceversa; comparar ambos lados normalizados
    let norm = |s: &str| s.replace('\'', "h");
    // Strip checksum si lo hay (miniscript añade #checksum tras serializar)
    let strip_checksum = |s: &str| s.split('#').next().unwrap_or(s).to_string();

    let expected = strip_checksum(&norm(FIXTURE_DESC.trim()));
    let got = strip_checksum(&norm(recovered.as_str()));

    assert_eq!(got, expected, "descriptor clásico round-trip sin regresión");
}
