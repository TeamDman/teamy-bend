// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape_in_book;
use std::path::Path;
use std::process::Command;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

const CASES: [(&str, &str, &str); 2] = [
    (
        "array_literal",
        "([3, 3, 3, 3], [True{}, True{}])",
        "Tuple{ANode{ANode{ALeaf{3}, ALeaf{3}}, ANode{ALeaf{3}, ALeaf{3}}}, ANode{ALeaf{True{}}, ALeaf{True{}}}}",
    ),
    (
        "array_write_stmt",
        "([7, 1, 0, 0, 0, 42, 0, 0], 42)",
        "Tuple{ANode{ANode{ANode{ALeaf{7}, ALeaf{1}}, ANode{ALeaf{0}, ALeaf{0}}}, ANode{ANode{ALeaf{0}, ALeaf{42}}, ANode{ALeaf{0}, ALeaf{0}}}}, 42}",
    ),
];

#[test]
fn array_literal_pure_normalization_preserves_structure_and_raw_cli_text() {
    let (name, expected, core) = CASES[0];
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-array-syntax")
        .join(format!("{name}.bend"));
    let book = load(&source).unwrap();
    let checked = check_book(&book).unwrap();
    let value = checked.normalize_definition("main").unwrap();
    // Array display shares bracket notation with List, but the actual core
    // must retain its ALeaf/ANode tree, not a parsed List-shaped pretty string.
    assert_eq!(
        shape_in_book(&value, &book).unwrap(),
        shape_in_book(&parse_term(core).unwrap(), &book).unwrap()
    );
    assert_eq!(value.to_string(), expected);
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
        .args(["--output-format", "json", "run"])
        .arg(source)
        .env("BEND_GPU", "off")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn checked_data_runtime_preserves_array_construction_and_statement_writes() {
    for (name, expected, core) in CASES {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-array-syntax")
            .join(format!("{name}.bend"));
        let book = load(source).unwrap();
        let checked = check_book(&book).unwrap();
        // This explicitly qualifies evaluate_data. It does not stand in for
        // the pure-main CLI normalizer or waive its separate resource failure.
        let value = checked.evaluate_data("main", &[]).unwrap();
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(core).unwrap(), &book).unwrap(),
            "{name}"
        );
        assert_eq!(value.to_string(), expected, "{name}");
    }
}
