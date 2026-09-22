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
const SOURCE: &str = r#"import Base
def probe() -> IO(Unit): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- probe()
    IO.print("ok")
"#;
const SMALL_STACK: &[&str] = &[
    "BEND_MAX_DEPTH=32",
    "BEND_MAX_FRAMES=32",
    "BEND_MAX_ALLOC=65536",
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-segment-abi-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, foreign: &str, definitions: &[&str]) -> Output {
        self.run_with_prelude(foreign, definitions, "")
    }

    fn run_with_prelude(&self, foreign: &str, definitions: &[&str], prelude: &str) -> Output {
        fs::write(self.0.join("main.bend"), SOURCE).unwrap();
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
        let mut generated = format!(
            "{prelude}\n{}",
            generated.replace(
                "static int tb_program_main(void)",
                "#define TB_NO_MAIN 1\nstatic int checked_main(void)"
            )
        );
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {
    (void)fputs("segment ABI program leaked owners or active work\n", stderr);
    return 91;
  }
  return status;
}
"#,
        );
        let definitions = [SMALL_STACK, definitions].concat();
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

fn success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ok\n");
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

fn probe(callbacks: &str, body: &str, registrations: &str) -> String {
    let mut source = String::from(callbacks);
    source.push_str(
        "\nstatic Term probe_run(Env e, Term *fields, IoWork *work) {\n  (void)fields; (void)work;\n",
    );
    source.push_str(body);
    source.push_str(
        "\n  return term_pak(CID_UNIT, 0);\n}\nstatic void __attribute__((constructor)) probe_use(void) {\n",
    );
    source.push_str(registrations);
    source.push_str("\n  io_eff(CID_PROBE, probe_run, 0);\n}\n");
    source
}

const TEXT_CHECK: &str = r#"
static void check_text(Env e, Term value, const char *expected) {
  u64 length;
  char *bytes = io_cstr(e, value, &length);
  if (length != strlen(expected) || memcmp(bytes, expected, (size_t)length) != 0)
    err_fail("unexpected task text");
  free(bytes);
}
"#;

const RAW_PACKET: &str = r"
static const Term packet_owned[3] = {0, 0, 1};
static u32 packet_calls;
static TBOutcome echo_packet(const Env *e, TBCallFrame *frame) {
  (void)e;
  ++packet_calls;
  return tb_segment_words(frame->captures, packet_owned, 3);
}
";

#[test]
fn raw_task_and_hole_bits_round_trip_without_becoming_owners_or_control_flow() {
    let foreign = probe(
        &(String::from(TEXT_CHECK) + RAW_PACKET),
        r#"
  Term input[3] = {term_tsk(65535, 123), TERM_HOLE, io_str(e, "owned", 5)};
  Term result[3], result_owned[3];
  Term task = tb_word_task(e, 62000, 3, input, packet_owned);
  u32 count = corpus_eval_words(e.mem, task, result, result_owned, 3);
  if (count != 3 || packet_calls != 1 || result[0] != input[0] || result[1] != TERM_HOLE
      || result_owned[0] != 0 || result_owned[1] != 0 || result_owned[2] != 1)
    err_fail("raw segment packet was interpreted as a term");
  check_text(e, result[2], "owned");
  input[2] = io_str(e, "discarded", 9);
  term_drop(e, tb_word_task(e, 62000, 3, input, packet_owned));
  if (packet_calls != 1) err_fail("dropping a word task executed it");
"#,
        "  tb_register_segment(62000, echo_packet, 3, 3, 0);",
    );
    success(&Fixture::new().run(&foreign, &[]));
    let too_small = foreign.replace("result, result_owned, 3)", "result, result_owned, 2)");
    failure(
        &Fixture::new().run(&too_small, &[]),
        "task result buffer is too small",
    );
    let invalid_mask =
        foreign.replace("packet_owned[3] = {0, 0, 1}", "packet_owned[3] = {0, 2, 1}");
    failure(
        &Fixture::new().run(&invalid_mask, &[]),
        "invalid task ownership mask",
    );
}

const INVALID_DIRECT_CALL: &str = r#"
static const Term direct_raw_one[1] = {0};
static TBOutcome unexpected_direct_target(const Env *e, TBCallFrame *frame) {
  (void)e; (void)frame;
  err_fail("direct target executed before call validation");
}
static TBOutcome malformed_direct_call(const Env *e, TBCallFrame *frame) {
  TBOutcome outcome;
  (void)e;
  frame->values[0] = 41;
  outcome = tb_segment_call(62001, 1, frame->values, direct_raw_one);
  /* mutate call */
  return outcome;
}
"#;

