// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;

#[derive(facet::Facet)]
struct FixtureExpectation {
    case: String,
    expected: String,
    io: bool,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-meet-diagnostic-precedence")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-contextual-proof-{label}-{}",
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
fn meet_controls_keep_successful_values_and_runtime_results() {
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "fixtures/cpu-meet-diagnostic-precedence/positive-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 2);
    let artifacts = Artifacts::new("positive");
    for case in cases {
        let path = root().join(&case.case);
        let expected = format!("{}\n", case.expected);
        if case.io {
            check_book(&load(&path).unwrap()).expect_err("IO remains excluded from strict proofs");
        } else {
            let book = load(&path).unwrap();
            let normal = check_book(&book)
                .unwrap()
                .normalize_definition("main")
                .unwrap();
            assert_eq!(normal.to_string(), case.expected);
            let expected = teamy_bend::syntax::parse_term(&case.expected).unwrap();
            assert_eq!(
                normalization_shape::shape_in_book(&normal, &book).unwrap(),
                normalization_shape::shape_in_book(&expected, &book).unwrap()
            );
        }
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        // Pure run_main uses normalization; executable C/JS and explicit IO
        // below remain distinct from pure normalization evidence.
        let native_expected = format!("{}\n", case.expected);
        for _ in 0..2 {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0
            );
            assert_eq!(stdout, native_expected.as_bytes(), "{}", case.case);
            assert!(stderr.is_empty());
        }
        let executable = executable_c_compiler::compile(
            &artifacts.0,
            &compile_executable_c(&checked).unwrap(),
            &[],
        );
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(js);
        for mut command in [Command::new(executable), node] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(0), "{}: {output:?}", case.case);
            assert_eq!(output.stdout, expected.as_bytes(), "{}", case.case);
            assert!(output.stderr.is_empty(), "{}: {output:?}", case.case);
        }
    }
}

#[test]
fn meet_errors_keep_exact_precedence_sources_and_exit_status() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-meet-diagnostic-precedence/negative-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 10);
    let artifacts = Artifacts::new("negative");
    for case in cases {
        let path = root().join(&case);
        let expected = std::fs::read_to_string(path.with_extension("stderr")).unwrap();
        let plain_error = check_book(&load(&path).unwrap()).unwrap_err();
        assert_eq!(
            format!("{plain_error}\n"),
            std::fs::read_to_string(path.with_extension("internal")).unwrap(),
            "public error text: {case}"
        );
        for operation in ["check", "eval", "run", "c", "javascript", "native"] {
            let output_path = artifacts.0.join("unwritten");
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if matches!(operation, "check" | "eval" | "run") {
                command.arg(operation).arg(&path);
            } else {
                command
                    .arg("compile")
                    .arg(&path)
                    .args(["--executable", "--target", operation, "--output"])
                    .arg(&output_path);
            }
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(1), "{case}: {operation}");
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes(), "{case}: {operation}");
            assert!(!output_path.exists());
        }
    }
}
