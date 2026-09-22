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
    "BEND_MAX_ALLOC=1048576",
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-segments-{}-{}",
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
        join: bool,
        signature: Option<(usize, usize)>,
        counter: &str,
        extra: &[&str],
    ) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        let registrations = generated
            .lines()
            .filter_map(|line| {
                let line = line.trim().strip_prefix("tb_register_segment(")?;
                let fields = line.split(',').collect::<Vec<_>>();
                Some((
                    fields[2].trim().parse::<usize>().unwrap(),
                    fields[3].trim().parse::<usize>().unwrap(),
                ))
            })
            .collect::<Vec<_>>();
        assert!(
            !registrations.is_empty(),
            "checked definitions must register word segments"
        );
        assert!(
            generated.contains("tb_c_word_task(e,"),
            "checked direct calls must emit word tasks"
        );
        if join {
            assert!(
                generated.contains("tb_c_word_join(e,"),
                "simultaneous calls must emit a word join"
            );
        }
        if let Some(signature) = signature {
            assert!(
                registrations.contains(&signature),
                "missing flattened argument/result signature {signature:?}; found {registrations:?}"
            );
        }
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
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {{
    (void)fputs("segment program leaked owners or active work\n", stderr);
    return 91;
  }}
  if (status == 0 && (tb_segment_calls == 0 || !({counter}))) {{
    (void)fputs("checked program did not execute its expected word segments\n", stderr);
    return 92;
  }}
  return status;
}}
"#
        )
        .unwrap();
        let mut definitions = SMALL_STACK.to_vec();
        for definition in extra {
            let name = definition.split('=').next().unwrap();
            definitions.retain(|prior| prior.split('=').next() != Some(name));
            definitions.push(*definition);
        }
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

const PAIR_SEQUENTIAL: &str = r"import Base
type SegmentPair is Data: SegmentPair{left: U32, right: U32}
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def make_pair(n: Nat, seed: U32) -> SegmentPair: SegmentPair{count(n), seed}
def add_pair(value: SegmentPair) -> U32:
  match value:
    case SegmentPair{left, right}: U32.add(left, right)
def main() -> U32:
  value = make_pair(3n, 11)
  add_pair(value)
";

const PAIR_FORK: &str = r"import Base
type SegmentPair is Data: SegmentPair{left: U32, right: U32}
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def make_pair(n: Nat, seed: U32) -> SegmentPair: SegmentPair{count(n), seed}
def add_pair(value: SegmentPair) -> U32:
  match value:
    case SegmentPair{left, right}: U32.add(left, right)
def main() -> SegmentPair & SegmentPair:
  left right = make_pair(3n, 11) make_pair(4n, 22)
  (left, right)
";

const PAIR_NESTED_FORK: &str = r"import Base
type SegmentPair is Data: SegmentPair{left: U32, right: U32}
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def make_pair(n: Nat, seed: U32) -> SegmentPair: SegmentPair{count(n), seed}
def add_pair(value: SegmentPair) -> U32:
  match value:
    case SegmentPair{left, right}: U32.add(left, right)
def grouped(n: Nat, seed: U32) -> SegmentPair & SegmentPair:
  left right = make_pair(n, seed) make_pair(2n, 9)
  (left, right)
def main() -> (SegmentPair & SegmentPair) & (SegmentPair & SegmentPair):
  first second = grouped(3n, 11) grouped(4n, 22)
  (first, second)
";

const UNIT_ROOT: &str = r"import Base
def make_unit(n: Nat) -> Unit:
  match n:
    case 0n: Unit{}
    case 1n+p: make_unit(p)
def consume_unit(empty: Unit, value: U32) -> U32:
  match empty:
    case Unit{}: value
def consume_units(first: Unit, second: Unit) -> U32:
  match first:
    case Unit{}:
      match second:
        case Unit{}: 42
def main() -> Unit: make_unit(2n)
";

const UNIT_SEQUENTIAL: &str = r"import Base
def make_unit(n: Nat) -> Unit:
  match n:
    case 0n: Unit{}
    case 1n+p: make_unit(p)
def consume_unit(empty: Unit, value: U32) -> U32:
  match empty:
    case Unit{}: value
def consume_units(first: Unit, second: Unit) -> U32:
  match first:
    case Unit{}:
      match second:
        case Unit{}: 42