#[test]
fn malformed_direct_calls_fail_before_entering_the_target() {
    let foreign = probe(
        INVALID_DIRECT_CALL,
        "  (void)corpus_eval(e.mem, tb_word_task(e, 62000, 0, NULL, NULL));",
        "  tb_register_segment(62000, malformed_direct_call, 0, 1, 2);\n  tb_register_segment(62001, unexpected_direct_target, 1, 1, 0);\n  tb_register_segment(62003, unexpected_direct_target, 1, 2, 0);",
    );
    for (mutation, diagnostic) in [
        ("outcome.pending = 3;", "invalid segment outcome tag"),
        (
            "outcome.task = (UINT64_C(1) << 32) | 62001;",
            "invalid direct segment call",
        ),
        ("outcome.task = 65536;", "invalid direct segment call"),
        ("outcome.task = 62002;", "invalid direct segment call"),
        ("outcome.count = 0;", "invalid direct segment arguments"),
        ("outcome.words = NULL;", "invalid direct segment arguments"),
        ("outcome.task = 62003;", "task result width mismatch"),
        (
            "frame->values[1] = 2; outcome.owned = frame->values + 1;",
            "invalid task ownership mask",
        ),
        (
            "frame->values[0] = TERM_HOLE; outcome.owned = NULL;",
            "foreign task argument is missing",
        ),
        (
            "frame->values[0] = term_tsk(65535, 123); outcome.owned = NULL;",
            "runnable task contains a pending task",
        ),
        (
            "frame->pc = 1; frame->expected = 1; frame->waiting = true; outcome.task = 62003;",
            "task result width mismatch",
        ),
    ] {
        failure(
            &Fixture::new().run(&foreign.replace("/* mutate call */", mutation), &[]),
            diagnostic,
        );
    }
}

const DIRECT_REUSE_PRELUDE: &str = r"
static void record_direct_call(unsigned int fid, int reused);
#define TB_DIRECT_CALL(fid, reused) record_direct_call((fid), (reused))
";

const DIRECT_REUSE: &str = r#"
static const Term direct_packet_owned[3] = {0, 0, 1};
static const Term direct_loop_owned[4] = {0, 0, 0, 1};
static u32 direct_calls, reused_calls;
static void record_direct_call(unsigned int fid, int reused) {
  if (fid >= 62000 && fid <= 62002) {
    ++direct_calls;
    if (reused) {
      if (fid != 62001) err_fail("unexpected direct frame reuse");
      ++reused_calls;
    }
  }
}
static TBOutcome direct_entry(const Env *e, TBCallFrame *frame) {
  (void)e;
  return tb_segment_call(62001, 4, frame->captures + 1, direct_loop_owned);
}
static TBOutcome direct_permute(const Env *e, TBCallFrame *frame) {
  (void)e;
  for (u32 i = 0; i < 8; ++i) {
    if (frame->values[i] != 0) err_fail("reused frame retained stale scratch words");
  }
  if (frame->pc != 0 || frame->destination != 0 || frame->expected != 1
      || frame->argument != 0 || frame->waiting || frame->parent != NULL
      || frame->tail_result != TB_RESULT_NONE || frame->saved_result != TB_RESULT_NONE)
    err_fail("reused frame retained stale continuation state");
  if (frame->captures[0] == 0)
    return tb_segment_call(62002, 3, frame->captures + 1, direct_packet_owned);
  frame->values[0] = frame->captures[0] - 1;
  frame->values[1] = frame->captures[3];
  frame->values[2] = frame->captures[2];
  frame->values[3] = frame->captures[1];
  frame->values[4] = 0;
  frame->values[5] = frame->values[0] & 1;
  frame->values[6] = 0;
  frame->values[7] = 1 - frame->values[5];
  frame->pc = 17; frame->destination = 7; frame->expected = 3;
  return tb_segment_call(62001, 4, frame->values, frame->values + 4);
}
static TBOutcome direct_packet(const Env *e, TBCallFrame *frame) {
  (void)e;
  return tb_segment_words(frame->captures, direct_packet_owned, 3);
}
"#;

