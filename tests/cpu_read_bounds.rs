// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn decimal_readers_and_helpers_preserve_bounds_and_file_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-read-bounds");
    let artifact_path =
        std::env::temp_dir().join(format!("teamy-bend-read-bounds-{}", std::process::id()));
    std::fs::create_dir(&artifact_path).unwrap();
    let artifacts = Artifacts(artifact_path);
    let mut paths = Vec::new();
    for directory in ["positive", "negative"] {
        paths.extend(
            std::fs::read_dir(root.join(directory))
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "bend")
                }),
        );
    }
    paths.push(root.join("original/read_bounds.bend"));
    paths.push(root.join("original/show_read.bend"));
    paths.push(root.join("unresolved/word-edges.bend"));
    paths.sort();
    for path in paths {
        let checked = check_executable(
            &load_executable(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
        )
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        let (expected_code, expected_stderr) = if path.parent().unwrap().ends_with("negative") {
            (1_u32, std::fs::read(path.with_extension("stderr")).unwrap())
        } else {
            (0_u32, Vec::new())
        };
        // Reuse the checked book after completion and verify invocation
        // isolation through identical output and status.
        for _ in 0..2 {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display())),
                expected_code
            );
            assert_eq!(stdout, expected, "{}", path.display());
            assert_eq!(stderr, expected_stderr, "{}", path.display());
        }
        let executable = executable_c_compiler::compile(
            &artifacts.0,
            &compile_executable_c(&checked).unwrap(),
            &[],
        );
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(js);
        for mut command in [Command::new(executable), node] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(
                output.status.code(),
                Some(i32::try_from(expected_code).unwrap()),
                "{}: {output:?}",
                path.display()
            );
            assert_eq!(output.stdout, expected, "{}", path.display());
            assert_eq!(output.stderr, expected_stderr, "{output:?}");
        }
    }
}
