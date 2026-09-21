// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::Shutdown;
use std::net::SocketAddr;
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
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const TIMEOUT: Duration = Duration::from_secs(20);
const HELPERS: &str = r#"import Base
def keep_sent(pair: Socket & Result<&1, &1, (U32 & String), Unit>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    Unit <- IO.pass(Unit, result)
    return socket

def keep_text(pair: Socket & Result<&1, &1, (U32 & String), String>) -> IO(Socket):
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

def keep_packet(pair: Socket & Result<&1, &1, (U32 & String), String & U32 & String>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    packet : String & U32 & String <- IO.pass(String & U32 & String, result)
    Unit <- packet_text(packet)
    return socket

def poll_text(value: Maybe<&1, String & U32 & String>) -> IO(Unit):
  match value:
    case None{}: IO.print("none")
    case Some{packet}: packet_text(packet)

def keep_poll(pair: Socket & Result<&1, &1, (U32 & String), Maybe<&1, String & U32 & String>>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    value : Maybe<&1, String & U32 & String> <- IO.pass(Maybe<&1, String & U32 & String>, result)
    Unit <- poll_text(value)
    return socket
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-c-network-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        Self(path)
    }

    fn start(&self) -> Running {
        self.start_with_definitions(&[])
    }

    fn start_with_definitions(&self, definitions: &[&str]) -> Running {
        self.start_with_source_edit(definitions, |source| source)
    }

    fn start_with_source_edit(
        &self,
        definitions: &[&str],
        edit: impl FnOnce(String) -> String,
    ) -> Running {
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let source = edit(compile_executable_c(&checked).unwrap());
        let executable = executable_c_compiler::compile(&self.0, &source, definitions);
        let mut child = Command::new(executable)
            .current_dir(&self.0)
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
        Running {
            child,
            lines,
            stdout: Some(stdout),
            stderr: Some(stderr),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

struct Running {
    child: Child,
    lines: mpsc::Receiver<Vec<u8>>,
    stdout: Option<thread::JoinHandle<Vec<u8>>>,
    stderr: Option<thread::JoinHandle<Vec<u8>>>,
}

impl Running {
    fn line(&self) -> String {
        String::from_utf8(
            self.lines
                .recv_timeout(TIMEOUT)
                .expect("program output before deadline"),
        )
        .unwrap()
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
            "generated C network program exceeded deadline; stdout: {}; stderr: {}",
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

fn success(result: &Output, stdout: &str) {
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, stdout.as_bytes());
    assert!(result.stderr.is_empty());
}

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
                assert!(Instant::now() < deadline, "TCP connection before deadline");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("TCP accept failed: {error}"),
        }
    }
}

fn udp_peer() -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    socket.set_read_timeout(Some(TIMEOUT)).unwrap();
    socket.set_write_timeout(Some(TIMEOUT)).unwrap();
    socket
}

const TCP_CLIENT: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    sent : Socket & Result<&1, &1, (U32 & String), Unit> <- TCP.send(socket, "request-é\0")
    socket : Socket <- keep_sent(sent)
    Unit <- IO.spawn(Unit, IO.print("ready"))
    got : Socket & Result<&1, &1, (U32 & String), String> <- TCP.recv(socket, 3)
    socket : Socket <- keep_text(got)
    got : Socket & Result<&1, &1, (U32 & String), String> <- TCP.recv(socket, 3)
    socket : Socket <- keep_text(got)
    got : Socket & Result<&1, &1, (U32 & String), String> <- TCP.recv(socket, 64)
    socket : Socket <- keep_text(got)
    Socket.close(socket)
"#;

#[test]
fn tcp_preserves_short_receive_remainder_eof_and_native_text_bytes() {
    let peer = listener();
    let fixture =
        Fixture::new(&TCP_CLIENT.replace("PORT", &peer.local_addr().unwrap().port().to_string()));
    let running = fixture.start();
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
    got : Socket & Result<&1, &1, (U32 & String), Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 64)
    socket : Socket <- keep_poll(got)
    sent : Socket & Result<&1, &1, (U32 & String), Unit> <- UDP.send_to(socket, "127.0.0.1", PORT, "hello")
    socket : Socket <- keep_sent(sent)
    Unit <- IO.spawn(Unit, IO.print("ready"))
    got : Socket & Result<&1, &1, (U32 & String), String & U32 & String> <- UDP.recv_from(socket, 3)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, (U32 & String), String & U32 & String> <- UDP.recv_from(socket, 64)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, (U32 & String), String & U32 & String> <- UDP.recv_from(socket, 0)
    socket : Socket <- keep_packet(got)
    got : Socket & Result<&1, &1, (U32 & String), Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 64)
    socket : Socket <- keep_poll(got)
    Socket.close(socket)
