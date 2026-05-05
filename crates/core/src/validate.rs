//! BIP <0;1>/* multipath wildcard validation (CORE-03, D-08).
//!
//! Stub — to be replaced by full implementation in Task 2.

use bitcoin_encrypted_backup::miniscript::{Descriptor, DescriptorPublicKey};

use crate::CoreError;

/// Stub — real implementation in Task 2.
pub fn require_multipath_0_1(
    _desc: &Descriptor<DescriptorPublicKey>,
) -> Result<(), CoreError> {
    unimplemented!("validate::require_multipath_0_1 — Task 2 replaces this stub")
}
