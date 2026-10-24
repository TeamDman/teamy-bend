// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::syntax::parse;
use crate::syntax::parse_term;

const DATA: &str = "type N is Data:\n  Z{}\n  S{p: N}\n";

fn checked(source: &str) -> CheckedBook {
    check_book(&parse(&format!("{DATA}{source}")).unwrap()).unwrap()
}

#[test]
fn law_completion_uses_actual_arity_for_arrows_and_returned_functions() {
    let book = checked(
        "law f: N -> N\ndef f(x): x\nlaw g:\n  for x: N\n  N -> N\ndef g(): x => y => x\nlaw h:\n  for x: N\n  for y: N\n  N\ndef h(x): y => x\n",
    );
    for (name, count) in [("f", 1), ("g", 0), ("h", 1)] {
        let def = &book.engine.defs[name];
        assert_eq!(def.parameters.len(), count);
        assert!(def.completion_parameters.is_none());
    }
    assert_eq!(
        book.evaluate("f", &[parse_term("S{Z{}}").unwrap()])
            .unwrap()
            .to_string(),
        "S{Z{}}"
    );
    assert!(
        book.evaluate("f", &[])
            .unwrap_err()
            .to_string()
            .contains("needs 1")
    );
    assert!(
        book.evaluate("g", &[parse_term("Z{}").unwrap()])
            .unwrap_err()
            .to_string()
            .contains("needs 0")
    );
}

#[test]
fn law_completion_exposes_aliased_dependent_and_computed_telescopes() {
    let book = checked(
        "def Sig() -> Type: @-A: Type -> @value: A -> A\nlaw identity: Sig\ndef identity(B, renamed): renamed\ndef choose(T: Type) -> Type: T\nlaw selected: choose(N -> N)\ndef selected(n): n\n",
    );
    let def = &book.engine.defs["identity"];
    assert_eq!(def.parameters[0].quant, Quant::None);
    assert_eq!(def.parameters[0].name, "B");
    assert_eq!(def.parameters[1].name, "renamed");
    assert!(
        matches!(def.parameters[1].ty.as_ref(), Term::Var { id, .. } if *id == def.parameters[0].id)
    );
    assert_eq!(
        book.evaluate(
            "identity",
            &[parse_term("N").unwrap(), parse_term("S{Z{}}").unwrap()]
        )
        .unwrap()
        .to_string(),
        "S{Z{}}"
    );
    assert_eq!(
        book.evaluate("selected", &[parse_term("Z{}").unwrap()])
            .unwrap()
            .to_string(),
        "Z{}"
    );
}

#[test]
fn law_completion_derives_many_and_erased_quantities_from_the_law() {
    let book = checked(
        "law duplicate: @+x: N -> N\ndef duplicate(x):\n  match x:\n    case Z{}: Z{}\n    case S{p}: S{p}\nlaw erased: @-unused: N -> N\ndef erased(x): Z{}\n",
    );
    assert_eq!(
        book.engine.defs["duplicate"].parameters[0].quant,
        Quant::Many
    );
    assert_eq!(book.engine.defs["erased"].parameters[0].quant, Quant::None);
    assert_eq!(
        book.evaluate("erased", &[parse_term("Z{}").unwrap()])
            .unwrap()
            .to_string(),
        "Z{}"
    );
}

#[test]
fn law_completion_does_not_relax_invalid_bodies_holes_or_arity() {
    for (source, message) in [
        ("law f: N\ndef f(x): Z{}\n", "parameter list exceeds"),
        ("law f: N -> N\ndef f(x): N\n", "type"),
        ("law f: N -> N\ndef f(x): ?unfinished\n", "hole"),
        ("law f: N -> N\ndef f(x): x\ndef f(y): y\n", "duplicate"),
        ("law f: @-x: N -> N\ndef f(x): x\n", "permits none"),
    ] {
        let source = format!("{DATA}{source}");
        let error = match parse(&source) {
            Err(error) => error.to_string(),
            Ok(book) => check_book(&book).unwrap_err().to_string(),
        };
        assert!(error.to_lowercase().contains(message), "{source}: {error}");
    }
}

#[test]
fn law_completion_rejects_forged_metadata_and_identifier_exhaustion() {
    for mutation in [
        "unmatched",
        "law",
        "duplicate-id",
        "exhausted-id",
        "changed-type",
    ] {
        let mut book = parse(&format!("{DATA}law f: N -> N -> N\ndef f(x, y): x\n")).unwrap();
        let Declaration::Def(def) = book.declarations.last_mut().unwrap() else {
            unreachable!()
        };
        match mutation {
            "unmatched" => def.name = "absent".into(),
            "law" => def.body = None,
            "duplicate-id" => {
                let parameters = def.completion_parameters.as_mut().unwrap();
                parameters[1].1 = parameters[0].1;
            }
            "exhausted-id" => def.completion_parameters.as_mut().unwrap()[0].1 = usize::MAX,
            "changed-type" => def.ty = parse_term("N -> N").unwrap(),
            _ => unreachable!(),
        }
        check_book(&book).expect_err(mutation);
    }
}

