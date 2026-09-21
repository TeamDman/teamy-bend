// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const SMALL_STACK: &[&str] = &[
    "BEND_MAX_DEPTH=32",
    "BEND_MAX_FRAMES=32",
    "BEND_MAX_ALLOC=4096",
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-tail-calls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, source: &str, foreign: &str, released: bool, definitions: &[&str]) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let mut generated = compile_executable_c(&checked).unwrap();
        if released {
            assert_eq!(generated.matches("int main(void)").count(), 1);
            generated = generated.replace("int main(void)", "static int checked_main(void)");
            generated.push_str(
                r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0)) {
    (void)fputs("tail-call program leaked VM owners\n", stderr);
    return 91;
  }
  return status;
}
"#,
            );
        }
        let executable = executable_c_compiler::compile(&self.0, &generated, definitions);
        executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(20),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

fn failure(output: &Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

const DEEP_NAT: &str = r"import Base
def countdown(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p: countdown(p, U32.add(total, 1))
def main() -> U32: countdown(Nat.mul(100n, 20n), 0)
";

const STRICT_ARGUMENTS: &str = r"import Base
def countdown(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p: countdown(p, U32.add(total, 1))
def pair(left: U32, right: U32) -> U32 & U32: (left, right)
def main() -> U32 & U32:
  pair(countdown(Nat.mul(100n, 20n), 0), countdown(Nat.mul(50n, 20n), 7))
";

#[test]
fn deep_tail_recursion_and_bare_definition_chains_fit_a_small_call_stack() {
    success(
        &Fixture::new().run(DEEP_NAT, "", true, SMALL_STACK),
        "2000\n",
    );
    success(
        &Fixture::new().run(STRICT_ARGUMENTS, "", true, SMALL_STACK),
        "(2000, 1007)\n",
    );
    let mut source = String::from("import Base\ndef next80() -> U32: 73\n");
    for index in (0..80).rev() {
        writeln!(source, "def next{index}() -> U32: next{}", index + 1).unwrap();
    }
    source.push_str("def main() -> U32: next0\n");
    success(&Fixture::new().run(&source, "", true, SMALL_STACK), "73\n");
    let mut source = String::from("import Base\ndef next80(-A: Data) -> U32: 73\n");
    for index in (0..80).rev() {
        writeln!(
            source,
            "def next{index}(-A: Data) -> U32: next{}(A)",
            index + 1
        )
        .unwrap();
    }
    source.push_str("def main() -> U32: next0(U32)\n");
    success(&Fixture::new().run(&source, "", true, SMALL_STACK), "73\n");
}

const ROTATING_OWNERS: &str = r#"import Base
def keep(text: String) -> String -> String: ignored => text
def rotate(n: Nat, left: String, right: String) -> String:
  match n:
    case 0n: String.append(left, right)
    case 1n+p:
      +shared = left
      next = keep(String.append(shared, ""))
      rotate(p, right, next(shared))
def main() -> String: rotate(Nat.add(Nat.mul(100n, 20n), 1n), "L", "R")
"#;

#[test]
fn tail_arguments_rotate_and_discard_owned_string_captures_without_heap_growth() {
    success(
        &Fixture::new().run(ROTATING_OWNERS, "", true, SMALL_STACK),
        "\"RL\"\n",
    );
}

const INDIRECT_CAPTURE: &str = r#"import Base
def invoke(function: Unit -> String) -> String: function(Unit{})
def countdown(n: Nat, value: String) -> String:
  match n:
    case 0n: String.append("done:", value)
    case 1n+p: invoke(ignored => countdown(p, value))
def main() -> String: countdown(Nat.mul(100n, 20n), String.append("owned-", "value"))
"#;

#[test]
fn indirect_tail_calls_preserve_captured_owned_arguments() {
    success(
        &Fixture::new().run(INDIRECT_CAPTURE, "", true, SMALL_STACK),
        "\"done:owned-value\"\n",
    );
}

const MATCH_FALLBACK: &str = r#"import Base
type TailRoute is Data:
  End{}
  Hop{count: Nat, left: String, right: String}
def finish(route: TailRoute) -> String:
  match route:
    case End{}: "end"
    case rest:
      match rest:
        case Hop{count, left, right}: String.append(left, right)
def follow(n: Nat, left: String, right: String) -> String:
  match n:
    case 0n: finish(Hop{0n, left, right})
    case rest:
      match rest:
        case 1n+p: follow(p, right, left)
def main() -> String: follow(Nat.add(Nat.mul(100n, 20n), 1n), "left", "right")
"#;

#[test]
fn match_fallback_and_last_constructor_field_application_are_tail_positions() {
    success(
        &Fixture::new().run(MATCH_FALLBACK, "", true, SMALL_STACK),
        "\"rightleft\"\n",
    );
}

const RETURNED_ERASED: &str = r#"import Base
def carry(-A: Data, n: Nat) -> A -> A:
  match n:
    case 0n: value => value
    case 1n+p: value => carry(A, p, value)
def main() -> String:
  finish = carry(String, Nat.mul(100n, 20n))
  finish(String.append("returned-", "owner"))
"#;

const TRAILING_ERASED: &str = r#"import Base
def carry(n: Nat, value: String, -A: Data) -> String:
  match n:
    case 0n: value
    case 1n+p: carry(p, value, A)
def main() -> String:
  carry(Nat.mul(100n, 20n), String.append("trailing-", "erased"), String)
"#;

#[test]
fn returned_partially_applied_closures_preserve_erased_arguments_and_live_values() {
    success(
        &Fixture::new().run(RETURNED_ERASED, "", true, SMALL_STACK),
        "\"returned-owner\"\n",
    );
    success(
        &Fixture::new().run(TRAILING_ERASED, "", true, SMALL_STACK),
        "\"trailing-erased\"\n",
    );
}

const NON_TAIL: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32: count(Nat.mul(100n, 20n))
";

const TAIL_SPIN: &str = r#"import Base
def spin() -> IO(Unit): import "effect.c"
def main() -> IO(Unit): spin()
"#;

const TAIL_SPIN_C: &str = r"
static Term spin_apply(Env e, const Term *captures, Term argument) {
  Loc application = task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0);
  (void)captures;
  e.mem[application] = tb_closure(e, 62000, 0, NULL);
  e.mem[application + 1] = argument;
  return term_tsk(FID_CLO_APPLY, application);
}
static Term spin_run(Env e, Term *fields, IoWork *work) {
  (void)fields; (void)work;
  return tb_apply(e, tb_closure(e, 62000, 0, NULL), term_pak(CID_UNIT, 0));
}
static void __attribute__((constructor)) spin_use(void) {
  tb_register_closure(62000, spin_apply, 0);
  io_eff(CID_SPIN, spin_run, 0);
}
";

#[test]
fn non_tail_recursion_keeps_the_depth_limit_and_tail_spins_keep_the_step_limit() {
    failure(
        &Fixture::new().run(NON_TAIL, "", false, SMALL_STACK),
        "call depth budget exhausted",
    );
    failure(
        &Fixture::new().run(
            TAIL_SPIN,
            TAIL_SPIN_C,
            false,
            &[
                "BEND_MAX_DEPTH=32",
                "BEND_MAX_FRAMES=32",
                "BEND_MAX_STEPS=2000",
                "BEND_MAX_ALLOC=4096",
            ],
        ),
        "evaluation budget exhausted",
    );
}

const FOREIGN_CALLBACK: &str = r#"import Base
def callback(function: String -> String, value: String) -> IO(String): import "effect.c"
def carry(n: Nat, prefix: String) -> String -> String:
  match n:
    case 0n: value => String.append(prefix, value)
    case 1n+p: value => carry(p, prefix, value)
def main() -> IO(Unit):
  do IO<Unit>:
    value : String <- callback(carry(Nat.mul(100n, 20n), String.append("foreign-", "capture:")), "argument")
    IO.print(value)
"#;

const FOREIGN_CALLBACK_C: &str = r"
static Term callback_run(Env e, Term *fields, IoWork *work) {
  Loc application = task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0);
  (void)work;
  e.mem[application] = fields[0];
  e.mem[application + 1] = fields[1];
  return corpus_eval(e.mem, term_tsk(FID_CLO_APPLY, application));
}
static void __attribute__((constructor)) callback_use(void) {
  io_eff(CID_CALLBACK, callback_run, 0);
}
";

#[test]
fn foreign_task_callback_finishes_a_deep_chain_and_releases_its_root_payload() {
    success(
        &Fixture::new().run(FOREIGN_CALLBACK, FOREIGN_CALLBACK_C, true, SMALL_STACK),
        "foreign-capture:argument\n",
    );
}

const FOREIGN_DROP: &str = r#"import Base
def discard(function: String -> String, value: String) -> IO(Unit): import "effect.c"
def keep(prefix: String) -> String -> String: value => String.append(prefix, value)
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- discard(keep(String.append("captured-", "owner")), String.append("argument-", "owner"))
    IO.print("discarded")
"#;

const FOREIGN_DROP_C: &str = r"
static Term discard_run(Env e, Term *fields, IoWork *work) {
  Loc application = task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0);
  (void)work;
  e.mem[application] = fields[0];
  e.mem[application + 1] = fields[1];
  term_drop(e, term_tsk(FID_CLO_APPLY, application));
  return term_pak(CID_UNIT, 0);
}
static void __attribute__((constructor)) discard_use(void) {
  io_eff(CID_DISCARD, discard_run, 0);
}
";

