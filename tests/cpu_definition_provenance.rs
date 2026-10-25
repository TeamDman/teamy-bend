// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;
use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_c;
use teamy_bend::compiler::compile_javascript;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::protocol::DataValue;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-definition-provenance")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-definition-provenance-{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn constructor_definition_failures_keep_exact_labels_windows_and_status() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-definition-provenance/qualified-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 18);
    let artifacts = Artifacts::new("negative");
    let destination = artifacts.0.join("unwritten");
    for case in cases {
        let path = root().join(&case);
        let expected: String = facet_json::from_str(
            &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
        )
        .unwrap();
        for operation in ["check", "eval", "run", "c", "javascript", "native"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if matches!(operation, "check" | "eval" | "run") {
                command.arg(operation).arg(&path);
            } else {
                command
                    .arg("compile")
                    .arg(&path)
                    .args(["--executable", "--target", operation, "--output"])
                    .arg(&destination);
            }
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(1), "{case}: {operation}");
            assert!(output.stdout.is_empty(), "{case}: {operation}");
            assert_eq!(output.stderr, expected.as_bytes(), "{case}: {operation}");
            assert!(!destination.exists());
        }
    }
}

#[test]
fn pending_template_origins_preserve_rejection_without_claiming_diagnostic_parity() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-definition-provenance/pending-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 5);
    for case in cases {
        let path = root().join(&case);
        let book = teamy_bend::syntax::load(&path).unwrap();
        let error = teamy_bend::kernel::check_book(&book).unwrap_err();
        assert!(
            error.message.ends_with(": undefined name later"),
            "{case}: {error}"
        );
        let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
        command.arg("check").arg(&path);
        let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
        assert_eq!(output.status.code(), Some(1), "{case}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("- observed : later\n"));
        // Upstream source windows are retained as evidence, not asserted equal
        // until occurrence provenance can preserve shared compile bindings.
    }
}

#[derive(Facet)]
struct Printed {
    value: DataValue,
}
#[test]
fn constructor_and_template_values_preserve_complete_graphs_and_pure_runtime_values() {
    let artifacts = Artifacts::new("positive");
    for name in ["constructor-template"] {
        let path = root().join("positive").join(format!("{name}.bend"));
        let expected_text = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
        let expected_graph: Shape =
            facet_json::from_str(&std::fs::read_to_string(path.with_extension("json")).unwrap())
                .unwrap();
        let expected_value =
            DataValue::from_term(&parse_term(expected_text.trim()).unwrap()).unwrap();
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        for _ in 0..2 {
            let normal = checked.normalize_definition("main").unwrap();
            assert_eq!(format!("{normal}\n"), expected_text);
            assert_eq!(shape_in_book(&normal, &book).unwrap(), expected_graph);
            let value = checked.evaluate_data("main", &[]).unwrap();
            assert_eq!(shape_in_book(&value, &book).unwrap(), expected_graph);
        }
        let exe =
            executable_c_compiler::compile(&artifacts.0, &compile_c(&book, "main").unwrap(), &[]);
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_javascript(&book, "main").unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(&js);
        for mut command in [Command::new(exe), node] {
            for _ in 0..2 {
                let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert!(output.status.success(), "{output:?}");
                assert!(output.stderr.is_empty());
                let printed: Printed =
                    facet_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
                assert_eq!(printed.value, expected_value);
            }
        }
    }
}
