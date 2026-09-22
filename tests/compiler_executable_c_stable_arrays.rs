// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-stable-arrays-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("main.bend"), source).unwrap();
        Self(directory)
    }

    fn generate(&self) -> Result<String, teamy_bend::compiler::CompileError> {
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        compile_executable_c(&checked)
    }

    fn expect(&self, expected: &str) {
        let generated = self.generate().unwrap();
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
    (void)fputs("stable Array program retained runtime owners\n", stderr);
    return 91;
  }
  return status;
}
"#,
        );
        let executable =
            executable_c_compiler::compile(&self.0, &generated, &["BEND_MAX_ALLOC=4096"]);
        let output = executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(20),
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

const LIST_WORD: &str = r"import Base
def make(-A: Data, value: A) -> Array<List<&2, A>>:
  ALeaf{Con{value, Nil{}}}
def use(f: @-A: Data -> A -> Array<List<&2, A>>) -> Array<List<&2, U32>>:
  f(U32, 3)
def main() -> Array<List<&2, U32>> & List<&2, U32>:
  Array.get(List<&2, U32>, use(make), 0)
";

const LIST_STRING: &str = r#"import Base
def make(-A: Data, value: A) -> Array<List<&2, A>>:
  +item = {Con{value, Nil{}} : List<&2, A>}
  Array.new(List<&2, A>, 1n, item)
def use(f: @-A: Data -> A -> Array<List<&2, A>>) -> Array<List<&2, String>>:
  f(String, String.append("ke", "pt"))
def swapped(pair: Array<List<&2, String>> & List<&2, String>, read: List<&2, String>) ->
  Array<List<&2, String>> & List<&2, String> & List<&2, String>:
  (changed, old) = pair
  (Array.set(List<&2, String>, changed, 0, ["new"]), read, old)
def change(pair: Array<List<&2, String>> & List<&2, String>) ->
  Array<List<&2, String>> & List<&2, String> & List<&2, String>:
  (array, read) = pair
  swapped(Array.swap(List<&2, String>, array, 1, ["replacement"]), read)
def main() -> Array<List<&2, String>> & List<&2, String> & List<&2, String>:
  change(Array.get(List<&2, String>, use(make), 0))
"#;

#[test]
fn generic_recursive_list_cells_preserve_values_and_shared_owners() {
    Fixture::new(LIST_WORD).expect("([[3]], [3])\n");
    Fixture::new(LIST_STRING).expect("([[\"new\"], [\"replacement\"]], [\"kept\"], [\"kept\"])\n");
}

const LIST_PRODUCT: &str = r#"import Base
type BoxedPair<-A: Data> is Data:
  BoxedPair{left: List<&2, A>, right: List<&2, A>}
def make(-A: Data, left: A, right: A) -> Array<BoxedPair<A>>:
  ALeaf{BoxedPair{Con{left, Nil{}}, Con{right, Nil{}}}}
def use(f: @-A: Data -> A -> A -> Array<BoxedPair<A>>) -> Array<BoxedPair<String>>:
  f(String, String.append("le", "ft"), String.append("ri", "ght"))
def main() -> Array<BoxedPair<String>> & BoxedPair<String>:
  Array.get(BoxedPair<String>, use(make), 0)
"#;

#[test]
fn finite_products_flatten_multiple_stable_generic_list_fields() {
    Fixture::new(LIST_PRODUCT)
        .expect("([BoxedPair{[\"left\"], [\"right\"]}], BoxedPair{[\"left\"], [\"right\"]})\n");
}

const SUM_OWNERSHIP: &str = r#"import Base
type StableChoice<-A: Data> is Data:
  RawChoice{value: U32}
  ListChoice{values: List<&2, A>}
def make(-A: Data, value: A) -> Array<StableChoice<A>>:
  ALeaf{ListChoice{Con{value, Nil{}}}}
def use(f: @-A: Data -> A -> Array<StableChoice<A>>) -> Array<StableChoice<String>>:
  f(String, String.append("ro", "ot"))
