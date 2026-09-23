// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

const TREE: &str = r"import Base
type Tree is Data:
  Tip{value: U32}
  Fork{left: Tree, right: Tree}
def sum(tree: Tree) -> U32:
  match tree:
    case Tip{value}: value
    case Fork{left, right}: U32.add(sum(left), sum(right))
def share(+tree: Tree) -> U32:
  left right = sum(tree) sum(tree)
  U32.add(left, right)
def main() -> U32:
  share!(Fork{Fork{Tip{1}, Tip{2}}, Fork{Tip{3}, Tip{4}}})
";

const NESTED_CLOSURE: &str = r"import Base
type Tree is Data:
  Tip{value: U32}
  Fork{left: Tree, right: Tree}
def sum(tree: Tree) -> U32:
  match tree:
    case Tip{value}: value
    case Fork{left, right}: U32.add(sum(left), sum(right))
type Holder is Type: Holder{run: Unit -> U32}
def first(tree: Tree) -> Unit -> U32:
  ignored => sum(tree)
def wrap(f: Unit -> U32) -> Unit -> U32:
  ignored => f(Unit{})
def use(holder: Holder) -> U32:
  match holder:
    case Holder{run}: run(Unit{})
@unsafe
def share(+holder: Holder) -> U32:
  left right = use(holder) use(holder)
  U32.add(left, right)
def main() -> U32:
  share!(Holder{wrap(first(Fork{Fork{Tip{1}, Tip{2}}, Fork{Tip{3}, Tip{4}}}))})
";

const SCALAR: &str = r"import Base
type Tree is Data:
  Tip{value: U32}
  Fork{left: Tree, right: Tree}
def sum(tree: Tree) -> U32:
  match tree:
    case Tip{value}: value
    case Fork{left, right}: U32.add(sum(left), sum(right))
def builder(+tree: Tree) -> Unit -> U32:
  ignored => U32.add(sum(tree), sum(tree))
def invoke(f: Unit -> U32) -> U32: f(Unit{})
def main() -> U32:
  invoke!(builder(Fork{Fork{Tip{1}, Tip{2}}, Fork{Tip{3}, Tip{4}}}))
";

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

