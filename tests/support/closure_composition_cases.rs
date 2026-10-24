// SPDX-License-Identifier: MPL-2.0
use super::normalization_kernel::Term;
use super::normalization_kernel::TermRef;

pub fn assert_binders(value: &TermRef) {
    let Term::Lam {
        id: first, body, ..
    } = value.as_ref()
    else {
        panic!("expected f binder");
    };
    let Term::Lam {
        id: second, body, ..
    } = body.as_ref()
    else {
        panic!("expected g binder");
    };
    let Term::Lam {
        id: input, body, ..
    } = body.as_ref()
    else {
        panic!("expected x binder");
    };
    assert_ne!(first, second);
    assert_ne!(first, input);
    assert_ne!(second, input);
    let Term::App(g, argument) = body.as_ref() else {
        panic!("expected outer call");
    };
    let Term::App(f, x) = argument.as_ref() else {
        panic!("expected inner call");
    };
    assert!(matches!(f.as_ref(), Term::Var { id, .. } if id == first));
    assert!(matches!(g.as_ref(), Term::Var { id, .. } if id == second));
    assert!(matches!(x.as_ref(), Term::Var { id, .. } if id == input));
}

pub struct Case {
    pub first: &'static str,
    pub second: &'static str,
    pub input: u32,
    pub expected: u32,
}

pub fn cases() -> Vec<Case> {
    const FUNCTIONS: [&str; 3] = ["x => x", "x => (x + 1 : U32)", "x => (x * 2 : U32)"];
    fn apply(index: usize, value: u32) -> u32 {
        match index {
            0 => value,
            1 => value.wrapping_add(1),
            2 => value.wrapping_mul(2),
            _ => unreachable!("three fixed functions"),
        }
    }
    let mut cases = Vec::new();
    for (first_index, first) in FUNCTIONS.iter().enumerate() {
        for (second_index, second) in FUNCTIONS.iter().enumerate() {
            for input in [0, 5, u32::MAX] {
                cases.push(Case {
                    first,
                    second,
                    input,
                    expected: apply(second_index, apply(first_index, input)),
                });
            }
        }
    }
    cases
}
