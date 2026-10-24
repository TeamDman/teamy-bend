// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::check_executable_with_origins;
use crate::syntax::load_executable_with_diagnostics;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::eyre;
use facet::Facet;
use figue as args;
use std::io::Write;
use teamy_cancellation::CancellationToken;

/// Check executable contracts and run main with the native effect driver.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RunArgs {
    /// Bend source file whose main entry should run.
    #[facet(args::positional)]
    pub file: String,
}

impl RunArgs {
    /// Execute main and preserve its raw output and process status.
    ///
    /// Tracked checking/pattern-load failures write a Bend diagnostic and return output with
    /// exit status 1. Callers must inspect `CliOutput::exit_code()`.
    ///
    /// # Errors
    /// Rejects invalid programs, unsupported foreign implementations, runtime
    /// limits, output errors and cancellation.
    pub fn invoke(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let (source, origins) =
            match load_executable_with_diagnostics(std::path::Path::new(&self.file)) {
                Ok(loaded) => loaded,
                Err(error) => {
                    return crate::cli::checker_diagnostic::reject_load(&error, cancellation);
                }
            };
        let checked = match check_executable_with_origins(&source, Some(origins.tracker.clone())) {
            Ok(checked) => checked,
            Err(error) => {
                return crate::cli::checker_diagnostic::reject(&error, &origins, cancellation);
            }
        };
        drop(origins);
        let mut stdout = std::io::stdout().lock();
        let mut stderr = std::io::stderr().lock();
        let result = checked.run_main(&mut stdout, &mut stderr, &|| {
            cancellation.bail_if_cancelled().is_err()
        });
        // Preserve already-written bytes before main reports an execution error
        // on stderr, including an IO.write prefix without a newline.
        let flushed = stdout.flush();
        let code = result.map_err(|error| eyre!("{error}"))?;
        flushed.wrap_err("cannot flush program stdout")?;
        Ok(CliOutput::exit_status(code))
    }
}
