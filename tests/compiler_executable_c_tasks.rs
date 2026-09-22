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
const SMALL_NATIVE_STACK: &[&str] = &[
    "BEND_MAX_DEPTH=32",
    "BEND_MAX_FRAMES=32",
    "BEND_MAX_ALLOC=1048576",
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-tasks-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, source: &str, foreign: &str, joins: &str, extra: &[&str]) -> Output {
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
        write!(
            generated,
            r#"
int main(void) {{
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_continuations != 0
      || tb_frames != 0 || tb_depth != 0 || tb_tasks != 0)) {{
    (void)fputs("task program leaked owners or pending work\n", stderr);
    return 91;
  }}
  if (status == 0 && !({joins})) {{
    (void)fputs("task join count differed from checked source eligibility\n", stderr);
    return 92;
  }}
  return status;
}}
"#,
        )
        .unwrap();
        let definitions = [SMALL_NATIVE_STACK, extra].concat();
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

const PLAIN_JOIN: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32 & U32:
  left right = count(3n) count(4n)
  (left, right)
";

const FLAT_JOIN: &str = r"import Base
def sum(left: U32, right: U32) -> U32: U32.add(left, right)
def main() -> U32 & U32:
  left right = sum(1, 2) sum(3, 4)
  (left, right)
";

const MULTIWORD_JOIN: &str = r#"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
type ForkPair is Data: ForkPair{left: U32, right: U32}
def pair(n: Nat, seed: U32) -> ForkPair: ForkPair{count(n), seed}
def both(text: String) -> String & ForkPair & ForkPair:
  left right = pair(3n, 11) pair(4n, 22)
  (text, left, right)
def main() -> String & ForkPair & ForkPair: both("held")
"#;

const RAISED_SATURATED: &str = r"import Base
def make(n: Nat) -> U32 -> U32:
  match n:
    case 0n: value => value
    case 1n+p: value => U32.add(value, 1)
def main() -> U32 & U32:
  left right = make(0n, 7) make(1n, 7)
  (left, right)
";

const LEADING_LET_RETURN: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def make(n: Nat) -> U32 -> U32:
  kept = count(n)
  value => U32.add(kept, value)
def main() -> U32 & U32:
  left right = make(3n) make(4n)
  (left(7), right(7))
";

const DYNAMIC_GROUP: &str = r"import Base
def sum(left: U32, right: U32) -> U32: U32.add(left, right)
def both(left: U32 -> U32, right: U32 -> U32) -> U32 & U32:
  a b = left(3) right(4)
  (a, b)
def main() -> U32 & U32: both(sum(1), sum(2))
";

const MIXED_VALUE: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32 & U32:
  left right = count(3n) {7 : U32}
  (left, right)
";

const UNUSED_RHS: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32:
  left unused = count(3n) count(4n)
  left
";

const INTRINSIC_GROUP: &str = r"import Base
def main() -> U32 & U32:
  left right = U32.add(1, 2) U32.add(3, 4)
  (left, right)
";

const PARTIAL_GROUP: &str = r"import Base
def sum(left: U32, right: U32) -> U32: U32.add(left, right)
def main() -> U32 & U32:
  left right = sum(1) sum(2)
  (left(3), right(4))
";

const RAISED_PARTIAL: &str = r"import Base
def make(n: Nat) -> U32 -> U32:
  match n:
    case 0n: value => value
    case 1n+p: value => U32.add(value, 1)
def main() -> U32 & U32:
  left right = make(0n) make(1n)
  (left(7), right(7))
";

#[test]
fn saturated_and_dynamic_simultaneous_calls_adopt_task_joins() {
    for (source, expected) in [
        (PLAIN_JOIN, "(3, 4)\n"),
        (FLAT_JOIN, "(3, 7)\n"),
        (
            MULTIWORD_JOIN,
            "(\"held\", ForkPair{3, 11}, ForkPair{4, 22})\n",
        ),
        (RAISED_SATURATED, "(7, 8)\n"),
        (LEADING_LET_RETURN, "(10, 11)\n"),
        (DYNAMIC_GROUP, "(4, 6)\n"),
    ] {
        success(
            &Fixture::new().run(source, "", "tb_task_joins == 1", &[]),
            expected,
        );
    }
}

#[test]
fn intrinsics_partial_applications_and_pruned_groups_do_not_adopt_joins() {
    for (source, expected) in [
        (MIXED_VALUE, "(3, 7)\n"),
        (UNUSED_RHS, "3\n"),
        (INTRINSIC_GROUP, "(3, 7)\n"),
        (PARTIAL_GROUP, "(4, 6)\n"),
        (RAISED_PARTIAL, "(7, 8)\n"),
    ] {
        success(
            &Fixture::new().run(source, "", "tb_task_joins == 0", &[]),
            expected,
        );
    }
}

const NESTED_FORKS: &str = r"import Base
def identity(value: U32) -> U32: value
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p:
      left right = count(p) identity(1)
      U32.add(left, right)
def main() -> U32: count(Nat.mul(64n, 4n))
";

#[test]
fn nested_task_joins_resume_with_a_small_native_stack() {
    success(
        &Fixture::new().run(NESTED_FORKS, "", "tb_task_joins == 256", &[]),
        "256\n",
    );
}

