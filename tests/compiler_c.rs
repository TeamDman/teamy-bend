// SPDX-License-Identifier: MPL-2.0

use facet::Facet;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_c;
use teamy_bend::compiler::compile_javascript;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::term;
use teamy_bend::protocol::DataValue;
use teamy_bend::syntax::parse;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
const ADD: &str = "def add(a: Nat, b: Nat) -> Nat:\n  match a:\n    case Zero{}: b\n    case Succ{p}: Succ{add(p, b)}\n";
static NEXT: AtomicUsize = AtomicUsize::new(0);
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
        panic!("C runtime tests require a C11 compiler on PATH or TEAMY_BEND_CC");
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
            // Read only compiler environment variables, not the complete environment.
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
            let output = command.output().expect("load MSVC developer environment");
            assert!(
                output.status.success(),
                "MSVC environment setup failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
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

struct Program {
    source: PathBuf,
    executable: PathBuf,
    object: PathBuf,
    javascript: PathBuf,
}

impl Program {
    fn compile(source: &str, limits: &[&str]) -> Self {
        let base = std::env::temp_dir().join(format!(
            "teamy-bend-c-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let program = Self {
            source: base.with_extension("c"),
            executable: base.with_extension(std::env::consts::EXE_EXTENSION),
            object: base.with_extension("obj"),
            javascript: base.with_extension("cjs"),
        };
        fs::write(&program.source, source).expect("write generated C fixture");
        let compiler = COMPILER.get_or_init(CCompiler::discover);
        let mut command = Command::new(&compiler.program);
        command.envs(compiler.environment.iter().cloned());
        if compiler.msvc {
            command.args(["/nologo", "/TC", "/std:c11", "/W4", "/WX"]);
            for limit in limits {
                command.arg(format!("/D{limit}"));
            }
            command
                .arg(&program.source)
                .arg(format!("/Fe:{}", program.executable.display()))
                .arg(format!("/Fo:{}", program.object.display()))
                .args(["/link", "/INCREMENTAL:NO"]);
        } else {
            command.args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic"]);
            for limit in limits {
                command.arg(format!("-D{limit}"));
            }
            command
                .arg(&program.source)
                .arg("-o")
                .arg(&program.executable);
        }
        let output = command.output().expect("execute installed C compiler");
        assert!(
            output.status.success(),
            "C compilation failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        program
    }

    fn run(&self) -> Output {
        Command::new(&self.executable)
            .output()
            .expect("execute compiled C fixture")
    }

    fn javascript_output(&self, book: &Book, entry: &str) -> Output {
        fs::write(&self.javascript, compile_javascript(book, entry).unwrap()).unwrap();
        let node = std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into());
        Command::new(node)
            .arg(&self.javascript)
            .output()
            .expect("execute JS comparison fixture")
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        for file in [
            &self.source,
            &self.executable,
            &self.object,
            &self.javascript,
        ] {
            let _result = fs::remove_file(file);
        }
    }
}

#[derive(Debug, Facet)]
struct Printed {
    value: DataValue,
}

fn assert_matches(book: &Book, entry: &str) {
    let checked = check_book(book).unwrap();
    let normal = checked.evaluate(entry, &[]).unwrap();
    let expected = DataValue::from_term(&normal).unwrap();
    let program = Program::compile(&compile_c(book, entry).unwrap(), &[]);
    let output = program.run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed: Printed =
        facet_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(printed.value, expected);
    let javascript = program.javascript_output(book, entry);
    assert!(javascript.status.success());
    let printed_js: Printed =
        facet_json::from_str(&String::from_utf8(javascript.stdout).unwrap()).unwrap();
    assert_eq!(printed.value, printed_js.value);
}

#[test]
fn c_recursive_arithmetic_matches_normalizer_and_javascript() {
    let book = parse(&format!("{NAT}{ADD}\ndef main() -> Nat: add(2n, 3n)\n")).unwrap();
    assert_matches(&book, "main");
}

#[test]
fn c_closures_preserve_lexical_capture() {
    let book = parse(&format!(
        "{NAT}{ADD}{}",
        r"
def make_adder(n: Nat) -> @x: Nat -> Nat: x => add(n, x)
def apply_function(f: @x: Nat -> Nat, n: Nat) -> Nat: f(n)
def main() -> Nat: apply_function(make_adder(3n), 4n)
"
    ))
    .unwrap();
    assert_matches(&book, "main");
}

#[test]
fn c_nested_patterns_and_fallback_preserve_constructor_fields() {
    let book = parse(&format!(
        "{NAT}{}",
        r"
def pick(n: Nat) -> Nat:
  match n:
    case Zero{}: 9n
    case Succ{Zero{}}: 8n
    case Succ{Succ{p}}: p
def main() -> Nat: pick(5n)
"
    ))
    .unwrap();
    assert_matches(&book, "main");
}

#[test]
fn c_parallel_let_right_sides_use_the_outer_environment() {
    let book = parse(&format!(
        "{NAT}{}",
        r"
def swap(a: Nat, b: Nat) -> Nat:
  a b = b a
  a
def main() -> Nat: swap(2n, 5n)
"
    ))
    .unwrap();
    assert_matches(&book, "main");
}

#[test]
fn c_ignores_unused_arguments_and_constructor_fields() {
    let book = parse(&format!(
        "{NAT}{}",
        r"
type Pair is Data:
  Pair{first: Nat, second: Nat}
def huge() -> Nat: 97n
def first(pair: Pair) -> Nat:
  match pair:
    case Pair{a, b}: a
def discard(x: Nat) -> Nat: 1n
def main() -> Nat: first(Pair{discard(huge()), huge()})
"
    ))
    .unwrap();
    let program = Program::compile(&compile_c(&book, "main").unwrap(), &["BEND_MAX_DEPTH=32"]);
    let output = program.run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Succ"));
    let huge = Program::compile(&compile_c(&book, "huge").unwrap(), &["BEND_MAX_DEPTH=32"]);
    let output = huge.run();
    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "failed output must not emit partial JSON"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("nesting limit"));
}

#[test]
fn c_checks_proofs_before_emitting_an_explicit_erased_marker() {
    let book = parse(&format!(
        "{NAT}\nlaw main:\n  {{0n == 0n : Nat}}\ndef main(): {{==}}\n"
    ))
    .unwrap();
    let program = Program::compile(&compile_c(&book, "main").unwrap(), &[]);
    let output = program.run();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "{\"value\":{\"erased\":true}}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&program.javascript_output(&book, "main").stdout).trim()
    );
}

