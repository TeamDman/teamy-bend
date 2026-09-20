// SPDX-License-Identifier: Apache-2.0
//! Native networking adapted from Bend 2.0.5 `effs/tcp_*.c`, `effs/udp_*.c`
//! and `comp.ts`. Copyright 2026 `HigherOrderCO`.
//! Rust ownership, bounds and Windows adaptation: `TeamDman`.
//! See NOTICE and licenses/Apache-2.0.txt.
//!
//! Every socket remains nonblocking. This layer never parks a VM or starts a
//! worker: its caller owns continuation ordering and readiness registrations.
//! Windows returns native Winsock errors; Unix retains errno and strerror.

use super::host_files::Error;
use super::host_files::Failure;
use socket2::Domain;
use socket2::MaybeUninitSlice;
use socket2::Protocol;
use socket2::SockAddr;
use socket2::Type;
use std::io;
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;

mod readiness;
pub(super) use readiness::Interest;
pub(super) use readiness::Registration;
pub(super) use readiness::Source;
pub(super) use readiness::WakePair;
pub(super) use readiness::WakeSender;
pub(super) use readiness::poll;

type HostResult<T> = Result<T, Failure>;

/// socket2 owns each descriptor/SOCKET and closes it exactly once.
#[derive(Debug)]
pub(super) struct NativeSocket(socket2::Socket);

#[derive(Debug)]
pub(super) struct NativeListener(socket2::Socket);

#[cfg(test)]
impl NativeSocket {
    pub(super) fn local_addr(&self) -> HostResult<SocketAddrV4> {
        self.0
            .local_addr()
            .map_err(host_error)?
            .as_socket_ipv4()
            .ok_or(Failure::Limit("native socket has no IPv4 address"))
    }

    pub(super) fn set_send_buffer_size(&self, size: usize) -> HostResult<()> {
        self.0.set_send_buffer_size(size).map_err(host_error)
    }

    pub(super) fn set_recv_buffer_size(&self, size: usize) -> HostResult<()> {
        self.0.set_recv_buffer_size(size).map_err(host_error)
    }
}

#[cfg(test)]
impl NativeListener {
    pub(super) fn local_addr(&self) -> HostResult<SocketAddrV4> {
        self.0
            .local_addr()
            .map_err(host_error)?
            .as_socket_ipv4()
            .ok_or(Failure::Limit("native listener has no IPv4 address"))
    }

