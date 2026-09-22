// SPDX-License-Identifier: MPL-2.0
//! Scheduling annotations remain inert to checking and host evaluation.
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::kernel::substitute;
use teamy_bend::kernel::term;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
const WALK: &str = r"
def walk(n: Nat) -> Nat:
  match n:
    case Zero{}: Zero{}
    case Succ{p}: Succ{walk(p)}
";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-gpu-marks-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn body<'a>(book: &'a Book, name: &str) -> &'a TermRef {
    book.declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.name == name => definition.body.as_ref(),
            _ => None,
        })
        .unwrap()
}

fn head(value: &TermRef) -> &Term {
    let mut value = value.as_ref();
    while let Term::App(function, _) | Term::Ann(function, _) = value {
        value = function.as_ref();
    }
    value
}

#[test]
fn a_gpu_mark_belongs_to_a_named_reference_and_preserves_ordinary_calls() {
    for source in ["f!(x)", "f !(x)", "(f)!(x)"] {
        let parsed = parse_term(source).unwrap();
        assert!(matches!(head(&parsed), Term::GpuRef(name) if name == "f"));
        assert_eq!(parsed.to_string(), "f!(x)");
    }
    let plain = parse_term("f(x)").unwrap();
    assert!(matches!(head(&plain), Term::Ref(name) if name == "f"));
    assert!(matches!(parse_term("f!()").unwrap().as_ref(), Term::GpuRef(name) if name == "f"));
    for source in ["f! (x)", "f!", "f!(x)!(y)", "(x => x)!(0n)", "f!() => f"] {
        parse_term(source).expect_err("marks require a named call, never a lambda binder");
    }
    let bound = format!("{NAT}def run(f: Nat -> Nat, x: Nat) -> Nat: f!(x)\n");
    assert!(
        parse(&bound)
            .unwrap_err()
            .message
            .contains("a named def before !")
    );
}

#[test]
fn marks_do_not_change_proof_equality_or_either_host_evaluator() {
    let source = format!(
        "{NAT}{WALK}{}",
        r"
law inert:
  for +n: Nat
  {walk!(n) == walk(n) : Nat}
def inert(n): {==}
def main() -> Nat: walk!(3n)
"
    );
    let parsed = parse(&source).unwrap();
    let checked = check_book(&parsed).unwrap();
    let normalized = checked.evaluate("main", &[]).unwrap().to_string();
    let executed = checked.evaluate_data("main", &[]).unwrap().to_string();
    assert_eq!(normalized, "Succ{Succ{Succ{Zero{}}}}");
    assert_eq!(executed, normalized);
    assert!(matches!(head(body(&parsed, "main")), Term::GpuRef(name) if name == "walk"));
}

#[test]
fn marks_grant_no_descent_affinity_definition_or_proof_exceptions() {
    for invalid in [
        "def loop(n: Nat) -> Nat: loop!(n)",
        "law wrong: {walk!(0n) == 1n : Nat}\ndef wrong(): {==}",
        "def missing() -> Nat: unknown!(0n)",
        "type Cell is Type: Cell{}\ntype Two is Type: Two{left: Cell, right: Cell}\ndef take(c: Cell) -> Cell: c\ndef duplicate(c: Cell) -> Two: Two{take!(c), take(c)}",
    ] {
        let parsed = parse(&format!("{NAT}{WALK}{invalid}\n")).unwrap();
        check_book(&parsed).expect_err("a scheduling hint cannot justify an invalid program");
    }
}

#[test]
fn template_instances_keep_call_marks_and_distinguish_marked_compile_arguments() {
    let fixture = Fixture::new();
    fixture.write(
        "nat.bend",
        &format!("{NAT}def inc(n: Nat) -> Nat: Succ{{n}}\n"),
    );
    fixture.write(
        "library.bend",
        "import nat.bend as D\ndef app(~f: D.Nat -> D.Nat, n: D.Nat) -> D.Nat: f(n)\n",
    );
    let entry = fixture.write(
        "main.bend",
        r"import nat.bend as N
import library.bend as A
import ./library.bend as B
def marked() -> N.Nat: A.app!(~N.inc, N.Zero{})
def plain() -> N.Nat: B.app(~N.inc, N.Zero{})
def marked_argument() -> N.Nat: B.app(~N.inc!(), N.Zero{})
",
    );
    let parsed = load(entry).unwrap();
    let marked_name = match head(body(&parsed, "marked")) {
        Term::GpuRef(name) => name,
        other => panic!("marked template lost its reference annotation: {other:?}"),
    };
    assert!(matches!(head(body(&parsed, "plain")), Term::Ref(name) if name == marked_name));
    let instances = parsed
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.name.starts_with("library.app~") => {
                definition.body.as_ref()
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(instances.len(), 2);
    assert!(
        instances
            .iter()
            .any(|body| body.to_string().contains("nat.inc!"))
    );
    let checked = check_book(&parsed).unwrap();
    for name in ["marked", "plain", "marked_argument"] {
        assert_eq!(
            checked.evaluate_data(name, &[]).unwrap().to_string(),
            "nat.Succ{nat.Zero{}}"
        );
    }
}

#[test]
fn substituting_a_marked_reference_preserves_its_name_and_annotation() {
    let placeholder = term(Term::Var {
        name: "f".into(),
        id: 7,
    });
    let source = term(Term::App(placeholder, parse_term("0n").unwrap()));
    let replacement = term(Term::GpuRef("library.walk".into()));
    let result = substitute(&source, 7, &replacement);
    assert!(matches!(head(&result), Term::GpuRef(name) if name == "library.walk"));
    assert_eq!(result.to_string(), "library.walk!(Zero{})");
}

#[test]
fn marked_closures_keep_native_and_javascript_values() {
    let fixture = Fixture::new();
    let source = r"import Base
type FBox is Type: MkF{f: U32 -> U32}
def add2(a: U32, b: U32) -> U32: U32.add(a, b)
def run(f: U32 -> U32, x: U32) -> U32: f(x)
def unbox(box: FBox, x: U32) -> U32:
  match box:
    case MkF{f}: f(x)
def intrinsic() -> U32:
  f = {U32.add!(1) : U32 -> U32}
  f(2)
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(run!(add2!(2), 40)))
    Unit <- IO.print(U32.show(unbox!(MkF{y => U32.mul(y, 5)}, 4)))
    IO.print(U32.show(intrinsic()))
";
    let parsed = load_executable(fixture.write("main.bend", source)).unwrap();
    let checked = check_executable(&parsed).unwrap();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        checked
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"42\n20\n3\n");
    assert!(stderr.is_empty());
    let javascript = teamy_bend::compiler::compile_executable_javascript(&checked).unwrap();
    let output = Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
        .arg(fixture.write("main.cjs", &javascript))
        .output()
        .expect("executable compiler tests require Node.js");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, stdout);
    assert!(output.stderr.is_empty());
}
