//! Encrypt path: descriptor cleartext → .bed binary + armored + QR PNG.
//!
//! Receives `&mut Zeroizing<String>` to honor D-10 (no value moves of cleartext).
//!
//! ## Detección de formato (3 ramas)
//!
//! 1. Si `cleartext.trim_start().starts_with('{')`:
//!    a. Intentar parse Liana (`LianaBackup`): si ok → ruta Liana.
//!    b. Si Liana falla → intentar parse JSONL Sparrow línea por línea:
//!       - Ok + ≥1 xpub + ≥1 origin → ruta Sparrow.
//!       - Err → `CoreError::DescriptorParse`.
//! 2. Else → ruta descriptor clásica (sin cambios).

use std::str::FromStr;

use bitcoin_encrypted_backup::{
    descriptor::{descr_to_dpks, dpks_to_derivation_keys_paths},
    miniscript::{Descriptor, DescriptorPublicKey},
    Content, EncryptedBackup,
};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{
    armored::encode_armored,
    qr::render_qr_png,
    sparrow::{compose_descriptor_from_sparrow, parse_sparrow_jsonl},
    validate::require_multipath_wildcard,
    CoreError,
};

/// Output triple from a single encrypt call: binary `.bed`, armored PEM string,
/// and QR PNG bytes. Server serializes these into one JSON response (D-05).
/// `qr_png` is `None` when the armored payload exceeds `MAX_QR_BYTES` (2900 B).
pub struct EncryptOutput {
    pub bed_bytes: Vec<u8>,
    pub armored: String,
    pub qr_png: Option<Vec<u8>>,
}

#[derive(Deserialize)]
struct LianaAccount {
    descriptor: String,
}

#[derive(Deserialize)]
struct LianaBackup {
    accounts: Vec<LianaAccount>,
}

/// Encrypt a cleartext descriptor, Liana JSON backup, or Sparrow BIP329 JSONL.
///
/// Valida multipath wildcard antes de cifrar (para descriptores clásicos y
/// Liana). Para JSONL Sparrow, la validación se hace sobre el descriptor
/// compuesto reconstruido desde el `origin` template + xpubs.
///
/// Consultar doc del módulo para las 3 ramas de detección.
pub fn encrypt_descriptor(cleartext: &mut Zeroizing<String>) -> Result<EncryptOutput, CoreError> {
    let bed_bytes = if cleartext.trim_start().starts_with('{') {
        // --- Intentar ruta JSON Liana primero ---
        match serde_json::from_str::<LianaBackup>(cleartext.as_str()) {
            Ok(backup) => {
                // --- Ruta Liana ---
                let account = backup
                    .accounts
                    .into_iter()
                    .next()
                    .ok_or(CoreError::DescriptorParse)?;
                let desc: Descriptor<DescriptorPublicKey> =
                    Descriptor::from_str(&account.descriptor)
                        .map_err(|_| CoreError::DescriptorParse)?;
                require_multipath_wildcard(&desc)?;
                let dpks = descr_to_dpks(&desc)?;
                let (keys, paths) = dpks_to_derivation_keys_paths(&dpks);
                let payload: Vec<u8> = cleartext.as_bytes().to_vec();
                EncryptedBackup::new()
                    .set_keys(keys)
                    .set_derivation_paths(paths)
                    .set_content_type(Content::None)
                    .set_payload(&payload)?
                    .encrypt()?
            }
            Err(_) => {
                // --- Fall-through: intentar JSONL Sparrow BIP329 ---
                let sparrow_data = parse_sparrow_jsonl(cleartext.as_str())
                    .map_err(|_| CoreError::DescriptorParse)?;
                let composed = compose_descriptor_from_sparrow(&sparrow_data)
                    .map_err(|_| CoreError::DescriptorParse)?;
                let desc: Descriptor<DescriptorPublicKey> =
                    Descriptor::from_str(&composed).map_err(|_| CoreError::DescriptorParse)?;
                require_multipath_wildcard(&desc)?;
                let dpks = descr_to_dpks(&desc)?;
                let (keys, paths) = dpks_to_derivation_keys_paths(&dpks);
                let payload: Vec<u8> = cleartext.as_bytes().to_vec();
                EncryptedBackup::new()
                    .set_keys(keys)
                    .set_derivation_paths(paths)
                    .set_content_type(Content::None)
                    .set_payload(&payload)?
                    .encrypt()?
            }
        }
    } else {
        // --- Ruta descriptor clásica (sin cambios) ---
        let desc: Descriptor<DescriptorPublicKey> =
            Descriptor::from_str(cleartext.as_str()).map_err(|_| CoreError::DescriptorParse)?;
        require_multipath_wildcard(&desc)?;
        EncryptedBackup::new().set_payload(&desc)?.encrypt()?
    };

    let armored = encode_armored(&bed_bytes);
    let qr_png = match render_qr_png(&armored) {
        Ok(png) => Some(png),
        Err(CoreError::QrTooLarge { .. }) => None,
        Err(e) => return Err(e),
    };
    Ok(EncryptOutput {
        bed_bytes,
        armored,
        qr_png,
    })
}
