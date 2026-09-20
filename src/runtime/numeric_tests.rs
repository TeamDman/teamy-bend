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
    for operation in [
        PureOptimization::Add,
        PureOptimization::Mul,
        PureOptimization::Shl,
    ] {
        let parameters = if operation.arity() == 1 {
            "a: U32"
        } else {
            "a: U32, b: U32"
        };
        let source = format!(
            "type U32 is Data: Left{{}} Right{{}}\ndef {}({parameters}) -> U32: a\n",
            operation.name()
        );
        for origins in [&[][..], &["U32"][..], &[operation.name()][..]] {
            let program = checked_program(&source, origins);
            assert!(program.optimizations.is_empty());
            let mut arguments = vec![parse_term("Left{}").unwrap()];
            if operation.arity() == 2 {
                arguments.push(parse_term("Right{}").unwrap());
            }
            let result = program.evaluate(operation.name(), &arguments).unwrap();
            assert_eq!(result.to_string(), "Left{}");
        }
    }
}

#[test]
fn optimization_requires_a_filled_ordinary_body_and_exact_affine_signature() {
    let program = checked_program(
        include_str!("../syntax/base.bend"),
        &["U32", "U32.add", "U32.mul", "U32.shl"],
    );
    assert_eq!(program.optimizations.len(), 3);
    assert!(program.numeric.is_empty());
    assert!(program.foreign.is_empty());
    let strict = Program::from_checked(&program.definitions, &program.datatypes);
    assert!(strict.optimizations.is_empty());
    for operation in [
        PureOptimization::Add,
        PureOptimization::Mul,
        PureOptimization::Shl,
    ] {
        let original = &program.definitions[operation.name()];
        assert!(original.body.is_some());
        let origins = BTreeSet::from(["U32".into(), operation.name().into()]);
        for variant in 0..11 {
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
                6 => {
                    let Term::All { quant, .. } = Rc::make_mut(&mut invalid.ty) else {
                        panic!("function")
                    };
                    *quant = Quant::Many;
                }
                7 => invalid.parameters[0].id += 1,
                8 => invalid.parameters[0].ty = term(Term::Ref("F32".into())),
                9 => {
                    let Term::All { body, .. } = Rc::make_mut(&mut invalid.ty) else {
                        panic!("function")
                    };
                    if operation.arity() == 2 {
                        let Term::All { body, .. } = Rc::make_mut(body) else {
                            panic!("second argument")
                        };
                        *body = term(Term::Ref("F32".into()));
                    } else {
                        *body = term(Term::Ref("F32".into()));
                    }
                }
                10 => invalid.name = "user.other".into(),
                _ => unreachable!(),
            }
            let definitions = BTreeMap::from([(operation.name().into(), invalid)]);
            assert!(
                checked_optimizations(&definitions, &origins).is_empty(),
                "{} variant {variant}",
                operation.name()
            );
        }
    }
}

fn packed_program() -> Program {
    checked_program(
        include_str!("../syntax/base.bend"),
        &[
            "U32", "F32", "Word", "Word.Nil", "Word.Con", "Nat", "Bool", "U32.add", "U32.mul",
            "U32.shl",
        ],
    )
}

#[test]
fn ordinary_word_optimizations_wrap_at_the_declared_width() {
    let program = packed_program();
    assert!(program.packed);
    for (name, arguments, expected) in [
        ("U32.add", vec![u32::MAX, 1], 0),
        ("U32.mul", vec![0, u32::MAX], 0),
        ("U32.mul", vec![3, 7], 21),
        ("U32.mul", vec![65_535, 65_537], u32::MAX),
        ("U32.mul", vec![u32::MAX, u32::MAX], 1),
        ("U32.mul", vec![0x8000_0000, 2], 0),
        ("U32.shl", vec![0], 0),
        ("U32.shl", vec![1], 2),
        ("U32.shl", vec![0x7fff_ffff], 0xffff_fffe),
        ("U32.shl", vec![0x8000_0000], 0),
        ("U32.shl", vec![u32::MAX], 0xffff_fffe),
    ] {
        let arguments = arguments
            .iter()
            .map(|value| parse_term(&value.to_string()).unwrap())
            .collect::<Vec<_>>();
        let actual = program.evaluate(name, &arguments).unwrap();
        let expected = parse_term(&expected.to_string()).unwrap();
        assert_eq!(actual.to_string(), expected.to_string(), "{name}");
    }
}

