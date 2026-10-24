// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::compiler::CompileError;
use eyre::Context;
use eyre::eyre;
use std::io::Write;
use teamy_cancellation::CancellationToken;

pub(super) fn reject(
    error: &CompileError,
    cancellation: &CancellationToken,
) -> eyre::Result<CliOutput> {
    emit(error, cancellation, &mut std::io::stderr().lock())
}

fn emit(
    error: &CompileError,
    cancellation: &CancellationToken,
    stderr: &mut impl Write,
) -> eyre::Result<CliOutput> {
    cancellation.bail_if_cancelled()?;
    let Some(message) = error.language_diagnostic() else {
        return Err(eyre!("{error}"));
    };
    tracing::debug!(internal_error = %error, "Bend compilation rejected");
    writeln!(stderr, "Error: {message}").wrap_err("cannot write Bend compiler diagnostic")?;
    stderr
        .flush()
        .wrap_err("cannot flush Bend compiler diagnostic")?;
    Ok(CliOutput::exit_status(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_executable_c;
    use crate::compiler::compile_executable_javascript;
    use crate::kernel::check_executable;
    use crate::syntax::load_executable;
    use std::io;

    #[test]
    fn compiler_diagnostic_checks_status_cancellation_writes_and_flushes() {
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
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for (path, expected) in [
            (
                "tests/fixtures/cpu-no-main/proof/gadt_refutation.bend",
                "Error: a build needs import Base\n",
            ),
            (
                "tests/fixtures/cpu-ragged-rows/refusals/type-value.bend",
                "Error: main's type Type cannot be printed (a function, a Type, an erased or dependent field)\n",
            ),
        ] {
            let checked = check_executable(&load_executable(root.join(path)).unwrap()).unwrap();
            for error in [
                compile_executable_c(&checked).unwrap_err(),
                compile_executable_javascript(&checked).unwrap_err(),
            ] {
                let mut bytes = Vec::new();
                let token = CancellationToken::new();
                token.request_cancel("stop compiler diagnostic");
                assert!(
                    emit(&error, &token, &mut bytes)
                        .unwrap_err()
                        .to_string()
                        .contains("stop compiler diagnostic")
                );
                assert!(bytes.is_empty());
                for fail_write in [true, false] {
                    let error = emit(&error, &CancellationToken::new(), &mut Writer(fail_write))
                        .unwrap_err();
                    assert!(error.to_string().contains(if fail_write {
                        "cannot write"
                    } else {
                        "cannot flush"
                    }));
                }
                assert_eq!(
                    emit(&error, &CancellationToken::new(), &mut bytes)
                        .unwrap()
                        .exit_code(),
                    1
                );
                assert_eq!(bytes, expected.as_bytes());
            }
        }
    }
    #[test]
    fn unrelated_compilation_errors_keep_the_host_failure_boundary() {
        let book = crate::syntax::parse("def main(x: Type) -> Type: x\n").unwrap();
        for entry in ["main", "absent"] {
            let error = crate::compiler::compile_javascript(&book, entry).unwrap_err();
            let mut bytes = Vec::new();
            assert_eq!(
                emit(&error, &CancellationToken::new(), &mut bytes)
                    .unwrap_err()
                    .to_string(),
                error.to_string()
            );
            assert!(bytes.is_empty());
        }
    }
}