#[test]
fn direct_self_tail_calls_reuse_frames_and_preserve_aliased_raw_and_owned_arguments() {
    let foreign = probe(
        &(String::from(TEXT_CHECK) + DIRECT_REUSE),
        r#"
  const Term input_owned[5] = {0, 0, 0, 0, 1};
  Term input[5] = {99, 10000, term_tsk(65535, 123), TERM_HOLE, io_str(e, "owned", 5)};
  Term result[3], owned[3];
  direct_calls = 0; reused_calls = 0;
  u32 count = corpus_eval_words(e.mem, tb_word_task(e, 62000, 5, input, input_owned), result, owned, 3);
  if (count != 3 || result[0] != input[2] || result[1] != TERM_HOLE
      || memcmp(owned, direct_packet_owned, sizeof(owned)) != 0)
    err_fail("direct argument transfer changed raw bits or ownership");
  if (direct_calls != 10002 || reused_calls != 10000)
    err_fail("self-tail calls did not reuse their dispatcher frame");
  check_text(e, result[2], "owned");
"#,
        "  tb_register_segment(62000, direct_entry, 5, 3, 0);\n  tb_register_segment(62001, direct_permute, 4, 3, 8);\n  tb_register_segment(62002, direct_packet, 3, 3, 0);",
    );
    success(&Fixture::new().run_with_prelude(
        &foreign,
        &["BEND_MAX_CONTINUATIONS=8"],
        DIRECT_REUSE_PRELUDE,
    ));
}

const REORDERED_PACKETS: &str = r#"
static const Term pair_owned[2] = {0, 1};
static const Term join_owned[5] = {1, 0, 1, 0, 1};
static u32 completion_order;
static TBOutcome slow_step(const Env *e, TBCallFrame *frame) {
  (void)e;
  if (completion_order != 1) err_fail("slow result preceded the ready sibling");
  completion_order = 2;
  return tb_segment_words(frame->captures, pair_owned, 2);
}
static TBOutcome slow_text(const Env *e, TBCallFrame *frame) {
  if (completion_order != 1) err_fail("queued child preceded the ready sibling");
  frame->values[0] = io_str(*e, "slow", 4);
  return tb_segment_words(frame->values, NULL, 1);
}
static TBOutcome slow_pair(const Env *e, TBCallFrame *frame) {
  if (frame->pc == 0) {
    Loc continuation = task_node(*e, 62004, TERM_HOLE, 0, 1);
    Loc child = task_node(*e, 62005, term_tsk(62004, continuation), 1, 0);
    e->mem[continuation] = 10;
    tb_mark_raw(*e, continuation, 1);
    frame->pc = 1; frame->destination = 0; frame->expected = 2; frame->waiting = true;
    return tb_segment_task(term_tsk(62005, child));
  }
  if (frame->pc != 1) err_fail("unexpected slow segment resume");
  return tb_segment_words(frame->values, pair_owned, 2);
}
static TBOutcome fast_pair(const Env *e, TBCallFrame *frame) {
  if (completion_order != 0) err_fail("fast sibling completed twice");
  completion_order = 1;
  frame->values[0] = 20;
  frame->values[1] = io_str(*e, "fast", 4);
  return tb_segment_words(frame->values, pair_owned, 2);
}
static TBOutcome joined_pairs(const Env *e, TBCallFrame *frame) {
  (void)e;
  if (completion_order != 2) err_fail("join resumed before both result spans arrived");
  return tb_segment_words(frame->captures, join_owned, 5);
}
static TBOutcome launch_pairs(const Env *e, TBCallFrame *frame) {
  Term held = io_str(*e, "held", 4);
  Term children[2];
  (void)frame;
  children[0] = tb_word_task(*e, 62000, 0, NULL, NULL);
  children[1] = tb_word_task(*e, 62001, 0, NULL, NULL);
  return tb_segment_task(tb_c_word_join(e, 62002, 1, &held, NULL, 2, children));
}
"#;

