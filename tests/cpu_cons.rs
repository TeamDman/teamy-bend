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
use teamy_bend::syntax::parse_term;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-cons")
        .join(format!("{name}.bend"))
}

#[test]
fn original_cons_programs_and_pure_derivative_preserve_output() {
    for (name, expected) in [
        ("cons_operator", "391\n"),
        ("cons_plus_pattern", "12\n"),
        ("cons_operator_pure", "391\n"),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "run"])
            .arg(fixture(name))
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(0), "{name}: {:?}", result.stderr);
        assert_eq!(result.stdout, expected.as_bytes());
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn cons_pure_normalization_preserves_structure_and_text() {
    #[derive(facet::Facet)]
    struct Report {
        entry: String,
        value: String,
    }
    for (name, entry, expected) in [
        ("cons_plus_pattern", "main", "12"),
        ("cons_operator_pure", "main", "391"),
        ("cons_operator_pure", "main.out", "391"),
    ] {
        let file = fixture(name);
        let book = load(&file).unwrap();
        let normal = check_book(&book)
            .unwrap()
            .normalize_definition(entry)
            .unwrap();
        assert_eq!(
            shape_in_book(&normal, &book).unwrap(),
            shape_in_book(&parse_term(expected).unwrap(), &book).unwrap()
        );
        assert_eq!(normal.to_string(), expected);
        let result = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "eval"])
            .arg(file)
            .args(["--entry", entry])
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(0), "{:?}", result.stderr);
        assert!(result.stderr.is_empty());
        let report: Report = facet_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report.entry, entry);
        assert_eq!(report.value, expected);
    }
}

#[test]
fn original_cons_io_remains_outside_strict_proof_mode() {
    let file = fixture("cons_operator");
    assert_eq!(
        check_book(&load(&file).unwrap()).unwrap_err().message,
        "main: undefined name IO"
    );
    let result = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
        .args(["--output-format", "json", "eval"])
        .arg(file)
        .args(["--entry", "main.out"])
        .env("BEND_GPU", "off")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("Error:\n- expected : a defined name\n- observed : IO\nLocation: main\n")
    );
}
