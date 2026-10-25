// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::Declaration;
use crate::kernel::check_book_with_origins;
use crate::syntax;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;
use teamy_cancellation::CancellationToken;

/// Check a complete Bend program, including every law's proof.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CheckArgs {
    /// Bend source file to check.
    #[facet(args::positional)]
    pub file: String,
}

#[derive(Facet, Debug)]
struct CheckReport {
    checked: bool,
    definitions: usize,
    datatypes: usize,
    laws: usize,
}

impl CheckArgs {
    /// Check a program and return counts only after complete validation.
    ///
    /// Tracked checking/pattern-load failures and final open-law summaries write
    /// a Bend diagnostic and return output with exit status 1. Callers must
    /// inspect `CliOutput::exit_code()`.
    ///
    /// # Errors
    /// Returns parse, import, type, proof or cancellation errors.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let (book, origins) = match syntax::load_with_diagnostics(std::path::Path::new(&self.file))
        {
            Ok(loaded) => loaded,
            Err(error) => return crate::cli::checker_diagnostic::reject_load(&error, cancellation),
        };
        if let Err(error) = check_book_with_origins(&book, Some(origins.tracker.clone())) {
            return crate::cli::checker_diagnostic::reject(&error, &origins, cancellation);
        }
        cancellation.bail_if_cancelled()?;
        let mut report = CheckReport {
            checked: true,
            definitions: 0,
            datatypes: 0,
            laws: 0,
        };
        for declaration in &book.declarations {
            match declaration {
                Declaration::Adt(_) => report.datatypes += 1,
                Declaration::Def(definition) if definition.body.is_none() => report.laws += 1,
                Declaration::Def(_) => report.definitions += 1,
            }
        }
        Ok(CliOutput::facet(report))
    }
}
