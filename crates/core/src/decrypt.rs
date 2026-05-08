//! Decrypt path: .bed bytes + xpub → cleartext descriptor wrapped in Zeroizing.
//!
//! En `bitcoin-encrypted-backup` v0.0.2 (tag publicado, compatible con Liana),
//! `set_encrypted_payload` espera el blob binario con magic `BEB`. NO hace
//! autodetección de base64 ni strip de PEM. El server decodifica armored vía
//! `armored::decode_armored` ANTES de llamar a esta función; este wrapper solo
//! recibe binario crudo.

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
        Decrypted::Raw(bytes) => String::from_utf8(bytes)
            .map(Zeroizing::new)
            .map_err(|_| CoreError::Crypto),
        _ => Err(CoreError::Crypto),
    }
}
