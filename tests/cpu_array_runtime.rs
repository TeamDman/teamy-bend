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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-array-runtime")
}

#[test]
fn array_runtime_originals_and_boundaries_preserve_exact_values() {
    let directory =
        std::env::temp_dir().join(format!("teamy-bend-array-runtime-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let artifacts = Artifacts(directory);
    for name in [
        "original/array_fill_fold",
        "original/array_poly_elem",
        "original/array_keep_cow",
        "original/array_copy_pair",
        "controls/wrap-boundaries",
        "controls/string-owners",
        "controls/float-bits",
        "controls/affine-swap",
    ] {
        let path = root().join(format!("{name}.bend"));
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        // Reuse the same checked book; completed runs must not retain mutable
        // array state in the next invocation.
        for _ in 0..2 {
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            assert_eq!(
                checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap(),
                0,
                "{name}"
            );
            assert_eq!(stdout, expected, "{name}");
            assert!(stderr.is_empty(), "{name}: {stderr:?}");
        }
        let generated = compile_executable_c(&checked).unwrap();
        let executable = executable_c_compiler::compile(&artifacts.0, &generated, &[]);
        let js = artifacts.0.join("main.cjs");
        std::fs::write(&js, compile_executable_javascript(&checked).unwrap()).unwrap();
        let mut node =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()));
        node.arg(js);
        for mut command in [Command::new(executable), node] {
            let output = executable_c_compiler::bounded(&mut command, Duration::from_secs(20));
            assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
            assert_eq!(output.stdout, expected, "{name}");
            assert!(output.stderr.is_empty(), "{name}: {output:?}");
        }
    }
}

#[test]
fn arrays_reject_duplicate_owners_wrong_cells_and_copying_nondata_cells() {
    for name in ["affine-owner", "element-type", "nondata-get"] {
        let path = root().join("negative").join(format!("{name}.bend"));
        let loaded = load_executable(path).unwrap();
        assert!(check_executable(&loaded).is_err(), "{name}");
    }
}

#[test]
fn completed_array_programs_release_owned_strings_closures_and_float_cells() {
    let directory =
        std::env::temp_dir().join(format!("teamy-bend-array-owners-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let artifacts = Artifacts(directory);
    for name in ["string-owners", "affine-swap", "float-bits"] {
        let path = root().join("controls").join(format!("{name}.bend"));
        let checked = check_executable(&load_executable(&path).unwrap()).unwrap();
        let expected = std::fs::read(path.with_extension("stdout")).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert_eq!(
            generated
                .matches("static int tb_program_main(void)")
                .count(),
            1
        );
        let mut generated = generated.replace(
            "static int tb_program_main(void)",
            "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
        );
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 ||
      tb_tasks != 0 || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {
    (void)fputs("completed array program retained runtime owners\n", stderr);
    return 91;
  }
  return status;
}
"#,
        );
        let executable = executable_c_compiler::compile(&artifacts.0, &generated, &[]);
        let output =
            executable_c_compiler::bounded(&mut Command::new(executable), Duration::from_secs(20));
        assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
        assert_eq!(output.stdout, expected, "{name}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
    }
}
