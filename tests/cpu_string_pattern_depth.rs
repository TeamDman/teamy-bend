// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn single_character_and_empty_literal_patterns_execute_hits_and_misses() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-string-pattern-depth");
    let artifact_path = std::env::temp_dir().join(format!(
        "teamy-bend-string-pattern-depth-{}",
        std::process::id()
    ));
    std::fs::create_dir(&artifact_path).unwrap();
    let artifacts = Artifacts(artifact_path);
    let mut paths: Vec<_> = std::fs::read_dir(root.join("positive"))
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "bend"))
        .collect();
    paths.sort();
    for path in paths {
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        let expected_stderr = Vec::<u8>::new();
        let expected_code = 0_u32;
        // Reuse the checked book after completion and verify invocation
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
fn recursive_and_wide_patterns_keep_their_compilation_guard() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-string-pattern-depth/negative");
    for name in ["nat65.bend", "columns65.bend"] {
        let error = teamy_bend::syntax::load(root.join(name)).unwrap_err();
        assert!(
            error
                .message
                .contains("pattern compilation exceeds the resource limit of 64 levels"),
            "{name}: {error}"
        );
    }
}

#[test]
fn string_patterns_do_not_accept_a_nat_scrutinee() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-string-pattern-depth/negative/wrong-type.bend");
    let strict = teamy_bend::syntax::load(&path).expect("pattern syntax is valid");
    let _strict_error =
        teamy_bend::kernel::check_book(&strict).expect_err("String pattern cannot consume a Nat");
    let executable = load_executable(path).expect("executable pattern syntax is valid");
    let _executable_error =
        check_executable(&executable).expect_err("runtime checking rejects the same invalid match");
}
