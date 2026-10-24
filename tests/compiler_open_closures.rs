// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::ExecutableEntry;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

struct Artifacts(PathBuf);

impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn composition_grids_compile_and_original_io_executes_with_exact_outputs() {
    qualify();
}

#[test]
fn aggregate_cancellation_before_evaluation_and_before_output_keeps_writers_empty() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-open-closures/wrappers/open_closure_pipe.bend");
    let checked = check_executable(&load_executable(path).unwrap()).unwrap();
    for cancel_at in [1, 2] {
        let observations = std::cell::Cell::new(0);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = checked
            .run_main(&mut stdout, &mut stderr, &|| {
                observations.set(observations.get() + 1);
                observations.get() == cancel_at
            })
            .unwrap_err();
        assert!(error.to_string().contains("execution cancelled"));
        assert_eq!(observations.get(), cancel_at);
        assert!(stdout.is_empty() && stderr.is_empty());
    }
}

fn qualify() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-open-closures");
    let artifacts = Artifacts(
        std::env::temp_dir().join(format!("teamy-bend-open-closures-{}", std::process::id())),
    );
    std::fs::create_dir(&artifacts.0).unwrap();
    for (relative, entry, expected) in [
        (
            "wrappers/open_closure",
            ExecutableEntry::Pure,
            include_bytes!("fixtures/cpu-open-closures/wrappers/open_closure.stdout").as_slice(),
        ),
        (
            "wrappers/open_closure_pipe",
            ExecutableEntry::Pure,
            include_bytes!("fixtures/cpu-open-closures/wrappers/open_closure_pipe.stdout")
                .as_slice(),
        ),
        (
            "wrappers/stuck_split_mint",
            ExecutableEntry::Pure,
            include_bytes!("fixtures/cpu-open-closures/wrappers/stuck_split_mint.stdout")
                .as_slice(),
        ),
        ("open_closure", ExecutableEntry::Io, b"12\n".as_slice()),
    ] {
        let name = relative;
        let checked =
            check_executable(&load_executable(root.join(format!("{relative}.bend"))).unwrap())
                .unwrap();
        assert_eq!(checked.entry_kind().unwrap(), entry);
        {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0
            );
            assert_eq!(stdout, expected, "{name}: native");
            assert!(stderr.is_empty());
        }
        let c = compile_executable_c(&checked).unwrap();
        let executable = executable_c_compiler::compile(&artifacts.0, &c, &[]);
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(js);
        for mut command in [Command::new(executable), node] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert!(output.status.success(), "{name}: {output:?}");
            assert_eq!(output.stdout, expected, "{name}");
            assert!(output.stderr.is_empty(), "{name}: {output:?}");
        }
    }
}
