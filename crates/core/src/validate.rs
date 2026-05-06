//! BIP <0;1>/* multipath wildcard validation (CORE-03, D-08).
//!
//! The bitcoin-encrypted-backup crate does NOT enforce this — it accepts
//! any descriptor with at least one non-NUMS key. This module is the
//! application-layer guard that rejects descriptors which would expose
//! the xpub on-chain at first spend.

use bitcoin_encrypted_backup::miniscript::{
    descriptor::Wildcard, Descriptor, DescriptorPublicKey, ForEachKey,
};

use crate::CoreError;

/// Require every key to be a `MultiXPub` with `derivation_paths == [0, 1]`
/// and `Wildcard::Unhardened`. Rejects bare xpubs, single wildcards, and
/// non-`<0;1>` multipath indices.
pub fn require_multipath_0_1(desc: &Descriptor<DescriptorPublicKey>) -> Result<(), CoreError> {
    let mut all_ok = true;

    desc.for_each_key(|k| {
        let ok = match k {
            DescriptorPublicKey::MultiXPub(mx) => {
                if mx.wildcard != Wildcard::Unhardened {
                    false
                } else {
                    let paths = mx.derivation_paths.paths();
                    paths.len() == 2 && paths[0].to_string() == "0" && paths[1].to_string() == "1"
                }
            }
            _ => false, // Single, XPub (single wildcard or none) → reject
        };
        if !ok {
            all_ok = false;
        }
        true // continue iteration
    });

    if all_ok {
        Ok(())
    } else {
        Err(CoreError::MissingMultipathWildcard)
    }
}
