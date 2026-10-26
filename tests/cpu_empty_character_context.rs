// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

#[derive(facet::Facet)]
struct FixtureExpectation {
    case: String,
    path: String,
    kind: String,
    io: Option<bool>,
    exact: Option<bool>,
    expected: Option<String>,
    line: Option<usize>,
    column: Option<usize>,
    message: Option<String>,
    error_source: Option<String>,
}

#[test]
fn empty_character_diagnostics_keep_failure_status_and_refuse_every_emitter() {
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "fixtures/cpu-empty-character-context/cases.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let scratch = std::env::temp_dir().join(format!(
        "bend-empty-character-context-{}",
        std::process::id()
    ));
    std::fs::create_dir(&scratch).unwrap();
    let mut exact = 0;
    let mut different = 0;
    for case in cases.into_iter().filter(|c| c.kind == "negative") {
        let expected = std::fs::read(
            root.join("cpu-empty-character-context")
                .join(case.expected.unwrap()),
        )
        .unwrap();
        let mut prior = None;
        for operation in ["check", "eval", "run", "c", "javascript", "native"] {
            let output_path = scratch.join("unwritten");
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if matches!(operation, "check" | "eval" | "run") {
                command.arg(operation).arg(root.join(&case.path));
            } else {
                command
                    .arg("compile")
                    .arg(root.join(&case.path))
                    .args(["--executable", "--target", operation, "--output"])
                    .arg(&output_path);
            }
            let result = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(result.status.code(), Some(1), "{}: {operation}", case.case);
            assert!(result.stdout.is_empty());
            assert!(!output_path.exists());
            if case.exact.unwrap() {
                assert_eq!(result.stderr, expected, "{}: {operation}", case.case);
            } else {
                assert_ne!(
                    result.stderr, expected,
                    "retained difference unexpectedly closed: {}",
                    case.case
                );
            }
            if let Some(prior) = &prior {
                assert_eq!(&result.stderr, prior);
            }
            prior = Some(result.stderr);
        }
        if case.exact.unwrap() {
            exact += 1;
        } else {
            different += 1;
        }
    }
    assert_eq!((exact, different), (18, 4));
    std::fs::remove_dir(scratch).unwrap();
}

#[test]
fn accepted_apostrophe_and_template_controls_still_execute() {
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "fixtures/cpu-empty-character-context/cases.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let scratch = std::env::temp_dir().join(format!(
        "bend-empty-character-positive-{}",
        std::process::id()
    ));
    std::fs::create_dir(&scratch).unwrap();
    for case in cases.into_iter().filter(|c| c.kind == "positive") {
        let checked = teamy_bend::kernel::check_executable(
            &teamy_bend::syntax::load_executable(root.join(&case.path)).unwrap(),
        )
        .unwrap();
        let expected = match case.case.as_str() {
            "positive-triple-quote" | "positive-escaped-quote" => "'\\''\n",
            "positive-adjacent-pair" => "78\n",
            "positive-template-body" => "['\\'', '\\'', '\\'']\n",
            "positive-io" => "'\n",
            _ => panic!("unqualified positive control"),
        };
        let executable = executable_c_compiler::compile(
            &scratch,
            &teamy_bend::compiler::compile_executable_c(&checked).unwrap(),
            &[],
        );
        let js = scratch.join("program.cjs");
        std::fs::write(
            &js,
            teamy_bend::compiler::compile_executable_javascript(&checked).unwrap(),
        )
        .unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(js);
        for mut command in [Command::new(executable), node] {
            let result = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(result.status.code(), Some(0), "{}: {result:?}", case.case);
            assert_eq!(result.stdout, expected.as_bytes());
            assert!(result.stderr.is_empty());
        }
    }
    std::fs::remove_dir_all(scratch).unwrap();
}
