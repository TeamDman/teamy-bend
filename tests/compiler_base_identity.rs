// SPDX-License-Identifier: MPL-2.0
#[path = "support/base_identity.rs"]
mod base_identity;
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use base_identity::root;
use base_identity::source;
use facet::Facet;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_c;
use teamy_bend::compiler::compile_javascript;
use teamy_bend::kernel::check_book;
use teamy_bend::protocol::DataValue;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "teamy-bend-base-identity-{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}
#[derive(Facet)]
struct Printed {
    value: DataValue,
}

#[test]
fn strict_pure_backends_preserve_complete_constructor_results_without_base() {
    let artifacts = Artifacts::new("runtime");
    for key in [
        "main_forged_io",
        "base_prelude",
        "depth-0",
        "depth-1",
        "depth-3",
        "depth-8",
    ] {
        let (program, entry, expected) = if key.starts_with("depth-") {
            (
                source("applications", key),
                "observe",
                root().join("applications").join(format!("{key}.stdout")),
            )
        } else {
            (
                std::fs::read_to_string(root().join(format!("{key}.bend"))).unwrap(),
                "main",
                root().join(format!("{key}.stdout")),
            )
        };
        let expected = DataValue::from_term(
            &parse_term(std::fs::read_to_string(expected).unwrap().trim()).unwrap(),
        )
        .unwrap();
        let book = parse(&program).unwrap();
        let checked = check_book(&book).unwrap();
        assert_eq!(
            DataValue::from_term(&checked.evaluate_data(entry, &[]).unwrap()).unwrap(),
            expected
        );
        let exe =
            executable_c_compiler::compile(&artifacts.0, &compile_c(&book, entry).unwrap(), &[]);
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_javascript(&book, entry).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(js);
        for mut command in [Command::new(exe), node] {
            for _ in 0..2 {
                let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert!(output.status.success(), "{key}: {output:?}");
                assert!(output.stderr.is_empty(), "{key}: {output:?}");
                let printed: Printed =
                    facet_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
                assert_eq!(printed.value, expected, "{key}");
            }
        }
    }
}

#[test]
fn user_io_alias_is_pure_in_native_and_generated_executables() {
    use teamy_bend::compiler::compile_executable_c;
    use teamy_bend::compiler::compile_executable_javascript;
    use teamy_bend::kernel::ExecutableEntry;
    use teamy_bend::kernel::check_executable;
    let artifacts = Artifacts::new("executable");
    for path in [
        root().join("main_forged_io.bend"),
        root().join("adapters/real-base-fake-main.bend"),
    ] {
        let checked =
            check_executable(&teamy_bend::syntax::load_executable(&path).unwrap()).unwrap();
        assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Pure);
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
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
        let exe = executable_c_compiler::compile(
            &artifacts.0,
            &compile_executable_c(&checked).unwrap(),
            &[],
        );
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(&js);
        for mut command in [Command::new(exe), node] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stdout, expected);
            assert!(output.stderr.is_empty());
        }
    }
}
#[test]
fn invalid_calls_keep_exact_source_diagnostics_and_nonzero_boundary_status() {
    let artifacts = Artifacts::new("negative");
    for key in ["no-fake-print", "wrong-domain"] {
        let path = artifacts.0.join(format!("{key}.bend"));
        std::fs::write(&path, source("negative", key)).unwrap();
        let expected: String = facet_json::from_str(
            &std::fs::read_to_string(root().join("negative").join(format!("{key}.stderr.json")))
                .unwrap(),
        )
        .unwrap();
        for operation in ["check", "eval", "run", "c", "javascript", "native"] {
            assert_refusal(&path, operation, &expected, &artifacts.0);
        }
    }
}
#[test]
fn no_base_executable_build_refusal_is_distinct_from_pure_compilation() {
    let artifacts = Artifacts::new("no-base");
    let expected: String = facet_json::from_str(include_str!(
        "fixtures/cpu-base-identity/base_prelude.build.stderr.json"
    ))
    .unwrap();
    for target in ["c", "javascript", "native"] {
        assert_refusal(
            &root().join("base_prelude.bend"),
            target,
            &expected,
            &artifacts.0,
        );
    }
}
fn assert_refusal(path: &std::path::Path, operation: &str, expected: &str, work: &std::path::Path) {
    let destination = work.join("unwritten");
    let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
    if matches!(operation, "check" | "eval" | "run") {
        command.arg(operation).arg(path);
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

#[test]
fn undefined_names_keep_bound_context_and_imported_source_origins() {
    let artifacts = Artifacts::new("origins");
    for relative in [
        "diagnostics/nested.bend",
        "diagnostics/imported/main.bend",
        "diagnostics/type-position.bend",
        "diagnostics/erased-position.bend",
        "diagnostics/missing-order.bend",
    ] {
        let path = root().join(relative);
        let expected: String = facet_json::from_str(
            &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
        )
        .unwrap();
        for operation in ["check", "eval", "run", "c", "javascript", "native"] {
            assert_refusal(&path, operation, &expected, &artifacts.0);
        }
    }
}
