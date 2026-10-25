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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-pattern-locations")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-pattern-locations-{label}-{}",
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
fn pattern_fields_keep_exact_diagnostics_and_compilation_refusals() {
    let cases: Vec<_> = facet_json::from_str::<Vec<String>>(include_str!(
        "fixtures/cpu-pattern-locations/negative-cases.json"
    ))
    .unwrap()
    .into_iter()
    .map(|case| root().join(case))
    .collect();
    assert_eq!(cases.len(), 11);
    let artifacts = Artifacts::new("negative");
    let destination = artifacts.0.join("unwritten");
    for path in cases {
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
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}: {operation}",
                path.display()
            );
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                expected.as_bytes(),
                "{}: {operation}",
                path.display()
            );
            assert!(!destination.exists());
        }
    }
}

#[test]
fn reusable_function_field_reports_the_let_kind_location() {
    let path = root().join("controls/reusable-refusal.bend");
    let upstream: String = facet_json::from_str(include_str!(
        "fixtures/cpu-pattern-locations/controls/reusable-refusal.stderr.json"
    ))
    .unwrap();
    let historical: String = facet_json::from_str(include_str!(
        "fixtures/cpu-pattern-locations/controls/reusable-refusal.port-stderr.json"
    ))
    .unwrap();
    assert_ne!(historical, upstream); // Preserve the earlier failed observation.
    let book = load(&path).unwrap();
    check_book(&book).unwrap_err();
    for operation in ["check", "eval", "run"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
        command.arg(operation).arg(&path);
        let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, upstream.as_bytes());
    }
}

#[derive(Facet)]
struct Printed {
    value: DataValue,
}
#[test]
fn nested_and_reusable_pattern_values_keep_graphs_and_runtime_values() {
    let artifacts = Artifacts::new("positive");
    for name in ["nested", "reuse"] {
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
fn reusable_lets_keep_exact_diagnostics_and_compilation_refusals() {
    let cases: Vec<_> = facet_json::from_str::<Vec<String>>(include_str!(
        "fixtures/cpu-let-kind-locations/negative-cases.json"
    ))
    .unwrap()
    .into_iter()
    .map(|case| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-let-kind-locations")
            .join(case)
    })
    .collect();
    assert_eq!(cases.len(), 6);
    let artifacts = Artifacts::new("let-negative");
    let destination = artifacts.0.join("unwritten");
    for path in cases {
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
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}: {operation}",
                path.display()
            );
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                expected.as_bytes(),
                "{}: {operation}",
                path.display()
            );
            assert!(!destination.exists());
        }
    }
}

#[test]
fn nested_kind_children_keep_exact_diagnostics_and_compilation_refusals() {
    let cases: Vec<_> = facet_json::from_str::<Vec<String>>(include_str!(
        "fixtures/cpu-nested-kind/negative-cases.json"
    ))
    .unwrap()
    .into_iter()
    .map(|case| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-nested-kind")
            .join(case)
    })
    .collect();
    assert_eq!(cases.len(), 13);
    let artifacts = Artifacts::new("nested-kind-negative");
    let destination = artifacts.0.join("unwritten");
    for path in cases {
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
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}: {operation}",
                path.display()
            );
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                expected.as_bytes(),
                "{}: {operation}",
                path.display()
            );
            assert!(!destination.exists());
        }
    }
}

#[test]
fn invalid_tokens_keep_exact_source_diagnostics_and_failure_status() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-token-expectations");
    let cases: Vec<String> =
        facet_json::from_str(include_str!("fixtures/cpu-token-expectations/cases.json")).unwrap();
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-token-refusal-unwritten-{}",
        std::process::id()
    ));
    assert!(!destination.exists());
    for case in cases {
        let path = root.join(case);
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
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}: {operation}",
                path.display()
            );
            assert!(output.stdout.is_empty(), "{}: {operation}", path.display());
            assert_eq!(
                output.stderr,
                expected.as_bytes(),
                "{}: {operation}",
                path.display()
            );
            assert!(!destination.exists());
        }
    }
}

#[test]
fn declaration_boundaries_keep_exact_diagnostics_and_failure_status() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-declaration-boundary");
    let cases: Vec<String> =
        facet_json::from_str(include_str!("fixtures/cpu-declaration-boundary/cases.json")).unwrap();
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-declaration-refusal-unwritten-{}",
        std::process::id()
    ));
    assert!(!destination.exists());
    for case in cases {
        let path = root.join(case);
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
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}: {operation}",
                path.display()
            );
            assert!(output.stdout.is_empty(), "{}: {operation}", path.display());
            assert_eq!(
                output.stderr,
                expected.as_bytes(),
                "{}: {operation}",
                path.display()
            );
            assert!(!destination.exists());
        }
    }
}

#[test]
fn valid_datatype_boundaries_keep_complete_normal_values() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-declaration-boundary/positive");
    for name in ["empty", "rows", "fields"] {
        let path = root.join(format!("{name}.bend"));
        let expected_text = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
        let expected = parse_term(expected_text.trim()).unwrap();
        let expected_shape = shape_in_book(&expected, &load(&path).unwrap()).unwrap();
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        let normal = checked.normalize_definition("main").unwrap();
        assert_eq!(format!("{normal}\n"), expected_text);
        assert_eq!(shape_in_book(&normal, &book).unwrap(), expected_shape);
        let value = checked.evaluate_data("main", &[]).unwrap();
        assert_eq!(shape_in_book(&value, &book).unwrap(), expected_shape);
    }
}

#[test]
fn lambda_arm_boundaries_keep_exact_diagnostics_and_failure_status() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-lambda-arm-diagnostics");
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-lambda-arm-diagnostics/cases.json"
    ))
    .unwrap();
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-lambda-arm-refusal-unwritten-{}",
        std::process::id()
    ));
    assert!(!destination.exists());
    for case in cases {
        let path = root.join(case);
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
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}: {operation}",
                path.display()
            );
            assert!(output.stdout.is_empty(), "{}: {operation}", path.display());
            assert_eq!(
                output.stderr,
                expected.as_bytes(),
                "{}: {operation}",
                path.display()
            );
            assert!(!destination.exists());
        }
    }
}

#[test]
fn named_lambda_arms_and_fallbacks_keep_complete_normal_values() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-lambda-arm-diagnostics/positive");
    for name in ["constructors", "fallback", "function-tail", "numeric-tail"] {
        let path = root.join(format!("{name}.bend"));
        let expected_text = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
        let expected = parse_term(expected_text.trim()).unwrap();
        let expected_shape = shape_in_book(&expected, &load(&path).unwrap()).unwrap();
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        let normal = checked.normalize_definition("main").unwrap();
        assert_eq!(format!("{normal}\n"), expected_text);
        assert_eq!(shape_in_book(&normal, &book).unwrap(), expected_shape);
        let value = checked.evaluate_data("main", &[]).unwrap();
        assert_eq!(shape_in_book(&value, &book).unwrap(), expected_shape);
    }
}