"#;

#[test]
fn udp_poll_is_immediate_and_datagram_truncation_preserves_next_packet_and_sender() {
    let peer = udp_peer();
    let port = peer.local_addr().unwrap().port();
    let fixture = Fixture::new(&UDP_ROUNDTRIP.replace("PORT", &port.to_string()));
    let running = fixture.start();
    let mut buffer = [0; 64];
    let (length, destination) = peer.recv_from(&mut buffer).unwrap();
    assert_eq!(&buffer[..length], b"hello");
    peer.send_to(b"0123456789", destination).unwrap();
    peer.send_to(b"tail-intact", destination).unwrap();
    peer.send_to(b"discarded-by-zero-length-receive", destination)
        .unwrap();
    success(
        &running.finish(),
        &format!(
            "none\nready\n127.0.0.1\n{port}\npacket:012\n127.0.0.1\n{port}\npacket:tail-intact\n127.0.0.1\n{port}\npacket:\nnone\n"
        ),
    );
}

const INVALID_ADDRESSES: &str = r#"def failed_send(pair: Socket & Result<&1, &1, (U32 & String), Unit>) -> IO(Socket):
  (socket, result) = pair
  match result:
    case Done{value}: IO.die(Socket, 1, "BAD send succeeded")
    case Fail{error}:
      (code, message) = error
      do IO<Socket>:
        Unit <- IO.print(U32.show(code))
        return socket

def probe(socket: Socket, host: String, port: U32) -> IO(Socket):
  IO.bind(Socket & Result<&1, &1, (U32 & String), Unit>, Socket, UDP.send_to(socket, host, port, "x"), failed_send)

def opened(result: Result<&1, &1, (U32 & String), Socket>) -> IO(Unit):
  match result:
    case Done{socket}: IO.die(Unit, 1, "BAD open succeeded")
    case Fail{error}:
      (code, message) = error
      IO.print(U32.show(code))

def listened(result: Result<&1, &1, (U32 & String), Listener>) -> IO(Unit):
  match result:
    case Done{listener}: IO.die(Unit, 1, "BAD listen succeeded")
    case Fail{error}:
      (code, message) = error
      IO.print(U32.show(code))

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    socket : Socket <- probe(socket, "1.2.3.", 9)
    socket : Socket <- probe(socket, "1.2.3.256", 9)
    socket : Socket <- probe(socket, "1.2.3", 9)
    socket : Socket <- probe(socket, "0127.0.0.1", 9)
    socket : Socket <- probe(socket, "127.0.0.1\0tail", 9)
    socket : Socket <- probe(socket, "localhost", 9)
    socket : Socket <- probe(socket, "127.0.0.1", 65536)
    sent : Socket & Result<&1, &1, (U32 & String), Unit> <- UDP.send_to(socket, "127.0.0.1", PORT, "retained")
    socket : Socket <- keep_sent(sent)
    Unit <- Socket.close(socket)
    result : Result<&1, &1, (U32 & String), Socket> <- UDP.bind(65536)
    Unit <- opened(result)
    result : Result<&1, &1, (U32 & String), Listener> <- TCP.listen(4294967295)
    Unit <- listened(result)
    result : Result<&1, &1, (U32 & String), Socket> <- TCP.connect("127.0.0.1", 65536)
    Unit <- opened(result)
    result : Result<&1, &1, (U32 & String), Socket> <- TCP.connect("127.0.0.1\0tail", PORT)
    Unit <- opened(result)
    result : Result<&1, &1, (U32 & String), Socket> <- TCP.connect("0127.0.0.1", PORT)
    opened(result)
"#;

#[test]
fn invalid_addresses_and_ports_fail_without_losing_the_live_udp_socket() {
    let peer = udp_peer();
    let fixture = Fixture::new(
        &INVALID_ADDRESSES.replace("PORT", &peer.local_addr().unwrap().port().to_string()),
    );
    let running = fixture.start();
    let mut buffer = [0; 64];
    let (length, _source) = peer.recv_from(&mut buffer).unwrap();
    assert_eq!(&buffer[..length], b"retained");
    success(&running.finish(), &"22\n".repeat(12));
}

const TCP_LISTENER: &str = r#"def retry(port: U32, result: Result<&1, &1, (U32 & String), Listener>, rest: IO(Listener & U32)) -> IO(Listener & U32):
  match result:
    case Done{listener}: IO.pure(Listener & U32, (listener, port))
    case Fail{error}: rest

def listen(tries: Nat, +port: U32) -> IO(Listener & U32):
  match tries:
    case 0n: IO.die(Listener & U32, 1, "no free test port")
    case 1n+rest:
      fallback = listen(rest, U32.add(port, 1))
      do IO<Listener & U32>:
        result : Result<&1, &1, (U32 & String), Listener> <- TCP.listen(port)
        retry(port, result, fallback)

