// SPDX-License-Identifier: MPL-2.0
use super::*;
use std::io::Read;
use std::io::Write;

fn socket_registration(socket: &NativeSocket, interest: Interest) -> Registration<'_> {
    Registration {
        source: Source::Socket(socket),
        interest,
    }
}

fn wait_socket(socket: &NativeSocket, interest: Interest) {
    assert_eq!(
        poll(&[socket_registration(socket, interest)], 2_000).unwrap(),
        [true]
    );
}

fn connected() -> (NativeSocket, NativeSocket) {
    let listener = listen(0).unwrap();
    let connection = connect(
        b"127.0.0.1",
        u32::from(listener.local_addr().unwrap().port()),
    )
    .unwrap();
    if connection.pending {
        wait_socket(&connection.socket, Interest::Write);
        finish_connect(&connection.socket).unwrap();
    }
    assert_eq!(
        poll(
            &[Registration {
                source: Source::Listener(&listener),
                interest: Interest::Read
            }],
            2_000
        )
        .unwrap(),
        [true]
    );
    let Step::Ready(accepted) = accept(&listener).unwrap() else {
        panic!("readable listener did not accept")
    };
    (connection.socket, accepted)
}

fn send_all(socket: &NativeSocket, mut bytes: &[u8]) {
    while !bytes.is_empty() {
        match send(socket, bytes).unwrap() {
            Step::Ready(count) => bytes = &bytes[count..],
            Step::Wait => wait_socket(socket, Interest::Write),
        }
    }
}

fn receive(socket: &NativeSocket, max: u32) -> Vec<u8> {
    wait_socket(socket, Interest::Read);
    let Step::Ready(bytes) = recv(socket, max, 100).unwrap() else {
        panic!("readable socket did not receive")
    };
    bytes
}

#[test]
fn addresses_reject_non_numeric_or_noncanonical_ipv4_and_large_ports() {
    for host in [
        b"localhost".as_slice(),
        b"127.1",
        b"01.2.3.4",
        b"1.2.3.04",
        b"1.2.3.256",
        b"1.2.3.4.5",
        b"1.2.3.",
        b"+1.2.3.4",
        b"1.2.3.4\0",
        b" 1.2.3.4",
        b"::1",
        b"\xff.2.3.4",
    ] {
        assert!(matches!(
            address(host, 80),
            Err(Failure::Io(Error { code: 22, .. }))
        ));
    }
    assert!(matches!(
        address(b"127.0.0.1", 65536),
        Err(Failure::Io(Error { code: 22, .. }))
    ));
    assert_eq!(
        address(b"0.1.255.0", 65535)
            .unwrap()
            .as_socket_ipv4()
            .unwrap(),
        SocketAddrV4::new(Ipv4Addr::new(0, 1, 255, 0), 65535)
    );
}

#[test]
fn tcp_accept_connect_short_reads_and_eof_preserve_stream_bytes() {
    let listener = listen(0).unwrap();
    assert!(matches!(accept(&listener).unwrap(), Step::Wait));
    let (client, server) = connected();
    assert_eq!(recv(&server, 10, 100).unwrap(), Step::Wait);
    send_all(&client, b"abc\0\xffz");
    assert_eq!(receive(&server, 3), b"abc");
    assert_eq!(receive(&server, 3), b"\0\xffz");
    drop(client);
    assert!(receive(&server, 10).is_empty());
}

#[test]
fn refused_connect_becomes_ready_and_so_error_reports_failure() {
    // Own this port for the whole test, but never listen: refusal cannot race
    // another parallel test reusing a just-closed listener's ephemeral port.
    let occupied = socket2::Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    occupied
        .bind(&SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into())
        .unwrap();
    let port = u32::from(
        occupied
            .local_addr()
            .unwrap()
            .as_socket_ipv4()
            .unwrap()
            .port(),
    );
    match connect(b"127.0.0.1", port) {
        Ok(connection) => {
            assert!(connection.pending);
            assert_eq!(
                poll(
                    &[socket_registration(&connection.socket, Interest::Write)],
                    5_000
                )
                .unwrap(),
                [true]
            );
            assert!(matches!(
                finish_connect(&connection.socket),
                Err(Failure::Io(_))
            ));
        }
        Err(Failure::Io(_)) => {}
        Err(other) => panic!("unexpected connection failure: {other:?}"),
    }
}

#[test]
fn udp_poll_truncates_one_datagram_and_retains_peer_and_next_datagram() {
    let receiver = bind(0).unwrap();
    let sender = bind(0).unwrap();
    let port = u32::from(receiver.local_addr().unwrap().port());
    assert_eq!(recv_from(&receiver, 10, 100).unwrap(), Step::Wait);
    assert_eq!(
        send_to(&sender, b"127.0.0.1", port, b"abcdef").unwrap(),
        Step::Ready(())
    );
    assert_eq!(
        send_to(&sender, b"127.0.0.1", port, b"tail").unwrap(),
        Step::Ready(())
    );
    wait_socket(&receiver, Interest::Read);
    let Step::Ready(first) = recv_from(&receiver, 3, 100).unwrap() else {
        panic!("missing first datagram")
    };
    assert_eq!(first.data, b"abc");
    assert_eq!(first.host, b"127.0.0.1");
    assert_eq!(first.port, u32::from(sender.local_addr().unwrap().port()));
    wait_socket(&receiver, Interest::Read);
    let Step::Ready(second) = recv_from(&receiver, 100, 100).unwrap() else {
        panic!("missing second datagram")
    };
    assert_eq!(second.data, b"tail");
    assert_eq!(recv_from(&receiver, 100, 100).unwrap(), Step::Wait);
}

