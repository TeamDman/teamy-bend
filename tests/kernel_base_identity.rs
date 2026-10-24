// SPDX-License-Identifier: MPL-2.0
#[path = "support/base_identity.rs"]
mod base_identity;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;
use base_identity::root;
use base_identity::source;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

fn graph(path: &std::path::Path) -> Shape {
    facet_json::from_str(&std::fs::read_to_string(path.with_extension("json")).unwrap()).unwrap()
}
#[test]
fn user_io_family_and_prelude_free_definitions_keep_full_normal_forms() {
    for (name, entries) in [
        ("main_forged_io", &["main", "IO"][..]),
        ("base_prelude", &["main", "depth"][..]),
    ] {
        let book = load(root().join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        for entry in entries {
            let expected = root().join("normal-forms").join(format!("{name}--{entry}"));
            let text = std::fs::read_to_string(expected.with_extension("stdout")).unwrap();
            for _ in 0..2 {
                let value = checked.normalize_definition(entry).unwrap();
                assert_eq!(format!("{value}\n"), text);
                assert_eq!(shape_in_book(&value, &book).unwrap(), graph(&expected));
                assert_eq!(
                    shape_in_book(&parse_term(text.trim()).unwrap(), &book).unwrap(),
                    graph(&expected)
                );
            }
        }
    }
}
#[test]
fn four_depth_calls_and_typed_observations_preserve_every_constructor_child() {
    let book = load(root().join("base_prelude.bend")).unwrap();
    let checked = check_book(&book).unwrap();
    for n in [0, 1, 3, 8] {
        let key = format!("depth-{n}");
        let path = root().join("applications").join(&key);
        let text = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
        let arg = parse_term(text.trim()).unwrap();
        let expected = graph(&path);
        for _ in 0..2 {
            for value in [
                checked
                    .evaluate("depth", std::slice::from_ref(&arg))
                    .unwrap(),
                checked
                    .evaluate_data("depth", std::slice::from_ref(&arg))
                    .unwrap(),
            ] {
                assert_eq!(format!("{value}\n"), text);
                assert_eq!(shape_in_book(&value, &book).unwrap(), expected);
            }
        }
        let assembled = parse(&source("applications", &key)).unwrap();
        let value = check_book(&assembled)
            .unwrap()
            .evaluate("observe", &[])
            .unwrap();
        assert_eq!(shape_in_book(&value, &assembled).unwrap(), expected);
    }
}
#[test]
fn actual_base_does_not_turn_imported_user_io_into_effects() {
    let path = root().join("adapters/real-base-fake-main.bend");
    let book = load(&path).unwrap();
    let checked = check_book(&book).unwrap();
    let value = checked.evaluate_data("main", &[]).unwrap();
    assert_eq!(shape_in_book(&value, &book).unwrap(), graph(&path));
}
