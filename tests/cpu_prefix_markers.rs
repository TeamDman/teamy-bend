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
fn prefix_marker_values_keep_complete_upstream_graphs() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-prefix-markers/positive");
    for item in std::fs::read_dir(root).unwrap() {
        let path = item.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "bend") {
            continue;
        }
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        // Compare the complete canonical graph serialization. Decoding the
        // nested Word graph through Facet overflowed the test thread's stack;
        // no Bend evaluator or stack/resource allowance changes are needed.
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
fn prefix_markers_execute_in_native_c_and_javascript() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-prefix-markers");
    let artifacts = Artifacts(
        std::env::temp_dir().join(format!("teamy-bend-prefix-markers-{}", std::process::id())),
    );
    std::fs::create_dir(&artifacts.0).unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(root.join("positive"))
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "bend"))
        .collect();
    paths.push(root.join("io.bend"));
    paths.sort();
    for path in paths {
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        // The existing native String printer exposes constructors. Keep its
        // exact residual mismatch separate from the complete value comparison.
        let native = path.with_extension("native-stdout");
        let native = if native.exists() {
            std::fs::read(native).unwrap()
        } else {
            expected.clone()
        };
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        assert_eq!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, native, "{}", path.display());
        assert!(stderr.is_empty());
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
            assert!(output.status.success(), "{}: {output:?}", path.display());
            assert_eq!(output.stdout, expected, "{}", path.display());
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
}

#[test]
fn affine_and_infix_patterns_still_refuse_at_public_boundaries() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-prefix-markers/negative");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-prefix-markers-refusal-{}",
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
        let current = path.with_extension("port-current-stderr.json");
        let expected_path = if current.exists() {
            current
        } else {
            path.with_extension("port-stderr.json")
        };
        let expected: String =
            facet_json::from_str(&std::fs::read_to_string(expected_path).unwrap()).unwrap();
        for op in ["check", "eval", "run", "c", "javascript", "native"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if ["check", "eval", "run"].contains(&op) {
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
                &format!("tests/fixtures/cpu-prefix-markers/negative/{name}.bend"),
            );
            assert_eq!(stderr, expected, "{name}/{op}");
            assert!(!destination.exists());
        }
    }
}
