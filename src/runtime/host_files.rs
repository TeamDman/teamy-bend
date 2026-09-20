// SPDX-License-Identifier: Apache-2.0
//! Native environment/file effects adapted from Bend 2.0.5 `effs/get_env.c`,
//! `effs/file_*.c` and `comp.ts` `io_str`. Copyright 2026 `HigherOrderCO`.
//! Rust ownership, resource bounds and Windows policy: `TeamDman`.
//! See NOTICE and licenses/Apache-2.0.txt.
//!
//! Unix retains native path/environment bytes and errno/strerror text. Windows
//! accepts UTF-8 paths and environment names, maps host failures to the CRT
//! errno-number domain, and uses CRT error descriptions on MSVC targets. This
//! is an explicit Windows adaptation, not a claim of POSIX/Win32 equivalence.

use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;

/// Host work has no VM references and may move into a worker thread.
#[derive(Debug)]
pub(super) struct NativeFile {
    file: File,
    mode: Mode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Read,
    Write,
    Append,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Error {
    pub(super) code: u32,
    pub(super) message: Vec<u8>,
}

/// Resource exhaustion is an evaluator error, never an ordinary IO.Fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    Io(Error),
    Limit(&'static str),
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::Io(error)
    }
}

type HostResult<T> = Result<T, Failure>;

#[cfg(unix)]
mod codes {
    pub(super) const NOT_FOUND: i32 = libc::ENOENT;
    pub(super) const BAD_FILE: i32 = libc::EBADF;
    pub(super) const INVALID: i32 = libc::EINVAL;
    pub(super) const INVALID_TEXT: i32 = libc::EILSEQ;
    pub(super) const IO: i32 = libc::EIO;
}

#[cfg(not(unix))]
mod codes {
    pub(super) const NOT_FOUND: i32 = 2;
    pub(super) const BAD_FILE: i32 = 9;
    pub(super) const INVALID: i32 = 22;
    pub(super) const INVALID_TEXT: i32 = 42;
    pub(super) const IO: i32 = 5;
}

pub(super) fn open(path: &[u8], mode: &[u8]) -> HostResult<NativeFile> {
    let mode = validate_open_mode(path, mode)?;
    let path = os_string(path)?;
    let mut options = OpenOptions::new();
    match mode {
        Mode::Read => {
            options.read(true);
        }
        Mode::Write => {
            options.write(true).create(true).truncate(true);
        }
        Mode::Append => {
            options.append(true).create(true);
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o644);
    }
    options
        .open(path)
        .map(|file| NativeFile { file, mode })
        .map_err(|error| Failure::Io(host_error(&error)))
}

/// Native argument errors finish synchronously before scheduling host work.
pub(super) fn validate_open(path: &[u8], mode: &[u8]) -> HostResult<()> {
    validate_open_mode(path, mode).map(|_mode| ())
}

fn validate_open_mode(path: &[u8], mode: &[u8]) -> HostResult<Mode> {
    // Upstream gives the path's embedded NUL priority over an invalid mode.
    if path.contains(&0) {
        return Err(errno(codes::INVALID_TEXT).into());
    }
    let mode = match mode {
        b"r" => Mode::Read,
        b"w" => Mode::Write,
        b"a" => Mode::Append,
        _ => return Err(errno(codes::INVALID).into()),
    };
    #[cfg(not(unix))]
    std::str::from_utf8(path).map_err(|_error| errno(codes::INVALID_TEXT))?;
    Ok(mode)
}

pub(super) fn get_env(name: &[u8], byte_limit: usize) -> HostResult<Vec<u32>> {
    get_env_with(name, byte_limit, std::env::var_os)
}

fn get_env_with(
    name: &[u8],
    byte_limit: usize,
    lookup: impl FnOnce(OsString) -> Option<OsString>,
) -> HostResult<Vec<u32>> {
    // getenv cannot find a name containing '='. Handle it without exposing
    // platform-specific Rust environment-name validation to a Bend program.
    if name.contains(&0) || name.contains(&b'=') {
        return Err(errno(codes::NOT_FOUND).into());
    }
    let value = lookup(os_string(name)?).ok_or_else(|| errno(codes::NOT_FOUND))?;
    let bytes = os_bytes(value)?;
    decode_native_text(&bytes, byte_limit)
}

/// Validate before creating a worker or allocating its transfer buffer.
pub(super) fn read_size(requested: u32, byte_limit: usize) -> HostResult<usize> {
    let count = usize::try_from(requested.min(i32::MAX.cast_unsigned()))
        .map_err(|_error| Failure::Limit("native file read size exceeds host range"))?;
    if count > byte_limit {
        return Err(Failure::Limit("native file read exceeds byte budget"));
    }
    Ok(count)
}

