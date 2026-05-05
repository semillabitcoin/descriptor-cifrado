//! Errors emitted by bed-core. The server crate maps these to HTTP responses
//! via its own AppError type (D-16). Internal-to-core only.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain.")]
    MissingMultipathWildcard,

    #[error("No se pudo parsear el descriptor.")]
    DescriptorParse,

    #[error("La xpub proporcionada no descifra este .bed (no es un cosigner válido).")]
    XpubMismatch,

    #[error("El descriptor cifrado excede capacidad QR ({size} > {max} bytes). Usa el archivo .bed o el armored.")]
    QrTooLarge { size: usize, max: usize },

    #[error("error de codificación armored: {0}")]
    Armored(String),

    #[error("error interno de cifrado")]
    Crypto,
}

// Map crate's bitcoin_encrypted_backup::Error → CoreError
impl From<bitcoin_encrypted_backup::Error> for CoreError {
    fn from(e: bitcoin_encrypted_backup::Error) -> Self {
        use bitcoin_encrypted_backup::Error as E;
        match e {
            E::WrongKey | E::NoKey | E::DescriptorHasNoKeys => CoreError::XpubMismatch,
            E::Descriptor | E::Utf8 => CoreError::DescriptorParse,
            _ => CoreError::Crypto,
        }
    }
}
