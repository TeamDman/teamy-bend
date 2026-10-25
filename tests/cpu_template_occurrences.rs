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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-template-occurrences")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-template-occurrences-{label}-{}",
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
fn template_occurrences_keep_exact_diagnostics_and_compilation_refusals() {
    let prior = root().with_file_name("cpu-definition-provenance");
    let mut cases: Vec<_> = facet_json::from_str::<Vec<String>>(include_str!(
        "fixtures/cpu-definition-provenance/pending-cases.json"
    ))
    .unwrap()
    .into_iter()
    .map(|case| prior.join(case))
    .collect();
    cases.push(prior.join("controls/qualified-call.bend"));
    cases.extend(
        facet_json::from_str::<Vec<String>>(include_str!(
            "fixtures/cpu-template-occurrences/negative-cases.json"
        ))
        .unwrap()
        .into_iter()
        .map(|case| root().join(case)),
    );
    assert_eq!(cases.len(), 20);
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
fn shared_compile_bindings_still_refuse_lambda_and_pattern_rebinding() {
    for (name, message) in [
        (
            "lambda-binder",
            "a template argument cannot be rebound as a lambda binder",
        ),
        (
            "pattern-binder",
            "a template argument cannot be rebound as a pattern",
        ),
    ] {
        let path = root().join("negative").join(format!("{name}.bend"));
        let error = load(&path).unwrap_err();
        assert_eq!(error.message, message);
        for operation in ["check", "eval", "run"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            command.arg(operation).arg(&path);
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            if name == "pattern-binder" {
                let expected: String = facet_json::from_str(
                    &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
                )
                .unwrap();
                assert_eq!(output.stderr, expected.as_bytes());
            } else {
                assert!(String::from_utf8_lossy(&output.stderr).contains(message));
            }
        }
        // Lambda-binder spelling remains a separate unqualified diagnostic.
    }
}

#[derive(Facet)]
struct Printed {
    value: DataValue,
}
#[test]
fn template_macro_cache_and_shared_values_keep_graphs_and_runtime_values() {
    let artifacts = Artifacts::new("positive");
    for name in ["macro-cache", "shared-value"] {
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
fn occurrence_parser_keeps_nested_rule_guards_on_the_default_stack() {
    for source in [
        format!("{}x", "+ ".repeat(64)),
        format!("{}x", "@x:Type -> ".repeat(64)),
        format!("{}x", "%r: Type; ".repeat(64)),
    ] {
        let error = parse_term(&source).expect_err("original parser nesting guard");
        assert!(error.message.contains("resource limit of 64"), "{error}");
    }
}
