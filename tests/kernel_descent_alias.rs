// SPDX-License-Identifier: MPL-2.0
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";

fn program(body: &str) -> String {
    format!(
        "{NAT}def identity(n: Nat) -> Nat: n\ndef go(n: Nat) -> Nat:\n  match n:\n    case Zero{{}}: Zero{{}}\n    case Succ{{p}}:\n{body}\n"
    )
}

#[test]
fn transparent_let_aliases_preserve_visible_structural_descent() {
    for body in [
        "      +q = p\n      go(q)",
        "      +q = {p : Nat}\n      +r = q\n      go(r)",
        "      +shared = p\n      q r = shared shared\n      go(r)",
    ] {
        let source = format!(
            "{}law result: {{go(3n) == 0n : Nat}}\ndef result(): {{==}}\n",
            program(body)
        );
        check_book(&parse(&source).unwrap()).unwrap_or_else(|error| panic!("{body}: {error}"));
    }
}

#[test]
fn aliases_do_not_launder_non_decreasing_or_computed_recursive_arguments() {
    for body in [
        "      +q = {Succ{p} : Nat}\n      go(q)",
        "      +q = p\n      go(Succ{q})",
        "      +q = identity(p)\n      go(q)",
        "      +shared = p\n      +q = {Succ{shared} : Nat}\n      q r = shared q\n      go(r)",
        "      +q = p\n      q r = {Succ{q} : Nat} {Succ{q} : Nat}\n      go(r)",
    ] {
        let error = check_book(&parse(&program(body)).unwrap())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("recursive self-call must decrease structurally"),
            "{body}: {error}"
        );
    }
}

#[test]
fn alias_comparison_keeps_lexicographic_argument_order() {
    let source = format!(
        "{NAT}def go(+carry: Nat, n: Nat) -> Nat:\n  match n:\n    case Zero{{}}: carry\n    case Succ{{p}}:\n      +same = carry\n      +tail = p\n      go(same, tail)\nlaw result: {{go(2n, 3n) == 2n : Nat}}\ndef result(): {{==}}\n"
    );
    check_book(&parse(&source).unwrap()).unwrap();
    let invalid = source.replace("+same = carry", "+same = {Succ{carry} : Nat}");
    assert!(
        check_book(&parse(&invalid).unwrap())
            .unwrap_err()
            .to_string()
            .contains("recursive self-call must decrease structurally")
    );
}
