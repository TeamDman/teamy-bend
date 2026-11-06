// SPDX-License-Identifier: MPL-2.0
//! Differential and small-stack tests for iterative substitution and cleanup.
use std::rc::Rc;
use teamy_bend::kernel::LetBinding;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::release_term_iteratively;
use teamy_bend::kernel::substitute;
use teamy_bend::kernel::term;

fn map_constructor(
    value: &TermRef,
    name: &str,
    args: &[TermRef],
    mut transform: impl FnMut(&TermRef) -> TermRef,
) -> TermRef {
    let mapped: Vec<_> = args.iter().map(&mut transform).collect();
    if args.iter().zip(&mapped).all(|(a, b)| Rc::ptr_eq(a, b)) {
        Rc::clone(value)
    } else {
        term(Term::Ctr {
            name: name.into(),
            args: mapped,
        })
    }
}
fn recursive_substitute(value: &TermRef, target: usize, replacement: &TermRef) -> TermRef {
    let sub = |x: &TermRef| recursive_substitute(x, target, replacement);
    match value.as_ref() {
        Term::Var { id, .. } if *id == target => Rc::clone(replacement),
        Term::Var { .. }
        | Term::Ref(_)
        | Term::GpuRef(_)
        | Term::Qnt
        | Term::Qua(_)
        | Term::Efq
        | Term::Rfl
        | Term::Hole(_) => Rc::clone(value),
        Term::Typ(g) => term(Term::Typ(sub(g))),
        Term::Min(a, b) => term(Term::Min(sub(a), sub(b))),
        Term::All {
            quant,
            name,
            id,
            domain,
            body,
        } => term(Term::All {
            quant: *quant,
            name: name.clone(),
            id: *id,
            domain: sub(domain),
            body: if *id == target {
                Rc::clone(body)
            } else {
                sub(body)
            },
        }),
        Term::Lam { name, id, body } => {
            if *id == target {
                Rc::clone(value)
            } else {
                term(Term::Lam {
                    name: name.clone(),
                    id: *id,
                    body: sub(body),
                })
            }
        }
        Term::App(f, x) => map_application(value, f, x, sub),
        Term::Adt {
            name,
            args,
            excluded,
        } => term(Term::Adt {
            name: name.clone(),
            args: args.iter().map(sub).collect(),
            excluded: excluded.clone(),
        }),
        Term::Ctr { name, args } => map_constructor(value, name, args, sub),
        Term::Mat {
            constructor,
            arm,
            fallback,
        } => term(Term::Mat {
            constructor: constructor.clone(),
            arm: sub(arm),
            fallback: sub(fallback),
        }),
        Term::Eql { left, right, ty } => term(Term::Eql {
            left: sub(left),
            right: sub(right),
            ty: sub(ty),
        }),
        Term::Rwt {
            evidence,
            motive,
            body,
        } => term(Term::Rwt {
            evidence: sub(evidence),
            motive: sub(motive),
            body: sub(body),
        }),
        Term::Ann(x, t) => term(Term::Ann(sub(x), sub(t))),
        Term::Let { bindings, body } => term(Term::Let {
            bindings: bindings
                .iter()
                .map(|b| LetBinding {
                    quant: b.quant,
                    name: b.name.clone(),
                    id: b.id,
                    value: sub(&b.value),
                })
                .collect(),
            body: if bindings.iter().any(|b| b.id == target) {
                Rc::clone(body)
            } else {
                sub(body)
            },
        }),
    }
}

fn map_application(
    value: &TermRef,
    f: &TermRef,
    x: &TermRef,
    mut transform: impl FnMut(&TermRef) -> TermRef,
) -> TermRef {
    // An unrelated binder must not copy an already shared computation.
    let next_f = transform(f);
    let next_x = transform(x);
    if Rc::ptr_eq(f, &next_f) && Rc::ptr_eq(x, &next_x) {
        Rc::clone(value)
    } else {
        term(Term::App(next_f, next_x))
    }
}

fn generate(seed: &mut u64, d: usize) -> TermRef {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let tag = (*seed >> 32) % 17;
    if d == 0 {
        return match tag % 4 {
            0 => term(Term::Var {
                name: "x".into(),
                id: 0,
            }),
            1 => term(Term::Var {
                name: "y".into(),
                id: 1,
            }),
            2 => term(Term::Ref("z".into())),
            _ => term(Term::Rfl),
        };
    }
    let a = generate(seed, d - 1);
    let b = generate(seed, d - 1);
    let c = generate(seed, d - 1);
    match tag {
        0 => term(Term::Typ(a)),
        1 => term(Term::Min(a, b)),
        2 => term(Term::All {
            quant: Quant::Lone,
            name: "x".into(),
            id: 0,
            domain: a,
            body: b,
        }),
        3 => term(Term::Lam {
            name: "x".into(),
            id: 0,
            body: a,
        }),
        4 => term(Term::App(a, b)),
        5 => term(Term::Adt {
            name: "T".into(),
            args: vec![a, b],
            excluded: vec!["K".into()],
        }),
        6 => term(Term::Ctr {
            name: "C".into(),
            args: vec![a, b],
        }),
        7 => term(Term::Mat {
            constructor: "C".into(),
            arm: a,
            fallback: b,
        }),
        8 => term(Term::Eql {
            left: a,
            right: b,
            ty: c,
        }),
        9 => term(Term::Rwt {
            evidence: a,
            motive: b,
            body: c,
        }),
        10 => term(Term::Ann(a, b)),
        11 => term(Term::Let {
            bindings: vec![
                LetBinding {
                    quant: Quant::Lone,
                    name: "x".into(),
                    id: 0,
                    value: a,
                },
                LetBinding {
                    quant: Quant::Many,
                    name: "y".into(),
                    id: 1,
                    value: b,
                },
            ],
            body: c,
        }),
        12 => term(Term::GpuRef("gpu".into())),
        13 => term(Term::Qnt),
        14 => term(Term::Qua(Quant::None)),
        15 => term(Term::Efq),
        _ => term(Term::Hole("h".into())),
    }
}

