// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

#[test]
fn list_constructors_preserve_normal_forms_and_all_78_pinned_applications() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-list-constructors");
    let mut observations = 0;
    for (fixture, expected_count, expected_type) in [
        ("list_ctor_000.bend", 7, "@a:List<&2, Bit> -> R"),
        ("list_ctor_001.bend", 7, "@a:List<&2, Nat> -> R"),
        ("list_ctor_002.bend", 7, "@a:List<&2, Nat> -> R"),
        ("list_ctor_003.bend", 7, "@a:List<&2, Nat> -> R"),
        (
            "list_ctor_004.bend",
            50,
            "@a:Nat -> @b:List<&2, Nat> -> @c:List<&2, Nat> -> R",
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
        assert_list_kind(&book);
        let definition_type = checked.definition_type("main").unwrap();
        assert_eq!(definition_type.to_string(), expected_type, "{fixture}");
        let oracle_type = parse_term(expected_type).unwrap();
        let mut actual = definition_type;
        let mut oracle = &oracle_type;
        while let (
            Term::All {
                quant,
                domain,
                body,
                ..
            },
            Term::All {
                quant: expected_quant,
                domain: expected_domain,
                body: expected_body,
                ..
            },
        ) = (actual.as_ref(), oracle.as_ref())
        {
            assert_eq!(quant, expected_quant, "{fixture}");
            assert_eq!(
                shape_in_book(domain, &book).unwrap(),
                shape_in_book(expected_domain, &book).unwrap()
            );
            actual = body;
            oracle = expected_body;
        }
        assert_eq!(
            shape_in_book(actual, &book).unwrap(),
            shape_in_book(oracle, &book).unwrap()
        );
        let value = checked.normalize_definition("main").unwrap();
        assert_eq!(value.to_string(), expected, "{fixture}");
        // Upstream's shadow-level suffix is display notation, not source syntax.
        // Compare the actual lowered oracle graph, preserving every bound index.
        let oracle: normalization_shape::Shape = facet_json::from_str(
            &std::fs::read_to_string(
                root.join("normal-forms")
                    .join(fixture.replace(".bend", ".json")),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(shape_in_book(&value, &book).unwrap(), oracle);
        let mut count = 0;
        for line in include_str!("fixtures/cpu-list-constructors/applications.tsv").lines() {
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
    assert_eq!(observations, 78);
}

fn assert_variable(value: &TermRef, expected: usize) {
    assert!(matches!(value.as_ref(), Term::Var { id, .. } if *id == expected));
}

fn assert_list_kind(book: &Book) {
    let list = book
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Adt(adt) if adt.name == "List" => Some(adt),
            _ => None,
        })
        .unwrap();
    assert_eq!(list.parameters.len(), 2);
    let quantity = &list.parameters[0];
    let element = &list.parameters[1];
    assert_eq!(quantity.quant, Quant::None);
    assert_eq!(element.quant, Quant::None);
    assert!(matches!(quantity.ty.as_ref(), Term::Qnt));
    for kind in [&element.ty, &list.kind] {
        let Term::Typ(value) = kind.as_ref() else {
            panic!("expected quantity-dependent kind");
        };
        assert_variable(value, quantity.id);
    }
    assert_eq!(list.constructors.len(), 2);
    assert_eq!(list.constructors[0].name, "Nil");
    assert!(list.constructors[0].fields.is_empty());
    let cons = &list.constructors[1];
    assert_eq!(cons.name, "Cons");
    assert_eq!(cons.fields.len(), 2);
    assert!(cons.fields.iter().all(|field| field.quant == Quant::Lone));
    assert_variable(&cons.fields[0].ty, element.id);
    let Term::Adt {
        name,
        args,
        excluded,
    } = cons.fields[1].ty.as_ref()
    else {
        panic!("expected recursive List field");
    };
    assert_eq!(name, "List");
    assert!(excluded.is_empty());
    assert_eq!(args.len(), 2);
    assert_variable(&args[0], quantity.id);
    assert_variable(&args[1], element.id);
}
