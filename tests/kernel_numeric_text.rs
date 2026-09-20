// SPDX-License-Identifier: MPL-2.0
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn compare(result_type: &str, cases: &[(&str, &str)]) {
    let mut source = include_str!("../src/syntax/base.bend").to_owned();
    for (index, (body, _)) in cases.iter().enumerate() {
        writeln!(source, "\ndef value{index}() -> {result_type}: {body}").unwrap();
    }
    let book = parse(&source).expect("numeric text source parses");
    let checked = check_book(&book).expect("all ordinary definitions and proofs check");
    for (index, (body, expected)) in cases.iter().enumerate() {
        let actual = checked
            .evaluate_data(&format!("value{index}"), &[])
            .unwrap_or_else(|error| panic!("{body}: {error}"));
        let expected = parse_term(expected).expect("independent expected constructor value");
        assert_eq!(actual.to_string(), expected.to_string(), "{body}");
    }
}

#[test]
fn word_decimal_reader_checks_empty_digits_overflow_and_leading_zeroes() {
    compare(
        "Maybe<&2, U32>",
        &[
            ("U32.read(\"\")", "None{}"),
            ("U32.read(\"0\")", "Some{0}"),
            ("U32.read(\"00042\")", "Some{42}"),
            ("U32.read(\"4294967295\")", "Some{4294967295}"),
            ("U32.read(\"4294967296\")", "None{}"),
            ("U32.read(\"9999999999\")", "None{}"),
            ("U32.read(\"-1\")", "None{}"),
            ("U32.read(\"+1\")", "None{}"),
            ("U32.read(\" 1\")", "None{}"),
            ("U32.read(\"1 \")", "None{}"),
            ("U32.read(\"1/\")", "None{}"),
            ("U32.read(\"1:\")", "None{}"),
            ("U32.read(\"é\")", "None{}"),
            ("U32.read(\"1\\0\")", "None{}"),
        ],
    );
}

#[test]
fn word_reader_helpers_preserve_explicit_accumulators_and_conditions() {
    compare(
        "Maybe<&2, U32>",
        &[
            ("U32.read.go(\"\", 4294967295)", "Some{4294967295}"),
            ("U32.read.go(\"5\", 429496729)", "Some{4294967295}"),
            ("U32.read.go(\"6\", 429496729)", "None{}"),
            ("U32.read.go(\"0\", 429496730)", "None{}"),
            ("U32.read.if(\"\", 7, True{})", "Some{7}"),
            ("U32.read.if(\"3\", 12, True{})", "Some{123}"),
            ("U32.read.if(\"\", 7, False{})", "None{}"),
        ],
    );
}

#[test]
fn natural_reader_accepts_small_values_without_materializing_its_large_bound() {
    compare(
        "Maybe<&2, Nat>",
        &[
            ("Nat.read(\"\")", "None{}"),
            ("Nat.read(\"0\")", "Some{0n}"),
            ("Nat.read(\"0007\")", "Some{7n}"),
            ("Nat.read(\"42\")", "Some{42n}"),
            ("Nat.read(\"-1\")", "None{}"),
            ("Nat.read(\" 1\")", "None{}"),
            ("Nat.read(\"1:\")", "None{}"),
            ("Nat.read(\"1\\0\")", "None{}"),
            ("Nat.read.go(\"\", 64n)", "Some{64n}"),
            ("Nat.read.go(\"3\", 1n)", "Some{13n}"),
            ("Nat.read.if(\"\", 1n, 3, True{})", "Some{13n}"),
            ("Nat.read.if(\"\", 1n, 3, False{})", "None{}"),
        ],
    );
    compare(
        "Bool",
        &[
            ("Nat.read.fit(4n, (4n, 9n))", "True{}"),
            ("Nat.read.fit(4n, (3n, 0n))", "False{}"),
            ("Nat.read.bound_text(\"28147497671065\", 5, EQ{})", "True{}"),
            (
                "Nat.read.bound_text(\"28147497671065\", 6, EQ{})",
                "False{}",
            ),
        ],
    );
}

#[test]
fn reader_wrong_answers_and_false_commutativity_variants_are_not_proofs() {
    for (label, equality) in [
        ("word", "{U32.read(\"\") == Some{0} : Maybe<&2, U32>}"),
        (
            "natural",
            "{Nat.read.go(\"\", 1n) == Some{2n} : Maybe<&2, Nat>}",
        ),
        ("addition", "{U32.add(1, 2) == U32.add(2, 2) : U32}"),
    ] {
        let source = format!(
            "{}\nlaw wrong: {equality}\ndef wrong(): {{==}}\n",
            include_str!("../src/syntax/base.bend")
        );
        let parsed = parse(&source).expect("false equality remains well formed");
        let error = check_book(&parsed).expect_err("false equality must reject");
        let message = error.to_string();
        assert!(message.contains("wrong"), "{label}: {message}");
        assert!(
            !message.contains("budget") && !message.contains("limit"),
            "{label}: the negative control must expose a false equality: {message}"
        );
    }
}

#[test]
fn large_native_nat_results_fail_closed_at_existing_representation_limits() {
    let source = format!(
        "{}\ndef value() -> Maybe<&2, Nat>: Nat.read(\"281474976710655\")\n",
        include_str!("../src/syntax/base.bend")
    );
    let book = parse(&source).unwrap();
    let checked = check_book(&book).unwrap();
    let error = checked
        .evaluate_data("value", &[])
        .expect_err("the bounded unary runtime cannot materialize a 48-bit maximum Nat");
    let message = error.to_string();
    assert!(
        message.contains("budget") || message.contains("limit"),
        "{message}"
    );
}

struct Script(PathBuf);

impl Drop for Script {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn generated_native_nat_reader_retains_the_full_48_bit_contract() {
    let path = std::env::temp_dir().join(format!(
        "teamy-bend-numeric-text-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let fixture = Script(path);
    let cases = [
        (
            "Nat.read(\"281474976710655\")",
            "Maybe<&2, Nat>",
            "Some{281474976710655n}",
        ),
        ("Nat.read(\"281474976710656\")", "Maybe<&2, Nat>", "None{}"),
        ("Nat.read(\"999999999999999\")", "Maybe<&2, Nat>", "None{}"),
        (
            "Nat.read(\"000281474976710655\")",
            "Maybe<&2, Nat>",
            "Some{281474976710655n}",
        ),
        ("Nat.read.max()", "Nat", "281474976710655n"),
        (
            "Nat.read.go(\"\", Nat.read.max())",
            "Maybe<&2, Nat>",
            "Some{281474976710655n}",
        ),
        (
            "Nat.read.go(\"5\", Nat.div(Nat.read.max(), 10n))",
            "Maybe<&2, Nat>",
            "Some{281474976710655n}",
        ),
        (
            "Nat.read.go(\"6\", Nat.div(Nat.read.max(), 10n))",
            "Maybe<&2, Nat>",
            "None{}",
        ),
    ];
    for (expression, result_type, expected) in cases {
        let input = fixture.0.join("main.bend");
        fs::write(
            &input,
            format!("import Base\ndef main() -> {result_type}: {expression}\n"),
        )
        .unwrap();
        let source = load_executable(&input).unwrap();
        let checked = check_executable(&source).unwrap();
        let program = compile_executable_javascript(&checked).unwrap();
        let output_path = fixture.0.join("main.cjs");
        fs::write(&output_path, program).unwrap();
        let output =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
                .arg(output_path)
                .output()
                .expect("numeric compiler tests require Node.js");
        assert!(
            output.status.success(),
            "{expression}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{expected}\n"),
            "{expression}"
        );
    }
}