def accepted(pair: Listener & Result<&1, &1, (U32 & String), Socket>) -> IO(Unit):
  (listener, result) = pair
  do IO<Unit>:
    socket : Socket <- IO.pass(Socket, result)
    Unit <- Listener.close(listener)
    got : Socket & Result<&1, &1, (U32 & String), String> <- TCP.recv(socket, 64)
    socket : Socket <- keep_text(got)
    sent : Socket & Result<&1, &1, (U32 & String), Unit> <- TCP.send(socket, "accepted")
    socket : Socket <- keep_sent(sent)
    Socket.close(socket)

def listening(pair: Listener & U32) -> IO(Unit):
  (listener, port) = pair
  do IO<Unit>:
    Unit <- IO.print(U32.show(port))
    Unit <- IO.spawn(Unit, IO.print("ready"))
    pair : Listener & Result<&1, &1, (U32 & String), Socket> <- TCP.accept(listener)
    accepted(pair)

def main() -> IO(Unit): IO.bind(Listener & U32, Unit, listen(16n, PORT), listening)
"#;

#[test]
fn listener_announces_its_bound_port_and_accept_yields_before_receiving() {
    // The Bend fixture retries and announces the port it actually acquired;
    // releasing the reservation cannot create a port-selection race in the test.
    let reservation = listener();
    let port = reservation.local_addr().unwrap().port();
    let fixture = Fixture::new(&TCP_LISTENER.replace("PORT", &port.to_string()));
    drop(reservation);
    let running = fixture.start();
    let actual_port: u16 = running.line().trim().parse().unwrap();
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, actual_port));
    let mut socket = TcpStream::connect_timeout(&address, TIMEOUT).unwrap();
    socket.set_read_timeout(Some(TIMEOUT)).unwrap();
    socket.set_write_timeout(Some(TIMEOUT)).unwrap();
    socket.write_all(b"client").unwrap();
    socket.shutdown(Shutdown::Write).unwrap();
    let mut received = Vec::new();
    socket.read_to_end(&mut received).unwrap();
    success(
        &running.finish(),
        &format!("{actual_port}\nready\ntext:client\n"),
    );
    assert_eq!(received, b"accepted");
}

const MANY_READERS: &str = r#"def received(pair: Socket & Result<&1, &1, (U32 & String), String>) -> IO(Unit):
  (socket, result) = pair
  do IO<Unit>:
    Unit <- IO.print("BAD receiver continued")
    Socket.close(socket)

def quiet(port: U32, ready: Chan(Unit)) -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", port))
    sent : Bool <- Chan.send(Unit, ready, Unit{})
    pair : Socket & Result<&1, &1, (U32 & String), String> <- TCP.recv(socket, 1)
    received(pair)

def launch(count: Nat, port: U32, ready: Chan(Unit)) -> IO(Unit):
  match count:
    case 0n: IO.pure(Unit, Unit{})
    case 1n+rest:
      +p = port
      +r = ready
      do IO<Unit>:
        Unit <- IO.spawn(Unit, quiet(p, r))
        launch(rest, p, r)

def wait_ready(count: Nat, ready: Chan(Unit)) -> IO(Unit):
  match count:
    case 0n: IO.pure(Unit, Unit{})
    case 1n+rest:
      +r = ready
      do IO<Unit>:
        value : Maybe<&1, Unit> <- Chan.recv(Unit, r)
        wait_ready(rest, r)

def start(ready: Chan(Unit)) -> IO(Unit):
  +r = ready
  do IO<Unit>:
    Unit <- launch(70n, PORT, r)
    Unit <- wait_ready(70n, r)
    Unit <- IO.sleep(0)
    file : File <- IO.try(File, File.open("canary.tmp", "r"))
    Unit <- File.close(file)
    Unit <- IO.print("workers free")
    IO.die(Unit, 0, "")

def main() -> IO(Unit):
  do IO<Unit>:
    ready : Chan(Unit) <- Chan.new(Unit, 70)
    start(ready)
"#;

#[test]
fn seventy_parked_receivers_leave_file_workers_available_and_halt_closes_all_sockets() {
    let peer = listener();
    let fixture =
        Fixture::new(&MANY_READERS.replace("PORT", &peer.local_addr().unwrap().port().to_string()));
    std::fs::write(fixture.0.join("canary.tmp"), "canary").unwrap();
    let running = fixture.start();
    let mut sockets = (0..70).map(|_| accept(&peer)).collect::<Vec<_>>();
    let output = running.finish();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"workers free\n");
    assert_eq!(output.stderr, b"\n");
    for socket in &mut sockets {
        let mut byte = [0];
        assert_eq!(
            socket.read(&mut byte).unwrap(),
            0,
            "Halt closes every parked socket"
        );
    }
}

