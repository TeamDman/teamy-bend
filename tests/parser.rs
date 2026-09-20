// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";

#[test]
fn recursive_nat_definition_and_proof_are_checked() {
    let source = format!(
        "{NAT}{}",
        r"
def add(a: Nat, b: Nat) -> Nat:
  match a:
    case Zero{}: b
    case Succ{p}: Succ{add(p, b)}
law add_zero:
  for +n: Nat
  {add(0n, n) == n : Nat}
def add_zero(n): {==}
"
    );
    let book = parse(&source).expect("parse valid recursive source");
    let checked = check_book(&book).expect("check recursive addition and law");
    let value = checked
        .evaluate(
            "add",
            &[
                parse_term("2n").expect("literal"),
                parse_term("3n").expect("literal"),
            ],
        )
        .expect("evaluate add");
    assert_eq!(value.to_string(), "Succ{Succ{Succ{Succ{Succ{Zero{}}}}}}");
}

#[test]
fn nested_patterns_compile_at_parameter_binders() {
    let source = format!(
        "{NAT}{}",
        r"
def small(n: Nat) -> Nat:
  match n:
    case Zero{}: 0n
    case Succ{Zero{}}: 1n
    case Succ{Succ{p}}: 2n
"
    );
    let book = parse(&source).expect("parse nested match");
    let checked = check_book(&book).expect("check nested match");
    assert_eq!(
        checked
            .evaluate("small", &[parse_term("3n").expect("literal")])
            .expect("evaluate")
            .to_string(),
        "Succ{Succ{Zero{}}}"
    );
}

#[test]
fn computed_scrutinees_and_unknown_patterns_fail() {
    let bad =
        format!("{NAT}def wrong(n: Nat) -> Nat:\n  match Succ{{n}}:\n    case Succ{{p}}: p\n");
    assert!(
        parse(&bad)
            .expect_err("computed scrutinee is invalid")
            .message
            .contains("parameters or fields")
    );
    let bad = format!("{NAT}def wrong(n: Nat) -> Nat:\n  match n:\n    case Missing{{}}: 0n\n");
    assert!(
        parse(&bad)
            .expect_err("unknown pattern is invalid")
            .message
            .contains("unknown constructor")
    );
}

#[test]
fn literals_and_operator_namespaces_are_structural() {
    assert!(
        matches!(parse_term("4294967295").expect("max U32").as_ref(),Term::Ctr {name,..} if name=="U32")
    );
    parse_term("4294967296").expect_err("reject overflowing U32");
    parse_term("1xyz").expect_err("reject invalid suffix");
    assert_eq!(
        parse_term("(a + b * c : U32)")
            .expect("operators")
            .to_string(),
        "U32.add(a)(U32.mul(b)(c))"
    );
    assert_eq!(
        parse_term("a + b").expect("Nat operators").to_string(),
        "Nat.add(a)(b)"
    );
    parse_term("'ab'").expect_err("character literal has one scalar");
    parse_term("?TODO").expect("holes remain explicit syntax for checker rejection");
}

#[test]
fn unsupported_forms_fail_without_silently_accepting_a_proof() {
    parse("def template(~x) -> Type: x").expect_err("template arguments need explicit types");
    parse("def effect() -> Type: import \"file.c\"").expect_err("foreign definitions unsupported");
    parse_term("do M<>: return").expect_err("return needs a value");
    parse_term("{==} garbage").expect_err("trailing input rejected");
}

#[test]
fn excessive_syntax_depth_fails_before_native_stack_exhaustion() {
    let nested = format!("{}Type{}", "(".repeat(200), ")".repeat(200));
    assert!(
        parse_term(&nested)
            .expect_err("depth bound")
            .message
            .contains("resource limit")
    );
    assert!(
        parse_term("3000n")
            .expect_err("literal bound")
            .message
            .contains("resource limit")
    );
    let arrow = format!("{}Type", "Type -> ".repeat(200));
    assert!(
        parse_term(&arrow)
            .expect_err("arrow bound")
            .message
            .contains("resource limit")
    );
    let telescope = format!("law too_many:\n{}Type", "  for x: Type\n".repeat(200));
    assert!(
        parse(&telescope)
            .expect_err("law telescope bound")
            .message
            .contains("resource limit")
    );
}

#[test]
fn reusable_pattern_fields_are_explicit_checked_lets() {
    let source = format!(
        "{NAT}{}",
        r"
def first(a: Nat, b: Nat) -> Nat: a
def twice(n: Nat) -> Nat:
  match n:
    case Zero{}: 0n
    case Succ{+p}: first(p, p)
"
    );
    let book = parse(&source).expect("reusable field parses");
    let checked = check_book(&book).expect("reusable field is Data");
    assert_eq!(
        checked
            .evaluate("twice", &[parse_term("2n").expect("Nat")])
            .expect("evaluate")
            .to_string(),
        "Succ{Zero{}}"
    );
    let source = format!(
        "{NAT}{}",
        r"
def first(a: Nat, b: Nat) -> Nat: a
def twice() -> Nat -> Nat: +n => first(n, n)
"
    );
    check_book(&parse(&source).expect("reusable lambda parses")).expect("reusable lambda checks");
}

