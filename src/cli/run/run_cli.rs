// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::check_executable;
use crate::syntax::load_executable;
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
    /// # Errors
    /// Rejects invalid programs, unsupported foreign implementations, runtime
    /// limits, output errors and cancellation.
    pub fn invoke(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let source =
            load_executable(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
        let checked = check_executable(&source).map_err(|error| eyre!("{error}"))?;
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
