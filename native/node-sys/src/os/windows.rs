// SPDX-License-Identifier: MPL-2.0
use socket2::SockAddr;
use socket2::Socket;
use std::mem;
use std::os::windows::io::AsRawSocket;
use std::os::windows::io::FromRawSocket;
use std::ptr;
use windows_sys::Win32::Networking::WinSock as w;

pub(crate) type Raw = usize;
type HostResult<T> = Result<T, i32>;
pub(crate) fn raw(socket: &Socket) -> Raw {
    socket.as_raw_socket() as Raw
}
pub(crate) fn portable_error(code: i32, connecting: bool) -> i32 {
    if code == w::WSAEWOULDBLOCK {
        if connecting { 115 } else { 11 }
    } else if matches!(code, w::WSAEINPROGRESS | w::WSAEALREADY) {
        115
    } else {
        code
    }
}
fn last(connecting: bool) -> i32 {
    // SAFETY: WSAGetLastError reads thread-local error state.
    portable_error(unsafe { w::WSAGetLastError() }, connecting)
}
fn result(value: i32) -> HostResult<i32> {
    if value == -1 {
        Err(last(false))
    } else {
        Ok(value)
    }
}
pub(crate) fn get_flags(fd: Raw, known: i32) -> HostResult<i32> {
    socket_error_probe(fd).map(|()| known)
}
fn socket_error_probe(fd: Raw) -> HostResult<()> {
    let mut kind = 0i32;
    let mut size = 4;
    // SAFETY: Output pointers refer to live, correctly sized stack storage.
    result(unsafe {
        w::getsockopt(
            fd,
            w::SOL_SOCKET,
            w::SO_TYPE,
            ptr::from_mut(&mut kind).cast(),
            &mut size,
        )
    })
    .map(|_| ())
}
pub(crate) fn set_nonblocking(fd: Raw, value: bool) -> HostResult<()> {
    let mut enabled = u32::from(value);
    // SAFETY: The descriptor is passed to Winsock for validation, and enabled is live.
    result(unsafe { w::ioctlsocket(fd, w::FIONBIO, &mut enabled) }).map(|_| ())
}
pub(crate) fn bind(fd: Raw, address: &SockAddr) -> HostResult<i32> {
    // SAFETY: SockAddr owns the initialized native address for the call.
    result(unsafe { w::bind(fd, address.as_ptr().cast(), address.len()) })
}
pub(crate) fn connect(fd: Raw, address: &SockAddr) -> HostResult<i32> {
    // SAFETY: SockAddr owns the initialized native address for the call.
    let value = unsafe { w::connect(fd, address.as_ptr().cast(), address.len()) };
    if value == -1 {
        Err(last(true))
    } else {
        Ok(value)
    }
}
pub(crate) fn listen(fd: Raw, backlog: i32) -> HostResult<i32> {
    // SAFETY: Winsock validates the raw descriptor and backlog.
    result(unsafe { w::listen(fd, backlog) })
}
pub(crate) fn accept(fd: Raw) -> HostResult<(Socket, [u8; 16])> {
    let mut accepted = w::INVALID_SOCKET;
    // SAFETY: try_init supplies valid native output storage and its length. Only
    // success below turns the fresh descriptor into an owning Rust socket.
    let address = unsafe {
        SockAddr::try_init(|storage, size| {
            accepted = w::accept(fd, storage.cast(), size);
            if accepted == w::INVALID_SOCKET {
                Err(std::io::Error::from_raw_os_error(w::WSAGetLastError()))
            } else {
                Ok(())
            }
        })
    }
    .map_err(|error| super::error_code(&error, false))?
    .1;
    // SAFETY: accept succeeded and returned a fresh socket, with no other owner.
    let socket = unsafe { Socket::from_raw_socket(accepted as _) };
    Ok((socket, super::portable_address(&address)))
}
pub(crate) fn local_address(fd: Raw) -> HostResult<[u8; 16]> {
    // SAFETY: try_init supplies correctly sized native output storage.
    let (_, address) = unsafe {
        SockAddr::try_init(|storage, size| {
            if w::getsockname(fd, storage.cast(), size) == -1 {
                Err(std::io::Error::from_raw_os_error(w::WSAGetLastError()))
            } else {
                Ok(())
            }
        })
    }
    .map_err(|error| super::error_code(&error, false))?;
    Ok(super::portable_address(&address))
}
pub(crate) fn reuse_address(fd: Raw, value: i32) -> HostResult<i32> {
    // SAFETY: The value pointer and length describe initialized stack memory.
    result(unsafe {
        w::setsockopt(
            fd,
            w::SOL_SOCKET,
            w::SO_REUSEADDR,
            ptr::from_ref(&value).cast(),
            4,
        )
    })
}
pub(crate) fn socket_error(fd: Raw) -> HostResult<i32> {
    let mut error = 0i32;
    let mut size = 4;
    // SAFETY: The output pointer and length describe initialized stack memory.
    result(unsafe {
        w::getsockopt(
            fd,
            w::SOL_SOCKET,
            w::SO_ERROR,
            ptr::from_mut(&mut error).cast(),
            &mut size,
        )
    })?;
    Ok(error)
}
pub(crate) fn send(fd: Raw, data: *mut u8, length: usize) -> HostResult<i32> {
    // SAFETY: Caller validates the callback-local view and bounds <=INT_MAX.
    result(unsafe { w::send(fd, data.cast(), length as i32, 0) })
}
pub(crate) fn send_to(
    fd: Raw,
    data: *mut u8,
    length: usize,
    address: &SockAddr,
) -> HostResult<i32> {
    // SAFETY: Caller validates view bounds; SockAddr owns initialized address data.
    result(unsafe {
        w::sendto(
            fd,
            data.cast(),
            length as i32,
            0,
            address.as_ptr().cast(),
            address.len(),
        )
    })
}
pub(crate) fn receive(
    fd: Raw,
    data: *mut u8,
    length: usize,
    with_address: bool,
) -> HostResult<(i32, [u8; 16])> {
    let buffer = w::WSABUF {
        len: length as u32,
        buf: data,
    };
    let mut received = 0;
    let mut flags = 0;
    // SAFETY: Zeroed sockaddr storage is valid as writable native output memory.
    let mut address: w::SOCKADDR_STORAGE = unsafe { mem::zeroed() };
    let mut address_length = mem::size_of_val(&address) as i32;
    // SAFETY: All buffers remain live, bounds were checked, and null overlapped
    // selects a synchronous syscall. No pointers survive its return.
    let value = unsafe {
        if with_address {
            w::WSARecvFrom(
                fd,
                &buffer,
                1,
                &mut received,
                &mut flags,
                ptr::from_mut(&mut address).cast(),
                &mut address_length,
                ptr::null_mut(),
                None,
            )
        } else {
            w::WSARecv(
                fd,
                &buffer,
                1,
                &mut received,
                &mut flags,
                ptr::null_mut(),
                None,
            )
        }
    };
    if value == -1 {
        // SAFETY: Reads the error from the immediately preceding Winsock call.
        let code = unsafe { w::WSAGetLastError() };
        if code != w::WSAEMSGSIZE {
            return Err(portable_error(code, false));
        }
        // Winsock reports copied bytes and the sender even on datagram truncation.
    }
    let mut portable = [0; 16];
    portable[0] = 2;
    if address.ss_family == w::AF_INET {
        // SAFETY: A returned AF_INET sockaddr contains a complete SOCKADDR_IN.
        let address = unsafe { &*ptr::from_ref(&address).cast::<w::SOCKADDR_IN>() };
        portable[2..4].copy_from_slice(&address.sin_port.to_ne_bytes());
        // SAFETY: IN_ADDR is an initialized union returned by Winsock; S_addr
        // views the same four bytes as its octet representation.
        portable[4..8].copy_from_slice(&unsafe { address.sin_addr.S_un.S_addr }.to_ne_bytes());
    }
    Ok((received.min(length as u32) as i32, portable))
}
pub(crate) fn close(fd: Raw) -> HostResult<i32> {
    // SAFETY: Raw close explicitly transfers responsibility for a foreign socket.
    result(unsafe { w::closesocket(fd) })
}
pub(crate) fn poll(registrations: &[(Raw, i16)], timeout: i32) -> HostResult<Vec<u16>> {
    if registrations.is_empty() {
        if timeout == -1 {
            loop {
                std::thread::park();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(timeout as u64));
        return Ok(Vec::new());
    }
    let mut descriptors = registrations
        .iter()
        .map(|(fd, events)| w::WSAPOLLFD {
            fd: *fd,
            events: (if events & 1 != 0 { w::POLLRDNORM } else { 0 })
                | (if events & 4 != 0 { w::POLLWRNORM } else { 0 }),
            revents: 0,
        })
        .collect::<Vec<_>>();
    // SAFETY: The contiguous array and count match, timeout has been validated.
    let outcome =
        result(unsafe { w::WSAPoll(descriptors.as_mut_ptr(), descriptors.len() as u32, timeout) });
    if outcome == Err(w::WSAENOTSOCK) {
        // Winsock rejects a list containing only invalid sockets. POSIX poll
        // returns POLLNVAL in those positions. Probe only on this error path.
        let mut invalid = vec![false; registrations.len()];
        let mut valid = Vec::new();
        let mut positions = Vec::new();
        for (index, registration) in registrations.iter().enumerate() {
            if socket_error_probe(registration.0) == Err(w::WSAENOTSOCK) {
                invalid[index] = true;
            } else {
                valid.push(*registration);
                positions.push(index);
            }
        }
        if !invalid.iter().any(|value| *value) {
            return Err(w::WSAENOTSOCK);
        }
        let mut events = invalid
            .into_iter()
            .map(|invalid| if invalid { 32 } else { 0 })
            .collect::<Vec<_>>();
        if !valid.is_empty() {
            for (index, ready) in positions.into_iter().zip(poll(&valid, 0)?) {
                events[index] = ready;
            }
        }
        return Ok(events);
    }
    outcome?;
    Ok(descriptors
        .into_iter()
        .map(|entry| {
            let native = entry.revents;
            (if native & (w::POLLRDNORM | w::POLLRDBAND) != 0 {
                1
            } else {
                0
            }) | (if native & w::POLLWRNORM != 0 { 4 } else { 0 })
                | (if native & w::POLLERR != 0 { 8 } else { 0 })
                | (if native & w::POLLHUP != 0 { 16 } else { 0 })
                | (if native & w::POLLNVAL != 0 { 32 } else { 0 })
        })
        .collect())
}