def main() -> U32:
  empty = make_unit(3n)
  consume_unit(empty, 7)
";

const UNIT_FORK: &str = r"import Base
def make_unit(n: Nat) -> Unit:
  match n:
    case 0n: Unit{}
    case 1n+p: make_unit(p)
def consume_unit(empty: Unit, value: U32) -> U32:
  match empty:
    case Unit{}: value
def consume_units(first: Unit, second: Unit) -> U32:
  match first:
    case Unit{}:
      match second:
        case Unit{}: 42
def main() -> U32:
  first second = make_unit(3n) make_unit(4n)
  consume_units(first, second)
";

const UNIT_PAIR_RESULT: &str = r"import Base
def make_unit(n: Nat) -> Unit:
  match n:
    case 0n: Unit{}
    case 1n+p: make_unit(p)
def consume_unit(empty: Unit, value: U32) -> U32:
  match empty:
    case Unit{}: value
def consume_units(first: Unit, second: Unit) -> U32:
  match first:
    case Unit{}:
      match second:
        case Unit{}: 42
def make_empty_pair(n: Nat) -> Unit & Unit:
  first second = make_unit(n) make_unit(2n)
  (first, second)
def main() -> Unit & Unit: make_empty_pair(3n)
";

const MIXED_SEQUENTIAL: &str = r#"import Base
type SegmentMixed is Data: SegmentMixed{text: String, raw: U32, wide: Nat, empty: Unit}
def make_mixed(text: String, raw: U32, wide: Nat, empty: Unit) -> SegmentMixed:
  SegmentMixed{String.append(text, "!"), raw, wide, empty}
def pass_mixed(value: SegmentMixed) -> SegmentMixed: value
def main() -> SegmentMixed:
  pass_mixed(make_mixed("A", 7, Nat.pow(2n, 40n), Unit{}))
"#;

const MIXED_FORK: &str = r#"import Base
type SegmentMixed is Data: SegmentMixed{text: String, raw: U32, wide: Nat, empty: Unit}
def make_mixed(text: String, raw: U32, wide: Nat, empty: Unit) -> SegmentMixed:
  SegmentMixed{String.append(text, "!"), raw, wide, empty}
def pass_mixed(value: SegmentMixed) -> SegmentMixed: value
def main() -> SegmentMixed & SegmentMixed:
  left right = make_mixed("A", 7, Nat.pow(2n, 40n), Unit{}) make_mixed("B", 8, 9n, Unit{})
  (left, right)
"#;

const MIXED_NESTED_FORK: &str = r#"import Base
type SegmentMixed is Data: SegmentMixed{text: String, raw: U32, wide: Nat, empty: Unit}
def make_mixed(text: String, raw: U32, wide: Nat, empty: Unit) -> SegmentMixed:
  SegmentMixed{String.append(text, "!"), raw, wide, empty}
def pass_mixed(value: SegmentMixed) -> SegmentMixed: value
def grouped(text: String, raw: U32) -> SegmentMixed & SegmentMixed:
  left right = make_mixed(text, raw, 9n, Unit{}) make_mixed("inner", 2, 3n, Unit{})
  (left, right)
def main() -> (SegmentMixed & SegmentMixed) & (SegmentMixed & SegmentMixed):
  first second = grouped("A", 7) grouped("B", 8)
  (first, second)
"#;

const PAIR_ARGUMENT_FORK: &str = r"import Base
type SegmentPair is Data: SegmentPair{left: U32, right: U32}
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def make_pair(n: Nat, seed: U32) -> SegmentPair: SegmentPair{count(n), seed}
def add_pair(value: SegmentPair) -> U32:
  match value:
    case SegmentPair{left, right}: U32.add(left, right)
def main() -> U32 & U32:
  left right = add_pair(SegmentPair{1, 2}) add_pair(SegmentPair{3, 4})
  (left, right)
";

const ZERO_ARGUMENT_FORK: &str = r"import Base
def make_unit(n: Nat) -> Unit:
  match n:
    case 0n: Unit{}
    case 1n+p: make_unit(p)
def consume_unit(empty: Unit, value: U32) -> U32:
  match empty:
    case Unit{}: value
def consume_units(first: Unit, second: Unit) -> U32:
  match first:
    case Unit{}:
      match second:
        case Unit{}: 42
