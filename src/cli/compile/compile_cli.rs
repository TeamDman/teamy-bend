// SPDX-License-Identifier: MPL-2.0
use crate::cli::output::CliOutput;
use crate::compiler::compile_c;
use crate::compiler::compile_executable_javascript;
use crate::compiler::compile_javascript;
use crate::kernel::check_executable;
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

/// Compile checked Bend source into a standalone program.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CompileArgs {
    /// Bend source file.
    #[facet(args::positional)]
    pub file: String,
    /// Closed data entry point; executable mode requires main.
    #[facet(args::named)]
    pub entry: Option<String>,
    /// Output source language: javascript (default) or c.
    #[facet(args::named, default)]
    pub target: CompileTarget,
    /// Compile executable contracts and IO to JavaScript instead of pure data.
    #[facet(args::named, default)]
    pub executable: bool,
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
    socket_provider: Option<String>,
}

impl CompileArgs {
    /// Validate source and write the compiled program.
    ///
    /// # Errors
    /// Returns source, proof, compilation, output or cancellation errors.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let entry = self.entry.as_deref().unwrap_or("main");
        let (target, program) = if self.executable {
            if entry != "main" {
                return Err(eyre!("executable compilation requires the main entry"));
            }
            if self.target != CompileTarget::Javascript {
                return Err(eyre!(
                    "executable compilation currently supports JavaScript only"
                ));
            }
            let source = syntax::load_executable(std::path::Path::new(&self.file))
                .map_err(|error| eyre!("{error}"))?;
            let checked = check_executable(&source).map_err(|error| eyre!("{error}"))?;
            ("javascript", compile_executable_javascript(&checked))
        } else {
            let book =
                syntax::load(std::path::Path::new(&self.file)).map_err(|error| eyre!("{error}"))?;
            match self.target {
                CompileTarget::Javascript => ("javascript", compile_javascript(&book, entry)),
                CompileTarget::C => ("c", compile_c(&book, entry)),
            }
        };
        let program = program.map_err(|error| eyre!("{error}"))?;
        cancellation.bail_if_cancelled()?;
        if !self.force && std::path::Path::new(&self.output).exists() {
            return Err(eyre!(
                "cannot create compiled output (use --force to replace an existing file)"
            ));
        }
        let socket_provider = if self.executable {
            install_socket_provider(std::path::Path::new(&self.output))?
        } else {
            None
        };
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
            socket_provider,
        }))
    }
}

/// Package a built provider without putting this machine's location into source.
/// A different existing provider may be shared by other generated programs, so
/// replacing the requested source file does not silently replace that library.
fn install_socket_provider(output: &std::path::Path) -> Result<Option<String>> {
    if output
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("teamy-bend-sys.node"))
    {
        return Err(eyre!("compiled source cannot replace the socket provider"));
    }
    let executable = std::env::current_exe().wrap_err("cannot locate compiler executable")?;
    let directory = executable
        .parent()
        .ok_or_else(|| eyre!("compiler has no directory"))?;
    let library = if cfg!(windows) {
        "teamy_bend_sys.dll"
    } else if cfg!(target_os = "macos") {
        "libteamy_bend_sys.dylib"
    } else {
        "libteamy_bend_sys.so"
    };
    let source = directory.join(library);
    if !source.is_file() {
        // Custom BEND_SYS providers and explicit runtime module paths remain
        // valid. Compiling source never loads or executes a host provider.
        return Ok(None);
    }
    let destination = output
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("teamy-bend-sys.node");
    let bytes = std::fs::read(source).wrap_err("cannot read socket provider")?;
    if destination.exists() {
        if std::fs::read(&destination).wrap_err("cannot read existing socket provider")? != bytes {
            return Err(eyre!(
                "existing socket provider differs; choose another output directory or replace the provider explicitly"
            ));
        }
    } else {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .wrap_err("cannot create socket provider")?;
        file.write_all(&bytes)
            .wrap_err("cannot write socket provider")?;
    }
    Ok(Some(destination.to_string_lossy().into_owned()))
}
