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
fn ragged_rows_preserve_normal_forms_and_all_81_pinned_applications() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-ragged-rows");
    let mut observations = 0;
    for index in 0..5 {
        let fixture = format!("ragged_rows_{index:03}.bend");
        let path = root.join(&fixture);
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
        for line in include_str!("fixtures/cpu-ragged-rows/applications.tsv").lines() {
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
        assert_eq!(count, if matches!(index, 2 | 3) { 27 } else { 9 });
        observations += count;
    }
    assert_eq!(observations, 81);
}
