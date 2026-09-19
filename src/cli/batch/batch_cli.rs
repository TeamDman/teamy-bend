// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::check_book;
use crate::kernel::term;
use crate::syntax;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use facet::Facet;
use figue as args;
use std::io::Read;
use teamy_cancellation::CancellationToken;

// Unary naturals are deliberately bounded at this external data interface.
// The language checker has independent reduction limits.
const MAX_INPUT_NAT: u64 = 4096;
const MAX_INPUT_BYTES: u64 = 32 * 1024 * 1024;

/// Evaluate a table of natural-number arguments with one checked program.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct BatchArgs {
    /// Bend source file containing the function.
    #[facet(args::positional)]
    pub file: String,
    /// Function name to apply to every row.
    #[facet(args::named)]
    pub entry: String,
    /// JSON file containing an array of arrays of nonnegative integers.
    #[facet(args::named)]
    pub args_json: String,
}

#[derive(Facet, Debug)]
struct BatchReport {
    results: Vec<u64>,
}

impl BatchArgs {
    /// Check once, then evaluate each row as a typed natural-number call.
    ///
    /// # Errors
    /// Returns errors for invalid programs, malformed JSON, non-natural
    /// results, resource limits, incorrect function arguments or cancellation.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let book =
            syntax::load(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
        let checked = check_book(&book).map_err(|error| eyre!("{error}"))?;
        let file = std::fs::File::open(&self.args_json).wrap_err("cannot open batch arguments")?;
        let mut input = String::new();
        file.take(MAX_INPUT_BYTES + 1)
            .read_to_string(&mut input)
            .wrap_err("cannot read batch arguments")?;
        if u64::try_from(input.len())? > MAX_INPUT_BYTES {
            bail!("batch arguments exceed the 32 MiB limit");
        }
        let rows: Vec<Vec<u64>> = facet_json::from_str(&input)
            .wrap_err("expected a JSON array of arrays of nonnegative integers")?;
        let mut results = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            cancellation.bail_if_cancelled()?;
            let arguments = row
                .iter()
                .map(|value| natural(*value))
                .collect::<Result<Vec<_>>>()
                .wrap_err_with(|| format!("invalid arguments in row {index}"))?;
            let value = checked
                .evaluate(&self.entry, &arguments)
                .map_err(|error| eyre!("row {index}: {error}"))?;
            results.push(
                read_natural(&value)
                    .wrap_err_with(|| format!("row {index}: function must return Nat"))?,
            );
        }
        Ok(CliOutput::facet(BatchReport { results }))
    }
}

fn natural(value: u64) -> Result<TermRef> {
    if value > MAX_INPUT_NAT {
        bail!("batch Nat input exceeds {MAX_INPUT_NAT}");
    }
    let mut result = term(Term::Ctr {
        name: "Zero".to_owned(),
        args: Vec::new(),
    });
    for _ in 0..value {
        result = term(Term::Ctr {
            name: "Succ".to_owned(),
            args: vec![result],
        });
    }
    Ok(result)
}

fn read_natural(value: &TermRef) -> Result<u64> {
    let mut value = value;
    let mut result = 0u64;
    loop {
        match value.as_ref() {
            Term::Ctr { name, args } if name == "Zero" && args.is_empty() => return Ok(result),
            Term::Ctr { name, args } if name == "Succ" && args.len() == 1 => {
                result = result
                    .checked_add(1)
                    .ok_or_else(|| eyre!("Nat exceeds u64 output range"))?;
                value = &args[0];
            }
            _ => bail!("expected Zero/Succ Nat result"),
        }
    }
}
