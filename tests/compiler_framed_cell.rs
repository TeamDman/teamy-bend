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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-framed-cell")
}

#[test]
fn dependent_workers_and_unchanged_io_match_native_c_and_javascript() {
    let artifacts = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-framed-cell-runtime-{}",
        std::process::id()
    )));
    std::fs::create_dir(&artifacts.0).unwrap();
    let mut paths = vec![root().join("framed_cell_dep.bend")];
    for n in [0, 1, 3, 8] {
        for kind in ["worker", "caller"] {
            paths.push(root().join("applications").join(format!("{kind}-{n}.bend")));
        }
    }
    for path in paths {
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        assert_eq!(
            checked.entry_kind().unwrap(),
            if path.parent() == Some(root().as_path()) {
                ExecutableEntry::Io
            } else {
                ExecutableEntry::Pure
            }
        );
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = checked
            .run_main(&mut stdout, &mut stderr, &|| true)
            .unwrap_err();
        assert!(error.to_string().contains("execution cancelled"));
        assert!(stdout.is_empty() && stderr.is_empty());
        for _ in 0..2 {
            stdout.clear();
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0
            );
            assert_eq!(stdout, expected);
            assert!(stderr.is_empty());
        }
        let c = compile_executable_c(&checked).unwrap();
        let executable = executable_c_compiler::compile(&artifacts.0, &c, &[]);
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(&js);
        for mut command in [Command::new(executable), node] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert!(output.status.success(), "{path:?}: {output:?}");
            assert_eq!(output.stdout, expected);
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn dependent_function_printers_refuse_exactly_without_artifacts() {
    for entry in ["dup_eq", "dup_eq.go"] {
        let path = root().join("functions").join(format!("{entry}.bend"));
        for target in ["c", "javascript", "native"] {
            assert_refusal(&path, target);
        }
    }
}

#[test]
fn wrong_dependent_and_false_reflexivity_witnesses_refuse_before_execution() {
    for name in ["wrong-dependent-witness", "false-reflexivity"] {
        let path = root().join("negative").join(format!("{name}.bend"));
        for operation in ["run", "c", "javascript", "native"] {
            assert_refusal(&path, operation);
        }
    }
}

fn assert_refusal(path: &Path, operation: &str) {
    let artifacts = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-framed-cell-refusal-{operation}-{}-{}",
        path.file_stem().unwrap().to_str().unwrap(),
        std::process::id()
    )));
    std::fs::create_dir(&artifacts.0).unwrap();
    let destination = artifacts.0.join("refused");
    let expected: String =
        facet_json::from_str(&std::fs::read_to_string(path.with_extension("stderr.json")).unwrap())
            .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
    if operation == "run" {
        command.arg("run").arg(path);
    } else {
        command
            .arg("compile")
            .arg(path)
            .args(["--executable", "--target", operation, "--output"])
            .arg(&destination);
    }
    let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, expected.as_bytes());
    assert!(!destination.exists());
}
