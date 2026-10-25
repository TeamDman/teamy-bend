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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-synthesized-origins")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-synthesized-origins-{label}-{}",
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
fn synthesized_reference_failures_keep_exact_windows_and_failure_status() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-synthesized-origins/negative-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 12);
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

#[derive(Facet)]
struct Printed {
    value: DataValue,
}
#[test]
fn declared_sugar_heads_preserve_complete_normalization_and_pure_runtime_values() {
    let artifacts = Artifacts::new("positive");
    for name in ["logical-sugar", "monadic-sugar"] {
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

#[test]
fn actual_io_remains_an_executable_contract_and_not_a_strict_proof() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-framed-cell/framed_cell_dep.bend");
    assert_eq!(
        check_book(&load(&path).unwrap()).unwrap_err().message,
        "main: undefined name IO"
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
    command.arg("run").arg(&path);
    let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"3\n");
    assert!(output.stderr.is_empty());
    for operation in ["check", "eval"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
        command.arg(operation).arg(&path);
        let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("- expected : a defined name\n- observed : IO\n")
        );
    }
}
