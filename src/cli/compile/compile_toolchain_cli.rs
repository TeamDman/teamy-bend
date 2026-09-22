// SPDX-License-Identifier: MPL-2.0
#[cfg(windows)]
use super::compile_process_cli::capture_process;
use super::compile_process_cli::run_process;
use eyre::Context;
use eyre::Result;
use eyre::eyre;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use teamy_cancellation::CancellationToken;

#[derive(Debug)]
pub(super) struct Toolchain {
    program: PathBuf,
    msvc: bool,
    environment: Vec<(OsString, OsString)>,
}

impl Toolchain {
    pub fn discover(cancellation: &CancellationToken) -> Result<Self> {
        for variable in ["TEAMY_BEND_CC", "CC"] {
            if let Some(program) = std::env::var_os(variable) {
                let path = resolve_program(&program).ok_or_else(|| eyre!(
                    "{variable} must name an available C compiler executable (arguments are not shell-expanded)"
                ))?;
                return Self::configure(path, cancellation);
            }
        }
        let candidates: &[&str] = if cfg!(windows) {
            &["cl", "clang-cl", "clang", "gcc", "cc"]
        } else {
            &["cc", "clang", "gcc"]
        };
        for candidate in candidates {
            if let Some(path) = resolve_program(OsStr::new(candidate)) {
                return Self::configure(path, cancellation);
            }
        }
        #[cfg(windows)]
        if let Some(root) = visual_studio_root(cancellation)? {
            let directory = root.join("VC/Tools/MSVC");
            let tools = std::fs::read_dir(directory)
                .wrap_err("cannot inspect MSVC tool versions")?
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| path.join("bin/Hostx64/x64/cl.exe").is_file())
                .max()
                .ok_or_else(|| eyre!("Visual Studio has no installed x64 C compiler"))?;
            return Self::configure(tools.join("bin/Hostx64/x64/cl.exe"), cancellation);
        }
        Err(eyre!(
            "native compilation requires a C11 compiler; set TEAMY_BEND_CC or CC"
        ))
    }

    fn configure(program: PathBuf, cancellation: &CancellationToken) -> Result<Self> {
        cancellation.bail_if_cancelled()?;
        let name = program.file_stem().unwrap_or_default().to_string_lossy();
        let msvc = name.eq_ignore_ascii_case("cl") || name.eq_ignore_ascii_case("clang-cl");
        let environment = Vec::new();
        #[cfg(windows)]
        let environment = if std::env::var_os("INCLUDE").is_none()
            && (msvc || name.eq_ignore_ascii_case("clang"))
        {
            msvc_environment(cancellation)?
        } else {
            environment
        };
        Ok(Self {
            program,
            msvc,
            environment,
        })
    }

    pub fn compile(
        &self,
        source: &Path,
        output: &Path,
        cancellation: &CancellationToken,
    ) -> Result<()> {
        let directory = source
            .parent()
            .ok_or_else(|| eyre!("staged source directory missing"))?;
        if output.parent() != Some(directory) {
            return Err(eyre!(
                "native compiler inputs must share the owned staging directory"
            ));
        }
        // MSVC does not accept canonical Windows extended paths as source
        // arguments. These files belong to the same owned staging directory.
        let source = source
            .file_name()
            .ok_or_else(|| eyre!("staged source name missing"))?;
        let output = output
            .file_name()
            .ok_or_else(|| eyre!("staged executable name missing"))?;
        let mut command = Command::new(&self.program);
        command
            .current_dir(directory)
            .envs(self.environment.iter().cloned());
        if self.msvc {
            command
                .args(["/nologo", "/TC", "/std:c11", "/O2", "/W4", "/WX"])
                .arg(source)
                .arg(prefixed_argument("/Fe:", output))
                .arg("/Fo:program.obj")
                .args(["/link", "/INCREMENTAL:NO"]);
        } else {
            command
                .args([
                    "-std=c11",
                    "-O3",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-pedantic",
                ])
                .arg(source)
                .arg("-o")
                .arg(output)
                .arg("-lm");
            if cfg!(windows) {
                command.arg("-lws2_32");
            } else {
                command.arg("-pthread");
            }
            if cfg!(target_os = "linux") {
                command.arg("-ldl");
            }
        }
        run_process(&mut command, "native C compilation", cancellation)
    }
}

fn prefixed_argument(prefix: &str, value: &OsStr) -> OsString {
    let mut argument = OsString::from(prefix);
    argument.push(value);
    argument
}

fn resolve_program(program: &OsStr) -> Option<PathBuf> {
    let path = Path::new(program);
    if path.components().count() > 1 {
        return std::fs::canonicalize(path)
            .ok()
            .filter(|path| path.is_file());
    }
    let search = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&search) {
        let candidate = directory.join(program);
        if candidate.is_file() {
            return std::fs::canonicalize(candidate).ok();
        }
        if cfg!(windows) && candidate.extension().is_none() {
            let candidate = candidate.with_extension("exe");
            if candidate.is_file() {
                return std::fs::canonicalize(candidate).ok();
            }
        }
    }
    None
}

#[cfg(windows)]
fn visual_studio_root(cancellation: &CancellationToken) -> Result<Option<PathBuf>> {
    let Some(root) = std::env::var_os("ProgramFiles(x86)") else {
        return Ok(None);
    };
    let installer = PathBuf::from(root).join("Microsoft Visual Studio/Installer/vswhere.exe");
    if !installer.is_file() {
        return Ok(None);
    }
    let result = capture_process(
        Command::new(installer).args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ]),
        cancellation,
    )?;
    if !result.status.success() {
        return Err(eyre!("Visual Studio discovery failed"));
    }
    let path = String::from_utf8(result.stdout).wrap_err("invalid Visual Studio path encoding")?;
    Ok((!path.trim().is_empty()).then(|| PathBuf::from(path.trim())))
}

#[cfg(windows)]
fn msvc_environment(cancellation: &CancellationToken) -> Result<Vec<(OsString, OsString)>> {
    use std::os::windows::process::CommandExt;
    let root = visual_studio_root(cancellation)?.ok_or_else(|| {
        eyre!("MSVC needs its developer environment or an installed Visual Studio C++ toolchain")
    })?;
    let setup = root.join("VC/Auxiliary/Build/vcvars64.bat");
    let setup = setup
        .to_str()
        .ok_or_else(|| eyre!("MSVC setup path is not Unicode"))?;
    if setup.contains(['"', '\r', '\n', '%']) {
        return Err(eyre!("unsupported MSVC setup path"));
    }
    let mut command = Command::new("cmd");
    command.args(["/D", "/V:OFF", "/S", "/C"]);
    // The only shell fragment is the validated installed setup script. Source,
    // output and compiler overrides are always separate process arguments.
    command.raw_arg(format!(
        "call \"{setup}\" >nul && set INCLUDE && set LIB && set LIBPATH && set PATH"
    ));
    let output = capture_process(&mut command, cancellation)?;
    if !output.status.success() {
        return Err(eyre!("MSVC environment setup failed"));
    }
    let text = String::from_utf8(output.stdout).wrap_err("invalid MSVC environment encoding")?;
    Ok(text
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            ["INCLUDE", "LIB", "LIBPATH", "PATH"]
                .iter()
                .any(|name| key.eq_ignore_ascii_case(name))
                .then(|| (key.into(), value.into()))
        })
        .collect())
}
