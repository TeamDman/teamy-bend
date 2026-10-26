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
use teamy_bend::syntax::parse_term;

#[derive(facet::Facet)]
struct LiteralValue {
    case: String,
    value_type: String,
    expression: String,
    constructor: String,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-nonscalar-literal")
}
struct Artifacts(PathBuf);
impl Artifacts {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("bend-nonscalar-{label}-{}", std::process::id()));
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
fn literals_preserve_complete_constructor_graphs_and_code_order() {
    let cases: Vec<LiteralValue> =
        facet_json::from_str(include_str!("fixtures/cpu-nonscalar-literal/values.json")).unwrap();
    assert_eq!(cases.len(), 23);
    let artifacts = Artifacts::new("graphs");
    for case in cases {
        let literal = parse_term(&case.expression).unwrap();
        let constructor = parse_term(&case.constructor).unwrap();
        assert_eq!(
            normalization_shape::shape(&literal).unwrap(),
            normalization_shape::shape(&constructor).unwrap(),
            "{}",
            case.case
        );
        let book = load(root().join(&case.case)).unwrap();
        let normal = check_book(&book)
            .unwrap()
            .normalize_definition("main")
            .unwrap();
        let explicit_path = artifacts.0.join("constructor.bend");
        std::fs::write(
            &explicit_path,
            format!(
                "import Base\ndef main() -> {}: {}\n",
                case.value_type, case.constructor
            ),
        )
        .unwrap();
        let explicit = load(&explicit_path).unwrap();
        let explicit_normal = check_book(&explicit)
            .unwrap()
            .normalize_definition("main")
            .unwrap();
        assert_eq!(
            normalization_shape::shape_in_book(&normal, &book).unwrap(),
            normalization_shape::shape_in_book(&explicit_normal, &explicit).unwrap(),
            "{}",
            case.case
        );
    }
}

#[test]
fn nonscalar_io_preserves_raw_bytes_scalar_refusal_and_cancellation() {
    let artifacts = Artifacts::new("io");
    let mut generated = None;
    for name in ["io-literal.bend", "io-constructor.bend"] {
        let path = root().join(name);
        check_book(&load(&path).unwrap()).expect_err("IO remains proof-excluded");
        let checked = check_executable(&load_executable(path).unwrap()).unwrap();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        assert_eq!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, b"before|\xed\xa0\x80\n");
        assert!(stderr.is_empty());
        stdout.clear();
        assert!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| true)
                .unwrap_err()
                .to_string()
                .contains("execution cancelled")
        );
        assert!(stdout.is_empty() && stderr.is_empty());
        let c = compile_executable_c(&checked).unwrap();
        let js = compile_executable_javascript(&checked).unwrap();
        if let Some((previous_c, previous_js)) = &generated {
            assert_eq!(&c, previous_c);
            assert_eq!(&js, previous_js);
        }
        run_io_artifacts(&artifacts.0, &c, &js);
        generated = Some((c, js));
    }
}
fn run_io_artifacts(directory: &Path, c: &str, js: &str) {
    let executable = executable_c_compiler::compile(directory, c, &[]);
    let output =
        executable_c_compiler::bounded(&mut Command::new(executable), Duration::from_secs(20));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"before|\xed\xa0\x80\n");
    assert!(output.stderr.is_empty());
    let js_path = directory.join("main.cjs");
    std::fs::write(&js_path, js).unwrap();
    let mut node =
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
    let output = executable_c_compiler::bounded(node.arg(js_path), Duration::from_secs(20));
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"before|");
    assert_eq!(
        output.stderr,
        b"bend: 55296 is not a Unicode scalar value\n"
    );
}

#[test]
fn literal_and_kernel_guards_and_first_error_refusals_remain() {
    for name in ["lower-plus-nonscalar", "lower-nine-nonscalar"] {
        let error = load(root().join(format!("{name}.bend"))).unwrap_err();
        assert_eq!(error.message, "invalid Unicode code point");
        assert_eq!((error.line, error.column), (4, 1));
    }
    for name in [
        "upper-plus-nonscalar",
        "upper-nine-nonscalar",
        "upper-overflow",
        "upper-empty",
    ] {
        let error = load(root().join(format!("{name}.bend"))).unwrap_err();
        assert_eq!(error.message, "unknown escape: \\U");
    }
    for name in ["bad-before-code", "code-before-bad"] {
        assert_eq!(
            load(root().join(format!("{name}.bend")))
                .unwrap_err()
                .message,
            "unknown escape: \\q"
        );
    }
    assert!(
        load(root().join("literal-limit.bend"))
            .unwrap_err()
            .message
            .contains("128 characters")
    );
    let book = load(root().join("input-depth.bend")).unwrap();
    assert!(
        check_book(&book)
            .unwrap_err()
            .to_string()
            .contains("input nesting")
    );
    assert!(
        check_executable(&load_executable(root().join("input-depth.bend")).unwrap())
            .unwrap_err()
            .to_string()
            .contains("input nesting")
    );
    assert!(
        load(root().join("multi-character.bend"))
            .unwrap_err()
            .message
            .contains("one character")
    );
}

#[test]
fn nonscalar_reflexivity_rejects_distinct_codes_and_patterns_keep_default_priority() {
    let artifacts = Artifacts::new("proof");
    for (code, accepted) in [(55_296, true), (57_343, false)] {
        let path = artifacts.0.join("proof.bend");
        std::fs::write(
            &path,
            format!(
                "import Base\ndef main() -> {{'\\u{{d800}}' == Chr{{{code}}} : Char}}: {{==}}\n"
            ),
        )
        .unwrap();
        assert_eq!(check_book(&load(path).unwrap()).is_ok(), accepted);
    }
    for name in ["pattern-literal.bend", "pattern-constructor.bend"] {
        let checked = check_book(&load(root().join(name)).unwrap()).unwrap();
        assert_eq!(
            checked.normalize_definition("main").unwrap().to_string(),
            "0"
        );
        assert_eq!(
            checked
                .evaluate_data("f", &[parse_term("Chr{55296}").unwrap()])
                .unwrap()
                .to_string(),
            "1"
        );
    }
}
