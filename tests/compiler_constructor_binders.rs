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
fn constructor_binder_observations_and_original_io_agree_in_native_c_and_javascript() {
    qualify();
}

fn qualify() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-binders");
    let artifacts = Artifacts(std::env::temp_dir().join(format!(
        "teamy-bend-constructor-binders-{}",
        std::process::id()
    )));
    std::fs::create_dir(&artifacts.0).unwrap();
    for (relative, entry, expected) in [
        (
            "wrappers/printer_fixpoint",
            ExecutableEntry::Pure,
            include_bytes!("fixtures/cpu-constructor-binders/wrappers/printer_fixpoint.stdout")
                .as_slice(),
        ),
        (
            "wrappers/printer_fresh_binder",
            ExecutableEntry::Pure,
            include_bytes!("fixtures/cpu-constructor-binders/wrappers/printer_fresh_binder.stdout")
                .as_slice(),
        ),
        (
            "wrappers/printer_fresh_binder_pick",
            ExecutableEntry::Pure,
            include_bytes!(
                "fixtures/cpu-constructor-binders/wrappers/printer_fresh_binder_pick.stdout"
            )
            .as_slice(),
        ),
        (
            "printer_fresh_binder",
            ExecutableEntry::Io,
            b"2\n".as_slice(),
        ),
    ] {
        let name = relative;
        let checked =
            check_executable(&load_executable(root.join(format!("{relative}.bend"))).unwrap())
                .unwrap();
        assert_eq!(checked.entry_kind().unwrap(), entry);
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