def main() -> U32 & U32:
  left right = consume_units(Unit{}, Unit{}) consume_units(Unit{}, Unit{})
  (left, right)
";

const MIXED_ARGUMENT_FORK: &str = r#"import Base
type SegmentMixed is Data: SegmentMixed{text: String, raw: U32, wide: Nat, empty: Unit}
def make_mixed(text: String, raw: U32, wide: Nat, empty: Unit) -> SegmentMixed:
  SegmentMixed{String.append(text, "!"), raw, wide, empty}
def pass_mixed(value: SegmentMixed) -> SegmentMixed: value
def main() -> SegmentMixed & SegmentMixed:
  left right = pass_mixed(SegmentMixed{"A", 7, Nat.pow(2n, 40n), Unit{}}) pass_mixed(SegmentMixed{"B", 8, 9n, Unit{}})
  (left, right)
"#;

#[test]
fn finite_pairs_use_flat_arguments_results_and_nested_join_spans() {
    for (source, expected, join, signature) in [
        (PAIR_SEQUENTIAL, "14\n", false, (2, 2)),
        (
            PAIR_FORK,
            "(SegmentPair{3, 11}, SegmentPair{4, 22})\n",
            true,
            (2, 2),
        ),
        (
            PAIR_NESTED_FORK,
            "((SegmentPair{3, 11}, SegmentPair{2, 9}), SegmentPair{4, 22}, SegmentPair{2, 9})\n",
            true,
            (8, 8),
        ),
        (PAIR_ARGUMENT_FORK, "(3, 7)\n", true, (2, 1)),
        (TRAILING_ERASED_PAIR, "ErasedPair{7, 7}\n", false, (1, 2)),
        (ALL_ERASED_PAIR, "ErasedPair{7, 9}\n", false, (0, 2)),
    ] {
        success(
            &Fixture::new().run(
                source,
                "",
                join,
                Some(signature),
                "tb_segment_multiword_results != 0 && tb_segment_result_words >= 2",
                &[],
            ),
            expected,
        );
    }
}

const TRAILING_ERASED_PAIR: &str = r"import Base
type ErasedPair is Data: ErasedPair{left: U32, right: U32}
def pair(value: U32, -A: Data) -> ErasedPair:
  +shared = value
  ErasedPair{shared, shared}
def main() -> ErasedPair: pair(7, U32)
";

const ALL_ERASED_PAIR: &str = r"import Base
type ErasedPair is Data: ErasedPair{left: U32, right: U32}
def pair(-A: Data) -> ErasedPair: ErasedPair{7, 9}
def main() -> ErasedPair: pair(U32)
";

#[test]
fn zero_word_unit_arguments_keep_boxed_definition_results() {
    for (source, expected, join, signature) in [
        (UNIT_ROOT, "Unit{}\n", false, (0, 1)),
        (UNIT_SEQUENTIAL, "7\n", false, (1, 1)),
        (UNIT_FORK, "42\n", true, (0, 1)),
        (UNIT_PAIR_RESULT, "(Unit{}, Unit{})\n", true, (0, 1)),
        (ZERO_ARGUMENT_FORK, "(42, 42)\n", true, (0, 1)),
    ] {
        success(
            &Fixture::new().run(source, "", join, Some(signature), "1", &[]),
            expected,
        );
    }
}

#[test]
fn owned_strings_and_raw_words_keep_their_layout_across_calls_and_nested_forks() {
    for (source, expected, join, signature) in [
        (
            MIXED_SEQUENTIAL,
            "SegmentMixed{\"A!\", 7, 1099511627776n, Unit{}}\n",
            false,
            (3, 3),
        ),
        (
            MIXED_FORK,
            "(SegmentMixed{\"A!\", 7, 1099511627776n, Unit{}}, SegmentMixed{\"B!\", 8, 9n, Unit{}})\n",
            true,
            (3, 3),
        ),
        (
            MIXED_NESTED_FORK,
            "((SegmentMixed{\"A!\", 7, 9n, Unit{}}, SegmentMixed{\"inner!\", 2, 3n, Unit{}}), SegmentMixed{\"B!\", 8, 9n, Unit{}}, SegmentMixed{\"inner!\", 2, 3n, Unit{}})\n",
            true,
            (12, 12),
        ),
        (
            MIXED_ARGUMENT_FORK,
            "(SegmentMixed{\"A\", 7, 1099511627776n, Unit{}}, SegmentMixed{\"B\", 8, 9n, Unit{}})\n",
            true,
            (3, 3),
        ),
    ] {
        success(
            &Fixture::new().run(
                source,
                "",
                join,
                Some(signature),
                "tb_segment_multiword_results != 0 && tb_segment_result_words >= 2",
                &[],
            ),
            expected,
        );
    }
}

