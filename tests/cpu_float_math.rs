// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;

#[test]
fn float_math_originals_and_bit_observations_preserve_exact_output() {
    for (name, expected) in [
        ("float_unary_math", "2.7182817 0.99999994 3 3 1.4142135\n"),
        ("float_trig", "0.84147096 0.5403023 1.5574077\n"),
        ("float_tanh", "1.1752012 1.5430807 0.7615942 1\n"),
        (
            "float-math-observations",
            concat!(
                "1076754516 2.7182817\n1065353215 0.99999994\n1077936128 3\n1077936128 3\n",
                "1068827891 1.4142135\n1062693540 0.84147096\n1057640768 0.5403023\n",
                "1070029091 1.5574077\n1066822910 1.1752012\n1069908907 1.5430807\n",
                "1061353430 0.7615942\n1065353216 1\n2147483648 -0\n4286578688 -inf\n",
                "2139095040 inf\nnan\n",
            ),
        ),
    ] {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-float-math")
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
