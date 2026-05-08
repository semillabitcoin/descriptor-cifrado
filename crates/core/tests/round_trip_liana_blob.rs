//! Validation: round-trip de payload arbitrario (JSON Liana) usando v0.0.2 con
//! extracción manual de claves del descriptor anidado en el JSON.
//!
//! Hipótesis:
//!   1. `EncryptedBackup` v0.0.2 acepta `Vec<u8>` como payload (impl ToPayload).
//!   2. `Vec<u8>::content_type() == Unknown` y `Unknown.is_known() == false`,
//!      luego `set_content_type(None)` previo sobrevive a `set_payload`.
//!   3. `set_keys` + `set_derivation_paths` antes de `set_payload` colocan las
//!      claves derivadas del descriptor anidado (que `Vec<u8>` no aporta).
//!   4. Cualquier xpub del descriptor descifra (1-of-N) y obtiene
//!      `Decrypted::Raw(bytes)` byte-idéntico al payload original.
//!
//! Si los 4 pasos pasan → la rama JSON-passthrough en encrypt/decrypt es viable
//! con cambios mínimos en `bed-core`. Si falla → replanteamos antes de tocar
//! el path de producción.

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::str::FromStr;

use bed_core::miniscript::{Descriptor, DescriptorPublicKey};
use bitcoin_encrypted_backup::{
    descriptor::{descr_to_dpks, dpk_to_pk, dpks_to_derivation_keys_paths},
    Content, Decrypted, EncryptedBackup,
};

const FIXTURE_DESC: &str = include_str!("fixtures/desc.txt");
const FIXTURE_XPUB: &str = include_str!("fixtures/xpub.txt");

fn synthetic_liana_backup(descriptor: &str) -> String {
    // Estructura mínima compatible con liana-gui/src/backup.rs:Backup
    // (campos: name, alias, accounts[{descriptor, labels, ...}], network, date,
    // proprietary, version). Aquí enviamos el subset suficiente para validar
    // el round-trip de bytes — la app real recibiría el JSON completo de Liana.
    format!(
        r#"{{"version":0,"network":"bitcoin","name":"test","accounts":[{{"descriptor":"{}","labels":null,"transactions":[],"psbts":[],"coins":{{}}}}],"proprietary":{{}}}}"#,
        descriptor
    )
}

#[test]
fn round_trip_liana_backup_json_with_fixture_xpub() {
    let descriptor_str = FIXTURE_DESC.trim();
    let backup_json = synthetic_liana_backup(descriptor_str);

    // Paso 1: extraer claves del descriptor anidado en el JSON.
    let descriptor: Descriptor<DescriptorPublicKey> =
        Descriptor::from_str(descriptor_str).expect("fixture descriptor parses");
    let dpks = descr_to_dpks(&descriptor).expect("dpks extracted");
    let (keys, paths) = dpks_to_derivation_keys_paths(&dpks);

    assert_eq!(
        keys.len(),
        3,
        "3 cosigner keys expected from sortedmulti(2,k,k,k)"
    );
    assert!(!paths.is_empty(), "at least one derivation path");

    // Paso 2: cifrar JSON como blob arbitrario con keys/paths del descriptor.
    let payload_bytes: Vec<u8> = backup_json.as_bytes().to_vec();
    let bed_bytes = EncryptedBackup::new()
        .set_keys(keys.clone())
        .set_derivation_paths(paths.clone())
        .set_content_type(Content::None)
        .set_payload(&payload_bytes)
        .expect("set_payload accepts Vec<u8>")
        .encrypt()
        .expect("encrypt succeeds");

    assert!(bed_bytes.starts_with(b"BEB"), "magic BEB header (v0.0.2)");

    // Paso 3: descifrar con la xpub del fixture (1 de los 3 cosigners).
    let xpub = DescriptorPublicKey::from_str(FIXTURE_XPUB.trim()).expect("xpub parses");
    let pk = dpk_to_pk(&xpub);
    let decrypted = EncryptedBackup::new()
        .set_encrypted_payload(&bed_bytes)
        .expect("decrypt header parses")
        .set_keys(vec![pk])
        .decrypt()
        .expect("decrypt succeeds with 1 of 3 cosigners");

    // Paso 4: verificar Raw + igualdad byte-a-byte.
    match decrypted {
        Decrypted::Raw(bytes) => {
            assert_eq!(bytes, payload_bytes, "byte-identical round trip");
            let recovered = std::str::from_utf8(&bytes).expect("valid utf8 JSON");
            assert!(recovered.contains("\"accounts\""), "JSON shape preserved");
            assert!(
                recovered.contains(descriptor_str),
                "descriptor preserved inside JSON"
            );
        }
        other => panic!("expected Decrypted::Raw, got {:?}", other),
    }
}

#[test]
fn round_trip_decrypts_with_each_cosigner_independently() {
    // Propiedad 1-of-N: cualquiera de las N xpubs del descriptor debe descifrar.
    let descriptor_str = FIXTURE_DESC.trim();
    let backup_json = synthetic_liana_backup(descriptor_str);

    let descriptor: Descriptor<DescriptorPublicKey> =
        Descriptor::from_str(descriptor_str).expect("descriptor parses");
    let dpks = descr_to_dpks(&descriptor).expect("dpks");
    let (keys, paths) = dpks_to_derivation_keys_paths(&dpks);

    let payload = backup_json.as_bytes().to_vec();
    let bed = EncryptedBackup::new()
        .set_keys(keys)
        .set_derivation_paths(paths)
        .set_content_type(Content::None)
        .set_payload(&payload)
        .expect("set_payload")
        .encrypt()
        .expect("encrypt");

    // Cada cosigner debe descifrar independientemente.
    for (i, dpk) in dpks.iter().enumerate() {
        let pk = dpk_to_pk(dpk);
        let decrypted = EncryptedBackup::new()
            .set_encrypted_payload(&bed)
            .expect("header parse")
            .set_keys(vec![pk])
            .decrypt()
            .unwrap_or_else(|e| panic!("cosigner #{i} decrypt failed: {:?}", e));

        match decrypted {
            Decrypted::Raw(bytes) => assert_eq!(bytes, payload, "cosigner #{i}: payload mismatch"),
            other => panic!("cosigner #{i}: expected Raw, got {:?}", other),
        }
    }
}

#[test]
fn descriptor_path_unaffected_by_blob_capability() {
    // Smoke test: el camino tradicional (descriptor como ToPayload) sigue
    // produciendo Decrypted::Descriptor — no hay regresión por añadir
    // capacidad de blob arbitrario.
    let descriptor_str = FIXTURE_DESC.trim();
    let descriptor: Descriptor<DescriptorPublicKey> =
        Descriptor::from_str(descriptor_str).expect("parse");

    let bed = EncryptedBackup::new()
        .set_payload(&descriptor)
        .expect("set_payload(&Descriptor)")
        .encrypt()
        .expect("encrypt");

    let xpub = DescriptorPublicKey::from_str(FIXTURE_XPUB.trim()).expect("xpub");
    let pk = dpk_to_pk(&xpub);
    let decrypted = EncryptedBackup::new()
        .set_encrypted_payload(&bed)
        .expect("header")
        .set_keys(vec![pk])
        .decrypt()
        .expect("decrypt");

    assert!(
        matches!(decrypted, Decrypted::Descriptor(_)),
        "descriptor path must yield Decrypted::Descriptor, not Raw"
    );
}
