// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;

#[test]
fn finite_float_arithmetic_and_conversion_bits_preserve_exact_output() {
    for (name, expected) in [
        ("base-float-roundtrip", "144\n"),
        ("compile-float-roundtrip", "45\n"),
        ("float_arithmetic", "8\n"),
        (
            "float-arithmetic-observations",
            "1082130432 4\n1096810496 14\n1089470464 7.5\n1110704128 45\n1050253722 0.3\n1050253722 0.3\n1266679808 16777216\n1266679808 16777216\n1266679810 16777220\n1081711002 3.9\n3\n1206984704 123456\n1217559552 300000\n",
        ),
    ] {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-float-arithmetic")
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
