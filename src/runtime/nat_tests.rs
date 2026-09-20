// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::kernel::term;
use crate::runtime::Program;
use crate::syntax::parse_term;

fn program() -> Program {
    BASE.with(|(definitions, datatypes)| {
        let origins = definitions
            .keys()
            .chain(datatypes.keys())
            .cloned()
            .collect();
        Program::from_executable(
            &Rc::new(definitions.clone()),
            &Rc::new(datatypes.clone()),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &origins,
        )
    })
}

fn call(machine: &mut Machine<'_>, name: &str, arguments: &[ThunkId]) -> ThunkId {
    let mut result = machine.reference(name).unwrap();
    for argument in arguments {
        result = machine
            .allocate(Thunk::Application(result, *argument))
            .unwrap();
    }
    result
}

fn poisoned(machine: &mut Machine<'_>, name: &str) -> ThunkId {
    machine.expression(term(Term::Ref(name.into())), 0).unwrap()
}

#[test]
fn optimizations_require_exact_transitive_bodies_and_sealed_origins() {
    let program = program();
    assert_eq!(program.nat_optimizations.len(), 5);
    let origins = program
        .definitions
        .keys()
        .chain(program.datatypes.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    assert!(
        checked_optimizations(&program.definitions, &program.datatypes, &BTreeSet::new())
            .is_empty()
    );
    for (name, operation) in [
        ("Nat.sub", Operation::Sub),
        ("Nat.cmp", Operation::Cmp),
        ("Nat.show.fin", Operation::Show),
        ("U32.inc", Operation::ToU32),
        ("Word.to_nat", Operation::FromU32),
    ] {
        let mut definitions = program.definitions.as_ref().clone();
        definitions.get_mut(name).unwrap().body = Some(term(Term::Ctr {
            name: "Zero".into(),
            args: vec![],
        }));
        assert!(
            !checked_optimizations(&definitions, &program.datatypes, &origins).contains(&operation),
            "changed {name}"
        );
        let mut missing = origins.clone();
        missing.remove(name);
        assert!(
            !checked_optimizations(&program.definitions, &program.datatypes, &missing)
                .contains(&operation),
            "unsealed {name}"
        );
    }
    let strict = Program::from_checked(&program.definitions, &program.datatypes);
    assert!(strict.nat_optimizations.is_empty());
    Machine::new(&strict)
        .nat(1)
        .expect_err("strict programs never manufacture packed Nat");
}

#[test]
fn body_guards_compare_binding_identity_instead_of_printed_variable_names() {
    let lambda = |binder, variable| {
        term(Term::Lam {
            name: "a".into(),
            id: binder,
            body: term(Term::Var {
                name: "a".into(),
                id: variable,
            }),
        })
    };
    assert!(Shape::default().term(&lambda(3, 3), &lambda(7, 7)));
    assert!(!Shape::default().term(&lambda(3, 4), &lambda(7, 7)));
    let source = format!(
        "def earlier(-A: Data, x: A) -> A: x\n{}",
        include_str!("../syntax/base.bend")
    );
    let source = crate::syntax::parse(&source).unwrap();
    let mut definitions = BTreeMap::new();
    let mut datatypes = BTreeMap::new();
    for declaration in source.declarations {
        match declaration {
            Declaration::Def(definition) => {
                definitions.insert(definition.name.clone(), definition);
            }
            Declaration::Adt(datatype) => {
                datatypes.insert(datatype.name.clone(), datatype);
            }
        }
    }
    let origins = definitions
        .keys()
        .chain(datatypes.keys())
        .cloned()
        .collect();
    assert_eq!(
        checked_optimizations(&definitions, &datatypes, &origins).len(),
        5
    );
}

#[test]
fn optimized_dependency_closure_includes_datatype_terms_inside_word() {
    let program = program();
    let word = &program.definitions["Word"];
    let mut shape = Shape::default();
    assert!(shape.definition(word, word));
    assert!(shape.references.contains("Word.Nil"));
    assert!(shape.references.contains("Word.Con"));
    let origins = program
        .definitions
        .keys()
        .chain(program.datatypes.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut missing = origins.clone();
    missing.remove("Word.Nil");
    assert!(
        !checked_optimizations(&program.definitions, &program.datatypes, &missing)
            .contains(&Operation::FromU32)
    );
    let mut changed = program.datatypes.as_ref().clone();
    changed.get_mut("Word.Nil").unwrap().constructors[0].name = "OtherNil".into();
    assert!(
        !checked_optimizations(&program.definitions, &changed, &origins)
            .contains(&Operation::FromU32)
    );
}

#[test]
fn compact_values_match_one_constructor_and_preserve_output_limits() {
    let program = program();
    for value in [0, 1, 95, u64::from(u32::MAX) + 1, MAX] {
        let mut machine = Machine::new(&program);
        let id = machine.nat(value).unwrap();
        let result = machine.force(id).unwrap();
        let (name, fields) = machine.constructor_value(result).unwrap();
        if value == 0 {
            assert_eq!(name, "Zero");
            assert!(fields.is_empty());
        } else {
            assert_eq!(name, "Succ");
            assert!(
                matches!(machine.force(fields[0]).unwrap(), Value::PackedNat(n) if n == value - 1)
            );
        }
        if value < 96 {
            let materialized = machine.materialize(id, 0).unwrap();
            assert_eq!(
                materialized.to_string(),
                parse_term(&format!("{value}n")).unwrap().to_string()
            );
            assert_eq!(machine.output_nodes, usize::try_from(value + 1).unwrap());
        } else {
            assert!(
                machine
                    .materialize(id, 0)
                    .unwrap_err()
                    .to_string()
                    .contains("output depth or node budget")
            );
        }
        assert!(machine.arena.len() < 110);
    }
    Machine::new(&program)
        .nat(MAX + 1)
        .expect_err("native Nat rejects values above 48 bits");
}

#[test]
fn literals_do_not_unfold_or_evaluate_a_successor_tail() {
    for tail in [
        term(Term::Ref("bad_tail".into())),
        term(Term::Var {
            name: "n".into(),
            id: 0,
        }),
    ] {
        assert!(literal("Succ", &[tail]).is_none());
    }
    let program = program();
    let strict = Program::from_checked(&program.definitions, &program.datatypes);
    for (program, packed) in [(&program, true), (&strict, false)] {
        let mut machine = Machine::new(program);
        let id = machine.expression(parse_term("32n").unwrap(), 0).unwrap();
        assert_eq!(
            matches!(machine.force(id).unwrap(), Value::PackedNat(32)),
            packed
        );
    }
}

#[test]
fn full_width_timestamps_subtract_compare_show_and_convert_without_unary_expansion() {
    let program = program();
    for value in [0, 1, 123_456_789_012, u64::from(u32::MAX), MAX] {
        let mut machine = Machine::new(&program);
        let a = machine.nat(value).unwrap();
        let b = machine.nat(value.saturating_sub(50)).unwrap();
        let difference = call(&mut machine, "Nat.sub", &[a, b]);
        assert!(
            matches!(machine.force(difference).unwrap(), Value::PackedNat(n) if n == value.min(50))
        );
        let comparison = call(&mut machine, "Nat.cmp", &[a, a]);
        assert!(
            matches!(machine.force(comparison).unwrap(), Value::Constructor { name, fields } if name == "EQ" && fields.is_empty())
        );
        let low = call(&mut machine, "U32.from_nat", &[a]);
        assert!(
            matches!(machine.force(low).unwrap(), Value::PackedWord { bits, wrapper: Wrapper::U32 } if u64::from(bits) == value & u64::from(u32::MAX))
        );
        let shown = call(&mut machine, "Nat.show", &[a]);
        let mut text = String::new();
        let mut current = shown;
        loop {
            let Value::Constructor { name, fields } = machine.force(current).unwrap() else {
                panic!("string")
            };
            if name == "SNil" {
                break;
            }
            assert_eq!(name, "SCon");
            let Value::Constructor {
                name,
                fields: character,
            } = machine.force(fields[0]).unwrap()
            else {
                panic!("character")
            };
            assert_eq!(name, "Chr");
            let Value::PackedWord { bits, .. } = machine.force(character[0]).unwrap() else {
                panic!("codepoint")
            };
            text.push(char::from_u32(bits).unwrap());
            current = fields[1];
        }
        assert_eq!(text, value.to_string());
        assert!(machine.arena.len() < 150);
    }
}

#[test]
fn sampled_optimized_results_equal_checked_ordinary_bodies() {
    let program = program();
    let strict = Program::from_checked(&program.definitions, &program.datatypes);
    for a in 0..=7 {
        for b in 0..=7 {
            let arguments = [
                parse_term(&format!("{a}n")).unwrap(),
                parse_term(&format!("{b}n")).unwrap(),
            ];
            for name in ["Nat.sub", "Nat.cmp", "Nat.is_lt", "Nat.is_ge"] {
                assert_eq!(
                    program.evaluate(name, &arguments).unwrap().to_string(),
                    strict.evaluate(name, &arguments).unwrap().to_string(),
                    "{name}({a}, {b})"
                );
            }
        }
    }
    for a in [0, 1, 9, 10, 31, 63] {
        let arguments = [parse_term(&format!("{a}n")).unwrap()];
        for name in ["Nat.show", "U32.from_nat"] {
            assert_eq!(
                program.evaluate(name, &arguments).unwrap().to_string(),
                strict.evaluate(name, &arguments).unwrap().to_string(),
                "{name}({a})"
            );
        }
        let arguments = [parse_term(&a.to_string()).unwrap()];
        assert_eq!(
            program
                .evaluate("U32.to_nat", &arguments)
                .unwrap()
                .to_string(),
            strict
                .evaluate("U32.to_nat", &arguments)
                .unwrap()
                .to_string()
        );
    }
}

#[test]
fn comparison_and_subtraction_preserve_unused_tails_and_source_order() {
    let program = program();
    for name in ["Nat.sub", "Nat.cmp"] {
        let mut machine = Machine::new(&program);
        let zero = machine.nat(0).unwrap();
        let poison = poisoned(&mut machine, "unused_tail");
        let successor = machine.ready_constructor("Succ", vec![poison]).unwrap();
        let result = call(&mut machine, name, &[zero, successor]);
        let value = machine.force(result).unwrap();
        let (constructor, fields) = machine.constructor_value(value).unwrap();
        assert_eq!(constructor, if name == "Nat.sub" { "Zero" } else { "LT" });
        assert!(fields.is_empty());
        let result = call(&mut machine, name, &[successor, zero]);
        let value = machine.force(result).unwrap();
        let (constructor, _) = machine.constructor_value(value).unwrap();
        assert_eq!(constructor, if name == "Nat.sub" { "Succ" } else { "GT" });
        let first = poisoned(&mut machine, "first_argument");
        let second = poisoned(&mut machine, "second_argument");
        let result = call(&mut machine, name, &[first, second]);
        assert!(
            machine
                .force(result)
                .err()
                .unwrap()
                .to_string()
                .contains("first_argument")
        );
    }
}

#[test]
fn reconstructed_large_successor_tails_remain_compact_through_recursive_fallback() {
    let program = program();
    let mut machine = Machine::new(&program);
    let tail = machine.nat(MAX - 1).unwrap();
    let a = machine.ready_constructor("Succ", vec![tail]).unwrap();
    let b = machine.ready_constructor("Succ", vec![tail]).unwrap();
    for (name, expected) in [("Nat.sub", "Zero"), ("Nat.cmp", "EQ")] {
        let result = call(&mut machine, name, &[a, b]);
        let result = machine.force(result).unwrap();
        assert_eq!(machine.constructor_value(result).unwrap().0, expected);
    }
    let result = call(&mut machine, "U32.from_nat", &[a]);
    assert!(matches!(
        machine.force(result).unwrap(),
        Value::PackedWord { bits: u32::MAX, .. }
    ));
    let overflow = machine.nat(MAX).unwrap();
    let overflow = machine.ready_constructor("Succ", vec![overflow]).unwrap();
    let result = call(&mut machine, "Nat.show", &[overflow]);
    assert!(
        machine
            .force(result)
            .err()
            .unwrap()
            .to_string()
            .contains("48-bit bound")
    );
}

#[test]
fn nat_frames_and_partial_calls_remain_live_with_collection_at_every_transition() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = super::super::gc::Mode::EverySafePoint;
    let a = machine.nat(MAX).unwrap();
    let b = machine.nat(MAX - 37).unwrap();
    let difference = call(&mut machine, "Nat.sub", &[a, b]);
    assert!(matches!(
        machine.force(difference).unwrap(),
        Value::PackedNat(37)
    ));
    assert!(machine.gc.collections > 0);
}

#[test]
fn ordinary_u32_conversion_does_not_force_an_unused_word_tail() {
    let program = program();
    let mut machine = Machine::new(&program);
    let one = machine.ready_constructor("True", vec![]).unwrap();
    let tail = poisoned(&mut machine, "unused_word_tail");
    let word = machine.ready_constructor("WCon", vec![one, tail]).unwrap();
    let word = machine.ready_constructor("U32", vec![word]).unwrap();
    let result = call(&mut machine, "U32.to_nat", &[word]);
    let value = machine.force(result).unwrap();
    assert_eq!(machine.constructor_value(value).unwrap().0, "Succ");
}
