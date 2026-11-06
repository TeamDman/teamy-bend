// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;

#[test]
fn float_operation_text_and_shared_read_cases_preserve_exact_output() {
    for (name, expected) in [
        (
            "float_ops",
            "1.5 -1.25 1024 0.7853982 1.5707964 0 0.7853982 3 7\n",
        ),
        (
            "float_printing",
            "2 100.25 0.33333334 0.0009765625 123456\n",
        ),
        ("float_text", "2.5 0.1 none none 1.25\n"),
        (
            "float-text-observations",
            concat!(
                "1069547520 1.5\n3214934016 -1.25\n1149239296 1024\n1061752795 0.7853982\n",
                "1070141403 1.5707964\n0 0\n1061752795 0.7853982\n1088421888 7\n3\n",
                "1051372203 0.33333334\n981467136 0.0009765625\n1036831949\n2147483648\n",
                "1056964608\nnone\nnone\nnone\n1067450368\n",
            ),
        ),
    ] {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-float-text")
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

#[cfg(all(target_os = "linux", target_env = "gnu", target_arch = "x86_64"))]
#[test]
fn native_read_preserves_glibc_nan_payload_overflow_bits() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-float-text/nan-payload-overflow.bend");
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
        .arg("run")
        .arg(source)
        .env("BEND_GPU", "off")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output.stderr);
    assert_eq!(
        output.stdout,
        b"2147483647\n2147483647\n4294967295\n2143289344\n"
    );
    assert!(output.stderr.is_empty());
}
