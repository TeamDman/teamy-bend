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
    expected: String,
    port_normalized: String,
    compiled_c: String,
    compiled_js: String,
    io: bool,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-uppercase-unicode")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-uppercase-unicode-{label}-{}",
            std::process::id()
        ));
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
fn uppercase_unicode_preserves_values_normalization_and_execution() {
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "fixtures/cpu-uppercase-unicode/positive-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 16);
    let artifacts = Artifacts::new("positive");
    for case in cases {
        let path = root().join(&case.case);
        if case.io {
            check_book(&load(&path).unwrap()).expect_err("IO remains excluded from strict proofs");
        } else {
            let book = load(&path).unwrap();
            let normal = check_book(&book)
                .unwrap()
                .normalize_definition("main")
                .unwrap();
            assert_eq!(normal.to_string(), case.port_normalized);
            let expected = teamy_bend::syntax::parse_term(&case.expected).unwrap();
            assert_eq!(
                normalization_shape::shape_in_book(&normal, &book).unwrap(),
                normalization_shape::shape_in_book(&expected, &book).unwrap()
            );
        }
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        // Pure run_main is normalization; the explicit IO control exercises native execution.
        // Generated C/JS remain separate executable evidence.
        let native_expected = format!(
            "{}\n",
            if case.io {
                &case.expected
            } else {
                &case.port_normalized
            }
        );
        for _ in 0..2 {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0
            );
            assert_eq!(stdout, native_expected.as_bytes(), "{}", case.case);
            assert!(stderr.is_empty());
        }
        // Cover all scalar boundaries above; compile representative aggregate,
        // string, IO and unchanged original programs once per target.
        if !matches!(
            case.case.as_str(),
            "controls/aggregate.bend"
                | "controls/string.bend"
                | "controls/io.bend"
                | "show/string_escape.bend"
        ) {
            continue;
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
        for (mut command, rendered) in [
            (Command::new(executable), &case.compiled_c),
            (node, &case.compiled_js),
        ] {
            let expected = format!("{rendered}\n");
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(0), "{}: {output:?}", case.case);
            assert_eq!(output.stdout, expected.as_bytes(), "{}", case.case);
            assert!(output.stderr.is_empty(), "{}: {output:?}", case.case);
        }
    }
}

#[test]
fn uppercase_unicode_cancellation_remains_at_the_existing_boundaries() {
    let checked =
        check_executable(&load_executable(root().join("controls/ascii.bend")).unwrap()).unwrap();
    for cancel_at in [1, 2] {
        let observations = std::cell::Cell::new(0);
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let error = checked
            .run_main(&mut stdout, &mut stderr, &|| {
                observations.set(observations.get() + 1);
                observations.get() == cancel_at
            })
            .unwrap_err();
        assert!(error.to_string().contains("execution cancelled"));
        assert_eq!(observations.get(), cancel_at);
        assert!(stdout.is_empty() && stderr.is_empty());
    }
    let checked =
        check_executable(&load_executable(root().join("controls/io.bend")).unwrap()).unwrap();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    assert!(
        checked
            .run_main(&mut stdout, &mut stderr, &|| true)
            .unwrap_err()
            .to_string()
            .contains("execution cancelled")
    );
    assert!(stdout.is_empty() && stderr.is_empty());
}
