// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::path::PathBuf;

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-data-grades")
}

pub fn source(folder: &str, key: &str) -> String {
    let original = std::fs::read_to_string(root().join("accept_data.bend")).unwrap();
    let fragment =
        std::fs::read_to_string(root().join(folder).join(format!("{key}.bend"))).unwrap();
    if folder == "negative" {
        // The SPDX comment belongs to the fragment, not the pinned line window.
        format!("{original}\n{}", fragment.split_once('\n').unwrap().1)
    } else {
        let support = std::fs::read_to_string(root().join("support.bend")).unwrap();
        format!("{original}\n{support}\n{fragment}")
    }
}
