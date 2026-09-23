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

const TYPED: &str = r"import Base
def seed(x: U32) -> U32: U32.add(x, 12345)
def make(depth: Nat, x: U32) -> Array<U32>: Array.new(U32, depth, seed(x))
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def main() -> U32: pick(Array.get(U32, make!(6n, 6), 63))
";

const CLOSURE: &str = r"import Base
def seed(x: U32) -> U32: U32.add(x, 12345)
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def apply(f: U32 -> U32, x: U32) -> U32: f(x)
def builder() -> U32 -> U32:
  x => pick(Array.get(U32, Array.new(U32, 6n, seed(x)), 63))
def main() -> U32: apply!(builder(), 6)
";

const SIBLINGS: &str = r"import Base
def seed(x: U32) -> U32: U32.add(x, 12345)
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def raw(depth: Nat, x: U32) -> U32:
  pick(Array.get(U32, Array.new(U32, depth, seed(x)), 63))
def walk(n: Nat, x: U32) -> U32:
  match n:
    case 0n: x
    case 1n+p: walk(p, U32.add(x, 1))
def arrays(+depth: Nat) -> U32:
  left right = raw(depth, 6) raw(depth, 8)
  U32.add(left, right)
def mixed(depth: Nat) -> U32:
  a b = arrays(depth) walk(12n, 1)
  U32.add(a, b)
def main() -> U32: mixed!(6n)
";

const DEPTH_ZERO: &str = r"import Base
def make(depth: Nat) -> Array<U32>: Array.new(U32, depth, 7)
def main() -> Array<U32>: make!(0n)
";

const PADDED: &str = r"import Base
type Three is Data: Three{a: U32, b: U32, c: U32}
def make(depth: Nat) -> Array<Three>: Array.new(Three, depth, Three{1, 2, 3})
def pick(pair: Array<Three> & Three) -> Three:
  (array, value) = pair
  value
def main() -> Three: pick(Array.get(Three, make!(2n), 3))
";

const UNIT: &str = r"import Base
def make(depth: Nat) -> Array<Unit>: Array.new(Unit, depth, Unit{})
def main() -> Array<Unit>: make!(1n)
";

const WIDE: &str = r"import Base
def double(n: Nat, +x: Nat) -> Nat:
  match n:
    case 0n: x
    case 1n+p: double(p, Nat.add(x, x))
def make(depth: Nat) -> Array<Nat>: Array.new(Nat, depth, Nat.add(double(40n, 1n), 7n))
def pick(pair: Array<Nat> & Nat) -> Nat:
  (array, value) = pair
  value
def main() -> Nat: pick(Array.get(Nat, make!(2n), 3))
";

const PACKED_CLONE: &str = r"import Base
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def sum_pair(pair: Array<U32> & Array<U32>) -> U32:
  (left, right) = pair
  U32.add(pick(Array.get(U32, left, 4095)), pick(Array.get(U32, right, 0)))
def run_copy(array: Array<U32>) -> U32:
  sum_pair(Array.clone(U32, array))
