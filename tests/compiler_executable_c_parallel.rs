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
const BOUNDS: &[&str] = &[
    "BEND_MAX_DEPTH=32",
    "BEND_MAX_FRAMES=32",
    "BEND_MAX_ALLOC=1048576",
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-parallel-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(
        &self,
        source: &str,
        foreign: &str,
        workers: u32,
        overlap: bool,
        extra: &[&str],
    ) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert!(
            generated.contains("tb_c_word_join(e,") || generated.contains("tb_c_join(e,"),
            "fixture must generate a real fork"
        );
        let fid = if overlap { overlap_fid(&generated) } else { 0 };
        assert_eq!(generated.matches("int main(void)").count(), 1);
        let mut compiled = String::from(HOOK_DECLARATIONS);
        compiled.push_str(&generated.replace("int main(void)", "static int checked_main(void)"));
        compiled.push_str(HOOK_IMPLEMENTATION);
        writeln!(
            compiled,
            r#"
int main(void) {{
  tb_test_main_thread = tb_test_thread();
  tb_test_selected_fid = {fid};
  int status = checked_main();
#if BEND_TEST_REPEAT_RUNTIME
  if (status == 0) {{
    if (tb_cpu_live_workers != 0 || tb_live_words != 0 || tb_live_blocks != 0 ||
        tb_tasks != 0 || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 94;
    status = tb_run(tb_entry, 0, tb_show_main, tb_test_initialize_and_evaluate);
  }}
#endif
  if (tb_cpu_live_workers != 0) {{
    (void)fputs("CPU workers survived runtime shutdown\n", stderr);
    return 90;
  }}
#if BEND_TEST_REQUIRE_DIRECT_REUSE
  if (tb_test_worker_reused < 128) {{
    (void)fputs("worker failure did not exercise repeated direct self calls\n", stderr);
    return 95;
  }}
#endif
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0 ||
      tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {{
    (void)fputs("parallel program retained owners or pending work\n", stderr);
    return 91;
  }}
  if (status == 0 && tb_task_joins == 0) {{
    (void)fputs("parallel source did not execute a task join\n", stderr);
    return 92;
  }}
  if (status == 0 && {fid} != 0 && !tb_test_overlap_ok({workers})) {{
    (void)fputs("generated sibling bodies did not overlap on distinct worker threads\n", stderr);
    return 93;
  }}
  return status;
}}
"#
        )
        .unwrap();
        let worker_definition = format!("BEND_CPU_WORKERS={workers}");
        let mut definitions = BOUNDS
            .iter()
            .copied()
            .filter(|default| {
                !extra
                    .iter()
                    .any(|value| value.split('=').next() == default.split('=').next())
            })
            .collect::<Vec<_>>();
        definitions.push(&worker_definition);
        definitions.extend_from_slice(extra);
        let executable = executable_c_compiler::compile(&self.0, &compiled, &definitions);
        executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(30),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

// The selected source function is the only three-word/two-word segment. Resolve
// its emitted ID so this probe does not depend on incidental numbering.
fn overlap_fid(generated: &str) -> u32 {
    let matches = generated
        .lines()
        .filter_map(|line| line.trim().strip_prefix("tb_register_segment("))
        .filter_map(|line| {
            let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
            (fields.len() == 5 && fields[2] == "3" && fields[3] == "2")
                .then(|| fields[0].parse::<u32>().unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "overlap source needs one unique segment signature"
    );
    matches[0]
}

fn success(output: &Output, expected: &str, workers: u32) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "workers={workers}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes(), "workers={workers}");
    assert!(output.stderr.is_empty());
}

fn failure(output: &Output, diagnostic: &str) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("teamy-bend executable C: {diagnostic}\n")
    );
}

const HOOK_DECLARATIONS: &str = r"
static void tb_test_resume_enter(unsigned fid, const void *frame);
static void tb_test_resume_leave(unsigned fid, const void *frame);
static int tb_test_on_coordinator(void);
static void tb_test_direct_call(unsigned fid, int reused);
#define TB_CPU_RESUME_ENTER(fid, frame) tb_test_resume_enter((fid), (frame))
#define TB_CPU_RESUME_LEAVE(fid, frame) tb_test_resume_leave((fid), (frame))
#define TB_DIRECT_CALL(fid, reused) tb_test_direct_call((unsigned)(fid), (reused))
";

const HOOK_IMPLEMENTATION: &str = r#"
#ifndef BEND_TEST_REPEAT_RUNTIME
#define BEND_TEST_REPEAT_RUNTIME 0
#endif
#ifndef BEND_TEST_REQUIRE_DIRECT_REUSE
#define BEND_TEST_REQUIRE_DIRECT_REUSE 0
#endif
#ifdef _WIN32
typedef DWORD TBTestThread;
static TBTestThread tb_test_thread(void) { return GetCurrentThreadId(); }
static bool tb_test_same_thread(TBTestThread a, TBTestThread b) { return a == b; }
static u64 tb_test_millis(void) { return (u64)GetTickCount64(); }
static void tb_test_yield(void) { Sleep(1); }
#else
typedef pthread_t TBTestThread;
static TBTestThread tb_test_thread(void) { return pthread_self(); }
static bool tb_test_same_thread(TBTestThread a, TBTestThread b) { return pthread_equal(a, b) != 0; }
static u64 tb_test_millis(void) {
  struct timespec time;
  (void)clock_gettime(CLOCK_MONOTONIC, &time);
  return (u64)time.tv_sec * 1000 + (u64)time.tv_nsec / 1000000;
}
static void tb_test_yield(void) { struct timespec delay = {0, 1000000}; (void)nanosleep(&delay, NULL); }
#endif
static TBTestThread tb_test_main_thread, tb_test_threads[2];
static TBMutex tb_test_mutex = TB_MUTEX_INIT;
static u32 tb_test_selected_fid, tb_test_arrivals, tb_test_mask, tb_test_active, tb_test_peak;
static u32 tb_test_worker_reused;
static TB_THREAD_LOCAL const void *tb_test_entered;
static int tb_test_on_coordinator(void) {
  return tb_test_same_thread(tb_test_thread(), tb_test_main_thread);
}
static void tb_test_direct_call(unsigned fid, int reused) {
  (void)fid;
  if (!reused || tb_test_on_coordinator()) return;
  tb_lock(&tb_test_mutex);
  ++tb_test_worker_reused;
  tb_unlock(&tb_test_mutex);
}
static void tb_test_resume_enter(unsigned fid, const void *pointer) {
  const TBCallFrame *frame = (const TBCallFrame *)pointer;
  u32 marker;
  u64 deadline;
  if (fid != tb_test_selected_fid || frame->pc != 0) return;
  marker = (u32)frame->captures[0];
  if (marker < 1 || marker > 2 || tb_test_on_coordinator()) err_fail("invalid overlap worker entry");
  tb_lock(&tb_test_mutex);
  if ((tb_test_mask & (1u << marker)) != 0) {
    tb_unlock(&tb_test_mutex);
    err_fail("generated sibling entry repeated");
  }
  tb_test_mask |= 1u << marker;
  tb_test_threads[marker - 1] = tb_test_thread();
  ++tb_test_arrivals;
  ++tb_test_active;
  if (tb_test_active > tb_test_peak) tb_test_peak = tb_test_active;
  tb_test_entered = pointer;
  tb_unlock(&tb_test_mutex);
  deadline = tb_test_millis() + 5000;
  for (;;) {
    u32 arrived;
    tb_lock(&tb_test_mutex);
    arrived = tb_test_arrivals;
    tb_unlock(&tb_test_mutex);
    if (arrived == 2) break;
    if (tb_test_millis() >= deadline) err_fail("generated sibling rendezvous timed out");
    tb_test_yield();
  }
}
static void tb_test_resume_leave(unsigned fid, const void *pointer) {
  (void)fid;
  if (tb_test_entered != pointer) return;
  tb_lock(&tb_test_mutex);
  --tb_test_active;
  tb_test_entered = NULL;
  tb_unlock(&tb_test_mutex);
}
static bool tb_test_overlap_ok(u32 workers) {
  if (workers == 1) return tb_test_arrivals == 0;
  return tb_test_arrivals == 2 && tb_test_mask == 6 && tb_test_peak == 2 && tb_test_active == 0 &&
    !tb_test_same_thread(tb_test_threads[0], tb_test_threads[1]);
}
#if BEND_TEST_REPEAT_RUNTIME
static void tb_test_initialize_and_evaluate(Env e) {
  Term value;
  tb_initialize(e);
  value = tb_entry(e);
  tb_show_main(e, value);
  term_sink(e, value);
  if (tb_cpu_live_workers != 0 || tb_task_joins == 0)
    err_fail("initializer dispatched through stale CPU pool state");
}
#endif
"#;

const OVERLAP: &str = r"import Base
type WorkerPair is Data: WorkerPair{count: U32, seed: U32}
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def probe(marker: U32, n: Nat, seed: U32) -> WorkerPair:
  WorkerPair{U32.add(marker, count(n)), seed}
def main() -> WorkerPair & WorkerPair:
  left right = probe(1, 48n, 11) probe(2, 64n, 22)
  (left, right)
";

#[test]
fn generated_sibling_bodies_overlap_on_distinct_worker_threads() {
    for workers in [1, 2, 4] {
        success(
            &Fixture::new().run(OVERLAP, "", workers, true, &[]),
            "(WorkerPair{49, 11}, WorkerPair{66, 22})\n",
            workers,
        );
    }
}

const SHARED_TREE: &str = r#"import Base
type SharedTree is Data:
  Leaf{text: String}
  Branch{left: SharedTree, right: SharedTree}
def size(tree: SharedTree) -> U32:
  match tree:
    case Leaf{text}: U32.from_nat(String.length(text))
    case Branch{left, right}: U32.add(size(left), size(right))
def rounds(n: Nat, +tree: SharedTree, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p:
      left right = size(tree) size(tree)
      rounds(p, tree, U32.add(total, U32.add(left, right)))
def main() -> U32:
  +text = String.append("ro", "ot")
  +leaf = {Leaf{text} : SharedTree}
  rounds(64n, Branch{leaf, Branch{leaf, Leaf{"tail"}}}, 0)
"#;

#[test]
fn shared_constructor_descendants_survive_repeated_parallel_takes_and_reuse() {
    for workers in [1, 2, 4] {
        success(
            &Fixture::new().run(SHARED_TREE, "", workers, false, &["BEND_MAX_ALLOC=16384"]),
            "1536\n",
            workers,
        );
    }
}

const NESTED_FORKS: &str = r"import Base
def identity(value: U32) -> U32: value
def nested(n: Nat, seed: U32) -> U32:
  match n:
    case 0n: seed
    case 1n+p:
      left right = nested(p, seed) identity(1)
      U32.add(left, right)
def main() -> U32 & U32:
  left right = nested(Nat.mul(64n, 4n), 7) nested(Nat.mul(64n, 3n), 9)
  (left, right)
";

#[test]
fn nested_parallel_forks_resume_with_small_native_stacks() {
    for workers in [1, 2, 4] {
        success(
            &Fixture::new().run(NESTED_FORKS, "", workers, false, &[]),
            "(263, 201)\n",
            workers,
        );
    }
}

const FOUR_CHILDREN: &str = r"import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32 & U32 & U32 & U32:
  a b c d = count(64n) count(65n) count(66n) count(67n)
  (a, b, c, d)
";

#[test]
fn partial_worker_creation_failure_joins_already_started_workers() {
    failure(
        &Fixture::new().run(
            FOUR_CHILDREN,
            "",
            4,
            false,
            &["BEND_CPU_TEST_SPAWN_FAIL_AFTER=1"],
        ),
        "CPU worker creation failed",
    );
}

#[test]
fn repeated_runtime_invocation_can_evaluate_generated_forks_before_pool_initialization() {
    success(
        &Fixture::new().run(FOUR_CHILDREN, "", 2, false, &["BEND_TEST_REPEAT_RUNTIME=1"]),
        "(64, 65, 66, 67)\n(64, 65, 66, 67)\n(64, 65, 66, 67)\n",
        2,
    );
}

const DIVERGENT_CHILD: &str = r"import Base
@unsafe
def spin(n: Nat) -> U32: spin(n)
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32 & U32:
  left right = spin(0n) count(Nat.mul(64n, 4n))
  (left, right)
";

#[test]
fn worker_budget_failure_cancels_and_joins_the_other_children() {
    for workers in [2, 4] {
        failure(
            &Fixture::new().run(
                DIVERGENT_CHILD,
                "",
                workers,
                false,
                &["BEND_MAX_STEPS=4096", "BEND_TEST_REQUIRE_DIRECT_REUSE=1"],
            ),
            "evaluation budget exhausted",
        );
        failure(
            &Fixture::new().run(NESTED_FORKS, "", workers, false, &["BEND_MAX_TASKS=32"]),
            "task budget exhausted",
        );
    }
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
static u32 staged_trace, completed_mask;
static Term stage_apply(Env e, const Term *captures, Term argument) {
  u32 marker = (u32)captures[0];
  (void)e;
  if (!tb_test_on_coordinator()) err_fail("foreign callback ran on a CPU worker");
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
  if (!tb_test_on_coordinator()) err_fail("foreign callback ran on a CPU worker");
  if (staged_trace != 2 || marker < 3 || marker > 4 || (completed_mask & (1u << (marker - 3))) != 0)
    err_fail("task execution preceded staging or repeated a callback");
  completed_mask |= 1u << (marker - 3);
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
  if (!tb_test_on_coordinator()) err_fail("foreign effect ran on a CPU worker");
  function = tb_apply(e, function, tb_closure(e, 62000, 1, stage1));
  function = tb_apply(e, function, tb_closure(e, 62000, 1, stage2));
  function = tb_apply(e, function, tb_closure(e, 62001, 3, first));
  result = tb_apply(e, function, tb_closure(e, 62001, 3, second));
  if (staged_trace != 2 || completed_mask != 3) err_fail("task callback was omitted or repeated");
  return result;
}
static void __attribute__((constructor)) drive_use(void) {
  tb_register_closure(62000, stage_apply, 1);
  tb_register_closure(62001, work_apply, 3);
  io_eff(CID_DRIVE, drive_run, 0);
}
"#;

#[test]
fn foreign_callbacks_stay_on_coordinator_and_reenter_nested_generated_forks() {
    let resumable = STAGED_REENTRY_C
        .replace("static Term drive_run", &format!("{FOREIGN_RESUME_C}\nstatic Term drive_run"))
        .replace(
            "tb_register_closure(62001, work_apply, 3);",
            "tb_register_generated(62001, work_apply, foreign_resume, 3, 1);",
        )
        .replace(
            "if (staged_trace != 2 || completed_mask != 3)",
            "if (staged_trace != 2 || completed_mask != 3 || foreign_resumes != 2 || foreign_segments != 2)",
        );
    for workers in [1, 2, 4] {
        for foreign in [STAGED_REENTRY_C, &resumable] {
            success(
                &Fixture::new().run(STAGED_REENTRY, foreign, workers, false, &[]),
                "stage1\nstage2\n141\n",
                workers,
            );
        }
    }
}

const FOREIGN_RESUME_C: &str = r#"
static u32 foreign_resumes, foreign_segments;
static TBOutcome foreign_segment(const Env *e, TBCallFrame *frame) {
  (void)e;
  if (!tb_test_on_coordinator()) err_fail("foreign segment ran on a CPU worker");
  ++foreign_segments;
  frame->values[0] = frame->captures[0];
  return tb_segment_words(frame->values, NULL, 1);
}
static Term foreign_resume(const Env *e, TBCallFrame *frame) {
  Term words[1];
  if (!tb_test_on_coordinator()) err_fail("foreign resume ran on a CPU worker");
  ++foreign_resumes;
  words[0] = work_apply(*e, frame->captures, frame->argument);
  /* Reentry has finished and sibling workers must have drained before an
   * untrusted callback may register another untrusted callback. */
  tb_register_segment(62002, foreign_segment, 1, 1, 1);
  return tb_word_task(*e, 62002, 1, words, NULL);
}
"#;

const DIRECT_HANDOFF: &str = r#"import Base
def probe(seed: U32) -> IO(Unit): import "effect.c"
def identity(value: U32) -> U32: value
def main() -> IO(Unit):
  left right = identity(1) identity(2)
  do IO<Unit>:
    Unit <- probe(U32.add(left, right))
    IO.print("ok")
"#;

const DIRECT_HANDOFF_C: &str = r#"
static const Term handoff_pair_owned[2] = {0, 1};
static const Term handoff_join_owned[4] = {0, 1, 0, 1};
#if BEND_CPU_WORKERS > 1
static TBMutex handoff_mutex = TB_MUTEX_INIT;
static u32 handoff_workers;
#endif
static u32 handoff_foreign;
static TBOutcome handoff_bridge(const Env *e, TBCallFrame *frame) {
  u32 marker = (u32)frame->captures[0];
  (void)e;
  if (marker < 1 || marker > 2) err_fail("direct handoff lost raw argument");
#if BEND_CPU_WORKERS > 1
  if (tb_test_on_coordinator()) err_fail("direct handoff never entered a worker");
  tb_lock(&handoff_mutex);
  handoff_workers |= 1u << marker;
  tb_unlock(&handoff_mutex);
#else
  if (!tb_test_on_coordinator()) err_fail("serial handoff entered a worker");
#endif
  /* Captures are borrowed from the source frame. The dispatcher must copy
   * them before freeing it, then reconsider the target's worker eligibility. */
  return tb_segment_call(62001, 2, frame->captures, handoff_pair_owned);
}
static TBOutcome handoff_foreign_segment(const Env *e, TBCallFrame *frame) {
  u32 marker = (u32)frame->captures[0];
  (void)e;
  if (!tb_test_on_coordinator() || tb_cpu_pending != 0)
    err_fail("direct foreign segment ran before CPU workers drained");
  if (marker < 1 || marker > 2 || (handoff_foreign & (1u << marker)) != 0)
    err_fail("direct foreign segment repeated or lost arguments");
  handoff_foreign |= 1u << marker;
  return tb_segment_words(frame->captures, handoff_pair_owned, 2);
}
static TBOutcome handoff_join(const Env *e, TBCallFrame *frame) {
  (void)e;
  if (!tb_test_on_coordinator()) err_fail("unmarked direct join ran on a CPU worker");
  return tb_segment_words(frame->captures, handoff_join_owned, 4);
}
static TBOutcome handoff_launch(const Env *e, TBCallFrame *frame) {
  Term first[2] = {1, io_str(*e, "first", 5)};
  Term second[2] = {2, io_str(*e, "second", 6)};
  Term children[2];
  (void)frame;
  children[0] = tb_word_task(*e, 62000, 2, first, handoff_pair_owned);
  children[1] = tb_word_task(*e, 62000, 2, second, handoff_pair_owned);
  return tb_segment_task(tb_c_word_join(e, 62002, 0, NULL, NULL, 2, children));
}
static void handoff_check_text(Env e, Term value, const char *expected) {
  u64 length;
  char *text = io_cstr(e, value, &length);
  if (length != strlen(expected) || memcmp(text, expected, (size_t)length) != 0)
    err_fail("direct handoff lost owned text");
  free(text);
}
static Term probe_run(Env e, Term *fields, IoWork *work) {
  Term result[4], owned[4], entry;
  u32 count;
  (void)work;
  if (!tb_test_on_coordinator() || fields[0] != 3) err_fail("invalid direct handoff setup");
  entry = tb_word_task(e, 62003, 0, NULL, NULL);
  count = corpus_eval_words(e.mem, entry, result, owned, 4);
  if (count != 4 || result[0] != 1 || result[2] != 2 || handoff_foreign != 6)
    err_fail("direct handoff did not preserve both sibling results");
#if BEND_CPU_WORKERS > 1
  if (handoff_workers != 6) err_fail("direct handoff did not run both siblings on workers");
#endif
  for (u32 i = 0; i < 4; ++i)
    if (owned[i] != handoff_join_owned[i]) err_fail("direct handoff changed result ownership");
  handoff_check_text(e, result[1], "first");
  handoff_check_text(e, result[3], "second");
  return term_pak(CID_UNIT, 0);
}
static void __attribute__((constructor)) probe_use(void) {
  tb_register_segment(62000, handoff_bridge, 2, 2, 0);
  tb_register_parallel(62000);
  tb_register_segment(62001, handoff_foreign_segment, 2, 2, 0);
  tb_register_segment(62002, handoff_join, 4, 4, 0);
  tb_register_segment(62003, handoff_launch, 0, 4, 0);
  io_eff(CID_PROBE, probe_run, 0);
}
"#;

#[test]
fn direct_worker_calls_recheck_foreign_eligibility_and_preserve_owned_results() {
    for workers in [1, 2, 4] {
        success(
            &Fixture::new().run(DIRECT_HANDOFF, DIRECT_HANDOFF_C, workers, false, &[]),
            "ok\n",
            workers,
        );
    }
}

// This deliberately exercises native reference-counted copy-on-write under the
// explicit unsafe quantity relaxation. Upstream JavaScript mutates aliased
// arrays here, so its result is not a parity oracle for this ownership stress.
const SHARED_ARRAY_CLOSURE: &str = r#"import Base
type SharedCell is Data:
  RawCell{number: Nat}
  TextCell{text: String}
type SharedState is Type:
  SharedState{arrays: Array<Array<SharedCell>>, function: Unit -> String}
type WorkerResult is Type:
  WorkerResult{text: String, old: SharedCell, changed: List<SharedCell>, original: List<SharedCell>}
def finish(pair: Array<SharedCell> & SharedCell, original: Array<SharedCell>, function: Unit -> String) -> WorkerResult:
  (changed, old) = pair
  WorkerResult{function(Unit{}), old, Array.to_list(~SharedCell, changed), Array.to_list(~SharedCell, original)}
def copied(pair: Array<SharedCell> & Array<SharedCell>, function: Unit -> String, label: String) -> WorkerResult:
  (left, right) = pair
  finish(Array.swap(SharedCell, left, 0, TextCell{label}), right, function)
def extracted(pair: Array<Array<SharedCell>> & Array<SharedCell>, function: Unit -> String, label: String) -> WorkerResult:
  (outer, inner) = pair
  copied(Array.clone(SharedCell, inner), function, label)
def inspect(state: SharedState, label: String) -> WorkerResult:
  match state:
    case SharedState{arrays, function}:
      extracted(Array.swap(Array<SharedCell>, arrays, 0, ALeaf{RawCell{0n}}), function, label)
@unsafe
def both(+state: SharedState) -> WorkerResult & WorkerResult:
  left right = inspect(state, "left") inspect(state, "right")
  (left, right)
def main() -> WorkerResult & WorkerResult:
  text = String.append("hel", "d")
  both(SharedState{ALeaf{ANode{ALeaf{RawCell{Nat.mul(Nat.pow(2n, 20n), Nat.pow(2n, 20n))}},
    ALeaf{TextCell{String.append("ro", "ot")}}}}, ignored => String.append(text, "!")})
"#;

#[test]
fn shared_closure_and_nested_array_views_keep_raw_words_and_private_updates() {
    for workers in [1, 2, 4] {
        success(
            &Fixture::new().run(
                SHARED_ARRAY_CLOSURE,
                "",
                workers,
                false,
                &["BEND_MAX_ALLOC=16384"],
            ),
            "(WorkerResult{\"held!\", RawCell{1099511627776n}, [TextCell{\"left\"}, TextCell{\"root\"}], [RawCell{1099511627776n}, TextCell{\"root\"}]}, WorkerResult{\"held!\", RawCell{1099511627776n}, [TextCell{\"right\"}, TextCell{\"root\"}], [RawCell{1099511627776n}, TextCell{\"root\"}]})\n",
            workers,
        );
    }
}
