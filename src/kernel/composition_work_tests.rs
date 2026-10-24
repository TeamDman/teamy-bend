// SPDX-License-Identifier: MPL-2.0
use super::*;

fn empty() -> Engine {
    empty_engine(&Book {
        declarations: vec![],
    })
    .unwrap()
}

fn boxed(argument: TermRef) -> TermRef {
    term(Term::App(
        term(Term::Lam {
            name: "x".into(),
            id: 100,
            body: term(Term::Ctr {
                name: "Box".into(),
                args: vec![term(Term::Var {
                    name: "x".into(),
                    id: 100,
                })],
            }),
        }),
        argument,
    ))
}

#[test]
fn substitution_keeps_unaffected_applications_shared_and_changes_actual_uses() {
    let variable = term(Term::Var {
        name: "free".into(),
        id: 101,
    });
    let call = term(Term::App(
        term(Term::Ref("wrap".into())),
        Rc::clone(&variable),
    ));
    let replacement = term(Term::Rfl);
    assert!(Rc::ptr_eq(&substitute(&call, 999, &replacement), &call));
    let changed = substitute(&call, 101, &replacement);
    assert!(!Rc::ptr_eq(&changed, &call));
    let Term::App(_, argument) = changed.as_ref() else {
        unreachable!()
    };
    assert!(Rc::ptr_eq(argument, &replacement));
    assert_eq!(call.to_string(), "wrap(free)");
}

#[test]
fn constructor_memo_is_lazy_charged_and_still_checks_entry_guards() {
    let mut engine = empty();
    let cycle = term(Term::Var {
        name: "cycle".into(),
        id: 101,
    });
    engine.aliases.insert(101, Rc::clone(&cycle));
    let call = boxed(Rc::clone(&cycle));
    engine.normalization_memo = Some(Default::default());
    engine.fuel = 128;
    let value = engine.whnf(&call).unwrap();
    let Term::Ctr { args, .. } = value.as_ref() else {
        unreachable!()
    };
    assert!(Rc::ptr_eq(&args[0], &cycle));
    engine.fuel = 2;
    assert!(Rc::ptr_eq(&engine.whnf(&call).unwrap(), &value));
    assert_eq!((engine.fuel, engine.depth), (1, 0));
    engine.fuel = 0;
    assert!(
        engine
            .whnf(&call)
            .unwrap_err()
            .to_string()
            .contains("budget exhausted")
    );
    engine.fuel = 2;
    engine.depth = 128;
    assert!(
        engine
            .whnf(&call)
            .unwrap_err()
            .to_string()
            .contains("nesting limit exhausted")
    );
    assert_eq!((engine.fuel, engine.depth), (1, 128));
}

#[test]
fn normalization_drops_memo_after_success_and_error_and_never_keeps_stale_aliases() {
    let mut engine = empty();
    let variable = term(Term::Var {
        name: "alias".into(),
        id: 101,
    });
    let call = boxed(Rc::clone(&variable));
    let owners = Rc::strong_count(&call);
    for quant in [Quant::None, Quant::Many] {
        engine.aliases.insert(101, term(Term::Qua(quant)));
        assert_eq!(
            engine.normalize(&call).unwrap().to_string(),
            format!("Box{{{}}}", term(Term::Qua(quant)))
        );
        assert!(engine.normalization_memo.is_none());
        assert_eq!(Rc::strong_count(&call), owners);
    }
    engine.aliases.insert(101, Rc::clone(&variable));
    engine.fuel = 128;
    assert!(
        engine
            .normalize(&call)
            .unwrap_err()
            .to_string()
            .contains("budget exhausted")
    );
    assert_eq!((engine.fuel, engine.depth), (0, 0));
    assert!(engine.normalization_memo.is_none());
    assert_eq!(Rc::strong_count(&call), owners);
}

