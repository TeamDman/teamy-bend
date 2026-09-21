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
            "teamy-bend-c-task-abi-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, foreign: &str, definitions: &[&str]) -> Output {
        fs::write(self.0.join("main.bend"), SOURCE).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert_eq!(generated.matches("int main(void)").count(), 1);
        let mut generated = generated.replace("int main(void)", "static int checked_main(void)");
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {
    (void)fputs("task ABI program leaked owners or active work\n", stderr);
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

#[test]
fn registered_tasks_transfer_captures_and_emit_retains_its_owned_payload() {
    let callbacks = String::from(TEXT_CHECK)
        + r#"
static Term direct_apply(Env e, const Term *captures, Term argument) {
  check_text(e, captures[0], "capture");
  return argument;
}
"#;
    let foreign = probe(
        &callbacks,
        r#"
  Loc direct = task_node(e, 62000, TERM_HOLE, 0, 0);
  e.mem[direct] = io_str(e, "capture", 7);
  e.mem[direct + 1] = io_str(e, "argument", 8);
  check_text(e, corpus_eval(e.mem, term_tsk(62000, direct)), "argument");
  Term payload = io_str(e, "retained", 8);
  Loc emit = task_node(e, FID_IO_EMIT, TERM_HOLE, 0, 0);
  e.mem[emit] = payload;
  Term result = corpus_eval(e.mem, term_tsk(FID_IO_EMIT, emit));
  if (term_tag(result) != TAG_CTR || term_aux(result) != CID_EMIT
      || e.mem[term_peek(e, result)] != payload)
    err_fail("Emit lost its owned payload");
  Term child;
  Loc spare = ctr_take(e, result, 1, &child);
  spare_free(e, 0, spare);
  check_text(e, child, "retained");
"#,
        "  tb_register_closure(62000, direct_apply, 1);",
    );
    success(&Fixture::new().run(&foreign, &[]));
}

fn linear_chain() -> String {
    let callbacks = String::from(TEXT_CHECK)
        + r#"
#ifndef CHAIN_DEPTH
#define CHAIN_DEPTH 512u
#endif
static u32 linear_calls;
static Term linear_apply(Env e, const Term *captures, Term argument) {
  ++linear_calls;
  check_text(e, captures[0], "h");
  return argument;
}
static Term leaf_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  return argument;
}
"#;
    probe(
        &callbacks,
        r#"
  Term continuation = TERM_HOLE;
  for (u32 i = 0; i < CHAIN_DEPTH; ++i) {
    Loc parent = task_node(e, 62000, continuation, continuation == TERM_HOLE ? 0 : 1, 1);
    e.mem[parent] = io_str(e, "h", 1);
    continuation = term_tsk(62000, parent);
  }
  Loc leaf = task_node(e, 62001, continuation, 1, 0);
  e.mem[leaf] = io_str(e, "payload", 7);
  check_text(e, corpus_eval(e.mem, term_tsk(62001, leaf)), "payload");
  if (linear_calls != CHAIN_DEPTH || tb_task_joins < CHAIN_DEPTH || tb_task_peak < CHAIN_DEPTH)
    err_fail("linear continuation chain did not execute once");
"#,
        "  tb_register_closure(62000, linear_apply, 1);\n  tb_register_closure(62001, leaf_apply, 0);",
    )
}

#[test]
fn nonroot_linear_continuations_preserve_owners_without_native_recursion() {
    success(&Fixture::new().run(&linear_chain(), &[]));
}

#[test]
fn task_budget_accepts_a_shallow_chain_before_rejecting_the_deep_chain() {
    let foreign = linear_chain();
    success(&Fixture::new().run(&foreign, &["BEND_MAX_TASKS=16", "CHAIN_DEPTH=4"]));
    failure(
        &Fixture::new().run(&foreign, &["BEND_MAX_TASKS=16", "CHAIN_DEPTH=64"]),
        "task budget exhausted",
    );
}

