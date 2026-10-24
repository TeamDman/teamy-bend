// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-checker-diagnostic-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("main.bend"), source).unwrap();
        Self(path)
    }
    fn source(&self) -> PathBuf {
        self.0.join("main.bend")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_diagnostic(path: &Path, expected: &str) {
    for command in ["check", "eval", "run"] {
        for format in ["text", "json", "csv"] {
            let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
                .args(["--output-format", format, command])
                .arg(path)
                .env("BEND_GPU", "off")
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{command}/{format}");
            assert!(output.stdout.is_empty(), "{command}/{format}");
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                expected,
                "{command}/{format}"
            );
        }
    }
}

#[test]
fn original_false_lists_and_brace_report_normalized_endpoints_and_source() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-checking");
    // Exact expectations verified against the pinned checker. These are failures
    // before any main execution, including the no-main list books.
    for (name, expected) in [
        (
            "wrong_reverse",
            "Error:\n- expected : [3, 2, 1]\n- observed : [3, 1, 2]\nLocation: rev\n62 | def rev():\n63>|   {==}\n64 | \n",
        ),
        (
            "lost_duplicate",
            "Error:\n- expected : [1, 2, 3, 3]\n- observed : [1, 2, 3]\nLocation: srt\n74 | def srt():\n75>|   {==}\n76 | \n",
        ),
        (
            "false_brace",
            "Error:\n- expected : 3\n- observed : 4\nLocation: refl\n 8 | \n 9>| def refl(): {==}\n10 | \n",
        ),
    ] {
        assert_diagnostic(&fixtures.join(format!("{name}.bend")), expected);
    }
}

#[test]
fn context_and_multiple_proofs_use_the_rejected_node() {
    let fixture = Fixture::new(&format!(
        "{NAT}def bad() -> {{0n == 1n : Nat}}:\n  -ok = {{{{==}} : {{0n == 0n : Nat}}}}\n  {{==}}\ndef main() -> Nat: 0n\n"
    ));
    assert_diagnostic(
        &fixture.source(),
        "Error:\n- expected : 0n\n- observed : 1n\nContext:\n- ok : {0n == 0n : Nat}\nLocation: bad\n5 |   -ok = {{==} : {0n == 0n : Nat}}\n6>|   {==}\n7 | def main() -> Nat: 0n\n",
    );
}

#[test]
fn crlf_and_final_line_keep_exact_source_excerpts() {
    let original = include_str!("fixtures/cpu-checking/false_brace.bend");
    let crlf = Fixture::new(&original.replace('\n', "\r\n"));
    assert_diagnostic(
        &crlf.source(),
        "Error:\n- expected : 3\n- observed : 4\nLocation: refl\n 8 | \r\n 9>| def refl(): {==}\r\n10 | \r\n",
    );
    let eof = Fixture::new(&format!(
        "{NAT}def main() -> Nat: 0n\ndef bad() -> {{0n == 1n : Nat}}: {{==}}"
    ));
    assert_diagnostic(
        &eof.source(),
        "Error:\n- expected : 0n\n- observed : 1n\nLocation: bad\n4 | def main() -> Nat: 0n\n5>| def bad() -> {0n == 1n : Nat}: {==}\n",
    );
}

#[test]
fn imported_and_instantiated_proofs_retain_the_original_source() {
    let imported = Fixture::new("import helper.bend as H\ndef main() -> H.Nat: H.Zero{}\n");
    std::fs::write(
        imported.0.join("helper.bend"),
        format!("# λ source\n{NAT}def bad() -> {{Zero{{}} == Succ{{Zero{{}}}} : Nat}}: {{==}}\n"),
    )
    .unwrap();
    assert_diagnostic(
        &imported.source(),
        "Error:\n- expected : helper.Zero{}\n- observed : helper.Succ{helper.Zero{}}\nLocation: helper.bad\n4 |   Succ{pred: Nat}\n5>| def bad() -> {Zero{} == Succ{Zero{}} : Nat}: {==}\n6 | \n",
    );
    let instance = Fixture::new(&format!(
        "{NAT}def claim(~n: Nat) -> {{n == 0n : Nat}}:\n  {{==}}\ndef main() -> {{1n == 0n : Nat}}: claim(1n)\n"
    ));
    assert_diagnostic(
        &instance.source(),
        "Error:\n- expected : n\n- observed : 0n\nContext:\n- n : Nat\nLocation: claim~0\n4 | def claim(~n: Nat) -> {n == 0n : Nat}:\n5>|   {==}\n6 | def main() -> {1n == 0n : Nat}: claim(1n)\n",
    );
}