def owned(pair: Array<StableChoice<String>> & StableChoice<String>, read: StableChoice<String>, old: StableChoice<String>) ->
  Array<StableChoice<String>> & StableChoice<String> & StableChoice<String> & StableChoice<String>:
  (array, number) = pair
  (Array.set(StableChoice<String>, array, 0, RawChoice{42}), read, old, number)
def raw(pair: Array<StableChoice<String>> & StableChoice<String>, read: StableChoice<String>) ->
  Array<StableChoice<String>> & StableChoice<String> & StableChoice<String> & StableChoice<String>:
  (array, old) = pair
  owned(Array.swap(StableChoice<String>, array, 0, ListChoice{["new"]}), read, old)
def change(pair: Array<StableChoice<String>> & StableChoice<String>) ->
  Array<StableChoice<String>> & StableChoice<String> & StableChoice<String> & StableChoice<String>:
  (array, read) = pair
  raw(Array.swap(StableChoice<String>, array, 0, RawChoice{9}), read)
def main() -> Array<StableChoice<String>> & StableChoice<String> & StableChoice<String> & StableChoice<String>:
  change(Array.get(StableChoice<String>, use(make), 0))
"#;

#[test]
fn finite_sum_get_swap_and_set_change_raw_and_owned_arm_masks() {
    Fixture::new(SUM_OWNERSHIP).expect(
        "([RawChoice{42}], ListChoice{[\"root\"]}, ListChoice{[\"root\"]}, RawChoice{9})\n",
    );
}

const PHANTOM_FIELDS: &str = r#"import Base
type Phantom<-A: Data> is Data: Phantom{number: U32}
type Witness<-A: Data> is Data: Witness{number: U32, -evidence: A}
def make(-A: Data, value: A) -> Array<Phantom<A>> & Array<Witness<A>>:
  (ALeaf{Phantom{7}}, ALeaf{Witness{9, value}})
def use(f: @-A: Data -> A -> (Array<Phantom<A>> & Array<Witness<A>>)) ->
  Array<Phantom<String>> & Array<Witness<String>>:
  f(String, String.append("dis", "card"))
def add(left: Phantom<String>, right: Witness<String>) -> U32:
  match left, right:
    case Phantom{x}, Witness{y, evidence}: U32.add(x, y)
def right_read(pair: Array<Witness<String>> & Witness<String>, p: Phantom<String>) -> U32:
  (array, w) = pair
  add(p, w)
def left_read(pair: Array<Phantom<String>> & Phantom<String>, right: Array<Witness<String>>) -> U32:
  (array, p) = pair
  right_read(Array.get(Witness<String>, right, 0), p)
def finish(pair: Array<Phantom<String>> & Array<Witness<String>>) -> U32:
  (left, right) = pair
  left_read(Array.get(Phantom<String>, left, 0), right)
def main() -> U32: finish(use(make))
"#;

#[test]
fn phantom_arguments_and_erased_fields_do_not_change_live_cell_layouts() {
    Fixture::new(PHANTOM_FIELDS).expect("16\n");
}

const INNER_ARRAY: &str = r#"import Base
def make(-A: Data, value: Array<A>) -> Array<Array<A>>:
  ALeaf{value}
def use(f: @-A: Data -> Array<A> -> Array<Array<A>>) -> Array<Array<String>>:
  f(String, ALeaf{String.append("o", "ld")})
def main() -> Array<Array<String>> & Array<String>:
  Array.swap(Array<String>, use(make), 0, ALeaf{String.append("n", "ew")})
"#;

const CLOSURE_CELL: &str = r#"import Base
type FunctionCell<-A: Data> is Type: FunctionCell{function: Unit -> A}
def make(-A: Data, value: A) -> Array<FunctionCell<A>>:
  ALeaf{FunctionCell{ignored => value}}
def use(f: @-A: Data -> A -> Array<FunctionCell<A>>) -> Array<FunctionCell<String>>:
  f(String, String.append("o", "ld"))
