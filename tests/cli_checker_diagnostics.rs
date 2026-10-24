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
fn reconstructed_open_constructor_retains_exact_rejection_and_source() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-constructors/context-open.bend");
    let book = teamy_bend::syntax::load(&path).unwrap();
    teamy_bend::kernel::check_book(&book).unwrap_err();
    assert_diagnostic(
        &path,
        "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{n}\nContext:\n- n : Nat\nLocation: bad\n3 |   Succ{pred: Nat}\n4>| def bad(n: Nat) -> Nat: Wrong{n}\n5 | def main() -> Nat: bad(0n)\n",
    );
}

#[test]
fn transformed_and_pattern_origins_keep_nested_imported_and_template_locations() {
    for (main, helper, phase, expected) in [
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  Succ{\n    Wrong{n}\n  }\ndef main() -> Nat: bad(0n)\n",
            "",
            "checking",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{n}\nContext:\n- n : Nat\nLocation: bad\n5 |   Succ{\n6>|     Wrong{n}\n7 |   }\n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.bad(H.Zero{})\n",
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  Succ{\n    Wrong{n}\n  }\n",
            "checking",
            "Error:\n- expected : a declared constructor (helper.Nat declares helper.Zero, helper.Succ)\n- observed : Wrong{n}\nContext:\n- n : helper.Nat\nLocation: helper.bad\n5 |   Succ{\n6>|     Wrong{n}\n7 |   }\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{p}:\n      Wrong{p}\n    case Zero{}: Zero{}\ndef main() -> Nat: bad(0n)\n",
            "",
            "checking",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{p}\nContext:\n- p : Nat\nLocation: bad\n6 |     case Succ{p}:\n7>|       Wrong{p}\n8 |     case Zero{}: Zero{}\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef Nat.add(a: Nat, b: Nat) -> Nat: a\ndef bad(n: Nat) -> Nat: Wrong{n + 0n}\ndef main() -> Nat: bad(0n)\n",
            "",
            "checking",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{n}\nContext:\n- n : Nat\nLocation: bad\n4 | def Nat.add(a: Nat, b: Nat) -> Nat: a\n5>| def bad(n: Nat) -> Nat: Wrong{n + 0n}\n6 | def main() -> Nat: bad(0n)\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef wrap(~x: Nat) -> Nat: Wrong{x}\ndef main() -> Nat: wrap(0n)\n",
            "",
            "checking",
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Wrong{x}\nContext:\n- x : Nat\nLocation: wrap~0\n3 |   Succ{pred: Nat}\n4>| def wrap(~x: Nat) -> Nat: Wrong{x}\n5 | def main() -> Nat: wrap(0n)\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{\n      Missing{}\n    }: Zero{}\n    case Zero{}: Zero{}\ndef main() -> Nat: bad(0n)\n",
            "",
            "parse_load",
            "Error:\n- message  : a declared constructor (unknown: Missing)\nLocation:\n6 |     case Succ{\n7>|       Missing{}\n8 |     }: Zero{}\n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.value(H.Zero{})\n",
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef value(n: Nat) -> Nat: 1n+n\n",
            "checking",
            "Error:\n- expected : a declared constructor (helper.Nat declares helper.Zero, helper.Succ)\n- observed : 1n+n\nContext:\n- n : helper.Nat\nLocation: helper.value\n3 |   Succ{pred: Nat}\n4>| def value(n: Nat) -> Nat: 1n+n\n5 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Nat: H.value(H.Succ{H.Zero{}})\n",
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef value(n: Nat) -> Nat:\n  match n:\n    case 0n: Zero{}\n    case 1n+p: p\n",
            "parse_load",
            "Error:\n- message  : a declared constructor (unknown: Zero)\nLocation:\n5 |   match n:\n6>|     case 0n: Zero{}\n7 |     case 1n+p: p\n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.List: H.value\n",
            "type Item is Data:\n  Val{}\ntype List is Data:\n  Nil{}\n  Con{head: Item, tail: List}\ndef tail(xs: List) -> List:\n  match xs:\n    case []: Nil{}\n    case h <> t: t\ndef value() -> List: tail(Con{Val{}, Nil{}})\n",
            "parse_load",
            "Error:\n- message  : a declared constructor (unknown: Nil)\nLocation:\n7 |   match xs:\n8>|     case []: Nil{}\n9 |     case h <> t: t\n",
        ),
    ] {
        let fixture = Fixture::new(main);
        std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        if phase == "parse_load" {
            teamy_bend::syntax::load(fixture.source()).unwrap_err();
        } else {
            teamy_bend::kernel::check_book(&teamy_bend::syntax::load(fixture.source()).unwrap())
                .unwrap_err();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn false_fusion_rewrite_reports_normalized_types_and_original_source() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-checking/false_fusion.bend");
    assert_diagnostic(
        &path,
        "Error:\n- expected : {U32.add(U32.mul(h, 2), 1) <> map(AddOne{}, map(MulTwo{}, t)) == U32.add(U32.mul(h, 2), 2) <> map(MulTwoAddOne{}, t) : List<&2, U32>}\n- observed : {U32.add(U32.mul(h, 2), 1) <> map(AddOne{}, map(MulTwo{}, t)) == U32.add(U32.mul(h, 2), 1) <> map(MulTwoAddOne{}, t) : List<&2, U32>}\nContext:\n- h : U32\n- t : List<&2, U32>\nLocation: fuse\n51 |     case h <> t:\n52>|       %fuse(t) : {U32.add(U32.mul(h, 2), 1) <> map(AddOne{}, map(MulTwo{}, t)) == U32.add(U32.mul(h, 2), 1) <> _ : List<&2, U32>}\n53 |       {==}\n",
    );
}

#[test]
fn rewritten_imports_templates_and_nested_goals_keep_the_rejected_origin() {
    for (main, helper, expected) in [
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(n: N, e: {n == n : N}) -> N:\n  %e : {_ == _ : N}\n  {==}\n",
            None,
            "Error:\n- expected : N\n- observed : {n == n : N}\nContext:\n- n : N\n- e : {n == n : N}\nLocation: bad\n4 | def bad(n: N, e: {n == n : N}) -> N:\n5>|   %e : {_ == _ : N}\n6 |   {==}\n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.N: H.Z{}\n",
            Some(
                "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(n: N, e: {n == n : N}) -> N:\n  %e : {_ == _ : N}\n  {==}\n",
            ),
            "Error:\n- expected : helper.N\n- observed : {n == n : helper.N}\nContext:\n- n : helper.N\n- e : {n == n : helper.N}\nLocation: helper.bad\n4 | def bad(n: N, e: {n == n : N}) -> N:\n5>|   %e : {_ == _ : N}\n6 |   {==}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(~n: N, e: {n == n : N}) -> N:\n  %e : {_ == _ : N}\n  {==}\ndef main() -> N: bad(Z{}, {==})\n",
            None,
            "Error:\n- expected : N\n- observed : {n == n : N}\nContext:\n- n : N\n- e : {n == n : N}\nLocation: bad~0\n4 | def bad(~n: N, e: {n == n : N}) -> N:\n5>|   %e : {_ == _ : N}\n6 |   {==}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(n: N, e: {n == n : N}) -> N:\n  %e : N\n  %e : {_ == _ : N}\n  {==}\n",
            None,
            "Error:\n- expected : N\n- observed : {n == n : N}\nContext:\n- n : N\n- e : {n == n : N}\nLocation: bad\n5 |   %e : N\n6>|   %e : {_ == _ : N}\n7 |   {==}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(n: N, e: {n == n : N}) -> N:\n  %e : N\n  Wrong{}\n",
            None,
            "Error:\n- expected : a declared constructor (N declares Z, S)\n- observed : Wrong{}\nContext:\n- n : N\n- e : {n == n : N}\nLocation: bad\n5 |   %e : N\n6>|   Wrong{}\n7 | \n",
        ),
    ] {
        let fixture = Fixture::new(main);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn local_binding_rejections_use_the_original_binder_source() {
    let original = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-destructure/destructure_let_001.bend");
    assert_diagnostic(
        &original,
        "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n13 | def main(p):\n14>|   q = p\n15 |   P{a, b} = q\n",
    );
    for (source, helper, expected) in [
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef main(n: N) -> N:\n  q = n\n  match q:\n    case Z{}: Z{}\n    case S{p}: p\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n4 | def main(n: N) -> N:\n5>|   q = n\n6 |   match q:\n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.N: H.Z{}\n",
            Some(
                "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(n: N) -> N:\n  q = n\n  match q:\n    case Z{}: Z{}\n    case S{p}: p\n",
            ),
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n4 | def bad(n: N) -> N:\n5>|   q = n\n6 |   match q:\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef main(n: N) -> N:\n  q =\n    S{n}\n  match q:\n    case Z{}: Z{}\n    case S{p}: p\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n4 | def main(n: N) -> N:\n5>|   q =\n6 |     S{n}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef main(n: N) -> N:\n  match n:\n    case Z{}: Z{}\n    case S{p}:\n      q = p\n      match q:\n        case Z{}: Z{}\n        case S{k}: k\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n7 |     case S{p}:\n8>|       q = p\n9 |       match q:\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(~k: N, n: N) -> N:\n  q = n\n  match q:\n    case Z{}: k\n    case S{p}: p\ndef main(n: N) -> N: bad(Z{}, n)\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n4 | def bad(~k: N, n: N) -> N:\n5>|   q = n\n6 |   match q:\n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn unbound_reusable_markers_keep_context_source_and_first_error() {
    let original = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-plus-binders/plus_binder_term.bend");
    assert_diagnostic(
        &original,
        "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: main\n6 |   x = U32.add(1, 1)\n7>|   U32.add(+x, 1)\n8 | \n",
    );
    for (source, helper, expected) in [
        (
            "import Base\ndef main() -> {0n == 1n : Nat}: {==}\n",
            None,
            "Error:\n- expected : 0n\n- observed : 1n\nLocation: main\n1 | \n2>| def main() -> {0n == 1n : Nat}: {==}\n3 | \n",
        ),
        (
            "import Base\ndef main() -> Nat: Missing{}\n",
            None,
            "Error:\n- expected : a declared constructor (Nat declares Zero, Succ)\n- observed : Missing{}\nLocation: main\n1 | \n2>| def main() -> Nat: Missing{}\n3 | \n",
        ),
        (
            "import Base\r\ndef main(x: U32) -> U32: +x\r\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: main\n1 | \n2>| def main(x: U32) -> U32: +x\r\n3 | \n",
        ),
        (
            "import Base\ndef main(x: U32) -> U32: +x",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: main\n1 | \n2>| def main(x: U32) -> U32: +x\n",
        ),
        (
            "import Base\ndef main(x: U32) -> U32: +x\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: main\n1 | \n2>| def main(x: U32) -> U32: +x\n3 | \n",
        ),
        (
            "import Base\ntype Box is Data:\n  Box{v: U32}\ndef main(b: Box) -> U32:\n  match b:\n    case Box{x}: +x\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: main\n5 |   match b:\n6>|     case Box{x}: +x\n7 | \n",
        ),
        (
            "import Base\nimport helper.bend as H\ndef main() -> U32: 0\n",
            Some("import Base\ndef bad(x: U32) -> U32: +x\n"),
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: helper.bad\n1 | \n2>| def bad(x: U32) -> U32: +x\n3 | \n",
        ),
        (
            "import Base\ndef main(x: U32, y: U32) -> U32: U32.add(+x, +y)\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\n- y : U32\nLocation: main\n1 | \n2>| def main(x: U32, y: U32) -> U32: U32.add(+x, +y)\n3 | \n",
        ),
        (
            "import Base\ndef main(x: U32) -> U32:\n  x = U32.inc(x)\n  +x\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\n- x : U32\nLocation: main\n3 |   x = U32.inc(x)\n4>|   +x\n5 | \n",
        ),
        (
            "import Base\ndef bad(~n: U32, x: U32) -> U32: U32.add(+x, n)\ndef main() -> U32: bad(0, 1)\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- n : U32\n- x : U32\nLocation: bad~0\n1 | \n2>| def bad(~n: U32, x: U32) -> U32: U32.add(+x, n)\n3 | def main() -> U32: bad(0, 1)\n",
        ),
        (
            "import Base\ndef main() -> U32: +missing\n",
            None,
            "Error:\n- expected : a bound variable\n- observed : missing\nLocation: main\n1 | \n2>| def main() -> U32: +missing\n3 | \n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn original_rewrite_failures_preserve_phase_and_exact_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-rewrite-checking");
    for (name, expected) in [
        (
            "rewrite_non_equation",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : 5\nLocation: main\n 9 | \n10>| def main(): %5 : U32; 1\n11 | \n",
        ),
        (
            "rewrite_ctor_brace",
            "Error:\n- expected : @_:Opt8<Un8> -> Opt8<Un8>\n- observed : @_:Opt8<Emp8> -> Opt8<Emp8>\nContext:\n- e : {(x => x) == (x => Zer8{}) : @_:Opt8<Emp8> -> Opt8<Emp8>}\n- _ : @_:Opt8<Emp8> -> Opt8<Emp8>\n- q : {(x => x) == _ : @_:Opt8<Emp8> -> Opt8<Emp8>}\nLocation: bad_fun\n21 | def bad_fun(e):\n22>|   %q@e : {(x => x) == _ : Opt8<Un8> -> Opt8<Un8>}; {==}\n23 | \n",
        ),
        (
            "dead_motive_syntax",
            "Error:\n- expected : ':'\n- observed : 'e'\nLocation:\n12 | def sym3(x, y, e):\n13>|   %[h => {y == h : U32}] e {=}\n14 | \n",
        ),
    ] {
        let path = root.join(format!("{name}.bend"));
        if name == "dead_motive_syntax" {
            teamy_bend::syntax::load(&path).unwrap_err();
        } else {
            let book = teamy_bend::syntax::load(&path).unwrap();
            teamy_bend::kernel::check_book(&book).unwrap_err();
        }
        assert_diagnostic(&path, expected);
    }
}

#[test]
fn rewritten_operand_locations_and_token_errors_survive_imports_and_layout() {
    for (source, helper, expected) in [
        (
            "import Base\ndef main() -> U32:\n  %5 : U32; 1\n",
            None,
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : 5\nLocation: main\n2 | def main() -> U32:\n3>|   %5 : U32; 1\n4 | \n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some("import Base\ndef bad() -> U32:\n  %5 : U32; 1\n"),
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : 5\nLocation: helper.bad\n2 | def bad() -> U32:\n3>|   %5 : U32; 1\n4 | \n",
        ),
        (
            "# Cross-type (\"untyped global\") rewrite -- the faithful attack. Given e\n# at carrier Opt(Empty)->Opt(Empty), an untyped substitution would\n# coerce it into an Opt(Unit)->Opt(Unit) goal. The type-checked motive\n# REJECTS this: the abstracted motive variable cannot have both carrier\n# types.\n# ported from bend3: bend's rewrite spells the motive explicitly, and\n# the guard fires as the carrier mismatch\ntype Emp8 is Data:\n\ntype Un8 is Data:\n  Uu8{}\n\ntype Opt8<-A: Data> is Data:\n  Zer8{}\n  One8{x: A}\n\nlaw bad_fun:\n  for e: {(x => x) == (x => Zer8{}) : Opt8<Emp8> -> Opt8<Emp8>}\n  {(x => x) == (x => Zer8{}) : Opt8<Un8> -> Opt8<Un8>}\n\ndef bad_fun(e):\n  %q@e : {\n    (x => x) ==\n    _\n    : Opt8<Un8> -> Opt8<Un8>\n  }; {==}\n\nlaw main:\n  Un8\n\ndef main(): Uu8{}\n\n#|Error:\n#|- expected : @_:Opt8<Un8> -> Opt8<Un8>\n#|- observed : @_:Opt8<Emp8> -> Opt8<Emp8>\n#|Context:\n#|- e : {(x => x) == (x => Zer8{}) : @_:Opt8<Emp8> -> Opt8<Emp8>}\n#|- _ : @_:Opt8<Emp8> -> Opt8<Emp8>\n#|- q : {(x => x) == _ : @_:Opt8<Emp8> -> Opt8<Emp8>}\n#|Location: bad_fun\n#|21 | def bad_fun(e):\n#|22>|   %q@e : {(x => x) == _ : Opt8<Un8> -> Opt8<Un8>}; {==}\n#|23 |\n#|exit 1\n",
            None,
            "Error:\n- expected : @_:Opt8<Un8> -> Opt8<Un8>\n- observed : @_:Opt8<Emp8> -> Opt8<Emp8>\nContext:\n- e : {(x => x) == (x => Zer8{}) : @_:Opt8<Emp8> -> Opt8<Emp8>}\n- _ : @_:Opt8<Emp8> -> Opt8<Emp8>\n- q : {(x => x) == _ : @_:Opt8<Emp8> -> Opt8<Emp8>}\nLocation: bad_fun\n23 |     (x => x) ==\n24>|     _\n25 |     : Opt8<Un8> -> Opt8<Un8>\n",
        ),
        (
            "import helper.bend as H\ndef main() -> H.Un8: H.Uu8{}\n",
            Some(
                "# Cross-type (\"untyped global\") rewrite -- the faithful attack. Given e\n# at carrier Opt(Empty)->Opt(Empty), an untyped substitution would\n# coerce it into an Opt(Unit)->Opt(Unit) goal. The type-checked motive\n# REJECTS this: the abstracted motive variable cannot have both carrier\n# types.\n# ported from bend3: bend's rewrite spells the motive explicitly, and\n# the guard fires as the carrier mismatch\ntype Emp8 is Data:\n\ntype Un8 is Data:\n  Uu8{}\n\ntype Opt8<-A: Data> is Data:\n  Zer8{}\n  One8{x: A}\n\nlaw bad_fun:\n  for e: {(x => x) == (x => Zer8{}) : Opt8<Emp8> -> Opt8<Emp8>}\n  {(x => x) == (x => Zer8{}) : Opt8<Un8> -> Opt8<Un8>}\n\ndef bad_fun(e):\n  %q@e : {(x => x) == _ : Opt8<Un8> -> Opt8<Un8>}; {==}\n\n\n\n#|Error:\n#|- expected : @_:Opt8<Un8> -> Opt8<Un8>\n#|- observed : @_:Opt8<Emp8> -> Opt8<Emp8>\n#|Context:\n#|- e : {(x => x) == (x => Zer8{}) : @_:Opt8<Emp8> -> Opt8<Emp8>}\n#|- _ : @_:Opt8<Emp8> -> Opt8<Emp8>\n#|- q : {(x => x) == _ : @_:Opt8<Emp8> -> Opt8<Emp8>}\n#|Location: bad_fun\n#|21 | def bad_fun(e):\n#|22>|   %q@e : {(x => x) == _ : Opt8<Un8> -> Opt8<Un8>}; {==}\n#|23 |\n#|exit 1\n",
            ),
            "Error:\n- expected : @_:helper.Opt8<helper.Un8> -> helper.Opt8<helper.Un8>\n- observed : @_:helper.Opt8<helper.Emp8> -> helper.Opt8<helper.Emp8>\nContext:\n- e : {(x => x) == (x => helper.Zer8{}) : @_:helper.Opt8<helper.Emp8> -> helper.Opt8<helper.Emp8>}\n- _ : @_:helper.Opt8<helper.Emp8> -> helper.Opt8<helper.Emp8>\n- q : {(x => x) == _ : @_:helper.Opt8<helper.Emp8> -> helper.Opt8<helper.Emp8>}\nLocation: helper.bad_fun\n21 | def bad_fun(e):\n22>|   %q@e : {(x => x) == _ : Opt8<Un8> -> Opt8<Un8>}; {==}\n23 | \n",
        ),
        (
            "type A is Data:\n  A0{}\ntype B is Data:\n  B0{}\ndef bad(x: A, e: {x == x : A}) -> Type:\n  %e : {\n    {_ == _ : A} ==\n    {_ == _ : B} : Type\n  }; {==}\n",
            None,
            "Error:\n- expected : B\n- observed : A\nContext:\n- x : A\n- e : {x == x : A}\n- _ : A\n-   : {x == _ : A}\nLocation: bad\n7 |     {_ == _ : A} ==\n8>|     {_ == _ : B} : Type\n9 |   }; {==}\n",
        ),
        (
            "import Base\ndef main() -> U32:\n  %5 elephant\n",
            None,
            "Error:\n- expected : ':'\n- observed : 'e'\nLocation:\n2 | def main() -> U32:\n3>|   %5 elephant\n4 | \n",
        ),
        (
            "import Base\ndef main() -> U32: %5",
            None,
            "Error:\n- expected : ':'\n- observed : end of input\nLocation:\n1 | \n2>| def main() -> U32: %5\n",
        ),
        (
            "import Base\r\ndef main() -> U32:\r\n  %5 elephant\r\n",
            None,
            "Error:\n- expected : ':'\n- observed : 'e'\nLocation:\n2 | def main() -> U32:\r\n3>|   %5 elephant\r\n4 | \n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some("import Base\ndef bad() -> U32:\n  %5 elephant\n"),
            "Error:\n- expected : ':'\n- observed : 'e'\nLocation:\n2 | def bad() -> U32:\n3>|   %5 elephant\n4 | \n",
        ),
        (
            "import Base\ndef main() -> U32: {0 : U32]\n",
            None,
            "Error:\n- expected : '}'\n- observed : ']'\nLocation:\n1 | \n2>| def main() -> U32: {0 : U32]\n3 | \n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn rewrite_proof_originals_reject_during_checking_with_exact_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-rewrite-proof");
    for (name, expected) in [
        (
            "rewrite_non_equation",
            "Error:\n- expected : an equation {a == b : T}\n- observed : U32\nContext:\n- e : U32\nLocation: f\n 8 | def f(e):\n 9>|   %e : U32\n10 |   3\n",
        ),
        (
            "error_window_rewrite_proof",
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- h : {Type == _ : Type}\nLocation: bad\n3 | def bad(e: {Type == Type : Type}) -> Type:\n4>|   %h@e : h\n5 |   Type\n",
        ),
        (
            "missing_ih_rewrite",
            "Error:\n- expected : Sh{add(p, Zh{})}\n- observed : Sh{p}\nContext:\n- p : Nh\nLocation: add_zero\n28 |     case Sh{p}:\n29>|       {==}\n30 | \n",
        ),
        (
            "reject_rewrite",
            "Error:\n- expected : a function type\n- observed : %e : Kind(_); @_:Nat -> Nat\nContext:\n- e : {&1 == &2 : Quant}\n- f : %e : Kind(_); @_:Nat -> Nat\nLocation: cheat\n22 | def cheat(e, f):\n23>|   f(f(Zero{}))\n24 | \n",
        ),
        (
            "rewrite_goal",
            "Error:\n- expected : S{plus1(p, x)}\n- observed : plus1(p, S{x})\nContext:\n- p : MN\n- x : MN\nLocation: spec_thm\n46 |       %spec_thm(p, S{x}) : {S{plus1(p, x)} == _ : MN}\n47>|       {==}\n48 | \n",
        ),
    ] {
        let path = root.join(format!("{name}.bend"));
        let book = teamy_bend::syntax::load(&path).unwrap();
        teamy_bend::kernel::check_book(&book).unwrap_err();
        assert_diagnostic(&path, expected);
    }
}

#[test]
fn rewrite_proof_and_application_origins_preserve_use_sites_and_first_errors() {
    for (source, helper, expected) in [
        (
            "import Base\ndef f(e: U32) -> U32:\n  %{e : U32} : U32; 3\n",
            None,
            "Error:\n- expected : an equation {a == b : T}\n- observed : U32\nContext:\n- e : U32\nLocation: f\n2 | def f(e: U32) -> U32:\n3>|   %{e : U32} : U32; 3\n4 | \n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some("import Base\ndef bad(e: U32) -> U32:\n  %e : U32; 3\n"),
            "Error:\n- expected : an equation {a == b : T}\n- observed : U32\nContext:\n- e : U32\nLocation: helper.bad\n2 | def bad(e: U32) -> U32:\n3>|   %e : U32; 3\n4 | \n",
        ),
        (
            "import Base\ndef f(e: U32) -> U32:\n  %(\n    e\n  ) : U32; 3\n",
            None,
            "Error:\n- expected : an equation {a == b : T}\n- observed : U32\nContext:\n- e : U32\nLocation: f\n3 |   %(\n4>|     e\n5 |   ) : U32; 3\n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some("import Base\ndef bad(n: U32) -> U32: n({Type : U32})\n"),
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: helper.bad\n1 | \n2>| def bad(n: U32) -> U32: n({Type : U32})\n3 | \n",
        ),
        (
            "import Base\ndef f(n: U32) -> U32:\n  n(\n    0\n  )\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: f\n2 | def f(n: U32) -> U32:\n3>|   n(\n4 |     0\n",
        ),
        (
            "import Base\ndef bad(n: U32) -> U32: n(0, {Type : U32})\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: bad\n1 | \n2>| def bad(n: U32) -> U32: n(0, {Type : U32})\n3 | \n",
        ),
        (
            "import Base\ndef bad(n: U32) -> U32:\n  (\n    n\n  )(0)\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: bad\n3 |   (\n4>|     n\n5 |   )(0)\n",
        ),
        (
            "import Base\ndef bad(n: U32) -> U32: n({Type : U32})\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: bad\n1 | \n2>| def bad(n: U32) -> U32: n({Type : U32})\n3 | \n",
        ),
        (
            "def bad(e: {Type == Type : Type}) -> Type:\r\n  %h@e : h\r\n  Type\r\n",
            None,
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- h : {Type == _ : Type}\nLocation: bad\n1 | def bad(e: {Type == Type : Type}) -> Type:\r\n2>|   %h@e : h\r\n3 |   Type\r\n",
        ),
        (
            "def bad(e: {Type == Type : Type}) -> Type:\n  %h@e : {\n    h == h : Type\n  }\n  Type\n",
            None,
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- h : {Type == _ : Type}\nLocation: bad\n2 |   %h@e : {\n3>|     h == h : Type\n4 |   }\n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some("def bad(e: {Type == Type : Type}) -> Type:\n  %h@e :\n    h\n  Type\n"),
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- h : {Type == _ : Type}\nLocation: helper.bad\n2 |   %h@e :\n3>|     h\n4 |   Type\n",
        ),
        (
            "def bad(e: {Type == Type : Type}) -> Type:\n  %h@e :\n    h\n  Type\n",
            None,
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- h : {Type == _ : Type}\nLocation: bad\n2 |   %h@e :\n3>|     h\n4 |   Type\n",
        ),
        (
            "def bad(e: {Type == Type : Type}) -> Type:\n  %h@e : Type\n  %q@e :\n    q\n  Type\n",
            None,
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- q : {Type == _ : Type}\nLocation: bad\n3 |   %q@e :\n4>|     q\n5 |   Type\n",
        ),
        (
            "def bad(e: {Type == Type : Type}) -> Type:\n  %h@e : (\n    h\n  )\n  Type\n",
            None,
            "Error:\n- expected : Type\n- observed : {Type == _ : Type}\nContext:\n- e : {Type == Type : Type}\n- _ : Type\n- h : {Type == _ : Type}\nLocation: bad\n2 |   %h@e : (\n3>|     h\n4 |   )\n",
        ),
        (
            "def bad(~A: Type, e: {A == A : Type}) -> Type:\n  %h@e :\n    h\n  Type\ndef main() -> Type: bad(Type, {==})\n",
            None,
            "Error:\n- expected : Type\n- observed : {A == _ : Type}\nContext:\n- A : Type\n- e : {A == A : Type}\n- _ : Type\n- h : {A == _ : Type}\nLocation: bad~0\n2 |   %h@e :\n3>|     h\n4 |   Type\n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn computed_match_originals_keep_exact_load_failures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-computed-scrutinee");
    for (name, expected) in [
        (
            "motive_suffix_join",
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n13 | def j9(n, e):\n14>|   match Bool.not(n):\n15 |     case False{}:\n",
        ),
        (
            "computed_match_split",
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n36 | def or_true(n):\n37>|   match isz(n):\n38 |     case True{}:\n",
        ),
    ] {
        let path = root.join(format!("{name}.bend"));
        teamy_bend::syntax::load(&path).unwrap_err();
        assert_diagnostic(&path, expected);
    }
}

#[test]
fn computed_scrutinees_and_holes_keep_source_and_first_error_contracts() {
    for (source, helper, expected) in [
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match {b : Bool}:\n    case True{}: 1\n    case False{}: 0\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(b: Bool) -> U32:\n3>|   match {b : Bool}:\n4 |     case True{}: 1\n",
        ),
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match Bool.not(b):\n    case True{}: 1\n    case False{}: 0\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(b: Bool) -> U32:\n3>|   match Bool.not(b):\n4 |     case True{}: 1\n",
        ),
        (
            "import Base\r\ndef bad(b: Bool) -> U32:\r\n  match Bool.not(b):\r\n    case True{}: 0\r\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(b: Bool) -> U32:\r\n3>|   match Bool.not(b):\r\n4 |     case True{}: 0\r\n",
        ),
        (
            "import Base\ndef bad(n: Nat) -> Nat:\n  Succ{p} = Nat.add(n, 1n)\n  p\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(n: Nat) -> Nat:\n3>|   Succ{p} = Nat.add(n, 1n)\n4 |   p\n",
        ),
        (
            "import Base\ndef bad() -> U32: ?missing\n",
            None,
            "Error:\n- expected : U32\n- observed : ?missing\nLocation: bad\n1 | \n2>| def bad() -> U32: ?missing\n3 | \n",
        ),
        (
            "import Base\ndef first(n: U32) -> U32: n(0)\ndef second() -> U32: ?later\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: first\n1 | \n2>| def first(n: U32) -> U32: n(0)\n3 | def second() -> U32: ?later\n",
        ),
        (
            "import Base\ntype Box is Data:\n  Box{x: ?field}\ndef main() -> U32: 0\n",
            None,
            "Error:\n- expected : Data\n- observed : ?field\nLocation: Box\n2 | type Box is Data:\n3>|   Box{x: ?field}\n4 | def main() -> U32: 0\n",
        ),
        (
            "import Base\ndef bad() -> U32: U32.add(?first, ?second)\n",
            None,
            "Error:\n- expected : U32\n- observed : ?first\nLocation: bad\n1 | \n2>| def bad() -> U32: U32.add(?first, ?second)\n3 | \n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some("import Base\ndef bad(n: U32) -> U32: n(?unused)\n"),
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: helper.bad\n1 | \n2>| def bad(n: U32) -> U32: n(?unused)\n3 | \n",
        ),
        (
            "import Base\ndef bad(n: U32) -> U32: n(0, ?unused)\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: bad\n1 | \n2>| def bad(n: U32) -> U32: n(0, ?unused)\n3 | \n",
        ),
        (
            "import Base\ndef bad(n: U32) -> U32: n(?unused)\n",
            None,
            "Error:\n- expected : a function type\n- observed : U32\nContext:\n- n : U32\nLocation: bad\n1 | \n2>| def bad(n: U32) -> U32: n(?unused)\n3 | \n",
        ),
        (
            "import Base\ndef bad() -> U32: ?missing(0)\n",
            None,
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : ?missing\nLocation: bad\n1 | \n2>| def bad() -> U32: ?missing(0)\n3 | \n",
        ),
        (
            "import Base\ndef first() -> U32: ?earlier\ndef second(n: U32) -> U32: n(0)\n",
            None,
            "Error:\n- expected : U32\n- observed : ?earlier\nLocation: first\n1 | \n2>| def first() -> U32: ?earlier\n3 | def second(n: U32) -> U32: n(0)\n",
        ),
        (
            "import Base\ndef bad(n: U32) -> U32:\n  U32.add(0,\n    ?later\n  )\n",
            None,
            "Error:\n- expected : U32\n- observed : ?later\nContext:\n- n : U32\nLocation: bad\n3 |   U32.add(0,\n4>|     ?later\n5 |   )\n",
        ),
        (
            "import Base\ndef bad(e: U32) -> U32:\n  %e : ?motive; ?body\n",
            None,
            "Error:\n- expected : an equation {a == b : T}\n- observed : U32\nContext:\n- e : U32\nLocation: bad\n2 | def bad(e: U32) -> U32:\n3>|   %e : ?motive; ?body\n4 | \n",
        ),
        (
            "def bad(x: ?signature) -> Type: Type\n",
            None,
            "Error:\n- expected : Type\n- observed : ?signature\nLocation: bad\n1>| def bad(x: ?signature) -> Type: Type\n2 | \n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some(
                "import Base\ndef bad(b: Bool) -> U32:\n  match Bool.not(b):\n    case True{}: 0\n",
            ),
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(b: Bool) -> U32:\n3>|   match Bool.not(b):\n4 |     case True{}: 0\n",
        ),
        (
            "import Base\ndef bad(a: Bool, b: Bool) -> U32:\n  match a, Bool.not(b):\n    case True{}, True{}: 1\n    case False{}, False{}: 0\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(a: Bool, b: Bool) -> U32:\n3>|   match a, Bool.not(b):\n4 |     case True{}, True{}: 1\n",
        ),
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match (\n    Bool.not(b)\n  ):\n    case True{}: 1\n    case False{}: 0\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n3 |   match (\n4>|     Bool.not(b)\n5 |   ):\n",
        ),
        (
            "import Base\ndef bad(n: Nat) -> U32:\n  match n:\n    case Zero{}: 0\n    case Succ{p}:\n      match Nat.is_zero(p):\n        case True{}: 1\n",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n5 |     case Succ{p}:\n6>|       match Nat.is_zero(p):\n7 |         case True{}: 1\n",
        ),
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match Bool.not(b):\n    case True{}: 0",
            None,
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)\nLocation:\n2 | def bad(b: Bool) -> U32:\n3>|   match Bool.not(b):\n4 |     case True{}: 0\n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}

#[test]
fn substituted_consumed_and_out_of_order_originals_keep_exact_load_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-scrutinee-origins");
    for (name, expected) in [
        (
            "error_window_substituted_scrutinee",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n11 |     case y, 1n+q:\n12>|       match y:\n13 |         case 0n:\n",
        ),
        (
            "consumed_column_error_001",
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n18 |     case Sw3{p}:\n19>|       match a:\n20 |         case Zw3{}:\n",
        ),
        (
            "order_ban_000",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n14 | def f(a, b):\n15>|   match b, a:\n16 |     case Zy{}, y:\n",
        ),
        (
            "order_ban_001",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n18 |     case Pp2{a, b}:\n19>|       match b, a:\n20 |         case Zy2{}, y:\n",
        ),
        (
            "order_ban_002",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n15 |   z = Zw{}\n16>|   Sw{p} = y\n17 |   p\n",
        ),
        (
            "order_ban_003",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n18 |       t = Zb{}\n19>|       match p:\n20 |         case Zb{}:\n",
        ),
        (
            "dup_guard_000",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n13 | def main(a):\n14>|   match a, a:\n15 |     case Z{}, x:\n",
        ),
        (
            "dup_guard_001",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n15 | def main(a):\n16>|   match a, a:\n17 |     case Z{}, x:\n",
        ),
        (
            "dead_residue_elision_003",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n24 |     case T{}:\n25>|       match x:\n26 |         case Z{}:\n",
        ),
        (
            "framed_cell_capture",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n17 | def frame(b):\n18>|   match a : Bool = b, c : Bool = a:\n19 |     case x, y:\n",
        ),
        (
            "column_order_search",
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n16 |     case Succ{p}:\n17>|       match n:\n18 |         case Zero{}:\n",
        ),
    ] {
        assert_diagnostic(&root.join(format!("{name}.bend")), expected);
    }
}

#[test]
fn scrutinee_occurrences_keep_match_or_use_locations_through_substitution() {
    for (source, helper, expected) in [
        (
            "import Base\ndef bad() -> U32:\n  match 0:\n    case Zero{}: 0\n",
            None,
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n2 | def bad() -> U32:\n3>|   match 0:\n4 |     case Zero{}: 0\n",
        ),
        (
            "import Base\ndef value() -> Bool: True{}\ndef bad() -> U32:\n  match value:\n    case True{}: 0\n",
            None,
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n3 | def bad() -> U32:\n4>|   match value:\n5 |     case True{}: 0\n",
        ),
        (
            "import Base\ndef bad() -> Nat:\n  match (\n    Zero{}\n  ):\n    case Zero{}: 0n\n",
            None,
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n2 | def bad() -> Nat:\n3>|   match (\n4 |     Zero{}\n",
        ),
        (
            "import Base\ndef value() -> Nat: 0n\ndef bad() -> Nat:\n  match (\n    value\n  ):\n    case Zero{}: 0n\n",
            None,
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n4 |   match (\n5>|     value\n6 |   ):\n",
        ),
        (
            "import Base\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{p}:\n      match (\n        n\n      ):\n        case Zero{}: 0n\n",
            None,
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n4 |     case Succ{p}:\n5>|       match (\n6 |         n\n",
        ),
        (
            "import Base\ndef bad(a: Nat, b: Nat) -> Nat:\n  match a, b:\n    case y, Succ{q}:\n      match (\n        y\n      ):\n        case Zero{}: 0n\n",
            None,
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n5 |       match (\n6>|         y\n7 |       ):\n",
        ),
        (
            "import Base\ndef bad(a: Nat, b: Nat) -> Nat:\n  match b, (\n    a\n  ):\n    case Zero{}, y: 0n\n",
            None,
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n3 |   match b, (\n4>|     a\n5 |   ):\n",
        ),
        (
            "import Base\ndef bad(a: Nat, b: Nat) -> Nat:\n  match a, b:\n    case y, Succ{q}:\n      Succ{p} = (\n        y\n      )\n      p\n",
            None,
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n5 |       Succ{p} = (\n6>|         y\n7 |       )\n",
        ),
        (
            "import Base\ndef bad() -> Nat:\n  Succ{p} = (\n    Succ{Zero{}}\n  )\n  p\n",
            None,
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n3 |   Succ{p} = (\n4>|     Succ{Zero{}}\n5 |   )\n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> Nat: 0n\n",
            Some(
                "import Base\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{p}:\n      match n:\n        case Zero{}: 0n\n",
            ),
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n4 |     case Succ{p}:\n5>|       match n:\n6 |         case Zero{}: 0n\n",
        ),
        (
            "import Base\ndef bad(~f: Nat -> Nat, a: Nat, b: Nat) -> Nat:\n  match b, a:\n    case Zero{}, y:\n      match y:\n        case Zero{}: 0n\ndef main() -> Nat: bad(~(x => x), 0n, 0n)\n",
            None,
            "Error:\n- message  : match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)\nLocation:\n2 | def bad(~f: Nat -> Nat, a: Nat, b: Nat) -> Nat:\n3>|   match b, a:\n4 |     case Zero{}, y:\n",
        ),
        (
            "import Base\r\ndef bad() -> Nat:\r\n  match Zero{}:\r\n    case Zero{}: 0n",
            None,
            "Error:\n- message  : an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)\nLocation:\n2 | def bad() -> Nat:\r\n3>|   match Zero{}:\r\n4 |     case Zero{}: 0n\n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}