#[test]
fn foreign_root_task_drop_releases_both_payloads_and_rejects_malformed_metadata() {
    success(
        &Fixture::new().run(FOREIGN_DROP, FOREIGN_DROP_C, true, SMALL_STACK),
        "discarded\n",
    );
    let small_callback = FOREIGN_CALLBACK.replace("Nat.mul(100n, 20n)", "0n");
    for corruption in [
        "e.mem[application + 2] = 0;",
        "e.mem[application + 3] = 1;",
        "e.mem[application + 3] = UINT64_C(1) << 32;",
    ] {
        let foreign = FOREIGN_CALLBACK_C.replace(
            "return corpus_eval",
            &format!("{corruption}\n  return corpus_eval"),
        );
        failure(
            &Fixture::new().run(&small_callback, &foreign, false, SMALL_STACK),
            "foreign task continuation is unsupported",
        );
    }
    for (original, replacement, diagnostic) in [
        (
            "term_tsk(FID_CLO_APPLY, application)",
            "term_tsk(65535, application)",
            "foreign task is unsupported",
        ),
        (
            "term_tsk(FID_CLO_APPLY, application)",
            "(term_tsk(FID_CLO_APPLY, application) | RFC_BIT)",
            "reference-counted foreign task is unsupported",
        ),
        (
            "task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0)",
            "heap_alloc(e, 1)",
            "native allocation class mismatch",
        ),
    ] {
        let foreign = FOREIGN_CALLBACK_C.replace(original, replacement);
        failure(
            &Fixture::new().run(&small_callback, &foreign, false, SMALL_STACK),
            diagnostic,
        );
    }
}
