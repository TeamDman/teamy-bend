// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

#[test]
fn mixed_computed_columns_preserve_normal_forms_and_all_63_pinned_applications() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-mixed-computed-columns");
    let mut observations = 0;
    for (fixture, expected_count) in [
        ("mixed_computed_column_000.bend", 12),
        ("mixed_computed_column_000_par.bend", 6),
        ("mixed_computed_column_000_sel.bend", 9),
        ("mixed_computed_column_000_walk.bend", 9),
        ("mixed_computed_column_001.bend", 8),
        ("mixed_computed_column_001_flip.bend", 2),
        ("mixed_computed_column_001_sel.bend", 8),
        ("mixed_computed_column_002.bend", 9),
    ] {
        let path = root.join(fixture);
        let source = std::fs::read_to_string(&path).unwrap();
        let expected = source
            .lines()
            .filter_map(|line| line.strip_prefix("#|"))
            .collect::<Vec<_>>()
            .join("\n");
        let book = load(path).unwrap();
        let checked = check_book(&book).unwrap();
        let value = checked.normalize_definition("main").unwrap();
        assert_eq!(value.to_string(), expected, "{fixture}");
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(&expected).unwrap(), &book).unwrap()
        );
        let mut count = 0;
        for line in include_str!("fixtures/cpu-mixed-computed-columns/applications.tsv").lines() {
            let mut fields = line.splitn(3, '\t');
            let file = fields.next().unwrap();
            let arguments = fields.next().unwrap();
            let expected = fields.next().unwrap();
            if file != fixture {
                continue;
            }
            let arguments = arguments
                .split(';')
                .map(|arg| parse_term(arg).unwrap())
                .collect::<Vec<_>>();
            let value = checked.evaluate("main", &arguments).unwrap();
            assert_eq!(value.to_string(), expected, "{fixture}: {arguments:?}");
            assert_eq!(
                shape_in_book(&value, &book).unwrap(),
                shape_in_book(&parse_term(expected).unwrap(), &book).unwrap()
            );
            count += 1;
        }
        assert_eq!(count, expected_count);
        observations += count;
    }
    assert_eq!(observations, 63);
}
