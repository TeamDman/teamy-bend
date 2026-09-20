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
