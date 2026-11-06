// SPDX-License-Identifier: MPL-2.0
//! A deep pattern must be rejected or checked, never overflow the host stack.
#![cfg(target_os = "linux")]

use std::path::Path;

#[test]
fn deep_nat_pattern_does_not_abort_checker() {
    if std::env::var_os("TEAMY_BEND_DEEP_PATTERN_CHILD").is_some() {
        let no_core = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: the pointer refers to a valid rlimit in this child process.
        assert_eq!(
            unsafe { libc::setrlimit(libc::RLIMIT_CORE, &raw const no_core) },
            0
        );
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/cpu-nat-pattern-depth/controls/bool-122.bend");
                match teamy_bend::syntax::load(path) {
                    Ok(book) => match teamy_bend::kernel::check_book(&book) {
                        Ok(checked) => assert_eq!(
                            checked.normalize_definition("main").unwrap().to_string(),
                            "False{}"
                        ),
                        Err(error) => assert!(
                            error.message.contains("kernel nesting limit exhausted"),
                            "{error}"
                        ),
                    },
                    Err(error) => assert!(
                        error.message.contains(
                            "pattern compilation exceeds the resource limit of 64 levels"
                        ),
                        "{error}"
                    ),
                }
            })
            .unwrap()
            .join()
            .unwrap();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "deep_nat_pattern_does_not_abort_checker",
            "--nocapture",
        ])
        .env("TEAMY_BEND_DEEP_PATTERN_CHILD", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "child failed: {output:?}");
}