#[test]
fn returned_multiword_joins_deliver_complete_spans_out_of_order_and_resume_once() {
    let foreign = probe(
        &(String::from(TEXT_CHECK) + REORDERED_PACKETS),
        r#"
  Term result[5], owned[5];
  u32 count = corpus_eval_words(e.mem, tb_word_task(e, 62003, 0, NULL, NULL), result, owned, 5);
  if (count != 5 || result[1] != 10 || result[3] != 20 || completion_order != 2
      || memcmp(owned, join_owned, sizeof(owned)) != 0)
    err_fail("multiword join changed a result span or its ownership");
  check_text(e, result[0], "held");
  check_text(e, result[2], "slow");
  check_text(e, result[4], "fast");
"#,
        "  tb_register_segment(62000, slow_pair, 0, 2, 2);\n  tb_register_segment(62001, fast_pair, 0, 2, 2);\n  tb_register_segment(62002, joined_pairs, 5, 5, 0);\n  tb_register_segment(62003, launch_pairs, 0, 5, 0);\n  tb_register_segment(62004, slow_step, 2, 2, 0);\n  tb_register_segment(62005, slow_text, 0, 1, 1);",
    );
    success(&Fixture::new().run(&foreign, &[]));
}

const INVALID_SPANS: &str = r#"
#ifndef PARENT_ARITY
#define PARENT_ARITY 4
#endif
#ifndef PARENT_WIDTH
#define PARENT_WIDTH 1
#endif
#ifndef OVERLAPPING_SPANS
#define OVERLAPPING_SPANS 0
#endif
static TBOutcome unexpected_child(const Env *e, TBCallFrame *frame) {
  (void)e; (void)frame;
  err_fail("child callback executed before graph validation");
}
static TBOutcome unexpected_join(const Env *e, TBCallFrame *frame) {
  (void)e; (void)frame;
  err_fail("join callback executed before graph validation");
}
static TBOutcome invalid_graph(const Env *e, TBCallFrame *frame) {
  Loc at = task_node(*e, 62002, TERM_HOLE, 0, 2);
  Term parent = term_tsk(62002, at);
  Loc first = task_node(*e, 62000, parent, 0, 0);
  Loc second = task_node(*e, 62000, parent, 2, 0);
  (void)frame;
  e->mem[at] = term_tsk(62000, first);
  e->mem[at + 2] = term_tsk(62000, second);
  if (OVERLAPPING_SPANS) {
    e->mem[at + 1] = e->mem[at + 2];
    e->mem[at + 2] = TERM_HOLE;
    e->mem[task_tail(term_tsk(62000, second)) + 1] = UINT64_C(1) << 32;
  }
  return tb_segment_task(parent);
}
"#;

#[test]
fn invalid_result_widths_and_overlapping_spans_are_rejected_before_callbacks() {
    let foreign = probe(
        INVALID_SPANS,
        "  (void)corpus_eval(e.mem, tb_word_task(e, 62001, 0, NULL, NULL));",
        "  tb_register_segment(62000, unexpected_child, 0, 2, 0);\n  tb_register_segment(62001, invalid_graph, 0, 1, 0);\n  tb_register_segment(62002, unexpected_join, PARENT_ARITY, PARENT_WIDTH, 0);",
    );
    for (definition, diagnostic) in [
        ("PARENT_WIDTH=2", "task result width mismatch"),
        ("OVERLAPPING_SPANS=1", "task result spans overlap"),
        ("PARENT_ARITY=3", "task result span is out of bounds"),
    ] {
        failure(&Fixture::new().run(&foreign, &[definition]), diagnostic);
    }
}

#[test]
fn boxed_generated_closures_keep_all_255_captures_across_private_resumption() {
    let callbacks = String::from(TEXT_CHECK)
        + r#"
static u32 capture_resumes;
static Term boxed_child(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  return argument + 1;
}

static Term captured_legacy(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures; (void)argument;
  err_fail("private resume unexpectedly used the legacy callback");
}
static Term captured_resume(const Env *e, TBCallFrame *frame) {
  if (frame->pc == 0) {
    frame->pc = 1; frame->destination = 0; frame->expected = 1; frame->waiting = true;
    return tb_tail_apply(*e, tb_closure(*e, 62000, 0, NULL), 41);
  }
  if (frame->pc != 1 || ++capture_resumes != 1 || frame->values[0] != 42)
    err_fail("private boxed closure resumed incorrectly");
  for (u32 i = 0; i < 255; ++i) check_text(*e, frame->captures[i], "held");
  term_drop(*e, frame->argument);
  return term_pak(CID_UNIT, 0);
}
"#;
    let registrations = "  tb_register_closure(62000, boxed_child, 0);\n  tb_register_generated(62001, captured_legacy, captured_resume, 255, 1);";
    let foreign = probe(
        &callbacks,
        r#"
  Term captures[255];
  for (u32 i = 0; i < 255; ++i) captures[i] = io_str(e, "held", 4);
  term_drop(e, tb_apply(e, tb_closure(e, 62001, 255, captures), term_pak(CID_UNIT, 0)));
  if (capture_resumes != 1) err_fail("private boxed closure was not resumed");
"#,
        registrations,
    );
    success(&Fixture::new().run(&foreign, &[]));
    let oversized_task = probe(
        &callbacks,
        "  (void)task_node(e, 62001, TERM_HOLE, 0, 0);",
        registrations,
    );
    failure(
        &Fixture::new().run(&oversized_task, &[]),
        "foreign task arity is unsupported",
    );
}

