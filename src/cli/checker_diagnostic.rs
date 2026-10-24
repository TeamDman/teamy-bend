// SPDX-License-Identifier: MPL-2.0
use super::output::CliOutput;
use crate::kernel::KernelError;
use crate::syntax::diagnostics::DiagnosticOrigins;
use eyre::Context;
use eyre::eyre;
use std::fmt::Write as _;
use std::io::Write;
use teamy_cancellation::CancellationToken;

pub(super) fn reject(
    error: &KernelError,
    origins: &DiagnosticOrigins,
    cancellation: &CancellationToken,
) -> eyre::Result<CliOutput> {
    emit(error, origins, cancellation, &mut std::io::stderr().lock())
}

fn emit(
    error: &KernelError,
    origins: &DiagnosticOrigins,
    cancellation: &CancellationToken,
    stderr: &mut impl Write,
) -> eyre::Result<CliOutput> {
    cancellation.bail_if_cancelled()?;
    let Some(proof) = error.reflexivity_diagnostic() else {
        return Err(eyre!("{error}"));
    };
    let Some(excerpt) = origins.excerpt(proof.proof_identity) else {
        // Programmatically constructed or untracked terms keep the ordinary
        // host diagnostic. Never guess a source line from an error string.
        return Err(eyre!("{error}"));
    };
    let mut diagnostic = format!(
        "Error:\n- expected : {}\n- observed : {}",
        proof.expected, proof.observed
    );
    if !proof.context.is_empty() {
        diagnostic.push_str("\nContext:");
        let width = proof
            .context
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        for (name, ty) in &proof.context {
            write!(diagnostic, "\n- {name:width$} : {ty}").expect("String formatting");
        }
    }
    write!(diagnostic, "\nLocation: {}\n{excerpt}\n", proof.definition).expect("String formatting");
    tracing::debug!(internal_error = %error, "Bend reflexivity check rejected");
    stderr
        .write_all(diagnostic.as_bytes())
        .wrap_err("cannot write Bend checker diagnostic")?;
    stderr
        .flush()
        .wrap_err("cannot flush Bend checker diagnostic")?;
    Ok(CliOutput::exit_status(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::check_book;
    use crate::syntax::load_with_diagnostics;
    use std::io;

    fn rejected() -> (KernelError, DiagnosticOrigins) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-checking/false_brace.bend");
        let (book, origins) = load_with_diagnostics(path).unwrap();
        (check_book(&book).unwrap_err(), origins)
    }

    #[test]
    fn cancellation_and_untracked_errors_never_emit_a_partial_language_report() {
        let (error, origins) = rejected();
        let cancellation = CancellationToken::new();
        cancellation.request_cancel("stop diagnostic");
        let mut bytes = Vec::new();
        assert!(
            emit(&error, &origins, &cancellation, &mut bytes)
                .unwrap_err()
                .to_string()
                .contains("stop diagnostic")
        );
        assert!(bytes.is_empty());
        let fallback = emit(
            &error,
            &DiagnosticOrigins::default(),
            &CancellationToken::new(),
            &mut bytes,
        )
        .unwrap_err();
        assert_eq!(fallback.to_string(), error.to_string());
        assert!(bytes.is_empty());
        let ordinary = KernelError::new("ordinary type error");
        assert_eq!(
            emit(&ordinary, &origins, &CancellationToken::new(), &mut bytes)
                .unwrap_err()
                .to_string(),
            "ordinary type error"
        );
        assert!(bytes.is_empty());
    }

    #[test]
    fn diagnostic_write_and_flush_failures_remain_host_errors() {
        struct Writer {
            fail_write: bool,
            flushed: bool,
        }
        impl Write for Writer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                if self.fail_write {
                    Err(io::Error::other("write failed"))
                } else {
                    Ok(buf.len())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                self.flushed = true;
                Err(io::Error::other("flush failed"))
            }
        }
        let (error, origins) = rejected();
        for fail_write in [true, false] {
            let mut writer = Writer {
                fail_write,
                flushed: false,
            };
            let failure =
                emit(&error, &origins, &CancellationToken::new(), &mut writer).unwrap_err();
            assert!(failure.to_string().contains(if fail_write {
                "cannot write"
            } else {
                "cannot flush"
            }));
            assert_eq!(writer.flushed, !fail_write);
        }
        let output = emit(&error, &origins, &CancellationToken::new(), &mut Vec::new()).unwrap();
        assert_eq!(output.exit_code(), 1);
    }
}
