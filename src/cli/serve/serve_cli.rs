// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::CheckedBook;
use crate::kernel::check_book;
use crate::protocol::DataValue;
use crate::protocol::Request;
use crate::protocol::Response;
use crate::protocol::arguments;
use crate::syntax;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use facet::Facet;
use figue as args;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;
use teamy_cancellation::CancellationToken;

const MAX_LINE_BYTES: usize = 1024 * 1024;
const MAX_JSON_DEPTH: usize = 208;

/// Check once and serve typed data calls until standard input ends.
///
/// Standard output is always newline-delimited JSON, independent of the global
/// output-format option. Requests have strictly increasing integer IDs.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ServeArgs {
    /// Bend source file containing the functions to invoke.
    #[facet(args::positional)]
    pub file: String,
}

impl ServeArgs {
    /// Run a persistent, bounded data session.
    ///
    /// # Errors
    /// Returns errors for invalid source, framing, I/O or cancellation.
    /// Individual typed-call failures become JSON error responses.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        let cancellation = cancellation.clone();
        // Recursive reflection for tagged Nat64 exceeds the native Windows
        // main-thread stack. A bounded worker stack supports the documented
        // constructor depth; explicit JSON/AST budgets remain unchanged.
        // Keep this boundary for library callers invoking ServeArgs directly,
        // even though the executable also supplies a bounded CLI worker stack.
        std::thread::Builder::new()
            .name("bend-data-session".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || self.serve(&cancellation))
            .wrap_err("cannot start data session worker")?
            .join()
            .map_err(|_panic_payload| eyre!("data session worker panicked"))??;
        Ok(CliOutput::none())
    }

    fn serve(self, cancellation: &CancellationToken) -> Result<()> {
        let book =
            syntax::load(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
        let checked = check_book(&book).map_err(|error| eyre!("{error}"))?;
        let (send, receive) = mpsc::sync_channel(1);
        // A dedicated stdin reader keeps cancellation responsive while a client
        // has no request ready. No Bend terms cross the thread boundary.
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut input = stdin.lock();
            loop {
                match read_line(&mut input) {
                    Ok(Some(line)) => {
                        if send.send(Ok(line)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        let _ = send.send(Err(error.to_string()));
                        break;
                    }
                }
            }
        });
        let mut stdout = std::io::stdout().lock();
        let mut last_id = None;
        loop {
            cancellation.bail_if_cancelled()?;
            let line = match receive.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(line)) => line,
                Ok(Err(error)) => {
                    emit(&mut stdout, &failure(None, &error))?;
                    bail!("request framing failed: {error}");
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };
            let response = respond(&checked, &line, &mut last_id);
            emit(&mut stdout, &response)?;
        }
        Ok(())
    }
}

fn read_line(input: &mut impl BufRead) -> Result<Option<String>> {
    let mut bytes = Vec::new();
    let count = input
        .take((MAX_LINE_BYTES + 1) as u64)
        .read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    if bytes.len() > MAX_LINE_BYTES {
        bail!("request line exceeds 1 MiB");
    }
    String::from_utf8(bytes)
        .map(Some)
        .wrap_err("request line is not UTF-8")
}

fn respond(checked: &CheckedBook, line: &str, last_id: &mut Option<u64>) -> Response {
    if let Err(error) = check_json_depth(line) {
        return failure(None, &error.to_string());
    }
    let request: Request = match facet_json::from_str(line) {
        Ok(request) => request,
        Err(error) => return failure(None, &format!("invalid request: {error}")),
    };
    if last_id.is_some_and(|last| request.id <= last) {
        return failure(Some(request.id), "request IDs must strictly increase");
    }
    *last_id = Some(request.id);
    let result = arguments(&request.args)
        .and_then(|args| {
            checked
                .evaluate_data(&request.entry, &args)
                .map_err(|error| eyre!("{error}"))
        })
        .and_then(|value| DataValue::from_term(&value));
    match result {
        Ok(value) => Response {
            id: Some(request.id),
            value: Some(value),
            error: None,
        },
        Err(error) => failure(Some(request.id), &error.to_string()),
    }
}

fn emit(output: &mut impl Write, response: &Response) -> Result<()> {
    writeln!(output, "{}", facet_json::to_string(response)?)?;
    output.flush()?;
    Ok(())
}

fn failure(id: Option<u64>, error: &str) -> Response {
    Response {
        id,
        value: None,
        error: Some(error.to_owned()),
    }
}

fn check_json_depth(input: &str) -> Result<()> {
    let (mut quoted, mut escaped, mut started, mut complete) = (false, false, false, false);
    let mut delimiters = Vec::new();
    for byte in input.bytes() {
        if complete {
            if !matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
                bail!("invalid request: trailing content after JSON object");
            }
            continue;
        }
        if !started {
            if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
                continue;
            }
            if byte != b'{' {
                bail!("invalid request: expected one JSON object");
            }
            started = true;
        }
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    delimiters.push(byte);
                    if delimiters.len() > MAX_JSON_DEPTH {
                        bail!("JSON nesting exceeds {MAX_JSON_DEPTH}");
                    }
                }
                b'}' | b']' => {
                    let expected = if byte == b'}' { b'{' } else { b'[' };
                    if delimiters.pop() != Some(expected) {
                        bail!("invalid request: mismatched JSON delimiters");
                    }
                    complete = delimiters.is_empty();
                }
                _ => (),
            }
        }
    }
    if !complete {
        bail!("invalid request: incomplete JSON object");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::MAX_JSON_DEPTH;
    use super::MAX_LINE_BYTES;
    use super::check_json_depth;
    use super::read_line;
    use std::io::Cursor;

    #[test]
    fn request_framing_enforces_byte_limit_and_utf8() {
        let mut accepted = Cursor::new(vec![b' '; MAX_LINE_BYTES]);
        assert_eq!(
            read_line(&mut accepted)
                .expect("maximum line accepted")
                .expect("one line")
                .len(),
            MAX_LINE_BYTES
        );
        let mut rejected = Cursor::new(vec![b' '; MAX_LINE_BYTES + 1]);
        assert!(
            read_line(&mut rejected)
                .expect_err("oversize line")
                .to_string()
                .contains("1 MiB")
        );
        let mut invalid = Cursor::new(vec![0xff, b'\n']);
        assert!(
            read_line(&mut invalid)
                .expect_err("invalid UTF-8")
                .to_string()
                .contains("UTF-8")
        );
    }

    #[test]
    fn request_framing_checks_balancing_depth_and_exact_json_whitespace() {
        for text in [
            "{}\u{000b}",
            "{}\u{000c}",
            "{]",
            "{} {}",
            "{}garbage",
            "",
            "{\"x\":\"",
        ] {
            let _failure = check_json_depth(text).expect_err("invalid JSON framing");
        }
        check_json_depth(" \t{\"x\":\"\\\"}][\"}\r\n").expect("quoted delimiters are ignored");
        let excessive = format!(
            "{{\"x\":{}0{}}}",
            "[".repeat(MAX_JSON_DEPTH),
            "]".repeat(MAX_JSON_DEPTH)
        );
        assert!(
            check_json_depth(&excessive)
                .expect_err("excessive JSON depth")
                .to_string()
                .contains("nesting")
        );
    }
}
