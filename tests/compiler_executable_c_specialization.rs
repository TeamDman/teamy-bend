// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const DATA_PRODUCTS: &str = "type SpecialPair is Data: SpecialPair{word: U32, text: String}\ntype SpecialQuad is Data: SpecialQuad{a: U32, b: U32, c: U32, d: U32}\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-specialization-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("main.bend"), source).unwrap();
        Self(directory)
    }

    fn run(&self) -> Output {
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert_eq!(generated.matches("int main(void)").count(), 1);
        let mut generated = generated.replace("int main(void)", "static int checked_main(void)");
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (tb_live_words != 0 || tb_live_blocks != 0) {
    (void)fputs("specialized program leaked VM owners\n", stderr);
    return 91;
  }
  return status;
}
"#,
        );
        let executable = executable_c_compiler::compile(&self.0, &generated, &[]);
        executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(20),
        )
    }

    fn rejection(&self) -> String {
        match load_executable(self.0.join("main.bend")) {
            Err(error) => error.to_string(),
            Ok(loaded) => match check_executable(&loaded) {
                Err(error) => error.to_string(),
                Ok(_) => panic!("invalid source passed executable checking"),
            },
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn local_erased_and_live_type_aliases_specialize_packed_and_owned_array_elements() {
    for (kind, value, expected) in [
        ("U32", "7", "[7, 7]\n"),
        ("String", "\"kept\"", "[\"kept\", \"kept\"]\n"),
        ("Maybe<&2, U32>", "Some{7}", "[Some{7}, Some{7}]\n"),
        (
            "SpecialPair",
            "SpecialPair{7, \"kept\"}",
            "[SpecialPair{7, \"kept\"}, SpecialPair{7, \"kept\"}]\n",
        ),
    ] {
        for qualifier in ["-", ""] {
            let source = format!(
                "import Base\n{DATA_PRODUCTS}def main() -> List<({kind})>:\n  {qualifier}Local = ({kind})\n  Array.to_list(~({kind}), Array.new(Local, 1n, {value}))\n"
            );
            success(&Fixture::new(&source).run(), expected);
        }
    }
}

const SIMULTANEOUS: &str = r#"import Base
def main() -> List<U32> & List<String>:
  Word Text = U32 String
  (Array.to_list(~U32, Array.new(Word, 1n, 7)),
    Array.to_list(~String, Array.new(Text, 1n, "yes")))
"#;

const SHADOWING: &str = r#"import Base
def make(-Local: Data, discarded: Local) -> Array<U32>:
  -Local = U32
  Array.new(Local, 1n, 7)
def main() -> List<U32>:
  Array.to_list(~U32, make(String, String.append("drop-", "me")))
"#;

const PARALLEL_SHADOWING: &str = r"import Base
def main() -> List<U32>:
  Original = U32
  Original Alias = String Original
  Array.to_list(~U32, Array.new(Alias, 1n, 7))
";

#[test]
fn simultaneous_closed_aliases_and_shadowed_type_parameters_keep_distinct_binding_ids() {
    success(
        &Fixture::new(SIMULTANEOUS).run(),
        "([7, 7], [\"yes\", \"yes\"])\n",
    );
    success(&Fixture::new(SHADOWING).run(), "[7, 7]\n");
    success(&Fixture::new(PARALLEL_SHADOWING).run(), "[7, 7]\n");
}

#[test]
fn direct_calls_specialize_erased_binders_in_returned_closures() {
    for (kind, value, expected) in [
        ("U32", "7", "[7]\n"),
        ("String", "\"returned\"", "[\"returned\"]\n"),
        ("Maybe<&2, U32>", "Some{7}", "[Some{7}]\n"),
        (
            "SpecialPair",
            "SpecialPair{7, \"returned\"}",
            "[SpecialPair{7, \"returned\"}]\n",
        ),
        (
            "SpecialQuad",
            "SpecialQuad{11, 22, 33, 44}",
            "[SpecialQuad{11, 22, 33, 44}]\n",
        ),
    ] {
        let source = format!(
            "import Base\n{DATA_PRODUCTS}def make() -> @-A: Data -> A -> Array<A>:\n  A => value => ALeaf{{value}}\ndef main() -> List<({kind})>:\n  Array.to_list(~({kind}), make(({kind}), {value}))\n"
        );
        success(&Fixture::new(&source).run(), expected);
    }
}

const LIVE_THEN_ERASED: &str = r#"import Base
def make(prefix: String) -> @-A: Data -> A -> (String & Array<A>):
  A => value => (prefix, ALeaf{value})
def flatten(pair: String & Array<U32>) -> String & List<U32>:
  (prefix, array) = pair
  (prefix, Array.to_list(~U32, array))
def main() -> String & List<U32>:
  flatten(make(String.append("cap", "tured"), U32, 7))
"#;

const LET_THEN_ERASED: &str = r#"import Base
def make(prefix: U32) -> @-A: Data -> A -> (String & Array<A>):
  text = String.append("made:", U32.show(prefix))
  A => value => (text, ALeaf{value})
def flatten(pair: String & Array<U32>) -> String & List<U32>:
  (prefix, array) = pair
  (prefix, Array.to_list(~U32, array))
def main() -> String & List<U32>: flatten(make(19, U32, 7))
"#;

#[test]
fn returned_erased_binders_after_live_arguments_and_lets_preserve_owned_captures() {
    success(
        &Fixture::new(LIVE_THEN_ERASED).run(),
        "(\"captured\", [7])\n",
    );
    success(&Fixture::new(LET_THEN_ERASED).run(), "(\"made:19\", [7])\n");
}

const PARTIAL_ALIAS: &str = r"import Base
def main() -> List<U32>:
  -Local = U32
  make = Array.new(Local, 1n)
  Array.to_list(~U32, make(7))
";

const PARTIAL_CAPTURE: &str = r#"import Base
def make(-A: Data, prefix: String) -> A -> (String & Array<A>):
  value => (prefix, ALeaf{value})
def flatten(pair: String & Array<String>) -> String & List<String>:
  (prefix, array) = pair
  (prefix, Array.to_list(~String, array))
def main() -> String & List<String>:
  function = make(String, String.append("cap", "tured"))
  flatten(function("retained"))
"#;

#[test]
fn partially_applied_monomorphic_closures_keep_alias_layouts_and_owned_arguments() {
    success(&Fixture::new(PARTIAL_ALIAS).run(), "[7, 7]\n");
    success(
        &Fixture::new(PARTIAL_CAPTURE).run(),
        "(\"captured\", [\"retained\"])\n",
    );
}

#[test]
fn constructor_discovery_visits_array_elements_in_both_used_and_unused_variants() {
    let declarations = "import Base\ntype Element is Data:\n  Element{value: U32}\ntype Holder is Type:\n  Empty{}\n  Full{items: Array<Element>}\n";
    for (value, expected) in [
        ("Empty{}", "Empty{}\n"),
        (
            "Full{ANode{ALeaf{Element{11}}, ALeaf{Element{22}}}}",
            "Full{[Element{11}, Element{22}]}\n",
        ),
    ] {
        let source = format!("{declarations}\ndef main() -> Holder: {value}\n");
        success(&Fixture::new(&source).run(), expected);
    }
}

const ERASED_REACHABILITY: &str = r#"import Base
type Element is Data:
  Element{value: U32}
def produce(-A: Data) -> IO(A): import "produce.c"
def discard(-A: Data, value: A) -> Unit:
  array = {ALeaf{value} : Array<A>}
  Unit{}
def work(-A: Data) -> IO(Unit):
  do IO<Unit>:
    value : A <- produce(A)
    IO.pure(Unit, discard(A, value))
def main() -> IO(Unit): work(Element)
"#;

const ERASED_REACHABILITY_C: &str = r#"
static u32 produced;
static Term produce_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  if (++produced != 1) err_fail("producer executed more than once");
  io_out(stdout, "made\n", 5);
  return term_pak(CID_ELEMENT, 31);
}
static void __attribute__((constructor)) produce_use(void) {
  io_eff(CID_PRODUCE, produce_run, 0);
}
"#;

#[test]
fn constructors_reachable_only_through_checked_erased_type_arguments_are_registered() {
    let fixture = Fixture::new(ERASED_REACHABILITY);
    fs::write(fixture.0.join("produce.c"), ERASED_REACHABILITY_C).unwrap();
    success(&fixture.run(), "made\n");
}

#[test]
fn specialization_preserves_affinity_erasure_proof_and_parallel_alias_rejections() {
    for (source, expected) in [
        (
            "import Base\ndef bad(file: File) -> U32 -> (File & File): ignored => (file, file)\ndef main() -> U32: 0\n",
            "binder file permits Lone use, observed Many",
        ),
        (
            "import Base\ndef bad(-value: U32) -> U32: value\ndef main() -> U32: bad(7)\n",
            "binder value permits None use, observed Lone",
        ),
        (
            "import Base\ndef require(-proof: {0 == 1 : U32}, value: U32) -> U32: value\ndef main() -> U32: require({==}, 7)\n",
            "reflexivity requires equal endpoints",
        ),
        (
            "import Base\ndef main() -> List<U32>:\n  First Second = U32 First\n  Array.to_list(~U32, Array.new(Second, 1n, 7))\n",
            "undefined name First",
        ),
    ] {
        let error = Fixture::new(source).rejection();
        assert!(error.contains(expected), "{error}");
    }
}
