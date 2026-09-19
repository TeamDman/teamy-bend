// SPDX-License-Identifier: MPL-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_javascript;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::term;
use teamy_bend::syntax::parse;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
const ADD: &str = r"
def add(a: Nat, b: Nat) -> Nat:
  match a:
    case Zero{}: b
    case Succ{p}: Succ{add(p, b)}
";

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Script(PathBuf);

impl Script {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-compiler-{}-{}.cjs",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, source).expect("write generated JavaScript fixture");
        Self(path)
    }

    fn run(&self) -> Output {
        let node = std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into());
        Command::new(node)
            .arg(&self.0)
            .output()
            .expect("compiler runtime tests require Node.js on PATH or TEAMY_BEND_NODE")
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        let _result = fs::remove_file(&self.0);
    }
}

fn data_json(value: &TermRef) -> String {
    match value.as_ref() {
        Term::Ctr { name, args } => format!(
            "{{\"constructor\":{},\"fields\":[{}]}}",
            facet_json::to_string(name).expect("constructor name JSON"),
            args.iter().map(data_json).collect::<Vec<_>>().join(",")
        ),
        Term::Rfl => "{\"erased\":true}".to_owned(),
        other => panic!("differential fixture must normalize to constructor data: {other}"),
    }
}

fn assert_matches_kernel(book: &Book, entry: &str) {
    let checked = check_book(book).expect("reference kernel accepts source");
    let normal = checked
        .evaluate(entry, &[])
        .expect("reference kernel normalizes source");
    if matches!(normal.as_ref(), Term::Ctr { .. }) {
        let native = checked
            .evaluate_data(entry, &[])
            .expect("native lazy runtime evaluates data");
        assert_eq!(data_json(&native), data_json(&normal));
    }
    let source = compile_javascript(book, entry).expect("compile checked source");
    let output = Script::new(&source).run();
    assert!(
        output.status.success(),
        "generated program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("JSON UTF-8").trim(),
        format!("{{\"value\":{}}}", data_json(&normal))
    );
}

#[test]
fn compiled_recursive_addition_matches_rust_normalization() {
    let book = parse(&format!("{NAT}{ADD}\ndef main() -> Nat: add(2n, 3n)\n"))
        .expect("addition source parses");
    assert_matches_kernel(&book, "main");
}

#[test]
fn compiled_higher_order_closures_preserve_captured_values() {
    let book = parse(&format!(
        "{NAT}{ADD}{}",
        r"
def make_adder(n: Nat) -> @x: Nat -> Nat:
  x => add(n, x)
def apply_function(f: @x: Nat -> Nat, n: Nat) -> Nat:
  f(n)
def main() -> Nat:
  apply_function(make_adder(3n), 2n)
"
    ))
    .expect("higher-order source parses");
    assert_matches_kernel(&book, "main");
}

#[test]
fn compiled_pattern_fallback_returns_boolean_constructor() {
    let book = parse(&format!(
        "{NAT}{}",
        r"
type Bool is Data:
  False{}
  True{}
def is_zero(n: Nat) -> Bool:
  match n:
    case Zero{}: True{}
    case Succ{p}: False{}
def main() -> Bool: is_zero(1n)
"
    ))
    .expect("Boolean source parses");
    assert_matches_kernel(&book, "main");
}

#[test]
fn compiled_parallel_let_values_use_the_outer_scope() {
    let book = parse(&format!(
        "{NAT}{ADD}{}",
        r"
def swap(a: Nat, b: Nat) -> Nat:
  a b = b a
  add(a, b)
def main() -> Nat: swap(2n, 5n)
"
    ))
    .expect("parallel let source parses");
    assert_matches_kernel(&book, "main");
}

#[test]
fn compiler_refuses_false_proofs_before_generating_code() {
    let book = parse(&format!(
        "{NAT}{}",
        r"
law false_claim:
  {0n == 1n : Nat}
def false_claim(): {==}
def main() -> Nat: 0n
"
    ))
    .expect("false proof is valid source syntax");
    let error = compile_javascript(&book, "main").expect_err("false proof must fail");
    assert!(error.to_string().contains("equal endpoints"));
}

#[test]
fn compiler_requires_a_closed_declared_entry() {
    let book = parse(&format!("{NAT}{ADD}")).expect("addition source parses");
    assert!(
        compile_javascript(&book, "add")
            .expect_err("arguments required")
            .to_string()
            .contains("requires 2 arguments")
    );
    assert!(
        compile_javascript(&book, "missing")
            .expect_err("undefined entry")
            .to_string()
            .contains("undefined entry")
    );
}

#[test]
fn compiler_checks_proofs_then_emits_an_explicit_erased_marker() {
    let book = parse(&format!(
        "{NAT}\nlaw main:\n  {{0n == 0n : Nat}}\ndef main(): {{==}}\n"
    ))
    .expect("equality source parses");
    assert_matches_kernel(&book, "main");
}

#[test]
fn function_results_fail_without_pretending_to_be_normalized_data() {
    let book = parse(&format!("{NAT}\ndef main() -> @x: Nat -> Nat: x => x\n"))
        .expect("function source parses");
    let source = compile_javascript(&book, "main").expect("internal functions compile");
    let output = Script::new(&source).run();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("standalone entry must return data"));
}

#[test]
fn source_names_are_data_and_cannot_inject_javascript() {
    let mut book = parse("type Token is Data:\n  Safe{}\ndef main() -> Token: Safe{}\n")
        .expect("token source parses");
    let name = "\"); process.exit(99); /*";
    let entry = "untrusted\nentry\"; process.exit(88);";
    for declaration in &mut book.declarations {
        match declaration {
            Declaration::Adt(datatype) => datatype.constructors[0].name = name.to_owned(),
            Declaration::Def(definition) => {
                definition.name = entry.to_owned();
                definition.body = Some(term(Term::Ctr {
                    name: name.to_owned(),
                    args: Vec::new(),
                }));
            }
        }
    }
    assert_matches_kernel(&book, entry);
}

#[test]
fn compilation_is_deterministic_without_running_the_entry() {
    let book = parse(&format!("{NAT}\ndef main() -> Nat: 3n\n")).expect("closed source parses");
    let first = compile_javascript(&book, "main").expect("first compilation");
    let second = compile_javascript(&book, "main").expect("second compilation");
    assert_eq!(first, second);
    // The definition's constructors remain instructions in the output, instead
    // of a precomputed Rust result or an embedded call back into the compiler.
    assert!(first.contains("definitions[0] = lazy"));
    assert!(!first.contains("child_process"));
}
