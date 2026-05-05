use std::str::FromStr;

use bed_core::miniscript::{Descriptor, DescriptorPublicKey};
use bed_core::validate::require_multipath_0_1;
use bed_core::CoreError;

#[allow(clippy::panic)]
fn parse(s: &str) -> Descriptor<DescriptorPublicKey> {
    Descriptor::<DescriptorPublicKey>::from_str(s)
        .unwrap_or_else(|e| panic!("fixture parse failed: {s} -> {e}"))
}

const VALID_FIXTURE: &str = include_str!("fixtures/desc.txt");

// Real xpubs extracted from the fixture file (public, non-secret test data)
const XPUB_A: &str = "xpub6PLACEHOLDER2xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const XPUB_B: &str = "xpub6DhWabCb9j5vGn3VU2YJ9kNgk258AoBcQWHQPstRY2bYZJjJUDvxTZPrg58An3CCEWhhVuxk9A6nobikMQjtK8Xk5JakJc1HSu13Z3dKfpo";

#[test]
fn rejects_bare_xpub() {
    let d = parse(&format!("wsh(pk({XPUB_A}))"));
    assert!(matches!(
        require_multipath_0_1(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn rejects_single_wildcard() {
    let d = parse(&format!("wsh(pk({XPUB_A}/0/*))"));
    assert!(matches!(
        require_multipath_0_1(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn rejects_wrong_multipath_indices() {
    let d = parse(&format!("wsh(pk({XPUB_A}/<2;3>/*))"));
    assert!(matches!(
        require_multipath_0_1(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn rejects_mixed_one_good_one_bad() {
    let d = parse(&format!(
        "wsh(sortedmulti(2,{XPUB_A}/<0;1>/*,{XPUB_B}/<2;3>/*))"
    ));
    assert!(matches!(
        require_multipath_0_1(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn accepts_valid_fixture() {
    let d = parse(VALID_FIXTURE.trim());
    assert!(require_multipath_0_1(&d).is_ok(), "valid fixture should pass");
}

#[test]
fn accepts_synthetic_2_of_3_multipath() {
    // Triple-key sortedmulti, all <0;1>/*
    let d = parse(&format!(
        "wsh(sortedmulti(2,{XPUB_A}/<0;1>/*,{XPUB_B}/<0;1>/*,{XPUB_A}/<0;1>/*))"
    ));
    assert!(require_multipath_0_1(&d).is_ok());
}
