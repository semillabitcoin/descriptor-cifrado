//! Encrypt path: descriptor cleartext → .bed binary + armored + QR PNG.
//!
//! Receives `&mut Zeroizing<String>` to honor D-10 (no value moves of cleartext).

use std::str::FromStr;

use bitcoin_encrypted_backup::{
    miniscript::{Descriptor, DescriptorPublicKey},
    EncryptedBackup,
};
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

/// Encrypt a cleartext descriptor. Valida multipath wildcard primero, then calls the
/// crate to produce binary, wraps to armored, generates QR.
pub fn encrypt_descriptor(cleartext: &mut Zeroizing<String>) -> Result<EncryptOutput, CoreError> {
    let desc: Descriptor<DescriptorPublicKey> =
        Descriptor::from_str(cleartext.as_str()).map_err(|_| CoreError::DescriptorParse)?;

    require_multipath_wildcard(&desc)?;

    let bed_bytes: Vec<u8> = EncryptedBackup::new().set_payload(&desc)?.encrypt()?;

    let armored = encode_armored(&bed_bytes);
    let qr_png = render_qr_png(&armored)?;

    Ok(EncryptOutput {
        bed_bytes,
        armored,
        qr_png,
    })
}