fn raw_bits_target() -> WordTarget {
    WordTarget::Arguments(NumericArguments {
        operation: NumericOperation::Intrinsic(NumericIntrinsic::Bits),
        arguments: vec![0],
        words: vec![],
    })
}

#[test]
fn packed_numeric_results_and_decoding_keep_exact_nan_payloads_without_word_expansion() {
    let program = packed_program();
    let mut machine = Machine::new(&program);
    for bits in [0, 0x8000_0000, 1, 0x7f80_0001, 0xffc0_4321, u32::MAX] {
        let before = machine.arena.len();
        let input = machine.numeric_word_value("F32", bits).unwrap();
        assert_eq!(machine.arena.len(), before + 1);
        let value = machine.force(input).unwrap();
        let result = machine
            .numeric_step(NumericFrame::Unwrap(raw_bits_target()), value, &mut vec![])
            .unwrap();
        assert_eq!(machine.arena.len(), before + 2);
        assert!(
            matches!(machine.force(result).unwrap(), Value::PackedWord { wrapper: Wrapper::U32, bits: result } if result == bits)
        );
    }
    let strict = Program::from_checked(&program.definitions, &program.datatypes);
    let mut ordinary = Machine::new(&strict);
    ordinary.numeric_word_value("U32", 7).unwrap();
    assert_eq!(
        ordinary.arena.len(),
        66,
        "ordinary fallback representation remains available"
    );
}

#[test]
fn packed_tail_decoder_combines_prefix_bits_and_rejects_invalid_widths_or_wrappers() {
    let program = packed_program();
    for (consumed, prefix, bits, width, expected) in [
        (0, 0, 0xffff_ffff, 32, 0xffff_ffff),
        (1, 1, 0x4000_0000, 31, 0x8000_0001),
        (31, 3, 1, 1, 0x8000_0003),
        (32, 0x7f80_0001, 0, 0, 0x7f80_0001),
    ] {
        let mut machine = Machine::new(&program);
        let result = machine
            .numeric_step(
                NumericFrame::Word(raw_bits_target(), consumed, prefix),
                Value::PackedBits { bits, width },
                &mut vec![],
            )
            .unwrap();
        assert!(
            matches!(machine.force(result).unwrap(), Value::PackedWord { wrapper: Wrapper::U32, bits } if bits == expected)
        );
    }
    for (consumed, bits, width) in [
        (0, 0, 31),
        (0, 0, 33),
        (1, 0x8000_0000, 31),
        (32, 1, 0),
        (33, 0, 0),
    ] {
        let mut machine = Machine::new(&program);
        let error = machine
            .numeric_step(
                NumericFrame::Word(raw_bits_target(), consumed, 0),
                Value::PackedBits { bits, width },
                &mut vec![],
            )
            .unwrap_err()
            .to_string();
        assert!(error.contains("exactly 32 Word bits"), "{error}");
    }
    let mut machine = Machine::new(&program);
    let error = machine
        .numeric_step(
            NumericFrame::Unwrap(raw_bits_target()),
            Value::PackedWord {
                wrapper: Wrapper::U32,
                bits: 0,
            },
            &mut vec![],
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid wrapper"), "{error}");
    for value in [
        Value::PackedBits { bits: 0, width: 32 },
        Value::Request {
            name: "blocked".into(),
            arguments: vec![],
            continuation: 0,
        },
    ] {
        let error = machine
            .numeric_step(NumericFrame::Unwrap(raw_bits_target()), value, &mut vec![])
            .unwrap_err()
            .to_string();
        assert!(error.contains("not constructor data"), "{error}");
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
fn numeric_word_decoder_rejects_wrong_wrappers_lengths_and_bit_values() {
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
                operation: NumericOperation::Intrinsic(NumericIntrinsic::U32ToF32),
                arguments: vec![],
            }))
            .unwrap();
        let first = machine.expression(argument, 0).unwrap();
        let call = machine
            .allocate(Thunk::Application(function, first))
            .unwrap();
        let error = machine
            .force(call)
            .err()
            .expect("malformed internal values fail closed")
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
}

