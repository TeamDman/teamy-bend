// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

#[test]
fn original_base_requirement_and_adapted_printer_refusals_preserve_status_and_artifacts() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-continuations");
    let destination = std::env::temp_dir().join(format!(
        "teamy-bend-constructor-refusal-{}",
        std::process::id()
    ));
    for name in [
        "ctor_construct_000",
        "ctor_construct_001",
        "ctor_continuation",
        "ctor_continuation_go",
        "ctor_default",
    ] {
        for adapter in [false, true] {
            let source = if adapter {
                root.join("applicability")
            } else {
                root.clone()
            }
            .join(format!("{name}.bend"));
            let expected = if adapter {
                std::fs::read(source.with_extension("stderr")).unwrap()
            } else {
                b"Error: a build needs import Base\n".to_vec()
            };
            for target in ["c", "javascript", "native"] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
                command
                    .arg("compile")
                    .arg(&source)
                    .args(["--executable", "--target", target, "--output"])
                    .arg(&destination);
                let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert_eq!(output.stderr, expected, "{name}: {target}");
                assert!(!destination.exists());
            }
        }
    }
}
