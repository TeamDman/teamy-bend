// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

struct Fixture {
    directory: PathBuf,
    path: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-host-files-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = os_bytes(directory.join("fixture.bin").into_os_string()).unwrap();
        Self { directory, path }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // This fixture owns one flat, uniquely created directory. Remove only
        // its named file and the now-empty directory, never a recursive tree.
        let _ = std::fs::remove_file(self.directory.join("fixture.bin"));
        let _ = std::fs::remove_dir(&self.directory);
    }
}

fn error_code<T: std::fmt::Debug>(result: HostResult<T>) -> u32 {
    match result.expect_err("operation must fail") {
        Failure::Io(error) => {
            assert!(!error.message.is_empty());
            error.code
        }
        Failure::Limit(message) => panic!("unexpected resource error: {message}"),
    }
}

#[test]
fn file_modes_preserve_truncate_append_binary_data_position_and_eof() {
    let fixture = Fixture::new();
    let mut file = open(&fixture.path, b"w").unwrap();
    write(&mut file, b"first\0\xff", 100).unwrap();
    close(file);
    let mut file = open(&fixture.path, b"a").unwrap();
    write(&mut file, b"tail", 100).unwrap();
    close(file);
    let mut file = open(&fixture.path, b"r").unwrap();
    assert_eq!(read(&mut file, 2, 100).unwrap(), b"fi");
    assert_eq!(read(&mut file, 100, 100).unwrap(), b"rst\0\xfftail");
    assert!(read(&mut file, 100, 100).unwrap().is_empty());
    close(file);
    close(open(&fixture.path, b"w").unwrap());
    assert_eq!(
        std::fs::metadata(os_string(&fixture.path).unwrap())
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn failed_reads_and_writes_retain_the_owned_file_and_empty_write_succeeds() {
    let fixture = Fixture::new();
    let mut file = open(&fixture.path, b"w").unwrap();
    #[cfg(windows)]
    assert!(read(&mut file, 0, 100).unwrap().is_empty());
    #[cfg(not(windows))]
    assert_eq!(
        error_code(read(&mut file, 0, 100)),
        codes::BAD_FILE.cast_unsigned()
    );
    assert_eq!(
        error_code(read(&mut file, 1, 100)),
        codes::BAD_FILE.cast_unsigned()
    );
    write(&mut file, b"kept", 100).unwrap();
    close(file);
    let mut file = open(&fixture.path, b"r").unwrap();
    assert_eq!(
        error_code(write(&mut file, b"x", 100)),
        codes::BAD_FILE.cast_unsigned()
    );
    write(&mut file, b"", 100).unwrap();
    assert_eq!(read(&mut file, 10, 100).unwrap(), b"kept");
    close(file);
}

#[test]
fn invalid_paths_and_modes_fail_before_opening_or_truncating() {
    let fixture = Fixture::new();
    std::fs::write(os_string(&fixture.path).unwrap(), b"preserved").unwrap();
    for mode in [b"rw".as_slice(), b"w\0", b"", b"R"] {
        assert_eq!(
            error_code(open(&fixture.path, mode)),
            codes::INVALID.cast_unsigned()
        );
    }
    let mut nul_path = fixture.path.clone();
    nul_path.push(0);
    assert_eq!(
        error_code(open(&nul_path, b"bad")),
        codes::INVALID_TEXT.cast_unsigned()
    );
    assert_eq!(
        std::fs::read(os_string(&fixture.path).unwrap()).unwrap(),
        b"preserved"
    );
    assert_eq!(
        error_code(open(b"", b"r")),
        codes::NOT_FOUND.cast_unsigned()
    );
}

#[test]
fn environment_distinguishes_empty_missing_and_invalid_names_without_global_mutation() {
    assert!(
        get_env_with(b"EMPTY", 100, |_| Some(OsString::new()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        error_code(get_env_with(b"MISSING", 100, |_| None)),
        codes::NOT_FOUND.cast_unsigned()
    );
    for name in [b"BAD\0NAME".as_slice(), b"BAD=NAME"] {
        assert_eq!(
            error_code(get_env_with(name, 100, |_| panic!(
                "invalid name reached host"
            ))),
            codes::NOT_FOUND.cast_unsigned()
        );
    }
    assert_eq!(
        get_env_with(b"TEXT", 100, |name| {
            assert_eq!(name, "TEXT");
            Some(OsString::from("aé🦀"))
        })
        .unwrap(),
        [0x61, 0xe9, 0x1f980]
    );
}

#[test]
fn budgets_fail_before_a_host_transfer_or_mutation() {
    assert_eq!(read_size(u32::MAX, usize::MAX).unwrap(), i32::MAX as usize);
    assert!(matches!(read_size(u32::MAX, 100), Err(Failure::Limit(_))));
    assert!(matches!(
        decode_native_text(b"123", 2),
        Err(Failure::Limit(_))
    ));
    assert!(matches!(
        get_env_with(b"LONG", 2, |_| Some(OsString::from("123"))),
        Err(Failure::Limit(_))
    ));
    let fixture = Fixture::new();
    let mut file = open(&fixture.path, b"w").unwrap();
    assert!(matches!(
        write(&mut file, b"123", 2),
        Err(Failure::Limit(_))
    ));
    close(file);
    assert_eq!(
        std::fs::metadata(os_string(&fixture.path).unwrap())
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn native_decoder_preserves_malformed_overlong_surrogate_and_out_of_range_values() {
    let cases: &[(&[u8], &[u32])] = &[
        (
            b"plain\0text",
            &[112, 108, 97, 105, 110, 0, 116, 101, 120, 116],
        ),
        ("é🦀".as_bytes(), &[0xe9, 0x1f980]),
        (b"\x80\x81", &[0x80, 0x81]),
        (b"\xc0\xaf", &[0x2f]),
        (b"\xed\xa0\x80", &[0xd800]),
        (b"\xf4\x90\x80\x80", &[0x0011_0000]),
        (b"\xff\xbf\xbf\xbf", &[0x001f_ffff]),
        (b"\xf0\x80\x80", &[0xf0, 0x80, 0x80]),
        (b"A\xe2\x82B", &[65, 0xe2, 0x82, 66]),
    ];
    for (bytes, expected) in cases {
        assert_eq!(decode_native_text(bytes, 100).unwrap(), *expected);
    }
}

#[test]
fn native_encoder_matches_scalar_utf8_and_keeps_native_nonscalar_behavior() {
    for character in ['\0', 'A', 'é', '\u{ffff}', '🦀', '\u{10ffff}'] {
        let (bytes, len) = encode_native_char(u32::from(character));
        let mut expected = [0; 4];
        assert_eq!(
            &bytes[..len],
            character.encode_utf8(&mut expected).as_bytes()
        );
    }
    for (codepoint, expected) in [
        (0xd800, b"\xed\xa0\x80".as_slice()),
        (0x0011_0000, b"\xf4\x90\x80\x80"),
        (u32::MAX, b"\xff\xbf\xbf\xbf"),
    ] {
        let (bytes, len) = encode_native_char(codepoint);
        assert_eq!(&bytes[..len], expected);
    }
}

struct ShortReader {
    calls: usize,
}

impl Read for ShortReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        buf[0] = 7;
        Ok(1)
    }
}

#[test]
fn read_returns_one_short_host_read_without_filling_the_buffer() {
    let mut reader = ShortReader { calls: 0 };
    assert_eq!(read_once(&mut reader, 100).unwrap(), [7]);
    assert_eq!(reader.calls, 1);
}

struct ShortWriter {
    bytes: Vec<u8>,
    maximum: usize,
    interrupt_after: Option<usize>,
}

impl Write for ShortWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.interrupt_after == Some(self.bytes.len()) {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        let count = buf.len().min(self.maximum);
        self.bytes.extend_from_slice(&buf[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("File.write does not flush");
    }
}

#[test]
fn write_finishes_short_writes_and_preserves_partial_output_on_interruption() {
    let mut writer = ShortWriter {
        bytes: Vec::new(),
        maximum: 2,
        interrupt_after: None,
    };
    write_all_chunks(&mut writer, b"abcdefg").unwrap();
    assert_eq!(writer.bytes, b"abcdefg");
    writer.bytes.clear();
    writer.interrupt_after = Some(2);
    assert_eq!(error_code(write_all_chunks(&mut writer, b"abcdefg")), 4);
    assert_eq!(writer.bytes, b"ab");
    writer.bytes.clear();
    writer.interrupt_after = None;
    writer.maximum = 0;
    assert!(matches!(
        write_all_chunks(&mut writer, b"x"),
        Err(Failure::Limit(_))
    ));
}

#[cfg(windows)]
#[test]
fn windows_uses_crt_codes_messages_and_explicit_unknown_error_fallback() {
    assert_eq!(
        errno(codes::NOT_FOUND).message,
        b"No such file or directory"
    );
    assert_eq!(codes::INVALID_TEXT, 42);
    for (native, expected) in [
        (2, 2),
        (3, 2),
        (5, 13),
        (6, 9),
        (80, 17),
        (206, 38),
        (i32::MAX, 5),
    ] {
        assert_eq!(
            host_error(&io::Error::from_raw_os_error(native)).code,
            expected
        );
    }
    assert_eq!(
        error_code(open(b"\xff", b"r")),
        codes::INVALID_TEXT.cast_unsigned()
    );
}

#[cfg(unix)]
#[test]
fn unix_environment_and_paths_preserve_non_utf8_bytes() {
    use std::os::unix::ffi::OsStringExt;
    assert_eq!(
        get_env_with(b"raw\xff", 100, |name| {
            assert_eq!(name.into_vec(), b"raw\xff");
            Some(OsString::from_vec(vec![0xff, 0x80]))
        })
        .unwrap(),
        [0xff, 0x80]
    );
    assert_eq!(
        os_bytes(os_string(b"raw\xff").unwrap()).unwrap(),
        b"raw\xff"
    );
}