const FORK_CALLBACKS: &str = r#"
static u32 trace[4], trace_count, joined, reverse_order;
static Term identity_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  return argument;
}
static Term late_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  trace[trace_count++] = 3;
  return argument;
}
static Term left_apply(Env e, const Term *captures, Term argument) {
  (void)captures;
  trace[trace_count++] = 1;
  if (reverse_order) {
    Loc parent = task_node(e, 62002, TERM_HOLE, 0, 1);
    Term join = term_tsk(62002, parent);
    Loc child = task_node(e, 62003, join, 0, 0);
    e.mem[child] = argument;
    e.mem[parent] = term_tsk(62003, child);
    return join;
  }
  return argument;
}
static Term right_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  trace[trace_count++] = 2;
  return argument;
}
static Term join_apply(Env e, const Term *captures, Term argument) {
  ++joined;
  check_text(e, captures[0], "left");
  check_text(e, captures[1], "right");
  term_sink(e, argument);
  return term_pak(CID_UNIT, 0);
}
static Term fork_apply(Env e, const Term *captures, Term argument) {
  (void)captures; term_sink(e, argument);
  Loc parent = task_node(e, 62004, TERM_HOLE, 0, 2);
  Term join = term_tsk(62004, parent);
  Loc left = task_node(e, 62000, join, 0, 0);
  Loc right = task_node(e, 62001, join, 1, 0);
  e.mem[left] = io_str(e, "left", 4);
  e.mem[right] = io_str(e, "right", 5);
  e.mem[parent] = term_tsk(62000, left);
  e.mem[parent + 1] = term_tsk(62001, right);
  e.mem[parent + 2] = 0;
  return join;
}
"#;

const FORK_REGISTRATIONS: &str = "  tb_register_closure(62000, left_apply, 0);\n  tb_register_closure(62001, right_apply, 0);\n  tb_register_closure(62002, identity_apply, 0);\n  tb_register_closure(62003, late_apply, 0);\n  tb_register_closure(62004, join_apply, 2);\n  tb_register_closure(62005, fork_apply, 0);";

#[test]
fn returned_joins_deliver_each_result_once_even_when_children_finish_out_of_order() {
    let callbacks = String::from(TEXT_CHECK) + FORK_CALLBACKS;
    let foreign = probe(
        &callbacks,
        r#"
  for (reverse_order = 0; reverse_order < 2; ++reverse_order) {
    trace_count = 0; joined = 0;
    term_sink(e, tb_apply(e, tb_closure(e, 62005, 0, NULL), 0));
    if (joined != 1 || trace_count != 2 + reverse_order || trace[0] != 1 || trace[1] != 2
        || (reverse_order && trace[2] != 3))
      err_fail("fork completion order or count changed");
  }
  if (tb_task_joins < 3) err_fail("returned task graphs were not scheduled");
"#,
        FORK_REGISTRATIONS,
    );
    success(&Fixture::new().run(&foreign, &[]));
}

#[test]
fn dropping_pending_graphs_consumes_payloads_but_never_follows_parent_backlinks() {
    let callbacks = String::from(TEXT_CHECK)
        + r#"
static Term unused_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures; (void)argument;
  err_fail("discarded task executed");
}
"#;
    let foreign = probe(
        &callbacks,
        r#"
  u64 before_words = tb_live_words, before_blocks = tb_live_blocks;
  Loc parent = task_node(e, 62000, TERM_HOLE, 0, 2);
  Term join = term_tsk(62000, parent);
  e.mem[parent] = io_str(e, "held", 4);
  e.mem[parent + 3] = 0;
  for (u32 i = 1; i <= 2; ++i) {
    Term capture = io_str(e, "capture", 7);
    Loc child = task_node(e, FID_CLO_APPLY, join, i, 0);
    e.mem[child] = tb_closure(e, 62001, 1, &capture);
    e.mem[child + 1] = io_str(e, "argument", 8);
    e.mem[parent + i] = term_tsk(FID_CLO_APPLY, child);
  }
  term_drop(e, join);
  if (before_words != tb_live_words || before_blocks != tb_live_blocks)
    err_fail("pending graph drop leaked payloads");
  parent = task_node(e, 62001, TERM_HOLE, 0, 1);
  join = term_tsk(62001, parent);
  e.mem[parent] = io_str(e, "retained", 8);
  Loc child = task_node(e, 62002, join, 1, 0);
  e.mem[child] = io_str(e, "discarded", 9);
  term_drop(e, term_tsk(62002, child));
  check_text(e, e.mem[parent], "retained");
  e.mem[parent] = 0;
  term_drop(e, join);
"#,
        "  tb_register_closure(62000, unused_apply, 3);\n  tb_register_closure(62001, unused_apply, 1);\n  tb_register_closure(62002, unused_apply, 0);",
    );
    success(&Fixture::new().run(&foreign, &[]));
}

