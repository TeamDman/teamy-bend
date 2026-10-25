// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::Declaration;
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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-alias-fallback")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-alias-fallback-{label}-{}",
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
fn known_alias_members_keep_precedence_over_raw_declarations() {
    let book = load(root().join("known/main.bend")).unwrap();
    let filled: Vec<_> = book
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Def(d) if d.body.is_some() => Some(d.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        filled.iter().filter(|&&name| name == "defs.value").count(),
        1
    );
    assert_eq!(filled.iter().filter(|&&name| name == "D.value").count(), 1);
    let checked = check_book(&book).unwrap();
    for (name, value) in [("defs.value", "3"), ("D.value", "4"), ("main", "3")] {
        assert_eq!(
            checked.normalize_definition(name).unwrap().to_string(),
            value
        );
    }
}

#[test]
fn missing_alias_members_fall_back_without_changing_known_member_precedence() {
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "fixtures/cpu-alias-fallback/positive-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 10);
    let artifacts = Artifacts::new("positive");
    for case in cases {
        let path = root().join(&case.case);
        let expected = format!("{}\n", case.expected);
        if case.io {
            check_book(&load(&path).unwrap()).expect_err("IO remains excluded from strict proofs");
        } else {
            assert_eq!(
                check_book(&load(&path).unwrap())
                    .unwrap()
                    .normalize_definition("main")
                    .unwrap()
                    .to_string(),
                case.expected
            );
        }
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        for _ in 0..2 {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0
            );
            assert_eq!(stdout, expected.as_bytes(), "{}", case.case);
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
fn alias_fallback_does_not_hide_genuine_conflicts_or_undefined_names() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-alias-fallback/negative-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 7);
    let artifacts = Artifacts::new("negative");
    for case in cases {
        let path = root().join(&case);
        let expected: String = facet_json::from_str(
            &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
        )
        .unwrap();
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

#[test]
fn unknown_template_still_fails_before_checking() {
    let path = root().join("negative/template_missing.bend");
    let strict = load(&path).unwrap_err();
    let executable = load_executable(&path).unwrap_err();
    assert_eq!(strict.to_string(), executable.to_string());
    assert!(
        strict
            .message
            .contains("~ arguments require a previously declared template")
    );
}
