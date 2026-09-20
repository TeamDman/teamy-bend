// SPDX-License-Identifier: MPL-2.0
use socket2::SockAddr;
use socket2::Socket;
use std::mem;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::ptr;

pub(crate) type Raw = i32;
type HostResult<T> = Result<T, i32>;
pub(crate) fn raw(socket: &Socket) -> Raw {
    socket.as_raw_fd()
}
pub(crate) fn portable_error(code: i32, connecting: bool) -> i32 {
    if code == libc::EAGAIN || code == libc::EWOULDBLOCK {
        if connecting { 115 } else { 11 }
    } else if code == libc::EINPROGRESS || code == libc::EALREADY {
        115
    } else {
        code
    }
}
fn last(connecting: bool) -> i32 {
    super::error_code(&std::io::Error::last_os_error(), connecting)
}
fn result(value: i32) -> HostResult<i32> {
    if value == -1 {
        Err(last(false))
    } else {
        Ok(value)
    }
}
pub(crate) fn get_flags(fd: Raw, _known: i32) -> HostResult<i32> {
    // SAFETY: The kernel validates the raw descriptor.
    let flags = result(unsafe { libc::fcntl(fd, libc::F_GETFL) })?;
    Ok(if flags & libc::O_NONBLOCK != 0 {
        0x800
    } else {
        0
    })
}
pub(crate) fn set_nonblocking(fd: Raw, value: bool) -> HostResult<()> {
    // SAFETY: The kernel validates the raw descriptor and flags.
    let flags = result(unsafe { libc::fcntl(fd, libc::F_GETFL) })?;
    let flags = if value {
        flags | libc::O_NONBLOCK
    } else {
        flags & !libc::O_NONBLOCK
    };
    // SAFETY: The descriptor and integer flags are passed directly to the kernel.
    result(unsafe { libc::fcntl(fd, libc::F_SETFL, flags) }).map(|_| ())
}
pub(crate) fn bind(fd: Raw, address: &SockAddr) -> HostResult<i32> {
    // SAFETY: SockAddr owns the initialized native address for the call.
    result(unsafe { libc::bind(fd, address.as_ptr(), address.len()) })
}
pub(crate) fn connect(fd: Raw, address: &SockAddr) -> HostResult<i32> {
    // SAFETY: SockAddr owns the initialized native address for the call.
    let value = unsafe { libc::connect(fd, address.as_ptr(), address.len()) };
    if value == -1 {
        Err(last(true))
    } else {
        Ok(value)
    }
}
pub(crate) fn listen(fd: Raw, backlog: i32) -> HostResult<i32> {
    // SAFETY: The kernel validates the descriptor and backlog.
    result(unsafe { libc::listen(fd, backlog) })
}
pub(crate) fn accept(fd: Raw) -> HostResult<(Socket, [u8; 16])> {
    let mut accepted = -1;
    // SAFETY: try_init supplies valid native output storage and length.
    let (_, address) = unsafe {
        SockAddr::try_init(|storage, size| {
            accepted = libc::accept(fd, storage.cast(), size);
            if accepted == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        })
    }
    .map_err(|error| super::error_code(&error, false))?;
    // SAFETY: accept succeeded and returned a fresh descriptor with no owner.
    let socket = unsafe { Socket::from_raw_fd(accepted) };
    Ok((socket, super::portable_address(&address)))
}
pub(crate) fn local_address(fd: Raw) -> HostResult<[u8; 16]> {
    // SAFETY: try_init supplies correctly sized native output storage.
    let (_, address) = unsafe {
        SockAddr::try_init(|storage, size| {
            if libc::getsockname(fd, storage.cast(), size) == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        })
    }
    .map_err(|error| super::error_code(&error, false))?;
    Ok(super::portable_address(&address))
}
pub(crate) fn reuse_address(fd: Raw, value: i32) -> HostResult<i32> {
    // SAFETY: Value and byte size refer to live stack storage.
    result(unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            ptr::from_ref(&value).cast(),
            4,
        )
    })
}
pub(crate) fn socket_error(fd: Raw) -> HostResult<i32> {
    let mut error = 0i32;
    let mut size: libc::socklen_t = 4;
    // SAFETY: Output and length pointers refer to live stack storage.
    result(unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_ERROR,
            ptr::from_mut(&mut error).cast(),
            &mut size,
        )
    })?;
    Ok(error)
}
fn send_flags(fd: Raw) -> HostResult<i32> {
    #[cfg(target_vendor = "apple")]
    {
        let value = 1i32;
        // SAFETY: Value and byte size refer to live stack storage. Suppress
        // process-wide SIGPIPE without changing the host's signal disposition.
        result(unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_NOSIGPIPE,
                ptr::from_ref(&value).cast(),
                4,
            )
        })?;
        Ok(0)
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        let _ = fd;
        Ok(libc::MSG_NOSIGNAL)
    }
}
pub(crate) fn send(fd: Raw, data: *mut u8, length: usize) -> HostResult<i32> {
    let flags = send_flags(fd)?;
    // SAFETY: Caller validated the live callback-local view and byte length.
    let count = unsafe { libc::send(fd, data.cast(), length, flags) };
    result(count as i32)
}
pub(crate) fn send_to(
    fd: Raw,
    data: *mut u8,
    length: usize,
    address: &SockAddr,
) -> HostResult<i32> {
    let flags = send_flags(fd)?;
    // SAFETY: Caller validated the view; SockAddr owns its initialized storage.
    let count = unsafe {
        libc::sendto(
            fd,
            data.cast(),
            length,
            flags,
            address.as_ptr(),
            address.len(),
        )
    };
    result(count as i32)
}
pub(crate) fn receive(
    fd: Raw,
    data: *mut u8,
    length: usize,
    with_address: bool,
) -> HostResult<(i32, [u8; 16])> {
    // SAFETY: Zeroed sockaddr storage is valid as writable native output memory.
    let mut address: libc::sockaddr_storage = unsafe { mem::zeroed() };
    let mut size = mem::size_of_val(&address) as libc::socklen_t;
    // SAFETY: Caller validated the view; output address and length remain live.
    let count = unsafe {
        if with_address {
            libc::recvfrom(
                fd,
                data.cast(),
                length,
                0,
                ptr::from_mut(&mut address).cast(),
                &mut size,
            )
        } else {
            libc::recv(fd, data.cast(), length, 0)
        }
    };
    let count = result(count as i32)?;
    let mut portable = [0; 16];
    portable[0] = 2;
    if i32::from(address.ss_family) == libc::AF_INET {
        // SAFETY: A returned AF_INET sockaddr contains a complete sockaddr_in.
        let address = unsafe { &*ptr::from_ref(&address).cast::<libc::sockaddr_in>() };
        portable[2..4].copy_from_slice(&address.sin_port.to_ne_bytes());
        portable[4..8].copy_from_slice(&address.sin_addr.s_addr.to_ne_bytes());
    }
    Ok((count, portable))
}
pub(crate) fn close(fd: Raw) -> HostResult<i32> {
    // SAFETY: Calling close explicitly releases an externally supplied descriptor.
    result(unsafe { libc::close(fd) })
}
pub(crate) fn poll(registrations: &[(Raw, i16)], timeout: i32) -> HostResult<Vec<u16>> {
    let mut descriptors = registrations
        .iter()
        .map(|(fd, events)| libc::pollfd {
            fd: *fd,
            events: (if events & 1 != 0 { libc::POLLIN } else { 0 })
                | (if events & 4 != 0 { libc::POLLOUT } else { 0 }),
            revents: 0,
        })
        .collect::<Vec<_>>();
    // SAFETY: The contiguous array/count match and timeout was validated.
    result(unsafe {
        libc::poll(
            descriptors.as_mut_ptr(),
            descriptors.len() as libc::nfds_t,
            timeout,
        )
    })?;
    Ok(descriptors
        .into_iter()
        .map(|entry| {
            let native = entry.revents;
            (if native & (libc::POLLIN | libc::POLLPRI) != 0 {
                1
            } else {
                0
            }) | (if native & libc::POLLOUT != 0 { 4 } else { 0 })
                | (if native & libc::POLLERR != 0 { 8 } else { 0 })
                | (if native & libc::POLLHUP != 0 { 16 } else { 0 })
                | (if native & libc::POLLNVAL != 0 { 32 } else { 0 })
        })
        .collect())
}