#[test]
fn native_float_text_matches_c_decimal_hex_and_nul_boundaries() {
    // Independent Microsoft C strtof witnesses, using upstream's original
    // n > 0 && *end == 0 success test. NaN payloads have a separate target test.
    for (input, expected) in [
        ("", None),
        ("\0", Some(0)),
        ("\0garbage", Some(0)),
        ("garbage", None),
        (" ", None),
        ("\t\n 1.5", Some(1_069_547_520)),
        ("1.5 ", None),
        ("1.5\n", None),
        ("1.5\0tail", Some(1_069_547_520)),
        ("nan(a-b)", None),
        ("inf", Some(2_139_095_040)),
        ("-infinity", Some(4_286_578_688)),
        ("INFINITY", Some(2_139_095_040)),
        ("0x1.8p+1", Some(1_077_936_128)),
        ("0x1", Some(1_065_353_216)),
        ("0x.8", Some(1_056_964_608)),
        ("0x1p", None),
        ("0x1p-149", Some(1)),
        ("0x1p-150", Some(0)),
        ("0x1.000002p-150", Some(1)),
        ("0x1.fffffep127", Some(2_139_095_039)),
        ("0x1.ffffffp127", Some(2_139_095_040)),
        ("0x1.ffffff00000000000000000001p127", Some(2_139_095_040)),
        ("0x1.fffffdp127", Some(2_139_095_038)),
        ("0x1.000001p0", Some(1_065_353_216)),
        ("0x1.000001000000000000001p0", Some(1_065_353_217)),
        ("0x1.000003p0", Some(1_065_353_218)),
        ("0x1.000003000000000000001p0", Some(1_065_353_218)),
        (".5", Some(1_056_964_608)),
        ("1.", Some(1_065_353_216)),
        ("-0", Some(2_147_483_648)),
        ("1e-9999", Some(0)),
        ("-1e-9999", Some(2_147_483_648)),
        ("1e9999", Some(2_139_095_040)),
        ("-1e9999", Some(4_286_578_688)),
        ("\u{a0}1", None),
        // Guard/sticky rounding must not shift a 32-bit accumulator by 32.
        ("0x1.0000000000000001p-150", Some(1)),
        ("0x0.ffffffp-126", Some(8_388_608)),
        ("-0x1p-999999999999999999999", Some(2_147_483_648)),
        ("0x1p999999999999999999999", Some(2_139_095_040)),
        ("0x", None),
        ("0x.p1", None),
        ("0x1.2.3", None),
        ("1e+", None),
        ("+", None),
        ("  \0", None),
        ("1_0", None),
    ] {
        assert_eq!(text::read(input), expected, "{input:?}");
    }
}

#[test]
fn native_float_nan_spelling_uses_the_documented_crt_payload_policy() {
    for (input, payload) in [
        ("nan", 0),
        ("nan()", 0),
        ("nan(1)", 1),
        ("nan(0x123)", 0x123),
        ("NaN(foo)", 0),
    ] {
        let expected = if cfg!(target_env = "msvc") {
            0x7fff_ffff
        } else {
            0x7fc0_0000 | payload
        };
        assert_eq!(text::read(input), Some(expected));
        assert_eq!(
            text::read(&format!("-{input}")),
            Some(expected | 0x8000_0000)
        );
        assert_eq!(text::read(&format!("+{input}")), Some(expected));
    }
}

#[test]
fn native_float_show_preserves_signed_zero_and_decimal_style_boundaries() {
    for (bits, expected) in [
        (0, "0"),
        (0x8000_0000, "-0"),
        (0x7f80_0000, "inf"),
        (0xff80_0000, "-inf"),
        (0x7f80_0001, "nan"),
        (0xffc0_0000, "nan"),
        (1, "1e-45"),
        (0x007f_ffff, "1.1754942e-38"),
        (0x0080_0000, "1.1754944e-38"),
        (0x7f7f_ffff, "3.4028235e+38"),
        (0x3f80_0000, "1"),
        (0x3eaa_aaab, "0.33333334"),
        (0x3586_37bd, "0.000001"),
        (0x33d6_bf95, "1e-7"),
        (0x60ad_78ec, "100000000000000000000"),
        (0x6258_d727, "1e+21"),
    ] {
        assert_eq!(text::show(f32::from_bits(bits)), expected, "{bits:08x}");
    }
}