#[test]
fn c_rejects_false_unused_proofs_before_emission() {
    let book = parse(&format!(
        "{NAT}\nlaw bad:\n  {{0n == 1n : Nat}}\ndef bad(): {{==}}\ndef main() -> Nat: 0n\n"
    ))
    .unwrap();
    assert!(
        compile_c(&book, "main")
            .unwrap_err()
            .to_string()
            .contains("equal endpoints")
    );
}

#[test]
fn c_rejects_missing_open_and_incomplete_entries() {
    let book = parse(&format!("{NAT}{ADD}")).unwrap();
    assert!(
        compile_c(&book, "add")
            .unwrap_err()
            .to_string()
            .contains("requires 2 arguments")
    );
    assert!(
        compile_c(&book, "missing")
            .unwrap_err()
            .to_string()
            .contains("undefined entry")
    );
    let incomplete = parse(&format!("{NAT}\ndef main() -> Nat: ?missing\n")).unwrap();
    compile_c(&incomplete, "main").unwrap_err();
}

#[test]
fn c_function_results_fail_without_partial_json() {
    let book = parse(&format!("{NAT}\ndef main() -> @x: Nat -> Nat: x => x\n")).unwrap();
    let program = Program::compile(&compile_c(&book, "main").unwrap(), &[]);
    let output = program.run();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("standalone entry must return data"));
}