#[test]
fn law_completion_tracks_elided_parameter_ids_and_keeps_bounded_exposure() {
    let book = parse(&format!(
        "{DATA}law f: N -> N\ndef f(x):\n  match x:\n    case Z{{}}: Z{{}}\n    case S{{p}}: p\n"
    ))
    .unwrap();
    let Declaration::Def(def) = book.declarations.last().unwrap() else {
        unreachable!()
    };
    let id = def.completion_parameters.as_ref().unwrap()[0].1;
    let (engine, _) = prepare_book(&book).unwrap();
    assert!(engine.fresh_id > id);
    let source = parse(&format!(
        "{DATA}def Sig() -> Type: N -> N\nlaw f: Sig\ndef f(n): n\n"
    ))
    .unwrap();
    let (mut engine, _) = prepare_book(&source).unwrap();
    for declaration in &source.declarations[..source.declarations.len() - 1] {
        match declaration {
            Declaration::Adt(d) => engine.validate_adt(d).unwrap(),
            Declaration::Def(d) => engine.validate_def(d).unwrap(),
        }
    }
    let Declaration::Def(completion) = source.declarations.last().unwrap() else {
        unreachable!()
    };
    let mut measured = engine.clone();
    let before = measured.fuel;
    measured.validate_def(completion).unwrap();
    let required = before - measured.fuel;
    assert!(required > 1);
    for fuel in [0, 1, required - 1] {
        let mut bounded = engine.clone();
        bounded.fuel = fuel;
        assert!(
            bounded
                .validate_def(completion)
                .unwrap_err()
                .to_string()
                .contains("budget")
        );
        assert_eq!(bounded.depth, 0);
        assert!(bounded.defs["f"].body.is_none());
    }
    let mut bounded = engine.clone();
    bounded.depth = 128;
    assert!(
        bounded
            .validate_def(completion)
            .unwrap_err()
            .to_string()
            .contains("nesting limit")
    );
    assert_eq!(bounded.depth, 128);
    engine.fuel = required;
    engine.validate_def(completion).unwrap();
    assert_eq!(engine.fuel, 0);
    assert!(engine.defs["f"].completion_parameters.is_none());
}

#[test]
fn strict_emitters_use_completion_arity_and_preserve_returned_closures() {
    let book = parse(&format!(
        "{DATA}law f: N -> N\ndef f(x): x\nlaw g:\n  for x: N\n  N\ndef g(): x => x\n"
    ))
    .unwrap();
    for compiler in [
        crate::compiler::compile_c,
        crate::compiler::compile_javascript,
    ] {
        assert!(
            compiler(&book, "f")
                .unwrap_err()
                .to_string()
                .contains("requires 1")
        );
        compiler(&book, "g").unwrap();
    }
}

#[test]
fn residual_named_fallback_receives_the_value_before_saturation() {
    let book = checked(
        "law fallback: N -> N -> N\ndef fallback(x, y): y\nlaw dispatch: N -> N -> N\ndef dispatch(): \\{Z: y => y; fallback}\ndef main() -> N: dispatch(S{Z{}}, S{S{Z{}}})\n",
    );
    assert_eq!(
        book.normalize_definition("main").unwrap().to_string(),
        "S{S{Z{}}}"
    );
    assert!(
        book.normalize_definition("dispatch")
            .unwrap()
            .to_string()
            .contains("fallback")
    );
}

#[test]
fn residual_fallback_preserves_unselected_demand_and_work_refusal() {
    let book = checked(
        "law fallback: N -> N\ndef fallback(x): x\nlaw dispatch: N -> N\ndef dispatch(): \\{Z: Z{}; fallback}\n",
    );
    let mut engine = book.engine.clone();
    let mut recursive = engine.defs["fallback"].clone();
    recursive.body = Some(parse_term("x => fallback(x)").unwrap());
    Rc::make_mut(&mut engine.defs).insert("fallback".into(), recursive);
    engine.fuel = 512;
    let value = engine
        .normalize(&parse_term("dispatch(Z{})").unwrap())
        .unwrap();
    assert_eq!(value.to_string(), "Z{}");
    engine.fuel = 512;
    assert!(
        engine
            .normalize(&parse_term("dispatch(S{Z{}})").unwrap())
            .unwrap_err()
            .to_string()
            .contains("budget")
    );
    assert_eq!(engine.depth, 0);
}

#[test]
fn completion_metadata_cannot_authorize_foreign_numeric_or_opaque_contracts() {
    let book = parse(&format!("{DATA}law f: N -> N\ndef f(n): n\n")).unwrap();
    let (mut engine, _) = prepare_book(&book).unwrap();
    for declaration in &book.declarations[..book.declarations.len() - 1] {
        match declaration {
            Declaration::Adt(d) => engine.validate_adt(d).unwrap(),
            Declaration::Def(d) => engine.validate_def(d).unwrap(),
        }
    }
    let Declaration::Def(completion) = book.declarations.last().unwrap() else {
        unreachable!()
    };
    for contract in ["foreign", "numeric", "opaque"] {
        let mut forged = completion.clone();
        forged.body = None;
        forged.foreign = contract == "foreign";
        let mut candidate = engine.clone();
        let result = match contract {
            "foreign" => candidate.validate_foreign_definition(&forged),
            "numeric" => candidate.validate_numeric_definition(&forged),
            "opaque" => candidate.validate_opaque_definition(&forged),
            _ => unreachable!(),
        };
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("completion metadata")
        );
        assert!(candidate.foreign_contracts.is_empty());
        assert!(candidate.numeric_contracts.is_empty());
        assert!(candidate.opaque_contracts.is_empty());
    }
}

#[test]
fn residual_empty_matches_retain_the_original_stuck_graph() {
    let book = checked("law id: N -> N\ndef id(x): x\n");
    for text in [
        "(\\{Z: Z{}; \\{}})(S{Z{}})",
        "(\\{Z: Z{}; \\{Other: Z{}; \\{}}})(S{Z{}})",
    ] {
        // Deliberately unreachable/ill-typed constructor applications exercise
        // reducer reconstruction, without granting a checked source token.
        let value = parse_term(text).unwrap();
        let mut engine = book.engine.clone();
        assert_eq!(engine.whnf(&value).unwrap().to_string(), value.to_string());
        assert_eq!(engine.depth, 0);
    }
}
