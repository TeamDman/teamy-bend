// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::ExecutableEntry;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

struct Artifacts(PathBuf);

impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn closed_proof_printing_and_complete_helpers_match_native_c_and_javascript() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-proof-case-trees");
    let artifacts = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-proof-case-trees-{}",
        std::process::id()
    )));
    std::fs::create_dir(&artifacts.0).unwrap();
    for folder in ["wrappers", "values", "proofs"] {
        let mut paths = std::fs::read_dir(root.join(folder))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "bend")
            })
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
            let expected = std::fs::read(path.with_extension("stdout")).unwrap();
            let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
            assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Pure);
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0
            );
            assert_eq!(stdout, expected);
            assert!(stderr.is_empty());
            let c = compile_executable_c(&checked).unwrap();
            let executable = executable_c_compiler::compile(&artifacts.0, &c, &[]);
            let js = artifacts.0.join("main.cjs");
            std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
            let mut node =
                Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
            node.arg(js);
            for mut command in [Command::new(executable), node] {
                let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert!(output.status.success(), "{name}: {output:?}");
                assert_eq!(output.stdout, expected, "{name}");
                assert!(output.stderr.is_empty(), "{name}: {output:?}");
            }
        }
    }
}

#[test]
fn original_base_requirement_and_adapted_printer_refusals_preserve_status_and_artifacts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-proof-case-trees");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-proof-case-trees-refusal-{}",
        std::process::id()
    ));
    for name in [
        "eval_case_tree_001",
        "eval_case_tree_002",
        "eval_split_apply",
    ] {
        for adapter in [false, true] {
            let source = if adapter {
                root.join("applicability")
            } else {
                root.clone()
            }
            .join(format!("{name}.bend"));
            let expected = if adapter {
                std::fs::read(source.with_extension("stderr")).unwrap()
            } else {
                b"Error: a build needs import Base\n".to_vec()
            };
            for target in ["c", "javascript", "native"] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
                command
                    .arg("compile")
                    .arg(&source)
                    .args(["--executable", "--target", target, "--output"])
                    .arg(&destination);
                let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert_eq!(output.stderr, expected, "{name}: {target}");
                assert!(!destination.exists());
            }
        }
    }
}

#[test]
fn false_proof_claims_reject_at_check_run_and_compile_boundaries() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-proof-case-trees/negative");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-proof-claim-refusal-{}",
        std::process::id()
    ));
    for name in [
        "eval_case_tree_001",
        "eval_case_tree_002",
        "eval_split_apply",
    ] {
        let path = root.join(format!("{name}.bend"));
        let expected: String = facet_json::from_str(
            &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
        )
        .unwrap();
        for operation in ["check", "run", "c", "javascript", "native"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if matches!(operation, "check" | "run") {
                command.arg(operation).arg(&path);
            } else {
                command
                    .arg("compile")
                    .arg(&path)
                    .args(["--executable", "--target", operation, "--output"])
                    .arg(&destination);
            }
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes(), "{name}: {operation}");
            assert!(!destination.exists());
        }
    }
}
