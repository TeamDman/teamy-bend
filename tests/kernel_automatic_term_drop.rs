// SPDX-License-Identifier: MPL-2.0
//! Ordinary stack-safe destruction across all core term variants.
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::rc::Rc;
use std::rc::Weak;
use teamy_bend::kernel::LetBinding;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::term;

fn leaf() -> TermRef {
    term(Term::Rfl)
}

fn wrap(i: usize, child: TermRef) -> TermRef {
    term(match i % 12 {
        0 => Term::Typ(child),
        1 => Term::Min(child, leaf()),
        2 => Term::All {
            quant: Quant::Many,
            name: "x".into(),
            id: i,
            domain: leaf(),
            body: child,
        },
        3 => Term::Lam {
            name: "x".into(),
            id: i,
            body: child,
        },
        4 => Term::App(leaf(), child),
        5 => Term::Adt {
            name: "A".into(),
            args: vec![leaf(), child],
            excluded: vec![],
        },
        6 => Term::Ctr {
            name: "C".into(),
            args: vec![child, leaf()],
        },
        7 => Term::Mat {
            constructor: "C".into(),
            arm: child,
            fallback: leaf(),
        },
        8 => Term::Eql {
            left: leaf(),
            right: child,
            ty: leaf(),
        },
        9 => Term::Rwt {
            evidence: leaf(),
            motive: child,
            body: leaf(),
        },
        10 => Term::Ann(child, leaf()),
        _ => Term::Let {
            bindings: vec![LetBinding {
                quant: Quant::Lone,
                name: "x".into(),
                id: i,
                value: child,
            }],
            body: leaf(),
        },
    })
}

fn chain() -> TermRef {
    (0..50_000).fold(leaf(), |value, i| wrap(i, value))
}

fn on_small_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn unique_shared_last_and_weak_owners() {
    on_small_stack(|| {
        let root = chain();
        let weak = Rc::downgrade(&root);
        let another = Rc::clone(&root);
        drop(root);
        assert!(weak.upgrade().is_some());
        drop(another);
        assert!(weak.upgrade().is_none());
        let root = chain();
        let weak = Rc::downgrade(&root);
        drop(root);
        assert!(weak.upgrade().is_none());
        let root = chain();
        let weak = Rc::downgrade(&root);
        let owned = Rc::try_unwrap(root).unwrap_or_else(|_| panic!("unique root"));
        assert!(weak.upgrade().is_none());
        drop(owned);
    });
}

#[test]
fn shared_diamond_and_wide_graph() {
    on_small_stack(|| {
        let shared = chain();
        let weak = Rc::downgrade(&shared);
        let held = Rc::clone(&shared);
        let wide = term(Term::Ctr {
            name: "wide".into(),
            args: vec![shared; 20_000],
        });
        drop(wide);
        assert!(weak.upgrade().is_some());
        drop(held);
        assert!(weak.upgrade().is_none());
    });
}

#[test]
fn error_and_unwind_automatically_release() {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "exercise ownership on early return"
    )]
    fn fail(root: TermRef, weak: &mut Weak<Term>) -> Result<(), ()> {
        *weak = Rc::downgrade(&root);
        Err(())
    }
    on_small_stack(|| {
        let mut weak = Weak::new();
        assert!(fail(chain(), &mut weak).is_err());
        assert!(weak.upgrade().is_none());
        let root = chain();
        let weak = Rc::downgrade(&root);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _root = root;
                panic!("probe");
            }))
            .is_err()
        );
        assert!(weak.upgrade().is_none());
    });
}

#[test]
#[expect(clippy::too_many_lines, reason = "enumerate the term child positions")]
fn every_leaf_and_child_position() {
    on_small_stack(|| {
        let leaves = vec![
            Term::Var {
                name: "x".into(),
                id: 0,
            },
            Term::Ref("x".into()),
            Term::GpuRef("x".into()),
            Term::Qnt,
            Term::Qua(Quant::None),
            Term::Efq,
            Term::Rfl,
            Term::Hole("x".into()),
        ];
        drop(leaves);
        for slot in 0..20 {
            let mut root = leaf();
            for i in 0..5_000 {
                root = term(match slot {
                    0 => Term::Min(leaf(), root),
                    1 => Term::All {
                        quant: Quant::Many,
                        name: "x".into(),
                        id: i,
                        domain: root,
                        body: leaf(),
                    },
                    2 => Term::App(root, leaf()),
                    3 => Term::Adt {
                        name: "A".into(),
                        args: vec![root, leaf()],
                        excluded: vec!["C".into()],
                    },
                    4 => Term::Ctr {
                        name: "C".into(),
                        args: vec![leaf(), root],
                    },
                    5 => Term::Mat {
                        constructor: "C".into(),
                        arm: leaf(),
                        fallback: root,
                    },
                    6 => Term::Eql {
                        left: root,
                        right: leaf(),
                        ty: leaf(),
                    },
                    7 => Term::Eql {
                        left: leaf(),
                        right: leaf(),
                        ty: root,
                    },
                    8 => Term::Rwt {
                        evidence: root,
                        motive: leaf(),
                        body: leaf(),
                    },
                    9 => Term::Rwt {
                        evidence: leaf(),
                        motive: leaf(),
                        body: root,
                    },
                    10 => Term::Ann(leaf(), root),
                    11 => Term::Let {
                        bindings: vec![],
                        body: root,
                    },
                    12 => Term::Let {
                        bindings: vec![
                            LetBinding {
                                quant: Quant::None,
                                name: "x".into(),
                                id: i,
                                value: leaf(),
                            },
                            LetBinding {
                                quant: Quant::Many,
                                name: "y".into(),
                                id: i + 1,
                                value: root,
                            },
                        ],
                        body: leaf(),
                    },
                    13 => Term::Typ(root),
                    14 => Term::Lam {
                        name: "x".into(),
                        id: i,
                        body: root,
                    },
                    15 => Term::Min(root, leaf()),
                    16 => Term::App(leaf(), root),
                    17 => Term::Eql {
                        left: leaf(),
                        right: root,
                        ty: leaf(),
                    },
                    18 => Term::Rwt {
                        evidence: leaf(),
                        motive: root,
                        body: leaf(),
                    },
                    _ => Term::Ann(root, leaf()),
                });
            }
            let weak = Rc::downgrade(&root);
            drop(root);
            assert!(weak.upgrade().is_none());
        }
    });
}
