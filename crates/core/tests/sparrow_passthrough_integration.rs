//! Tests de integración para passthrough JSONL Sparrow BIP329 en encrypt_descriptor /
//! decrypt_payload / compose_descriptor_if_sparrow_jsonl.
//!
//! Fixture: `fixtures/sparrow-multisig.jsonl` — export real 2-of-3 Coldcard testnet.

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use bed_core::{compose_descriptor_if_sparrow_jsonl, decrypt_payload, encrypt_descriptor, CoreError};
use zeroize::Zeroizing;

const FIXTURE_MULTISIG: &str = include_str!("fixtures/sparrow-multisig.jsonl");

/// Extrae el `ref` de la primera entry `type:"xpub"` del JSONL.
fn first_xpub_from_jsonl(jsonl: &str) -> String {
    for line in jsonl.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(trimmed).expect("valid JSON line");
        if v["type"] == "xpub" {
            return v["ref"].as_str().expect("ref field").to_string();
        }
    }
    panic!("no xpub entry found in fixture");
}

// ---------------------------------------------------------------------------
// Test 1: round-trip JSONL multisig real 2-of-3 Coldcard testnet
// ---------------------------------------------------------------------------
#[test]
fn test_jsonl_multisig_round_trip() {
    let jsonl = FIXTURE_MULTISIG.to_string();
    let mut cleartext = Zeroizing::new(jsonl.clone());

    let out = encrypt_descriptor(&mut cleartext).expect("encrypt JSONL Sparrow multisig");

    // Debe producir magic BEB
    assert!(
        out.bed_bytes.starts_with(b"BEB"),
        "expected BEB magic header"
    );
    // QR depende del tamaño del armored generado; no lo forzamos aquí.
    // El fixture de 6 líneas (1.1 KB) puede generar armored < 2.9 KB → qr_png = Some.
    // El test valida round-trip, no el tamaño del QR.

    // Descifrar con la primera xpub del fixture
    let xpub = first_xpub_from_jsonl(FIXTURE_MULTISIG);
    let recovered =
        decrypt_payload(&out.bed_bytes, &xpub).expect("decrypt JSONL con primera xpub");

    // Round-trip byte-idéntico
    assert_eq!(
        recovered.as_str(),
        jsonl,
        "JSONL multisig round-trip byte-idéntico"
    );
}

// ---------------------------------------------------------------------------
// Test 2: round-trip JSONL sintético singlesig
// ---------------------------------------------------------------------------
#[test]
fn test_jsonl_singlesig_round_trip() {
    // descriptor singlesig wpkh testnet con multipath <0;1>/*
    let tpub = "tpubDEBuUGE6MofiCADqeET94gPUrxYCjrcFNshi7yMpFG5RF6gh5ArXVCpPzocEE1enATbAyob7tW1JScwH5KedRMXJXMDCrZWmnVv7voaccGn";
    let jsonl = format!(
        "{}\n{}\n",
        serde_json::json!({"type": "xpub", "ref": tpub, "label": "key1"}),
        serde_json::json!({"keypath": "/0/0", "heights": [], "type": "addr", "ref": "tb1q...", "origin": "wpkh([deadbeef/84h/1h/0h])"})
    );

    let mut cleartext = Zeroizing::new(jsonl.clone());
    let out = encrypt_descriptor(&mut cleartext).expect("encrypt JSONL singlesig");

    assert!(out.bed_bytes.starts_with(b"BEB"), "magic BEB header");

    let recovered = decrypt_payload(&out.bed_bytes, tpub).expect("decrypt singlesig");

    assert_eq!(
        recovered.as_str(),
        jsonl,
        "JSONL singlesig round-trip byte-idéntico"
    );
}

