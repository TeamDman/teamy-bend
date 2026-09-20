// SPDX-License-Identifier: MPL-2.0
//! Pure Image/Event contracts retain ordinary checking and bounded evaluation.

use std::fmt::Write;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const BASE: &str = include_str!("../src/syntax/base.bend");
const HELPERS: &str = r"
def small_quad() -> Image: Qua{Pix{1}, Pix{2}, Pix{3}, Pix{4}}
def nested_image() -> Image: Qua{Pix{0}, small_quad(), Pix{5}, Pix{6}}

def colors(image: Image) -> List<&2, Nat>:
  match image:
    case Pix{color}: [U32.to_nat(color)]
    case Qua{tl, tr, bl, br}:
      a b c d = colors(tl) colors(tr) colors(bl) colors(br)
      List.append(&2, Nat, a, List.append(&2, Nat, b, List.append(&2, Nat, c, d)))

def event_fields(event: Event) -> List<&2, U32>:
  match event:
    case Key{code, down}: [0, code, Bool.to_u32(down)]
    case Mouse{x, y, button, down}: [1, x, y, button, Bool.to_u32(down)]
    case Move{x, y}: [2, x, y]
    case Close{}: [3]

def share_image(+image: Image) -> Image & Image: (image, image)
def share_event(+event: Event) -> Event & Event: (event, event)
";

fn checked(extra: &str) -> (Book, CheckedBook) {
    let book =
        parse(&format!("{BASE}\n{HELPERS}\n{extra}\n")).expect("finite image/event fixtures parse");
    let checked = check_book(&book).expect("image/event source uses ordinary strict checking");
    (book, checked)
}

fn expect_value(checked: &CheckedBook, entry: &str, expected: &str) {
    let actual = checked.evaluate_data(entry, &[]).unwrap();
    let expected = parse_term(expected).unwrap();
    assert_eq!(actual.to_string(), expected.to_string(), "{entry}");
}

#[test]
fn image_and_event_layouts_have_reusable_kinds_and_ordinary_field_quantities() {
    let (book, _) = checked("");
    for (name, constructors) in [
        (
            "Image",
            vec![
                ("Pix", vec![("color", "U32")]),
                (
                    "Qua",
                    vec![
                        ("tl", "Image"),
                        ("tr", "Image"),
                        ("bl", "Image"),
                        ("br", "Image"),
                    ],
                ),
            ],
        ),
        (
            "Event",
            vec![
                ("Key", vec![("code", "U32"), ("down", "Bool")]),
                (
                    "Mouse",
                    vec![
                        ("x", "U32"),
                        ("y", "U32"),
                        ("button", "U32"),
                        ("down", "Bool"),
                    ],
                ),
                ("Move", vec![("x", "U32"), ("y", "U32")]),
                ("Close", vec![]),
            ],
        ),
    ] {
        let datatype = book
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Adt(datatype) if datatype.name == name => Some(datatype),
                _ => None,
            })
            .unwrap();
        assert!(datatype.parameters.is_empty());
        assert_eq!(datatype.kind.to_string(), "Data");
        assert_eq!(datatype.constructors.len(), constructors.len());
        for (actual, (expected_name, expected_fields)) in
            datatype.constructors.iter().zip(constructors)
        {
            assert_eq!(actual.name, expected_name);
            assert_eq!(actual.fields.len(), expected_fields.len());
            for (field, (expected_name, expected_type)) in actual.fields.iter().zip(expected_fields)
            {
                assert_eq!(field.name, expected_name);
                assert_eq!(field.ty.to_string(), expected_type);
                assert_eq!(field.quant, Quant::Lone);
            }
        }
    }
    let free = book
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.name == "Image.free" => Some(definition),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        free.parameters
            .iter()
            .map(|parameter| parameter.quant)
            .collect::<Vec<_>>(),
        [Quant::Many, Quant::Lone]
    );
    assert!(free.body.is_some() && !free.foreign && !free.unsafe_);
}

#[test]
fn finite_image_disposal_has_checked_equalities_at_zero_and_nested_depths() {
    let cases = [
        "Image.sink(Pix{4294967295})",
        "Image.sink(nested_image())",
        "Image.drop.join(Unit{}, Unit{}, Unit{}, Unit{})",
        "Image.free(0n, nested_image())",
        "Image.free(1n, nested_image())",
        "Image.free(2n, nested_image())",
        "Image.free(8n, nested_image())",
        "Image.free(3n, Pix{0})",
        "Image.drop(Pix{17})",
        "Image.drop(nested_image())",
    ];
    let mut source = String::new();
    for (index, expression) in cases.iter().enumerate() {
        writeln!(source, "def value{index}() -> Unit: {expression}").unwrap();
        writeln!(
            source,
            "def proof{index}() -> {{value{index}() == Unit{{}} : Unit}}: {{==}}"
        )
        .unwrap();
    }
    let (_, checked) = checked(&source);
    for index in 0..cases.len() {
        expect_value(&checked, &format!("value{index}"), "Unit{}");
    }
}