const UDP_EMPTY: &str = r#"def polled(socket: Socket, value: Maybe<&1, String & U32 & String>, rest: Socket -> IO(Unit)) -> IO(Unit):
  match value:
    case None{}:
      do IO<Unit>:
        Unit <- IO.sleep(100)
        rest(socket)
    case Some{packet}:
      do IO<Unit>:
        Unit <- packet_text(packet)
        Socket.close(socket)

def poll_result(pair: Socket & Result<&1, &1, (U32 & String), Maybe<&1, String & U32 & String>>, rest: Socket -> IO(Unit)) -> IO(Unit):
  (socket, result) = pair
  do IO<Unit>:
    value : Maybe<&1, String & U32 & String> <- IO.pass(Maybe<&1, String & U32 & String>, result)
    polled(socket, value, rest)

def until_packet(count: Nat, socket: Socket) -> IO(Unit):
  match count:
    case 0n: IO.die(Unit, 1, "datagram deadline")
    case 1n+rest:
      do IO<Unit>:
        pair : Socket & Result<&1, &1, (U32 & String), Maybe<&1, String & U32 & String>> <- UDP.poll(socket, 64)
        poll_result(pair, next => until_packet(rest, next))

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, UDP.bind(0))
    sent : Socket & Result<&1, &1, (U32 & String), Unit> <- UDP.send_to(socket, "127.0.0.1", PORT, "")
    socket : Socket <- keep_sent(sent)
    until_packet(64n, socket)
"#;

#[test]
fn udp_sends_empty_datagrams_and_poll_distinguishes_them_from_no_datagram() {
    let peer = udp_peer();
    let port = peer.local_addr().unwrap().port();
    let fixture = Fixture::new(&UDP_EMPTY.replace("PORT", &port.to_string()));
    let running = fixture.start();
    let mut buffer = [0; 1];
    let (length, destination) = peer.recv_from(&mut buffer).unwrap();
    assert_eq!(length, 0);
    peer.send_to(b"", destination).unwrap();
    success(&running.finish(), &format!("127.0.0.1\n{port}\npacket:\n"));
}

const TCP_SLOW_SEND: &str = r#"def chunk(count: Nat, acc: String) -> String:
  match count:
    case 0n: acc
    case 1n+rest: chunk(rest, String.append("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef", acc))

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    sent : Socket & Result<&1, &1, (U32 & String), Unit> <- TCP.send(socket, chunk(Nat.mul(64n, 3n), ""))
    socket : Socket <- keep_sent(sent)
    Unit <- Socket.close(socket)
    IO.print("sent")
"#;

#[test]
fn tcp_send_preserves_the_complete_payload_with_a_slow_reader() {
    let peer = listener();
    let payload = "0123456789abcdef".repeat(768);
    let source = TCP_SLOW_SEND.replace("PORT", &peer.local_addr().unwrap().port().to_string());
    let fixture = Fixture::new(&source);
    let running = fixture.start();
    let mut socket = accept(&peer);
    let mut received = Vec::new();
    let mut buffer = [0; 127];
    loop {
        let length = socket.read(&mut buffer).unwrap();
        if length == 0 {
            break;
        }
        received.extend_from_slice(&buffer[..length]);
        thread::sleep(Duration::from_millis(1));
    }
    success(&running.finish(), "sent\n");
    assert_eq!(received, payload.as_bytes());
}

const REFUSED_CONNECT: &str = r#"def report(result: Result<&1, &1, (U32 & String), Socket>) -> IO(Unit):
  match result:
    case Done{socket}: IO.die(Unit, 1, "BAD connected")
    case Fail{error}:
      (code, message) = error
      IO.print(U32.show(code))

def main() -> IO(Unit):
  IO.bind(Result<&1, &1, (U32 & String), Socket>, Unit, TCP.connect("127.0.0.1", PORT), report)
"#;

#[test]
fn refused_connect_wakes_and_reports_the_native_socket_error() {
    // Bind without listening so the refused endpoint stays reserved throughout
    // the test; another process cannot win a freshly-closed-port race.
    let reservation = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .unwrap();
    reservation
        .bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, 0)).into())
        .unwrap();
    let port = reservation
        .local_addr()
        .unwrap()
        .as_socket()
        .unwrap()
        .port();
    let fixture = Fixture::new(&REFUSED_CONNECT.replace("PORT", &port.to_string()));
    #[cfg(windows)]
    let code = 10061;
    #[cfg(unix)]
    let code = libc::ECONNREFUSED;
    success(&fixture.start().finish(), &format!("{code}\n"));
}