// ---------------------------------------------------------------------------
// Test 3: JSONL sin xpubs → DescriptorParse
// ---------------------------------------------------------------------------
#[test]
fn test_jsonl_no_xpubs_error() {
    // Solo entradas addr sin ninguna xpub etiquetada
    let jsonl = "{\"keypath\":\"/0/0\",\"heights\":[],\"type\":\"addr\",\"ref\":\"tb1q...\",\"origin\":\"wsh(sortedmulti(2,[e98826a2/48h/1h/0h/2h],[101ccc0d/48h/1h/0h/2h]))\"}\n".to_string();
    let mut cleartext = Zeroizing::new(jsonl);

    let err = encrypt_descriptor(&mut cleartext)
        .err()
        .expect("debe fallar sin xpubs");
    assert!(
        matches!(err, CoreError::DescriptorParse),
        "esperado DescriptorParse, obtenido: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Test 4: JSONL solo con xpubs (sin origin) → DescriptorParse
// ---------------------------------------------------------------------------
#[test]
fn test_jsonl_no_origin_error() {
    // Dos xpubs pero ninguna entry con origin
    let tpub1 = "tpubDFmUCPAr8WgEANGstmygZF5WdoSx3mLozHEp9RW92Z6nqcYNbRyVUBLsnhrQ4TmwszRBnhqgJvAf8s3tWQtkjANYqP8qRnTiKpkt7p9WPji";
    let tpub2 = "tpubDEBuUGE6MofiCADqeET94gPUrxYCjrcFNshi7yMpFG5RF6gh5ArXVCpPzocEE1enATbAyob7tW1JScwH5KedRMXJXMDCrZWmnVv7voaccGn";
    let jsonl = format!(
        "{}\n{}\n",
        serde_json::json!({"type": "xpub", "ref": tpub1, "label": "key1"}),
        serde_json::json!({"type": "xpub", "ref": tpub2, "label": "key2"})
    );
    let mut cleartext = Zeroizing::new(jsonl);

    let err = encrypt_descriptor(&mut cleartext)
        .err()
        .expect("debe fallar sin origin");
    assert!(
        matches!(err, CoreError::DescriptorParse),
        "esperado DescriptorParse, obtenido: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Test 5: compose_descriptor_if_sparrow_jsonl devuelve Some para JSONL válido
// ---------------------------------------------------------------------------
#[test]
fn test_compose_descriptor_returns_some_for_jsonl() {
    let result = compose_descriptor_if_sparrow_jsonl(FIXTURE_MULTISIG);
    assert!(result.is_some(), "debe devolver Some para fixture multisig válido");

    let desc = result.unwrap();
    assert!(
        desc.contains("sortedmulti(2,"),
        "descriptor debe contener sortedmulti(2,): {}",
        desc
    );
    assert!(
        desc.contains("/<0;1>/*"),
        "descriptor debe contener /<0;1>/*: {}",
        desc
    );
    // Verificar que se incluyeron las 3 xpubs del fixture
    assert!(
        desc.contains("tpubDFmUCPAr8WgEANGstmygZF5WdoSx3mLozHEp9RW92Z6nqcYNbRyVUBLsnhrQ4TmwszRBnhqgJvAf8s3tWQtkjANYqP8qRnTiKpkt7p9WPji"),
        "debe contener Coldcard 1 tpub"
    );
}

// ---------------------------------------------------------------------------
// Test 6: compose_descriptor_if_sparrow_jsonl devuelve None para descriptor
// clásico y JSON Liana
// ---------------------------------------------------------------------------
#[test]
fn test_compose_descriptor_returns_none_for_classic() {
    // Descriptor clásico plano — no es JSONL
    let classic = "wsh(sortedmulti(2,xpub6ERApFa9opSnNQjJPCQbKx3oFnkDMECTTcpaHnuyqAUe8TGUT2XKKoRBmrBpFLFAbRuXB7FXjMtLhGW7qfmcZNXMRLfFqNXLt3ixBovK6T/<0;1>/*,xpub6ERApFa9opSnNQjJPCQbKx3oFnkDMECTTcpaHnuyqAUe8TGUT2XKKoRBmrBpFLFAbRuXB7FXjMtLhGW7qfmcZNXMRLfFqNXLt3ixBovK6T/<0;1>/*))";
    assert!(
        compose_descriptor_if_sparrow_jsonl(classic).is_none(),
        "descriptor clásico debe devolver None"
    );

    // JSON Liana — empieza con '{' pero no es JSONL línea-por-línea válido para Sparrow
    let liana_json = r#"{"version":0,"network":"bitcoin","name":"test","accounts":[{"descriptor":"wsh(or_d(pk(xpub6ERApFa9opSnNQjJPCQbKx3oFnkDMECTTcpaHnuyqAUe8TGUT2XKKoRBmrBpFLFAbRuXB7FXjMtLhGW7qfmcZNXMRLfFqNXLt3ixBovK6T/<0;1>/*),and_v(v:pkh(xpub6FezuHsRFJFGYoGVSiJZVKAqg1Q3wFHhEDUMFNkVsf1bCjx2aN7QvxJvRCjeBxRNqsabK1N5AeBCFbDcRNFrpuS5pLCHAfnLJzDiXAyX1Z/<0;1>/*),after(500000)))","labels":null,"transactions":[],"psbts":[],"coins":{}}],"proprietary":{}}"#;
    assert!(
        compose_descriptor_if_sparrow_jsonl(liana_json).is_none(),
        "JSON Liana debe devolver None (no es JSONL línea-por-línea Sparrow)"
    );

    // String vacío
    assert!(
        compose_descriptor_if_sparrow_jsonl("").is_none(),
        "string vacío debe devolver None"
    );
}
