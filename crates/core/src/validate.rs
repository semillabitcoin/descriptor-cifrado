//! Validación de wildcard multipath BIP (CORE-03, D-08).
//!
//! La crate bitcoin-encrypted-backup NO enforce esto — acepta cualquier descriptor
//! con al menos una clave no-NUMS. Este módulo es el guard de capa de aplicación
//! que rechaza descriptors que expondrían la xpub on-chain al primer gasto.

use bitcoin_encrypted_backup::miniscript::{
    descriptor::Wildcard, Descriptor, DescriptorPublicKey, ForEachKey,
};

use crate::CoreError;

/// Require que cada clave sea un `MultiXPub` con exactamente 2 paths distintos
/// y `Wildcard::Unhardened`. Acepta cualquier par `<a;b>/*` con `a ≠ b`
/// (por ejemplo `<0;1>/*`, `<2;3>/*`). Rechaza bare xpubs, single wildcards
/// y pares degenerados como `<5;5>/*`.
pub fn require_multipath_wildcard(desc: &Descriptor<DescriptorPublicKey>) -> Result<(), CoreError> {
    let mut all_ok = true;

    desc.for_each_key(|k| {
        let ok = match k {
            DescriptorPublicKey::MultiXPub(mx) => {
                if mx.wildcard != Wildcard::Unhardened {
                    false
                } else {
                    let paths = mx.derivation_paths.paths();
                    paths.len() == 2 && paths[0] != paths[1]
                }
            }
            _ => false, // Single, XPub (single wildcard o ninguno) → rechazar
        };
        if !ok {
            all_ok = false;
        }
        true // continuar iteración
    });

    if all_ok {
        Ok(())
    } else {
        Err(CoreError::MissingMultipathWildcard)
    }
}