def replacement() -> FunctionCell<String>:
  text = String.append("n", "ew")
  FunctionCell{ignored => text}
def apply(cell: FunctionCell<String>) -> String:
  match cell:
    case FunctionCell{function}: function(Unit{})
def swapped(pair: Array<FunctionCell<String>> & FunctionCell<String>, old: FunctionCell<String>) -> String & String:
  (array, current) = pair
  (apply(old), apply(current))
def finish(pair: Array<FunctionCell<String>> & FunctionCell<String>) -> String & String:
  (array, old) = pair
  swapped(Array.swap(FunctionCell<String>, array, 0, FunctionCell{ignored => "drop"}), old)
def main() -> String & String:
  finish(Array.swap(FunctionCell<String>, use(make), 0, replacement))
"#;

#[test]
fn affine_boxed_inner_arrays_and_closure_cells_transfer_ownership_through_swap() {
    Fixture::new(INNER_ARRAY).expect("([[\"new\"]], [\"old\"])\n");
    Fixture::new(CLOSURE_CELL).expect("(\"old\", \"new\")\n");
}

const UNKNOWN_ELEMENT: &str = r"import Base
def make(-A: Data, value: A) -> Array<A>: ALeaf{value}
def use(f: @-A: Data -> A -> Array<A>) -> Array<U32>: f(U32, 3)
def main() -> Array<U32>: use(make)
";

const MAYBE_ELEMENT: &str = r"import Base
def make(-A: Data, value: A) -> Array<Maybe<&2, A>>: ALeaf{Some{value}}
def use(f: @-A: Data -> A -> Array<Maybe<&2, A>>) -> Array<Maybe<&2, U32>>:
  f(U32, 3)
def main() -> Array<Maybe<&2, U32>>: use(make)
";

const UNSTABLE_PRODUCT: &str = r"import Base
type UnstableProduct<-A: Data> is Data:
  UnstableProduct{stable: List<&2, A>, exposed: A}
def make(-A: Data, value: A) -> Array<UnstableProduct<A>>:
  ALeaf{UnstableProduct{Nil{}, value}}
def use(f: @-A: Data -> A -> Array<UnstableProduct<A>>) -> Array<UnstableProduct<U32>>:
  f(U32, 3)
def main() -> Array<UnstableProduct<U32>>: use(make)
";

const UNSTABLE_SUM: &str = r"import Base
type UnstableSum<-A: Data> is Data:
  StableArm{values: List<&2, A>}
  ExposedArm{value: A}
def make(-A: Data, value: A) -> Array<UnstableSum<A>>:
  ALeaf{StableArm{Con{value, Nil{}}}}
def use(f: @-A: Data -> A -> Array<UnstableSum<A>>) -> Array<UnstableSum<U32>>:
  f(U32, 3)
def main() -> Array<UnstableSum<U32>>: use(make)
";

const STUCK_APPLICATION: &str = r"import Base
def family(flag: Bool) -> Data:
  match flag:
    case True{}: List<&2, U32>
    case False{}: U32
def make(-flag: Bool, value: family(flag)) -> Array<family(flag)>: ALeaf{value}
def use(f: @-flag: Bool -> family(flag) -> Array<family(flag)>) -> Array<List<&2, U32>>:
  f(True{}, [3])
def main() -> Array<List<&2, U32>>: use(make)
";

#[test]
fn unresolved_live_fields_and_stuck_applications_remain_rejected_after_checking() {
    for (name, source) in [
        ("unknown element", UNKNOWN_ELEMENT),
        ("Maybe element", MAYBE_ELEMENT),
        ("mixed product", UNSTABLE_PRODUCT),
        ("unselected sum arm", UNSTABLE_SUM),
        ("stuck type application", STUCK_APPLICATION),
    ] {
        let error = Fixture::new(source).generate().unwrap_err();
        assert!(
            error.to_string().contains("unspecialized element type"),
            "{name}: {error}"
        );
    }
}
