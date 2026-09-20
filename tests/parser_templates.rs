// SPDX-License-Identifier: MPL-2.0
use std::fmt::Write;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
const ADD: &str = "def add(a: Nat, b: Nat) -> Nat:\n  match a:\n    case Zero{}: b\n    case Succ{p}: Succ{add(p, b)}\n";

fn source(body: &str) -> String {
    format!("{NAT}{ADD}{body}")
}

fn evaluate(body: &str) -> String {
    let book = parse(&source(body)).expect("valid template source");
    check_book(&book)
        .expect("specializations and callers checked")
        .evaluate_data("main", &[])
        .expect("closed constructor result")
        .to_string()
}

fn instances(book: &Book, name: &str) -> usize {
    book.declarations.iter().filter(|declaration| {
        matches!(declaration, Declaration::Def(definition) if definition.name.starts_with(&format!("{name}~")))
    }).count()
}

#[test]
fn local_matcher_lambda_uses_its_own_binder_and_expected_type() {
    assert_eq!(
        evaluate(
            r"
type Bool is Data:
  False{}
  True{}
def flip(b: Bool) -> Bool:
  f = {(x =>
    match x:
      case False{}: True{}
      case True{}: False{}) : Bool -> Bool}
  f(b)
def main() -> Bool: flip(False{})
"
        ),
        "True{}"
    );
}

#[test]
fn macro_substitution_preserves_affine_and_reusable_resource_checks() {
    let program = r"
def app(~f: Nat -> Nat, +x: Nat) -> Nat: f(x)
def main() -> Nat: app(~(y => add(y, y)), 3n)
";
    assert_eq!(
        evaluate(program),
        "Succ{Succ{Succ{Succ{Succ{Succ{Zero{}}}}}}}"
    );
    let affine = parse(&source(&program.replace("+x: Nat", "x: Nat"))).expect("affine syntax");
    let error = check_book(&affine).expect_err("duplicating an affine argument must fail");
    assert!(error.to_string().contains("Many"), "{error}");
}

#[test]
fn recursive_specializations_cache_closed_names_and_alpha_equivalent_lambdas() {
    let program = source(
        r"
type Nats is Data:
  Nil{}
  Cons{head: Nat, tail: Nats}
def inc(n: Nat) -> Nat: Succ{n}
def map(~f: Nat -> Nat, xs: Nats) -> Nats:
  match xs:
    case Nil{}: Nil{}
    case Cons{head, tail}: Cons{f(head), map(~f, tail)}
def first() -> Nats: map(~inc, Cons{1n, Nil{}})
def second() -> Nats: map(~(x => Succ{x}), Cons{2n, Nil{}})
def third() -> Nats: map(~(renamed => Succ{renamed}), Cons{3n, Nil{}})
def main() -> Nats: map(~inc, Cons{4n, Nil{}})
",
    );
    let book = parse(&program).expect("recursive template");
    assert_eq!(instances(&book, "map"), 2);
    assert_eq!(
        check_book(&book)
            .expect("recursive instances checked")
            .evaluate_data("main", &[])
            .expect("run map")
            .to_string(),
        "Cons{Succ{Succ{Succ{Succ{Succ{Zero{}}}}}}, Nil{}}"
    );
}

#[test]
fn specialization_keys_preserve_nested_shadowed_binding_identity() {
    // During specialization f(x) substitutes the outer x underneath another
    // binder also called x. Both lambdas then print `x => x => x`, but one
    // returns its outer argument and the other returns its inner argument.
    let program = r"
type Pair is Data:
  Pair{left: Nat, right: Nat}
def run(~f: Nat -> Nat -> Nat, a: Nat, b: Nat) -> Nat: f(a, b)
def outer(~f: Nat -> Nat -> Nat) -> Nat: run(~(x => f(x)), 1n, 2n)
def main() -> Pair: Pair{outer(~(y => x => y)), run(~(x => x => x), 1n, 2n)}
";
    assert_eq!(evaluate(program), "Pair{Succ{Zero{}}, Succ{Succ{Zero{}}}}");
    assert_eq!(
        instances(&parse(&source(program)).expect("shadow fixture"), "run"),
        2
    );
}

