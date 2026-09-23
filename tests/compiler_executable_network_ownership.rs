// SPDX-License-Identifier: MPL-2.0
//! Checked-source ownership transitions with a deterministic raw-descriptor host.
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

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-network-ownership-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, source: &str, foreign: &str) -> Output {
        fs::write(self.0.join("main.bend"), format!("import Base\n{source}")).unwrap();
        fs::write(self.0.join("effect.js"), format!("{PROVIDER}\n{foreign}")).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let javascript = compile_executable_javascript(&checked).unwrap();
        fs::write(self.0.join("main.cjs"), javascript).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .env("TEAMY_BEND_TEST_LOOPBACK_NETWORK", "1")
            .output()
            .expect("network ownership tests require Node.js")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

const PROVIDER: &str = r"
const assert = require('node:assert/strict');
let opens = 0;
let open = false;
const closed = [];
let closeAlias;
const provider = Object.freeze({
  get mac() { assert.equal(this, provider); return false; },
  ptr(bytes) { assert.equal(this, provider); return bytes; },
  socket(domain, kind, protocol) {
    assert.equal(this, provider); assert.equal(open, false);
    assert.deepEqual([domain, kind, protocol], [2, 2, 0]);
    opens++; open = true; return 42;
  },
  bind(fd) { assert.equal(this, provider); assert.equal(fd, 42); return 0; },
  fcntl(fd) { assert.equal(this, provider); assert.equal(fd, 42); return 0; },
  close(fd) {
    assert.equal(this, provider); assert.equal(fd, 42); assert.equal(open, true);
    closed.push(fd); open = false; return 0;
  }
});
function setup() {
  globalThis.BEND_SYS = provider;
  const view = io_sys();
  assert.notEqual(view, provider);
  assert.equal(view, io_sys());
  assert.equal(globalThis.BEND_SYS, provider);
  assert.equal(Object.isFrozen(provider), true);
  assert.deepEqual(Object.keys(view), Object.keys(provider));
  closeAlias = view.close;
  assert.equal(closeAlias, io_sys().close);
  return { $: 'Unit' };
}
";

#[test]
fn raw_close_alias_allows_builtin_descriptor_reuse_without_exit_double_close() {
    let output = Fixture::new().run(
        r#"def setup() -> IO(Unit): import "effect.js"
def raw_close(socket: Socket) -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    Unit <- raw_close(socket)
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    Unit <- Socket.close(socket)
    IO.print("reused")
"#,
        r"
function raw_close(fd) { closeAlias(fd); return { $: 'Unit' }; }
process.on('exit', () => { assert.equal(opens, 2); assert.equal(open, false); assert.deepEqual(closed, [42, 42]); });
",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"reused\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn raw_close_then_external_reuse_does_not_transfer_replacement_ownership_to_the_vm() {
    let output = Fixture::new().run(
        r#"def setup() -> IO(Unit): import "effect.js"
def external_reuse(socket: Socket) -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    Unit <- external_reuse(socket)
    IO.die(Unit, 7, "halt")
"#,
        r"
function external_reuse(fd) {
  assert.equal(io_sys().close(fd), 0);
  assert.equal(io_sys().socket(2, 2, 0), fd);
  return { $: 'Unit' };
}
process.on('exit', () => { assert.equal(opens, 2); assert.equal(open, true); assert.deepEqual(closed, [42]); });
",
    );
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"halt\n");
}

#[test]
fn captured_close_alias_forgets_only_its_original_provider_after_global_replacement() {
    let output = Fixture::new().run(
        r#"def setup() -> IO(Unit): import "effect.js"
def replace_and_close(socket: Socket) -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    Unit <- replace_and_close(socket)
    IO.print("closed original")
"#,
        r"
function replace_and_close(fd) {
  const old = io_sys();
  const replacement = Object.freeze({ close() { throw Error('closed wrong provider'); } });
  globalThis.BEND_SYS = replacement;
  assert.notEqual(io_sys(), old);
  closeAlias(fd);
  assert.equal(globalThis.BEND_SYS, replacement);
  return { $: 'Unit' };
}
process.on('exit', () => { assert.equal(open, false); assert.deepEqual(closed, [42]); });
",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"closed original\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn explicit_raw_close_failure_is_not_retried_by_exit_cleanup() {
    let output = Fixture::new().run(
        r#"def setup_failure() -> IO(Unit): import "effect.js"
def raw_close(socket: Socket) -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup_failure()
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    raw_close(socket)
"#,
        r"
let attempts = 0;
function setup_failure() {
  const failing = { ...provider, close(fd) { assert.equal(this, failing); assert.equal(fd, 42); attempts++; throw Error('explicit close failed'); } };
  globalThis.BEND_SYS = failing;
  // Retain the frozen provider's receiver requirements for the other methods.
  for (const key of ['socket','ptr','bind','fcntl']) failing[key] = provider[key].bind(provider);
  return { $: 'Unit' };
}
function raw_close(fd) { io_sys().close(fd); return { $: 'Unit' }; }
process.on('exit', () => { assert.equal(attempts, 1); });
",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("explicit close failed"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("AssertionError"));
}

#[test]
fn replacing_a_provider_method_through_the_view_updates_the_next_real_poll() {
    let output = Fixture::new().run(
        r#"def setup_dynamic() -> IO(Unit): import "effect.js"
def replace_method() -> IO(Unit): import "effect.js"
def wait_descriptor() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup_dynamic()
    Unit <- replace_method()
    Unit <- wait_descriptor()
    IO.print("updated")
"#,
        r"
let polls = 0;
const dynamic = { poll_descriptors() { throw Error('stale poll method'); } };
function setup_dynamic() { globalThis.BEND_SYS = dynamic; return { $: 'Unit' }; }
function replace_method() {
  const view = io_sys(), old = view.poll_descriptors;
  const replacement = function(rows, ms) {
    assert.equal(this, dynamic);
    assert.deepEqual(rows, [{ fd: 23, events: 1 }]); assert.equal(ms, 1000);
    polls++; return [1];
  };
  view.poll_descriptors = replacement;
  assert.equal(dynamic.poll_descriptors, replacement);
  assert.notEqual(view.poll_descriptors, old);
  assert.equal(view.poll_descriptors, io_sys().poll_descriptors);
  view.scratch = 3; assert.equal(dynamic.scratch, 3);
  delete view.scratch; assert.equal('scratch' in dynamic, false);
  assert.equal(globalThis.BEND_SYS, dynamic);
  return { $: 'Unit' };
}
function wait_descriptor(k) { io_park_on(23, false, k, () => ({ $: 'Unit' })); }
process.on('exit', () => { assert.equal(polls, 1); });
",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"updated\n");
    assert!(output.stderr.is_empty());
}