const SCALAR_ADAPTERS: &str = r#"
static const Term raw_one[1] = {0};
static Term wide_legacy(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  return argument;
}
static TBOutcome adapt_legacy(const Env *e, TBCallFrame *frame) {
  Term task = tb_tail_apply(*e, tb_closure(*e, 62000, 0, NULL), frame->captures[1]);
  return tb_segment_task(tb_tail_result(frame, task, (u32)frame->captures[0]));
}
static TBOutcome owned_child(const Env *e, TBCallFrame *frame) {
  frame->values[0] = io_str(*e, "child", 5);
  return tb_segment_words(frame->values, NULL, 1);
}
static TBOutcome pending_middle(const Env *e, TBCallFrame *frame) {
  if (frame->pc == 0) {
    frame->pc = 1; frame->destination = 0; frame->expected = 1; frame->waiting = true;
    return tb_segment_task(tb_word_task(*e, 62004, 0, NULL, NULL));
  }
  if (frame->pc != 1) err_fail("scalar adapter changed resume state");
  check_text(*e, frame->values[0], "child");
  frame->values[0] = UINT64_C(0x200000009);
  return tb_segment_words(frame->values, raw_one, 1);
}
static TBOutcome adapt_word(const Env *e, TBCallFrame *frame) {
  Term task = tb_word_task(*e, (Fid)frame->captures[1], 0, NULL, NULL);
  return tb_segment_task(tb_tail_result(frame, task, (u32)frame->captures[0]));
}
static TBOutcome composed_adapter(const Env *e, TBCallFrame *frame) {
  const Term raw_two[2] = {0, 0};
  Term input[2] = {frame->captures[1], UINT64_C(0x100000007)};
  Term task = tb_word_task(*e, 62001, 2, input, raw_two);
  return tb_segment_task(tb_tail_result(frame, task, (u32)frame->captures[0]));
}
static TBOutcome joined_children(const Env *e, TBCallFrame *frame) {
  check_text(*e, frame->captures[0], "child");
  check_text(*e, frame->captures[1], "child");
  frame->values[0] = UINT64_C(0x200000009);
  return tb_segment_words(frame->values, raw_one, 1);
}
static TBOutcome fork_children(const Env *e, TBCallFrame *frame) {
  Term children[2];
  (void)frame;
  children[0] = tb_word_task(*e, 62004, 0, NULL, NULL);
  children[1] = tb_word_task(*e, 62004, 0, NULL, NULL);
  return tb_segment_task(tb_c_word_join(e, 62007, 0, NULL, NULL, 2, children));
}
"#;