pub(super) fn read(
    file: &mut NativeFile,
    requested: u32,
    byte_limit: usize,
) -> HostResult<Vec<u8>> {
    let count = read_size(requested, byte_limit)?;
    // The Windows CRT accepts a zero-byte _read on a write-only descriptor;
    // POSIX read still validates that descriptor's access mode.
    #[cfg(windows)]
    if count == 0 {
        return Ok(Vec::new());
    }
    if file.mode != Mode::Read {
        return Err(errno(codes::BAD_FILE).into());
    }
    read_once(&mut file.file, count)
}

fn read_once(reader: &mut impl Read, count: usize) -> HostResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(count)
        .map_err(|_error| Failure::Limit("native file read allocation failed"))?;
    bytes.resize(count, 0);
    let used = reader
        .read(&mut bytes)
        .map_err(|error| Failure::Io(host_error(&error)))?;
    bytes.truncate(used);
    Ok(bytes)
}

pub(super) fn write(file: &mut NativeFile, bytes: &[u8], byte_limit: usize) -> HostResult<()> {
    if bytes.len() > byte_limit {
        return Err(Failure::Limit("native file write exceeds byte budget"));
    }
    // The upstream empty write succeeds without checking the descriptor.
    if !bytes.is_empty() && file.mode == Mode::Read {
        return Err(errno(codes::BAD_FILE).into());
    }
    write_all_chunks(&mut file.file, bytes)
}

fn write_all_chunks(writer: &mut impl Write, mut bytes: &[u8]) -> HostResult<()> {
    while !bytes.is_empty() {
        let used = writer
            .write(bytes)
            .map_err(|error| Failure::Io(host_error(&error)))?;
        if used == 0 {
            // Upstream loops forever in this case. The bounded runtime stops.
            return Err(Failure::Limit("native file write made no progress"));
        }
        bytes = &bytes[used..];
    }
    Ok(())
}

/// Upstream ignores close failures; File's owned destructor has the same policy.
pub(super) fn close(file: NativeFile) {
    drop(file);
}

/// Reproduce `io_str`'s reverse scan, including its handling of malformed UTF-8.
/// Values may be surrogate or out-of-range code points; do not use `from_utf8`.
pub(super) fn decode_native_text(bytes: &[u8], byte_limit: usize) -> HostResult<Vec<u32>> {
    if bytes.len() > byte_limit {
        return Err(Failure::Limit("native text exceeds byte budget"));
    }
    let mut text = Vec::new();
    text.try_reserve_exact(bytes.len())
        .map_err(|_error| Failure::Limit("native text allocation failed"))?;
    let mut remaining = bytes.len();
    while remaining > 0 {
        let mut continuation_count = 0;
        while continuation_count < 3
            && continuation_count + 1 < remaining
            && bytes[remaining - 1 - continuation_count] & 0xc0 == 0x80
        {
            continuation_count += 1;
        }
        let first = bytes[remaining - 1 - continuation_count];
        let mut len = match first {
            0..0xc0 => 0,
            0xc0..0xe0 => 2,
            0xe0..0xf0 => 3,
            _ => 4,
        };
        let mut codepoint = u32::from(bytes[remaining - 1]);
        if len == continuation_count + 1 {
            codepoint = u32::from(first & (0x7f >> len));
            for index in 1..len {
                codepoint = (codepoint << 6) | u32::from(bytes[remaining - len + index] & 0x3f);
            }
        } else {
            len = 1;
        }
        remaining -= len;
        text.push(codepoint);
    }
    text.reverse();
    Ok(text)
}

/// Encode all U32 values exactly as native `io_utf8` does, including truncating
/// the leading byte for values outside Unicode. File data can create such
/// characters through `io_str`, so native effect text must not scalar-check them.
pub(super) fn encode_native_char(mut codepoint: u32) -> ([u8; 4], usize) {
    let len = match codepoint {
        0..0x80 => 1,
        0x80..0x800 => 2,
        0x800..0x1_0000 => 3,
        _ => 4,
    };
    let mut bytes = [0; 4];
    for index in (1..len).rev() {
        bytes[index] = 0x80 | (codepoint & 0x3f).to_le_bytes()[0];
        codepoint >>= 6;
    }
    let first = if len == 1 {
        codepoint
    } else {
        (0xf00 >> len) | codepoint
    };
    bytes[0] = first.to_le_bytes()[0];
    (bytes, len)
}

#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "Windows text conversion can fail through the same platform-neutral interface"
)]
fn os_string(bytes: &[u8]) -> Result<OsString, Error> {
    use std::os::unix::ffi::OsStrExt;
    Ok(std::ffi::OsStr::from_bytes(bytes).to_owned())
}

