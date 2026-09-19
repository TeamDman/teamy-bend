// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::compiler::compile_c;
use crate::compiler::compile_javascript;
use crate::syntax;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use eyre::eyre;
use facet::Facet;
use figue as args;
use std::io::Write;
use teamy_cancellation::CancellationToken;

/// Generated source language for a standalone program.
#[derive(Facet, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum CompileTarget {
    #[default]
    Javascript,
    C,
}

/// Compile checked pure Bend source into a standalone program.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CompileArgs {
    /// Bend source file.
    #[facet(args::positional)]
    pub file: String,
    /// Closed data entry point, defaulting to main.
    #[facet(args::named)]
    pub entry: Option<String>,
    /// Output source language: javascript (default) or c.
    #[facet(args::named, default)]
    pub target: CompileTarget,
    /// Output source file to create.
    #[facet(args::named)]
    pub output: String,
    /// Replace an existing output file after successful checking.
    #[facet(args::named, default)]
    pub force: bool,
}

#[derive(Debug, Facet)]
struct CompileReport {
    output: String,
    target: String,
    bytes: usize,
}

impl CompileArgs {
    /// Validate source and write the compiled program.
    ///
    /// # Errors
    /// Returns source, proof, compilation, output or cancellation errors.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let book =
            syntax::load(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
        let entry = self.entry.as_deref().unwrap_or("main");
        let (target, program) = match self.target {
            CompileTarget::Javascript => ("javascript", compile_javascript(&book, entry)),
            CompileTarget::C => ("c", compile_c(&book, entry)),
        };
        let program = program.map_err(|error| eyre!("{error}"))?;
        cancellation.bail_if_cancelled()?;
        let mut options = std::fs::OpenOptions::new();
        options.write(true);
        if self.force {
            options.create(true).truncate(true);
        } else {
            options.create_new(true);
        }
        let mut output = options
            .open(&self.output)
            .wrap_err("cannot create compiled output (use --force to replace an existing file)")?;
        output
            .write_all(program.as_bytes())
            .wrap_err("cannot write compiled output")?;
        Ok(CliOutput::facet(CompileReport {
            output: self.output,
            target: target.to_owned(),
            bytes: program.len(),
        }))
    }
}