#[test]
fn numeric_string_decoder_rejects_malformed_data_and_enforces_text_budget() {
    let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
    for (argument, diagnostic) in [
        ("Unit{}", "String value"),
        ("SCon{Unit{}, SNil{}}", "Char value"),
        ("SCon{Chr{Unit{}}, SNil{}}", "invalid wrapper"),
        ("SCon{Chr{55296}, SNil{}}", "invalid Unicode scalar"),
        ("SCon{Chr{1114112}, SNil{}}", "invalid Unicode scalar"),
    ] {
        let mut machine = Machine::new(&program);
        let function = machine
            .allocate(Thunk::Ready(Value::Numeric {
                operation: NumericOperation::Intrinsic(NumericIntrinsic::Read),
                arguments: vec![],
            }))
            .unwrap();
        let argument = machine
            .expression(parse_term(argument).unwrap(), 0)
            .unwrap();
        let call = machine
            .allocate(Thunk::Application(function, argument))
            .unwrap();
        let error = machine
            .force(call)
            .err()
            .expect("malformed internal text fails closed")
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
    let mut machine = Machine::new(&program);
    let target = WordTarget::Character("x".repeat(super::super::executable::TEXT_BYTES), 0);
    let error = machine
        .numeric_word(target, u32::from('x'), &mut Vec::new())
        .expect_err("text decoding uses the existing byte budget")
        .to_string();
    assert!(error.contains("text byte budget exhausted"), "{error}");
}

fn float_literal(bits: u32) -> TermRef {
    let value = parse_term(&bits.to_string()).unwrap();
    let Term::Ctr { args, .. } = value.as_ref() else {
        panic!("numeric literal")
    };
    term(Term::Ctr {
        name: "F32".into(),
        args: args.clone(),
    })
}

#[test]
fn collection_preserves_partial_numeric_arguments_and_decoder_continuations() {
    let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
    let mut machine = Machine::new(&program);
    machine.gc_mode = super::super::gc::Mode::EverySafePoint;
    let function = machine
        .allocate(Thunk::Ready(Value::Numeric {
            operation: NumericOperation::Intrinsic(NumericIntrinsic::Add),
            arguments: vec![],
        }))
        .unwrap();
    let first = machine.expression(float_literal(0x3f80_0000), 0).unwrap();
    let partial = machine
        .allocate(Thunk::Application(function, first))
        .unwrap();
    assert!(matches!(
        machine.force(partial).unwrap(),
        Value::Numeric { .. }
    ));
    let second = machine.expression(float_literal(0x4000_0000), 0).unwrap();
    let call = machine
        .allocate(Thunk::Application(partial, second))
        .unwrap();
    assert_eq!(
        machine.materialize(call, 0).unwrap().to_string(),
        float_literal(0x4040_0000).to_string()
    );
    assert!(machine.gc.collections > 1);
}

#[test]
fn collection_preserves_numeric_text_tails_through_character_and_word_decoding() {
    let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
    let mut machine = Machine::new(&program);
    machine.gc_mode = super::super::gc::Mode::EverySafePoint;
    let function = machine
        .allocate(Thunk::Ready(Value::Numeric {
            operation: NumericOperation::Intrinsic(NumericIntrinsic::Read),
            arguments: vec![],
        }))
        .unwrap();
    let input = machine
        .expression(parse_term("\"1.5\"").unwrap(), 0)
        .unwrap();
    let call = machine
        .allocate(Thunk::Application(function, input))
        .unwrap();
    let expected = term(Term::Ctr {
        name: "Some".into(),
        args: vec![float_literal(0x3fc0_0000)],
    });
    assert_eq!(
        machine.materialize(call, 0).unwrap().to_string(),
        expected.to_string()
    );
    assert!(machine.gc.collections > 1);
}

#[test]
fn collection_preserves_original_arguments_when_ordinary_optimization_falls_back() {
    let program = packed_program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = super::super::gc::Mode::EverySafePoint;
    let function = machine.reference("U32.add").unwrap();
    let first = machine.expression(parse_term("1").unwrap(), 0).unwrap();
    let second_literal = parse_term("2").unwrap();
    let Term::Ctr { args, .. } = second_literal.as_ref() else {
        panic!("U32 literal")
    };
    // The annotation preserves an ordinary outer wrapper; the Word remains
    // lazy and requires the checked source fallback after the first argument.
    let second = machine
        .expression(
            term(Term::Ctr {
                name: "U32".into(),
                args: vec![term(Term::Ann(
                    Rc::clone(&args[0]),
                    parse_term("Word(32n)").unwrap(),
                ))],
            }),
            0,
        )
        .unwrap();
    let partial = machine
        .allocate(Thunk::Application(function, first))
        .unwrap();
    let call = machine
        .allocate(Thunk::Application(partial, second))
        .unwrap();
    assert_eq!(
        machine.materialize(call, 0).unwrap().to_string(),
        parse_term("3").unwrap().to_string()
    );
    assert!(machine.gc.collections > 1);
}