const RECURSIVE_RESULTS: &str = r#"import Base
type RecursivePair is Data: RecursivePair{word: U32, text: String}
def identity(value: RecursivePair) -> RecursivePair: value
def merge(left: RecursivePair, right: RecursivePair) -> RecursivePair:
  match left:
    case RecursivePair{first, text}:
      match right:
        case RecursivePair{second, ignored}: RecursivePair{U32.add(first, second), text}
def nested(n: Nat, text: String) -> RecursivePair:
  match n:
    case 0n: RecursivePair{0, text}
    case 1n+p:
      left right = nested(p, text) identity(RecursivePair{1, "temporary"})
      merge(left, right)
def main() -> RecursivePair: nested(Nat.mul(64n, 4n), String.append("ro", "ot"))
"#;

const FINITE_SUMS: &str = r#"import Base
type SegmentChoice is Data:
  RawValue{number: Nat}
  OwnedValue{text: String}
def flip(value: SegmentChoice) -> SegmentChoice:
  match value:
    case RawValue{number}: OwnedValue{String.append("value:", U32.show(U32.from_nat(number)))}
    case OwnedValue{text}: RawValue{String.length(text)}
def cycle(n: Nat, value: SegmentChoice) -> SegmentChoice:
  match n:
    case 0n: value
    case 1n+p: cycle(p, flip(value))
def main() -> SegmentChoice & SegmentChoice:
  left right = cycle(64n, RawValue{7n}) cycle(64n, OwnedValue{"a"})
  (left, right)
"#;

#[test]
fn recursive_multiword_results_and_variant_ownership_fit_a_small_native_stack() {
    success(
        &Fixture::new().run(
            RECURSIVE_RESULTS,
            "",
            true,
            Some((2, 2)),
            "tb_task_joins == 256 && tb_segment_multiword_results != 0 && tb_segment_result_words >= 2",
            &[],
        ),
        "RecursivePair{256, \"root\"}\n",
    );
    success(
        &Fixture::new().run(
            FINITE_SUMS,
            "",
            true,
            Some((2, 2)),
            "tb_segment_multiword_results != 0 && tb_segment_result_words >= 2",
            &[],
        ),
        "(RawValue{7n}, OwnedValue{\"value:7\"})\n",
    );
}

#[test]
fn word_segment_graphs_fail_closed_at_task_and_host_budgets() {
    let shallow = RECURSIVE_RESULTS.replace("Nat.mul(64n, 4n)", "4n");
    for (definitions, diagnostic) in [
        (&["BEND_MAX_TASKS=32"][..], "task budget exhausted"),
        (
            &["BEND_MAX_HOST_ALLOC=65536"][..],
            "host allocation budget exhausted",
        ),
    ] {
        success(
            &Fixture::new().run(
                &shallow,
                "",
                true,
                Some((2, 2)),
                "tb_task_joins == 4",
                definitions,
            ),
            "RecursivePair{4, \"root\"}\n",
        );
        failure(
            &Fixture::new().run(RECURSIVE_RESULTS, "", true, Some((2, 2)), "1", definitions),
            diagnostic,
        );
    }
}

const LEGACY_INTEROP: &str = r#"import Base
type SegmentPair is Data: SegmentPair{left: U32, right: U32}
def drive(function: (U32 -> U32) -> SegmentPair -> SegmentPair, value: SegmentPair) -> IO(SegmentPair): import "effect.c"
def bridge(callback: U32 -> U32, value: SegmentPair) -> SegmentPair:
  match value:
    case SegmentPair{left, right}: SegmentPair{callback(left), U32.add(right, 1)}
def outer(callback: U32 -> U32, value: SegmentPair) -> SegmentPair: bridge(callback, value)
def total(value: SegmentPair) -> U32:
  match value:
    case SegmentPair{left, right}: U32.add(left, right)