const RAW_LISTENER: &str = r#"def open_listener() -> IO(Listener): import "effect.c"
def announce(listener: Listener) -> IO(Listener): import "effect.c"
def foreign_send(socket: Socket) -> IO(Socket): import "effect.c"
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

const RAW_LISTENER_C: &str = r#"
#ifdef _WIN32
typedef SOCKET TestSocket;
typedef int TestSockLen;
#define TEST_BAD_SOCKET INVALID_SOCKET
#else
typedef int TestSocket;
typedef socklen_t TestSockLen;
#define TEST_BAD_SOCKET (-1)
#endif
static Term open_listener_run(Env e, Term *fields, IoWork *work) {
  struct sockaddr_in address = {0};
  TestSocket listener = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
  (void)e; (void)fields; (void)work;
  if (listener == TEST_BAD_SOCKET) abort();
  address.sin_family = AF_INET;
  address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
  if (bind(listener, (const struct sockaddr *)&address, sizeof(address)) != 0 || listen(listener, 16) != 0) abort();
#ifdef _WIN32
  u_long mode = 1;
  if (ioctlsocket(listener, FIONBIO, &mode) != 0) abort();
#else
  if (fcntl(listener, F_SETFL, fcntl(listener, F_GETFL, 0) | O_NONBLOCK) != 0) abort();
#endif
  return io_hand((u64)(uintptr_t)listener);
}
static Term announce_run(Env e, Term *fields, IoWork *work) {
  struct sockaddr_in address = {0}; TestSockLen length = sizeof(address);
  TestSocket listener = (TestSocket)(uintptr_t)io_hand_v(fields[0]);
  (void)e; (void)work;
  if (getsockname(listener, (struct sockaddr *)&address, &length) != 0) abort();
  printf("port:%u\n", (unsigned)ntohs(address.sin_port));
  fflush(stdout);
  return fields[0];
}
static Term foreign_send_run(Env e, Term *fields, IoWork *work) {
  TestSocket socket = (TestSocket)(uintptr_t)io_hand_v(fields[0]);
  (void)e; (void)work;
  if (send(socket, "raw", 3, 0) != 3) abort();
  return fields[0];
}
static void __attribute__((constructor)) raw_listener_use(void) {
#ifdef CID_OPEN_LISTENER
  io_eff(CID_OPEN_LISTENER, open_listener_run, 0);
#endif
  io_eff(CID_ANNOUNCE, announce_run, 0);
  io_eff(CID_FOREIGN_SEND, foreign_send_run, 0);
}
"#;

#[test]
fn raw_foreign_and_builtin_listeners_exchange_native_socket_descriptors() {
    for foreign_listener in [true, false] {
        let source = if foreign_listener {
            RAW_LISTENER.to_owned()
        } else {
            RAW_LISTENER.replace(
                "listener : Listener <- open_listener()",
                "listener : Listener <- IO.try(Listener, TCP.listen(0))",
            )
        };
        let fixture = Fixture::new(&source);
        std::fs::write(fixture.0.join("effect.c"), RAW_LISTENER_C).unwrap();
        let running = fixture.start();
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

const NATIVE_DECODE: &str = r#"def code_line(text: String) -> String:
  match text:
    case SNil{}: ""
    case SCon{Chr{code}, rest}: U32.show(code) ++ "," ++ code_line(rest)

def show_codes(pair: Socket & Result<&1, &1, U32 & String, String>) -> IO(Socket):
  (socket, result) = pair
  do IO<Socket>:
    text : String <- IO.pass(String, result)
    Unit <- IO.print(code_line(text))
    return socket

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- TCP.send(socket, "")
    socket : Socket <- keep_sent(sent)
    received : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 64)
    socket : Socket <- show_codes(received)
    Socket.close(socket)
"#;

#[test]
fn tcp_empty_send_and_malformed_text_preserve_native_c_codec_values() {
    let peer = listener();
    let fixture = Fixture::new(
        &NATIVE_DECODE.replace("PORT", &peer.local_addr().unwrap().port().to_string()),
    );
    let running = fixture.start();
    let mut socket = accept(&peer);
    socket
        .write_all(&[
            0xef, 0xbb, 0xbf, 0x80, 0xc0, 0x80, 0xed, 0xa0, 0x80, 0xf4, 0x90, 0x80, 0x80, 0xff,
            0xc3, 0,
        ])
        .unwrap();
    socket.shutdown(Shutdown::Write).unwrap();
    let mut received = Vec::new();
    socket.read_to_end(&mut received).unwrap();
    assert!(received.is_empty());
    success(&running.finish(), "65279,128,0,55296,1114112,255,195,0,\n");
}

#[test]
fn oversized_tcp_and_udp_receive_requests_fail_before_running_the_continuation() {
    for (open, operation, result) in [
        ("TCP.connect(\"127.0.0.1\", PORT)", "TCP.recv", "String"),
        ("UDP.bind(0)", "UDP.recv_from", "String & U32 & String"),
        (
            "UDP.bind(0)",
            "UDP.poll",
            "Maybe<&1, String & U32 & String>",
        ),
    ] {
        // An IO_READ request initially parks; a readable UDP socket is needed
        // before its receive body can reject the allocation.
        let peer = listener();
        let udp = udp_peer();
        let handshake = if operation == "UDP.recv_from" {
            "    sent : Socket & Result<&1, &1, U32 & String, Unit> <- UDP.send_to(socket, \"127.0.0.1\", PORT, \"ready\")\n    socket : Socket <- keep_sent(sent)\n"
        } else {
            ""
        };
        let source = format!(
            "def main() -> IO(Unit):\n  do IO<Unit>:\n    socket : Socket <- IO.try(Socket, {open})\n{handshake}    result : Socket & Result<&1, &1, U32 & String, {result}> <- {operation}(socket, 8388609)\n    IO.print(\"BAD\")\n"
        );
        let port = if operation == "TCP.recv" {
            peer.local_addr().unwrap().port()
        } else {
            udp.local_addr().unwrap().port()
        };
        let source = source.replace("PORT", &port.to_string());
        let fixture = Fixture::new(&source);
        let running = fixture.start();
        if operation == "TCP.recv" {
            let mut socket = accept(&peer);
            socket.write_all(b"ready").unwrap();
        } else if operation == "UDP.recv_from" {
            let mut bytes = [0; 16];
            let (length, destination) = udp.recv_from(&mut bytes).unwrap();
            assert_eq!(&bytes[..length], b"ready");
            udp.send_to(b"ready", destination).unwrap();
        }
        let output = running.finish();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("budget"));
    }
}

