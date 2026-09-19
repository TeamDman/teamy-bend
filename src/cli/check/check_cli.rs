// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::Declaration;
use crate::kernel::check_book;
use crate::syntax;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::eyre;
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
    /// # Errors
    /// Returns parse, import, type, proof or cancellation errors.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let book =
            syntax::load(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
        check_book(&book).map_err(|error| eyre!("{error}"))?;
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
