// SPDX-License-Identifier: MPL-2.0
//! Borrowed readiness registrations and a socket wake source for host jobs.

use super::HostResult;
use super::NativeListener;
use super::NativeSocket;
use super::Step;
use super::host_error;
use crate::runtime::ARENA_LIMIT;
use crate::runtime::host_files::Failure;
use socket2::Domain;
use socket2::Protocol;
use socket2::Type;
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) enum Source<'a> {
    Socket(&'a NativeSocket),
    Listener(&'a NativeListener),
}

impl<'a> Source<'a> {
    fn socket(self) -> &'a socket2::Socket {
        match self {
            Self::Socket(socket) => &socket.0,
            Self::Listener(listener) => &listener.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum Interest {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct Registration<'a> {
    pub(in crate::runtime) source: Source<'a>,
    pub(in crate::runtime) interest: Interest,
}

pub(in crate::runtime) fn poll(
    registrations: &[Registration<'_>],
    timeout_millis: u32,
) -> HostResult<Vec<bool>> {
    if registrations.len() > ARENA_LIMIT + 1 {
        return Err(Failure::Limit("native socket readiness budget exhausted"));
    }
    if registrations.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(u64::from(timeout_millis)));
        return Ok(Vec::new());
    }
    platform_poll(
        registrations,
        i32::try_from(timeout_millis).unwrap_or(i32::MAX),
    )
}

#[cfg(windows)]
fn platform_poll(registrations: &[Registration<'_>], timeout: i32) -> HostResult<Vec<bool>> {
    use std::os::windows::io::AsRawSocket;
    use windows::Win32::Networking::WinSock;

    let mut rows = Vec::new();
    rows.try_reserve_exact(registrations.len())
        .map_err(|_error| Failure::Limit("native socket poll allocation failed"))?;
    for registration in registrations {
        let raw = registration.source.socket().as_raw_socket();
        rows.push(WinSock::WSAPOLLFD {
            fd: WinSock::SOCKET(
                usize::try_from(raw).map_err(|_error| {
                    Failure::Limit("native socket identity exceeds host range")
                })?,
            ),
            events: match registration.interest {
                Interest::Read => WinSock::POLLRDNORM,
                Interest::Write => WinSock::POLLWRNORM,
            },
            revents: WinSock::WSAPOLL_EVENT_FLAGS(0),
        });
    }
    let count = u32::try_from(rows.len())
        .map_err(|_error| Failure::Limit("native socket poll size exceeds host range"))?;
    // SAFETY: every row borrows a live socket, and the writable array has count
    // initialized entries. Sockets cannot be dropped during the borrowed call.
    let result = unsafe { WinSock::WSAPoll(rows.as_mut_ptr(), count, timeout) };
    if result < 0 {
        // SAFETY: WSAGetLastError has no pointer preconditions and is read on
        // this same thread immediately after the failed Winsock call.
        let code = unsafe { WinSock::WSAGetLastError() }.0;
        if code == WinSock::WSAEINTR.0 {
            return Ok(vec![false; rows.len()]);
        }
        return Err(host_error(io::Error::from_raw_os_error(code)));
    }
    Ok(rows.into_iter().map(|row| row.revents.0 != 0).collect())
}

#[cfg(unix)]
fn platform_poll(registrations: &[Registration<'_>], timeout: i32) -> HostResult<Vec<bool>> {
    use std::os::fd::AsRawFd;
    let mut rows = Vec::new();
    rows.try_reserve_exact(registrations.len())
        .map_err(|_error| Failure::Limit("native socket poll allocation failed"))?;
    for registration in registrations {
        rows.push(libc::pollfd {
            fd: registration.source.socket().as_raw_fd(),
            events: match registration.interest {
                Interest::Read => libc::POLLIN,
                Interest::Write => libc::POLLOUT,
            },
            revents: 0,
        });
    }
    let count = libc::nfds_t::try_from(rows.len())
        .map_err(|_error| Failure::Limit("native socket poll size exceeds host range"))?;
    // SAFETY: every row borrows a live descriptor, and rows contains exactly
    // count initialized, writable pollfd entries for the duration of the call.
    let result = unsafe { libc::poll(rows.as_mut_ptr(), count, timeout) };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EINTR) {
            return Ok(vec![false; rows.len()]);
        }
        return Err(host_error(error));
    }
    Ok(rows.into_iter().map(|row| row.revents != 0).collect())
}

#[derive(Debug)]
pub(in crate::runtime) struct WakePair {
    pub(in crate::runtime) reader: NativeSocket,
    pub(in crate::runtime) sender: Arc<WakeSender>,
}

#[derive(Debug)]
pub(in crate::runtime) struct WakeSender(socket2::Socket);

impl WakePair {
    pub(in crate::runtime) fn new() -> HostResult<Self> {
        let reader = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(host_error)?;
        reader
            .bind(&SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into())
            .map_err(host_error)?;
        reader.set_nonblocking(true).map_err(host_error)?;
        let sender = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(host_error)?;
        sender
            .bind(&SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into())
            .map_err(host_error)?;
        sender
            .connect(&reader.local_addr().map_err(host_error)?)
            .map_err(host_error)?;
        reader
            .connect(&sender.local_addr().map_err(host_error)?)
            .map_err(host_error)?;
        sender.set_nonblocking(true).map_err(host_error)?;
        Ok(Self {
            reader: NativeSocket(reader),
            sender: Arc::new(WakeSender(sender)),
        })
    }

    pub(in crate::runtime) fn drain(&self) -> HostResult<()> {
        while let Step::Ready(_data) = super::recv(&self.reader, 1, 1)? {}
        Ok(())
    }
}

impl WakeSender {
    pub(in crate::runtime) fn notify(&self) {
        // A full datagram buffer already has a readable wake byte. On teardown
        // the receiver may be gone; either case may safely discard this notice.
        let _notification = self.0.send(&[1]);
    }
}
