// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::path::PathBuf;
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-base-identity")
}
pub fn source(folder: &str, key: &str) -> String {
    let original = if key == "no-fake-print" {
        "main_forged_io"
    } else {
        "base_prelude"
    };
    let original = std::fs::read_to_string(root().join(format!("{original}.bend"))).unwrap();
    let fragment =
        std::fs::read_to_string(root().join(folder).join(format!("{key}.bend"))).unwrap();
    format!("{original}\n{fragment}")
}
