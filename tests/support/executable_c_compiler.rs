// SPDX-License-Identifier: MPL-2.0
//! Strict host-C compilation for executable-backend integration tests.
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use std::time::Instant;

static COMPILER: OnceLock<CCompiler> = OnceLock::new();

struct CCompiler {
    program: OsString,
    msvc: bool,
    environment: Vec<(OsString, OsString)>,
}

impl CCompiler {
    fn discover() -> Self {
        if let Some(program) = std::env::var_os("TEAMY_BEND_CC") {
            return Self::configured(program);
        }
        for name in ["cc", "clang", "gcc", "cl"] {
            if Command::new(name).arg("--version").output().is_ok() {
                return Self::configured(name.into());
            }
        }
        if let Some(root) = visual_studio_root() {
            let tools = fs::read_dir(root.join("VC/Tools/MSVC"))
                .expect("read installed MSVC tool versions")
                .map(|entry| entry.expect("MSVC directory entry").path())
                .max()
                .expect("installed MSVC tool version");
            return Self::configured(tools.join("bin/Hostx64/x64/cl.exe").into_os_string());
        }
        panic!("executable C tests require a C11 compiler on PATH or TEAMY_BEND_CC");
    }

    fn configured(program: OsString) -> Self {
        let name = Path::new(&program)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let msvc = name.eq_ignore_ascii_case("cl") || name.eq_ignore_ascii_case("clang-cl");
        let mut environment = Vec::new();
        if msvc && std::env::var_os("INCLUDE").is_none() {
            let root = visual_studio_root().expect("MSVC needs its developer environment");
            let setup = root.join("VC/Auxiliary/Build/vcvars64.bat");
            let setup = setup.to_string_lossy();
            assert!(!setup.contains(['"', '\r', '\n', '%']));
            let mut command = Command::new("cmd");
            command.args(["/D", "/S", "/C"]);
            let script = format!(
                "call \"{setup}\" >nul && set INCLUDE && set LIB && set LIBPATH && set PATH"
            );
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.raw_arg(script)
            };
            #[cfg(not(windows))]
            command.arg(script);
            let output = bounded(&mut command, Duration::from_secs(30));
            assert!(output.status.success(), "MSVC environment setup failed");
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if let Some((key, value)) = line.split_once('=')
                    && ["INCLUDE", "LIB", "LIBPATH", "PATH"]
                        .iter()
                        .any(|name| key.eq_ignore_ascii_case(name))
                {
                    environment.push((key.into(), value.into()));
                }
            }
        }
        Self {
            program,
            msvc,
            environment,
        }
    }
}

fn visual_studio_root() -> Option<PathBuf> {
    let installer = PathBuf::from(std::env::var_os("ProgramFiles(x86)")?)
        .join("Microsoft Visual Studio/Installer/vswhere.exe");
    let output = Command::new(installer)
        .args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ])
        .output()
        .ok()?;
    let path = String::from_utf8(output.stdout).ok()?;
    (!path.trim().is_empty()).then(|| PathBuf::from(path.trim()))
}

pub fn compile(directory: &Path, source: &str, definitions: &[&str]) -> PathBuf {
    let file = directory.join("program.c");
    let executable = directory.join(format!("program{}", std::env::consts::EXE_SUFFIX));
    fs::write(&file, source).unwrap();
    let compiler = COMPILER.get_or_init(CCompiler::discover);
    let mut command = Command::new(&compiler.program);
    command
        .current_dir(directory)
        .envs(compiler.environment.iter().cloned());
    if compiler.msvc {
        command.args(["/nologo", "/TC", "/std:c11", "/W4", "/WX"]);
        for definition in definitions {
            command.arg(format!("/D{definition}"));
        }
        command
            .arg(&file)
            .arg(format!("/Fe:{}", executable.display()))
            .arg(format!("/Fo:{}", directory.join("program.obj").display()))
            .args(["/link", "/INCREMENTAL:NO"]);
    } else {
        command.args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic"]);
        for definition in definitions {
            command.arg(format!("-D{definition}"));
        }
        command.arg(&file).arg("-o").arg(&executable).arg("-lm");
        if cfg!(windows) {
            command.arg("-lws2_32");
        } else {
            command.arg("-pthread");
        }
    }
    let output = bounded(&mut command, Duration::from_secs(45));
    assert!(
        output.status.success(),
        "C compilation failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

pub fn bounded(command: &mut Command, timeout: Duration) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout = thread::spawn(move || {
        let mut bytes = Vec::new();
        std::io::BufReader::new(stdout)
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    });
    let stderr = thread::spawn(move || {
        let mut bytes = Vec::new();
        std::io::BufReader::new(stderr)
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    });
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break (status, false);
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            break (child.wait().unwrap(), true);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = Output {
        status,
        stdout: stdout.join().unwrap(),
        stderr: stderr.join().unwrap(),
    };
    assert!(
        !timed_out,
        "C compiler/program exceeded its deadline:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