#[test]
fn constructor_originals_parse_then_reject_with_exact_language_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructors");
    for (name, expected) in [
        (
            "nat_literal_import",
            "Error:\n- expected : a declared constructor\n- observed : 2n\nLocation: main\n 9 | def main():\n10>|   x = 2n\n11 |   Type\n",
        ),
        (
            "tuple_import_required",
            "Error:\n- expected : a declared constructor\n- observed : (1, 2)\nLocation: main\n 9 | def main():\n10>|   x = (1, 2)\n11 |   Type\n",
        ),
        (
            "list_sugar",
            "Error:\n- expected : a declared constructor\n- observed : [Zx{}]\nLocation: main\n10 | def main():\n11>|   h = [Zx{}]\n12 |   Zx{}\n",
        ),
        (
            "qualified_constructor",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Nat.Succ{0n}\nLocation: main\n 9 | def main():\n10>|   Nat.Succ{Zero{}}\n11 | \n",
        ),
        (
            "ctor_list_mismatch_000",
            "Error:\n- expected : a declared constructor (MB declares T, F)\n- observed : Tru{}\nLocation: main\n 8 | \n 9>| def main(): Tru{}\n10 | \n",
        ),
        (
            "arity-zero",
            "Error:\n- expected : Zero with 0 fields\n- observed : Zero{0n}\nLocation: main\n3 |   Succ{pred: Nat}\n4>| def main() -> Nat: Zero{0n}\n5 | \n",
        ),
        (
            "arity-one",
            "Error:\n- expected : Succ with 1 field\n- observed : Succ{}\nLocation: main\n3 |   Succ{pred: Nat}\n4>| def main() -> Nat: Succ{}\n5 | \n",
        ),
        (
            "wrong-family",
            "Error:\n- expected : Nat\n- observed : Other\nLocation: main\n5 |   Else{}\n6>| def main() -> Nat: Else{}\n7 | \n",
        ),
        (
            "known-inference",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : 1n\nLocation: main\n4 | def main() -> Nat:\n5>|   x = Succ{Zero{}}\n6 |   x\n",
        ),
        (
            "context-closed",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{}\nContext:\n- n : Nat\nLocation: bad\n3 |   Succ{pred: Nat}\n4>| def bad(n: Nat) -> Nat: Wrong{}\n5 | def main() -> Nat: bad(0n)\n",
        ),
        (
            "parenthesized",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{}\nLocation: main\n4 | def main() -> Nat: (\n5>|   Wrong{}\n6 | )\n",
        ),
    ] {
        let path = root.join(format!("{name}.bend"));
        let book = teamy_bend::syntax::load(&path).expect("parsing/loading succeeds");
        assert!(teamy_bend::kernel::check_book(&book).is_err(), "{name}");
        assert_diagnostic(&path, expected);
    }
}

