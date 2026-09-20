// SPDX-License-Identifier: MPL-2.0
//! Raw syscalls accept untrusted descriptor values without creating Rust borrows.
//! Only successfully created sockets enter socket2's owning type.

#[cfg(windows)]
#[path = "os/windows.rs"]
mod platform;
#[cfg(unix)]
#[path = "os/unix.rs"]
mod platform;
pub(super) use platform::*;

pub(super) fn error_code(error: &std::io::Error, connecting: bool) -> i32 {
    portable_error(error.raw_os_error().unwrap_or(22), connecting)
}

pub(super) fn portable_address(address: &socket2::SockAddr) -> [u8; 16] {
    let mut bytes = [0; 16];
    bytes[0] = 2;
    if let Some(address) = address.as_socket_ipv4() {
        bytes[2..4].copy_from_slice(&address.port().to_be_bytes());
        bytes[4..8].copy_from_slice(&address.ip().octets());
    }
    bytes
}
