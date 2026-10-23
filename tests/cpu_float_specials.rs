// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;

const OBSERVATIONS: &str = "1077936128 3\n1077936128 3\n1075838976 2.5\n2\n1073741660 1.9999804\n1240736612 1999980.5\n5\n2139095040 inf\n4286578688 -inf\n2147483648 -0\nnan\n0 1 0 0 0 0\n0 1 0 0 0 0\n0 1 0 0 1 1\n0 1 1 1 0 0\n1 0 0 1 0 1\n0 0 0 0 0 0 1 4294967040 0\n";

#[test]
fn float_comparisons_nonfinite_classes_and_text_preserve_exact_output() {
    for (name, expected) in [
        ("float_text_roundtrip", "233\n"),
        ("float_compare", "1999985\n"),
        ("float_specials", "12\n"),
        ("float-special-observations", OBSERVATIONS),
        ("float-specials-combined-depth", OBSERVATIONS),
    ] {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-float-specials")
            .join(format!("{name}.bend"));
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "run"])
            .arg(source)
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{name}: {:?}", output.stderr);
        assert_eq!(output.stdout, expected.as_bytes(), "{name}");
        assert!(output.stderr.is_empty(), "{name}");
    }
}
