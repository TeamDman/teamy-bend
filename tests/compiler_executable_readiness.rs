// SPDX-License-Identifier: MPL-2.0
//! Deterministic scheduler/foreign-ABI tests with an injected synchronous host.
//! These are not OS networking or Node-API provider validation.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const EFFECT_SOURCE: &str = "import Base\ndef effect() -> IO(Unit): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    Unit <- effect()\n    IO.print(\"AFTER\")\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-readiness-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, source: &str, foreign: &str, tail: &str) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        fs::write(self.0.join("effect.js"), foreign).unwrap();
        let source = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&source).unwrap();
        let javascript = compile_executable_javascript(&checked).unwrap();
        fs::write(self.0.join("main.cjs"), format!("{javascript}\n{tail}\n")).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("readiness tests require Node.js")
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

fn failure(output: &Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn legacy_read_hooks_defer_the_first_call_and_wake_on_data_error_hangup_or_invalid() {
    let fixture = Fixture::new();
    let source = "import Base\ndef read(fd: U32) -> IO(U32): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    n : U32 <- read(7)\n    IO.print(U32.show(n))\n";
    for mask in [1, 8, 16, 32] {
        let foreign = format!(
            r"
let state = 0;
const provider = Object.freeze({{
  ptr(buffer) {{ if (!(buffer instanceof Int32Array)) throw Error('poll buffer'); return buffer; }},
  poll(buffer, count, ms) {{
    if (this !== provider || state !== 1 || count !== 1 || buffer.length !== 2
        || buffer[0] !== 7 || buffer[1] !== 1 || ms !== 1000) throw Error('poll ABI');
    buffer[1] |= {mask} << 16; state = 2; return 1;
  }}
}});
globalThis.BEND_SYS = provider;
function read_need() {{
  if (arguments.length !== 0 || this.args[0] !== 7 || state !== 0) throw Error('need receiver');
  state = 1; return {{ read: true, time: true, write: true }};
}}
function read(fd, k) {{
  if (arguments.length !== 2 || fd !== 7 || this.kont !== k || state !== 2) throw Error('read ran before poll');
  if (globalThis.BEND_SYS !== provider || io_sys() !== io_sys()) throw Error('provider replaced or view unstable');
  return 42;
}}
"
        );
        success(&fixture.run(source, &foreign, ""), "42\n");
    }
}

#[test]
fn write_only_hook_runs_immediately_and_explicit_write_parks_preserve_retry_closures() {
    let output = Fixture::new().run(
        "import Base\ndef write(fd: U32) -> IO(String): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    text : String <- write(9)\n    IO.print(text)\n",
        r"
let polls = 0;
const trace = [];
const provider = {
  poll_descriptors(rows, ms) {
    if (this !== provider || rows.length !== 1 || rows[0].fd !== 9
        || rows[0].events !== 4 || ms !== 1000) throw Error('write poll ABI');
    trace.push('poll'); polls++; return new Uint16Array([4]);
  }
};
globalThis.BEND_SYS = provider;
function write_need() { return { write: true }; }
function write(fd, k) {
  if (polls !== 0 || this.kont !== k) throw Error('write-only hook must not park');
  trace.push('run');
  const offset = 2;
  const more = () => {
    if (offset !== 2) throw Error('captured offset');
    if (polls < 2) { trace.push('retry'); io_park_on(fd, true, k, more); return undefined; }
    trace.push('done'); return trace.join(',');
  };
  io_park_on(fd, true, k, more);
}
",
        "",
    );
    success(&output, "run,poll,retry,poll,done\n");
}

#[test]
fn mixed_timers_duplicate_descriptors_and_reparks_preserve_registration_order() {
    let output = Fixture::new().run(
        r#"import Base
def read(fd: U32, label: String) -> IO(Unit): import "effect.js"
def write(fd: U32) -> IO(Unit): import "effect.js"
def reader(+label: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- read(7, label)
    IO.print(label)
def timer(ms: U32, label: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(ms)
    IO.print(label)
def writer() -> IO(Unit):
  do IO<Unit>:
    Unit <- write(8)
    IO.print("write")
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, timer(5, "timer0"))
    Unit <- IO.spawn(Unit, reader("read0"))
    Unit <- IO.spawn(Unit, timer(1, "timer1"))
    Unit <- IO.spawn(Unit, reader("read1"))
    Unit <- IO.spawn(Unit, writer())
    IO.print("main")
"#,
        r#"
let now = 0.25;
let polls = 0;
const needs = new Set();
Object.defineProperty(globalThis, 'performance', { value: { now: () => now }, configurable: true });
globalThis.BEND_SYS = {
  poll_descriptors(rows, ms) {
    polls++;
    if (polls === 1) {
      if (JSON.stringify(rows) !== '[{"fd":7,"events":1},{"fd":7,"events":1},{"fd":8,"events":4}]'
          || ms !== 1) throw Error('mixed registration ABI');
      now = 10; return [8, 1, 16];
    }
    if (polls !== 2 || rows.length !== 1 || rows[0].fd !== 7 || rows[0].events !== 1 || ms !== 1000) {
      throw Error('repark must follow the completed batch');
    }
    return [1];
  }
};
function read_need() {
  const tag = this.args[1];
  if (needs.has(tag)) throw Error('need called again during retry');
  needs.add(tag); return { read: true };
}
function read(fd, tag, k) {
  if (polls !== 1) throw Error('read before initial poll');
  if (tag === 'read0') { io_park_on(fd, false, k, () => ({ $: 'Unit' })); return undefined; }
  return { $: 'Unit' };
}
function write(fd, k) { now = 0.75; io_park_on(fd, true, k, () => ({ $: 'Unit' })); }
"#,
        "",
    );
    success(&output, "main\ntimer0\ntimer1\nread1\nwrite\nread0\n");
}

#[test]
fn extension_preserves_full_width_handles_and_legacy_provider_refuses_narrowing() {
    let fixture = Fixture::new();
    for descriptor in ["4294967297", "0xfedcba9876543210n"] {
        let foreign = format!(
            r"
const descriptor = {descriptor};
globalThis.BEND_SYS = {{ poll_descriptors(rows) {{
  if (rows.length !== 1 || rows[0].fd !== descriptor || rows[0].events !== 1) throw Error('descriptor narrowed');
  return [1];
}} }};
function effect(k) {{ io_park_on(descriptor, false, k, () => ({{ $: 'Unit' }})); }}
"
        );
        success(&fixture.run(EFFECT_SOURCE, &foreign, ""), "AFTER\n");
        let foreign = format!(
            r"
globalThis.BEND_SYS = {{ ptr: b => b, poll() {{ process.stdout.write('BAD'); return 0; }} }};
function effect(k) {{ io_park_on({descriptor}, false, k, () => ({{ $: 'Unit' }})); }}
"
        );
        failure(
            &fixture.run(EFFECT_SOURCE, &foreign, ""),
            "full-width handle",
        );
    }
}

#[test]
fn supplied_pollers_keep_owning_timer_waits_after_the_last_descriptor_completes() {
    let fixture = Fixture::new();
    let source = r#"import Base
def read(fd: U32) -> IO(Unit): import "effect.js"
def reader() -> IO(Unit):
  do IO<Unit>:
    Unit <- read(3)
    IO.print("descriptor")
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, reader())
    Unit <- IO.sleep(5)
    IO.print("timer")
"#;
    for implementation in [
        r"poll_descriptors(rows, ms) {
  if (polls === 0) {
    if (rows.length !== 1 || rows[0].fd !== 3 || ms !== 5) throw Error('initial descriptor batch');
    now = 2; polls++; return [1];
  }
  if (polls !== 1 || rows.length !== 0 || ms !== 3) throw Error('timer-only extension batch');
  now = 5; polls++; return [];
}",
        r"ptr: b => b, poll(buffer, count, ms) {
  if (polls === 0) {
    if (count !== 1 || buffer[0] !== 3 || ms !== 5) throw Error('initial descriptor batch');
    buffer[1] |= 1 << 16; now = 2; polls++; return 1;
  }
  if (polls !== 1 || buffer !== null || count !== 0 || ms !== 3) throw Error('timer-only legacy batch');
  now = 5; polls++; return 0;
}",
    ] {
        let foreign = format!(
            r"
let now = 0; let polls = 0;
Object.defineProperty(globalThis, 'performance', {{ value: {{ now: () => now }}, configurable: true }});
globalThis.BEND_SYS = {{ {implementation} }};
function read_need() {{ return {{ read: true }}; }}
function read() {{ if (polls !== 1) throw Error('read before poll'); return {{ $: 'Unit' }}; }}
"
        );
        success(&fixture.run(source, &foreign, ""), "descriptor\ntimer\n");
    }
}