#[test]
fn imported_nat_and_list_constructor_diagnostics_use_the_declared_family() {
    for (main, helper, expected) in [
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.value\n",
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef value() -> Nat: 0n\n",
            "Error:\n- expected : a declared constructor (helper.Nat declares helper.Zero, helper.Succ)\n- observed : 0n\nLocation: helper.value\n3 |   Succ{pred: Nat}\n4>| def value() -> Nat: 0n\n5 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.value\n",
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef value() -> Nat: 1n\n",
            "Error:\n- expected : a declared constructor (helper.Nat declares helper.Zero, helper.Succ)\n- observed : 1n\nLocation: helper.value\n3 |   Succ{pred: Nat}\n4>| def value() -> Nat: 1n\n5 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.Zero{}\n",
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef proof() -> {0n == 0n : Nat}: {==}\n",
            "Error:\n- expected : a declared constructor (helper.Nat declares helper.Zero, helper.Succ)\n- observed : 0n\nLocation: helper.proof\n3 |   Succ{pred: Nat}\n4>| def proof() -> {0n == 0n : Nat}: {==}\n5 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.value\n",
            "import Base\ntype Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef value() -> Nat: 1n\n",
            "Error:\n- expected : helper.Nat\n- observed : Nat\nLocation: helper.value\n4 |   Succ{pred: Nat}\n5>| def value() -> Nat: 1n\n6 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.Zero{}\n",
            "# λ source\ntype Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad() -> {0n == 1n : Nat}: {==}\n",
            "Error:\n- expected : a declared constructor (helper.Nat declares helper.Zero, helper.Succ)\n- observed : 0n\nLocation: helper.bad\n4 |   Succ{pred: Nat}\n5>| def bad() -> {0n == 1n : Nat}: {==}\n6 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "type Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: []\n",
            "Error:\n- expected : a declared constructor (helper.List declares helper.Nil, helper.Con)\n- observed : []\nLocation: helper.value\n5 |   Con{head: Item, tail: List}\n6>| def value() -> List: []\n7 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "import Base\ntype Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: []\n",
            "Error:\n- expected : helper.List\n- observed : List\nLocation: helper.value\n6 |   Con{head: Item, tail: List}\n7>| def value() -> List: []\n8 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "type Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: [Val{}]\n",
            "Error:\n- expected : a declared constructor (helper.List declares helper.Nil, helper.Con)\n- observed : [helper.Val{}]\nLocation: helper.value\n5 |   Con{head: Item, tail: List}\n6>| def value() -> List: [Val{}]\n7 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "import Base\ntype Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: [Val{}]\n",
            "Error:\n- expected : helper.List\n- observed : List\nLocation: helper.value\n6 |   Con{head: Item, tail: List}\n7>| def value() -> List: [Val{}]\n8 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "type Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: [Val{}, Val{}]\n",
            "Error:\n- expected : a declared constructor (helper.List declares helper.Nil, helper.Con)\n- observed : [helper.Val{}, helper.Val{}]\nLocation: helper.value\n5 |   Con{head: Item, tail: List}\n6>| def value() -> List: [Val{}, Val{}]\n7 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "import Base\ntype Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: [Val{}, Val{}]\n",
            "Error:\n- expected : helper.List\n- observed : List\nLocation: helper.value\n6 |   Con{head: Item, tail: List}\n7>| def value() -> List: [Val{}, Val{}]\n8 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "type Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: Val{} <> Nil{}\n",
            "Error:\n- expected : a declared constructor (helper.List declares helper.Nil, helper.Con)\n- observed : helper.Val{} <> helper.Nil{}\nLocation: helper.value\n5 |   Con{head: Item, tail: List}\n6>| def value() -> List: Val{} <> Nil{}\n7 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "import Base\ntype Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef value() -> List: Val{} <> Nil{}\n",
            "Error:\n- expected : helper.List\n- observed : List\nLocation: helper.value\n6 |   Con{head: Item, tail: List}\n7>| def value() -> List: Val{} <> Nil{}\n8 | \n",
        ),
    ] {
        let fixture = Fixture::new(main);
        std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn reconstructed_open_constructor_keeps_rejection_without_guessing_source() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-constructors/context-open.bend");
    let book = teamy_bend::syntax::load(&path).unwrap();
    teamy_bend::kernel::check_book(&book).unwrap_err();
    for command in ["check", "eval", "run"] {
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args([command])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Wrong is not a remaining constructor of Nat"));
        assert!(!stderr.contains("Location: bad"));
    }
}
