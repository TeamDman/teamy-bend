// SPDX-License-Identifier: MPL-2.0
use std::rc::Rc;
use teamy_bend::kernel::LetBinding;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::term;

fn variable(name: &str, id: usize) -> TermRef {
    term(Term::Var {
        name: name.into(),
        id,
    })
}

fn reference(name: &str) -> TermRef {
    term(Term::Ref(name.into()))
}

fn lambda(name: &str, id: usize, body: TermRef) -> TermRef {
    term(Term::Lam {
        name: name.into(),
        id,
        body,
    })
}

fn constructor(name: &str, args: Vec<TermRef>) -> TermRef {
    term(Term::Ctr {
        name: name.into(),
        args,
    })
}

fn assert_display(value: &TermRef, expected: &str) {
    let before = format!("{value:?}");
    let owners = Rc::strong_count(value);
    assert_eq!(value.to_string(), expected);
    assert_eq!(format!("{value:?}"), before);
    assert_eq!(Rc::strong_count(value), owners);
}

#[test]
fn shadowed_names_use_scope_levels_without_changing_semantic_ids() {
    let value = lambda(
        "a",
        700,
        lambda(
            "b",
            900,
            lambda(
                "b",
                1000,
                constructor(
                    "D",
                    vec![variable("a", 700), variable("b", 900), variable("b", 1000)],
                ),
            ),
        ),
    );
    assert_display(&value, "a => b => b => D{a, b^1, b}");
    let value = lambda(
        "x",
        500,
        constructor(
            "Pair",
            vec![
                reference("x"),
                term(Term::GpuRef("x".into())),
                variable("x", 500),
            ],
        ),
    );
    assert_display(&value, "x => Pair{x^, x^!, x}");
    assert_display(
        &constructor(
            "Pair",
            vec![lambda("x", 400, reference("x")), reference("x")],
        ),
        "Pair{x => x^, x}",
    );
    assert_display(&variable("external", 999), "external");
}

#[test]
fn sugar_precedence_and_child_fields_keep_their_lexical_scope() {
    let outer = variable("x", 50);
    let inner = variable("x", 60);
    for (body, expected) in [
        (constructor("Succ", vec![Rc::clone(&outer)]), "1n+x^0"),
        (
            constructor("Con", vec![Rc::clone(&outer), Rc::clone(&inner)]),
            "x^0 <> x",
        ),
        (
            constructor("Con", vec![Rc::clone(&outer), constructor("Nil", vec![])]),
            "[x^0]",
        ),
        (
            constructor("Tuple", vec![Rc::clone(&outer), Rc::clone(&inner)]),
            "(x^0, x)",
        ),
        (constructor("ALeaf", vec![Rc::clone(&outer)]), "[x^0]"),
        (term(Term::App(reference("f"), Rc::clone(&outer))), "f(x^0)"),
        (
            term(Term::Ann(Rc::clone(&outer), reference("T"))),
            "{x^0 : T}",
        ),
        (
            term(Term::Eql {
                left: Rc::clone(&outer),
                right: Rc::clone(&inner),
                ty: reference("T"),
            }),
            "{x^0 == x : T}",
        ),
        (
            term(Term::Min(Rc::clone(&outer), Rc::clone(&inner))),
            "(x^0 <&> x)",
        ),
        (term(Term::Typ(Rc::clone(&outer))), "Kind(x^0)"),
        (
            term(Term::Adt {
                name: "Box".into(),
                args: vec![Rc::clone(&outer), Rc::clone(&inner)],
                excluded: vec![],
            }),
            "Box<x^0, x>",
        ),
    ] {
        assert_display(
            &lambda("x", 50, lambda("x", 60, body)),
            &format!("x => x => {expected}"),
        );
    }
    let arrow = term(Term::All {
        quant: Quant::Lone,
        name: "x".into(),
        id: 50,
        domain: term(Term::Typ(term(Term::Qua(Quant::Lone)))),
        body: term(Term::All {
            quant: Quant::Lone,
            name: "x".into(),
            id: 60,
            domain: outer,
            body: constructor("P", vec![variable("x", 50), inner]),
        }),
    });
    assert_display(&arrow, "@x:Type -> @x:x -> P{x^0, x}");
}

