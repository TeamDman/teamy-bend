// SPDX-License-Identifier: MPL-2.0
//! Synchronous, real-descriptor Node-API adapter for Bend's portable syscall ABI.
//! No descriptor is narrowed, borrowed as a Rust socket, or replaced with an ID.

mod memory;
mod os;

use memory::View;
use napi::Env;
use napi::Result;
use napi::bindgen_prelude::Array;
use napi::bindgen_prelude::BigInt;
use napi::bindgen_prelude::Either;
use napi::bindgen_prelude::Unknown;
use napi_derive::napi;
use socket2::Domain;
use socket2::Protocol;
use socket2::SockAddr;
use socket2::Socket;
use socket2::Type;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;

type Descriptor = Either<f64, BigInt>;
const EINVAL: i32 = 22;
const MAX_REGISTRATIONS: usize = 131_072;

/// Owned sockets stay alive with the provider. Externally supplied raw sockets
/// are operated on directly, without manufacturing a Rust ownership guarantee.
#[napi]
pub struct Sys {
    sockets: HashMap<os::Raw, Socket>,
    flags: HashMap<os::Raw, i32>,
    last_error: i32,
}

#[napi(js_name = "create_sys")]
pub fn create_sys() -> Sys {
    Sys {
        sockets: HashMap::new(),
        flags: HashMap::new(),
        last_error: 0,
    }
}

#[napi(object)]
pub struct PollDescriptor {
    pub fd: Descriptor,
    pub events: f64,
}

#[napi]
impl Sys {
    #[napi(getter)]
    pub fn mac(&self) -> bool {
        false
    }

    #[napi]
    pub fn errno(&self) -> i32 {
        self.last_error
    }

    #[napi]
    pub fn strerror(&self, code: i32) -> String {
        match code {
            11 => "Resource temporarily unavailable".into(),
            22 => "Invalid argument".into(),
            115 => "Operation now in progress".into(),
            _ => std::io::Error::from_raw_os_error(code).to_string(),
        }
    }

    #[napi]
    pub fn socket(&mut self, domain: i32, kind: i32, protocol: i32) -> Descriptor {
        if self.flags.len() >= MAX_REGISTRATIONS {
            return self.fd_error(24);
        }
        if domain != 2 || !matches!(kind, 1 | 2) || !matches!(protocol, 0 | 6 | 17) {
            return self.fd_error(EINVAL);
        }
        let protocol = (protocol != 0).then(|| Protocol::from(protocol));
        match Socket::new(Domain::IPV4, Type::from(kind), protocol) {
            Ok(socket) => self.retain(socket),
            Err(error) => self.fd_error(os::error_code(&error, false)),
        }
    }

