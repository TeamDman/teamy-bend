// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;

#[test]
fn float_runtime_originals_and_observations_preserve_exact_output() {
    for (name, expected) in [
        ("float_builtin_agreement", "-2 -1 -1 1.5\n"),
        ("float_accumulation", "2775\n"),
        ("float_unboxed_ctor", "375\n"),
        (
            "float-runtime-observations",
            concat!(
                "3221225472 -2\n3212836864 -1\n3212836864 -1\n1069547520 1.5\n",
                "2147483648 -0\n2147483648 -0\n2147483648 -0\n0 0\n",
                "1065353217 1.0000001\n2147483648 -0\n2147483648 -0\n1067450368 1.25\n",
                "2139095040 inf\n4286578688 -inf\nnan\n",
            ),
        ),
    ] {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-float-runtime")
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
