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

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-direct-calls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, source: &str, minimum_reuse: u32, definitions: &[&str]) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert!(generated.contains("return tb_segment_call("));
        // This probe runs on the coordinator only. Worker routing and concurrent
        // ownership have separate tests with synchronized instrumentation.
        let mut instrumented = String::from(
            r"
static unsigned long long direct_calls, reused_calls;
static void record_direct_call(unsigned int fid, int reused) {
  (void)fid; ++direct_calls;
  if (reused) ++reused_calls;
}
#define TB_DIRECT_CALL(fid, reused) record_direct_call(fid, reused)
",
        );
        instrumented.push_str(&generated.replace(
            "static int tb_program_main(void)",
            "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
        ));
        write!(
            instrumented,
            r#"
int main(void) {{
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {{
    (void)fputs("direct-call program retained runtime owners\n", stderr);
    return 91;
  }}
  if (status == 0 && (direct_calls == 0 || reused_calls < {minimum_reuse}u)) {{
    (void)fputs("direct-call program did not reuse the expected frames\n", stderr);
    return 92;
  }}
  return status;
}}
"#,
        )
        .unwrap();
        let mut selected = vec![
            "BEND_CPU_WORKERS=1",
            "BEND_MAX_CONTINUATIONS=32",
            "BEND_MAX_ALLOC=16384",
        ];
        for definition in definitions {
            let name = definition.split('=').next().unwrap();
            selected.retain(|prior| prior.split('=').next() != Some(name));
            selected.push(*definition);
        }
        let executable = executable_c_compiler::compile(&self.0, &instrumented, &selected);
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

const PERMUTATION: &str = r#"import Base
type Shipment is Data: Shipment{left: U32, right: U32, text: String}
def cycle(n: Nat, left: U32, right: U32, text: String) -> Shipment:
  match n:
    case 0n: Shipment{left, right, text}
    case 1n+p: cycle(p, right, left, text)
def main() -> Shipment:
  cycle(U32.to_nat(5001), 7, 11, String.append("per", "muted"))
"#;

#[test]
fn self_tail_calls_reuse_frames_while_permuting_arguments_and_preserving_owners() {
    success(
        &Fixture::new().run(PERMUTATION, 5001, &[]),
        "Shipment{11, 7, \"permuted\"}\n",
    );
}

const ALTERNATING: &str = r"import Base
type DirectChoice is Data:
  RawChoice{number: U32}
  OwnedChoice{text: String}
def flip(value: DirectChoice) -> DirectChoice:
  match value:
    case RawChoice{number}: OwnedChoice{U32.show(number)}
    case OwnedChoice{text}: RawChoice{U32.from_nat(String.length(text))}
def cycle(n: Nat, value: DirectChoice) -> DirectChoice:
  match n:
    case 0n: value
    case 1n+p: cycle(p, flip(value))
def main() -> DirectChoice: cycle(U32.to_nat(1001), RawChoice{12345})
";

#[test]
fn a_reused_tail_frame_accepts_changing_sum_ownership_after_a_non_tail_call() {
    // U32.show traverses its 32 bits with non-tail calls. Leave room for that
    // helper while keeping the budget well below the 1,001 outer tail calls.
    success(
        &Fixture::new().run(ALTERNATING, 1001, &["BEND_MAX_CONTINUATIONS=128"]),
        "OwnedChoice{\"1\"}\n",
    );
}

const NESTED: &str = r#"import Base
type DirectPair is Data: DirectPair{number: U32, text: String}
def increment(value: DirectPair) -> DirectPair:
  match value:
    case DirectPair{number, saved}: DirectPair{U32.add(number, 1), saved}
def sum(n: Nat, text: String) -> DirectPair:
  match n:
    case 0n: DirectPair{0, text}
    case 1n+p: increment(sum(p, text))
def pass(value: DirectPair) -> DirectPair: value
def main() -> DirectPair: pass(sum(12n, String.append("ret", "ained")))
"#;

#[test]
fn non_tail_direct_calls_restore_multiword_results_and_owned_pending_values() {
    success(
        &Fixture::new().run(NESTED, 0, &[]),
        "DirectPair{12, \"retained\"}\n",
    );
}

#[test]
fn reused_tail_calls_still_charge_the_execution_budget() {
    let output = Fixture::new().run(PERMUTATION, 0, &["BEND_MAX_STEPS=100"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"teamy-bend executable C: evaluation budget exhausted\n"
    );
}