const CLEANUP: &str = r#"def remember_socket(socket: Socket) -> IO(Socket): import "effect.c"
def remember_listener(listener: Listener) -> IO(Listener): import "effect.c"
def waiting(socket: Socket) -> IO(Unit):
  do IO<Unit>:
    result : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 1)
    IO.print("BAD resumed")

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    socket : Socket <- remember_socket(socket)
    udp : Socket <- IO.try(Socket, UDP.bind(0))
    udp : Socket <- remember_socket(udp)
    listener : Listener <- IO.try(Listener, TCP.listen(0))
    listener : Listener <- remember_listener(listener)
ENDING
"#;

const CLEANUP_C: &str = r"
#ifdef _WIN32
typedef SOCKET CleanupSocket;
typedef int CleanupSockLen;
#else
typedef int CleanupSocket;
typedef socklen_t CleanupSockLen;
#endif
static CleanupSocket remembered[3];
static unsigned remembered_count;
static Term remember_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work;
  if (remembered_count >= 3) abort();
  remembered[remembered_count++] = (CleanupSocket)(uintptr_t)io_hand_v(fields[0]);
  return fields[0];
}
static void observe_closed_sockets(void) {
  if (remembered_count != 3) abort();
  for (unsigned index = 0; index < remembered_count; ++index) {
    int kind = 0; CleanupSockLen length = sizeof(kind);
    if (getsockopt(remembered[index], SOL_SOCKET, SO_TYPE, (char *)&kind, &length) == 0) abort();
#ifdef _WIN32
    if (WSAGetLastError() != WSAENOTSOCK) abort();
#else
    if (errno != EBADF) abort();
#endif
  }
#ifdef _WIN32
  (void)WSACleanup();
#endif
}
static void __attribute__((constructor)) cleanup_use(void) {
#ifdef _WIN32
  /* Keep Winsock live after tb_run's release: WSACleanup must not mask leaked
   * sockets by destroying them when its final process reference disappears. */
  WSADATA data;
  if (WSAStartup(MAKEWORD(2, 2), &data) != 0) abort();
#endif
  io_eff(CID_REMEMBER_SOCKET, remember_run, 0);
  io_eff(CID_REMEMBER_LISTENER, remember_run, 0);
  if (atexit(observe_closed_sockets) != 0) abort();
}
";

#[test]
fn normal_exit_and_halt_close_owned_sockets_before_process_and_winsock_teardown() {
    for halt in [false, true] {
        let peer = listener();
        let ending = if halt {
            "    Unit <- IO.spawn(Unit, waiting(socket))\n    Unit <- IO.sleep(0)\n    IO.die(Unit, 7, \"halt\")"
        } else {
            "    IO.pure(Unit, Unit{})"
        };
        let source = CLEANUP
            .replace("PORT", &peer.local_addr().unwrap().port().to_string())
            .replace("ENDING", ending);
        let fixture = Fixture::new(&source);
        std::fs::write(fixture.0.join("effect.c"), CLEANUP_C).unwrap();
        let running = fixture.start();
        let mut socket = accept(&peer);
        let output = running.finish();
        assert_eq!(
            output.status.code(),
            Some(if halt { 7 } else { 0 }),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, if halt { &b"halt\n"[..] } else { &b""[..] });
        assert_eq!(socket.read(&mut [0]).unwrap(), 0);
    }
}