const BAD_EXTENSION_RESULTS: &[(&str, &str)] = &[
    ("undefined", "invalid result count"),
    ("{}", "invalid result count"),
    ("[]", "invalid result count"),
    ("[1, 1]", "invalid result count"),
    ("[NaN]", "invalid event mask"),
    ("[-1]", "invalid event mask"),
    ("[65536]", "invalid event mask"),
    ("[1.5]", "invalid event mask"),
    ("new BigUint64Array([1n])", "invalid event mask"),
    ("Promise.resolve([1])", "asynchronous foreign results"),
];

#[test]
fn malformed_readiness_results_fail_before_callbacks_and_following_effects() {
    let fixture = Fixture::new();
    for (result, diagnostic) in BAD_EXTENSION_RESULTS {
        let foreign = format!(
            r"
globalThis.BEND_SYS = {{ poll_descriptors() {{ return {result}; }} }};
function effect(k) {{ io_park_on(3, false, k, () => {{ process.stdout.write('BAD'); return {{ $: 'Unit' }}; }}); }}
"
        );
        failure(&fixture.run(EFFECT_SOURCE, &foreign, ""), diagnostic);
    }
    for (count, mask, diagnostic) in [
        ("-1", 0, "invalid poll count"),
        ("undefined", 0, "invalid poll count"),
        ("1.5", 0, "invalid poll count"),
        ("2", 1, "invalid poll count"),
        ("0", 1, "inconsistent poll events"),
        ("1", 0, "inconsistent poll events"),
    ] {
        let foreign = format!(
            r"
globalThis.BEND_SYS = {{ ptr: b => b, poll(b) {{ b[1] |= {mask} << 16; return {count}; }} }};
function effect(k) {{ io_park_on(3, false, k, () => {{ process.stdout.write('BAD'); return {{ $: 'Unit' }}; }}); }}
"
        );
        failure(&fixture.run(EFFECT_SOURCE, &foreign, ""), diagnostic);
    }
}

