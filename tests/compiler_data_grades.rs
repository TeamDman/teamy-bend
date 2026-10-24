// SPDX-License-Identifier: MPL-2.0
#[path = "support/data_grades.rs"]
mod data_grades;
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use data_grades::root;
use data_grades::source;
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
            "teamy-bend-data-grades-{label}-{}",
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
    for key in ["main", "data-aggregate", "functions-aggregate"] {
        let (program, entry, expected) = if key == "main" {
            (
                std::fs::read_to_string(root().join("accept_data.bend")).unwrap(),
                "main",
                root().join("normal-forms/main.stdout"),
            )
        } else {
            (
                source("fragments", key),
                "observe",
                root().join("expected").join(format!("{key}.stdout")),
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
fn quantity_failures_keep_exact_diagnostics_and_failure_status_at_cli_boundaries() {
    let artifacts = Artifacts::new("negative");
    let destination = artifacts.0.join("unwritten");
    let mut paths = std::fs::read_dir(root().join("negative"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bend"))
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        let key = path.file_stem().unwrap().to_str().unwrap();
        let input = artifacts.0.join(format!("{key}.bend"));
        std::fs::write(&input, source("negative", key)).unwrap();
        let expected: String = facet_json::from_str(
            &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
        )
        .unwrap();
        for operation in ["check", "eval", "run", "c", "javascript", "native"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            if matches!(operation, "check" | "eval" | "run") {
                command.arg(operation).arg(&input);
            } else {
                command
                    .arg("compile")
                    .arg(&input)
                    .args(["--executable", "--target", operation, "--output"])
                    .arg(&destination);
            }
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(1), "{key}: {operation}");
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes(), "{key}: {operation}");
            assert!(!destination.exists());
        }
    }
}

#[test]
fn executable_base_requirement_and_both_import_orders_remain_distinct_refusals() {
    let artifacts = Artifacts::new("applicability");
    let destination = artifacts.0.join("unwritten");
    for (file, expected) in [
        (
            root().join("accept_data.bend"),
            "Error: a build needs import Base\n".to_owned(),
        ),
        (
            root().join("diagnostics/goal_ladder_state_027.bend"),
            facet_json::from_str::<String>(
                &std::fs::read_to_string(
                    root().join("diagnostics/goal_ladder_state_027.stderr.json"),
                )
                .unwrap(),
            )
            .unwrap(),
        ),
        (
            root().join("imported/main.bend"),
            facet_json::from_str::<String>(
                &std::fs::read_to_string(root().join("imported/main.stderr.json")).unwrap(),
            )
            .unwrap(),
        ),
        (
            root().join("module-probes/base-first.bend"),
            facet_json::from_str::<String>(
                &std::fs::read_to_string(root().join("module-probes/base-first.stderr.json"))
                    .unwrap(),
            )
            .unwrap(),
        ),
        (
            root().join("module-probes/import-first.bend"),
            facet_json::from_str::<String>(
                &std::fs::read_to_string(root().join("module-probes/import-first.stderr.json"))
                    .unwrap(),
            )
            .unwrap(),
        ),
    ] {
        for target in ["c", "javascript", "native"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
            command
                .arg("compile")
                .arg(&file)
                .args(["--executable", "--target", target, "--output"])
                .arg(&destination);
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes());
            assert!(!destination.exists());
        }
    }
}
