// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::Shutdown;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const TIMEOUT: Duration = Duration::from_secs(20);
const HELPERS: &str = r#"import Base
def keep_sent(pair: Socket & Result<&1, &1, U32 & String, Unit>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    Unit <- IO.pass(Unit, result)
    return socket

def keep_text(pair: Socket & Result<&1, &1, U32 & String, String>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    text : String <- IO.pass(String, result)
    Unit <- IO.print(String.append("text:", text))
    return socket

def packet_text(packet: String & U32 & String) -> IO(Unit):
  (host, port, text) = packet
  do IO<Unit>:
    Unit <- IO.print(host)
    Unit <- IO.print(U32.show(port))
    IO.print(String.append("packet:", text))

def keep_packet(pair: Socket & Result<&1, &1, U32 & String, String & U32 & String>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    packet : String & U32 & String <- IO.pass(String & U32 & String, result)
    Unit <- packet_text(packet)
    return socket

def poll_text(value: Maybe<&1, String & U32 & String>) -> IO(Unit):
  match value:
    case None{}: IO.print("none")
    case Some{packet}: packet_text(packet)

def keep_poll(pair: Socket & Result<&1, &1, U32 & String, Maybe<&1, String & U32 & String>>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    value : Maybe<&1, String & U32 & String> <- IO.pass(Maybe<&1, String & U32 & String>, result)
    Unit <- poll_text(value)
    return socket
"#;

// This provider is deliberately a syscall script. Real loopback tests below use
// the native addon; scripting here makes partial writes and errno transitions
// deterministic without relying on platform socket buffer sizes.
const SCRIPT_PROVIDER: &str = r"
const assert = require('node:assert/strict');
const unit = () => ({ $: 'Unit' });
let error = 0;
const closed = [];
const polls = [];
const provider = {
  mac: false, ptr: value => value, errno: () => error,
  strerror: code => 'custom-' + code,
  fcntl: () => 0, setsockopt: () => 0, bind: () => 0, listen: () => 0,
  close: fd => { closed.push(fd); return -1; },
  poll_descriptors: (rows, milliseconds) => {
    assert.ok(Number.isInteger(milliseconds));
    for (const row of rows) polls.push([row.fd, row.events]);
    return rows.map(row => row.events);
  },
  poll: (words, count, milliseconds) => {
    assert.ok(Number.isInteger(milliseconds));
    for (let i = 0; i < count; i++) {
      const events = words[i * 2 + 1] & 0xffff;
      polls.push([words[i * 2], events]);
      words[i * 2 + 1] = events | events << 16;
    }
    return count;
  }
};
function setup() { globalThis.BEND_SYS = provider; return unit(); }
";

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str, foreign: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-network-js-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        fs::write(path.join("effect.js"), foreign).unwrap();
        let loaded = load_executable(path.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let javascript = compile_executable_javascript(&checked).unwrap();
        fs::write(path.join("main.cjs"), javascript).unwrap();
        Self(path)
    }

    fn start(&self, native_provider: bool) -> Running {
        let module = if native_provider {
            let filename = if cfg!(windows) {
                "teamy_bend_sys.dll"
            } else if cfg!(target_os = "macos") {
                "libteamy_bend_sys.dylib"
            } else {
                "libteamy_bend_sys.so"
            };
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/debug")
                .join(filename);
            assert!(
                path.is_file(),
                "native JS test provider missing: {}",
                path.display()
            );
            path
        } else {
            // An already installed BEND_SYS must take precedence, even when the
            // configured module does not exist.
            self.0.join("must-not-load.cjs")
        };
        Running::start(
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
                .arg(self.0.join("main.cjs"))
                .current_dir(&self.0)
                .env("TEAMY_BEND_SYS_MODULE", module),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

struct Running {
    child: Child,
    lines: mpsc::Receiver<Vec<u8>>,
    stdout: Option<thread::JoinHandle<Vec<u8>>>,
    stderr: Option<thread::JoinHandle<Vec<u8>>>,
}

impl Running {
    fn start(command: &mut Command) -> Self {
        let mut child = command
            .env("TEAMY_BEND_TEST_LOOPBACK_NETWORK", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (sender, lines) = mpsc::channel();
        let stdout = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut all = Vec::new();
            loop {
                let mut line = Vec::new();
                if reader.read_until(b'\n', &mut line).unwrap() == 0 {
                    return all;
                }
                all.extend_from_slice(&line);
                let _receiver_gone = sender.send(line);
            }
        });
        let stderr = thread::spawn(move || {
            let mut bytes = Vec::new();
            BufReader::new(stderr).read_to_end(&mut bytes).unwrap();
            bytes
        });
        Self {
            child,
            lines,
            stdout: Some(stdout),
            stderr: Some(stderr),
        }
    }

    fn line(&self) -> String {
        String::from_utf8(self.lines.recv_timeout(TIMEOUT).unwrap()).unwrap()
    }

    fn finish(mut self) -> Output {
        let deadline = Instant::now() + TIMEOUT;
        let (status, timed_out) = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break (status, false);
            }
            if Instant::now() >= deadline {
                self.child.kill().unwrap();
                break (self.child.wait().unwrap(), true);
            }
            thread::sleep(Duration::from_millis(10));
        };
        let stdout = self.stdout.take().unwrap().join().unwrap();
        let stderr = self.stderr.take().unwrap().join().unwrap();
        assert!(
            !timed_out,
            "generated network program timed out; stdout: {}; stderr: {}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        );
        Output {
            status,
            stdout,
            stderr,
        }
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _killed = self.child.kill();
        let _waited = self.child.wait();
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

const SCRIPT_SEND: &str = r#"def setup() -> IO(Unit): import "effect.js"
def raw() -> IO(Socket): import "effect.js"
def observe(socket: Socket) -> IO(Socket): import "effect.js"
def verify() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- raw()
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- TCP.send(socket, "é\0abc")
    socket : Socket <- keep_sent(sent)
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- TCP.send(socket, "")
    socket : Socket <- keep_sent(sent)
    socket : Socket <- observe(socket)
    Unit <- Socket.close(socket)
    verify()
"#;
const SCRIPT_SEND_JS: &str = r"
let call = 0;
const bytes = [];
function raw() { return 43; }
provider.send = (fd, part, length, flags) => {
  assert.equal(fd, 43); assert.equal(length, part.length); assert.equal(flags, 0);
  const count = [2, -1, 1, -1, 3][call++];
  assert.notEqual(count, undefined);
  if (count < 0) { error = 11; return -1; }
  bytes.push(...part.subarray(0, count)); return count;
};
function observe(fd) {
  assert.equal(fd, 43); assert.equal(globalThis.BEND_SYS, provider);
  assert.deepEqual(bytes, [195, 169, 0, 97, 98, 99]);
  assert.equal(call, 5); assert.deepEqual(polls, [[43, 4], [43, 4]]);
  return fd;
}
function verify() { assert.deepEqual(closed, [43]); console.log('partial-send-ok'); return unit(); }
";

#[test]
fn custom_provider_and_foreign_socket_preserve_partial_send_offsets_across_parks() {
    let fixture = Fixture::new(SCRIPT_SEND, &format!("{SCRIPT_PROVIDER}{SCRIPT_SEND_JS}"));
    success(&fixture.start(false).finish(), "partial-send-ok\n");
}

const SCRIPT_FAILURES: &str = r#"def setup() -> IO(Unit): import "effect.js"
def verify() -> IO(Unit): import "effect.js"
def opened(result: Result<&1, &1, U32 & String, Socket>) -> IO(Unit):
  match result:
    case Done{socket}: IO.die(Unit, 1, "unexpected connection")
    case Fail{(code, message)}: IO.print(message)

def accepted(pair: Listener & Result<&1, &1, U32 & String, Socket>) -> IO(Unit):
  (listener, result) = pair
  do IO<Unit>:
    Unit <- opened(result)
    Listener.close(listener)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    result : Result<&1, &1, U32 & String, Socket> <- TCP.connect("127.0.0.1", 99)
    Unit <- opened(result)
    listener : Listener <- IO.try(Listener, TCP.listen(0))
    Unit <- IO.spawn(Unit, IO.print("ready"))
    result : Listener & Result<&1, &1, U32 & String, Socket> <- TCP.accept(listener)
    Unit <- accepted(result)
    verify()
"#;
const SCRIPT_FAILURES_JS: &str = r"
let socketCount = 0;
provider.socket = () => [81, 82][socketCount++];
provider.connect = fd => { assert.equal(fd, 81); error = 115; return -1; };
provider.getsockopt = (fd, level, option, value, length) => {
  assert.equal(fd, 81); assert.equal(level, 1); assert.equal(option, 4);
  assert.equal(length[0], 4); value[0] = 111; return 0;
};
provider.accept = (fd, address, length) => {
  assert.equal(fd, 82); assert.equal(address, null); assert.equal(length, null); return 83;
};
provider.fcntl = (fd, command) => {
  if (fd === 83 && command === 4) { error = 4; return -1; } return 0;
};
function verify() {
  assert.equal(socketCount, 2); assert.deepEqual(closed, [81, 83, 82]);
  assert.deepEqual(polls, [[81, 4], [82, 1]]);
  console.log('failures-ok'); return unit();
}
";

#[test]
fn failed_connect_and_accept_close_new_sockets_and_keep_listener_and_custom_errors() {
    let fixture = Fixture::new(
        SCRIPT_FAILURES,
        &format!("{SCRIPT_PROVIDER}{SCRIPT_FAILURES_JS}"),
    );
    success(
        &fixture.start(false).finish(),
        "custom-111\nready\ncustom-4\nfailures-ok\n",
    );
}

const SCRIPT_UDP: &str = r#"def setup() -> IO(Unit): import "effect.js"
def raw() -> IO(Socket): import "effect.js"
def verify() -> IO(Unit): import "effect.js"
def bad(pair: Socket & Result<&1, &1, U32 & String, Unit>) -> IO(Socket):
  (socket, result) = pair
  match result:
    case Done{value}: IO.die(Socket, 1, "unexpected send")
    case Fail{(code, message)}:
      do IO<Socket>:
        Unit <- IO.print(message)
        return socket

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- raw()
    got : Socket & Result<&1, &1, U32 & String, Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 3)
    socket : Socket <- keep_poll(got)
    got : Socket & Result<&1, &1, U32 & String, Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 3)
    socket : Socket <- keep_poll(got)
    Unit <- IO.spawn(Unit, IO.print("ready"))
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 3)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 0)
    socket : Socket <- keep_packet(got)
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- UDP.send_to(socket, "127.0.0.1\0tail", 9, "x")
    socket : Socket <- bad(sent)
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- UDP.send_to(socket, "127.0.0.1", 65536, "x")
    socket : Socket <- bad(sent)
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- UDP.send_to(socket, "127.0.0.1", 9, "")
    socket : Socket <- keep_sent(sent)
    Unit <- Socket.close(socket)
    verify()
"#;
const SCRIPT_UDP_JS: &str = r"
let receives = 0, sends = 0;
function raw() { return 91; }
provider.recvfrom = (fd, bytes, maximum, flags, peer, length) => {
  assert.equal(fd, 91); assert.equal(flags, 0); assert.equal(length[0], 16);
  const index = receives++;
  assert.equal(maximum, index === 3 ? 0 : 3);
  if (index === 0) { error = 11; return -1; }
  peer.set([2, 0, 0x9c, 0x41, 127, 0, 0, 1]);
  if (index === 2) { bytes.set([97, 98, 99]); return 3; }
  assert.ok(index === 1 || index === 3); return 0;
};
provider.sendto = (fd, bytes, length, flags, address, addressLength) => {
  assert.equal(fd, 91); assert.equal(length, 0); assert.equal(bytes.length, 0);
  assert.equal(flags, 0); assert.equal(addressLength, 16);
  assert.deepEqual([...address.subarray(0, 8)], [2, 0, 0, 9, 127, 0, 0, 1]);
  if (sends++ === 0) { error = 11; return -1; } return 0;
};
function verify() {
  assert.equal(receives, 4); assert.equal(sends, 2); assert.deepEqual(closed, [91]);
  assert.deepEqual(polls, [[91, 1], [91, 1], [91, 4]]);
  console.log('udp-ok'); return unit();
}
";

#[test]
fn udp_distinguishes_none_empty_and_zero_length_and_retains_socket_after_bad_address() {
    let fixture = Fixture::new(SCRIPT_UDP, &format!("{SCRIPT_PROVIDER}{SCRIPT_UDP_JS}"));
    success(
        &fixture.start(false).finish(),
        "none\n127.0.0.1\n40001\npacket:\nready\n127.0.0.1\n40001\npacket:abc\n127.0.0.1\n40001\npacket:\ncustom-22\ncustom-22\nudp-ok\n",
    );
}

const TCP_CLIENT: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- TCP.send(socket, "request-é\0")
    socket : Socket <- keep_sent(sent)
    Unit <- IO.spawn(Unit, IO.print("ready"))
    got : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 3)
    socket : Socket <- keep_text(got)
    got : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 3)
    socket : Socket <- keep_text(got)
    got : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 64)
    socket : Socket <- keep_text(got)
    Socket.close(socket)
