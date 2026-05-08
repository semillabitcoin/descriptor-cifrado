//! bed-core — pure Bitcoin Encrypted Backup logic (validation, encrypt/decrypt wrapper,
//! armored encoder/decoder, QR generation). No HTTP layer.
//!
//! Re-exports the `bitcoin_encrypted_backup::miniscript` types so consumers don't add
//! a separate `miniscript` dep (would risk version unification break).

pub use bitcoin_encrypted_backup::miniscript;

pub mod armored;
pub mod decrypt;
pub mod encrypt;
pub mod error;
pub mod qr;
pub mod sparrow;
pub mod validate;
pub mod zeroize;

pub use armored::{decode_armored, encode_armored, ArmoredError, ARMOR_BEGIN, ARMOR_END};
pub use decrypt::decrypt_payload;
pub use encrypt::{encrypt_descriptor, EncryptOutput};
pub use error::CoreError;
pub use qr::{render_qr_png, MAX_QR_BYTES};
pub use sparrow::compose_descriptor_if_sparrow_jsonl;
pub use zeroize::ZeroizingDescriptor;