    #[napi]
    pub fn fcntl(&mut self, descriptor: Descriptor, command: i32, value: i32) -> i32 {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return self.error(EINVAL);
        };
        if command == 4 && !self.flags.contains_key(&fd) && self.flags.len() >= MAX_REGISTRATIONS {
            return self.error(24);
        }
        match command {
            3 => match os::get_flags(fd, self.flags.get(&fd).copied().unwrap_or(0)) {
                Ok(flags) => flags,
                Err(code) => self.error(code),
            },
            4 if value & !0x800 == 0 => match os::set_nonblocking(fd, value & 0x800 != 0) {
                Ok(()) => {
                    self.flags.insert(fd, value);
                    0
                }
                Err(code) => self.error(code),
            },
            _ => self.error(EINVAL),
        }
    }

    #[napi]
    pub fn bind(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        address: Unknown<'_>,
        length: f64,
    ) -> Result<i32> {
        self.address_call(env, descriptor, address, length, false)
    }

    #[napi]
    pub fn connect(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        address: Unknown<'_>,
        length: f64,
    ) -> Result<i32> {
        self.address_call(env, descriptor, address, length, true)
    }

    #[napi]
    pub fn listen(&mut self, descriptor: Descriptor, backlog: i32) -> i32 {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return self.error(EINVAL);
        };
        self.finish(os::listen(fd, backlog))
    }

    #[napi]
    pub fn accept(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        address: Unknown<'_>,
        length: Unknown<'_>,
    ) -> Result<Descriptor> {
        if self.flags.len() >= MAX_REGISTRATIONS {
            return Ok(self.fd_error(24));
        }
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.fd_error(EINVAL));
        };
        let destination = address_destination(env, address, length)?;
        let accepted = os::accept(fd);
        match accepted {
            Ok((socket, peer)) => {
                if let Some((address, length)) = destination {
                    write_address(address, length, &peer);
                }
                Ok(self.retain(socket))
            }
            Err(code) => Ok(self.fd_error(code)),
        }
    }

    #[napi]
    pub fn getsockname(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        address: Unknown<'_>,
        length: Unknown<'_>,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let Some((address, length)) = address_destination(env, address, length)? else {
            return Ok(self.error(EINVAL));
        };
        match os::local_address(fd) {
            Ok(peer) => {
                write_address(address, length, &peer);
                Ok(0)
            }
            Err(code) => Ok(self.error(code)),
        }
    }

    #[napi]
    pub fn setsockopt(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        level: i32,
        option: i32,
        value: Unknown<'_>,
        length: f64,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let bytes = View::new(env, value)?;
        if length != 4.0 || bytes.len() < 4 || level != 1 || option != 2 {
            return Ok(self.error(EINVAL));
        }
        Ok(self.finish(os::reuse_address(fd, bytes.read_i32(0))))
    }

    #[napi]
    pub fn getsockopt(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        level: i32,
        option: i32,
        value: Unknown<'_>,
        length: Unknown<'_>,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let bytes = View::new(env, value)?;
        let size = View::new(env, length)?;
        if bytes.len() < 4 || size.len() < 4 || size.read_u32(0) < 4 || level != 1 || option != 4 {
            return Ok(self.error(EINVAL));
        }
        match os::socket_error(fd) {
            Ok(code) => {
                bytes.write_i32(0, os::portable_error(code, false));
                size.write_u32(0, 4);
                Ok(0)
            }
            Err(code) => Ok(self.error(code)),
        }
    }

    #[napi]
    pub fn send(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        data: Unknown<'_>,
        length: f64,
        flags: i32,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let data = View::new(env, data)?;
        if !data.valid_length(length) || flags != 0 {
            return Ok(self.error(EINVAL));
        }
        Ok(self.finish(os::send(fd, data.pointer(), length as usize)))
    }

    #[napi]
    pub fn recv(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        data: Unknown<'_>,
        length: f64,
        flags: i32,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let data = View::new(env, data)?;
        if !data.valid_length(length) || flags != 0 {
            return Ok(self.error(EINVAL));
        }
        Ok(self.finish(
            os::receive(fd, data.pointer(), length as usize, false).map(|(count, _)| count),
        ))
    }

    #[napi]
    #[allow(
        clippy::too_many_arguments,
        reason = "The syscall ABI requires these arguments; napi copies this attribute to smaller generated functions, so expect cannot be used"
    )]
    pub fn sendto(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        data: Unknown<'_>,
        length: f64,
        flags: i32,
        address: Unknown<'_>,
        address_length: f64,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let data = View::new(env, data)?;
        let address = View::new(env, address)?;
        let Some(address) = parse_address(address, address_length) else {
            return Ok(self.error(EINVAL));
        };
        if !data.valid_length(length) || flags != 0 {
            return Ok(self.error(EINVAL));
        }
        Ok(self.finish(os::send_to(fd, data.pointer(), length as usize, &address)))
    }

    #[napi]
    #[allow(
        clippy::too_many_arguments,
        reason = "The syscall ABI requires these arguments; napi copies this attribute to smaller generated functions, so expect cannot be used"
    )]
    pub fn recvfrom(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        data: Unknown<'_>,
        length: f64,
        flags: i32,
        address: Unknown<'_>,
        address_length: Unknown<'_>,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let data = View::new(env, data)?;
        let destination = address_destination(env, address, address_length)?;
        if !data.valid_length(length) || flags != 0 {
            return Ok(self.error(EINVAL));
        }
        match os::receive(fd, data.pointer(), length as usize, true) {
            Ok((count, peer)) => {
                if let Some((address, length)) = destination {
                    write_address(address, length, &peer);
                }
                Ok(count)
            }
            Err(code) => Ok(self.error(code)),
        }
    }

    #[napi]
    pub fn close(&mut self, descriptor: Descriptor) -> i32 {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return self.error(EINVAL);
        };
        self.flags.remove(&fd);
        if let Some(socket) = self.sockets.remove(&fd) {
            drop(socket);
            return 0;
        }
        self.finish(os::close(fd))
    }

    #[napi(js_name = "poll_descriptors")]
    pub fn poll_descriptors(&mut self, descriptors: Array<'_>, timeout: f64) -> Result<Vec<u32>> {
        if descriptors.len() as usize > MAX_REGISTRATIONS || !integer_in_range(timeout, 0.0, 1000.0)
        {
            return Err(napi::Error::from_reason("descriptor poll bounds exceeded"));
        }
        let registrations = (0..descriptors.len())
            .map(|index| {
                let entry = descriptors
                    .get::<PollDescriptor>(index)?
                    .ok_or_else(|| napi::Error::from_reason("missing poll registration"))?;
                let fd = descriptor_raw(&entry.fd)
                    .ok_or_else(|| napi::Error::from_reason("invalid raw descriptor"))?;
                if !integer_in_range(entry.events, 0.0, 5.0) || (entry.events as u32) & !5 != 0 {
                    return Err(napi::Error::from_reason("unsupported poll interests"));
                }
                Ok((fd, entry.events as i16))
            })
            .collect::<Result<Vec<_>>>()?;
        match os::poll(&registrations, timeout as i32) {
            Ok(events) => Ok(events.into_iter().map(u32::from).collect()),
            Err(code) => {
                self.last_error = code;
                Err(napi::Error::from_reason(self.strerror(code)))
            }
        }
    }

    #[napi]
    pub fn poll(
        &mut self,
        env: Env,
        descriptors: Unknown<'_>,
        count: f64,
        timeout: f64,
    ) -> Result<i32> {
        if !integer_in_range(count, 0.0, MAX_REGISTRATIONS as f64)
            || !integer_in_range(timeout, -1.0, f64::from(i32::MAX))
        {
            return Ok(self.error(EINVAL));
        }
        let count = count as usize;
        let timeout = timeout as i32;
        let data = View::optional(env, descriptors)?;
        if count == 0 {
            return Ok(self.finish(os::poll(&[], timeout).map(|_| 0)));
        }
        let Some(data) = data else {
            return Ok(self.error(EINVAL));
        };
        if data.len() < count * 8 {
            return Ok(self.error(EINVAL));
        }
        let mut registrations = Vec::with_capacity(count);
        for index in 0..count {
            let fd = data.read_i32(index * 8);
            let events = data.read_u32(index * 8 + 4) & 0xffff;
            if fd < 0 || events & !5 != 0 {
                return Ok(self.error(EINVAL));
            }
            registrations.push((fd as os::Raw, events as i16));
        }
        match os::poll(&registrations, timeout) {
            Ok(events) => {
                let mut ready = 0;
                for (index, events) in events.into_iter().enumerate() {
                    let old = data.read_u32(index * 8 + 4) & 0xffff;
                    data.write_u32(index * 8 + 4, old | u32::from(events) << 16);
                    ready += i32::from(events != 0);
                }
                Ok(ready)
            }
            Err(code) => Ok(self.error(code)),
        }
    }

    #[napi]
    pub fn inet_pton(
        &mut self,
        env: Env,
        family: i32,
        host: String,
        destination: Unknown<'_>,
    ) -> Result<i32> {
        let bytes = View::new(env, destination)?;
        if family != 2 || bytes.len() < 4 {
            return Ok(self.error(EINVAL));
        }
        match host.parse::<Ipv4Addr>() {
            Ok(address) => {
                bytes.write(0, &address.octets());
                Ok(1)
            }
            Err(_) => Ok(0),
        }
    }
}