#[test]
fn nested_task_graphs_obey_the_task_budget() {
    let shallow = NESTED_FORKS.replace("Nat.mul(64n, 4n)", "4n");
    success(
        &Fixture::new().run(&shallow, "", "tb_task_joins == 4", &["BEND_MAX_TASKS=32"]),
        "4\n",
    );
    failure(
        &Fixture::new().run(NESTED_FORKS, "", "1", &["BEND_MAX_TASKS=32"]),
        "task budget exhausted",
    );
}

const HELD_OWNERS: &str = r#"import Base
def identity(text: String) -> String: text
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def held(text: String, array: Array<String>) -> String & String & String & List<String> & U32:
  +shared = text
  left right = identity(shared) count(64n)
  (left, shared, shared, Array.to_list(~String, array), right)
def main() -> String & String & String & List<String> & U32:
  held(String.append("hel", "d"), ANode{ALeaf{String.append("a", "0")}, ALeaf{String.append("b", "1")}})
"#;

const RETURNED_OWNERS: &str = r#"import Base
def make(prefix: String) -> String -> String:
  kept = String.append(prefix, "!")
  suffix => String.append(kept, suffix)
def main() -> String & String:
  left right = make("A") make("B")
  (left("x"), right("y"))
"#;

#[test]
fn join_captures_hold_shared_strings_arrays_and_returned_closures_until_resume() {
    success(
        &Fixture::new().run(HELD_OWNERS, "", "tb_task_joins == 1", &[]),
        "(\"held\", \"held\", \"held\", [\"a0\", \"b1\"], 64)\n",
    );
    success(
        &Fixture::new().run(RETURNED_OWNERS, "", "tb_task_joins == 1", &[]),
        "(\"A!x\", \"B!y\")\n",
    );
}

const STAGED_REENTRY: &str = r#"import Base
def drive(function: (U32 -> U32) -> (U32 -> U32) -> (U32 -> U32) -> (U32 -> U32) -> U32,
          first: U32 -> U32, second: U32 -> U32) -> IO(U32): import "effect.c"
def identity(value: U32) -> U32: value
def nested(n: Nat, seed: U32) -> U32:
  match n:
    case 0n: seed
    case 1n+p:
      left right = nested(p, seed) identity(1)
      U32.add(left, right)
def both(first: U32 -> U32, second: U32 -> U32, work_first: U32 -> U32, work_second: U32 -> U32) -> U32:
  left right = work_first(first(1)) work_second(second(2))
  U32.add(left, right)
def main() -> IO(Unit):
  do IO<Unit>:
    result : U32 <- drive(both, nested(64n), nested(64n))
    IO.print(U32.show(result))
"#;

const STAGED_REENTRY_C: &str = r#"
static u32 staged_trace;
static Term stage_apply(Env e, const Term *captures, Term argument) {
  u32 marker = (u32)captures[0];
  (void)e;
  if (++staged_trace != marker) err_fail("argument staging order changed");
  if (marker == 1) io_out(stdout, "stage1\n", 7);
  else io_out(stdout, "stage2\n", 7);
  return argument + marker;
}
static Term work_apply(Env e, const Term *captures, Term argument) {
  u32 marker = (u32)captures[1];
  Term result;
  u64 length;
  char *held;
  if (++staged_trace != marker) err_fail("task execution preceded argument staging");
  if (marker == 3) io_out(stdout, "work1\n", 6);
  else io_out(stdout, "work2\n", 6);
  result = tb_apply(e, captures[0], argument);
  held = io_cstr(e, captures[2], &length);
  if (length != 4 || memcmp(held, "held", 4) != 0) err_fail("task reentry lost a foreign capture");
  free(held);
  return result + marker;
}
static Term drive_run(Env e, Term *fields, IoWork *work) {
  Term stage1[1] = {1}, stage2[1] = {2};
  Term first[3] = {fields[1], 3, io_str(e, "held", 4)};
  Term second[3] = {fields[2], 4, io_str(e, "held", 4)};
  Term function = fields[0];
  Term result;
  (void)work;
  function = tb_apply(e, function, tb_closure(e, 62000, 1, stage1));
  function = tb_apply(e, function, tb_closure(e, 62000, 1, stage2));
  function = tb_apply(e, function, tb_closure(e, 62001, 3, first));
  result = tb_apply(e, function, tb_closure(e, 62001, 3, second));
  if (staged_trace != 4) err_fail("task callback was omitted or repeated");
  return result;
}
static void __attribute__((constructor)) drive_use(void) {
  tb_register_closure(62000, stage_apply, 1);
  tb_register_closure(62001, work_apply, 3);
  io_eff(CID_DRIVE, drive_run, 0);
}
"#;

#[test]
fn all_arguments_are_staged_before_tasks_run_and_nested_foreign_reentry_keeps_pending_graphs() {
    success(
        &Fixture::new().run(
            STAGED_REENTRY,
            STAGED_REENTRY_C,
            "tb_task_joins == 129",
            &[],
        ),
        "stage1\nstage2\nwork1\nwork2\n141\n",
    );
}
