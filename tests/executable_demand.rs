// SPDX-License-Identifier: MPL-2.0
//! The native IO lane uses executable demand, while pure normalization stays separate.

use std::fs;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const NAT_FAILURE: &[u8] = b"bend: a Nat past the largest immediate 2^48-1\n";
const REQUEST_FAILURE: &[u8] = b"bend: runtime fail-stop\n";
const HELPERS: &str = r#"
import Base
def limit() -> Nat: Nat.mul(U32.to_nat(16777215), U32.to_nat(16777217))
def overflow() -> Nat: Nat.add(limit(), 1n)
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case _: 0n
def invalid() -> Nat: reject(IO.print("SECRET")(Unit, x => Emit{x}))
def discard(n: Nat) -> String: "kept"
def pair(a: Nat, b: Nat) -> String: "kept"
"#;

fn run(source: &str) -> (u32, Vec<u8>, Vec<u8>) {
    let directory = std::env::temp_dir().join(format!(
        "teamy-bend-demand-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("main.bend");
    fs::write(&path, source).unwrap();
    let loaded = load_executable(&path).unwrap();
    let checked = check_executable(&loaded).unwrap();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = checked.run_main(&mut stdout, &mut stderr, &|| false);
    fs::remove_dir_all(directory).unwrap();
    (result.unwrap(), stdout, stderr)
}

#[test]
fn approved_demand_cases_have_exact_language_failures() {
    for (source, diagnostic) in [
        (
            include_str!("fixtures/cpu-demand-review/unused_argument.bend"),
            NAT_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/partial_add_overflow.bend"),
            NAT_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/partial_double_overflow.bend"),
            NAT_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/unused_rhs_error.bend"),
            REQUEST_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/ordinary_rhs_error.bend"),
            REQUEST_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/forced_request_wildcard.bend"),
            REQUEST_FAILURE,
        ),
    ] {
        assert_eq!(run(source), (1, vec![], diagnostic.to_vec()));
    }
}

#[test]
fn live_fields_arguments_and_ordinary_successors_fail_in_source_order() {
    for (expression, diagnostic) in [
        ("pair(overflow(), invalid())", NAT_FAILURE),
        ("pair(invalid(), overflow())", REQUEST_FAILURE),
        ("discard(Succ{limit()})", NAT_FAILURE),
        ("discard(Nat.mul(0n, invalid()))", REQUEST_FAILURE),
        ("discard(Nat.sub(0n, invalid()))", REQUEST_FAILURE),
        ("ignore(Box{overflow(), invalid()})", NAT_FAILURE),
        ("ignore(Box{invalid(), overflow()})", REQUEST_FAILURE),
    ] {
        let source = format!(
            r#"{HELPERS}
type Box is Data:
  Box{{first: Nat, second: Nat}}
def ignore(x: Box) -> String: "kept"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    Unit <- IO.print({expression})
    IO.print("after")
"#
        );
        assert_eq!(
            run(&source),
            (1, b"before".to_vec(), diagnostic.to_vec()),
            "{expression}"
        );
    }
}

#[test]
fn erased_parameters_fields_and_unselected_branches_stay_erased() {
    for expression in [
        "erased(overflow())",
        "erased(invalid())",
        "open(Box{overflow(), 7n})",
        "choose(True{})",
        "unused_closure(n => overflow())",
        "discard(limit())",
        "discard(Nat.add(limit(), 0n))",
        "discard(Nat.double(0n))",
    ] {
        let source = format!(
            r#"{HELPERS}
type Box is Data:
  Box{{-tag: Nat, value: Nat}}
def erased(-n: Nat) -> String: "kept"
def open(x: Box) -> String:
  match x:
    case Box{{tag, value}}: "kept"
def choose(b: Bool) -> String:
  match b:
    case True{{}}: "kept"
    case False{{}}: discard(overflow())
def unused_closure(f: Nat -> Nat) -> String: "kept"
def main() -> IO(Unit): IO.print({expression})
"#
        );
        assert_eq!(
            run(&source),
            (0, b"kept\n".to_vec(), vec![]),
            "{expression}"
        );
    }
}

#[test]
fn constructing_an_unused_io_action_does_not_perform_its_effect() {
    let source = r#"import Base
def ignore(action: IO(Unit)) -> IO(Unit): IO.print("kept")
def main() -> IO(Unit): ignore(IO.print("SECRET"))
"#;
    assert_eq!(run(source), (0, b"kept\n".to_vec(), vec![]));
}

#[test]
fn shared_differential_controls_cover_erasure_order_and_word_tails() {
    for (source, status, stdout, stderr) in [
        (
            include_str!("fixtures/cpu-demand-review/erased_and_unselected.bend"),
            0,
            &b"erased\nerased\n7\nselected\nunused closures\n"[..],
            &b""[..],
        ),
        (
            include_str!("fixtures/cpu-demand-review/nat_first.bend"),
            1,
            &b"before"[..],
            NAT_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/request_first.bend"),
            1,
            &b"before"[..],
            REQUEST_FAILURE,
        ),
        (
            include_str!("fixtures/cpu-demand-review/word_live_tail.bend"),
            1,
            &b""[..],
            REQUEST_FAILURE,
        ),
    ] {
        assert_eq!(run(source), (status, stdout.to_vec(), stderr.to_vec()));
    }
}

#[test]
fn compressed_constructor_tails_cannot_expose_a_value_before_failure() {
    let source = r#"import Base
def reject(op: IO.OP<Unit>) -> List<U32>:
  match op:
    case Emit{x}: Nil{}
    case Halt{code, message}: Nil{}
def build(n: Nat) -> List<U32>:
  match n:
    case 0n: reject(IO.print("SECRET")(Unit, x => Emit{x}))
    case 1n+p: 1 <> build(p)
def first(xs: List<U32>) -> U32:
  match xs:
    case Nil{}: 0
    case h <> t: h
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    Unit <- IO.print(U32.show(first(build(U32.to_nat(5000)))))
    IO.print("after")
"#;
    assert_eq!(
        run(source),
        (1, b"before".to_vec(), REQUEST_FAILURE.to_vec())
    );
}