#[test]
fn nested_images_retain_quadrant_order_and_explicit_sharing() {
    let (_, checked) = checked(
        r"
def pixel_colors() -> List<&2, Nat>: colors(Pix{7})
def quad_colors() -> List<&2, Nat>: colors(small_quad())
def nested_colors() -> List<&2, Nat>: colors(nested_image())
def shared_image() -> Image & Image: share_image(nested_image())
def pixel_proof() -> {pixel_colors() == [7n] : List<&2, Nat>}: {==}
def quad_proof() -> {quad_colors() == [1n, 2n, 3n, 4n] : List<&2, Nat>}: {==}
def nested_proof() -> {nested_colors() == [0n, 1n, 2n, 3n, 4n, 5n, 6n] : List<&2, Nat>}: {==}
",
    );
    for (entry, expected) in [
        ("pixel_colors", "[7n]"),
        ("quad_colors", "[1n,2n,3n,4n]"),
        ("nested_colors", "[0n,1n,2n,3n,4n,5n,6n]"),
        (
            "shared_image",
            "(Qua{Pix{0},Qua{Pix{1},Pix{2},Pix{3},Pix{4}},Pix{5},Pix{6}},Qua{Pix{0},Qua{Pix{1},Pix{2},Pix{3},Pix{4}},Pix{5},Pix{6}})",
        ),
    ] {
        expect_value(&checked, entry, expected);
    }
}

#[test]
fn all_event_forms_preserve_fields_polarity_and_coordinate_boundaries() {
    let cases = [
        ("Key{97, True{}}", "[0,97,1]"),
        ("Key{0, False{}}", "[0,0,0]"),
        ("Mouse{2, 3, 4, True{}}", "[1,2,3,4,1]"),
        ("Mouse{0, 4294967295, 0, False{}}", "[1,0,4294967295,0,0]"),
        ("Move{7, 11}", "[2,7,11]"),
        ("Move{4294967295, 0}", "[2,4294967295,0]"),
        ("Close{}", "[3]"),
    ];
    let mut source = String::new();
    for (index, (event, expected)) in cases.iter().enumerate() {
        writeln!(
            source,
            "def value{index}() -> List<&2, U32>: event_fields({event})"
        )
        .unwrap();
        writeln!(
            source,
            "def proof{index}() -> {{value{index}() == {expected} : List<&2, U32>}}: {{==}}"
        )
        .unwrap();
    }
    source.push_str("def copied_event() -> Event & Event: share_event(Mouse{7, 9, 2, True{}})\n");
    let (_, checked) = checked(&source);
    for (index, (_, expected)) in cases.iter().enumerate() {
        expect_value(&checked, &format!("value{index}"), expected);
    }
    expect_value(
        &checked,
        "copied_event",
        "(Mouse{7,9,2,True{}},Mouse{7,9,2,True{}})",
    );
}

#[test]
fn image_and_event_helpers_cannot_prove_false_values_or_duplicate_affine_binders() {
    for (source, diagnostic) in [
        (
            "def invalid() -> {Pix{0} == Pix{1} : Image}: {==}",
            "reflexivity",
        ),
        (
            "def invalid() -> {Key{1, True{}} == Key{1, False{}} : Event}: {==}",
            "reflexivity",
        ),
        (
            "def invalid() -> {Mouse{2, 3, 4, True{}} == Mouse{3, 2, 4, True{}} : Event}: {==}",
            "reflexivity",
        ),
        (
            "def invalid(image: Image) -> Image & Image: (image, image)",
            "observed Many",
        ),
        (
            "def invalid(event: Event) -> Event & Event: (event, event)",
            "observed Many",
        ),
        (
            "def invalid(-image: Image) -> Unit: Image.drop(image)",
            "permits None",
        ),
        (
            "def invalid(image: Image) -> Unit: invalid(image)",
            "must decrease structurally",
        ),
    ] {
        let book = parse(&format!("{BASE}\n{source}\n")).unwrap();
        let error = check_book(&book)
            .expect_err("invalid image/event proof or quantity must reject")
            .to_string();
        assert!(error.contains(diagnostic), "{source}: {error}");
    }
}
