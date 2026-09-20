// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;
use teamy_bend::syntax::parse;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-opaque-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("main.bend"), source).unwrap();
        Self(directory)
    }

    fn path(&self) -> PathBuf {
        self.0.join("main.bend")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn channel_family_is_an_opaque_executable_contract_and_helpers_are_checked() {
    let fixture = Fixture::new("import Base\n");
    let checked = check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    assert_eq!(checked.opaque_names().collect::<Vec<_>>(), ["Chan", "File"]);
    assert_eq!(
        checked.definition_type("Chan").unwrap().to_string(),
        "@-A:Type -> Data"
    );
    for name in ["Chan.new", "Chan.send", "Chan.recv", "Chan.close"] {
        assert!(checked.foreign_names().any(|foreign| foreign == name));
    }
    for name in ["IO.fork.go", "IO.fork", "IO.join.go", "IO.join"] {
        assert!(checked.definition_type(name).is_some());
        assert!(!checked.foreign_names().any(|foreign| foreign == name));
    }
}

#[test]
fn strict_base_and_open_law_policy_are_unchanged() {
    let fixture = Fixture::new("import Base\n");
    let strict = load(fixture.path()).unwrap();
    check_book(&strict).unwrap();
    assert!(!strict.declarations.iter().any(|declaration| {
        matches!(declaration, teamy_bend::kernel::Declaration::Def(definition) if definition.name == "Chan")
    }));
    let law = parse("law Chan:\n  for -A: Type\n  Data\n").unwrap();
    let error = check_book(&law).unwrap_err().to_string();
    assert!(error.contains("unfilled laws: Chan"), "{error}");
    for body in [
        "law promise: {0 == 1 : U32}\n",
        "law false: {0 == 1 : U32}\ndef false(): {==}\n",
        "law future: U32\ndef premature() -> U32: future\ndef future(): 0\n",
    ] {
        let fixture = Fixture::new(&format!("import Base\n{body}"));
        let source = load_executable(fixture.path()).unwrap();
        assert!(
            check_executable(&source).is_err(),
            "ordinary law unexpectedly accepted: {body}"
        );
    }
}

#[test]
fn channel_handles_are_reusable_but_affine_payloads_are_not() {
    let fixture = Fixture::new(
        "import Base\ntype Token is Type: Token{}\ndef duplicate(+channel: Chan(Token)) -> Chan(Token) & Chan(Token): (channel, channel)\n",
    );
    check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    let fixture = Fixture::new(
        r"import Base
type Token is Type: Token{}
def duplicate(+channel: Chan(Token), payload: Token) -> IO(Bool):
  IO.bind(Bool, Bool, Chan.send(Token, channel, payload), sent => Chan.send(Token, channel, payload))
",
    );
    let error = check_executable(&load_executable(fixture.path()).unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("binder payload permits Lone use, observed Many"),
        "{error}"
    );
}

#[test]
fn opaque_handles_cannot_be_constructed_or_filled_by_source() {
    let fixture = Fixture::new("import Base\ndef fake() -> Chan(U32): Unit{}\n");
    check_executable(&load_executable(fixture.path()).unwrap())
        .expect_err("opaque handles have no constructors");
    let fixture = Fixture::new("import Base\ndef Chan(A): U32\n");
    if let Ok(source) = load_executable(fixture.path()) {
        check_executable(&source).expect_err("opaque contracts cannot be filled by source");
    }
    let fixture = Fixture::new("law Chan:\n  for -A: Type\n  Data\n");
    let error = check_executable(&load_executable(fixture.path()).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("unfilled laws: Chan"), "{error}");
}

#[test]
fn opaque_types_erase_without_requiring_runtime_definitions() {
    for argument in ["Chan(U32)", "Chan"] {
        let signature = if argument == "Chan" {
            "@-A: Type -> Data"
        } else {
            "Data"
        };
        let fixture = Fixture::new(&format!(
            "import Base\ndef ignore(value: {signature}) -> IO(Unit): IO.pure(Unit, Unit{{}})\ndef main() -> IO(Unit): ignore({argument})\n"
        ));
        let checked = check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
        compile_executable_javascript(&checked)
            .expect("opaque type values must lower without a body");
    }
}

#[test]
fn sequential_list_helper_specializes_with_existing_io_contracts() {
    let fixture = Fixture::new(
        "import Base\ndef main() -> IO(Unit): List.for_each(~&2, ~String, ~IO.print, [\"first\", \"second\"])\n",
    );
    let checked = check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    compile_executable_javascript(&checked).unwrap();
}

#[test]
fn pure_result_printer_does_not_invent_a_channel_handle_representation() {
    let fixture = Fixture::new(
        "import Base\ndef main() -> IO.OP<Chan(U32)>: Halt{0, \"unused handle branch\"}\n",
    );
    let checked = check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    let error = compile_executable_javascript(&checked)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("cannot be printed") && error.contains("Chan"),
        "{error}"
    );
}

#[test]
fn native_channel_creation_runs_its_checked_continuation() {
    let fixture = Fixture::new(
        "import Base\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    channel : Chan(U32) <- Chan.new(U32, 1)\n    IO.print(\"AFTER\")\n",
    );
    let checked = check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = checked
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap();
    assert_eq!(code, 0);
    assert_eq!(stdout, b"AFTER\n");
    assert!(stderr.is_empty());
}

#[test]
fn file_handles_are_affine_and_cannot_be_forged_or_promoted_to_data() {
    for body in [
        "def duplicate(file: File) -> File & File: (file, file)\n",
        "def duplicate(+file: File) -> File & File: (file, file)\n",
        "def fake() -> File: Unit{}\n",
        "def File(): U32\n",
        "def promote(file: File) -> List<&2, File>: [file]\n",
    ] {
        let fixture = Fixture::new(&format!("import Base\n{body}"));
        if let Ok(source) = load_executable(fixture.path()) {
            check_executable(&source).expect_err(body);
        }
    }
    let fixture = Fixture::new("import Base\ndef relay(file: File) -> File: file\n");
    check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    if let Ok(strict) = load(fixture.path()) {
        check_book(&strict).expect_err("execution-only File does not enter strict Base");
    }
    let fixture = Fixture::new(
        "import Base\ndef ignore(value: Type) -> IO(Unit): IO.pure(Unit, Unit{})\ndef main() -> IO(Unit): ignore(File)\n",
    );
    let checked = check_executable(&load_executable(fixture.path()).unwrap()).unwrap();
    compile_executable_javascript(&checked).unwrap();
}