#[cfg(not(unix))]
fn os_string(bytes: &[u8]) -> Result<OsString, Error> {
    std::str::from_utf8(bytes)
        .map(OsString::from)
        .map_err(|_error| errno(codes::INVALID_TEXT))
}

#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "Windows text conversion can fail through the same platform-neutral interface"
)]
fn os_bytes(value: OsString) -> Result<Vec<u8>, Error> {
    use std::os::unix::ffi::OsStringExt;
    Ok(value.into_vec())
}

#[cfg(not(unix))]
fn os_bytes(value: OsString) -> Result<Vec<u8>, Error> {
    value
        .into_string()
        .map(String::into_bytes)
        .map_err(|_error| errno(codes::INVALID_TEXT))
}

fn host_error(error: &io::Error) -> Error {
    #[cfg(unix)]
    {
        errno(error.raw_os_error().unwrap_or_else(|| match error.kind() {
            io::ErrorKind::Interrupted => libc::EINTR,
            io::ErrorKind::WouldBlock => libc::EAGAIN,
            _ => codes::IO,
        }))
    }
    #[cfg(not(unix))]
    {
        errno(windows_errno(error))
    }
}

#[cfg(not(unix))]
fn windows_errno(error: &io::Error) -> i32 {
    use io::ErrorKind;
    // Rust has no ErrorKind for invalid handles, descriptor exhaustion or
    // cross-device operations. Preserve those distinctions before kind mapping.
    match error.raw_os_error() {
        Some(4) => return 24,
        Some(6) => return codes::BAD_FILE,
        Some(8 | 14) => return 12,
        Some(17) => return 18,
        Some(206) => return 38,
        _ => {}
    }
    match error.kind() {
        ErrorKind::NotFound => 2,
        ErrorKind::PermissionDenied => 13,
        ErrorKind::AlreadyExists => 17,
        ErrorKind::NotADirectory => 20,
        ErrorKind::IsADirectory => 21,
        ErrorKind::InvalidInput | ErrorKind::InvalidData => 22,
        ErrorKind::FileTooLarge => 27,
        ErrorKind::StorageFull => 28,
        ErrorKind::NotSeekable => 29,
        ErrorKind::ReadOnlyFilesystem => 30,
        ErrorKind::BrokenPipe => 32,
        ErrorKind::DirectoryNotEmpty => 41,
        ErrorKind::OutOfMemory => 12,
        ErrorKind::Interrupted => 4,
        ErrorKind::WouldBlock => 11,
        ErrorKind::TimedOut => 138,
        // Unrecognized Win32 errors are EIO, never leaked as errno numbers.
        _ => codes::IO,
    }
}

fn errno(code: i32) -> Error {
    Error {
        code: code.cast_unsigned(),
        message: errno_text(code),
    }
}

#[cfg(unix)]
fn errno_text(code: i32) -> Vec<u8> {
    let mut buffer = [std::ffi::c_char::default(); 1024];
    // SAFETY: strerror_r receives a writable buffer and its exact length. The
    // POSIX variant used by libc writes a NUL-terminated result on success.
    let result = unsafe { libc::strerror_r(code, buffer.as_mut_ptr(), buffer.len()) };
    if result == 0 {
        // SAFETY: the successful call above NUL-terminated the live buffer.
        return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
            .to_bytes()
            .to_vec();
    }
    format!("OS error {code}").into_bytes()
}

#[cfg(all(windows, target_env = "msvc"))]
fn errno_text(code: i32) -> Vec<u8> {
    unsafe extern "C" {
        fn strerror_s(buffer: *mut std::ffi::c_char, size: usize, code: i32) -> i32;
    }
    let mut buffer = [std::ffi::c_char::default(); 1024];
    // SAFETY: the CRT receives a writable buffer and its exact size; strerror_s
    // accepts every error number and NUL-terminates the buffer on success.
    let result = unsafe { strerror_s(buffer.as_mut_ptr(), buffer.len(), code) };
    if result == 0 {
        // SAFETY: the successful call above NUL-terminated the live buffer.
        return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
            .to_bytes()
            .to_vec();
    }
    format!("OS error {code}").into_bytes()
}

#[cfg(not(any(unix, all(windows, target_env = "msvc"))))]
fn errno_text(code: i32) -> Vec<u8> {
    match code {
        codes::NOT_FOUND => b"No such file or directory".to_vec(),
        codes::BAD_FILE => b"Bad file descriptor".to_vec(),
        codes::INVALID => b"Invalid argument".to_vec(),
        codes::INVALID_TEXT => b"Invalid or incomplete multibyte or wide character".to_vec(),
        _ => b"Input/output error".to_vec(),
    }
}

#[cfg(test)]
#[path = "host_files/tests.rs"]
mod tests;
