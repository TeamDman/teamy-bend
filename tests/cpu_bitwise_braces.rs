// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape_in_book;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-bitwise-braces")
        .join(format!("{name}.bend"))
}

#[test]
fn original_bitwise_and_brace_programs_preserve_raw_output() {
    for (name, expected) in [
        ("bitwise_dot_operators", "671\n"),
        ("brace_term_forms", "42\n"),
        ("bitwise_dot_operators_pure", "671\n"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "run"])
            .arg(fixture(name))
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{name}: {:?}", output.stderr);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn pure_bitwise_and_brace_definitions_preserve_normalized_core_and_text() {
    #[derive(facet::Facet)]
    struct Report {
        entry: String,
        value: String,
    }
    for (name, entry, expected) in [
        ("bitwise_dot_operators_pure", "main", "671"),
        ("bitwise_dot_operators_pure", "main.out", "671"),
        ("brace_term_forms", "main", "42"),
        ("brace_term_forms", "refl", "{==}"),
        ("brace_term_forms", "app", "f => x => f(x)"),
    ] {
        let source = fixture(name);
        let book = load(&source).unwrap();
        let checked = check_book(&book).unwrap();
        let value = checked.normalize_definition(entry).unwrap();
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(expected).unwrap(), &book).unwrap()
        );
        assert_eq!(value.to_string(), expected);
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "eval"])
            .arg(source)
            .args(["--entry", entry])
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{name}: {:?}", output.stderr);
        assert!(output.stderr.is_empty());
        let report: Report = facet_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report.entry, entry);
        assert_eq!(report.value, expected);
    }
}

#[test]
fn bitwise_io_book_stays_outside_strict_proof_mode() {
    let source = fixture("bitwise_dot_operators");
    let error = check_book(&load(&source).unwrap()).unwrap_err();
    assert_eq!(error.to_string(), "main: undefined name IO");
    for entry in ["main", "main.out"] {
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "eval"])
            .arg(&source)
            .args(["--entry", entry])
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("main: undefined name IO"));
    }
}

#[test]
fn unused_false_brace_reflexivity_still_rejects_the_book() {
    let original = include_str!("fixtures/cpu-bitwise-braces/brace_term_forms.bend");
    let source = original.replace("{U32.add(1, 2) == 3 : U32}", "{U32.add(1, 2) == 4 : U32}");
    assert_ne!(source, original);
    let source = format!(
        "{}\n{}",
        include_str!("../src/syntax/base.bend"),
        source.replace("import Base\n", "")
    );
    let error = check_book(&parse(&source).unwrap()).unwrap_err();
    assert!(error.to_string().starts_with("refl:"));
    assert!(error.to_string().contains("reflexivity"));
}