#[test]
fn zero_length_udp_receive_discards_datagram_and_empty_datagrams_are_not_eof() {
    let receiver = bind(0).unwrap();
    let sender = bind(0).unwrap();
    let port = u32::from(receiver.local_addr().unwrap().port());
    assert_eq!(
        send_to(&sender, b"127.0.0.1", port, b"discard").unwrap(),
        Step::Ready(())
    );
    wait_socket(&receiver, Interest::Read);
    let Step::Ready(first) = recv_from(&receiver, 0, 100).unwrap() else {
        panic!("missing zero-length receive")
    };
    assert!(first.data.is_empty());
    assert_eq!(recv_from(&receiver, 100, 100).unwrap(), Step::Wait);
    assert_eq!(
        send_to(&sender, b"127.0.0.1", port, b"").unwrap(),
        Step::Ready(())
    );
    wait_socket(&receiver, Interest::Read);
    let Step::Ready(empty) = recv_from(&receiver, 100, 100).unwrap() else {
        panic!("missing empty datagram")
    };
    assert!(empty.data.is_empty());
    assert_eq!(empty.port, u32::from(sender.local_addr().unwrap().port()));
}

#[test]
fn receive_budgets_fail_before_consuming_waiting_data() {
    let (client, server) = connected();
    send_all(&client, b"kept");
    assert!(matches!(
        recv(&server, u32::MAX, 100),
        Err(Failure::Limit(_))
    ));
    assert_eq!(receive(&server, 100), b"kept");
}

#[test]
fn readiness_supports_more_than_64_sockets_and_preserves_registration_indices() {
    let sockets: Vec<_> = (0..70).map(|_| bind(0).unwrap()).collect();
    let sender = bind(0).unwrap();
    for index in [1, 35, 69] {
        let port = u32::from(sockets[index].local_addr().unwrap().port());
        send_to(&sender, b"127.0.0.1", port, b"x").unwrap();
        wait_socket(&sockets[index], Interest::Read);
    }
    let registrations: Vec<_> = sockets
        .iter()
        .map(|socket| socket_registration(socket, Interest::Read))
        .collect();
    let ready = poll(&registrations, 0).unwrap();
    assert_eq!(ready.len(), 70);
    for (index, readable) in ready.into_iter().enumerate() {
        assert_eq!(readable, [1, 35, 69].contains(&index));
    }
}

#[test]
fn wake_pair_notifies_drains_and_can_be_shared_with_host_workers() {
    let wake = WakePair::new().unwrap();
    assert_eq!(
        poll(&[socket_registration(&wake.reader, Interest::Read)], 0).unwrap(),
        [false]
    );
    let sender = std::sync::Arc::clone(&wake.sender);
    let notifier = std::thread::spawn(move || {
        sender.notify();
        sender.notify();
    });
    wait_socket(&wake.reader, Interest::Read);
    notifier.join().unwrap();
    wake.drain().unwrap();
    assert_eq!(
        poll(&[socket_registration(&wake.reader, Interest::Read)], 0).unwrap(),
        [false]
    );
    wake.sender.notify();
    wait_socket(&wake.reader, Interest::Read);
}

#[test]
fn duplicate_read_and_write_registrations_keep_their_individual_interests() {
    let (client, server) = connected();
    let registrations = [
        socket_registration(&server, Interest::Read),
        socket_registration(&server, Interest::Write),
        socket_registration(&server, Interest::Read),
    ];
    assert_eq!(poll(&registrations, 0).unwrap(), [false, true, false]);
    send_all(&client, b"x");
    wait_socket(&server, Interest::Read);
    assert_eq!(poll(&registrations, 0).unwrap(), [true, true, true]);
}

#[test]
fn host_tcp_interoperates_with_standard_library_peer_and_small_send_buffer() {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let connection = connect(
        b"127.0.0.1",
        u32::from(listener.local_addr().unwrap().port()),
    )
    .unwrap();
    if connection.pending {
        wait_socket(&connection.socket, Interest::Write);
        finish_connect(&connection.socket).unwrap();
    }
    connection.socket.set_send_buffer_size(1024).unwrap();
    let (mut peer, _address) = listener.accept().unwrap();
    send_all(&connection.socket, b"host");
    let mut received = [0; 4];
    peer.read_exact(&mut received).unwrap();
    assert_eq!(&received, b"host");
    peer.write_all(b"peer").unwrap();
    assert_eq!(receive(&connection.socket, 100), b"peer");
}