#[test]
fn do_notation_elaborates_pure_bind_and_annotated_lets() {
    let source = format!(
        "{NAT}{}",
        r"
def M.pure(-R: Type, x: R) -> R: x
def M.bind(-A: Type, -R: Type, x: A, f: A -> R) -> R: f(x)
def result() -> Nat:
  do M<Nat>:
    x: Nat <- 1n
    y: Nat = 2n
    return x
"
    );
    let checked = check_book(&parse(&source).expect("do syntax")).expect("do core terms check");
    assert_eq!(
        checked
            .evaluate("result", &[])
            .expect("do result")
            .to_string(),
        "Succ{Zero{}}"
    );
    assert_eq!(
        parse_term("do M<>: return x")
            .expect("empty telescope")
            .to_string(),
        "M.pure(x)"
    );
}

#[test]
fn array_sugar_and_nested_type_closers_parse() {
    assert_eq!(
        parse_term("[0n: Nat * 4n]")
            .expect("power of two")
            .to_string(),
        "Array.new(Nat)(Succ{Succ{Zero{}}})(Zero{})"
    );
    assert_eq!(
        parse_term("[0n: Nat ^ 2n]").expect("depth").to_string(),
        "Array.new(Nat)(Succ{Succ{Zero{}}})(Zero{})"
    );
    parse_term("[0n: Nat * 3n]").expect_err("array count must be a power of two");
    assert_eq!(
        parse_term("List<List<Nat>>")
            .expect("nested closers")
            .to_string(),
        "List<List<Nat>>"
    );
}

#[test]
fn empty_datatype_parameters_and_reusable_parallel_lets_work() {
    let source = format!(
        "{NAT}{}",
        r"
type Marker<> is Data:
  Marker{}
def id(n: Nat) -> Nat: n
def use(a: Nat, b: Nat, c: Nat, d: Nat) -> Nat: a
def result() -> Nat:
  +a +b = id(1n) id(2n)
  use(a,a,b,b)
"
    );
    let checked = check_book(&parse(&source).expect("source parses")).expect("source checks");
    assert_eq!(
        checked.evaluate("result", &[]).expect("result").to_string(),
        "Succ{Zero{}}"
    );
}

#[test]
fn array_write_statements_rebind_in_source_order() {
    let source = format!(
        "{}{}",
        include_str!("../src/syntax/base.bend"),
        r"
def writes(a: Array<U32>) -> Array<U32>:
  a[0] <- 1
  a[1] <- 2; b = a
  b
"
    );
    let checked =
        check_book(&parse(&source).expect("write statements parse")).expect("rebindings check");
    assert_eq!(
        checked
            .evaluate(
                "writes",
                &[parse_term("ANode{ALeaf{0}, ALeaf{0}}").expect("U32 array")]
            )
            .expect("result")
            .to_string(),
        parse_term("ANode{ALeaf{1}, ALeaf{2}}")
            .expect("expected U32 array")
            .to_string()
    );
}

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-parser-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create test fixture");
        Self(path)
    }
    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).expect("write fixture");
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _result = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn local_import_aliases_share_canonical_definitions() {
    let fixture = Fixture::new();
    fixture.write("nat.bend", NAT);
    let entry = fixture.write(
        "main.bend",
        "import nat.bend as A\nimport ./nat.bend as B\ndef same(x: A.Nat) -> B.Nat: x\n",
    );
    let book = load(entry).expect("aliased imports load");
    assert_eq!(book.declarations.len(), 2);
    assert!(matches!(&book.declarations[0],Declaration::Adt(a) if a.name=="nat.Nat"));
    check_book(&book).expect("canonical aliases agree");
}

#[test]
fn imported_templates_preserve_definition_aliases_and_share_instances() {
    let fixture = Fixture::new();
    fixture.write(
        "nat.bend",
        &format!("{NAT}def inc(n: Nat) -> Nat: Succ{{n}}\n"),
    );
    fixture.write("other.bend", "type Marker is Data:\n  Marker{}\n");
    fixture.write(
        "library.bend",
        "import nat.bend as D\ndef app(~f: D.Nat -> D.Nat, n: D.Nat) -> D.Nat: f(n)\n",
    );
    let entry = fixture.write("main.bend", "import nat.bend as N\nimport other.bend as D\nimport library.bend as A\nimport ./library.bend as B\ndef first() -> N.Nat: A.app(~N.inc, N.Zero{})\ndef second() -> N.Nat: B.app(~N.inc, N.Succ{N.Zero{}})\n");
    let book = load(entry).expect("aliased templates load");
    assert_eq!(book.declarations.iter().filter(|declaration| matches!(declaration, Declaration::Def(definition) if definition.name.starts_with("library.app~"))).count(), 1);
    let checked = check_book(&book).expect("definition aliases survive a caller alias collision");
    assert_eq!(
        checked
            .evaluate_data("first", &[])
            .expect("first instance")
            .to_string(),
        "nat.Succ{nat.Zero{}}"
    );
    assert_eq!(
        checked
            .evaluate_data("second", &[])
            .expect("cached instance")
            .to_string(),
        "nat.Succ{nat.Succ{nat.Zero{}}}"
    );
}