#[test]
fn multiargument_macro_beta_keeps_bidirectional_annotation_requirement() {
    // Upstream book_valid also rejects this at run~0: the remaining beta
    // application infers its result, and a constructor requires an annotation.
    let book = parse(&source(
        r"
def run(~f: Nat -> Nat -> Nat) -> Nat: f(1n, 2n)
def main() -> Nat: run(~(x => y => x))
",
    ))
    .expect("literal macro syntax");
    let error = check_book(&book).expect_err("unannotated constructor cannot be inferred");
    assert!(
        error.to_string().contains("expected type or annotation"),
        "{error}"
    );
    assert_eq!(
        evaluate(
            r"
def run(~f: Nat -> Nat -> Nat) -> Nat: f({1n : Nat}, {2n : Nat})
def main() -> Nat: run(~(x => y => x))
"
        ),
        "Succ{Zero{}}"
    );
}

#[test]
fn caller_variables_do_not_leak_into_specializations_or_capture_globals() {
    for global in ["", "def x() -> Nat: 10n\n"] {
        let program = format!(
            r"
{global}
def app(~f: Nat -> Nat, y: Nat) -> Nat: f(y)
def g(x: Nat) -> Nat: app(~(y => add(y, x)), 1n)
def main() -> Nat: g(3n)
"
        );
        let book = parse(&source(&program)).expect("open arguments remain plain calls");
        assert_eq!(instances(&book, "app"), 0);
        let error = check_book(&book).expect_err("caller local cannot become a global reference");
        assert!(error.to_string().contains("app"), "{error}");
    }
    assert_eq!(
        evaluate(
            r"
def app(~f: Nat -> Nat, y: Nat) -> Nat: f(y)
def g(x: Nat) -> Nat: app(~(x => Succ{x}), x)
def h(k: Nat) -> Nat: app(~(y => (k = {1n : Nat}; add(y, k))), k)
def main() -> Nat: add(g(1n), h(1n))
"
        ),
        "Succ{Succ{Succ{Succ{Zero{}}}}}"
    );
}

#[test]
fn compile_argument_operators_are_fixed_before_the_caller_annotation() {
    let base = include_str!("../src/syntax/base.bend");
    let program = r"
def apply_word(~f: U32 -> U32, x: U32) -> U32: f(x)
def main() -> U32: (apply_word(~(x => (x + 1 : U32)), 41) : U32)
";
    let book = parse(&format!("{base}{program}")).expect("annotated compile argument");
    let value = check_book(&book)
        .expect("inner annotation selects U32.add")
        .evaluate_data("main", &[])
        .expect("word result");
    assert_eq!(
        value.to_string(),
        parse_term("42").expect("expected word").to_string()
    );
    let wrong = program.replace("(x + 1 : U32)", "x + 1");
    let book = parse(&format!("{base}{wrong}")).expect("outer annotation syntax");
    check_book(&book).expect_err("outer annotation must not rename the argument's Nat.add");
}

#[test]
fn erased_template_arguments_keep_type_checking_and_nonleading_tilde_is_plain() {
    assert_eq!(
        evaluate(
            r"
def id(~A: Type, x: A) -> A: x
def keep(x: Nat, ~ignored: Nat) -> Nat: x
def main() -> Nat: keep(id(~Nat, 2n), 7n)
"
        ),
        "Succ{Succ{Zero{}}}"
    );
    for arguments in ["~True{}, 0n", "~?unfinished, 0n"] {
        let program = source(&format!(
            "type Bool is Data:\n  True{{}}\ndef discard(~unused: Nat, n: Nat) -> Nat: n\ndef main() -> Nat: discard({arguments})\n"
        ));
        let book = parse(&program).expect("argument syntax");
        check_book(&book).expect_err("even unused erased arguments must check");
    }
    let missing = parse(&source(
        "def app(~f: Nat -> Nat, n: Nat) -> Nat: f(n)\ndef main() -> Nat: app((x => x), 0n)\n",
    ))
    .expect("missing tilde syntax");
    check_book(&missing).expect_err("unfilled erased template parameter cannot be used live");
}

