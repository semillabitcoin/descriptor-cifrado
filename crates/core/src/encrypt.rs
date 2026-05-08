//! Encrypt path: descriptor cleartext → .bed binary + armored + QR PNG.
//!
//! Receives `&mut Zeroizing<String>` to honor D-10 (no value moves of cleartext).

use std::str::FromStr;

use bitcoin_encrypted_backup::{
    descriptor::{descr_to_dpks, dpks_to_derivation_keys_paths},
    miniscript::{Descriptor, DescriptorPublicKey},
    Content, EncryptedBackup,
};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{
    armored::encode_armored, qr::render_qr_png, validate::require_multipath_wildcard, CoreError,
};

/// Output triple from a single encrypt call: binary `.bed`, armored PEM string,
/// and QR PNG bytes. Server serializes these into one JSON response (D-05).
pub struct EncryptOutput {
    pub bed_bytes: Vec<u8>,
    pub armored: String,
    pub qr_png: Vec<u8>,
}

#[derive(Deserialize)]
struct LianaAccount {
    descriptor: String,
}

#[derive(Deserialize)]
struct LianaBackup {
    accounts: Vec<LianaAccount>,
}

/// Encrypt a cleartext descriptor or Liana JSON backup. Valida multipath wildcard
/// primero, then calls the crate to produce binary, wraps to armored, generates QR.
///
/// Si el cleartext empieza con `{`, se interpreta como JSON de backup Liana:
/// extrae `accounts[0].descriptor` para derivar claves/paths y cifra el JSON
/// completo como blob (Decrypted::Raw al descifrar). En caso contrario, usa
/// la ruta clásica de descriptor.
pub fn encrypt_descriptor(cleartext: &mut Zeroizing<String>) -> Result<EncryptOutput, CoreError> {
    let bed_bytes = if cleartext.trim_start().starts_with('{') {
        // --- Ruta JSON Liana ---
        let backup: LianaBackup = serde_json::from_str(cleartext.as_str())
            .map_err(|_| CoreError::DescriptorParse)?;
        let account = backup
            .accounts
            .into_iter()
            .next()
            .ok_or(CoreError::DescriptorParse)?;
        let desc: Descriptor<DescriptorPublicKey> =
            Descriptor::from_str(&account.descriptor).map_err(|_| CoreError::DescriptorParse)?;
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
    } else {
        // --- Ruta descriptor clásica (sin cambios) ---
        let desc: Descriptor<DescriptorPublicKey> =
            Descriptor::from_str(cleartext.as_str()).map_err(|_| CoreError::DescriptorParse)?;
        require_multipath_wildcard(&desc)?;
        EncryptedBackup::new().set_payload(&desc)?.encrypt()?
    };

    let armored = encode_armored(&bed_bytes);
    let qr_png = render_qr_png(&armored)?;
    Ok(EncryptOutput {
        bed_bytes,
        armored,
        qr_png,
    })
}
