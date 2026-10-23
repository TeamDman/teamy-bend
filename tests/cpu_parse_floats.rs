// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;
use teamy_bend::syntax::parse_term;

#[test]
fn original_float_programs_and_bit_controls_preserve_exact_output() {
    for (name, expected) in [
        ("chain_suffix_f32", "7.5\n"),
        ("float_exponent", "0.0025 1000 150\n"),
        ("float_pattern", "1\n"),
        (
            "float-bits-and-patterns",
            "992204554\n1065353216\n1065353218\n1065353216\n10\n2147483648\n20\n1071644672\n7\n99\n1\n4294967295\n",
        ),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-parse-floats")
            .join(format!("{name}.bend"));
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "run"])
            .arg(path)
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{name}: {:?}", output.stderr);
        assert_eq!(output.stdout, expected.as_bytes(), "{name}");
        assert!(output.stderr.is_empty(), "{name}");
    }
}

#[test]
fn u32_rendering_requires_complete_words_and_preserves_other_terms() {
    for value in [0, 1, 17, u32::MAX] {
        let literal = value.to_string();
        assert_eq!(parse_term(&literal).unwrap().to_string(), literal);
    }
    for width in [0, 31, 33] {
        let word = format!(
            "{}WNil{{}}{}",
            "WCon{False{}, ".repeat(width),
            "}".repeat(width)
        );
        let source = format!("U32{{{word}}}");
        assert_eq!(parse_term(&source).unwrap().to_string(), source);
    }
    for source in ["U32{tail}", "U32{f(17)}", "{17 : U32}"] {
        assert_eq!(parse_term(source).unwrap().to_string(), source);
    }
    for source in ["3.4028236e38", "1.0e400"] {
        assert!(parse_term(source).unwrap_err().message.contains("finite"));
    }
}
