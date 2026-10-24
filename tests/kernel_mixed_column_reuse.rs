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
fn mixed_column_reuse_preserve_normal_forms_and_all_39_pinned_applications() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-mixed-column-reuse");
    let mut observations = 0;
    for (fixture, expected_count, expected_type) in [
        ("mixed_computed_column_003.bend", 9, "@a:Nt -> @+b:Nt -> W"),
        ("mixed_computed_column_003_id.bend", 3, "@n:Nt -> Nt"),
        (
            "mixed_computed_column_003_sel.bend",
            27,
            "@a:Nt -> @b:Nt -> @w:Nt -> W",
        ),
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
        let definition_type = checked.definition_type("main").unwrap();
        assert_eq!(definition_type.to_string(), expected_type, "{fixture}");
        assert_eq!(
            shape_in_book(definition_type, &book).unwrap(),
            shape_in_book(&parse_term(expected_type).unwrap(), &book).unwrap()
        );
        let value = checked.normalize_definition("main").unwrap();
        assert_eq!(value.to_string(), expected, "{fixture}");
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(&expected).unwrap(), &book).unwrap()
        );
        let mut count = 0;
        for line in include_str!("fixtures/cpu-mixed-column-reuse/applications.tsv").lines() {
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
    assert_eq!(observations, 39);
}