const CLOSED_SOCKET: &str = r#"def closed_socket() -> IO(Socket): import "effect.c"
def inspect(socket: Socket) -> IO(Socket): import "effect.c"
def report(pair: Socket & Result<&1, &1, U32 & String, String>) -> IO(Socket):
  (socket, result) = pair
  match result:
    case Done{value}: IO.die(Socket, 1, "BAD receive succeeded")
    case Fail{error}:
      (code, message) = error
      do IO<Socket>:
        Unit <- IO.print(U32.show(code))
        return socket

def idle(socket: Socket) -> IO(Unit):
  do IO<Unit>:
    packet : Socket & Result<&1, &1, U32 & String, String & U32 & String> <- UDP.recv_from(socket, 1)
    IO.print("BAD idle socket resumed")

def main() -> IO(Unit):
  do IO<Unit>:
SETUP
    socket : Socket <- closed_socket()
    result : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 1)
    socket : Socket <- report(result)
    socket : Socket <- inspect(socket)
    Unit <- Socket.close(socket)
    Unit <- IO.print("closed")
FINISH
"#;

const CLOSED_SOCKET_C: &str = r"
static Term closed_value;
static Term closed_socket_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
#ifdef _WIN32
  SOCKET socket_value = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
  if (socket_value == INVALID_SOCKET) abort();
  if (closesocket(socket_value) != 0) abort();
#else
  int socket_value = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
  if (socket_value < 0 || close(socket_value) != 0) abort();
#endif
  closed_value = io_hand((u64)(uintptr_t)socket_value);
  return closed_value;
}
static Term inspect_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work;
  if (fields[0] != closed_value) abort();
  return fields[0];
}
static void __attribute__((constructor)) closed_socket_use(void) {
  io_eff(CID_CLOSED_SOCKET, closed_socket_run, 0);
  io_eff(CID_INSPECT, inspect_run, 0);
}
";

#[test]
fn closed_local_socket_receive_returns_its_os_error_and_original_handle() {
    for mixed_wait in [false, true] {
        let source = CLOSED_SOCKET
            .replace(
                "SETUP",
                if mixed_wait {
                    "    waiting : Socket <- IO.try(Socket, UDP.bind(0))\n    Unit <- IO.spawn(Unit, idle(waiting))"
                } else {
                    ""
                },
            )
            .replace(
                "FINISH",
                if mixed_wait {
                    "    IO.die(Unit, 0, \"\")"
                } else {
                    "    IO.pure(Unit, Unit{})"
                },
            );
        let fixture = Fixture::new(&source);
        std::fs::write(fixture.0.join("effect.c"), CLOSED_SOCKET_C).unwrap();
        let output = fixture.start().finish();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let error = if cfg!(windows) { 10038 } else { 9 };
        assert_eq!(output.stdout, format!("{error}\nclosed\n").as_bytes());
        assert_eq!(
            output.stderr,
            if mixed_wait { &b"\n"[..] } else { &b""[..] }
        );
    }
}

const BACKPRESSURE: &str = r#"def tune(socket: Socket) -> IO(Socket): import "effect.c"
def payload() -> IO(String): import "effect.c"
def observed() -> IO(Bool): import "effect.c"
def sender(socket: Socket, text: String) -> IO(Unit):
  do IO<Unit>:
    sent : Socket & Result<&1, &1, U32 & String, Unit> <- TCP.send(socket, text)
    socket : Socket <- keep_sent(sent)
    Unit <- Socket.close(socket)
    IO.print("sent")

def check(value: Bool) -> IO(Unit):
  match value:
    case False{}: IO.die(Unit, 1, "send did not park with a retained suffix")
    case True{}: IO.pure(Unit, Unit{})

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    socket : Socket <- tune(socket)
    ready : Socket & Result<&1, &1, U32 & String, String> <- TCP.recv(socket, 1)
    socket : Socket <- keep_text(ready)
    text : String <- payload()
    Unit <- IO.spawn(Unit, sender(socket, text))
    Unit <- IO.sleep(0)
    value : Bool <- observed()
    check(value)
"#;

