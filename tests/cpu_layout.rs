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
        .join("tests/fixtures/cpu-layout")
        .join(format!("{name}.bend"))
}

#[test]
fn original_layout_programs_preserve_streams_and_exit_status() {
    for (name, expected) in [
        ("match_indent_column", "Ff7{}\n"),
        ("ctor_row_layout", "Ss{Ss{Ss{Zz{}}}}\n"),
        ("case_body_newline", "Ff{}\n"),
        ("tab_character", "Qv1{}\n"),
        ("if_branch_newline", "7\n"),
        ("elif_alignment", "56\n"),
        ("do_line_depth", "1n\n"),
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
fn pure_layout_normalization_preserves_actual_core_and_text() {
    #[derive(facet::Facet)]
    struct Report {
        entry: String,
        value: String,
    }
    for (name, expected) in [
        ("match_indent_column", "Ff7{}"),
        ("ctor_row_layout", "Ss{Ss{Ss{Zz{}}}}"),
        ("case_body_newline", "Ff{}"),
        ("tab_character", "Qv1{}"),
        ("if_branch_newline_pure", "7"),
        ("elif_alignment_pure", "56"),
        ("do_line_depth_pure", "1n"),
    ] {
        let file = fixture(name);
        let book = load(&file).unwrap();
        let checked = check_book(&book).unwrap();
        let entries: &[&str] = if name.ends_with("_pure") {
            &["main", "main.out"]
        } else {
            &["main"]
        };
        for entry in entries {
            let normal = checked.normalize_definition(entry).unwrap();
            assert_eq!(
                shape_in_book(&normal, &book).unwrap(),
                shape_in_book(&parse_term(expected).unwrap(), &book).unwrap()
            );
            assert_eq!(normal.to_string(), expected);
            let result = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
                .args(["--output-format", "json", "eval"])
                .arg(&file)
                .args(["--entry", entry])
                .env("BEND_GPU", "off")
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(0), "{name}: {:?}", result.stderr);
            assert!(result.stderr.is_empty());
            let report: Report = facet_json::from_slice(&result.stdout).unwrap();
            assert_eq!(report.entry, *entry);
            assert_eq!(report.value, expected);
        }
    }
}

#[test]
fn original_layout_io_books_remain_outside_strict_proof_mode() {
    for name in ["if_branch_newline", "elif_alignment", "do_line_depth"] {
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
        assert!(
            String::from_utf8_lossy(&result.stderr)
                .contains("Error:\n- expected : a defined name\n- observed : IO\nLocation: main\n")
        );
    }
}
