// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-unfilled-law")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-unfilled-law-{label}-{}",
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
fn unfilled_laws_keep_exact_diagnostics_and_compilation_refusals() {
    let cases: Vec<_> = facet_json::from_str::<Vec<String>>(include_str!(
        "fixtures/cpu-unfilled-law/negative-cases.json"
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
fn filled_erased_recursive_imported_and_sealed_definitions_remain_usable() {
    let artifacts = Artifacts::new("positive");
    for name in [
        "filled",
        "dead-before-fill",
        "recursion",
        "imported/main",
        "native-contract",
    ] {
        let path = root().join("positive").join(format!("{name}.bend"));
        let expected = if name == "native-contract" {
            "3\n"
        } else {
            "7\n"
        };
        if name == "native-contract" {
            // Strict proof loading excludes IO; executable checking is separate.
            check_book(&load(&path).unwrap()).unwrap_err();
        } else {
            let checked = check_book(&load(&path).unwrap()).unwrap();
            assert_eq!(
                checked.normalize_definition("main").unwrap().to_string(),
                "7"
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
            assert_eq!(stdout, expected.as_bytes());
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
            assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
            assert_eq!(output.stdout, expected.as_bytes(), "{name}");
            assert!(output.stderr.is_empty(), "{name}: {output:?}");
        }
    }
}
