pub mod base;
pub mod batch;
pub mod cache;
pub mod check;
pub mod compile;
pub mod eval;
pub mod facet_shape;
pub mod global_args;
pub mod home;
pub mod output;
pub mod run;
pub mod serve;

use crate::cli::base::BaseArgs;
use crate::cli::batch::BatchArgs;
use crate::cli::cache::CacheArgs;
use crate::cli::check::CheckArgs;
use crate::cli::compile::CompileArgs;
use crate::cli::eval::EvalArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::home::HomeArgs;
use crate::cli::output::CliOutput;
use crate::cli::run::RunArgs;
use crate::cli::serve::ServeArgs;
use arbitrary::Arbitrary;
use eyre::Context;
use facet::Facet;
use figue::FigueBuiltins;
use figue::{self as args};
use teamy_cancellation::CancellationToken;

/// Check Bend laws and evaluate pure Bend programs with a native Rust kernel.
///
///
/// Environment variables:
/// - `TEAMY_BEND_HOME_DIR` overrides the resolved application home directory.
/// - `TEAMY_BEND_CACHE_DIR` overrides the resolved cache directory.
/// - `RUST_LOG` provides a tracing filter when `--log-filter` is omitted.
#[derive(Facet, Arbitrary, Debug)]
pub struct Cli {
    /// Global arguments (`debug`, `log_filter`, `log_file`).
    #[facet(flatten)]
    pub global_args: GlobalArgs,

    /// Standard CLI options (help, version, completions).
    #[facet(flatten)]
    #[arbitrary(default)]
    pub builtins: FigueBuiltins,

    /// The command to run.
    #[facet(args::subcommand)]
    pub command: Command,
}

impl PartialEq for Cli {
    fn eq(&self, other: &Self) -> bool {
        // Ignore builtins in comparison since FigueBuiltins doesn't implement PartialEq
        self.global_args == other.global_args && self.command == other.command
    }
}

impl Cli {
    /// # Errors
    ///
    /// This function will return an error if the tokio runtime cannot be built or if the command fails.
    pub fn invoke(self, cancellation_token: CancellationToken) -> eyre::Result<CliOutput> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .wrap_err("Failed to build tokio runtime")?;
        runtime.block_on(async move { self.command.invoke(cancellation_token).await })
    }
}

/// Bend checking, evaluation and application directories.
///
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    /// Print the bundled Base source, its types, or a named namespace.
    Base(BaseArgs),
    /// Cache-related commands.
    Cache(CacheArgs),
    /// Home-related commands.
    Home(HomeArgs),
    /// Check every definition and require all laws to have valid proofs.
    Check(CheckArgs),
    /// Evaluate a closed definition from a checked Bend source file.
    Eval(EvalArgs),
    /// Run main with executable foreign contracts and native console effects.
    Run(RunArgs),
    /// Evaluate rows of natural-number arguments against one checked program.
    Batch(BatchArgs),
    /// Compile checked pure data or executable contracts to a standalone program.
    Compile(CompileArgs),
    /// Serve typed constructor calls over persistent newline-delimited JSON.
    Serve(ServeArgs),
}

impl Command {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(self, cancellation_token: CancellationToken) -> eyre::Result<CliOutput> {
        cancellation_token.bail_if_cancelled()?;
        match self {
            Command::Base(args) => args.invoke(&cancellation_token),
            Command::Cache(args) => args.invoke().await,
            Command::Home(args) => args.invoke().await,
            Command::Check(args) => args.invoke(&cancellation_token),
            Command::Eval(args) => args.invoke(&cancellation_token),
            Command::Run(args) => args.invoke(&cancellation_token),
            Command::Batch(args) => args.invoke(&cancellation_token),
            Command::Compile(args) => args.invoke(&cancellation_token),
            Command::Serve(args) => args.invoke(&cancellation_token),
        }
    }
}