impl Sys {
    fn error(&mut self, code: i32) -> i32 {
        self.last_error = code;
        -1
    }
    fn fd_error(&mut self, code: i32) -> Descriptor {
        self.error(code);
        Either::A(-1.0)
    }
    fn finish(&mut self, result: std::result::Result<i32, i32>) -> i32 {
        match result {
            Ok(value) => value,
            Err(code) => self.error(code),
        }
    }
    fn retain(&mut self, socket: Socket) -> Descriptor {
        let fd = os::raw(&socket);
        self.sockets.insert(fd, socket);
        self.flags.insert(fd, 0);
        descriptor_value(fd as u64)
    }
    fn address_call(
        &mut self,
        env: Env,
        descriptor: Descriptor,
        address: Unknown<'_>,
        length: f64,
        connect: bool,
    ) -> Result<i32> {
        let Some(fd) = descriptor_raw(&descriptor) else {
            return Ok(self.error(EINVAL));
        };
        let address = View::new(env, address)?;
        let Some(address) = parse_address(address, length) else {
            return Ok(self.error(EINVAL));
        };
        Ok(self.finish(if connect {
            os::connect(fd, &address)
        } else {
            os::bind(fd, &address)
        }))
    }
}

fn descriptor_value(value: u64) -> Descriptor {
    if value < (1u64 << 53) {
        Either::A(value as f64)
    } else {
        Either::B(BigInt {
            sign_bit: false,
            words: vec![value],
        })
    }
}

