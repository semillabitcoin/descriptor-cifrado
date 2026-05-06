//! Decrypt path: .bed bytes + xpub → cleartext descriptor wrapped in Zeroizing.
//!
//! The crate's `set_encrypted_payload` auto-detects binary "BIPXXX" magic vs
//! base64 — but does NOT strip PEM-style headers. The server is responsible
//! for stripping headers via `armored::decode_armored` BEFORE calling this.

use std::str::FromStr;

use bitcoin_encrypted_backup::{
    descriptor::dpk_to_pk, miniscript::DescriptorPublicKey, Decrypted, EncryptedBackup,
};
use zeroize::Zeroizing;

use crate::CoreError;

/// Decrypt a .bed payload (binary or raw base64) using a single xpub. The xpub
/// is parsed as a DescriptorPublicKey and converted to secp256k1::PublicKey via
/// the crate's `dpk_to_pk` helper.
pub fn decrypt_payload(bed_bytes: &[u8], xpub_str: &str) -> Result<Zeroizing<String>, CoreError> {
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