const BACKPRESSURE_C: &str = r#"
static intptr_t sending_handle;
static Term tune_run(Env e, Term *fields, IoWork *work) {
  int size = 1024; (void)e; (void)work;
  sending_handle = (intptr_t)io_hand_v(fields[0]);
#ifdef _WIN32
  if (setsockopt((SOCKET)(uintptr_t)sending_handle, SOL_SOCKET, SO_SNDBUF, (const char *)&size, sizeof(size)) != 0) abort();
#else
  if (setsockopt((int)sending_handle, SOL_SOCKET, SO_SNDBUF, &size, sizeof(size)) != 0) abort();
#endif
  return fields[0];
}
static Term payload_run(Env e, Term *fields, IoWork *work) {
  const u32 size = 512 * 1024;
  char *bytes = (char *)io_mem(malloc(size));
  Term value; (void)fields; (void)work;
  for (u32 index = 0; index < size; ++index) bytes[index] = (char)('a' + index % 23);
  value = io_str(e, bytes, size);
  free(bytes);
  return value;
}
static Term observed_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  for (IoAct *action = io_park.head; action != NULL; action = action->next) {
    if (action->work.hand == sending_handle && action->work.pack == tb_tcp_send_more
        && action->work.data != NULL && action->work.size == 512 * 1024
        && action->work.made > 0 && (u64)action->work.made < action->work.size) {
      printf("parked:%llu\n", (unsigned long long)action->work.made);
      fflush(stdout);
      test_send_released = true;
      return term_pak(CID_TRUE, 0);
    }
  }
  return term_pak(CID_FALSE, 0);
}
static void __attribute__((constructor)) backpressure_use(void) {
  io_eff(CID_TUNE, tune_run, 0);
  io_eff(CID_PAYLOAD, payload_run, 0);
  io_eff(CID_OBSERVED, observed_run, 0);
}
"#;

const SCRIPTED_SEND: &str = r"
static u32 test_send_count;
static bool test_send_released;
static intptr_t test_send_system(intptr_t descriptor, const char *data, u32 size, int flags) {
  intptr_t count;
  if (!test_send_released) {
    if (test_send_count == 32768) {
#ifdef _WIN32
      WSASetLastError(WSAEWOULDBLOCK);
#else
      errno = EWOULDBLOCK;
#endif
      return -1;
    }
    if (size > 32768 - test_send_count) size = 32768 - test_send_count;
  }
  count = send((TBNetRaw)descriptor, data, (int)size, flags);
  if (count > 0) test_send_count += (u32)count;
  return count;
}
";

#[test]
fn tcp_send_retains_its_offset_after_a_controlled_short_send_and_would_block() {
    const BYTES: usize = 512 * 1024;
    let peer = listener();
    socket2::SockRef::from(&peer)
        .set_recv_buffer_size(1024)
        .unwrap();
    let fixture =
        Fixture::new(&BACKPRESSURE.replace("PORT", &peer.local_addr().unwrap().port().to_string()));
    std::fs::write(fixture.0.join("effect.c"), BACKPRESSURE_C).unwrap();
    // Winsock can buffer the whole payload despite small requested buffers.
    // This seam sends the first 32 KiB on the real connection, then reports
    // WouldBlock until the timer observes the parked action and saved offset.
    // Every remaining byte still travels through the real socket to EOF.
    let running = fixture.start_with_source_edit(&[], |source| {
        let declaration = "static intptr_t tb_net_send_system(";
        let send = "return send((TBNetRaw)descriptor, data, (int)size, flags);";
        assert_eq!(source.matches(declaration).count(), 1);
        assert_eq!(source.matches(send).count(), 1);
        source
            .replacen(declaration, &format!("{SCRIPTED_SEND}\n{declaration}"), 1)
            .replacen(
                send,
                "return test_send_system(descriptor, data, size, flags);",
                1,
            )
    });
    let mut socket = accept(&peer);
    socket2::SockRef::from(&socket)
        .set_recv_buffer_size(1024)
        .unwrap();
    socket.write_all(b"r").unwrap();
    assert_eq!(running.line(), "text:r\n");
    let observation = running.line();
    if !observation.starts_with("parked:") {
        let output = running.finish();
        panic!(
            "expected parked send observation, got {observation:?}; exit={:?}, stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let offset = observation
        .trim()
        .strip_prefix("parked:")
        .unwrap_or_else(|| panic!("expected parked send observation, got {observation:?}"))
        .parse::<usize>()
        .unwrap();
    assert_eq!(offset, 32 * 1024);
    socket2::SockRef::from(&socket)
        .set_recv_buffer_size(1024 * 1024)
        .unwrap();
    let mut received = Vec::new();
    socket.read_to_end(&mut received).unwrap();
    let expected = (0..BYTES)
        .map(|index| b'a' + u8::try_from(index % 23).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(received, expected);
    success(&running.finish(), &format!("text:r\n{observation}sent\n"));
}