#[test]
fn c_names_are_length_delimited_utf8_data_not_source_code() {
    let mut book = parse("type Token is Data:\n  Safe{}\ndef main() -> Token: Safe{}\n").unwrap();
    let name = "\"); exit(99); /*\\\n\0é🦀";
    let entry = "entry\"\n); exit(88);";
    for declaration in &mut book.declarations {
        match declaration {
            Declaration::Adt(datatype) => datatype.constructors[0].name = name.to_owned(),
            Declaration::Def(definition) => {
                definition.name = entry.to_owned();
                definition.body = Some(term(Term::Ctr {
                    name: name.to_owned(),
                    args: vec![],
                }));
            }
        }
    }
    assert_matches(&book, entry);
}

#[test]
fn c_runtime_budgets_fail_cleanly_and_deterministically() {
    let book = parse(&format!("{NAT}\ndef main() -> Nat: 8n\n")).unwrap();
    let source = compile_c(&book, "main").unwrap();
    assert_eq!(source, compile_c(&book, "main").unwrap());
    for (limit, message) in [
        ("BEND_MAX_STEPS=2", "evaluation budget"),
        ("BEND_MAX_ALLOC=64", "allocation budget"),
        ("BEND_MAX_OUTPUT=16", "output budget"),
    ] {
        let program = Program::compile(&source, &[limit]);
        let output = program.run();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    }
}

#[test]
fn compiled_templates_preserve_recursion_shadowing_and_reusable_arguments() {
    let book = parse(&format!(
        "{NAT}{ADD}{}",
        r"
type Nats is Data:
  Nil{}
  Con{head: Nat, tail: Nats}
type Report is Data:
  Report{mapped: Nats, outer: Nat, inner: Nat, duplicated: Nat}
def map(~f: Nat -> Nat, xs: Nats) -> Nats:
  match xs:
    case Nil{}: Nil{}
    case Con{head, tail}: Con{f(head), map(~f, tail)}
def run(~f: Nat -> Nat -> Nat, a: Nat, b: Nat) -> Nat: f(a, b)
def outer(~f: Nat -> Nat -> Nat, a: Nat, b: Nat) -> Nat: run(~(x => f(x)), a, b)
def twice(~f: Nat -> Nat, +x: Nat) -> Nat: f(x)
def main() -> Report:
  Report{
    map(~(n => Succ{n}), Con{1n, Con{2n, Nil{}}}),
    outer(~(y => x => y), 1n, 2n),
    run(~(x => x => x), 1n, 2n),
    twice(~(y => add(y, y)), 3n)
  }
"
    ))
    .expect("closed template program parses");
    let checked = check_book(&book).expect("template instances remain checked");
    let expected = "Report{Con{Succ{Succ{Zero{}}}, Con{Succ{Succ{Succ{Zero{}}}}, Nil{}}}, Succ{Zero{}}, Succ{Succ{Zero{}}}, Succ{Succ{Succ{Succ{Succ{Succ{Zero{}}}}}}}}";
    assert_eq!(checked.evaluate("main", &[]).unwrap().to_string(), expected);
    assert_eq!(
        checked.evaluate_data("main", &[]).unwrap().to_string(),
        expected
    );
    assert_matches(&book, "main");
}

#[test]
fn compiled_arrays_preserve_wrapped_updates_and_template_mapping() {
    let book = parse(&format!(
        "{}\n{}",
        include_str!("../src/syntax/base.bend"),
        r"
def increment(n: Nat) -> Nat: 1n+n
def finish(result: Array<Nat> & Nat) -> List<Nat> & Nat:
  (array, old) = result
  (Array.to_list(~Nat, Array.map(~Nat, ~Nat, ~increment, array)), old)
def main() -> List<Nat> & Nat:
  finish(Array.swap(Nat, Array.new(Nat, 2n, 1n), 5, 8n))
"
    ))
    .expect("bundled array program parses");
    let expected = teamy_bend::syntax::parse_term("([2n,9n,2n,2n],1n)").unwrap();
    let checked = check_book(&book).expect("all ordinary library definitions check");
    assert_eq!(
        checked.evaluate_data("main", &[]).unwrap().to_string(),
        expected.to_string()
    );
    assert_matches(&book, "main");
}
