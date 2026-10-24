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
        .join("tests/fixtures/cpu-typed-list")
        .join(format!("{name}.bend"))
}

#[test]
fn original_typed_and_list_programs_and_pure_derivatives_preserve_output() {
    for (name, expected) in [
        ("def_typed", "6\n"),
        ("list_literal_sugar", "3510\n"),
        ("def_typed_pure", "6\n"),
        ("list_literal_sugar_pure", "3510\n"),
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
fn typed_and_list_pure_normalization_preserves_structure_and_text() {
    #[derive(facet::Facet)]
    struct Report {
        entry: String,
        value: String,
    }
    for (name, entry, expected) in [
        ("def_typed_pure", "main", "6"),
        ("list_literal_sugar_pure", "main", "3510"),
        ("list_literal_sugar_pure", "main.out", "3510"),
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
fn original_typed_and_list_io_remain_outside_strict_proof_mode() {
    for name in ["def_typed", "list_literal_sugar"] {
        let file = fixture(name);
        assert_eq!(
            check_book(&load(&file).unwrap()).unwrap_err().message,
            "main: undefined name IO"
        );
        let result = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "eval"])
            .arg(file)
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("main: undefined name IO"));
    }
}
