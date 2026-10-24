// SPDX-License-Identifier: MPL-2.0
#[path = "../../tests/support/normalization_shape.rs"]
mod normalization_shape;

use crate::kernel as normalization_kernel;
use crate::kernel::ExecutableEntry;
use crate::kernel::check_book;
use crate::kernel::check_executable;
use crate::syntax::load;
use crate::syntax::load_executable;
use crate::syntax::parse_term;
use normalization_shape::shape_in_book;
use std::path::Path;

#[path = "../../tests/support/constructor_binder_assertions.rs"]
mod constructor_binder_assertions;

#[path = "../../tests/support/closure_composition_cases.rs"]
mod closure_composition_cases;

#[test]
fn io_composition_helper_preserves_core_and_closed_calls_without_granting_proofs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-open-closures");
    let path = root.join("open_closure.bend");
    let source = load_executable(&path).unwrap();
    let checked = check_executable(&source).unwrap();
    assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Io);
    assert!(
        check_book(&load(&path).unwrap())
            .unwrap_err()
            .to_string()
            .contains("undefined name IO")
    );
    let graph = std::fs::read_to_string(root.join("normal-forms/open_closure.json")).unwrap();
    let expected: normalization_shape::Shape = facet_json::from_str(&graph).unwrap();
    let text = std::fs::read_to_string(root.join("normal-forms/open_closure.stdout")).unwrap();
    for _ in 0..2 {
        let mut engine = checked.engine.clone();
        engine.reset();
        let actual = engine.normalize_definition("pipe").unwrap();
        assert_eq!(format!("{actual}\n"), text);
        assert_eq!(shape_in_book(&actual, &source.book).unwrap(), expected);
        closure_composition_cases::assert_binders(&actual);
    }
    let cases = closure_composition_cases::cases();
    assert_eq!(cases.len(), 27);
    for case in cases {
        let mut engine = checked.engine.clone();
        engine.reset();
        let call = crate::kernel::reduce::apply(
            crate::kernel::term(crate::kernel::Term::Ref("pipe".into())),
            [
                parse_term(case.first).unwrap(),
                parse_term(case.second).unwrap(),
                parse_term(&case.input.to_string()).unwrap(),
            ],
        );
        let actual = engine.normalize(&call).unwrap();
        let expected = parse_term(&case.expected.to_string()).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
        assert_eq!(
            shape_in_book(&actual, &source.book).unwrap(),
            shape_in_book(&expected, &source.book).unwrap()
        );
    }
}

#[test]
fn minted_binder_execution_core_and_applications_do_not_grant_io_proofs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-binders");
    let path = root.join("printer_fresh_binder.bend");
    let source = load_executable(&path).unwrap();
    let checked = check_executable(&source).unwrap();
    assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Io);
    assert!(
        check_book(&load(&path).unwrap())
            .unwrap_err()
            .to_string()
            .contains("undefined name IO")
    );
    let graph =
        std::fs::read_to_string(root.join("normal-forms/printer_fresh_binder.json")).unwrap();
    let oracle: normalization_shape::Shape = facet_json::from_str(&graph).unwrap();
    let expected =
        std::fs::read_to_string(root.join("normal-forms/printer_fresh_binder.stdout")).unwrap();
    for _ in 0..2 {
        let mut engine = checked.engine.clone();
        engine.reset();
        let value = engine.normalize_definition("pick").unwrap();
        assert_eq!(format!("{value}\n"), expected);
        assert_eq!(shape_in_book(&value, &source.book).unwrap(), oracle);
        constructor_binder_assertions::assert_fresh_match_binders(&value);
        assert_eq!(
            shape_in_book(&parse_term(expected.trim()).unwrap(), &source.book).unwrap(),
            oracle
        );
    }
    for n in [0, 1, 2, 3, 7] {
        let mut engine = checked.engine.clone();
        engine.reset();
        let call = crate::kernel::reduce::apply(
            crate::kernel::term(crate::kernel::Term::Ref("pick".into())),
            [parse_term(&format!("{n}n")).unwrap()],
        );
        let value = engine.normalize(&call).unwrap();
        let expected = parse_term(if n < 2 { "0n" } else { "2n" }).unwrap();
        assert_eq!(value.to_string(), expected.to_string());
        assert_eq!(
            shape_in_book(&value, &source.book).unwrap(),
            shape_in_book(&expected, &source.book).unwrap()
        );
    }
}

#[test]
fn float_suffix_functions_preserve_core_structure_without_granting_proof_contracts() {
    for suffix in ["f", "g", "h"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-normalization")
            .join(format!("chain_suffix_f32_{suffix}.bend"));
        let source = load_executable(&path).unwrap();
        let checked = check_executable(&source).unwrap();
        assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Pure);
        let golden = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter_map(|line| line.strip_prefix("#|"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut engine = checked.engine.clone();
        engine.reset();
        let actual = engine.normalize_definition("main").unwrap();
        let actual_shape = shape_in_book(&actual, &source.book).unwrap();
        assert_eq!(
            actual_shape,
            shape_in_book(&parse_term(&golden).unwrap(), &source.book).unwrap(),
            "{suffix}"
        );
        assert_eq!(actual.to_string(), golden, "{suffix}");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, format!("{golden}\n").as_bytes());
        assert!(stderr.is_empty());
        let error = check_book(&load(&path).unwrap())
            .expect_err("executable numeric contracts cannot grant proof-mode primitives");
        assert!(
            error.to_string().contains("undefined name F32.add"),
            "{error}"
        );
    }
}

#[test]
fn reusable_binder_helpers_preserve_core_without_granting_io_proofs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-plus-binders");
    let mut count = 0;
    for suffix in ["case", "lambda_do", "nested", "row"] {
        let name = format!("plus_binder_{suffix}.bend");
        let path = root.join(&name);
        let source = load_executable(&path).unwrap();
        let checked = check_executable(&source).unwrap();
        assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Io);
        assert!(
            check_book(&load(&path).unwrap())
                .unwrap_err()
                .to_string()
                .contains("undefined name IO")
        );
        for row in include_str!("../../tests/fixtures/cpu-plus-binders/normalization.tsv").lines() {
            let mut fields = row.splitn(3, '\t');
            let file = fields.next().unwrap();
            let entry = fields.next().unwrap();
            let expected = fields.next().unwrap();
            if file != name {
                continue;
            }
            let mut engine = checked.engine.clone();
            engine.reset();
            let value = engine.normalize_definition(entry).unwrap();
            assert_eq!(value.to_string(), expected, "{name}/{entry}");
            assert_eq!(
                shape_in_book(&value, &source.book).unwrap(),
                shape_in_book(&parse_term(expected).unwrap(), &source.book).unwrap()
            );
            count += 1;
        }
    }
    assert_eq!(count, 10);
}