"#;

fn listener() -> TcpListener {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}

fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match listener.accept() {
            Ok((socket, _address)) => {
                socket.set_nonblocking(false).unwrap();
                socket.set_read_timeout(Some(TIMEOUT)).unwrap();
                socket.set_write_timeout(Some(TIMEOUT)).unwrap();
                return socket;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "Bend client did not connect");
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("accept failed: {error}"),
        }
    }
}

#[test]
fn native_provider_tcp_preserves_read_yield_short_receive_and_eof() {
    let peer = listener();
    let fixture = Fixture::new(
        &TCP_CLIENT.replace("PORT", &peer.local_addr().unwrap().port().to_string()),
        "",
    );
    let running = fixture.start(true);
    let mut socket = accept(&peer);
    socket.write_all(b"abcdef").unwrap();
    socket.shutdown(Shutdown::Write).unwrap();
    let mut received = Vec::new();
    socket.read_to_end(&mut received).unwrap();
    success(&running.finish(), "ready\ntext:abc\ntext:def\ntext:\n");
    assert_eq!(received, "request-é\0".as_bytes());
}

const UDP_ROUNDTRIP: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    got : Socket & Result<&1, &1, U32 & String, Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 64)
    socket : Socket <- keep_poll(got)
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- UDP.send_to(socket, "127.0.0.1", PORT, "")
    socket : Socket <- keep_sent(sent)
    Unit <- IO.spawn(Unit, IO.print("ready"))
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 3)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 64)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 0)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 4)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, U32 & String, Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 64)
    socket : Socket <- keep_poll(got)
    Socket.close(socket)