#[test]
fn shared_heads_preserve_first_field_failure_and_unwind_depth() {
    let cycle = term(Term::Var {
        name: "cycle".into(),
        id: 101,
    });
    let deep = (0..128).fold(term(Term::Rfl), |x, _| boxed(x));
    for (first, second, message) in [
        (Rc::clone(&cycle), Rc::clone(&deep), "budget exhausted"),
        (deep, Rc::clone(&cycle), "nesting limit exhausted"),
    ] {
        let mut engine = empty();
        engine.aliases.insert(101, Rc::clone(&cycle));
        engine.fuel = 10_000;
        let pair = term(Term::Ctr {
            name: "Pair".into(),
            args: vec![first, second],
        });
        let call = boxed(pair);
        assert!(
            engine
                .normalize(&call)
                .unwrap_err()
                .to_string()
                .contains(message)
        );
        assert_eq!(engine.depth, 0);
        assert!(engine.normalization_memo.is_none());
    }
}

#[test]
fn shared_observations_still_exhaust_an_insufficient_total_work_budget() {
    let shared = boxed(term(Term::Rfl));
    let pair = term(Term::Ctr {
        name: "Pair".into(),
        args: vec![Rc::clone(&shared), shared],
    });
    let mut measured = empty();
    measured.fuel = 128;
    let expected = measured.normalize(&pair).unwrap().to_string();
    let required = 128 - measured.fuel;
    assert!(required > 1 && required < 128);
    for fuel in [required - 1, required] {
        let mut engine = empty();
        engine.fuel = fuel;
        let result = engine.normalize(&pair);
        if fuel == required {
            assert_eq!(result.unwrap().to_string(), expected);
        } else {
            assert!(result.unwrap_err().to_string().contains("budget exhausted"));
        }
        assert_eq!((engine.fuel, engine.depth), (0, 0));
        assert!(engine.normalization_memo.is_none());
    }
}

#[test]
fn composition_individual_prefix_and_aggregate_fit_the_original_budget() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-open-closures/wrappers/open_closure_pipe.bend");
    let source = crate::syntax::load(&path).unwrap();
    let checked = check_book(&source).unwrap();
    let body = checked.engine.defs["main"].body.as_ref().unwrap();
    let mut elements = vec![];
    let mut tail = Rc::clone(body);
    while let Term::Ctr { args, .. } = tail.as_ref() {
        if args.is_empty() {
            break;
        }
        elements.push(Rc::clone(&args[0]));
        tail = Rc::clone(&args[1]);
    }
    assert_eq!(elements.len(), 27);
    let expected =
        include_str!("../../tests/fixtures/cpu-open-closures/wrappers/open_closure_pipe.stdout")
            .trim();
    let values = expected
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(", ")
        .collect::<Vec<_>>();
    let mut sum = 0;
    for (i, element) in elements.iter().enumerate() {
        let mut engine = checked.engine.clone();
        engine.reset();
        let value = engine.normalize(element).unwrap();
        assert_eq!(value.to_string(), values[i]);
        assert_eq!(engine.depth, 0);
        assert!(engine.normalization_memo.is_none());
        let used = 2_000_000 - engine.fuel;
        sum += used;
        println!(
            "individual {} used {used} cumulative {sum} value {value}",
            i + 1
        );
    }
    for count in [23, 24, 27] {
        let mut engine = checked.engine.clone();
        engine.reset();
        let mut prefix = Rc::clone(&tail);
        for element in elements[..count].iter().rev() {
            let Term::Ctr { name, .. } = body.as_ref() else {
                unreachable!()
            };
            prefix = term(Term::Ctr {
                name: name.clone(),
                args: vec![Rc::clone(element), prefix],
            });
        }
        let result = if count == 27 {
            engine.normalize_definition("main")
        } else {
            engine.normalize(&prefix)
        }
        .unwrap();
        assert_eq!(
            result.to_string(),
            format!("[{}]", values[..count].join(", "))
        );
        assert_eq!(engine.depth, 0);
        assert!(engine.normalization_memo.is_none());
        println!(
            "prefix {count} used {} result {result}",
            2_000_000 - engine.fuel
        );
    }
}