#[test]
fn templates_require_prior_declaration_typed_binders_and_valid_call_positions() {
    for body in [
        "def main() -> Nat: app(~(x => x), 0n)\ndef app(~f: Nat -> Nat, n: Nat) -> Nat: f(n)\n",
        "def app(~f, n: Nat) -> Nat: f(n)\n",
        "def main() -> Nat: add(~0n, 1n)\n",
        "def app(~f: Nat -> Nat, n: Nat, m: Nat) -> Nat: f(add(n, m))\ndef main() -> Nat: app(~(x => x), 0n, ~1n)\n",
        "def app(~n: Nat) -> Nat: n\ndef app(~n: Nat) -> Nat: n\n",
        "def app(~n: Nat) -> Nat: n\nlaw app: Nat\n",
        "law app: @n: Nat -> Nat\ndef app(~n): n\n",
        "def inc(n: Nat) -> Nat: Succ{n}\ndef app(~f: Nat -> Nat, n: Nat) -> Nat -> Nat:\n  f = add(n)\n  f\ndef main() -> Nat: app(~inc, 1n)(2n)\n",
    ] {
        parse(&source(body)).expect_err("invalid template declaration or argument syntax");
    }
}

#[test]
fn recursive_template_growth_is_finitely_bounded() {
    for program in [
        "def grow(~n: Nat, x: Nat) -> Nat: grow(~Succ{n}, x)\ndef main() -> Nat: grow(~0n, 0n)\n",
        "def grow(~f: Nat -> Nat, x: Nat) -> Nat: grow(~(y => f(f(y))), x)\ndef main() -> Nat: grow(~(n => Succ{n}), 0n)\n",
    ] {
        let error =
            parse(&source(program)).expect_err("unbounded specialization must stop during parsing");
        assert!(
            error.message.contains("resource limit") || error.message.contains("budget exhausted"),
            "{error}"
        );
    }
}

#[test]
fn specialization_retains_forward_reference_safety_and_descent_checks() {
    for program in [
        "def app(~f: Nat -> Nat, n: Nat) -> Nat: f(n)\ndef main() -> Nat: app(~later, 0n)\ndef later(n: Nat) -> Nat: n\n",
        "def loop(~f: Nat -> Nat, n: Nat) -> Nat: loop(~f, n)\ndef main() -> Nat: loop(~(x => x), 0n)\n",
        "@unsafe\ndef app(~f: Nat -> Nat, n: Nat) -> Nat: f(n)\ndef main() -> Nat: app(~(x => x), 0n)\n",
    ] {
        let book = parse(&source(program)).expect("template program parses");
        check_book(&book).expect_err("specialization cannot bypass a proof-checking boundary");
    }
}

#[test]
fn independent_specialization_count_is_bounded_and_cached_calls_do_not_cost_more() {
    let mut program = source("def keep(~n: Nat, x: Nat) -> Nat: x\n");
    for index in 0..256 {
        writeln!(
            program,
            "def n{index}() -> Nat: 0n\ndef v{index}() -> Nat: keep(~n{index}, 0n)"
        )
        .expect("append test source");
    }
    program.push_str("def repeated() -> Nat: keep(~n0, 1n)\n");
    let book = parse(&program).expect("exact specialization budget and cached repeat");
    assert_eq!(instances(&book, "keep"), 256);
    check_book(&book).expect("all independently checked specializations");
    program.push_str("def overflow() -> Nat: keep(~1n, 0n)\n");
    let error = parse(&program).expect_err("one more unique specialization exceeds budget");
    assert!(error.message.contains("budget exhausted"), "{error}");
}

#[test]
fn template_bodies_are_checked_when_instantiated_and_do_not_hide_unfilled_laws() {
    for (declaration, result_type) in [
        ("def deferred(~n: Nat) -> Nat: ?body\n", "Nat"),
        (
            "def deferred(~n: Nat) -> {0n == 1n : Nat}: {==}\n",
            "{0n == 1n : Nat}",
        ),
        ("@unsafe\ndef deferred(~n: Nat) -> Nat: n\n", "Nat"),
    ] {
        let unused = parse(&source(&format!("{declaration}def main() -> Nat: 0n\n")))
            .expect("unused template syntax is valid");
        assert_eq!(instances(&unused, "deferred"), 0);
        check_book(&unused).expect("unused template text is deferred, not a checked declaration");
        let used = parse(&source(&format!(
            "{declaration}def main() -> {result_type}: deferred(~0n)\n"
        )))
        .expect("instantiated template parses");
        assert_eq!(instances(&used, "deferred"), 1);
        check_book(&used).expect_err("each instantiated body must satisfy the proof checker");
    }
    let unfilled = parse(&source(
        "law unfinished: Nat\ndef ignore(~n: Nat) -> Nat: 0n\ndef main() -> Nat: ignore(~unfinished)\n",
    ))
    .expect("ordinary law and erased template argument parse");
    check_book(&unfilled).expect_err("an ignored erased argument does not fill an ordinary law");
}
