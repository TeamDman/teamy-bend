// SPDX-License-Identifier: MPL-2.0
use std::fmt::Write;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

#[test]
fn decimal_word_text_matches_boundary_and_deterministic_sample_values() {
    let mut numbers = vec![0, 1, 9, 10, 11, 99, 100, 101, 999, 1000, u32::MAX];
    for bit in 1..32 {
        let power = 1_u32 << bit;
        numbers.extend([power - 1, power, power + 1]);
    }
    let mut sample = 0x6e62_656e_u32;
    for _ in 0..48 {
        sample = sample.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        numbers.push(sample);
    }
    numbers.sort_unstable();
    numbers.dedup();
    let mut source = include_str!("../src/syntax/base.bend").to_owned();
    for (index, number) in numbers.iter().enumerate() {
        writeln!(source, "\ndef value{index}() -> String: U32.show({number})").unwrap();
    }
    let book = parse(&source).expect("decimal witnesses parse");
    let checked = check_book(&book).expect("ordinary decimal implementation checks");
    for (index, number) in numbers.iter().enumerate() {
        let actual = checked
            .evaluate_data(&format!("value{index}"), &[])
            .unwrap();
        let expected = parse_term(&format!("\"{number}\"")).unwrap();
        assert_eq!(actual.to_string(), expected.to_string(), "{number}");
    }
}

#[test]
fn decimal_helper_apis_preserve_truncation_accumulator_and_explicit_conditions() {
    let mut source = include_str!("../src/syntax/base.bend").to_owned();
    let cases = [
        ("U32.show.go(0n, 1234, \"x\")", "x"),
        ("U32.show.go(2n, 1234, \"x\")", "34x"),
        ("U32.show.go(10n, 1234, \"x\")", "1234x"),
        ("U32.show.go(5n, 0, \"x\")", "x"),
        ("U32.show.fin(1n, \"x\", 1234, False{})", "34x"),
        ("U32.show.fin(0n, \"x\", 1234, False{})", "4x"),
        ("U32.show.fin(3n, \"x\", 0, False{})", "0x"),
        ("U32.show.fin(3n, \"x\", 1234, True{})", "x"),
        ("U32.show.if(1234, True{})", "0"),
        ("U32.show.if(0, False{})", ""),
        ("U32.show.if(1234, False{})", "1234"),
    ];
    for (index, (body, _)) in cases.iter().enumerate() {
        writeln!(source, "\ndef value{index}() -> String: {body}").unwrap();
    }
    let book = parse(&source).expect("decimal helper source parses");
    let checked = check_book(&book).expect("decimal helper source checks");
    for (index, (_, expected)) in cases.iter().enumerate() {
        let actual = checked
            .evaluate_data(&format!("value{index}"), &[])
            .unwrap();
        let expected = parse_term(&format!("\"{expected}\"")).unwrap();
        assert_eq!(actual.to_string(), expected.to_string(), "case {index}");
    }
}
