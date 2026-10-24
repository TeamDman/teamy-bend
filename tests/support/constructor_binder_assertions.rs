// SPDX-License-Identifier: MPL-2.0
use super::normalization_kernel::Term;
use super::normalization_kernel::TermRef;
use std::collections::BTreeSet;

pub fn assert_fresh_match_binders(value: &TermRef) {
    let Term::Mat {
        arm,
        fallback: outer,
        ..
    } = value.as_ref()
    else {
        panic!("expected outer constructor match");
    };
    let Term::Mat {
        arm,
        fallback: inner,
        ..
    } = arm.as_ref()
    else {
        panic!("expected nested constructor match");
    };
    let mut identities = BTreeSet::new();
    for (value, expected) in [(arm, "x"), (inner, "_4"), (outer, "n")] {
        let Term::Lam { name, id, .. } = value.as_ref() else {
            panic!("expected branch lambda");
        };
        assert_eq!(name, expected);
        assert!(
            identities.insert(*id),
            "distinct branch binders share an identity"
        );
    }
}