#[test]
fn missing_or_invalid_authoritative_provider_is_never_replaced() {
    let fixture = Fixture::new();
    for provider in [
        "{}",
        "{poll_descriptors: 1, ptr: b => b, poll() {throw Error('BAD FALLBACK');}}",
    ] {
        let foreign = format!(
            r"
const provider = {provider}; globalThis.BEND_SYS = provider;
function effect(k) {{ io_park_on(3, false, k, () => ({{ $: 'Unit' }})); }}
"
        );
        let output = fixture.run(EFFECT_SOURCE, &foreign, "");
        failure(&output, "synchronous poll provider");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("BAD FALLBACK"));
    }
    for descriptor in [
        "undefined",
        "NaN",
        "9007199254740992",
        "-1n",
        "0x10000000000000000n",
    ] {
        let foreign = format!(
            r"
globalThis.BEND_SYS = {{ poll_descriptors() {{ throw Error('BAD POLL'); }} }};
function effect(k) {{ io_park_on({descriptor}, false, k, () => ({{ $: 'Unit' }})); }}
"
        );
        failure(
            &fixture.run(EFFECT_SOURCE, &foreign, ""),
            "lossless descriptor",
        );
    }
}

const CLEANUP_TAIL: &str = r"
const saved = globalThis.savedScheduler;
if ($tbIo !== null || saved.live !== 0 || saved.runs.length !== 0 || saved.waits.length !== 0) {
  throw Error('scheduler cleanup failed');
}
process.stdout.write('clean\n');
";

#[test]
fn readiness_budget_errors_clear_registrations_and_captured_callbacks() {
    let fixture = Fixture::new();
    for (body, diagnostic) in [
        (
            "for(let i=0;i<=131072;i++)io_park_on(3,false,k,more);",
            "pending task budget exhausted",
        ),
        ("io_park_on(3,false,k,more);", "step budget exhausted"),
    ] {
        let foreign = format!(
            r"
globalThis.BEND_SYS = {{ poll_descriptors(rows) {{ return rows.map(() => 1); }} }};
function effect(k) {{
  globalThis.savedScheduler = $tbIo;
  const more = () => {{ io_park_on(3, false, k, more); return undefined; }};
  {body}
}}
"
        );
        let output = fixture.run(EFFECT_SOURCE, &foreign, CLEANUP_TAIL);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"clean\n");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn halt_drops_descriptor_waits_without_polling_or_invoking_their_callbacks() {
    let output = Fixture::new().run(
        "import Base\ndef parked() -> IO(Unit): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    Unit <- IO.spawn(Unit, IO.die(Unit, 7, \"stop\"))\n    parked()\n",
        r"
globalThis.BEND_SYS = { poll_descriptors() { throw Error('poll after Halt'); } };
function parked(k) {
  globalThis.savedScheduler = $tbIo;
  io_park_on(3, false, k, () => { throw Error('callback after Halt'); });
}
",
        CLEANUP_TAIL,
    );
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"clean\n");
    assert_eq!(output.stderr, b"stop\n");
}
