// SPDX-License-Identifier: MPL-2.0
use super::compile_output_cli::OutputPlan;
use super::compile_output_cli::Stage;
use super::compile_process_cli::run_process;
use super::compile_toolchain_cli::Toolchain;
use eyre::Context;
use eyre::Result;
use std::fs;
use std::process::Command;
use teamy_cancellation::CancellationToken;

/// Build in the output filesystem, prebuild CUDA without main, then install.
pub(super) fn compile_native(
    source: &str,
    output: &OutputPlan,
    cancellation: &CancellationToken,
) -> Result<(usize, Option<String>)> {
    let stage = Stage::new(&output.output)?;
    let input = stage.0.join("program.c");
    let binary = stage
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    fs::write(&input, source).wrap_err("cannot stage generated C source")?;
    let toolchain = Toolchain::discover(cancellation)?;
    toolchain.compile(&input, &binary, cancellation)?;
    cancellation.bail_if_cancelled()?;
    if output.cache.is_some() {
        run_process(
            Command::new(&binary)
                .arg("--gpu-build")
                .current_dir(&stage.0),
            "GPU program build",
            cancellation,
        )?;
    }
    cancellation.bail_if_cancelled()?;
    output.validate()?;
    let mut cache_name = binary.as_os_str().to_owned();
    cache_name.push(".gpu");
    let staged_cache = std::path::PathBuf::from(cache_name);
    // Install a valid cache first. If installing the binary then fails, the
    // previous binary can reject this different cache by its content identity.
    let cache = if let Some(destination) = &output.cache {
        if staged_cache.is_file() {
            output.install(&staged_cache, destination)?;
            Some(destination.to_string_lossy().into_owned())
        } else {
            None // Upstream --gpu-build succeeds without an available GPU.
        }
    } else {
        None
    };
    let bytes = usize::try_from(
        fs::metadata(&binary)
            .wrap_err("native compiler produced no executable")?
            .len(),
    )?;
    output.install(&binary, &output.output)?;
    Ok((bytes, cache))
}
