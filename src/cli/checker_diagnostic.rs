// SPDX-License-Identifier: MPL-2.0
use super::output::CliOutput;
use crate::kernel::KernelError;
use crate::syntax::diagnostics::DiagnosticOrigins;
use crate::syntax::diagnostics::LoadDiagnosticContent;
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
    let Some(report) = error.checker_diagnostic() else {
        return Err(eyre!("{error}"));
    };
    let Some(excerpt) = origins.excerpt(report.term_identity) else {
        // Programmatically constructed or untracked terms keep the ordinary
        // host diagnostic. Never guess a source line from an error string.
        return Err(eyre!("{error}"));
    };
    let mut diagnostic = format!(
        "Error:\n- expected : {}\n- observed : {}",
        report.expected, report.observed
    );
    if !report.context.is_empty() {
        diagnostic.push_str("\nContext:");
        let width = report
            .context
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        for (name, ty) in &report.context {
            write!(diagnostic, "\n- {name:width$} : {ty}").expect("String formatting");
        }
    }
    write!(diagnostic, "\nLocation: {}\n{excerpt}\n", report.definition)
        .expect("String formatting");
    tracing::debug!(internal_error = %error, "Bend check rejected");
    write_diagnostic(&diagnostic, stderr)
}

pub(super) fn reject_load(
    failure: &crate::syntax::diagnostics::DiagnosticLoadError,
    cancellation: &CancellationToken,
) -> eyre::Result<CliOutput> {
    emit_load(failure, cancellation, &mut std::io::stderr().lock())
}

fn emit_load(
    failure: &crate::syntax::diagnostics::DiagnosticLoadError,
    cancellation: &CancellationToken,
    stderr: &mut impl Write,
) -> eyre::Result<CliOutput> {
    cancellation.bail_if_cancelled()?;
    let Some(report) = &failure.diagnostic else {
        return Err(eyre!("{}", failure.error));
    };
    tracing::debug!(internal_error = %failure.error, "Bend source load rejected");
    let content = match &report.content {
        LoadDiagnosticContent::Message(message) => format!("- message  : {message}"),
        LoadDiagnosticContent::ExpectedObserved { expected, observed } => {
            format!("- expected : {expected}\n- observed : {observed}")
        }
    };
    write_diagnostic(
        &format!("Error:\n{content}\nLocation:\n{}\n", report.excerpt),
        stderr,
    )
}

fn write_diagnostic(diagnostic: &str, stderr: &mut impl Write) -> eyre::Result<CliOutput> {
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
    use crate::kernel::check_book_with_origins;
    use crate::syntax::load_with_diagnostics;
    use std::io;

    fn rejected(name: &str) -> (KernelError, DiagnosticOrigins) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let (book, origins) = load_with_diagnostics(path).unwrap();
        (
            check_book_with_origins(&book, Some(origins.tracker.clone())).unwrap_err(),
            origins,
        )
    }

    #[test]
    fn cancellation_and_untracked_errors_never_emit_a_partial_language_report() {
        for name in [
            "tests/fixtures/cpu-checking/false_brace.bend",
            "tests/fixtures/cpu-constructors/nat_literal_import.bend",
            "tests/fixtures/cpu-plus-binders/plus_binder_term.bend",
            "tests/fixtures/cpu-rewrite-checking/rewrite_ctor_brace.bend",
        ] {
            let (error, origins) = rejected(name);
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
    }

    #[test]
    fn pattern_load_boundary_preserves_cancellation_writer_errors_and_fallback() {
        struct Writer(bool);
        impl Write for Writer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                if self.0 {
                    Err(io::Error::other("write failed"))
                } else {
                    Ok(buf.len())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("flush failed"))
            }
        }
        for (file, excerpt) in [
            (
                "cpu-constructors/nested-pattern.bend",
                "7>|       Missing{}",
            ),
            (
                "cpu-rewrite-checking/dead_motive_syntax.bend",
                "13>|   %[h => {y == h : U32}] e {=}",
            ),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(file);
            let mut error = load_with_diagnostics(path)
                .err()
                .expect("pattern load rejects");
            let mut output = Vec::new();
            let token = CancellationToken::new();
            token.request_cancel("stop load diagnostic");
            assert!(
                emit_load(&error, &token, &mut output)
                    .unwrap_err()
                    .to_string()
                    .contains("stop load diagnostic")
            );
            assert!(output.is_empty());
            for fail_write in [true, false] {
                let failure = emit_load(&error, &CancellationToken::new(), &mut Writer(fail_write))
                    .unwrap_err();
                assert!(failure.to_string().contains(if fail_write {
                    "cannot write"
                } else {
                    "cannot flush"
                }));
            }
            let result = emit_load(&error, &CancellationToken::new(), &mut output).unwrap();
            assert_eq!(result.exit_code(), 1);
            assert!(String::from_utf8(output).unwrap().contains(excerpt));
            error.diagnostic = None;
            let mut bytes = Vec::new();
            assert_eq!(
                emit_load(&error, &CancellationToken::new(), &mut bytes)
                    .unwrap_err()
                    .to_string(),
                error.error.to_string()
            );
            assert!(bytes.is_empty());
        }
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
        for name in [
            "tests/fixtures/cpu-checking/false_brace.bend",
            "tests/fixtures/cpu-constructors/nat_literal_import.bend",
            "tests/fixtures/cpu-plus-binders/plus_binder_term.bend",
            "tests/fixtures/cpu-rewrite-checking/rewrite_ctor_brace.bend",
        ] {
            let (error, origins) = rejected(name);
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
            let output =
                emit(&error, &origins, &CancellationToken::new(), &mut Vec::new()).unwrap();
            assert_eq!(output.exit_code(), 1);
        }
    }
}
