// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::check_book;
use crate::syntax;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::eyre;
use facet::Facet;
use figue as args;
use teamy_cancellation::CancellationToken;

/// Normalize a closed definition after checking the entire source program.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct EvalArgs {
    /// Bend source file to evaluate.
    #[facet(args::positional)]
    pub file: String,
    /// Name of the closed definition, defaulting to main.
    #[facet(args::named)]
    pub entry: Option<String>,
}

#[derive(Facet, Debug)]
struct EvalReport {
    entry: String,
    value: String,
}

impl EvalArgs {
    /// Check and evaluate the selected entry point.
    ///
    /// # Errors
    /// Returns parse, check, evaluation or cancellation errors.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let book =
            syntax::load(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
        let checked = check_book(&book).map_err(|error| eyre!("{error}"))?;
        let entry = self.entry.unwrap_or_else(|| "main".to_owned());
        let value = checked
            .evaluate(&entry, &[])
            .map_err(|error| eyre!("{error}"))?;
        cancellation.bail_if_cancelled()?;
        Ok(CliOutput::facet(EvalReport {
            entry,
            value: value.to_string(),
        }))
    }
}