#[test]
fn import_cycles_are_rejected() {
    let fixture = Fixture::new();
    let entry = fixture.write("a.bend", "import b.bend as B\n");
    fixture.write("b.bend", "import a.bend as A\n");
    assert!(
        load(entry)
            .expect_err("cycle rejected")
            .message
            .contains("cycle")
    );
}

#[test]
fn unsupported_features_and_bad_imports_have_specific_locations() {
    let failure = parse_term("f!(x)").expect_err("offload unsupported");
    assert_eq!((failure.line, failure.column), (1, 2));
    assert!(failure.message.contains("GPU offload"));
    assert!(
        parse_term("f(~Type)")
            .expect_err("template head must be declared")
            .message
            .contains("previously declared template")
    );
    let fixture = Fixture::new();
    let entry = fixture.write("bad-import.bend", "# heading\n\n  import nope.txt as Foo\n");
    let failure = load(entry).expect_err("invalid import extension");
    assert_eq!((failure.line, failure.column), (3, 3));
    assert!(failure.message.contains(".bend path"));
}

#[test]
fn bundled_base_checks_and_encodes_word_width() {
    let fixture = Fixture::new();
    let entry = fixture.write("base-test.bend", "import Base\ndef value() -> U32: 42\n");
    let book = load(entry).expect("Base parses");
    let checked = check_book(&book).expect("Base and U32 literal check");
    assert!(
        matches!(checked.evaluate("value",&[]).expect("evaluate literal").as_ref(),Term::Ctr {name,..} if name=="U32")
    );
}

#[test]
fn imported_base_names_cannot_be_redeclared() {
    let fixture = Fixture::new();
    let entry = fixture.write(
        "duplicate.bend",
        "import Base\nlaw Nat.add:\n  for a: Nat\n  for b: Nat\n  Nat\ndef Nat.add(a,b): a\n",
    );
    assert!(
        load(entry)
            .expect_err("Base declaration collision")
            .message
            .contains("duplicate declaration")
    );
}

#[test]
fn structural_word_patterns_fail_cleanly_when_compilation_is_too_deep() {
    let fixture = Fixture::new();
    let entry = fixture.write(
        "pattern.bend",
        "import Base\ndef f(x: U32) -> U32:\n  match x:\n    case 0: 1\n    case x: 0\n",
    );
    assert!(
        load(entry)
            .expect_err("structural word depth bound")
            .message
            .contains("pattern compilation exceeds")
    );
}

#[test]
fn imported_nat_literals_resolve_like_explicit_local_constructors() {
    for literal in ["Zero{}", "0n", "Succ{Zero{}}", "1n"] {
        let fixture = Fixture::new();
        fixture.write("nat.bend", &format!("{NAT}def value() -> Nat: {literal}\n"));
        let entry = fixture.write(
            "main.bend",
            "import nat.bend as N\ndef main() -> N.Nat: N.value\n",
        );
        let book = load(entry).expect("local Nat literal loads");
        let checked = check_book(&book).expect("local literal uses its declared constructors");
        let expected = if literal == "Zero{}" || literal == "0n" {
            "nat.Zero{}"
        } else {
            "nat.Succ{nat.Zero{}}"
        };
        assert_eq!(checked.evaluate("main", &[]).unwrap().to_string(), expected);
    }
}

#[test]
fn imported_nat_patterns_and_default_operators_use_the_local_type() {
    let fixture = Fixture::new();
    fixture.write(
        "nat.bend",
        &format!(
            "{NAT}def Nat.add(a: Nat, b: Nat) -> Nat:\n  match a:\n    case 0n: b\n    case 1n+p: 1n+Nat.add(p,b)\ndef value() -> Nat: 1n + 2n\n"
        ),
    );
    let entry = fixture.write(
        "main.bend",
        "import nat.bend as N\ndef main() -> N.Nat: N.value\n",
    );
    let checked = check_book(&load(entry).unwrap()).expect("local Nat patterns and operators");
    assert_eq!(
        checked.evaluate("main", &[]).unwrap().to_string(),
        "nat.Succ{nat.Succ{nat.Succ{nat.Zero{}}}}"
    );
}

#[test]
fn imported_base_literals_stay_global_and_distinct_local_nats_do_not_unify() {
    let fixture = Fixture::new();
    fixture.write("library.bend", "import Base\ndef value() -> Nat: 1n + 2n\n");
    let entry = fixture.write(
        "main.bend",
        "import library.bend as L\ndef main() -> Nat: L.value\n",
    );
    let checked = check_book(&load(entry).unwrap()).expect("imported Base remains global");
    assert_eq!(
        checked.evaluate("main", &[]).unwrap().to_string(),
        "Succ{Succ{Succ{Zero{}}}}"
    );
    fixture.write("nat.bend", NAT);
    let wrong = fixture.write(
        "wrong.bend",
        &format!("import nat.bend as N\n{NAT}def wrong() -> N.Nat: 0n\n"),
    );
    check_book(&load(wrong).unwrap()).expect_err("local and imported Nat remain distinct");
}
