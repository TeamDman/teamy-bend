// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const STATE: &str = r#"type State is Type: State{value: U32}
def view(state: State) -> State & Image: (state, Pix{0})
def finish(result: Maybe<State>) -> IO(Unit):
  match result:
    case None{}: IO.print("none")
    case Some{State{value}}: IO.print(U32.show(value))
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-app-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.bend"), format!("import Base\n{source}")).unwrap();
        Self(path)
    }

    fn checked(&self) -> ExecutableBook {
        let source = load_executable(self.0.join("main.bend")).unwrap();
        check_executable(&source).unwrap()
    }

    fn javascript(&self, checked: &ExecutableBook) -> std::process::Output {
        let source = compile_executable_javascript(checked).unwrap();
        fs::write(self.0.join("main.cjs"), source).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("App compiler tests require Node.js")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn compare(source: &str, status: u32, stdout: &str, stderr: &str) {
    let fixture = Fixture::new(source);
    let checked = fixture.checked();
    let mut native_stdout = Vec::new();
    let mut native_stderr = Vec::new();
    assert_eq!(
        checked
            .run_main(&mut native_stdout, &mut native_stderr, &|| false)
            .unwrap(),
        status
    );
    assert_eq!(native_stdout, stdout.as_bytes());
    assert_eq!(native_stderr, stderr.as_bytes());
    let output = fixture.javascript(&checked);
    assert_eq!(
        output
            .status
            .code()
            .and_then(|code| u32::try_from(code).ok()),
        Some(status),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, stdout.as_bytes());
    assert_eq!(output.stderr, stderr.as_bytes());
}

#[test]
fn playback_preserves_affine_state_frame_order_and_all_event_fields() {
    compare(
        &format!(
            r#"{STATE}
def flag(value: Bool) -> String:
  match value:
    case False{{}}: "false"
    case True{{}}: "true"
def event(value: Event) -> String:
  match value:
    case Key{{code, down}}: "key:" ++ U32.show(code) ++ ":" ++ flag(down)
    case Mouse{{x, y, button, down}}:
      "mouse:" ++ U32.show(x) ++ ":" ++ U32.show(y) ++ ":" ++ U32.show(button) ++ ":" ++ flag(down)
    case Move{{x, y}}: "move:" ++ U32.show(x) ++ ":" ++ U32.show(y)
    case Close{{}}: "close"
def trace(events: List<Event>) -> IO(Unit):
  match events:
    case Nil{{}}: IO.pure(Unit, Unit{{}})
    case value <> rest:
      do IO<Unit>:
        Unit <- IO.print(event(value))
        trace(rest)
def tick(events: List<Event>, state: State) -> IO(Maybe<State>):
  State{{+value}} = state
  do IO<Maybe<State>>:
    Unit <- IO.print("begin:" ++ U32.show(value))
    Unit <- trace(events)
    Unit <- IO.print("end:" ++ U32.show(value))
    return Some{{State{{U32.inc(value)}}}}
def main() -> IO(Unit):
  IO.bind(Maybe<State>, Unit,
    App.play(~State, ~App{{view, tick}},
      [[Key{{17, True{{}}}}, Mouse{{3, 5, 2, True{{}}}}, Move{{7, 11}}, Close{{}}],
        [], [Key{{19, False{{}}}}, Mouse{{13, 23, 1, False{{}}}}]], State{{0}}), finish)
"#
        ),
        0,
        "begin:0\nkey:17:true\nmouse:3:5:2:true\nmove:7:11\nclose\nend:0\nbegin:1\nend:1\nbegin:2\nkey:19:false\nmouse:13:23:1:false\nend:2\n3\n",
        "",
    );
}

#[test]
fn empty_frames_return_the_same_state_without_ticking() {
    compare(
        &format!(
            r#"{STATE}
def tick(events: List<Event>, state: State) -> IO(Maybe<State>):
  IO.die(Maybe<State>, 8, "unexpected tick")
def main() -> IO(Unit):
  IO.bind(Maybe<State>, Unit, App.play(~State, ~App{{view, tick}}, [], State{{9}}), finish)
"#
        ),
        0,
        "9\n",
        "",
    );
}

#[test]
fn more_and_fold_call_the_continuation_only_after_a_successful_tick() {
    compare(
        &format!(
            r#"{STATE}
def tick(events: List<Event>, state: State) -> IO(Maybe<State>):
  do IO<Maybe<State>>:
    Unit <- IO.print("tick")
    return Some{{state}}
def rest(state: State) -> IO(Maybe<State>):
  do IO<Maybe<State>>:
    Unit <- IO.print("rest")
    return Some{{state}}
def main() -> IO(Unit):
  do IO<Unit>:
    a : Maybe<State> <- App.more(State, rest, None{{}})
    Unit <- finish(a)
    b : Maybe<State> <- App.fold(State, App{{view, tick}}, [], State{{4}}, rest)
    finish(b)
"#
        ),
        0,
        "none\ntick\nrest\n4\n",
        "",
    );
}

#[test]
fn none_stops_playback_before_later_frames() {
    compare(
        &format!(
            r#"{STATE}
def next(events: List<Event>, state: State) -> Maybe<State>:
  match events:
    case Nil{{}}: Some{{state}}
    case value <> rest:
      match value:
        case Close{{}}: None{{}}
        case other: Some{{state}}
def tick(events: List<Event>, state: State) -> IO(Maybe<State>):
  do IO<Maybe<State>>:
    Unit <- IO.print("tick")
    return next(events, state)
def main() -> IO(Unit):
  IO.bind(Maybe<State>, Unit,
    App.play(~State, ~App{{view, tick}}, [[], [Close{{}}], [Move{{2, 3}}]], State{{5}}), finish)
"#
        ),
        0,
        "tick\ntick\nnone\n",
        "",
    );
}

#[test]
fn halt_stops_the_remaining_frames_and_result_continuation() {
    compare(
        &format!(
            r#"{STATE}
def tick(events: List<Event>, state: State) -> IO(Maybe<State>):
  do IO<Maybe<State>>:
    Unit <- IO.print("tick")
    IO.die(Maybe<State>, 7, "stop")
def main() -> IO(Unit):
  IO.bind(Maybe<State>, Unit,
    App.play(~State, ~App{{view, tick}}, [[], []], State{{5}}), finish)
"#
        ),
        7,
        "tick\n",
        "stop\n",
    );
}

#[test]
fn playback_never_invokes_view_and_existing_execution_budgets_still_apply() {
    let helpers = format!(
        r"{STATE}
def burn(n: Nat) -> Image:
  match n:
    case Zero{{}}: Pix{{0}}
    case Succ{{rest}}: burn(rest)
def expensive_view(state: State) -> State & Image:
  (state, burn(U32.to_nat(1000000)))
def tick(events: List<Event>, state: State) -> IO(Maybe<State>):
  IO.pure(Maybe<State>, Some{{state}})
"
    );
    compare(
        &format!(
            "{helpers}def main() -> IO(Unit): IO.bind(Maybe<State>, Unit, App.play(~State, ~App{{expensive_view, tick}}, [[], []], State{{6}}), finish)\n"
        ),
        0,
        "6\n",
        "",
    );
    let fixture = Fixture::new(&format!(
        "{helpers}def color(image: Image) -> U32:\n  match image:\n    case Pix{{value}}: value\n    case Qua{{tl, tr, bl, br}}: 0\ndef use(pair: State & Image) -> IO(Unit):\n  (state, image) = pair\n  IO.print(U32.show(color(image)))\ndef main() -> IO(Unit): use(expensive_view(State{{0}}))\n"
    ));
    let checked = fixture.checked();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = checked
        .run_main(&mut stdout, &mut stderr, &|| false)
        .expect_err("an invoked view must retain the existing execution budget")
        .to_string();
    assert!(error.contains("continuation depth exhausted"), "{error}");
    assert!(stdout.is_empty() && stderr.is_empty());
    let output = fixture.javascript(&checked);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("budget"));
}