#[derive(Debug)]
struct Receipt {
    statistics: [u64; 9],
    observations: [u64; 32],
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-gpu-duplicate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, bend: &str, quantum: u32, mode: u32) -> Receipt {
        let path = self.0.join("main.bend");
        fs::write(&path, bend).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let source = instrument(generated, mode);
        let quantum = format!("BEND_GPU_PRIMITIVE_QUANTUM={quantum}");
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &[
                "BEND_MAX_ALLOC=1048576",
                "BEND_CPU_WORKERS=4",
                "BEND_GPU_QUANTUM=4",
                &quantum,
            ],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", "on"),
            Duration::from_mins(1),
        );
        fs::write(self.0.join("stdout.log"), &output.stdout).unwrap();
        fs::write(self.0.join("stderr.log"), &output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}\n{}",
            self.0.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"20\n");
        assert_eq!(
            output.stderr,
            match mode {
                0 => b"".as_slice(),
                1 => b"teamy-bend executable C: duplicate test interruption\n",
                2 => b"teamy-bend executable C: VM allocation budget exhausted\n",
                _ => panic!("unknown duplicate probe mode"),
            }
        );
        let words: Vec<u64> = fs::read_to_string(self.0.join("receipt.txt"))
            .unwrap()
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        assert_eq!(words.len(), 41);
        Receipt {
            statistics: words[..9].try_into().unwrap(),
            observations: words[9..].try_into().unwrap(),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU duplicate test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn replace_once(source: &mut String, from: &str, to: &str) {
    assert_eq!(source.matches(from).count(), 1, "missing unique {from}");
    *source = source.replace(from, to);
}

fn instrument(mut generated: String, mode: u32) -> String {
    let start = generated
        .find("static const char *const tb_gpu_source_parts[] = {\n")
        .unwrap();
    let end = start + generated[start..].find("\n};").unwrap() + 3;
    let mut device = decode_parts(&generated[start..end]);
    let mut words = false;
    let mut sites = 0;
    let mut observed = String::new();
    for line in device.split_inclusive('\n') {
        if line.contains(" tb_resume_") && line.trim_end().ends_with('{') {
            words = line.contains("TBOutcome");
        }
        if line.contains("tb_device_duplicate(e, tb_frame,") {
            sites += 1;
            observed.push_str(&line.replace(
                "tb_device_duplicate(e, tb_frame,",
                if words {
                    "probe_duplicate_words(e, tb_frame,"
                } else {
                    "probe_duplicate_scalar(e, tb_frame,"
                },
            ));
        } else {
            observed.push_str(line);
        }
    }
    assert!(sites > 0, "fixture must reach generated duplication");
    device = observed;
    replace_once(
        &mut device,
        "      tb_tick();\n      at = current & LOC_MASK;",
        "      PROBE_TICK_DROP();\n      at = current & LOC_MASK;",
    );
    replace_once(
        &mut device,
        "          tb_tick();\n          TBDupRef source =",
        "          PROBE_TICK_DUPLICATE();\n          TBDupRef source =",
    );
    replace_once(
        &mut device,
        "      case TB_DUP_SEAL_SCAN:\n        tb_tick();",
        "      case TB_DUP_SEAL_SCAN:\n        PROBE_TICK_DUPLICATE();",
    );
    replace_once(
        &mut device,
        "      } else if (tag == TAG_CTR) { count = cid_arity(aux); cls = cls_fit(count); }",
        "      } else if (tag == TAG_CTR) {\n        count = cid_arity(aux);\n        if (count == 2) probe_count_fork_drop();\n        cls = cls_fit(count);\n      }",
    );
    device = device.replace("tb_tick();", "PROBE_TICK_OTHER();");
    assert!(
        !device.contains("tb_tick();"),
        "all device ticks must be categorized"
    );
    replace_once(
        &mut device,
        "static __device__ TBDeviceControl *tb_device_control;",
        "static __device__ TBDeviceControl *tb_device_control;\nstatic __device__ void probe_tick(unsigned int);\nstatic __device__ void probe_count_fork_drop(void);\nstatic __device__ void probe_tick(unsigned int category) { atomicAdd(tb_device_control->root_words + 252 + category, 1ull); tb_tick(); }\nstatic __device__ void probe_count_fork_drop(void) { atomicAdd(tb_device_control->root_words + 251, 1ull); }",
    );
    let first_resume = device.find("static __device__ TB_NOINLINE ").unwrap();
    device.insert_str(first_resume, DEVICE_WRAPPERS);
    let yielded = "run->resume_tick = 1; run->resume_turn = turn;";
    replace_once(
        &mut device,
        yielded,
        &format!(
            "{yielded}\n        if (turn != 0) atomicAdd(tb_device_control->root_words + 238, 1ull);"
        ),
    );
    replace_once(
        &mut device,
        "  u32 first_turn = 0;",
        "  bool probe_resumed = run->resume_tick != 0;\n  u32 first_turn = 0;",
    );
    let ready = "  }\n}\n\n/* Graph indexing/adoption/delivery";
    replace_once(
        &mut device,
        ready,
        "  }\n  if (probe_resumed) atomicAdd(tb_device_control->root_words + 239, 1ull);\n}\n\n/* Graph indexing/adoption/delivery",
    );
    device.insert_str(0, DEVICE_PREFIX);
    device.push_str(DEVICE_OBSERVER);
    generated.replace_range(start..end, &encode_parts(&device));

    let upload = "  tb_gpu_require(tb_cuda_upload(TB_GPU_STATE, 0, &state, sizeof(state)));";
    replace_once(
        &mut generated,
        upload,
        &format!("  state.reserved = probe_mode;\n  PROBE_HOST_SNAPSHOT();\n{upload}"),
    );
    let download =
        "    tb_gpu_require(tb_cuda_download(TB_GPU_CONTROL, 0, &control, sizeof(control)));";
    replace_once(
        &mut generated,
        download,
        &format!("{download}\n    PROBE_ROUND(&state, &control);"),
    );
    let import = "  tb_gpu_require(tb_cuda_download(TB_GPU_CORPUS, 0, e.mem, used_bytes));";
    replace_once(
        &mut generated,
        import,
        &format!("  ++probe_imports;\n{import}"),
    );
    let cpu = "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {";
    replace_once(
        &mut generated,
        cpu,
        &format!("{cpu}\n  if (tb_gpu_marked((Fid)term_aux(task))) ++probe_cpu_replays;"),
    );
    replace_once(
        &mut generated,
        "static int tb_program_main(void)",
        "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
    );
    let mut source = String::from(HOST_PREFIX);
    source.push_str(&generated);
    writeln!(source, "\n#define PROBE_MODE {mode}").unwrap();
    source.push_str(HOST_MAIN);
    source
}

fn decode_parts(parts: &str) -> String {
    let mut result = Vec::new();
    for line in parts.lines().filter(|line| line.trim().starts_with('"')) {
        let literal = line.trim().strip_suffix(',').unwrap();
        let mut bytes = literal[1..literal.len() - 1].bytes();
        while let Some(byte) = bytes.next() {
            if byte != b'\\' {
                result.push(byte);
                continue;
            }
            let escaped = bytes.next().unwrap();
            result.push(match escaped {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'\\' | b'"' => escaped,
                b'0'..=b'7' => {
                    (escaped - b'0') * 64
                        + (bytes.next().unwrap() - b'0') * 8
                        + bytes.next().unwrap()
                        - b'0'
                }
                _ => panic!("unexpected generated C string escape"),
            });
        }
    }
    String::from_utf8(result).unwrap()
}

fn encode_parts(device: &str) -> String {
    let mut output = String::from("static const char *const tb_gpu_source_parts[] = {\n");
    for chunk in device.as_bytes().chunks(1024) {
        output.push_str("  \"");
        for byte in chunk {
            match byte {
                b'\n' => output.push_str("\\n"),
                b'\r' => output.push_str("\\r"),
                b'\t' => output.push_str("\\t"),
                b'\\' => output.push_str("\\\\"),
                b'"' => output.push_str("\\\""),
                32..=126 => output.push(char::from(*byte)),
                _ => write!(output, "\\{byte:03o}").unwrap(),
            }
        }
        output.push_str("\",\n");
    }
    output.push_str("  NULL\n};");
    output
}

// The unused tail of the one-word root result carries observer receipts. Every
// observation is outside production locks; the test never changes ownership.
// Failure mode 2 limits the corpus only after a partial rewrite, so the next
// actual count-cell reservation must refuse without changing its allocator.
const DEVICE_PREFIX: &str = r"
enum {
  P_START, P_COMPLETE, P_SLICE, P_CHILD_WRAPS, P_ROOT_WRAPS, P_RETAINS,
  P_CLONE_RESERVES, P_CLONE_COMPLETES, P_PARTIAL_CHILD, P_NESTED_CLONES,
  P_TYPED_ENTER, P_SCALAR_ENTER, P_TYPED_YIELDS, P_SCALAR_YIELDS,
  P_RESUME_NONZERO, P_RESUMED_READY, P_ERRORS, P_FAULT, P_FAST,
  P_CLEARED, P_WORK, P_ROOT_UNPUBLISHED, P_OWNER_CHECKS,
  P_FAILURE_BUMP, P_FAILURE_WORDS, P_FAILURE_BLOCKS, P_FAILURE_FREE_HASH,
  P_FAILURE_SCRATCH_CAPACITY, P_DROP_FORKS = P_FAILURE_SCRATCH_CAPACITY
};
static __device__ void probe_tick(unsigned int);
static __device__ void probe_count_fork_drop(void);
#define PROBE_TICK_DROP() probe_tick(0)
#define PROBE_TICK_DUPLICATE() probe_tick(1)
#define PROBE_TICK_OTHER() probe_tick(2)
struct TBDupWork;
static __device__ void probe_observe(unsigned int, unsigned long long *,
  const TBDupWork *, unsigned long long);
#define TB_DEVICE_DUPLICATE_OBSERVE(event, state, frame, work) probe_observe(event, state, frame, work)
";

const DEVICE_WRAPPERS: &str = r"
static __device__ bool probe_duplicate_call(const Env *e, TBCallFrame *frame,
    Term *owner, Term *state, Term *result, bool words) {
  if (state[0] == 0)
    atomicAdd(tb_device_control->root_words + 224 + (words ? P_TYPED_ENTER : P_SCALAR_ENTER), 1ull);
  bool done = tb_device_duplicate(e, frame, owner, state, result);
  if (!done) {
    atomicAdd(tb_device_control->root_words + 224 + (words ? P_TYPED_YIELDS : P_SCALAR_YIELDS), 1ull);
  } else {
    for (u32 i = 0; i < TB_DEVICE_DUPLICATE_STATE_WORDS; ++i)
      if (state[i] != 0) atomicAdd(tb_device_control->root_words + 224 + P_ERRORS, 1ull);
    atomicAdd(tb_device_control->root_words + 224 + P_CLEARED, 1ull);
  }
  return done;
}
static __device__ bool probe_duplicate_words(const Env *e, TBCallFrame *frame,
    Term *owner, Term *state, Term *result) {
  return probe_duplicate_call(e, frame, owner, state, result, true);
}
static __device__ bool probe_duplicate_scalar(const Env *e, TBCallFrame *frame,
    Term *owner, Term *state, Term *result) {
  return probe_duplicate_call(e, frame, owner, state, result, false);
}
";

const DEVICE_OBSERVER: &str = r#"
struct ProbeRewrite { TBDupRef owner; Term before, after; };
struct ProbeClone { Loc source, target; u64 count; Term captures[8]; bool done; };
struct ProbeDuplicate {
  Term *state;
  TBDupRef owner, destination;
  Term original;
  ProbeRewrite rewrites[32];
  ProbeClone clones[16];
  u32 rewrite_count, clone_count;
  bool active;
};
static __device__ ProbeDuplicate probe_operations[32];
static __device__ u32 probe_lock, probe_count;
static __device__ void probe_observe(u32 event, Term *state,
    const TBDupWork *frame, u64 work) {
  if (tb_device_control->root_count > 3) err_fail("invalid duplicate test root");
  for (;;) {
    tb_device_check_cancelled();
    if (atomicCAS(&probe_lock, 0u, 1u) == 0) break;
    __nanosleep(64);
  }
  Env e = {tb_memory, NULL};
  u64 *receipt = tb_device_control->root_words + 224;
  ProbeDuplicate *record = NULL;
  for (u32 i = 0; i < probe_count; ++i)
    if (probe_operations[i].state == state && probe_operations[i].active)
      record = probe_operations + i;
  if (event == TB_DEVICE_DUPLICATE_ENTER) {
    if ((state[0] == 0) == (record != NULL)) ++receipt[P_ERRORS];
  } else if (event == TB_DEVICE_DUPLICATE_FAST) {
    if (record != NULL || frame != NULL || state[0] != 0) ++receipt[P_ERRORS];
    ++receipt[P_FAST];
  } else if (event == TB_DEVICE_DUPLICATE_START) {
    if (record != NULL || frame == NULL || probe_count == 32)
      err_fail("invalid duplicate observer start");
    record = probe_operations + probe_count++;
    record->state = state; record->owner = frame->owner;
    record->destination = frame->destination; record->original = frame->value;
    record->active = true;
    if (*tb_device_dup_pointer(e, record->owner) != record->original
        || *tb_device_dup_pointer(e, record->destination) != 0)
      ++receipt[P_ERRORS];
    ++receipt[P_START];
  } else {
    if (record == NULL || frame == NULL) err_fail("missing duplicate observer state");
    if (!tb_device_dup_same_ref(tb_device_dup_frame(state[2])->owner, record->owner)
        || !tb_device_dup_same_ref(tb_device_dup_frame(state[2])->destination, record->destination))
      ++receipt[P_ERRORS];
    // Root output belongs to the operation until COMPLETE returns, even if a
    // previous child has already been installed in a private clone shell.
    if (*tb_device_dup_pointer(e, record->destination) != 0) ++receipt[P_ERRORS];
    ++receipt[P_ROOT_UNPUBLISHED];
    for (u32 i = 0; i < record->rewrite_count; ++i) {
      const ProbeRewrite *rewrite = record->rewrites + i;
      if (*tb_device_dup_pointer(e, rewrite->owner) != rewrite->after) ++receipt[P_ERRORS];
      ++receipt[P_OWNER_CHECKS];
    }
    if (event == TB_DEVICE_DUPLICATE_CHILD_WRAPPED || event == TB_DEVICE_DUPLICATE_ROOT_WRAPPED) {
      if (record->rewrite_count == 32) err_fail("duplicate test rewrite limit");
      for (u32 i = 0; i < record->rewrite_count; ++i)
        if (tb_device_dup_same_ref(record->rewrites[i].owner, frame->owner)) ++receipt[P_ERRORS];
      ProbeRewrite *rewrite = record->rewrites + record->rewrite_count++;
      rewrite->owner = frame->owner; rewrite->before = frame->value; rewrite->after = frame->output;
      if (term_rfc(frame->value) || !term_rfc(frame->output)
          || term_tag(frame->value) == TAG_CLO
          || *tb_device_dup_pointer(e, frame->owner) != frame->output
          || (rfc_view(e, term_loc(frame->output)) >> 24) != term_loc(frame->value)
          || (rfc_view(e, term_loc(frame->output)) & RFC_CNT)
            != (event == TB_DEVICE_DUPLICATE_CHILD_WRAPPED ? 1u : 2u))
        ++receipt[P_ERRORS];
      if (event == TB_DEVICE_DUPLICATE_CHILD_WRAPPED) {
        ++receipt[P_CHILD_WRAPS];
        Loc parent = frame->owner.base;
        if (frame->owner.arena == TB_DUP_CORPUS && frame->owner.index == 0
            && tb_meta_class(tb_heap_meta[parent]) >= 1
            && !tb_cell_sealed(e, parent) && tb_cell_owned(e, parent + 1)) {
          Term later = e.mem[parent + 1];
          if (term_tag(later) == TAG_CTR && !term_rfc(later)
              && cid_arity((u32)term_aux(later)) == 2 && !tb_cell_sealed(e, term_loc(later))) {
            ++receipt[P_PARTIAL_CHILD];
            if (tb_device_state->reserved == 1) {
              receipt[P_FAULT] = 1;
              __threadfence(); atomicExch(&probe_lock, 0u);
              err_fail("duplicate test interruption");
            }
            if (tb_device_state->reserved == 2 && receipt[P_FAULT] == 0) {
              tb_vm_acquire();
              if (tb_free_lists[0] != 0) err_fail("duplicate test requires fresh count cell");
              receipt[P_FAULT] = 2;
              receipt[P_FAILURE_BUMP] = tb_bump;
              receipt[P_FAILURE_WORDS] = tb_live_words;
              receipt[P_FAILURE_BLOCKS] = tb_live_blocks;
              receipt[P_FAILURE_FREE_HASH] = 0;
              for (u32 i = 0; i < NCLS_ALL; ++i)
                receipt[P_FAILURE_FREE_HASH] = (receipt[P_FAILURE_FREE_HASH] * 33u) ^ tb_free_lists[i];
              receipt[P_FAILURE_SCRATCH_CAPACITY] = tb_device_state->scratch_capacity;
              tb_capacity = tb_bump;
              tb_vm_release();
            }
          }
        }
      } else ++receipt[P_ROOT_WRAPS];
    } else if (event == TB_DEVICE_DUPLICATE_RETAINED) {
      ++receipt[P_RETAINS];
      if (!term_rfc(frame->value) || frame->output != frame->value
          || *tb_device_dup_pointer(e, frame->owner) != frame->value
          || (rfc_view(e, term_loc(frame->value)) & RFC_CNT) < frame->before + 1)
        ++receipt[P_ERRORS];
    } else if (event == TB_DEVICE_DUPLICATE_CLONE_RESERVED) {
      if (record->clone_count == 16 || frame->count > 8) err_fail("duplicate test clone limit");
      ProbeClone *clone = record->clones + record->clone_count++;
      clone->source = frame->source; clone->target = frame->target; clone->count = frame->count;
      for (u32 i = 0; i < clone->count; ++i) clone->captures[i] = e.mem[clone->source + i];
      if (clone->source == clone->target || term_tag(frame->value) != TAG_CLO
          || term_rfc(frame->value) || *tb_device_dup_pointer(e, frame->owner) != frame->value)
        ++receipt[P_ERRORS];
      ++receipt[P_CLONE_RESERVES];
    } else if (event == TB_DEVICE_DUPLICATE_CLONE_COMPLETE) {
      ProbeClone *clone = NULL;
      for (u32 i = 0; i < record->clone_count; ++i)
        if (record->clones[i].target == frame->target) clone = record->clones + i;
      if (clone == NULL || clone->done) err_fail("duplicate test clone completion");
      clone->done = true;
      if (*tb_device_dup_pointer(e, frame->owner) != frame->value
          || frame->output != term_clo(term_aux(frame->value), frame->target)) ++receipt[P_ERRORS];
      for (u32 i = 0; i < clone->count; ++i) {
        Term original = clone->captures[i], copy = e.mem[clone->target + i];
        if (e.mem[clone->source + i] != original || !tb_cell_owned(e, clone->target + i))
          ++receipt[P_ERRORS];
        if (term_tag(original) == TAG_CLO && term_loc(original) != 0) {
          if (term_tag(copy) != TAG_CLO || term_rfc(copy) || term_aux(copy) != term_aux(original)
              || term_loc(copy) == term_loc(original)) ++receipt[P_ERRORS];
          ++receipt[P_NESTED_CLONES];
        } else if (copy != original) ++receipt[P_ERRORS];
      }
      ++receipt[P_CLONE_COMPLETES];
    } else if (event == TB_DEVICE_DUPLICATE_SLICE) {
      if (work == 0 || work > BEND_GPU_PRIMITIVE_QUANTUM) ++receipt[P_ERRORS];
      ++receipt[P_SLICE]; receipt[P_WORK] += work;
    } else if (event == TB_DEVICE_DUPLICATE_COMPLETE) {
      if (frame->previous != 0 || frame->kind != TB_DUP_VALUE
          || frame->phase != TB_DUP_DONE || state[1] != state[2]) ++receipt[P_ERRORS];
      if (term_tag(record->original) == TAG_CLO) {
        if (*tb_device_dup_pointer(e, record->owner) != record->original
            || term_tag(frame->output) != TAG_CLO || term_rfc(frame->output)
            || term_loc(frame->output) == term_loc(record->original)) ++receipt[P_ERRORS];
      } else if (*tb_device_dup_pointer(e, record->owner) != frame->output
          || !term_rfc(frame->output)) ++receipt[P_ERRORS];
      for (u32 i = 0; i < record->clone_count; ++i)
        if (!record->clones[i].done) ++receipt[P_ERRORS];
      record->active = false; ++receipt[P_COMPLETE];
    } else ++receipt[P_ERRORS];
  }
  __threadfence(); atomicExch(&probe_lock, 0u);
}
"#;

const HOST_PREFIX: &str = r"
static unsigned int probe_mode, probe_failures, probe_completions, probe_errors;
static unsigned int probe_imports, probe_cpu_replays;
static unsigned long long probe_stats[9], probe_receipt[32];
static unsigned long long probe_host_bump, probe_host_live, probe_host_blocks, probe_host_hash;
static unsigned long long probe_host_incoming_steps;
#define PROBE_HOST_HASH(result) do { \
  (result) = 0; \
  for (u64 probe_i = 0; probe_i < tb_bump; ++probe_i) \
    (result) = (((result) * 33u) ^ tb_memory[probe_i]) + tb_heap_meta[probe_i]; \
  for (u32 probe_i = 0; probe_i < NCLS_ALL; ++probe_i) \
    (result) = ((result) * 33u) ^ tb_free_lists[probe_i]; \
} while (0)
#define PROBE_HOST_SNAPSHOT() do { \
  probe_host_bump = tb_bump; probe_host_live = tb_live_words; probe_host_blocks = tb_live_blocks; \
  probe_host_incoming_steps = state.steps; \
  PROBE_HOST_HASH(probe_host_hash); \
} while (0)
#define PROBE_ROUND(state, control) do { \
  if ((state)->error != 0) { \
    unsigned long long probe_hash_now; \
    ++probe_failures; \
    memcpy(probe_receipt, (control)->root_words + 224, sizeof(probe_receipt)); \
    PROBE_HOST_HASH(probe_hash_now); \
    if ((probe_mode != 1 && probe_mode != 2) || probe_receipt[17] != probe_mode || probe_receipt[8] == 0 \
        || probe_receipt[3] == 0 || probe_receipt[16] != 0 || (control)->primitive_live == 0 \
        || probe_imports != 0 || tb_bump != probe_host_bump || tb_live_words != probe_host_live \
        || tb_live_blocks != probe_host_blocks || probe_hash_now != probe_host_hash) ++probe_errors; \
    if (probe_mode == 2) { \
      unsigned long long probe_free_hash = 0; \
      for (u32 probe_i = 0; probe_i < NCLS_ALL; ++probe_i) \
        probe_free_hash = (probe_free_hash * 33u) ^ (state)->free_lists[probe_i]; \
      if ((state)->capacity != probe_receipt[23] || (state)->bump != probe_receipt[23] \
          || (state)->live_words != probe_receipt[24] || (state)->live_blocks != probe_receipt[25] \
          || probe_free_hash != probe_receipt[26] || (state)->free_lists[0] != 0 \
          || (state)->scratch_capacity != probe_receipt[27]) ++probe_errors; \
    } \
  } \
} while (0)
#define TB_GPU_COMPLETE(control, state, info) do { \
  ++probe_completions; \
  memcpy(probe_receipt, (control)->root_words + 224, sizeof(probe_receipt)); \
  probe_stats[0] = (state)->steps; probe_stats[1] = (info)->launches; \
  probe_stats[2] = (control)->primitive_progress; probe_stats[3] = (control)->primitive_starts; \
  probe_stats[4] = (control)->primitive_yields; probe_stats[5] = (control)->primitive_requeues; \
  probe_stats[6] = (control)->primitive_live; probe_stats[7] = (control)->forks; \
  probe_stats[8] = (control)->peak_lanes; \
  probe_receipt[31] = probe_host_incoming_steps; \
  if ((state)->scratch_live != 0 || (control)->live_frames != 0 \
      || (control)->helper_live != 0 || (control)->helper_depths != 0) ++probe_errors; \
} while (0)
";