def main() -> IO(Unit):
  do IO<Unit>:
    value : SegmentPair <- drive(outer, SegmentPair{7, 11})
    IO.print(U32.show(total(value)))
"#;

const LEGACY_INTEROP_C: &str = r#"
static u32 callback_calls;
static Term callback_apply(Env e, const Term *captures, Term argument) {
  (void)e; (void)captures;
  if (++callback_calls != 1) err_fail("segment callback repeated");
  io_out(stdout, "callback\n", 9);
  return argument + 5;
}
static Term drive_run(Env e, Term *fields, IoWork *work) {
  Term function, result;
  (void)work;
  function = tb_apply(e, fields[0], tb_closure(e, 62000, 0, NULL));
  result = tb_apply(e, function, fields[1]);
  if (callback_calls != 1) err_fail("segment callback omitted");
  return result;
}
static void __attribute__((constructor)) drive_use(void) {
  tb_register_closure(62000, callback_apply, 0);
  io_eff(CID_DRIVE, drive_run, 0);
}
"#;

#[test]
fn foreign_and_dynamic_closure_calls_cross_the_flat_segment_boundary_once() {
    success(
        &Fixture::new().run(
            LEGACY_INTEROP,
            LEGACY_INTEROP_C,
            false,
            Some((3, 2)),
            "tb_segment_multiword_results != 0 && tb_segment_result_words >= 2",
            &[],
        ),
        "callback\n24\n",
    );
}

const INDIRECT_U32_TAIL: &str = r"import Base
def invoke(function: Unit -> U32) -> U32: function(Unit{})
def countdown(n: Nat, value: U32) -> U32:
  match n:
    case 0n: U32.add(value, 1)
    case 1n+p: invoke(ignored => countdown(p, value))
def main() -> U32: countdown(Nat.mul(100n, 20n), 7)
";

const INDIRECT_U32_PENDING: &str = r"import Base
def invoke(function: Unit -> U32) -> U32: function(Unit{})
def countdown(n: Nat, value: U32) -> U32:
  match n:
    case 0n: U32.add(value, 1)
    case 1n+p: invoke(ignored => countdown(p, value))
def main() -> U32: U32.add(countdown(Nat.mul(100n, 20n), 7), 9)
";

const INDIRECT_NAT_TAIL: &str = r"import Base
def invoke(function: Unit -> Nat) -> Nat: function(Unit{})
def countdown(n: Nat, value: Nat) -> Nat:
  match n:
    case 0n: Nat.add(value, 1n)
    case 1n+p: invoke(ignored => countdown(p, value))
def main() -> Nat: countdown(Nat.mul(100n, 20n), Nat.mul(Nat.pow(2n, 20n), Nat.pow(2n, 20n)))
";

const INDIRECT_NAT_PENDING: &str = r"import Base
def invoke(function: Unit -> Nat) -> Nat: function(Unit{})
def countdown(n: Nat, value: Nat) -> Nat:
  match n:
    case 0n: Nat.add(value, 1n)
    case 1n+p: invoke(ignored => countdown(p, value))
def main() -> Nat: Nat.add(countdown(Nat.mul(100n, 20n), Nat.mul(Nat.pow(2n, 20n), Nat.pow(2n, 20n))), 9n)
";

#[test]
fn scalar_and_dynamic_tail_calls_stay_bounded_and_do_not_adapt_the_pending_parent() {
    for (name, source, expected) in [
        ("INDIRECT_U32_TAIL", INDIRECT_U32_TAIL, "8\n"),
        ("INDIRECT_U32_PENDING", INDIRECT_U32_PENDING, "17\n"),
        ("INDIRECT_NAT_TAIL", INDIRECT_NAT_TAIL, "1099511627777n\n"),
        (
            "INDIRECT_NAT_PENDING",
            INDIRECT_NAT_PENDING,
            "1099511627786n\n",
        ),
    ] {
        let output = Fixture::new().run(
            source,
            "",
            false,
            Some((1, 1)),
            "1",
            &["BEND_MAX_CONTINUATIONS=32", "BEND_MAX_ALLOC=4096"],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "{name}");
        assert!(output.stderr.is_empty(), "{name}: unexpected stderr");
    }
}