#[test]
fn playback_templates_do_not_capture_callers_or_duplicate_affine_values() {
    let captured = Fixture::new(&format!(
        "{STATE}def open(app: App<State>, frames: List<List<Event>>, state: State) -> IO(Maybe<State>): App.play(~State, ~app, frames, state)\n"
    ));
    let source = load_executable(captured.0.join("main.bend")).unwrap();
    let error = check_executable(&source)
        .expect_err("template arguments cannot capture the caller's app")
        .to_string();
    assert!(error.contains("undefined name App.play"), "{error}");
    for definition in [
        "def select(left: State, right: State) -> State: left\ndef bad_view(state: State) -> State & Image: (select(state, state), Pix{0})\n",
        "def simple_tick(events: List<Event>, state: State) -> IO(Maybe<State>): IO.pure(Maybe<State>, Some{state})\ndef bad_tick(events: List<Event>, state: State) -> IO(Maybe<State>):\n  IO.bind(Maybe<State>, Maybe<State>, simple_tick(events, state), result => IO.pure(Maybe<State>, Some{state}))\n",
        "def duplicate(app: App<State>) -> App<State> & App<State>: (app, app)\n",
    ] {
        let fixture = Fixture::new(&format!("{STATE}{definition}"));
        let source = load_executable(fixture.0.join("main.bend")).unwrap();
        let error = check_executable(&source)
            .expect_err("affine app/state must not be duplicated")
            .to_string();
        assert!(error.contains("permits Lone use, observed Many"), "{error}");
    }
}

#[test]
fn app_is_checked_ordinary_executable_base_without_new_runtime_assumptions() {
    let fixture = Fixture::new("");
    let checked = fixture.checked();
    for name in ["App.more", "App.fold"] {
        assert!(checked.definition_type(name).is_some());
        assert!(!checked.foreign_names().any(|foreign| foreign == name));
    }
    assert_eq!(checked.foreign_names().count(), 10);
    assert_eq!(checked.opaque_names().collect::<Vec<_>>(), ["Chan"]);
    let strict = load(fixture.0.join("main.bend")).unwrap();
    check_book(&strict).unwrap();
    assert!(!strict.declarations.iter().any(|declaration| {
        matches!(declaration, teamy_bend::kernel::Declaration::Adt(adt) if adt.name == "App")
    }));
}
