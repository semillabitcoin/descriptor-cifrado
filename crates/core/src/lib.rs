//! bed-core — pure Bitcoin Encrypted Backup logic (validation, encrypt/decrypt wrapper,
//! armored encoder/decoder, QR generation). No HTTP layer.
//!
//! Re-exports the `bitcoin_encrypted_backup::miniscript` types so consumers don't add
//! a separate `miniscript` dep (would risk version unification break).

pub use bitcoin_encrypted_backup::miniscript;

pub mod error;
pub mod validate;
pub mod zeroize;

pub use error::CoreError;
pub use zeroize::ZeroizingDescriptor;
