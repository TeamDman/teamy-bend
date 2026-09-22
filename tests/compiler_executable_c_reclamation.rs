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
const SMALL_HEAP: &[&str] = &["BEND_MAX_ALLOC=4096"];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-reclamation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, bend: &str, foreign: &str, require_released: bool) -> Output {
        self.run_with_definitions(bend, foreign, require_released, SMALL_HEAP)
    }

    fn run_with_definitions(
        &self,
        bend: &str,
        foreign: &str,
        require_released: bool,
        definitions: &[&str],
    ) -> Output {
        fs::write(self.0.join("main.bend"), bend).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let mut generated = compile_executable_c(&checked).unwrap();
        if require_released {
            // Observe real allocator ownership after the emitted program returns;
            // bulk arena teardown must not hide an unreleased reachable value.
            assert_eq!(
                generated
                    .matches("static int tb_program_main(void)")
                    .count(),
                1
            );
            generated = generated.replace(
                "static int tb_program_main(void)",
                "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
            );
            generated.push_str(
                r#"
int main(void) {
  int status = checked_main();
  if (tb_live_words != 0 || tb_live_blocks != 0) {
    (void)fprintf(stderr, "unreleased VM allocations: %llu words, %llu blocks\n",
      (unsigned long long)tb_live_words, (unsigned long long)tb_live_blocks);
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

const SHARED_BRANCHES: &str = r#"import Base
def choose(flag: Bool, left: String, right: String) -> String:
  match flag:
    case True{}: left
    case False{}: right

def rounds(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p:
      +remaining = p
      +index = U32.from_nat(remaining)
      +shared = String.append("dynamic:", U32.show(index))
      kept = choose(U32.is_eq(U32.and(index, 1), 0), shared, String.append(shared, "!"))
      rounds(remaining, U32.add(total, U32.from_nat(String.length(kept))))

def main() -> U32: rounds(64n, 0)
"#;

#[test]
fn dynamic_shared_strings_and_unused_branch_owners_reuse_a_small_heap() {
    // Each round constructs several linked strings. Their cumulative storage
    // exceeds 4 KiB, while only a single round's strings need remain live.
    success(&Fixture::new().run(SHARED_BRANCHES, "", true), "662\n");
}

const CAPTURED_CLOSURES: &str = r#"import Base
def make(prefix: String) -> U32 -> String:
  x => String.append(prefix, U32.show(x))

def lengths(pair: String & String) -> U32:
  (left, right) = pair
  U32.add(U32.from_nat(String.length(left)), U32.from_nat(String.length(right)))

def rounds(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p:
      +remaining = p
      +index = U32.from_nat(remaining)
      +prefix = String.append("captured:", U32.show(index))
      first = make(prefix)
      second = make(prefix)
      count = lengths((first(1), second(22)))
      rounds(remaining, U32.add(total, count))

def main() -> U32: rounds(64n, 0)
"#;

#[test]
fn separate_closures_preserve_shared_captured_strings_and_release_both_owners() {
    success(&Fixture::new().run(CAPTURED_CLOSURES, "", true), "1580\n");
}

const DUPLICATED_CLOSURE: &str = r#"import Base
def probe() -> IO(U32 & U32): import "effect.c"
def show(pair: U32 & U32) -> IO(Unit):
  (left, right) = pair
  do IO<Unit>:
    Unit <- IO.print(U32.show(left))
    IO.print(U32.show(right))
def main() -> IO(Unit): IO.bind(U32 & U32, Unit, probe(), show)
"#;

const DUPLICATED_CLOSURE_C: &str = r#"
static Term inner_apply(Env e, const Term *captures, Term argument) {
  u64 length;
  char *text = io_cstr(e, captures[0], &length);
  if (length != 6 || memcmp(text, "shared", 6) != 0) err_fail("inner capture changed");
  free(text);
  return argument + length;
}
static Term outer_apply(Env e, const Term *captures, Term argument) {
  Term result = tb_apply(e, captures[0], argument);
  u64 length;
  char *text = io_cstr(e, captures[1], &length);
  if (length != 4 || memcmp(text, "tail", 4) != 0) err_fail("outer capture changed");
  free(text);
  return result + length;
}
static Term probe_run(Env e, Term *fields, IoWork *work) {
  Term inner_fields[1] = {io_str(e, "shared", 6)};
  Term outer_fields[2] = {tb_closure(e, 62000, 1, inner_fields), io_str(e, "tail", 4)};
  Term original = tb_closure(e, 62001, 2, outer_fields);
  Term copied = tb_duplicate(e, &original);
  Term unused = tb_duplicate(e, &original);
  Term left, right;
  (void)fields; (void)work;
  term_drop(e, unused);
  left = tb_apply(e, original, 3);
  right = tb_apply(e, copied, 7);
  return io_tup(e, left, right);
}
static void __attribute__((constructor)) probe_use(void) {
  tb_register_closure(62000, inner_apply, 1);
  tb_register_closure(62001, outer_apply, 2);
  io_eff(CID_PROBE, probe_run, 0);
}
"#;

#[test]
fn trusted_foreign_closure_duplication_copies_nested_closures_and_releases_unused_captures() {
    // Function values are affine in checked Bend. The explicit helper path is
    // exercised through the trusted C ABI, not by weakening that type boundary.
    success(
        &Fixture::new().run(DUPLICATED_CLOSURE, DUPLICATED_CLOSURE_C, true),
        "13\n17\n",
    );
}

const NESTED_ARRAYS: &str = r#"import Base
def inner_length(pair: Array<String> & String) -> U32:
  (rest, value) = pair
  U32.from_nat(String.length(value))

def finish(pair: Array<Array<String>> & Array<String>, first: U32) -> U32:
  (rest, second) = pair
  U32.add(first, inner_length(Array.get(String, second, 0)))

def extract(pair: Array<Array<String>> & Array<String>) -> U32:
  (rest, first) = pair
  finish(Array.swap(Array<String>, rest, 1, ALeaf{"replacement"}),
    inner_length(Array.get(String, first, 0)))

def observe(pair: Array<String> & Array<String>) -> U32:
  (left, right) = pair
  outer = {ANode{ALeaf{Array.set(String, left, 0, "changed")}, ALeaf{right}} : Array<Array<String>>}
  extract(Array.swap(Array<String>, outer, 0, ALeaf{"replacement"}))

def rounds(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p:
      inner = Array.new(String, 1n, String.append("seed", U32.show(7)))
      count = observe(Array.clone(String, inner))
      rounds(p, U32.add(total, count))

def main() -> U32: rounds(64n, 0)
"#;

#[test]
fn nested_string_array_clones_and_updates_preserve_the_original_and_drop_old_elements() {
    success(&Fixture::new().run(NESTED_ARRAYS, "", true), "768\n");
}

const MIXED_ARRAY: &str = r#"import Base
type Cell is Data:
  Number{number: Nat}
  Text{text: String}

def expose(pair: Array<Cell> & Cell) -> Cell & List<Cell>:
  (array, old) = pair
  (old, Array.to_list(~Cell, array))

def change(pair: Array<Cell> & Array<Cell>) -> (Cell & List<Cell>) & List<Cell>:
  (left, right) = pair
  changed = Array.set(Cell, left, 1, Text{"changed"})
  (expose(Array.swap(Cell, changed, 0, Text{"new"})), Array.to_list(~Cell, right))

def inspect(pair: Array<Cell> & Cell) -> Cell & ((Cell & List<Cell>) & List<Cell>):
  (array, observed) = pair
  (observed, change(Array.clone(Cell, array)))

def main() -> Cell & ((Cell & List<Cell>) & List<Cell>):
  inspect(Array.get(Cell, ANode{ALeaf{Number{Nat.pow(2n, 47n)}},
    ALeaf{Text{String.append("old", U32.show(7))}}}, 0))
"#;

#[test]
fn mixed_nat_and_string_array_get_clone_set_and_swap_preserve_each_representation() {
    success(
        &Fixture::new().run(MIXED_ARRAY, "", true),
        "(Number{140737488355328n}, (Number{140737488355328n}, [Text{\"new\"}, Text{\"changed\"}]), [Number{140737488355328n}, Text{\"old7\"}])\n",
    );
}

const CHANNEL_REUSE: &str = r#"import Base
def churn(n: Nat, total: U32) -> IO(U32):
  match n:
    case 0n: IO.pure(U32, total)
    case 1n+p:
      +remaining = p
      +index = U32.from_nat(remaining)
      count = U32.from_nat(String.length(String.append("reclaimed:", U32.show(index))))
      do IO<U32>:
        Unit <- IO.sleep(0)
        churn(remaining, U32.add(total, count))

def send(channel: Chan(String), text: String) -> IO(Unit):
  do IO<Unit>:
    accepted : Bool <- Chan.send(String, channel, text)
    return Unit{}

def received(value: Maybe<&1, String>) -> String:
  match value:
    case None{}: "missing"
    case Some{text}: text

def use(channel: Chan(String), label: String) -> IO(Unit):
  +saved = channel
  do IO<Unit>:
    Unit <- IO.spawn(Unit, send(saved, String.append("queued:", U32.show(314))))
    count : U32 <- churn(64n, 0)
    item : Maybe<&1, String> <- Chan.recv(String, saved)
    Unit <- IO.print(String.append(label, received(item)))
    Unit <- IO.print(U32.show(count))
    Chan.close(String, saved)

def main() -> IO(Unit):
  IO.bind(Chan(String), Unit, Chan.new(String, CAPACITY), channel =>
    use(channel, String.append("received-", String.append(U32.show(7), ":"))))
"#;

#[test]
fn parked_continuations_and_buffered_or_waiting_channel_payloads_survive_heap_reuse() {
    for capacity in [0, 1] {
        success(
            &Fixture::new().run(
                &CHANNEL_REUSE.replace("CAPACITY", &capacity.to_string()),
                "",
                true,
            ),
            "received-7:queued:314\n758\n",
        );
    }
}

const CHANNEL_HALT: &str = r#"import Base
def send(channel: Chan(String), text: String) -> IO(Unit):
  do IO<Unit>:
    accepted : Bool <- Chan.send(String, channel, text)
    IO.print("child-completed")

def use(channel: Chan(String)) -> IO(Unit):
  +saved = channel
  do IO<Unit>:
    Unit <- IO.spawn(Unit, send(saved, String.append("pending:", U32.show(314))))
    Unit <- IO.sleep(0)
    IO.die(Unit, 7, "halt")

def main() -> IO(Unit):
  IO.bind(Chan(String), Unit, Chan.new(String, CAPACITY), use)
"#;

#[test]
fn halt_releases_buffered_payloads_and_blocked_senders_with_their_continuations() {
    for capacity in [0, 1] {
        let output = Fixture::new().run(
            &CHANNEL_HALT.replace("CAPACITY", &capacity.to_string()),
            "",
            true,
        );
        assert_eq!(output.status.code(), Some(7));
        assert_eq!(
            output.stdout,
            if capacity == 0 {
                b"".as_slice()
            } else {
                b"child-completed\n".as_slice()
            }
        );
        assert_eq!(output.stderr, b"halt\n");
    }
}

const DEEP_DROP: &str = r#"import Base
def deep() -> IO(String): import "effect.c"
def main() -> IO(Unit):
  IO.bind(String, Unit, deep(), unused => IO.print("deep-drop"))
"#;

const DEEP_DROP_C: &str = r"
static Term deep_run(Env e, Term *fields, IoWork *work) {
  const size_t count = 20000;
  char *text = io_mem(malloc(count));
  Term result;
  (void)fields; (void)work;
  memset(text, 'a', count);
  result = io_str(e, text, count);
  free(text);
  return result;
}
static void __attribute__((constructor)) deep_use(void) {
  io_eff(CID_DEEP, deep_run, 0);
}
";

#[test]
fn an_unused_generated_io_parameter_drops_a_deep_native_string_without_native_recursion() {
    // The C import constructs a valid native chain without consuming the
    // language's recursion budget; the checked continuation owns its drop.
    success(
        &Fixture::new().run_with_definitions(DEEP_DROP, DEEP_DROP_C, true, &[]),
        "deep-drop\n",
    );
}

const PROBE: &str = r#"import Base
def probe() -> IO(Unit): import "effect.c"
def main() -> IO(Unit): probe()
"#;

#[test]
fn invalid_frees_and_reference_count_overflow_fail_with_guarded_diagnostics() {
    for (operation, expected) in [
        (
            "Loc at = heap_alloc(e, 1); heap_free(e, 1, at); heap_free(e, 1, at);",
            "invalid native heap span",
        ),
        (
            "Loc at = heap_alloc(e, 1); heap_free(e, 0, at);",
            "native allocation class mismatch",
        ),
        (
            "Loc at = heap_alloc(e, 1); heap_free(e, 0, at + 1);",
            "native allocation class mismatch",
        ),
        (
            "Term value = term_keep(e, io_str(e, \"shared\", 6)); rfc_bump(e, term_loc(value), RFC_CNT);",
            "native reference count overflow",
        ),
    ] {
        let foreign = format!(
            "static Term probe_run(Env e, Term *fields, IoWork *work) {{\n  (void)fields; (void)work;\n  {operation}\n  return term_pak(CID_UNIT, 0);\n}}\nstatic void __attribute__((constructor)) probe_use(void) {{ io_eff(CID_PROBE, probe_run, 0); }}\n"
        );
        let output = Fixture::new().run(PROBE, &foreign, false);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
