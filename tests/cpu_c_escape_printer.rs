// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;
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

#[derive(facet::Facet)]
struct FixtureExpectation {
    case: String,
    kind: String,
    #[facet(rename = "type")]
    value_type: String,
    expected: String,
    port_normalized: String,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-c-escape-printer")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("bend-c-escape-{label}-{}", std::process::id()));
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
fn c_escape_spelling_and_scalar_readback_match_without_changing_other_lanes() {
    let cases: Vec<FixtureExpectation> =
        facet_json::from_str(include_str!("fixtures/cpu-c-escape-printer/cases.json")).unwrap();
    assert_eq!(cases.len(), 17);
    let artifacts = Artifacts::new("values");
    for case in cases.into_iter().filter(|case| case.kind != "input-depth") {
        let path = root().join(&case.case);
        let book = load(&path).unwrap();
        let normal = if case.kind == "io" {
            check_book(&book).expect_err("IO stays excluded from strict proof checking");
            None
        } else {
            let normal = check_book(&book)
                .unwrap()
                .normalize_definition("main")
                .unwrap();
            assert_eq!(normal.to_string(), case.port_normalized);
            Some(normal)
        };
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        assert_eq!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, format!("{}\n", case.port_normalized).as_bytes());
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
        for (is_c, mut command) in [(true, Command::new(executable)), (false, node)] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            if case.kind == "non-scalar" && !is_c {
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert!(
                    String::from_utf8_lossy(&output.stderr)
                        .contains("is not a Unicode scalar value")
                );
                continue;
            }
            assert_eq!(output.status.code(), Some(0), "{}: {output:?}", case.case);
            assert_eq!(
                output.stdout,
                format!("{}\n", case.expected).as_bytes(),
                "{}",
                case.case
            );
            assert!(output.stderr.is_empty());
            if case.kind == "io" {
                continue;
            }
            let printed = std::str::from_utf8(&output.stdout).unwrap();
            let readback = artifacts.0.join("readback.bend");
            std::fs::write(
                &readback,
                format!("import Base\ndef main() -> {}: {printed}", case.value_type),
            )
            .unwrap();
            if case.kind == "non-scalar" {
                let error = load(&readback).unwrap_err();
                assert!(
                    error.message.contains("invalid Unicode code point"),
                    "{error}"
                );
                continue;
            }
            let read_book = load(&readback).unwrap();
            let read_normal = check_book(&read_book)
                .unwrap()
                .normalize_definition("main")
                .unwrap();
            assert_eq!(
                normalization_shape::shape_in_book(normal.as_ref().unwrap(), &book).unwrap(),
                normalization_shape::shape_in_book(&read_normal, &read_book).unwrap(),
                "{}",
                case.case
            );
        }
    }
}

#[test]
fn c_escape_byte_accounting_and_existing_output_guards_remain_bounded() {
    let checked =
        check_executable(&load_executable(root().join("show/char_unicode.bend")).unwrap()).unwrap();
    let source = compile_executable_c(&checked).unwrap();
    let artifacts = Artifacts::new("budgets");
    // Exercise the actual generated helpers at their existing counter boundaries;
    // no production constants are replaced. These are C helper probes, not Bend values.
    for (body, code, stdout, diagnostic) in [
        (
            "tb_steps = 123; tb_show_nodes = 456; tb_show_character(UINT64_MAX, '\"'); return tb_show_bytes == 20 && tb_steps == 123 && tb_show_nodes == 456 ? 0 : 2;",
            0,
            "\\u{ffffffffffffffff}",
            "",
        ),
        (
            "tb_show_bytes = 8388603; tb_show_character(1, '\"'); if (tb_show_bytes != 8388608) return 2; tb_show_character(65, '\"'); return 3;",
            1,
            "\\u{1}",
            "pure output byte budget exhausted",
        ),
        (
            "tb_show_bytes = 8388603; tb_show_character(127, '\"'); return 3;",
            1,
            "",
            "pure output byte budget exhausted",
        ),
        (
            "tb_show_nodes = 16383; tb_show_step(96); if (tb_show_nodes != 16384) return 2; tb_show_step(96); return 3;",
            1,
            "",
            "pure output budget exhausted",
        ),
        (
            "tb_show_step(97); return 3;",
            1,
            "",
            "pure output budget exhausted",
        ),
    ] {
        let program = format!("{source}\nint main(void) {{ (void)tb_program_main; {body} }}\n");
        let executable = executable_c_compiler::compile(&artifacts.0, &program, &["TB_NO_MAIN=1"]);
        let output =
            executable_c_compiler::bounded(&mut Command::new(executable), Duration::from_secs(20));
        assert_eq!(output.status.code(), Some(code), "{output:?}");
        assert_eq!(output.stdout, stdout.as_bytes());
        let expected = if diagnostic.is_empty() {
            String::new()
        } else {
            format!("teamy-bend executable C: {diagnostic}\n")
        };
        assert_eq!(output.stderr, expected.as_bytes());
    }
}

#[test]
fn complete_ascii_aggregate_retains_the_existing_input_guard() {
    let path = root().join("controls/ascii.bend");
    assert!(
        check_book(&load(&path).unwrap())
            .unwrap_err()
            .to_string()
            .contains("input nesting limit")
    );
    assert!(
        check_executable(&load_executable(&path).unwrap())
            .unwrap_err()
            .to_string()
            .contains("input nesting limit")
    );
}
