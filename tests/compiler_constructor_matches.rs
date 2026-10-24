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
fn complete_and_projected_constructor_values_match_native_c_and_javascript() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-matches");
    let artifacts = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-constructor-matches-{}",
        std::process::id()
    )));
    std::fs::create_dir(&artifacts.0).unwrap();
    for folder in ["wrappers", "values"] {
        for name in [
            "simple_ctor_000",
            "simple_ctor_001",
            "single_ctor_type_001",
            "single_ctor_type_001_mk",
            "nested_literal",
        ] {
            let path = root.join(folder).join(format!("{name}.bend"));
            let expected = std::fs::read(path.with_extension("stdout")).unwrap();
            let checked = check_executable(&load_executable(path).unwrap()).unwrap();
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-matches");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-constructor-match-refusal-{}",
        std::process::id()
    ));
    for name in [
        "simple_ctor_000",
        "simple_ctor_001",
        "single_ctor_type_001",
        "single_ctor_type_001_mk",
        "nested_literal",
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
