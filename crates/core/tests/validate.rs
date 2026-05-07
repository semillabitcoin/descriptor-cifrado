#![allow(clippy::panic)]

use std::str::FromStr;

use bed_core::miniscript::{Descriptor, DescriptorPublicKey};
use bed_core::validate::require_multipath_wildcard;
use bed_core::CoreError;

fn parse(s: &str) -> Descriptor<DescriptorPublicKey> {
    Descriptor::<DescriptorPublicKey>::from_str(s)
        .unwrap_or_else(|e| panic!("fixture parse failed: {s} -> {e}"))
}

const VALID_FIXTURE: &str = include_str!("fixtures/desc.txt");

// xpubs reales de los fixtures (datos de test no secretos)
const XPUB_B: &str = "xpub6DhWabCb9j5vGn3VU2YJ9kNgk258AoBcQWHQPstRY2bYZJjJUDvxTZPrg58An3CCEWhhVuxk9A6nobikMQjtK8Xk5JakJc1HSu13Z3dKfpo";
const XPUB_C: &str = "xpub6DtCKqADjbNNP3Yg8TC88yqqpZfepov4feSKD8HrJGsqoDZe83vWNFi13qiD7uaXF65CBNERpvBMhT1oxzJJjjNq2A8hCNuQ4oXww7LHmJA";
const XPUB_D: &str = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";

#[test]
fn rejects_bare_xpub() {
    let d = parse(&format!("wsh(pk({XPUB_B}))"));
    assert!(matches!(
        require_multipath_wildcard(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn rejects_single_wildcard() {
    let d = parse(&format!("wsh(pk({XPUB_B}/0/*))"));
    assert!(matches!(
        require_multipath_wildcard(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn rejects_duplicate_multipath_indices() {
    // <5;5>/* — miniscript colapsa a single wildcard /5/*, que se rechaza
    let d = parse(&format!("wsh(pk({XPUB_B}/<5;5>/*))"));
    assert!(matches!(
        require_multipath_wildcard(&d),
        Err(CoreError::MissingMultipathWildcard)
    ));
}

#[test]
fn accepts_valid_fixture() {
    let d = parse(VALID_FIXTURE.trim());
    assert!(
        require_multipath_wildcard(&d).is_ok(),
        "el fixture válido debe pasar"
    );
}

#[test]
fn accepts_synthetic_2_of_3_multipath() {
    // Triple-key sortedmulti, todos <0;1>/*
    let d = parse(&format!(
        "wsh(sortedmulti(2,{XPUB_B}/<0;1>/*,{XPUB_C}/<0;1>/*,{XPUB_D}/<0;1>/*))"
    ));
    assert!(require_multipath_wildcard(&d).is_ok());
}

#[test]
fn accepts_alternate_multipath_indices() {
    // <2;3>/* es válido — Liana usa este par para recovery paths
    let d = parse(&format!("wsh(pk({XPUB_B}/<2;3>/*))"));
    assert!(require_multipath_wildcard(&d).is_ok());
}

#[test]
fn accepts_mixed_multipath_indices() {
    // Mix <0;1> + <2;3> — válido, cada clave tiene par distinto
    let d = parse(&format!(
        "wsh(sortedmulti(2,{XPUB_B}/<0;1>/*,{XPUB_C}/<2;3>/*))"
    ));
    assert!(require_multipath_wildcard(&d).is_ok());
}