#[test]
fn iterative_matches_original_on_sharing_binders_and_all_term_variants() {
    let replacement = term(Term::Ref("replacement".into()));
    let mut seed = 42;
    for _ in 0..2000 {
        let value = generate(&mut seed, 4);
        for id in [0, 1, 999] {
            let old = recursive_substitute(&value, id, &replacement);
            let new = substitute(&value, id, &replacement);
            assert_eq!(format!("{old:?}"), format!("{new:?}"));
            assert_eq!(Rc::ptr_eq(&old, &value), Rc::ptr_eq(&new, &value));
            assert_eq!(
                Rc::ptr_eq(&old, &replacement),
                Rc::ptr_eq(&new, &replacement)
            );
        }
    }
}

struct Cleanup(Option<TermRef>);
impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(value) = self.0.take() {
            release_term_iteratively(value);
        }
    }
}
fn deep_term(depth: usize) -> TermRef {
    let mut value = term(Term::Var {
        name: "target".into(),
        id: 999,
    });
    for _ in 0..depth {
        value = term(Term::Typ(value));
    }
    value
}
#[test]
fn deep_substitution_and_shared_release_use_small_stack() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let input = Cleanup(Some(deep_term(20_000)));
            let replacement = term(Term::Rfl);
            let output = Cleanup(Some(substitute(
                input.0.as_ref().unwrap(),
                999,
                &replacement,
            )));
            let mut cursor = output.0.as_ref().unwrap();
            for _ in 0..20_000 {
                let Term::Typ(next) = cursor.as_ref() else {
                    panic!("short result")
                };
                cursor = next;
            }
            assert!(Rc::ptr_eq(cursor, &replacement));
            let shared = deep_term(100_000);
            let weak = Rc::downgrade(&shared);
            let retained = Rc::clone(&shared);
            let graph = term(Term::Ctr {
                name: "Shared".into(),
                args: vec![Rc::clone(&shared), shared],
            });
            release_term_iteratively(graph);
            assert!(weak.upgrade().is_some());
            release_term_iteratively(retained);
            assert!(weak.upgrade().is_none());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn iterative_release_can_clean_up_during_unwinding() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let outcome = std::panic::catch_unwind(|| {
                let _cleanup = Cleanup(Some(deep_term(100_000)));
                panic!("deliberate failure after constructing deep term");
            });
            assert!(outcome.is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "Every composite term edge is exercised with a deep owned child."
)]
fn iterative_release_visits_every_composite_edge() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            fn leaf() -> TermRef {
                term(Term::Rfl)
            }
            let wrappers: Vec<Box<dyn Fn(TermRef) -> TermRef>> = vec![
                Box::new(|x| term(Term::Typ(x))),
                Box::new(|x| term(Term::Min(x, leaf()))),
                Box::new(|x| term(Term::Min(leaf(), x))),
                Box::new(|x| term(Term::App(x, leaf()))),
                Box::new(|x| term(Term::App(leaf(), x))),
                Box::new(|x| term(Term::Ann(x, leaf()))),
                Box::new(|x| term(Term::Ann(leaf(), x))),
                Box::new(|x| {
                    term(Term::All {
                        quant: Quant::Lone,
                        name: "x".into(),
                        id: 1,
                        domain: x,
                        body: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::All {
                        quant: Quant::Lone,
                        name: "x".into(),
                        id: 1,
                        domain: leaf(),
                        body: x,
                    })
                }),
                Box::new(|x| {
                    term(Term::Lam {
                        name: "x".into(),
                        id: 1,
                        body: x,
                    })
                }),
                Box::new(|x| {
                    term(Term::Adt {
                        name: "A".into(),
                        args: vec![x],
                        excluded: vec![],
                    })
                }),
                Box::new(|x| {
                    term(Term::Ctr {
                        name: "C".into(),
                        args: vec![x],
                    })
                }),
                Box::new(|x| {
                    term(Term::Mat {
                        constructor: "C".into(),
                        arm: x,
                        fallback: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::Mat {
                        constructor: "C".into(),
                        arm: leaf(),
                        fallback: x,
                    })
                }),
                Box::new(|x| {
                    term(Term::Eql {
                        left: x,
                        right: leaf(),
                        ty: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::Eql {
                        left: leaf(),
                        right: x,
                        ty: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::Eql {
                        left: leaf(),
                        right: leaf(),
                        ty: x,
                    })
                }),
                Box::new(|x| {
                    term(Term::Rwt {
                        evidence: x,
                        motive: leaf(),
                        body: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::Rwt {
                        evidence: leaf(),
                        motive: x,
                        body: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::Rwt {
                        evidence: leaf(),
                        motive: leaf(),
                        body: x,
                    })
                }),
                Box::new(|x| {
                    term(Term::Let {
                        bindings: vec![LetBinding {
                            quant: Quant::Lone,
                            name: "x".into(),
                            id: 1,
                            value: x,
                        }],
                        body: leaf(),
                    })
                }),
                Box::new(|x| {
                    term(Term::Let {
                        bindings: vec![],
                        body: x,
                    })
                }),
            ];
            for wrap in wrappers {
                let value = deep_term(10_000);
                let weak = Rc::downgrade(&value);
                release_term_iteratively(wrap(value));
                assert!(weak.upgrade().is_none());
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