fn scalar_adapter_probe(callbacks: &str) -> String {
    probe(
        &(String::from(TEXT_CHECK) + callbacks),
        r#"
  const Term raw_two[2] = {0, 0};
  const u32 modes[4] = {TB_RESULT_RAW32, TB_RESULT_RAW64, TB_RESULT_BOX32, TB_RESULT_BOX64};
  for (u32 i = 0; i < 4; ++i) {
    Term input[2] = {modes[i], UINT64_C(0x100000007)}, result[1], owned[1];
    Term expected = (i == 0 || i == 2) ? 7 : UINT64_C(0x100000007);
    u32 count = corpus_eval_words(e.mem, tb_word_task(e, 62001, 2, input, raw_two), result, owned, 1);
    if (count != 1 || result[0] != expected || owned[0] != (i >= 2))
      err_fail("scalar tail adapter changed a value or ownership mask");
  }
  for (u32 i = 0; i < 2; ++i) {
    Term input[2] = {TB_RESULT_RAW32, i == 0 ? 62003 : 62006}, result[1], owned[1];
    u32 count = corpus_eval_words(e.mem, tb_word_task(e, 62002, 2, input, raw_two), result, owned, 1);
    if (count != 1 || result[0] != 9 || owned[0] != 0)
      err_fail("outer scalar adapter was lost while its child ran");
  }
  for (u32 outer = 0; outer < 4; ++outer) {
    for (u32 inner = 0; inner < 4; ++inner) {
      Term input[2] = {modes[outer], modes[inner]}, result[1], owned[1];
      Term expected = (outer == 0 || outer == 2 || inner == 0 || inner == 2)
        ? 7 : UINT64_C(0x100000007);
      u32 count = corpus_eval_words(e.mem, tb_word_task(e, 62008, 2, input, raw_two), result, owned, 1);
      if (count != 1 || result[0] != expected || owned[0] != (outer >= 2))
        err_fail("nested scalar adapters did not preserve outer ownership and narrowing");
    }
  }
"#,
        "  tb_register_closure(62000, wide_legacy, 0);\n  tb_register_segment(62001, adapt_legacy, 2, 1, 0);\n  tb_register_segment(62002, adapt_word, 2, 1, 0);\n  tb_register_segment(62003, pending_middle, 0, 1, 1);\n  tb_register_segment(62004, owned_child, 0, 1, 1);\n  tb_register_segment(62006, fork_children, 0, 1, 0);\n  tb_register_segment(62007, joined_children, 2, 1, 1);\n  tb_register_segment(62008, composed_adapter, 2, 1, 0);",
    )
}

#[test]
fn scalar_tail_adapters_preserve_exact_masks_and_do_not_modify_nested_children() {
    let foreign = scalar_adapter_probe(SCALAR_ADAPTERS);
    success(&Fixture::new().run(&foreign, &[]));
    for (original, replacement) in [
        (
            "{TB_RESULT_RAW32, TB_RESULT_RAW64, TB_RESULT_BOX32, TB_RESULT_BOX64}",
            "{2, TB_RESULT_RAW64, TB_RESULT_BOX32, TB_RESULT_BOX64}",
        ),
        (
            "Term task = tb_tail_apply(*e,",
            "frame->waiting = true;\n  Term task = tb_tail_apply(*e,",
        ),
        (
            "tb_register_segment(62001, adapt_legacy, 2, 1, 0)",
            "tb_register_segment(62001, adapt_legacy, 2, 2, 0)",
        ),
    ] {
        let malformed = foreign.replace(original, replacement);
        failure(
            &Fixture::new().run(&malformed, &[]),
            "invalid scalar tail result adapter",
        );
    }
}

#[test]
fn direct_calls_preserve_scalar_adapters_across_nested_waits_and_task_graphs() {
    let callbacks = SCALAR_ADAPTERS
        .replace(
            "return tb_segment_task(tb_word_task(*e, 62004, 0, NULL, NULL));",
            "return tb_segment_call(62004, 0, NULL, NULL);",
        )
        .replace(
            r"  Term task = tb_word_task(*e, (Fid)frame->captures[1], 0, NULL, NULL);
  return tb_segment_task(tb_tail_result(frame, task, (u32)frame->captures[0]));",
            r"  (void)e;
  frame->tail_result = (u32)frame->captures[0];
  return tb_segment_call((Fid)frame->captures[1], 0, NULL, NULL);",
        )
        .replace(
            r"  const Term raw_two[2] = {0, 0};
  Term input[2] = {frame->captures[1], UINT64_C(0x100000007)};
  Term task = tb_word_task(*e, 62001, 2, input, raw_two);
  return tb_segment_task(tb_tail_result(frame, task, (u32)frame->captures[0]));",
            r"  static const Term raw_two[2] = {0, 0};
  (void)e;
  frame->values[0] = frame->captures[1];
  frame->values[1] = UINT64_C(0x100000007);
  frame->tail_result = (u32)frame->captures[0];
  return tb_segment_call(62001, 2, frame->values, raw_two);",
        );
    assert_eq!(callbacks.matches("return tb_segment_call(").count(), 3);
    let foreign = scalar_adapter_probe(&callbacks).replace(
        "tb_register_segment(62008, composed_adapter, 2, 1, 0)",
        "tb_register_segment(62008, composed_adapter, 2, 1, 2)",
    );
    success(&Fixture::new().run(&foreign, &[]));
}
