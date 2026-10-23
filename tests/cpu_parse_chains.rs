// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::process::Command;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;

const CASES: [(&str, &[u8]); 3] = [
    ("adjacent_application", b"True{}\n"),
    ("chain_paren_group", b"7\n"),
    ("chain_spacing_rules", b"2\n"),
];

#[test]
fn original_parse_chain_programs_preserve_raw_native_output_contracts() {
    for (name, expected) in CASES {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-parse-chains")
            .join(format!("{name}.bend"));
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "run"])
            .arg(source)
            .env("BEND_GPU", "off")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{name}");
        assert_eq!(output.stdout, expected, "{name}");
        assert!(output.stderr.is_empty(), "{name}");
    }
}

#[test]
fn successful_io_execution_does_not_admit_io_into_strict_proof_checking() {
    for (name, _) in CASES {
        let book = load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/cpu-parse-chains")
                .join(format!("{name}.bend")),
        )
        .unwrap();
        let result = check_book(&book);
        if name == "adjacent_application" {
            assert!(result.is_ok());
        } else {
            let error = result.expect_err("the proof Base has no IO family");
            assert_eq!(error.to_string(), "main: undefined name IO", "{name}");
        }
    }
}
