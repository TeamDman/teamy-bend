// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::collections::BTreeMap;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::substitute;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

const NAMES: [&str; 5] = [
    "ctor_construct_000",
    "ctor_construct_001",
    "ctor_continuation",
    "ctor_continuation_go",
    "ctor_default",
];

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-continuations")
}

fn fields(value: &TermRef, expected: &str) -> Vec<TermRef> {
    let Term::Ctr { name, args } = value.as_ref() else {
        panic!("expected {expected}");
    };
    assert_eq!(name, expected);
    args.clone()
}

fn lambda(value: &TermRef) -> (usize, TermRef) {
    let Term::Lam { id, body, .. } = value.as_ref() else {
        panic!("expected lambda");
    };
    (*id, body.clone())
}

fn natural(n: u32) -> String {
    (0..n).fold("Z{}".into(), |value, _| format!("S{{{value}}}"))
}

#[test]
fn original_graphs_text_and_continuation_captures_match_the_pinned_oracle() {
    for name in NAMES {
        let book = load(root().join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let text =
            std::fs::read_to_string(root().join(format!("normal-forms/{name}.stdout"))).unwrap();
        let json =
            std::fs::read_to_string(root().join(format!("normal-forms/{name}.json"))).unwrap();
        let expected: Shape = facet_json::from_str(&json).unwrap();
        for _ in 0..2 {
            let value = checked.normalize_definition("main").unwrap();
            assert_eq!(format!("{value}\n"), text, "{name}");
            assert_eq!(shape_in_book(&value, &book).unwrap(), expected, "{name}");
            assert_eq!(
                shape_in_book(&parse_term(text.trim()).unwrap(), &book).unwrap(),
                expected
            );
            if name == "ctor_continuation" {
                let print = fields(&value, "Print");
                let (outer, tick) = lambda(&print[1]);
                let tick = fields(&tick, "Tick");
                let (inner, done) = lambda(&tick[0]);
                assert_ne!(outer, inner);
                let done = fields(&done, "Done");
                let successor = fields(&done[0], "S");
                let Term::Var { id, .. } = successor[0].as_ref() else {
                    panic!("expected capture");
                };
                assert_eq!(*id, outer);
            }
        }
    }
}

#[derive(Facet)]
struct Application {
    name: String,
    arguments: Vec<String>,
    value: String,
}

#[test]
fn closed_original_applications_preserve_both_fields_unused_arguments_and_default_value() {
    let cases: Vec<Application> = facet_json::from_str(include_str!(
        "fixtures/cpu-constructor-continuations/closed-applications.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 25);
    for name in NAMES
        .into_iter()
        .filter(|name| *name != "ctor_continuation")
    {
        let book = load(root().join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        for case in cases.iter().filter(|case| case.name == name) {
            let arguments = case
                .arguments
                .iter()
                .map(|x| parse_term(x).unwrap())
                .collect::<Vec<_>>();
            let value = checked.evaluate("main", &arguments).unwrap();
            let expected = parse_term(&case.value).unwrap();
            assert_eq!(value.to_string(), case.value);
            assert_eq!(
                shape_in_book(&value, &book).unwrap(),
                shape_in_book(&expected, &book).unwrap()
            );
        }
    }
}

#[derive(Facet)]
struct ContinuationApplication {
    n: u32,
    m: u32,
    text: String,
    value: String,
}

#[test]
fn closed_constructor_only_continuations_keep_outer_capture_and_ignore_inner_input() {
    let cases: Vec<ContinuationApplication> = facet_json::from_str(include_str!(
        "fixtures/cpu-constructor-continuations/continuation-applications.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 6);
    let book = load(root().join("ctor_continuation.bend")).unwrap();
    let checked = check_book(&book).unwrap();
    for case in cases {
        let value = checked.evaluate("main", &[]).unwrap();
        let print = fields(&value, "Print");
        assert_eq!(print[0].to_string(), case.text);
        let (outer, body) = lambda(&print[1]);
        // The already-normalized bodies contain constructors, lambdas and bound
        // variables only. These beta applications need no host effect driver.
        let tick = substitute(&body, outer, &parse_term(&natural(case.n)).unwrap());
        let tick = fields(&tick, "Tick");
        let (inner, body) = lambda(&tick[0]);
        let done = substitute(&body, inner, &parse_term(&natural(case.m)).unwrap());
        assert_eq!(done.to_string(), case.value);
        assert_eq!(
            shape_in_book(&done, &book).unwrap(),
            shape_in_book(&parse_term(&case.value).unwrap(), &book).unwrap()
        );
    }
}

#[test]
fn bounded_projection_grids_run_as_strict_pure_programs() {
    let cases: BTreeMap<String, Vec<Vec<u32>>> = facet_json::from_str(include_str!(
        "fixtures/cpu-constructor-continuations/applications.json"
    ))
    .unwrap();
    let mut count = 0;
    for name in NAMES {
        let path = root().join(format!("wrappers/{name}.bend"));
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        for inputs in &cases[name] {
            let args = inputs
                .iter()
                .map(|x| parse_term(&format!("{x}n")).unwrap())
                .collect::<Vec<_>>();
            let wanted = match name {
                "ctor_construct_000" => format!("[{}, {}]", inputs[0], inputs[1]),
                "ctor_construct_001" => inputs[0].to_string(),
                "ctor_default" => {
                    if inputs[0] == 0 {
                        "[0, 0]".into()
                    } else {
                        format!("[1, {}]", inputs[0])
                    }
                }
                _ => format!("[2, 1, 0, {}]", inputs[0] + 1),
            };
            let value = checked.evaluate("apply", &args).unwrap();
            assert_eq!(value.to_string(), wanted);
            assert_eq!(
                shape_in_book(&value, &book).unwrap(),
                shape_in_book(&parse_term(&wanted).unwrap(), &book).unwrap()
            );
            count += 1;
        }
        let expected = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
        let value = checked.evaluate("main", &[]).unwrap();
        assert_eq!(format!("{value}\n"), expected);
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(expected.trim()).unwrap(), &book).unwrap()
        );
    }
    assert_eq!(count, 34); // Plus two callback observations in ctor_continuation_go's main.
}