const HOST_MAIN: &str = r#"
static bool probe_released(void) {
  bool clean = tb_memory == NULL && tb_heap_meta == NULL && tb_host_current == NULL
    && tb_corpus_storage.address == NULL && tb_corpus_storage.capacity == 0
    && tb_corpus_storage.reserved == 0 && tb_corpus_storage.committed == 0 && tb_corpus_storage.page_size == 0
    && tb_metadata_storage.address == NULL && tb_metadata_storage.capacity == 0
    && tb_metadata_storage.reserved == 0 && tb_metadata_storage.committed == 0 && tb_metadata_storage.page_size == 0
    && tb_failure_guard == NULL && tb_task_current == NULL && !tb_cpu_active
    && tb_cpu.created == 0 && tb_cpu_pending == 0 && tb_cpu.host == NULL
    && tb_gpu_status == 0 && !tb_cuda.initialized && tb_cuda.context == NULL
    && tb_cuda.module == NULL && tb_cuda.stream == NULL && tb_cuda.source == NULL
    && tb_cuda.cache_path == NULL && tb_cuda.driver == NULL && tb_cuda.compiler == NULL;
  for (u32 i = 0; i < TB_CUDA_BUFFERS; ++i)
    if (tb_cuda.buffers[i].address != 0 || tb_cuda.buffers[i].capacity != 0) clean = false;
  return clean;
}
static int probe_write(const char *name) {
  FILE *receipt = fopen(name, "wb");
  if (receipt == NULL) return 95;
  for (u32 i = 0; i < 9; ++i) fprintf(receipt, "%llu ", probe_stats[i]);
  for (u32 i = 0; i < 32; ++i) fprintf(receipt, "%llu ", probe_receipt[i]);
  return fclose(receipt) == 0 ? 0 : 96;
}
int main(void) {
  probe_mode = PROBE_MODE;
  int status = checked_main();
  if (probe_mode != 0) {
    if (probe_write("failure-receipt.txt") != 0) return 90;
    if (status != 1 || probe_failures != 1 || probe_completions != 0 || probe_imports != 0
        || probe_cpu_replays != 0 || probe_errors != 0 || !probe_released()) return 91;
    probe_mode = 0;
    status = checked_main();
  }
  int saved = probe_write("receipt.txt");
  if (saved != 0) return saved;
  if (status != 0 || probe_completions != 1 || probe_errors != 0 || probe_imports != 1
      || probe_cpu_replays != 0 || !probe_released()) return 92;
  if (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 93;
  if (probe_receipt[16] != 0 || probe_receipt[0] == 0
      || probe_receipt[0] != probe_receipt[1] || probe_receipt[0] != probe_stats[3]
      || probe_receipt[6] != probe_receipt[7] || probe_receipt[20] != probe_stats[2]
      || probe_receipt[10] + probe_receipt[11] != probe_receipt[0] + probe_receipt[18]
      || probe_receipt[19] != probe_receipt[0] + probe_receipt[18] || probe_stats[6] != 0
      || probe_stats[5] != probe_stats[4]
      || probe_receipt[28] + probe_receipt[29] + probe_receipt[30] + probe_receipt[31] != probe_stats[0]) return 94;
  return 0;
}
"#;

fn compare_slices(bend: &str, scalar: bool, variable_drop: bool) -> (Receipt, Receipt) {
    let tiny = Fixture::new().run(bend, 1, 0);
    let large = Fixture::new().run(bend, 4096, 0);
    let expected_duplicate_ticks = if bend == NESTED_CLOSURE { 15 } else { 9 };
    for receipt in [&tiny, &large] {
        assert_eq!(receipt.observations[29], expected_duplicate_ticks);
        if variable_drop {
            assert!(
                receipt.observations[27] <= 3,
                "at most one destruction per input Fork"
            );
            assert_eq!(receipt.observations[28], 3 * receipt.observations[27]);
        }
    }
    if variable_drop {
        assert_eq!(
            tiny.statistics[0] - tiny.observations[28],
            large.statistics[0] - large.observations[28],
            "logical steps excluding independently scheduled destructor work"
        );
    } else {
        assert_eq!(tiny.statistics[0], large.statistics[0], "logical steps");
    }
    assert_eq!(tiny.statistics[2], large.statistics[2], "completed work");
    assert_eq!(tiny.statistics[3], large.statistics[3], "operations");
    assert!(tiny.statistics[3] > 0);
    assert!(tiny.statistics[1] > large.statistics[1] + 2);
    assert!(tiny.statistics[4] > 0);
    assert_eq!(large.statistics[4], 0);
    for event in [0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19] {
        assert_eq!(
            tiny.observations[event], large.observations[event],
            "ownership event {event} must not depend on slice size"
        );
    }
    assert!(tiny.observations[if scalar { 13 } else { 12 }] > 0);
    assert!(tiny.observations[21] > 0, "root remained unpublished");
    assert!(
        tiny.observations[22] > 0,
        "rewritten owners checked on resume"
    );
    (tiny, large)
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn duplicate_slices_preserve_typed_and_scalar_owners_and_logical_steps() {
    compare_slices(TREE, false, true);
    let (tiny, _) = compare_slices(SCALAR, true, false);
    assert!(tiny.observations[8] > 0, "partial child wrapping observed");
    assert!(
        tiny.observations[14] > 0,
        "suspended at nonzero dispatch turn"
    );
    assert!(
        tiny.observations[15] > 0,
        "resumed work crossed READY boundary"
    );
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn nested_closure_duplication_preserves_source_shells_and_clones_captures() {
    let (tiny, _) = compare_slices(NESTED_CLOSURE, false, true);
    assert!(tiny.observations[6] >= 2, "actual nested closure clones");
    assert!(
        tiny.observations[9] > 0,
        "closure-valued capture was cloned"
    );
    assert!(
        tiny.observations[5] > 0,
        "shared captured data was retained"
    );
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn interrupted_child_rewrite_discards_device_state_without_replay_and_recovers() {
    let receipt = Fixture::new().run(SCALAR, 1, 1);
    assert!(receipt.observations[8] > 0);
    assert_eq!(receipt.observations[17], 0, "recovery used fresh state");
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn pending_duplicate_capacity_refusal_preserves_allocator_and_recovers() {
    let receipt = Fixture::new().run(SCALAR, 1, 2);
    assert!(receipt.observations[8] > 0);
    assert_eq!(
        receipt.observations[17], 0,
        "recovery restored full capacity"
    );
}