#[test]
fn raw_multiword_delivery_preserves_slot_ownership_and_counts_children_once() {
    let foreign = probe(
        r#"
static Term unused_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures; (void)argument;
  err_fail("raw delivery fixture unexpectedly executed a boxed task");
}
"#,
        r#"
  for (u32 reverse = 0; reverse < 2; ++reverse) {
    Loc parent = task_node(e, 62000, TERM_HOLE, 0, 2);
    Term join = term_tsk(62000, parent);
    Term pair[2] = {UINT64_C(0x0500000000001234), io_str(e, "owned", 5)};
    Term last = term_pak(CID_UNIT, 0);
    e.mem[parent] = io_str(e, "held", 4);
    tb_mark_raw(e, parent + 1, 1);
    Term first = reverse ? task_deliver(e.mem, join, 3, &last, 1)
                         : task_deliver(e.mem, join, 1, pair, 2);
    if (first != 0 || (u32)e.mem[task_tail(join) + 1] != 1)
      err_fail("multiword delivery decremented per word");
    Term second = reverse ? task_deliver(e.mem, join, 1, pair, 2)
                          : task_deliver(e.mem, join, 3, &last, 1);
    if (second != join || (u32)e.mem[task_tail(join) + 1] != 0
        || e.mem[parent + 1] != pair[0] || e.mem[parent + 2] != pair[1]
        || e.mem[parent + 3] != last || tb_cell_owned(e, parent + 1)
        || !tb_cell_owned(e, parent + 2))
      err_fail("multiword result layout or ownership changed");
    term_drop(e, join);
  }
"#,
        "  tb_register_closure(62000, unused_apply, 3);",
    );
    success(&Fixture::new().run(&foreign, &[]));
}

#[test]
fn malformed_graphs_reject_before_any_child_callback_executes() {
    // Observable callback markers distinguish validation-before-dispatch from
    // a late rejection after a child has already performed foreign work.
    let callbacks = (String::from(TEXT_CHECK) + FORK_CALLBACKS)
        .replace(
            "trace[trace_count++] = 1;",
            "io_out(stdout, \"child\\n\", 6); trace[trace_count++] = 1;",
        )
        .replace(
            "trace[trace_count++] = 2;",
            "io_out(stdout, \"child\\n\", 6); trace[trace_count++] = 2;",
        );
    for (corruption, diagnostic) in [
        (
            "e.mem[parent + 1] = e.mem[parent];",
            "duplicate or cyclic task graph",
        ),
        ("e.mem[parent] = join;", "duplicate or cyclic task graph"),
        (
            "e.mem[task_tail(e.mem[parent]) + 1] = UINT64_C(1) << 32;",
            "task child destination mismatch",
        ),
        (
            "e.mem[task_tail(e.mem[parent])] = TERM_HOLE;",
            "task child destination mismatch",
        ),
        (
            "e.mem[task_tail(join) + 1] = 1;",
            "task dependency count mismatch",
        ),
        (
            "e.mem[parent] |= RFC_BIT;",
            "reference-counted foreign task is unsupported",
        ),
        (
            "join = term_tsk(65535, parent);",
            "foreign task id is unsupported",
        ),
        (
            "join = term_tsk(62004, heap_alloc(e, 1));",
            "native allocation class mismatch",
        ),
        (
            "e.mem[parent + 2] = TERM_HOLE;",
            "foreign task argument is missing",
        ),
    ] {
        let mutated = callbacks.replace(
            "  return join;\n}",
            &format!("  {corruption}\n  return join;\n}}"),
        );
        let foreign = probe(
            &mutated,
            "  term_sink(e, tb_apply(e, tb_closure(e, 62005, 0, NULL), 0));",
            FORK_REGISTRATIONS,
        );
        failure(&Fixture::new().run(&foreign, &[]), diagnostic);
    }
}

#[test]
fn duplicate_delivery_and_incomplete_external_siblings_fail_closed() {
    let callbacks = r"
static Term identity_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  return argument;
}
";
    let registrations = "  tb_register_closure(62000, identity_apply, 2);\n  tb_register_closure(62001, identity_apply, 0);";
    for (body, diagnostic) in [
        (
            r"
  Loc parent = task_node(e, 62000, TERM_HOLE, 0, 2);
  Term join = term_tsk(62000, parent), value = 7;
  (void)task_deliver(e.mem, join, 0, &value, 1);
  (void)task_deliver(e.mem, join, 0, &value, 1);
",
            "duplicate task result delivery",
        ),
        (
            r"
  Loc parent = task_node(e, 62000, TERM_HOLE, 0, 2);
  Term join = term_tsk(62000, parent);
  e.mem[parent + 2] = 0;
  Loc child = task_node(e, 62001, join, 0, 0);
  e.mem[child] = 7;
  term_sink(e, corpus_eval(e.mem, term_tsk(62001, child)));
",
            "incomplete external task graph",
        ),
    ] {
        failure(
            &Fixture::new().run(&probe(callbacks, body, registrations), &[]),
            diagnostic,
        );
    }
}