    pub(super) fn set_recv_buffer_size(&self, size: usize) -> HostResult<()> {
        self.0.set_recv_buffer_size(size).map_err(host_error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Step<T> {
    Ready(T),
    Wait,
}

#[derive(Debug)]
pub(super) struct Connect {
    pub(super) socket: NativeSocket,
    pub(super) pending: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Datagram {
    pub(super) host: Vec<u8>,
    pub(super) port: u32,
    pub(super) data: Vec<u8>,
}

pub(super) fn listen(port: u32) -> HostResult<NativeListener> {
    let socket = socket2::Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .map_err(host_error)?;
    // Upstream attempts reuse but does not treat its failure as listen failure.
    let _reuse = socket.set_reuse_address(true);
    let address = address(b"0.0.0.0", port)?;
    socket.bind(&address).map_err(host_error)?;
    socket.listen(16).map_err(host_error)?;
    socket.set_nonblocking(true).map_err(host_error)?;
    Ok(NativeListener(socket))
}

pub(super) fn accept(listener: &NativeListener) -> HostResult<Step<NativeSocket>> {
    match listener.0.accept() {
        Ok((socket, _peer)) => {
            socket.set_nonblocking(true).map_err(host_error)?;
            Ok(Step::Ready(NativeSocket(socket)))
        }
        Err(error) if would_block(&error) => Ok(Step::Wait),
        Err(error) => Err(host_error(error)),
    }
}

pub(super) fn connect(host: &[u8], port: u32) -> HostResult<Connect> {
    let address = address(host, port)?;
    let socket = socket2::Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .map_err(host_error)?;
    socket.set_nonblocking(true).map_err(host_error)?;
    let pending = match socket.connect(&address) {
        Ok(()) => false,
        Err(error) if connect_pending(&error) => true,
        Err(error) => return Err(host_error(error)),
    };
    Ok(Connect {
        socket: NativeSocket(socket),
        pending,
    })
}

/// Call only after the connecting socket becomes writable/error-ready.
pub(super) fn finish_connect(socket: &NativeSocket) -> HostResult<()> {
    match socket.0.take_error().map_err(host_error)? {
        None => Ok(()),
        Some(error) => Err(host_error(error)),
    }
}

pub(super) fn bind(port: u32) -> HostResult<NativeSocket> {
    let socket =
        socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).map_err(host_error)?;
    socket
        .bind(&address(b"0.0.0.0", port)?)
        .map_err(host_error)?;
    socket.set_nonblocking(true).map_err(host_error)?;
    Ok(NativeSocket(socket))
}

/// One send: the driver retains the unsent suffix across readiness waits.
pub(super) fn send(socket: &NativeSocket, bytes: &[u8]) -> HostResult<Step<usize>> {
    if bytes.is_empty() {
        return Ok(Step::Ready(0));
    }
    match socket.0.send_with_flags(bytes, send_flags()) {
        Ok(0) => Err(Failure::Limit("native TCP send made no progress")),
        Ok(count) => Ok(Step::Ready(count)),
        Err(error) if would_block(&error) => Ok(Step::Wait),
        Err(error) => Err(host_error(error)),
    }
}

pub(super) fn recv(
    socket: &NativeSocket,
    requested: u32,
    byte_limit: usize,
) -> HostResult<Step<Vec<u8>>> {
    let mut bytes = receive_buffer(requested, byte_limit)?;
    let mut slices = [MaybeUninitSlice::new(as_uninit(&mut bytes))];
    match socket.0.recv_vectored(&mut slices) {
        Ok((count, _flags)) => {
            bytes.truncate(count);
            Ok(Step::Ready(bytes))
        }
        Err(error) if would_block(&error) => Ok(Step::Wait),
        Err(error) => Err(host_error(error)),
    }
}

pub(super) fn send_to(
    socket: &NativeSocket,
    host: &[u8],
    port: u32,
    bytes: &[u8],
) -> HostResult<Step<()>> {
    let address = address(host, port)?;
    match socket.0.send_to_with_flags(bytes, &address, send_flags()) {
        Ok(_count) => Ok(Step::Ready(())),
        Err(error) if would_block(&error) => Ok(Step::Wait),
        Err(error) => Err(host_error(error)),
    }
}

/// `UDP.poll` invokes this once and maps Wait to None; `UDP.recv_from` parks.
pub(super) fn recv_from(
    socket: &NativeSocket,
    requested: u32,
    byte_limit: usize,
) -> HostResult<Step<Datagram>> {
    let mut bytes = receive_buffer(requested, byte_limit)?;
    let mut slices = [MaybeUninitSlice::new(as_uninit(&mut bytes))];
    // On Windows socket2's WSARecvFrom adapter preserves WSAEMSGSIZE as a
    // successful truncated datagram, with its sender address and byte count.
    match socket.0.recv_from_vectored(&mut slices) {
        Ok((count, _flags, sender)) => {
            bytes.truncate(count);
            // recvfrom on a stream may leave sockaddr empty. Upstream starts
            // its sockaddr at zero and reports 0.0.0.0:0 in that case.
            let peer = sender
                .as_socket_ipv4()
                .unwrap_or_else(|| SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
            Ok(Step::Ready(Datagram {
                host: peer.ip().to_string().into_bytes(),
                port: u32::from(peer.port()),
                data: bytes,
            }))
        }
        Err(error) if would_block(&error) => Ok(Step::Wait),
        Err(error) => Err(host_error(error)),
    }
}

fn address(host: &[u8], port: u32) -> HostResult<SockAddr> {
    let port = u16::try_from(port).map_err(|_error| invalid_address())?;
    let mut octets = [0; 4];
    let mut parts = host.split(|byte| *byte == b'.');
    for octet in &mut octets {
        let component = parts.next().ok_or_else(invalid_address)?;
        if component.is_empty()
            || component.len() > 3
            || component.len() > 1 && component[0] == b'0'
        {
            return Err(invalid_address());
        }
        let mut number = 0_u16;
        for digit in component {
            if !digit.is_ascii_digit() {
                return Err(invalid_address());
            }
            number = number * 10 + u16::from(digit - b'0');
        }
        *octet = u8::try_from(number).map_err(|_error| invalid_address())?;
    }
    if parts.next().is_some() {
        return Err(invalid_address());
    }
    Ok(SocketAddrV4::new(Ipv4Addr::from(octets), port).into())
}

fn receive_buffer(requested: u32, byte_limit: usize) -> HostResult<Vec<u8>> {
    let count = usize::try_from(requested.min(i32::MAX.cast_unsigned()))
        .map_err(|_error| Failure::Limit("native socket receive size exceeds host range"))?;
    if count > byte_limit {
        return Err(Failure::Limit("native socket receive exceeds byte budget"));
    }
    let mut bytes = Vec::new();
    // Keep a valid backing byte even for a zero-length host receive buffer.
    bytes
        .try_reserve_exact(count.max(1))
        .map_err(|_error| Failure::Limit("native socket receive allocation failed"))?;
    bytes.resize(count, 0);
    Ok(bytes)
}

fn as_uninit(bytes: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    // SAFETY: u8 and MaybeUninit<u8> have identical layout. The receiving
    // socket2 methods promise never to write uninitialized bytes into a buffer,
    // so every byte stays initialized when this exclusive borrow ends.
    unsafe { std::slice::from_raw_parts_mut(bytes.as_mut_ptr().cast(), bytes.len()) }
}

fn would_block(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock
}

fn connect_pending(error: &io::Error) -> bool {
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(libc::EINPROGRESS)
    }
    #[cfg(windows)]
    {
        matches!(error.raw_os_error(), Some(10035 | 10036))
    }
}

fn send_flags() -> i32 {
    // Upstream ignores SIGPIPE process-wide. Suppress it per socket/send here
    // instead: socket2 sets SO_NOSIGPIPE on Apple; other Unix uses MSG_NOSIGNAL.
    #[cfg(all(unix, not(target_vendor = "apple")))]
    {
        libc::MSG_NOSIGNAL
    }
    #[cfg(any(windows, target_vendor = "apple"))]
    {
        0
    }
}

fn invalid_address() -> Failure {
    #[cfg(unix)]
    {
        host_error(io::Error::from_raw_os_error(libc::EINVAL))
    }
    #[cfg(windows)]
    {
        Failure::Io(Error {
            code: 22,
            message: b"Invalid argument".to_vec(),
        })
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "This converter is passed directly to Result::map_err, which supplies owned errors"
)]
fn host_error(error: io::Error) -> Failure {
    #[cfg(unix)]
    {
        let code = error.raw_os_error().unwrap_or(libc::EIO);
        let mut buffer = [std::ffi::c_char::default(); 1024];
        // SAFETY: strerror_r receives an initialized writable buffer of its
        // exact stated size; its successful result is NUL-terminated.
        let result = unsafe { libc::strerror_r(code, buffer.as_mut_ptr(), buffer.len()) };
        let message = if result == 0 {
            // SAFETY: the successful strerror_r call NUL-terminated the buffer.
            unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
                .to_bytes()
                .to_vec()
        } else {
            format!("OS error {code}").into_bytes()
        };
        Failure::Io(Error {
            code: code.cast_unsigned(),
            message,
        })
    }
    #[cfg(windows)]
    {
        Failure::Io(Error {
            code: error.raw_os_error().map_or(10022, i32::cast_unsigned),
            message: error.to_string().into_bytes(),
        })
    }
}

#[cfg(test)]
#[path = "host_network/tests.rs"]
mod tests;
