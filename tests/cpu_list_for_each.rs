// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape_in_book;
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

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn pure_list_templates_keep_complete_upstream_graphs() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-list-for-each/positive");
    for item in std::fs::read_dir(root).unwrap() {
        let path = item.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "bend")
            || !path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("pure-")
        {
            continue;
        }
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        // Compare every field of the canonical value graph captured from
        // the pinned interpreter, separately from display formatting.
        let expected = std::fs::read_to_string(path.with_extension("json")).unwrap();
        for _ in 0..2 {
            let value = checked.evaluate_data("main", &[]).unwrap();
            assert_eq!(
                facet_json::to_string(&shape_in_book(&value, &book).unwrap()).unwrap(),
                expected.strip_suffix('\n').unwrap(),
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn list_fold_and_for_each_execute_in_native_c_and_javascript() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-list-for-each");
    let artifacts = Artifacts(
        std::env::temp_dir().join(format!("teamy-bend-list-for-each-{}", std::process::id())),
    );
    std::fs::create_dir(&artifacts.0).unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(root.join("positive"))
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "bend"))
        .collect();
    paths.push(root.join("original/list_fold.bend"));
    paths.sort();
    for path in paths {
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        let expected_stderr = std::fs::read(path.with_extension("stderr")).unwrap();
        let expected_code: u32 = std::fs::read_to_string(path.with_extension("exit"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        // Reuse the checked book after completion/halt and verify invocation
        // isolation through identical output and status.
        for _ in 0..2 {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                expected_code
            );
            assert_eq!(stdout, expected, "{}", path.display());
            assert_eq!(stderr, expected_stderr, "{}", path.display());
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
            assert_eq!(
                output.status.code(),
                Some(i32::try_from(expected_code).unwrap()),
                "{}: {output:?}",
                path.display()
            );
            assert_eq!(output.stdout, expected, "{}", path.display());
            assert_eq!(output.stderr, expected_stderr, "{output:?}");
        }
    }
}

#[test]
fn invalid_callbacks_and_incomplete_proofs_refuse_executable_boundaries() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-list-for-each/negative");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-list-for-each-refusal-{}",
        std::process::id()
    ));
    assert!(!destination.exists());
    for item in std::fs::read_dir(&root).unwrap() {
        let path = item.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "bend") {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap();
        // Keep historical receipts immutable; a renewed snapshot still compares
        // every byte, including the host formatter's current implementation line.
        let current = path.with_extension("port-executable-current-stderr.json");
        let expected_path = if current.exists() {
            current
        } else {
            path.with_extension("port-executable-stderr.json")
        };
        let expected: String =
            facet_json::from_str(&std::fs::read_to_string(expected_path).unwrap()).unwrap();
        for op in ["run", "c", "javascript", "native"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if op == "run" {
                command.arg(op).arg(&path);
            } else {
                command
                    .arg("compile")
                    .arg(&path)
                    .args(["--executable", "--target", op, "--output"])
                    .arg(&destination);
            }
            let result = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(result.status.code(), Some(1), "{name}/{op}");
            assert!(result.stdout.is_empty());
            let stderr = String::from_utf8(result.stderr).unwrap().replace(
                path.to_str().unwrap(),
                &format!("tests/fixtures/cpu-list-for-each/negative/{name}.bend"),
            );
            assert_eq!(stderr, expected, "{name}/{op}");
            assert!(!destination.exists());
        }
    }
}

#[test]
fn executable_list_helper_does_not_become_a_strict_proof_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-list-for-each");
    for name in [
        "original/list_fold",
        "positive/direct",
        "positive/empty",
        "positive/halt",
        "positive/nested",
        "positive/sequence",
    ] {
        let error = load(root.join(format!("{name}.bend"))).unwrap_err();
        assert!(
            error
                .message
                .contains("~ arguments require a previously declared template head"),
            "{name}: {error}"
        );
    }
}