"#;

#[test]
fn native_provider_udp_preserves_empty_truncation_zero_length_and_sender() {
    let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    peer.set_read_timeout(Some(TIMEOUT)).unwrap();
    peer.set_write_timeout(Some(TIMEOUT)).unwrap();
    let port = peer.local_addr().unwrap().port();
    let fixture = Fixture::new(&UDP_ROUNDTRIP.replace("PORT", &port.to_string()), "");
    let running = fixture.start(true);
    let mut buffer = [0; 64];
    let (length, destination) = peer.recv_from(&mut buffer).unwrap();
    assert_eq!(length, 0);
    peer.send_to(b"0123456789", destination).unwrap();
    peer.send_to(b"tail-intact", destination).unwrap();
    peer.send_to(b"discarded", destination).unwrap();
    peer.send_to(b"", destination).unwrap();
    success(
        &running.finish(),
        &format!(
            "none\nready\n127.0.0.1\n{port}\npacket:012\n127.0.0.1\n{port}\npacket:tail-intact\n127.0.0.1\n{port}\npacket:\n127.0.0.1\n{port}\npacket:\nnone\n"
        ),
    );
}

const RAW_LISTENER: &str = r#"def open_listener() -> IO(Listener): import "effect.js"
def announce(listener: Listener) -> IO(Listener): import "effect.js"
def foreign_send(socket: Socket) -> IO(Socket): import "effect.js"
def accepted(pair: Listener & Result<&1, &1, U32 & String, Socket>) -> IO(Unit):
  (listener, result) = pair
  do IO<Unit>:
    socket : Socket <- IO.pass(Socket, result)
    Unit <- Listener.close(listener)
    got : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 3)
    socket : Socket <- keep_text(got)
    socket : Socket <- foreign_send(socket)
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- TCP.send(socket, "bend")
    socket : Socket <- keep_sent(sent)
    Socket.close(socket)

