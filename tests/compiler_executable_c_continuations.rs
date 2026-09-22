// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

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
const SMALL_NATIVE_STACK: &[&str] = &[
    "BEND_MAX_DEPTH=32",
    "BEND_MAX_FRAMES=32",
    "BEND_MAX_ALLOC=65536",
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-continuations-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, source: &str, foreign: &str, extra_definitions: &[&str]) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert_eq!(
            generated
                .matches("static int tb_program_main(void)")
                .count(),
            1
        );
        let mut generated = generated.replace(
            "static int tb_program_main(void)",
            "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
        );
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_continuations != 0
      || tb_frames != 0 || tb_depth != 0)) {
    (void)fputs("continuation program leaked owners or pending calls\n", stderr);
    return 91;
  }
  return status;
}
"#,
        );
        let definitions = [SMALL_NATIVE_STACK, extra_definitions].concat();
        let executable = executable_c_compiler::compile(&self.0, &generated, &definitions);
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
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

fn failure(output: &Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("teamy-bend executable C: {diagnostic}\n")
    );
}

const NON_TAIL_COUNT: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32: count(Nat.mul(100n, 20n))
";

#[test]
fn two_thousand_pending_scalar_calls_complete_with_a_small_native_stack() {
    success(&Fixture::new().run(NON_TAIL_COUNT, "", &[]), "2000\n");
}

const PENDING_STRING: &str = r#"import Base
def grow(n: Nat, text: String) -> String:
  match n:
    case 0n: text
    case 1n+p:
      +held = text
      child = grow(p, held)
      String.append(held, child)
def main() -> String: grow(64n, "x")
"#;

const PENDING_CAPTURE: &str = r#"import Base
def keep(text: String) -> String -> String: ignored => text
def retain(n: Nat, text: String) -> String:
  match n:
    case 0n: text
    case 1n+p:
      next = keep(text)
      child = retain(p, "x")
      next(child)
def main() -> String: retain(64n, "root")
"#;

#[test]
fn suspended_calls_preserve_shared_strings_and_unused_captured_owners() {
    success(
        &Fixture::new().run(PENDING_STRING, "", &[]),
        &format!("\"{}\"\n", "x".repeat(65)),
    );
    success(&Fixture::new().run(PENDING_CAPTURE, "", &[]), "\"root\"\n");
}

const PENDING_ARRAY: &str = r#"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def expose(pair: Array<String> & String) -> String & List<String>:
  (array, previous) = pair
  (previous, Array.to_list(~String, array))
def select(flag: Bool, array: Array<String>, value: String) -> String & List<String>:
  match flag:
    case True{}: expose(Array.swap(String, array, 0, value))
    case False{}: expose(Array.swap(String, array, 1, value))
def replace(n: Nat, array: Array<String>, value: String) -> String & List<String>:
  index = count(n)
  select(U32.is_eq(U32.and(index, 1), 0), array, value)
def main() -> String & List<String>:
  replace(Nat.mul(100n, 20n),
    ANode{ALeaf{String.append("a", "0")}, ALeaf{String.append("b", "1")}},
    String.append("chang", "ed"))
"#;

#[test]
fn owned_array_and_replacement_survive_a_nested_call_and_both_resume_branches() {
    success(
        &Fixture::new().run(PENDING_ARRAY, "", &[]),
        "(\"a0\", [\"changed\", \"b1\"])\n",
    );
    let odd = PENDING_ARRAY.replace("Nat.mul(100n, 20n)", "Nat.add(Nat.mul(100n, 20n), 1n)");
    success(
        &Fixture::new().run(&odd, "", &[]),
        "(\"b1\", [\"a0\", \"changed\"])\n",
    );
}

const SIBLING_RESULTS: &str = r"import Base
def count(n: Nat, seed: U32) -> U32:
  match n:
    case 0n: seed
    case 1n+p: U32.add(1, count(p, seed))
def main() -> U32 & U32:
  left right = count(Nat.mul(10n, 70n), 11) count(Nat.mul(10n, 90n), 22)
  (left, right)
";

const SHADOWED_SIBLINGS: &str = r"import Base
def count(n: Nat, seed: U32) -> U32:
  match n:
    case 0n: seed
    case 1n+p: U32.add(1, count(p, seed))
def main() -> U32 & U32:
  left right = {11 : U32} {22 : U32}
  left right = count(Nat.mul(10n, 70n), right) count(Nat.mul(10n, 90n), left)
  (left, right)
";

const CONSTRUCTOR_SIBLINGS: &str = r#"import Base
type SuspendedPair is Data: SuspendedPair{left: String, right: String}
def keep(text: String) -> String -> String: ignored => text
def retain(n: Nat, text: String) -> String:
  match n:
    case 0n: text
    case 1n+p:
      next = keep(text)
      child = retain(p, "discarded")
      next(child)
def main() -> SuspendedPair:
  SuspendedPair{retain(64n, "left"), retain(64n, "right")}
"#;

#[test]
fn strict_sibling_calls_keep_prior_results_and_simultaneous_let_scope() {
    success(
        &Fixture::new().run(SIBLING_RESULTS, "", &[]),
        "(711, 922)\n",
    );
    success(
        &Fixture::new().run(SHADOWED_SIBLINGS, "", &[]),
        "(722, 911)\n",
    );
    success(
        &Fixture::new().run(CONSTRUCTOR_SIBLINGS, "", &[]),
        "SuspendedPair{\"left\", \"right\"}\n",
    );
}

const RETURNED_CLOSURES: &str = r"import Base
def maker(n: Nat) -> U32 -> U32:
  match n:
    case 0n: value => U32.add(value, 1)
    case 1n+p:
      next = maker(p)
      value => U32.add(1, next(value))
def main() -> U32: maker(64n, 7)
";