def main() -> U32: run_copy!(Array.new(U32, 12n, 7))
";

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-gpu-primitives-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, bend: &str, quantum: u32, mode: u32, expected: &str) -> Vec<u64> {
        let path = self.0.join("main.bend");
        fs::write(&path, bend).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let source = instrument(generated, mode, bend.contains("12345"));
        let definition = format!("BEND_GPU_PRIMITIVE_QUANTUM={quantum}");
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4", &definition],
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
        assert_eq!(output.stdout, expected.as_bytes());
        let expected_error = match mode {
            0 => "",
            1 => "teamy-bend executable C: primitive test interruption\n",
            2 => "teamy-bend executable C: VM allocation budget exhausted\n",
            _ => unreachable!(),
        };
        assert_eq!(output.stderr, expected_error.as_bytes());
        fs::read_to_string(self.0.join("receipt.txt"))
            .unwrap()
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU primitive test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn instrument(mut generated: String, mode: u32, count_seed: bool) -> String {
    let start = generated
        .find("static const char *const tb_gpu_source_parts[] = {\n")
        .unwrap();
    let end = start + generated[start..].find("\n};").unwrap() + 3;
    let mut device = decode_parts(&generated[start..end]);
    let clone_prefix = "  if (!tb_device_array_clone_raw(e, tb_frame, &tb_values[";
    if let Some(at) = device.find(clone_prefix) {
        let owner_start = at + clone_prefix.len();
        let owner_end = owner_start + device[owner_start..].find(']').unwrap();
        let owner = device[owner_start..owner_end].to_owned();
        let share = format!(
            "  if (tb_device_control->root_words[233] == 0) probe_share_array(e, &tb_values[{owner}]);\n"
        );
        device.insert_str(at, &share);
        let call_start = at + share.len();
        let endif = call_start + device[call_start..].find("\n#endif").unwrap() + 7;
        device.insert_str(endif, "\n  probe_release_array_alias(e);");
    }
    let mut insertions = Vec::new();
    for (at, _) in device.match_indices("  if (!tb_device_array_new_raw(") {
        let unbox = device[..at].rfind("  tb_unbox_").unwrap();
        assert!(!device[unbox..at].contains("\n}"));
        insertions.push((unbox, "  probe_unbox();\n"));
    }
    assert!(
        !insertions.is_empty() || device.contains("tb_device_array_clone_raw("),
        "the instrumented program must exercise a bounded array primitive"
    );
    if count_seed {
        let mut offset = 0;
        let mut sites = 0;
        for line in device.split_inclusive('\n') {
            if line.contains("12345") && line.contains('=') {
                insertions.push((offset, "  probe_seed();\n"));
                sites += 1;
            }
            offset += line.len();
        }
        assert!(sites > 0, "seed constant assignment must be instrumented");
    }
    insertions.sort_unstable_by_key(|(at, _)| *at);
    insertions.dedup();
    for (at, insertion) in insertions.into_iter().rev() {
        device.insert_str(at, insertion);
    }
    device.insert_str(0, DEVICE_PREFIX);
    device.push_str(DEVICE_OBSERVER);
    generated.replace_range(start..end, &encode_parts(&device));

    let upload = "  tb_gpu_require(tb_cuda_upload(TB_GPU_STATE, 0, &state, sizeof(state)));";
    assert_eq!(generated.matches(upload).count(), 1);
    generated = generated.replace(upload, &format!("  state.reserved = probe_mode;\n{upload}"));
    let download =
        "    tb_gpu_require(tb_cuda_download(TB_GPU_CONTROL, 0, &control, sizeof(control)));";
    assert_eq!(generated.matches(download).count(), 1);
    generated = generated.replace(
        download,
        &format!("{download}\n    PROBE_ROUND(&state, &control);"),
    );
    let cpu = "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {";
    assert_eq!(generated.matches(cpu).count(), 1);
    generated = generated.replace(
        cpu,
        &format!("{cpu}\n  if (tb_gpu_marked((Fid)term_aux(task))) ++probe_cpu_replays;"),
    );
    let main = "static int tb_program_main(void)";
    assert_eq!(generated.matches(main).count(), 1);
    generated = generated.replace(main, "#define TB_NO_MAIN 1\nstatic int checked_main(void)");
    let mut source = String::from(HOST_PREFIX);
    source.push_str(&generated);
    writeln!(source, "\n#define PROBE_MODE {mode}").unwrap();
    source.push_str(HOST_MAIN);
    source
}

// Decode only the compiler's transport strings, then preserve the same small
// chunks. This lets the tests observe generated device call sites without
// changing production emitter interfaces or the eight-cell primitive ABI.
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

const DEVICE_PREFIX: &str = r"
static __device__ void probe_observe(unsigned int, unsigned long long *, unsigned long long,
  const unsigned long long *, unsigned int);
static __device__ void probe_payload_observe(unsigned int, unsigned long long *, unsigned long long,
  const unsigned long long *, const unsigned long long *, unsigned int, bool);
static __device__ void probe_duplicate_observe(unsigned int, unsigned long long *, void *,
  unsigned long long);
static __device__ void probe_seed(void);
static __device__ void probe_unbox(void);
static __device__ void probe_array_copy_observe(unsigned int, unsigned long long *, unsigned long long);
static __device__ void probe_share_array(const void *, void *);
static __device__ void probe_release_array_alias(const void *);
#define TB_DEVICE_ARRAY_NEW_OBSERVE(event, state, work) probe_observe(event, state, work, raw_values, count)
#define TB_DEVICE_ARRAY_COPY_OBSERVE(event, state, work) probe_array_copy_observe(event, state, work)
#define TB_DEVICE_PAYLOAD_OBSERVE(event, state, work, fields, mask, count, closure) \
  probe_payload_observe(event, state, work, fields, mask, count, closure)
#define TB_DEVICE_DUPLICATE_OBSERVE(event, state, frame, work) \
  probe_duplicate_observe(event, state, (void *)(frame), work)
";

// The fixture roots contain at most three words. The otherwise unused tail
// carries test receipts; ordinary values and root ownership are never touched.
// Record offsets: new entries, reserves, slices, completions, seeds, unboxes,
// errors, initialization words, fill slots, fault, bump, live words, blocks,
// free-list fingerprint, initialized-prefix observation.
const DEVICE_OBSERVER: &str = r#"
typedef struct {
  Term *key;
  u64 progress, starts, finishes, values[3];
} ProbeOperation;
static __device__ ProbeOperation probe_operations[8];
static __device__ u32 probe_lock, probe_operation_count;
static __device__ void probe_seed(void) { atomicAdd(tb_device_control->root_words + 244, 1ull); }
static __device__ void probe_unbox(void) { atomicAdd(tb_device_control->root_words + 245, 1ull); }
static __device__ void probe_payload_observe(u32 event, Term *state, u64 work,
    const Term *fields, const Term *mask, u32 count, bool closure) {
  (void)state; (void)fields; (void)mask; (void)count; (void)closure;
  if (event == TB_DEVICE_PAYLOAD_SLICE)
    atomicAdd(tb_device_control->root_words + 239, work);
  else if (event == TB_DEVICE_PAYLOAD_CTR_RESERVED
      || event == TB_DEVICE_PAYLOAD_CLO_RESERVED)
    atomicAdd(tb_device_control->root_words + 238, 1ull);
}
static __device__ void probe_duplicate_observe(u32 event, Term *state,
    void *frame, u64 work) {
  (void)state; (void)frame;
  if (event == TB_DEVICE_DUPLICATE_SLICE)
    atomicAdd(tb_device_control->root_words + 239, work);
  else if (event == TB_DEVICE_DUPLICATE_START)
    atomicAdd(tb_device_control->root_words + 238, 1ull);
}
static __device__ void probe_array_copy_observe(u32 event, Term *state, u64 work) {
  (void)state;
  u64 *receipt = tb_device_control->root_words;
  if (event == TB_DEVICE_ARRAY_COPY_RESERVED)
    atomicAdd(receipt + 234, 1ull);
  else if (event == TB_DEVICE_ARRAY_COPY_START)
    atomicAdd(receipt + 235, 1ull);
  else if (event == TB_DEVICE_ARRAY_COPY_SLICE) {
    atomicAdd(receipt + 236, work);
    atomicAdd(receipt + 237, 1ull);
  }
}
static __device__ void probe_share_array(const void *raw_env, void *raw_owner) {
  const Env *e = (const Env *)raw_env;
  Term *owner = (Term *)raw_owner;
  if (tb_device_control->root_words[233] != 0)
    err_fail("array clone probe alias already set");
  tb_device_control->root_words[233] = tb_c_duplicate(e, owner);
}
static __device__ void probe_release_array_alias(const void *raw_env) {
  const Env *e = (const Env *)raw_env;
  Term alias = tb_device_control->root_words[233];
  if (alias == 0) err_fail("array clone probe alias is missing");
  tb_device_control->root_words[233] = 0;
  term_drop(*e, alias);
}
static __device__ u64 probe_free_hash(void) {
  u64 hash = 0;
  for (u32 i = 0; i < NCLS_ALL; ++i) hash = (hash * 33u) ^ tb_free_lists[i];
  return hash;
}
static __device__ void probe_observe(u32 event, Term *state, u64 work,
    const Term *values, u32 count) {
  if (tb_device_control->root_count > 3 || count > 3) err_fail("invalid primitive test fixture");
  for (;;) {
    tb_device_check_cancelled();
    if (atomicCAS(&probe_lock, 0u, 1u) == 0) break;
    __nanosleep(64);
  }
  u64 *receipt = tb_device_control->root_words + 240;
  u32 index = 0;
  for (; index < probe_operation_count; ++index)
    if (probe_operations[index].key == state) break;
  if (index == probe_operation_count) {
    if (index == 8 || event != TB_DEVICE_ARRAY_NEW_ENTER || state[0] != 0)
      err_fail("invalid primitive test record");
    ++probe_operation_count; probe_operations[index].key = state;
    for (u32 i = 0; i < count; ++i) probe_operations[index].values[i] = values[i];
    ++receipt[0];
  } else if (event == TB_DEVICE_ARRAY_NEW_ENTER && state[0] == 0) ++receipt[6];
  ProbeOperation *record = probe_operations + index;
  for (u32 i = 0; i < count; ++i)
    if (record->values[i] != values[i]) ++receipt[6];
  if (event == TB_DEVICE_ARRAY_NEW_ENTER && state[0] == 0 && tb_device_state->reserved == 2) {
    // TYPED requests a fresh 32-word packed block. Refuse before any allocator
    // mutation, at the actual helper's capacity boundary, not task creation.
    if (tb_free_lists[5] != 0) err_fail("capacity test requires a fresh block");
    receipt[10] = tb_bump; receipt[11] = tb_live_words; receipt[12] = tb_live_blocks;
    receipt[13] = probe_free_hash(); receipt[9] = 2;
    tb_device_state->capacity = tb_bump;
  }
  if (event == TB_DEVICE_ARRAY_NEW_RESERVED) {
    ++receipt[1]; ++record->starts;
    if (record->starts != 1 || state[0] != 1 || state[2] != 0 || record->progress != 0)
      ++receipt[6];
  }
  if (event == TB_DEVICE_ARRAY_NEW_SLICE) {
    ++receipt[2];
    u64 words = 1ull << state[4], elements = 1ull << state[3];
    u64 progress = state[0] == 1 ? state[2] : words + state[2];
    if (work == 0 || work > BEND_GPU_PRIMITIVE_QUANTUM
        || progress <= record->progress || progress - record->progress != work
        || progress > words + elements) ++receipt[6];
    u64 old_init = record->progress < words ? record->progress : words;
    u64 new_init = progress < words ? progress : words;
    receipt[7] += new_init - old_init;
    receipt[8] += (progress - new_init) - (record->progress - old_init);
    record->progress = progress;
    if (state[0] == 1) {
      u64 metadata = tb_meta(state[1], (Cls)state[4]) | (state[7] ? 0 : TB_META_OWNED);
      for (u64 i = 0; i < state[2]; ++i)
        if (tb_memory[state[1] + i] != 0 || tb_heap_meta[state[1] + i] != metadata)
          ++receipt[6];
      ++receipt[14];
    }
    if (tb_device_state->reserved == 1 && state[0] == 1 && state[2] > 0) {
      receipt[9] = 1;
      __threadfence(); atomicExch(&probe_lock, 0u);
      err_fail("primitive test interruption");
    }
  }
  if (event == TB_DEVICE_ARRAY_NEW_COMPLETE) {
    ++receipt[3]; ++record->finishes;
    u64 words = 1ull << state[4], elements = 1ull << state[3];
    if (record->starts != 1 || record->finishes != 1 || state[0] != 3
        || state[2] != elements || record->progress != words + elements) ++receipt[6];
    u64 metadata = tb_meta(state[1], (Cls)state[4]) | (state[7] ? 0 : TB_META_OWNED);
    for (u64 i = 0; i < words; ++i) {
      u64 expected = 0;
      if (state[7]) {
        u32 column = (u32)i & ((u32)state[5] - 1);
        if (column < state[6]) expected = record->values[column];
      } else {
        for (u32 half = 0; half < 2; ++half) {
          u64 slot = 2 * i + half;
          u32 column = (u32)slot & ((u32)state[5] - 1);
          if (slot < elements && column < state[6])
            expected |= (u64)(u32)record->values[column] << (32 * half);
        }
      }
      if (tb_memory[state[1] + i] != expected || tb_heap_meta[state[1] + i] != metadata)
        ++receipt[6];
    }
  }
  __threadfence(); atomicExch(&probe_lock, 0u);
}
"#;

const HOST_PREFIX: &str = r"
static unsigned int probe_mode, probe_completions, probe_failures, probe_errors, probe_cpu_replays;
static unsigned long long probe_stats[9], probe_receipt[15];
static unsigned long long probe_copy_stats[4];
static unsigned long long probe_nested_progress, probe_nested_starts;
#define PROBE_ROUND(state, control) do { \
  if ((control)->root_count > 3) ++probe_errors; \
  if ((state)->error != 0) { \
    ++probe_failures; \
    memcpy(probe_receipt, (control)->root_words + 240, sizeof(probe_receipt)); \
    if (probe_receipt[6] != 0 || probe_receipt[9] != probe_mode) ++probe_errors; \
    if (probe_mode == 1 && (probe_receipt[1] != 1 || probe_receipt[3] != 0 \
        || probe_receipt[7] == 0 || probe_receipt[8] != 0 || (control)->primitive_live != 1)) \
      ++probe_errors; \
    if (probe_mode == 2) { \
      u64 hash = 0; \
      for (u32 i = 0; i < NCLS_ALL; ++i) hash = (hash * 33u) ^ (state)->free_lists[i]; \
      if (probe_receipt[1] != 0 || probe_receipt[2] != 0 || probe_receipt[3] != 0 \
          || (control)->primitive_starts != 0 || (control)->primitive_live != 0 \
          || (state)->bump != probe_receipt[10] || (state)->live_words != probe_receipt[11] \
          || (state)->live_blocks != probe_receipt[12] || hash != probe_receipt[13]) ++probe_errors; \
    } \
  } \
} while (0)
#define TB_GPU_COMPLETE(control, state, info) do { \
  ++probe_completions; \
  if ((control)->root_words[233] != 0) ++probe_errors; \
  memcpy(probe_receipt, (control)->root_words + 240, sizeof(probe_receipt)); \
  probe_nested_progress = (control)->root_words[239]; \
  probe_nested_starts = (control)->root_words[238]; \
  probe_copy_stats[0] = (control)->root_words[234]; \
  probe_copy_stats[1] = (control)->root_words[235]; \
  probe_copy_stats[2] = (control)->root_words[236]; \
  probe_copy_stats[3] = (control)->root_words[237]; \
  probe_stats[0] = (state)->steps; probe_stats[1] = (info)->launches; \
  probe_stats[2] = (control)->primitive_progress; probe_stats[3] = (control)->primitive_starts; \
  probe_stats[4] = (control)->primitive_yields; probe_stats[5] = (control)->primitive_requeues; \
  probe_stats[6] = (control)->primitive_live; probe_stats[7] = (control)->forks; \
  probe_stats[8] = (control)->peak_lanes; \
} while (0)
";