def main() -> IO(Unit):
  do IO<Unit>:
    listener : Listener <- open_listener()
    listener : Listener <- announce(listener)
    Unit <- IO.spawn(Unit, IO.print("ready"))
    pair : Listener & Result<&1, &1, U32 & String, Socket> <- TCP.accept(listener)
    accepted(pair)
"#;
const RAW_LISTENER_JS: &str = r"
const assert = require('node:assert/strict');
function open_listener() {
  const sys = io_sys(), fd = sys.socket(2, 1, 0);
  assert.ok(fd >= 0);
  const address = io_addr('127.0.0.1', 0);
  assert.equal(sys.bind(fd, sys.ptr(address), 16), 0);
  assert.equal(sys.listen(fd, 16), 0);
  assert.equal(sys.fcntl(fd, 4, sys.fcntl(fd, 3, 0) | (sys.mac ? 4 : 0x800)), 0);
  return fd;
}
function announce(fd) {
  const sys = io_sys(), address = new Uint8Array(16), length = new Uint32Array([16]);
  assert.equal(sys.getsockname(fd, sys.ptr(address), sys.ptr(length)), 0);
  console.log('port:' + ((address[2] << 8) | address[3])); return fd;
}
function foreign_send(fd) {
  const sys = io_sys(), bytes = new TextEncoder().encode('raw');
  assert.equal(sys.send(fd, sys.ptr(bytes), bytes.length, 0), 3); return fd;
}
";

