// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::kernel::Declaration;
use crate::kernel::check_book;
use crate::kernel::term;
use crate::runtime::Program;
use crate::syntax::parse;
use crate::syntax::parse_term;
use std::rc::Rc;

fn checked_program(source: &str, origins: &[&str]) -> Program {
    let book = parse(source).expect("runtime unit fixture parses");
    check_book(&book).expect("runtime unit fixture checks with ordinary strict rules");
    let mut definitions = BTreeMap::new();
    let mut datatypes = BTreeMap::new();
    for declaration in book.declarations {
        match declaration {
            Declaration::Def(definition) => {
                definitions.insert(definition.name.clone(), definition);
            }
            Declaration::Adt(datatype) => {
                datatypes.insert(datatype.name.clone(), datatype);
            }
        }
    }
    Program::from_executable(
        &Rc::new(definitions),
        &Rc::new(datatypes),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &origins.iter().map(|name| (*name).into()).collect(),
    )
}

#[test]
fn user_u32_name_and_signature_keep_the_ordinary_body_without_base_origin() {
    let source = "type U32 is Data: Left{} Right{}\ndef U32.add(a: U32, b: U32) -> U32: a\n";
    for origins in [&[][..], &["U32"][..], &["U32.add"][..]] {
        let program = checked_program(source, origins);
        assert!(program.optimizations.is_empty());
        let result = program
            .evaluate(
                "U32.add",
                &[
                    parse_term("Left{}").unwrap(),
                    parse_term("Right{}").unwrap(),
                ],
            )
            .unwrap();
        assert_eq!(result.to_string(), "Left{}");
    }
}

#[test]
fn optimization_requires_a_filled_ordinary_body_and_exact_affine_signature() {
    let program = checked_program(include_str!("../syntax/base.bend"), &["U32", "U32.add"]);
    assert_eq!(program.optimizations.len(), 1);
    assert!(program.numeric.is_empty());
    assert!(program.foreign.is_empty());
    assert!(program.definitions["U32.add"].body.is_some());
    let strict = Program::from_checked(&program.definitions, &program.datatypes);
    assert!(strict.optimizations.is_empty());
    let original = &program.definitions["U32.add"];
    let origins = BTreeSet::from(["U32".into(), "U32.add".into()]);
    for variant in 0..6 {
        let mut invalid = original.clone();
        match variant {
            0 => invalid.body = None,
            1 => invalid.foreign = true,
            2 => invalid.unsafe_ = true,
            3 => {
                invalid.parameters.pop();
            }
            4 => invalid.parameters[0].quant = Quant::Many,
            5 => {
                let Term::All { domain, .. } = Rc::make_mut(&mut invalid.ty) else {
                    panic!("function")
                };
                *domain = term(Term::Ref("F32".into()));
            }
            _ => unreachable!(),
        }
        let definitions = BTreeMap::from([("U32.add".into(), invalid)]);
        assert!(
            checked_optimizations(&definitions, &origins).is_empty(),
            "variant {variant}"
        );
    }
}

fn raw_word(wrapper: &str, bits: usize, head: &str) -> TermRef {
    let mut word = term(Term::Ctr {
        name: "WNil".into(),
        args: vec![],
    });
    for _ in 0..bits {
        word = term(Term::Ctr {
            name: "WCon".into(),
            args: vec![
                term(Term::Ctr {
                    name: head.into(),
                    args: vec![],
                }),
                word,
            ],
        });
    }
    term(Term::Ctr {
        name: wrapper.into(),
        args: vec![word],
    })
}

#[test]
fn optimized_word_decoder_rejects_wrong_wrappers_lengths_and_bit_values() {
    let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
    for (argument, diagnostic) in [
        (raw_word("F32", 32, "False"), "invalid wrapper"),
        (raw_word("U32", 0, "False"), "32-bit Word"),
        (raw_word("U32", 33, "False"), "exactly 32 Word bits"),
        (raw_word("U32", 32, "Unit"), "invalid Boolean bit"),
    ] {
        let mut machine = Machine::new(&program);
        let function = machine
            .allocate(Thunk::Ready(Value::Numeric {
                operation: NumericOperation::Optimized(PureOptimization::U32Add),
                arguments: vec![],
            }))
            .unwrap();
        let first = machine.expression(argument, 0).unwrap();
        let second = machine.expression(parse_term("1").unwrap(), 0).unwrap();
        let partial = machine
            .allocate(Thunk::Application(function, first))
            .unwrap();
        let call = machine
            .allocate(Thunk::Application(partial, second))
            .unwrap();
        let error = machine
            .force(call)
            .err()
            .expect("malformed internal values fail closed")
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
}
