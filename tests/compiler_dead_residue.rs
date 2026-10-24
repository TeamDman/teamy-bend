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
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-dead-residue");
    let artifacts = Artifacts(
        std::env::temp_dir().join(format!("teamy-bend-dead-residuees-{}", std::process::id())),
    );
    std::fs::create_dir(&artifacts.0).unwrap();
    for folder in ["wrappers", "values", "applicability"] {
        for name in [
            "dead_residue_elision_000",
            "dead_residue_elision_001",
            "dead_residue_elision_002",
            "dead_residue_elision_004",
        ] {
            if folder == "applicability" && name.ends_with("000") {
                continue;
            }
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-dead-residue");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-dead-residue-refusal-{}",
        std::process::id()
    ));
    for name in [
        "dead_residue_elision_000",
        "dead_residue_elision_001",
        "dead_residue_elision_002",
        "dead_residue_elision_004",
    ] {
        for adapter in [false, true] {
            if adapter && !name.ends_with("000") {
                continue;
            }
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
fn standalone_io_declaration_gate_preserves_status_outputs_and_lowering_precedence() {
    let fixture = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-dead-residue-gate-{}",
        std::process::id()
    )));
    std::fs::create_dir(&fixture.0).unwrap();
    std::fs::write(fixture.0.join("helper.bend"), "type IO is Data: Marker{}\n").unwrap();
    for source in [
        "type Tag is Data: OK{}\ndef main() -> Tag: OK{}\n",
        "import helper.bend as H\ntype Tag is Data: OK{}\ndef main() -> Tag: OK{}\n",
        "type Tag is Data: OK{}\ndef main() -> Tag: import \"missing.js\"\n",
    ] {
        let path = fixture.0.join("main.bend");
        std::fs::write(&path, source).unwrap();
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        for error in [
            compile_executable_c(&checked).unwrap_err(),
            compile_executable_javascript(&checked).unwrap_err(),
        ] {
            assert_eq!(error.to_string(), "a build needs import Base");
        }
        let output_path = fixture.0.join("output");
        for target in ["c", "javascript", "native"] {
            for existing in [false, true] {
                if existing {
                    std::fs::write(&output_path, b"retained").unwrap();
                }
                let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
                command
                    .arg("compile")
                    .arg(&path)
                    .args(["--executable", "--target", target, "--force", "--output"])
                    .arg(&output_path);
                let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert_eq!(output.stderr, b"Error: a build needs import Base\n");
                if existing {
                    assert_eq!(std::fs::read(&output_path).unwrap(), b"retained");
                    std::fs::remove_file(&output_path).unwrap();
                } else {
                    assert!(!output_path.exists());
                }
                assert!(!fixture.0.join("teamy-bend-sys.node").exists());
            }
        }
    }
}

#[test]
fn ordinary_io_declarations_enable_builds_without_changing_pure_interpretation_or_strict_emitters()
{
    let fixture = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-dead-residue-ordinary-io-{}",
        std::process::id()
    )));
    std::fs::create_dir(&fixture.0).unwrap();
    for declaration in [
        "",
        "def IO() -> Type: Type\n",
        "type IO is Data: Marker{}\n",
    ] {
        let path = fixture.0.join("main.bend");
        std::fs::write(
            &path,
            format!("{declaration}type Tag is Data: OK{{}}\ndef main() -> Tag: OK{{}}\n"),
        )
        .unwrap();
        let strict = teamy_bend::syntax::load(&path).unwrap();
        assert!(teamy_bend::compiler::compile_c(&strict, "main").is_ok());
        assert!(teamy_bend::compiler::compile_javascript(&strict, "main").is_ok());
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
        assert_eq!(stdout, b"OK{}\n");
        assert!(stderr.is_empty());
        if !declaration.is_empty() {
            let source = compile_executable_c(&checked).unwrap();
            let executable = executable_c_compiler::compile(&fixture.0, &source, &[]);
            let javascript = fixture.0.join("main.cjs");
            std::fs::write(
                &javascript,
                compile_executable_javascript(&checked).unwrap(),
            )
            .unwrap();
            let mut node =
                Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
            node.arg(javascript);
            for mut command in [Command::new(executable), node] {
                let result = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert!(result.status.success());
                assert_eq!(result.stdout, b"OK{}\n");
                assert!(result.stderr.is_empty());
            }
        }
    }
}