#[test]
fn native_provider_exchanges_raw_foreign_and_builtin_listener_and_socket_descriptors() {
    for foreign_listener in [true, false] {
        let source = if foreign_listener {
            RAW_LISTENER.to_owned()
        } else {
            RAW_LISTENER.replace(
                "listener : Listener <- open_listener()",
                "listener : Listener <- IO.try(Listener, TCP.listen(0))",
            )
        };
        let fixture = Fixture::new(&source, RAW_LISTENER_JS);
        let running = fixture.start(true);
        let announced = running.line();
        let port = announced
            .trim()
            .strip_prefix("port:")
            .unwrap()
            .parse::<u16>()
            .unwrap();
        let mut peer =
            TcpStream::connect_timeout(&(Ipv4Addr::LOCALHOST, port).into(), TIMEOUT).unwrap();
        peer.set_read_timeout(Some(TIMEOUT)).unwrap();
        peer.set_write_timeout(Some(TIMEOUT)).unwrap();
        peer.write_all(b"abc").unwrap();
        let mut received = Vec::new();
        peer.read_to_end(&mut received).unwrap();
        assert_eq!(received, b"rawbend");
        success(
            &running.finish(),
            &format!("port:{port}\nready\ntext:abc\n"),
        );
    }
}

const REFUSED_CONNECT: &str = r#"def show(result: Result<&1, &1, U32 & String, Socket>) -> IO(Unit):
  match result:
    case Done{socket}: IO.die(Unit, 1, "unexpected connection")
    case Fail{(code, message)}: IO.print(U32.show(code))

def main() -> IO(Unit):
  do IO<Unit>:
    result : Result<&1, &1, U32 & String, Socket> <- TCP.connect("127.0.0.1", PORT)
    show(result)
"#;