#[test]
fn let_and_rewrite_binders_apply_only_in_their_body_or_motive() {
    let value = lambda(
        "x",
        40,
        term(Term::Let {
            bindings: vec![LetBinding {
                quant: Quant::Lone,
                name: "x".into(),
                id: 80,
                value: variable("x", 40),
            }],
            body: constructor("Pair", vec![variable("x", 40), variable("x", 80)]),
        }),
    );
    assert_display(&value, "x => x = x; Pair{x^0, x}");
    let value = lambda(
        "x",
        40,
        term(Term::Rwt {
            evidence: variable("x", 40),
            motive: lambda(
                "_",
                80,
                lambda(
                    "x",
                    120,
                    constructor(
                        "P",
                        vec![variable("x", 40), variable("x", 120), variable("_", 80)],
                    ),
                ),
            ),
            body: variable("x", 40),
        }),
    );
    assert_display(&value, "x => %x@x : P{x^0, x, _}; x");
}

fn datatype(name: &str, args: Vec<TermRef>, excluded: &[&str]) -> TermRef {
    term(Term::Adt {
        name: name.into(),
        args,
        excluded: excluded.iter().map(|name| (*name).into()).collect(),
    })
}

fn assert_refined_display(name: &str, value: &TermRef) {
    let expected: std::collections::BTreeMap<String, String> = facet_json::from_str(include_str!(
        "fixtures/cpu-refined-type/renderer-expected.json"
    ))
    .unwrap();
    assert_display(value, &expected[name]);
}

#[test]
fn refined_datatype_display_keeps_order_arguments_and_shadowed_identity() {
    for (name, value) in [
        ("plain", datatype("N", vec![], &[])),
        ("parameterized", datatype("Box", vec![reference("T")], &[])),
        ("one", datatype("N", vec![], &["Z"])),
        ("multiple", datatype("N", vec![], &["Z", "S"])),
        (
            "parameters",
            datatype("Box", vec![reference("T")], &["Empty", "Full"]),
        ),
    ] {
        assert_refined_display(name, &value);
    }
    let outer = variable("T", 50);
    let inner = variable("T", 60);
    let refined = datatype(
        "Box",
        vec![Rc::clone(&outer), Rc::clone(&inner)],
        &["Empty"],
    );
    let value = lambda("T", 50, lambda("T", 60, refined));
    let owners = (Rc::strong_count(&outer), Rc::strong_count(&inner));
    assert_refined_display("shadowed", &value);
    assert_eq!((Rc::strong_count(&outer), Rc::strong_count(&inner)), owners);
}

#[test]
fn refined_datatype_precedence_is_preserved_in_high_and_low_contexts() {
    let refined = datatype("N", vec![], &["Z", "S"]);
    let arrow = |domain, body| {
        term(Term::All {
            quant: Quant::Lone,
            name: "x".into(),
            id: 70,
            domain,
            body,
        })
    };
    let owners = Rc::strong_count(&refined);
    for (name, value) in [
        ("domain", arrow(Rc::clone(&refined), reference("R"))),
        ("codomain", arrow(reference("R"), Rc::clone(&refined))),
        (
            "application_head",
            term(Term::App(Rc::clone(&refined), reference("x"))),
        ),
        (
            "argument",
            term(Term::App(reference("f"), Rc::clone(&refined))),
        ),
        (
            "list_head",
            constructor("Con", vec![Rc::clone(&refined), reference("tail")]),
        ),
        (
            "list_tail",
            constructor("Con", vec![reference("head"), Rc::clone(&refined)]),
        ),
        (
            "constructor",
            constructor("Wrap", vec![Rc::clone(&refined)]),
        ),
        (
            "nested",
            datatype("Box", vec![Rc::clone(&refined)], &["Empty"]),
        ),
        (
            "annotation",
            term(Term::Ann(reference("x"), Rc::clone(&refined))),
        ),
        ("nat_tail", constructor("Succ", vec![Rc::clone(&refined)])),
    ] {
        assert_refined_display(name, &value);
    }
    assert_eq!(Rc::strong_count(&refined), owners);
}