const HOST_MAIN: &str = r#"
static bool probe_released(void) {
  return tb_memory == NULL && tb_heap_meta == NULL && tb_host_current == NULL
    && tb_failure_guard == NULL && tb_task_current == NULL && !tb_cpu_active
    && tb_cpu.created == 0 && tb_cpu_pending == 0 && tb_cpu.host == NULL
    && tb_gpu_status == 0 && !tb_cuda.initialized && tb_cuda.context == NULL
    && tb_cuda.module == NULL && tb_cuda.stream == NULL && tb_cuda.source == NULL;
}
int main(void) {
  probe_mode = PROBE_MODE;
  int status = checked_main();
  if (probe_mode != 0) {
    if (status != 1 || probe_failures != 1 || probe_completions != 0
        || probe_cpu_replays != 0 || probe_errors != 0 || !probe_released()) return 91;
    // Discarding the failed invocation must permit the identical Bend program
    // and device source to run in the same process with only fault mode cleared.
    probe_mode = 0;
    status = checked_main();
  }
  if (status != 0 || probe_completions != 1 || probe_errors != 0
      || probe_cpu_replays != 0 || !probe_released()) return 92;
  if (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 93;
  if (probe_receipt[6] != 0 || (probe_receipt[0] == 0 && probe_copy_stats[1] == 0)
      || probe_receipt[0] != probe_receipt[1] || probe_receipt[0] != probe_receipt[3]
      || probe_receipt[0] != probe_receipt[5]
      || probe_receipt[0] + probe_copy_stats[1] + probe_nested_starts != probe_stats[3]
      || probe_receipt[7] + probe_receipt[8] + probe_copy_stats[2]
          + probe_nested_progress != probe_stats[2] || probe_stats[6] != 0
      || probe_stats[5] != probe_stats[4]) {
    fprintf(stderr, "array probe mismatch: errors=%llu operations=%llu/%llu/%llu/%llu/%llu work=%llu+%llu+%llu/%llu starts=%llu+%llu+%llu/%llu live=%llu requeues=%llu yields=%llu\n",
      probe_receipt[6], probe_receipt[0], probe_receipt[1], probe_receipt[3],
      probe_receipt[5], probe_stats[3], probe_receipt[7] + probe_receipt[8], probe_copy_stats[2], probe_nested_progress,
      probe_stats[2], probe_receipt[0], probe_copy_stats[1], probe_nested_starts, probe_stats[3], probe_stats[6],
      probe_stats[5], probe_stats[4]);
    return 94;
  }
  FILE *receipt = fopen("receipt.txt", "wb");
  if (receipt == NULL) return 95;
  for (u32 i = 0; i < 9; ++i) fprintf(receipt, "%llu ", probe_stats[i]);
  for (u32 i = 0; i < 15; ++i) fprintf(receipt, "%llu ", probe_receipt[i]);
  fprintf(receipt, "%llu %llu", probe_nested_progress, probe_nested_starts);
  for (u32 i = 0; i < 4; ++i) fprintf(receipt, " %llu", probe_copy_stats[i]);
  if (fclose(receipt) != 0) return 96;
  return 0;
}
"#;

fn compare_slices(bend: &str, expected: &str, operations: u64, forks: bool) {
    let tiny = Fixture::new().run(bend, 1, 0, expected);
    let large = Fixture::new().run(bend, 4096, 0, expected);
    assert_eq!(tiny.len(), 30);
    assert_eq!(large.len(), 30);
    assert_eq!(
        tiny[0], large[0],
        "primitive slices must not charge language steps"
    );
    assert_eq!(tiny[2], large[2], "the same payload work must complete");
    assert_eq!(tiny[9], operations, "array helper starts must be exact");
    assert_eq!(large[9], operations, "array helper starts must be exact");
    assert_eq!(
        tiny[3],
        operations + tiny[25],
        "all helper starts must be accounted"
    );
    assert_eq!(
        large[3],
        operations + large[25],
        "all helper starts must be accounted"
    );
    assert!(
        tiny[1] > large[1] + 2,
        "small slices must cause actual extra CUDA launches"
    );
    assert!(tiny[4] > 0);
    assert_eq!(large[4], 0);
    assert_eq!(tiny[13], operations, "seed must run once per operation");
    assert_eq!(large[13], operations);
    assert!(tiny[23] > 0, "observe a partially initialized allocation");
    if forks {
        assert!(tiny[7] >= 2 && large[7] >= 2);
        assert!(tiny[8] > 1 && large[8] > 1);
    }
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn packed_array_clone_copies_the_full_block_once_across_slices() {
    let tiny = Fixture::new().run(PACKED_CLONE, 1, 0, "14\n");
    let large = Fixture::new().run(PACKED_CLONE, 4096, 0, "14\n");
    assert_eq!(tiny.len(), 30);
    assert_eq!(large.len(), 30);
    assert_eq!(
        tiny[0], large[0],
        "copy slices must preserve language steps"
    );
    assert_eq!(tiny[2], large[2], "the same total work must complete");
    assert_eq!(
        tiny[26], 2,
        "shared input needs COW plus clone reservations"
    );
    assert_eq!(
        large[26], 2,
        "shared input needs COW plus clone reservations"
    );
    assert_eq!(tiny[27], 1, "one resumable clone operation must start");
    assert_eq!(large[27], 1, "one resumable clone operation must start");
    assert_eq!(tiny[28], 8192, "both copies initialize and copy 2048 words");
    assert_eq!(tiny[28], large[28], "copy work must not replay");
    assert_eq!(
        tiny[29], 8192,
        "quantum one must process one word per slice"
    );
    assert_eq!(large[29], 2, "each block copy completes in one large slice");
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn raw_array_slices_preserve_typed_and_dynamic_closure_operands_and_steps() {
    compare_slices(TYPED, "12351\n", 1, false);
    compare_slices(CLOSURE, "12351\n", 1, false);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn raw_array_slices_preserve_sibling_forks_and_ordinary_task_steps() {
    compare_slices(SIBLINGS, "24717\n", 2, true);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn raw_array_slices_initialize_depth_zero_padding_unit_and_wide_layouts() {
    for (bend, expected, progress) in [
        (DEPTH_ZERO, "[7]\n", 2),
        (PADDED, "Three{1, 2, 3}\n", 28),
        (UNIT, "[Unit{}, Unit{}]\n", 3),
        (WIDE, "1099511627783n\n", 8),
    ] {
        let receipt = Fixture::new().run(bend, 1, 0, expected);
        assert_eq!(receipt[2], progress);
        assert_eq!(receipt[9], 1, "one Array.new helper must complete");
        assert_eq!(receipt[3], receipt[9] + receipt[25]);
        assert!(receipt[4] > 0);
    }
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn interrupted_raw_array_initialization_cleans_up_without_replay_and_recovers() {
    Fixture::new().run(TYPED, 1, 1, "12351\n");
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn raw_array_capacity_failure_precedes_allocator_mutation_and_recovers() {
    Fixture::new().run(TYPED, 1, 2, "12351\n");
}