#[test]
fn native_provider_connect_refusal_is_a_result_with_platform_error_number() {
    // A bound non-listening socket keeps the ephemeral port reserved throughout
    // the test, avoiding a race with another test binding a recently closed port.
    let reserved =
        socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).unwrap();
    reserved
        .bind(&std::net::SocketAddr::from((Ipv4Addr::LOCALHOST, 0)).into())
        .unwrap();
    let port = reserved.local_addr().unwrap().as_socket().unwrap().port();
    let fixture = Fixture::new(&REFUSED_CONNECT.replace("PORT", &port.to_string()), "");
    let code = if cfg!(windows) {
        10061
    } else if cfg!(target_os = "macos") {
        61
    } else {
        111
    };
    success(&fixture.start(true).finish(), &format!("{code}\n"));
}

const CLEANUP: &str = r#"def setup() -> IO(Unit): import "effect.js"
def replace_provider() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    Unit <- replace_provider()
    IO.die(Unit, 7, "halt")
"#;
const CLEANUP_JS: &str = r"
provider.socket = () => 101;
function replace_provider() {
  globalThis.BEND_SYS = { close() { throw Error('wrong owner'); } }; return unit();
}
process.on('exit', () => { assert.deepEqual(closed, [101]); });
";

#[test]
fn halt_cleans_owned_sockets_through_original_provider_after_global_replacement() {
    let fixture = Fixture::new(CLEANUP, &format!("{SCRIPT_PROVIDER}{CLEANUP_JS}"));
    let output = fixture.start(false).finish();
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"halt\n");
}

#[test]
fn zero_progress_host_send_fails_with_a_bounded_diagnostic() {
    let foreign = format!("{SCRIPT_PROVIDER}{SCRIPT_SEND_JS}\nprovider.send = () => 0;");
    let fixture = Fixture::new(SCRIPT_SEND, &foreign);
    let output = fixture.start(false).finish();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid host socket send count"));
}

#[test]
fn oversized_receive_refuses_host_allocation_and_closes_owned_socket() {
    let fixture = Fixture::new(
        r#"def setup() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- setup()
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    got : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 8388609)
    socket : Socket <- keep_packet(got)
    Socket.close(socket)
"#,
        &format!(
            "{SCRIPT_PROVIDER}\nprovider.socket = () => 101;\nprovider.recvfrom = () => {{ throw Error('host read must not run'); }};\nprocess.on('exit', () => assert.deepEqual(closed, [101]));"
        ),
    );
    let output = fixture.start(false).finish();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("network byte buffer budget exhausted")
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("host read must not run"));
}

#[test]
fn cli_packages_provider_and_refuses_to_replace_different_existing_provider() {
    let fixture = Fixture::new(
        r#"def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    Unit <- Socket.close(socket)
    IO.print("packaged")
"#,
        "",
    );
    let compile = || {
        Running::start(
            Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
                .args([
                    "compile",
                    "--executable",
                    "main.bend",
                    "--output",
                    "packaged.cjs",
                    "--force",
                ])
                .current_dir(&fixture.0),
        )
        .finish()
    };
    let output = compile();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let copied = fixture.0.join("teamy-bend-sys.node");
    assert!(copied.is_file());
    let output = Running::start(
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg("packaged.cjs")
            .current_dir(&fixture.0)
            .env_remove("TEAMY_BEND_SYS_MODULE"),
    )
    .finish();
    success(&output, "packaged\n");

    let original_program = fs::read(fixture.0.join("packaged.cjs")).unwrap();
    if cfg!(windows) {
        let original_provider = fs::read(&copied).unwrap();
        let output = Running::start(
            Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
                .args([
                    "compile",
                    "--executable",
                    "main.bend",
                    "--output",
                    "TEAMY-BEND-SYS.NODE",
                    "--force",
                ])
                .current_dir(&fixture.0),
        )
        .finish();
        assert!(!output.status.success());
        assert_eq!(fs::read(&copied).unwrap(), original_provider);
    }
    fs::write(&copied, b"different provider").unwrap();
    let output = compile();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("provider"));
    assert_eq!(
        fs::read(fixture.0.join("packaged.cjs")).unwrap(),
        original_program
    );
    assert_eq!(fs::read(copied).unwrap(), b"different provider");
}