fn integer_in_range(value: f64, minimum: f64, maximum: f64) -> bool {
    value.is_finite() && value >= minimum && value <= maximum && value.fract() == 0.0
}

fn descriptor_raw(value: &Descriptor) -> Option<os::Raw> {
    let value = match value {
        Either::A(number)
            if number.is_finite()
                && *number >= 0.0
                && *number <= 9_007_199_254_740_991.0
                && number.fract() == 0.0 =>
        {
            *number as u64
        }
        Either::B(bigint) if !bigint.sign_bit && bigint.words.len() <= 1 => {
            bigint.words.first().copied().unwrap_or(0)
        }
        _ => return None,
    };
    os::Raw::try_from(value).ok()
}

fn parse_address(bytes: View, length: f64) -> Option<SockAddr> {
    if length != 16.0 || bytes.len() < 16 {
        return None;
    }
    let bytes = bytes.read::<16>(0);
    if bytes[0..2] != [2, 0] {
        return None;
    }
    Some(
        SocketAddrV4::new(
            Ipv4Addr::new(bytes[4], bytes[5], bytes[6], bytes[7]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        )
        .into(),
    )
}

fn address_destination(
    env: Env,
    address: Unknown<'_>,
    length: Unknown<'_>,
) -> Result<Option<(View, View)>> {
    let address = View::optional(env, address)?;
    let length = View::optional(env, length)?;
    match (address, length) {
        (None, None) => Ok(None),
        (Some(address), Some(length))
            if address.len() >= 16 && length.len() >= 4 && length.read_u32(0) >= 16 =>
        {
            Ok(Some((address, length)))
        }
        _ => Err(napi::Error::from_reason(
            "address output requires a 16-byte view and a length >=16",
        )),
    }
}

fn write_address(destination: View, length: View, address: &[u8; 16]) {
    destination.write(0, address);
    length.write_u32(0, 16);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn descriptor_numbers_are_lossless_and_bounded() {
        for number in [-1.0, 0.5, f64::NAN, f64::INFINITY, 9_007_199_254_740_992.0] {
            assert!(descriptor_raw(&Either::A(number)).is_none());
        }
        assert_eq!(descriptor_raw(&Either::A(17.0)), Some(17));
        #[cfg(windows)]
        assert_eq!(
            descriptor_raw(&descriptor_value(u64::MAX)),
            Some(usize::MAX)
        );
    }
}
