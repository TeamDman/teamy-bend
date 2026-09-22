// SPDX-License-Identifier: MPL-2.0
use super::compile_native_cli::compile_native;
use super::compile_output_cli::OutputPlan;
use super::compile_output_cli::Stage;
use crate::cli::output::CliOutput;
use crate::compiler::compile_c;
use crate::compiler::compile_executable_c;
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
use std::path::Path;
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;

/// Generated source language or native binary for a standalone program.
#[derive(Facet, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum CompileTarget {
    #[default]
    Javascript,
    C,
    Native,
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
    /// Output target: javascript (default), c source, or a native binary.
    #[facet(args::named, default)]
    pub target: CompileTarget,
    /// Compile executable contracts and IO instead of pure data.
    #[facet(args::named, default)]
    pub executable: bool,
    /// Output source file or native binary to create.
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
    gpu_artifact: Option<String>,
}

impl CompileArgs {
    /// Validate source and write the compiled program.
    ///
    /// # Errors
    /// Returns source, proof, compilation, output or cancellation errors.
    pub fn invoke(self, cancellation: &CancellationToken) -> Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let (program, inputs) = self.generate()?;
        cancellation.bail_if_cancelled()?;
        // Reserved emitter preamble contract: inert marks without an emitted
        // device program do not trigger a prebuild, nor does strict pure C.
        let gpu = self.executable
            && self.target == CompileTarget::Native
            && program
                .lines()
                .any(|line| line == "#define TB_GPU_ENABLED 1");
        let output = OutputPlan::new(Path::new(&self.output), self.force, &inputs, gpu)?;
        let socket_provider = if self.executable && self.target == CompileTarget::Javascript {
            install_socket_provider(&output.output)?
        } else {
            None
        };
        let (bytes, gpu_artifact) = if self.target == CompileTarget::Native {
            compile_native(&program, &output, cancellation)?
        } else {
            let stage = Stage::new(&output.output)?;
            let source = stage.0.join("output.source");
            std::fs::write(&source, &program).wrap_err("cannot stage compiled output")?;
            cancellation.bail_if_cancelled()?;
            output.install(&source, &output.output)?;
            (program.len(), None)
        };
        let target = match self.target {
            CompileTarget::Javascript => "javascript",
            CompileTarget::C => "c",
            CompileTarget::Native => "native",
        };
        Ok(CliOutput::facet(CompileReport {
            output: self.output,
            target: target.to_owned(),
            bytes,
            socket_provider,
            gpu_artifact,
        }))
    }

    fn generate(&self) -> Result<(String, Vec<PathBuf>)> {
        let entry = self.entry.as_deref().unwrap_or("main");
        let (program, inputs) = if self.executable {
            if entry != "main" {
                return Err(eyre!("executable compilation requires the main entry"));
            }
            let (source, mut inputs) = syntax::load_executable_with_sources(Path::new(&self.file))
                .map_err(|error| eyre!("{error}"))?;
            let checked = check_executable(&source).map_err(|error| eyre!("{error}"))?;
            for name in checked.foreign_names() {
                if let Some(imports) = checked.foreign_imports(name) {
                    for (target, path) in imports {
                        let selected = if self.target == CompileTarget::Javascript {
                            "js"
                        } else {
                            "c"
                        };
                        if target == selected && path.is_file() {
                            inputs.push(path.to_owned());
                        }
                    }
                }
            }
            let program = match self.target {
                CompileTarget::Javascript => compile_executable_javascript(&checked),
                CompileTarget::C | CompileTarget::Native => compile_executable_c(&checked),
            };
            (program, inputs)
        } else {
            let (book, inputs) = syntax::load_with_sources(Path::new(&self.file))
                .map_err(|error| eyre!("{error}"))?;
            let program = match self.target {
                CompileTarget::Javascript => compile_javascript(&book, entry),
                CompileTarget::C | CompileTarget::Native => compile_c(&book, entry),
            };
            (program, inputs)
        };
        let program = program.map_err(|error| eyre!("{error}"))?;
        Ok((program, inputs))
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