#[test]
fn recursively_returned_closures_resume_each_pending_application_once() {
    success(&Fixture::new().run(RETURNED_CLOSURES, "", &[]), "72\n");
}

const FOREIGN_REENTRY: &str = r#"import Base
def drive(outer: (U32 -> U32) -> U32, nested: U32 -> U32) -> IO(U32): import "effect.c"
def count(n: Nat, value: U32) -> U32:
  match n:
    case 0n: value
    case 1n+p: U32.add(1, count(p, value))
def outer(callback: U32 -> U32) -> U32:
  U32.add(callback(5), count(Nat.mul(50n, 10n), 7))
def main() -> IO(Unit):
  do IO<Unit>:
    result : U32 <- drive(outer, count(Nat.mul(100n, 20n)))
    IO.print(U32.show(result))
"#;

const FOREIGN_REENTRY_C: &str = r#"
static u32 drive_calls;
static u32 callback_calls;
static Term bridge_apply(Env e, const Term *captures, Term argument) {
  Term result;
  u64 length;
  char *held;
  if (++callback_calls != 1) err_fail("foreign callback repeated");
  io_out(stdout, "enter\n", 6);
  result = tb_apply(e, captures[0], argument);
  held = io_cstr(e, captures[1], &length);
  if (length != 4 || memcmp(held, "held", 4) != 0) err_fail("foreign capture changed across reentry");
  free(held);
  io_out(stdout, "leave\n", 6);
  return result + length;
}
static Term drive_run(Env e, Term *fields, IoWork *work) {
  Term captures[2] = {fields[1], io_str(e, "held", 4)};
  Loc application = task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0);
  (void)work;
  if (++drive_calls != 1) err_fail("foreign effect repeated");
  e.mem[application] = fields[0];
  e.mem[application + 1] = tb_closure(e, 62000, 2, captures);
  return corpus_eval(e.mem, term_tsk(FID_CLO_APPLY, application));
}
static void __attribute__((constructor)) drive_use(void) {
  tb_register_closure(62000, bridge_apply, 2);
  io_eff(CID_DRIVE, drive_run, 0);
}
"#;

const DIRECT_GENERATED_ABI_C: &str = r"
static Term call_generated_legacy(Env e, Term closure, Term argument) {
  u32 fid = (u32)term_aux(closure);
  u32 count = tb_closure_captures[fid];
  BendClosureFn function = tb_closure_functions[fid];
  Term *captures = NULL;
  Term result;
  if (count != 0) {
    Loc at = term_peek(e, closure);
    captures = (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
    memcpy(captures, e.mem + at, count * sizeof(Term));
    heap_free(e, cls_fit(count), at);
  }
  result = function(e, captures, argument);
  tb_host_free(captures);
  return result;
}
";

#[test]
fn foreign_reentry_preserves_outer_pending_calls_and_executes_side_effects_once() {
    success(
        &Fixture::new().run(FOREIGN_REENTRY, FOREIGN_REENTRY_C, &[]),
        "enter\nleave\n2516\n",
    );
    let deferred = FOREIGN_REENTRY_C
        .replace(
            "result = tb_apply(e, captures[0], argument);",
            "result = argument;",
        )
        .replace(
            "return result + length;",
            "return tb_tail_apply(e, captures[0], result + length);",
        );
    success(
        &Fixture::new().run(FOREIGN_REENTRY, &deferred, &[]),
        "enter\nleave\n2516\n",
    );
    // Trusted foreign code can invoke the registered generated callback ABI
    // directly after transferring its closure captures out of the VM shell.
    let direct = format!("{DIRECT_GENERATED_ABI_C}{FOREIGN_REENTRY_C}").replace(
        "result = tb_apply(e, captures[0], argument);",
        "result = call_generated_legacy(e, captures[0], argument);",
    );
    success(
        &Fixture::new().run(FOREIGN_REENTRY, &direct, &[]),
        "enter\nleave\n2516\n",
    );
}

#[test]
fn continuation_and_host_allocation_budgets_fail_closed_after_small_programs_succeed() {
    let shallow = NON_TAIL_COUNT.replace("Nat.mul(100n, 20n)", "4n");
    for (definitions, diagnostic) in [
        (
            &["BEND_MAX_CONTINUATIONS=64"][..],
            "generated continuation budget exhausted",
        ),
        (
            &["BEND_MAX_HOST_ALLOC=32768"][..],
            "host allocation budget exhausted",
        ),
    ] {
        success(&Fixture::new().run(&shallow, "", definitions), "4\n");
        failure(
            &Fixture::new().run(NON_TAIL_COUNT, "", definitions),
            diagnostic,
        );
    }
}

const FOREIGN_RECURSION: &str = r#"import Base
def recurse() -> IO(Unit): import "effect.c"
def main() -> IO(Unit): recurse()
"#;

const FOREIGN_RECURSION_C: &str = r"
static Term recursive_apply(Env e, const Term *captures, Term argument) {
  (void)captures;
  if (argument == 0) return term_pak(CID_UNIT, 0);
  return tb_apply(e, tb_closure(e, 62000, 0, NULL), argument - 1);
}
static Term recurse_run(Env e, Term *fields, IoWork *work) {
  (void)fields; (void)work;
  return tb_apply(e, tb_closure(e, 62000, 0, NULL), 2000);
}
static void __attribute__((constructor)) recurse_use(void) {
  tb_register_closure(62000, recursive_apply, 0);
  io_eff(CID_RECURSE, recurse_run, 0);
}
";

#[test]
fn true_recursive_foreign_c_calls_keep_the_native_depth_limit() {
    failure(
        &Fixture::new().run(FOREIGN_RECURSION, FOREIGN_RECURSION_C, &[]),
        "call depth budget exhausted",
    );
}
