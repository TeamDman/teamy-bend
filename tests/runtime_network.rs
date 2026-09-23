// SPDX-License-Identifier: MPL-2.0
use std::cell::Cell;
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
            "teamy-bend-native-network-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        Self(path)
    }

    fn start(&self) -> Running {
        let mut child = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["run", "main.bend"])
            .current_dir(&self.0)
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
            "native network program exceeded deadline; stdout: {}; stderr: {}",
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

const CANCEL_RECEIVE: &str = r#"def received(pair: Socket & Result<&1, &1, (U32 & String), String>) -> IO(Unit):
  (socket, result) = pair
  do IO<Unit>:
    Unit <- IO.print("BAD resumed")
    Socket.close(socket)

def receive(socket: Socket) -> IO(Unit):
  IO.bind(Socket & Result<&1, &1, (U32 & String), String>, Unit, TCP.recv(socket, 1), received)

def main() -> IO(Unit):
  do IO<Unit>:
    socket : Socket <- IO.try(Socket, TCP.connect("127.0.0.1", PORT))
    Unit <- IO.spawn(Unit, receive(socket))
    Unit <- IO.sleep(0)
    IO.write("cancel")
"#;

#[test]
fn cancellation_closes_a_parked_socket_without_exiting_the_host_process() {
    struct CancelOutput<'a> {
        cancelled: &'a Cell<bool>,
        bytes: Vec<u8>,
    }
    impl Write for CancelOutput<'_> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            self.cancelled.set(true);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let peer = listener();
    let fixture = Fixture::new(
        &CANCEL_RECEIVE.replace("PORT", &peer.local_addr().unwrap().port().to_string()),
    );
    let book = check_executable(&load_executable(fixture.0.join("main.bend")).unwrap()).unwrap();
    let cancelled = Cell::new(false);
    let mut stdout = CancelOutput {
        cancelled: &cancelled,
        bytes: Vec::new(),
    };
    let mut stderr = Vec::new();
    let deadline = Instant::now() + TIMEOUT;
    let error = book
        .run_main(&mut stdout, &mut stderr, &|| {
            cancelled.get() || Instant::now() >= deadline
        })
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"), "{error}");
    assert!(
        cancelled.get(),
        "cancellation came from output, not the watchdog"
    );
    assert_eq!(stdout.bytes, b"cancel");
    assert!(stderr.is_empty());
    let mut socket = accept(&peer);
    let mut byte = [0];
    assert_eq!(
        socket.read(&mut byte).unwrap(),
        0,
        "the abandoned VM released its socket"
    );
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
