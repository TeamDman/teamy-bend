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

#[test]
fn matched_value_and_row_arity_originals_keep_exact_failure_contracts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-matched-values");
    for (name, expected) in [
        (
            "error_window_matched_scrutinee",
            "Error:\n- expected : Type\n- observed : One\nLocation: bad\n7 |     case One.new{}:\n8>|       x\n9 | \n",
        ),
        (
            "chained_switch_conflation_000",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n16 |       20\n17>|     case 2+q:\n18 |       q\n",
        ),
        (
            "chained_switch_conflation_001",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n16 |       1\n17>|     case 2+q:\n18 |       (2 + q : U32)\n",
        ),
        (
            "dec_switch_respell_000",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n20 |           h\n21>|         case 1+p:\n22 |           nth(d, p, t)\n",
        ),
        (
            "dec_switch_respell_001",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n25 |           0\n26>|     case 2+p:\n27 |       p\n",
        ),
        (
            "dependent_switch_induction",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n15 |       0\n16>|     case 1+p:\n17 |       down(p)\n",
        ),
        (
            "fuel_inline_dep_match",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n15 |       h\n16>|     case 1+d:\n17 |       stack(d, (h * 5 : U32))\n",
        ),
        (
            "match_as_specialize",
            "Error:\n- expected : 3 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n12 |   match x as e:\n13>|     case Zero{}:\n14 |       h\n",
        ),
        (
            "arity_mismatch_000",
            "Error:\n- expected : 2 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n18 |   match a, b:\n19>|     case Zw{}, x, Zw{}:\n20 |       Aw{x}\n",
        ),
        (
            "arity_mismatch_001",
            "Error:\n- expected : 2 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n16 |   match a, b:\n17>|     case Zw2{}:\n18 |       Aw2{}\n",
        ),
        (
            "as_pattern_error",
            "Error:\n- expected : 4 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n12 |   match Bool.not(a), b as e:\n13>|     case True{}, True{}:\n14 |       1\n",
        ),
        (
            "kplus_missing_case",
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n14 |       0\n15>|     case 2+k:\n16 |       k\n",
        ),
        (
            "match_as_binding",
            "Error:\n- expected : 3 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n27 |   match nat_eq(h, i) as e:\n28>|     case True{}:\n29 |       h\n",
        ),
        (
            "hypothesis_retype",
            "Error:\n- expected : 3 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n32 |   match nat_eq(n, m) as e:\n33>|     case True{}:\n34 |       use(True{}, False{}, h)\n",
        ),
    ] {
        assert_diagnostic(&root.join(format!("{name}.bend")), expected);
    }
}

#[test]
fn matched_value_use_sites_and_row_arity_characters_survive_layout_and_imports() {
    for (source, helper, expected) in [
        (
            "type One is Data:\n  One.new{}\ndef bad(x: One) -> Type:\n  match x:\n    case One.new{}:\n      x\n",
            None,
            "Error:\n- expected : Type\n- observed : One\nLocation: bad\n5 |     case One.new{}:\n6>|       x\n7 | \n",
        ),
        (
            "type One is Data:\n  One.new{}\ndef bad(x: One) -> Type:\n  match x:\n    case One.new{}:\n      -ok = {Type : Type}\n      x\n",
            None,
            "Error:\n- expected : Type\n- observed : One\nContext:\n- ok : Type\nLocation: bad\n6 |       -ok = {Type : Type}\n7>|       x\n8 | \n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Type:\n  match n:\n    case Zero{}: Type\n    case Succ{p}:\n      n\n",
            None,
            "Error:\n- expected : Type\n- observed : Nat\nContext:\n- p : Nat\nLocation: bad\n7 |     case Succ{p}:\n8>|       n\n9 | \n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Type:\n  match n:\n    case Zero{}: Type\n    case Succ{p}:\n      match p:\n        case Zero{}: Type\n        case Succ{q}:\n          n\n",
            None,
            "Error:\n- expected : Type\n- observed : Nat\nContext:\n- q : Nat\nLocation: bad\n10 |         case Succ{q}:\n11>|           n\n12 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type One is Data:\n  One.new{}\ndef bad(x: One) -> Type:\n  match x:\n    case One.new{}:\n      x\n",
            ),
            "Error:\n- expected : Type\n- observed : helper.One\nLocation: helper.bad\n5 |     case One.new{}:\n6>|       x\n7 | \n",
        ),
        (
            "type One is Data:\n  One.new{}\ndef bad(~f: Type -> Type, x: One) -> Type:\n  match x:\n    case One.new{}:\n      x\ndef main() -> Type: bad(~(y => y), One.new{})\n",
            None,
            "Error:\n- expected : Type\n- observed : One\nContext:\n- f : @_:Type -> Type\nLocation: bad~0\n5 |     case One.new{}:\n6>|       x\n7 | def main() -> Type: bad(~(y => y), One.new{})\n",
        ),
        (
            "type One is Data:\n  One.new{}\ndef bad() -> Type:\n  One.new{}\n",
            None,
            "Error:\n- expected : Type\n- observed : One\nLocation: bad\n3 | def bad() -> Type:\n4>|   One.new{}\n5 | \n",
        ),
        (
            "def bad() -> Type:\n  Missing{}\n",
            None,
            "Error:\n- expected : Type\n- observed : non-inferrable term 'Missing{}'\nLocation: bad\n1 | def bad() -> Type:\n2>|   Missing{}\n3 | \n",
        ),
        (
            "type One is Data:\n  One.new{}\ndef bad() -> (Type -> Type):\n  One.new{}\n",
            None,
            "Error:\n- expected : @_:Type -> Type\n- observed : One\nLocation: bad\n3 | def bad() -> (Type -> Type):\n4>|   One.new{}\n5 | \n",
        ),
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match b:\n    case True{}, False{}: 0\n",
            None,
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : ' '\nLocation:\n3 |   match b:\n4>|     case True{}, False{}: 0\n5 | \n",
        ),
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match b:\n    case True{},\n         False{}:\n      0\n",
            None,
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n4 |     case True{},\n5>|          False{}:\n6 |       0\n",
        ),
        (
            "import Base\r\ndef bad(b: Bool) -> U32:\r\n  match b:\r\n    case True{}, False{}:\r\n      0\r\n",
            None,
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\r'\nLocation:\n3 |   match b:\r\n4>|     case True{}, False{}:\r\n5 |       0\r\n",
        ),
        (
            "import Base\ndef bad(b: Bool) -> U32:\n  match b:\n    case True{}, False{}:",
            None,
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : end of input\nLocation:\n3 |   match b:\n4>|     case True{}, False{}:\n",
        ),
        (
            "import helper.bend as H\nimport Base\ndef main() -> U32: 0\n",
            Some(
                "import Base\ndef bad(b: Bool) -> U32:\n  match b:\n    case True{}, False{}:\n      0\n",
            ),
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n3 |   match b:\n4>|     case True{}, False{}:\n5 |       0\n",
        ),
        (
            "import Base\ndef bad(~f: U32 -> U32, b: Bool) -> U32:\n  match b:\n    case True{}, False{}:\n      0\ndef main() -> U32: bad(~(x => x), True{})\n",
            None,
            "Error:\n- expected : 1 patterns (one per scrutinee)\n- observed : '\n'\nLocation:\n3 |   match b:\n4>|     case True{}, False{}:\n5 |       0\n",
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
fn match_failure_originals_preserve_phase_source_and_exact_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-match-failures");
    for (name, expected) in [
        (
            "efq_erased_binder_000",
            "Error:\n- expected : cases for B1\n- observed : \\{}\nContext:\n- v : Void\nLocation: bad\n20 |     case Pack{v, b}:\n21>|       match b:\n22 |         case B0{}: B0{}\n",
        ),
        (
            "error_window_match_missing",
            "Error:\n- expected : cases for False\n- observed : \\{}\nLocation: bad\n6 | def bad() -> Bool -> Bool:\n7>|   \\{True: True{}}\n8 | \n",
        ),
        (
            "annotated_computed_scrutinee_001",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{Sn: (k => T{}); Zn: F{}}\nContext:\n- n : Nt\nLocation: main\n15 | def main(n):\n16>|   \\{Sn: (k => T{}); Zn: F{}}({Sn{n} : Nt})\n17 | \n",
        ),
        (
            "dependent_goal_000",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{T: {==}; F: {==}}\nContext:\n- n : W\nLocation: main\n39 | def main(n):\n40>|   \\{T: {==}; F: {==}}(even(n))\n41 | \n",
        ),
        (
            "dependent_goal_001",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{T: \\{T: {==}; F: {==}}(odd(n)); F: \\{T: {==}; F: {==}}(odd(n))}\nContext:\n- n : W\nLocation: main\n62 | def main(n):\n63>|   \\{T: \\{T: {==}; F: {==}}(odd(n)); F: \\{T: {==}; F: {==}}(odd(n))}(pos(n))\n64 | \n",
        ),
        (
            "empty_match",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{}\nContext:\n- e : Emp\nLocation: main\n 9 | def main(e):\n10>|   (\\{})(e)\n11 | \n",
        ),
        (
            "lambda_match_literal_000",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{T: W; F: W}\nContext:\n- b : Bl\nLocation: main\n11 | law main:\n12>|   @b: Bl -> \\{T: W; F: W}(b)\n13 | \n",
        ),
        (
            "lambda_match_literal_001",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{T: {n == n : Nt}; F: Nt}\nContext:\n- n : Nt\n- b : Bl\nLocation: main\n11 | law main:\n12>|   @n: Nt -> @b: Bl -> \\{T: {n == n : Nt}; F: Nt}(b)\n13 | \n",
        ),
        (
            "lambda_match_literal_002",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{T: @x:W -> W; F: W}\nContext:\n- b : Bl\nLocation: main\n11 | law main:\n12>|   @b: Bl -> \\{T: @x: W -> W; F: W}(b)\n13 | \n",
        ),
        (
            "match_app_scrutinee_000",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{Zn: A{}; Sn: (x => B{x})}\nContext:\n- n : Nt\nLocation: main\n15 | def main(n):\n16>|   (\\{Zn: A{}; Sn: (x => B{x})})({n : Nt})\n17 | \n",
        ),
        (
            "match_app_scrutinee_001",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{Z: b; S: (d => d)}\nContext:\n- a : N\n- b : N\n- c : N\nLocation: main\n17 |       c = a\n18>|       (\\{Z:b;S: d =>d})(c)\n19 | \n",
        ),
        (
            "match_as_term",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : S{x}\nContext:\n- x : Nt\nLocation: main\n19 |     case T{}:\n20>|       y = S{x}\n21 |       y\n",
        ),
        (
            "noninfer_error_000",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{Zx5: Bx5{Zx5{}}; Sx5: (x => Ax5{x})}\nLocation: main\n14 | \n15>| def main(): (\\{Zx5: Bx5{Zx5{}}; Sx5: x => Ax5{x}})(Zx5{})\n16 | \n",
        ),
        (
            "noninfer_error_002",
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{Sn: (k => Z{})}\nContext:\n- n : Nt\nLocation: f\n15 | def f(n):\n16>|   \\{Sn: (k => Z{})}(Sn{n})\n17 | \n",
        ),
        (
            "nontotal_error_000",
            "Error:\n- expected : cases for Zx6\n- observed : \\{}\nLocation: f\n14 | def f(a):\n15>|   match a:\n16 |     case Sx6{a}:\n",
        ),
        (
            "nontotal_error_001",
            "Error:\n- expected : cases for Zx7\n- observed : \\{}\nLocation: f\n16 | def f(a):\n17>|   match a:\n18 |     case Sx7{Sx7{a}}:\n",
        ),
        (
            "nontotal_error_002",
            "Error:\n- expected : cases for Ze2\n- observed : \\{}\nLocation: f\n15 | def f(a):\n16>|   match a:\n17 |     case Se2{a}:\n",
        ),
        (
            "nontotal_error_003",
            "Error:\n- expected : cases for Ze3\n- observed : \\{}\nLocation: pred\n11 | def pred(n):\n12>|   match n:\n13 |     case Se3{p}:\n",
        ),
        (
            "nontotal_multi_missing",
            "Error:\n- expected : cases for Bb1, Cc1\n- observed : \\{}\nLocation: f\n12 | def f(x):\n13>|   match x:\n14 |     case Aa1{}:\n",
        ),
        (
            "nontotal_multi_scrutinee_000",
            "Error:\n- expected : cases for Zz0\n- observed : \\{}\nContext:\n- b : Lz0<Nz0>\nLocation: f\n25 | def f(a, b):\n26>|   match a, b:\n27 |     case Cl0{Sz0{a}, b}, Cl0{Sz0{c}, d}:\n",
        ),
        (
            "nontotal_multi_scrutinee_001",
            "Error:\n- expected : cases for Cl1\n- observed : \\{}\nContext:\n- b : Lz1<Nz1>\nLocation: f\n23 | def f(a, b, c):\n24>|   match a, b, c:\n25 |     case Nl1{}, a, Nl1{}:\n",
        ),
        (
            "nontotal_multi_scrutinee_002",
            "Error:\n- expected : cases for Cl2\n- observed : \\{}\nContext:\n- b : Lz2<Nz2>\n- c : Nz2\nLocation: f\n24 | def f(a, b, c, d):\n25>|   match a, b, c, d:\n26 |     case Zz2b{}, xs, b, Nl2{}:\n",
        ),
        (
            "nontotal_multi_scrutinee_003",
            "Error:\n- expected : cases for Ib\n- observed : \\{}\nLocation: mix\n13 | def mix(a, b):\n14>|   match a, b:\n15 |     case Ob{}, Ob{}:\n",
        ),
        (
            "nontotal_multi_scrutinee_004",
            "Error:\n- expected : cases for By, Cy\n- observed : \\{}\nLocation: f\n19 | def f(x, y):\n20>|   match x, y:\n21 |     case Oy{}, Ay{}:\n",
        ),
        (
            "pattern_shape_mismatch",
            "Error:\n- expected : cases for True\n- observed : \\{}\nLocation: main\n19 | def main(a):\n20>|   match a:\n21 |     case 0:\n",
        ),
        (
            "unsupported_pattern",
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : x => x\nLocation:\n16 |   match a:\n17>|     case x => x:\n18 |       Au{}\n",
        ),
        (
            "match_missing_ctors",
            "Error:\n- expected : cases for Tt, Ff\n- observed : \\{}\nLocation: f\n12 | def f(b):\n13>|   match b:\n14 | \n",
        ),
        (
            "match_missing_default",
            "Error:\n- expected : cases for U32\n- observed : \\{}\nLocation: main\n 9 | def main():\n10>|   \\{}\n11 | \n",
        ),
        (
            "invalid_assign_target",
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : f(1)\nLocation:\n16 | def main():\n17>|   f(1) = 3\n18 |   0\n",
        ),
    ] {
        assert_diagnostic(&root.join(format!("{name}.bend")), expected);
    }
}

#[test]
fn match_failures_keep_nested_imported_template_and_layout_origins() {
    for (source, helper, expected) in [
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{p}:\n      match p:\n        case Zero{}: Zero{}\n",
            None,
            "Error:\n- expected : cases for Succ\n- observed : \\{}\nLocation: bad\n6 |     case Succ{p}:\n7>|       match p:\n8 |         case Zero{}: Zero{}\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match (\n    n\n  ):\n    case Zero{}: Zero{}\n",
            None,
            "Error:\n- expected : cases for Succ\n- observed : \\{}\nLocation: bad\n4 | def bad(n: Nat) -> Nat:\n5>|   match (\n6 |     n\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  Succ{p} = n\n  p\n",
            None,
            "Error:\n- expected : cases for Zero\n- observed : \\{}\nLocation: bad\n4 | def bad(n: Nat) -> Nat:\n5>|   Succ{p} = n\n6 |   p\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad() -> Nat -> Nat:\n  \\{Zero: Zero{}}\n",
            None,
            "Error:\n- expected : cases for Succ\n- observed : \\{}\nLocation: bad\n4 | def bad() -> Nat -> Nat:\n5>|   \\{Zero: Zero{}}\n6 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{p}:\n      match p:\n        case Zero{}: Zero{}\n",
            ),
            "Error:\n- expected : cases for helper.Succ\n- observed : \\{}\nLocation: helper.bad\n6 |     case Succ{p}:\n7>|       match p:\n8 |         case Zero{}: Zero{}\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(~f: Type -> Type, n: Nat) -> Nat:\n  match n:\n    case Succ{p}:\n      match p:\n        case Zero{}: Zero{}\ndef main() -> Nat: bad(~(x => x), Zero{})\n",
            None,
            "Error:\n- expected : cases for Succ\n- observed : \\{}\nContext:\n- f : @_:Type -> Type\nLocation: bad~0\n6 |     case Succ{p}:\n7>|       match p:\n8 |         case Zero{}: Zero{}\n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  (\\{})(n)\n",
            None,
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{}\nContext:\n- n : Nat\nLocation: bad\n4 | def bad(n: Nat) -> Nat:\n5>|   (\\{})(n)\n6 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  (\\{})(n)\n",
            ),
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{}\nContext:\n- n : helper.Nat\nLocation: helper.bad\n4 | def bad(n: Nat) -> Nat:\n5>|   (\\{})(n)\n6 | \n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\n\ntype P is Data:\n  P{a: N, b: N}\n\n# a match application on a let-bound variable is the banned form too:\n# the checker rejects the application itself\nlaw main:\n  for p: P\n  N\n\ndef main(p):\n  match p:\n    case P{a, b}:\n      c = a\n      z = b\n      (\\{Z:b;S: d =>d})(c)\n\n",
            None,
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{Z: b; S: (d => d)}\nContext:\n- a : N\n- b : N\n- c : N\n- z : N\nLocation: main\n18 |       z = b\n19>|       (\\{Z:b;S: d =>d})(c)\n20 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type N is Data:\n  Z{}\n  S{p: N}\n\ntype P is Data:\n  P{a: N, b: N}\n\n# a match application on a let-bound variable is the banned form too:\n# the checker rejects the application itself\nlaw main:\n  for p: P\n  N\n\ndef main(p):\n  match p:\n    case P{a, b}:\n      c = a\n      (\\{Z:b;S: d =>d})(c)\n\n",
            ),
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : \\{helper.Z: b; helper.S: (d => d)}\nContext:\n- a : helper.N\n- b : helper.N\n- c : helper.N\nLocation: helper.main\n17 |       c = a\n18>|       (\\{Z:b;S: d =>d})(c)\n19 | \n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{x => x}: Zero{}\n",
            None,
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : x => x\nLocation:\n5 |   match n:\n6>|     case Succ{x => x}: Zero{}\n7 | \n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case {x : Nat}: Zero{}\n",
            None,
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : {x : Nat}\nLocation:\n5 |   match n:\n6>|     case {x : Nat}: Zero{}\n7 | \n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{x => x}: Zero{}\n",
            ),
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : x => x\nLocation:\n5 |   match n:\n6>|     case Succ{x => x}: Zero{}\n7 | \n",
        ),
        (
            "type Nat is Data:\r\n  Zero{}\r\n  Succ{pred: Nat}\r\ndef bad(n: Nat) -> Nat:\r\n  match n:\r\n    case Succ{x => x}: Zero{}\r\n",
            None,
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : x => x\nLocation:\n5 |   match n:\r\n6>|     case Succ{x => x}: Zero{}\r\n7 | \n",
        ),
        (
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad(n: Nat) -> Nat:\n  match n:\n    case Succ{x => x}: Zero{}",
            None,
            "Error:\n- expected : a pattern (a binder or a constructor)\n- observed : x => x\nLocation:\n5 |   match n:\n6>|     case Succ{x => x}: Zero{}\n",
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
fn match_guard_originals_keep_exact_rejections_and_locations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-match-guards");
    for (name, expected) in [
        (
            "match_not_datatype",
            "Error:\n- expected : a datatype\n- observed : @_:U32 -> U32\nLocation: f\n11 | def f(h):\n12>|   match h:\n13 |     case Z{}:\n",
        ),
        (
            "empty_match_datatype",
            "Error:\n- expected : a datatype\n- observed : @_:U32 -> U32\nLocation: main\n7 | \n8>| def main(): \\{}\n9 | \n",
        ),
        (
            "float_match_scrutinee",
            "Error:\n- expected : a constructor of F32 (missing, or already matched)\n- observed : \\{Chr: (c => 0)}\nLocation: f\n12 |   match x:\n13>|     case Chr{c}:\n14 |       0\n",
        ),
        (
            "error_window_match_repeated",
            "Error:\n- expected : a constructor of One (missing, or already matched)\n- observed : \\{One: One{}}\nLocation: bad\n5 | def bad() -> One -> One:\n6>|   \\{One: One{}; One: One{}}\n7 | \n",
        ),
        (
            "erased_match_empty",
            "Error:\n- message  : a live scrutinee (a - scrutinee matches only in a dead region)\nLocation: bad\n10 | \n11>| def bad(): \\{}\n12 | \n",
        ),
        (
            "erased_match_ctor",
            "Error:\n- message  : a live scrutinee (a - scrutinee matches only in a dead region)\nLocation: hidden\n10 | \n11>| def hidden(): \\{F8: T8{}; \\{T8: F8{}; \\{}}}\n12 | \n",
        ),
        (
            "unknown_constructor",
            "Error:\n- expected : a constructor of U32 (missing, or already matched)\n- observed : \\{Qux: 3}\nLocation: main\n6 | \n7>| def main(): \\{Qux: 3}\n8 | \n",
        ),
        (
            "bool_wrap_scope",
            "Error:\n- expected : a constructor of Bool (missing, or already matched)\n- observed : \\{Con: (h => t => h); Nil: 0}\nLocation: main.go\n12 |   match z:\n13>|     case h <> t:\n14 |       h\n",
        ),
    ] {
        assert_diagnostic(&root.join(format!("{name}.bend")), expected);
    }
}

#[test]
fn match_guard_reports_preserve_nested_imported_template_and_literal_locations() {
    for (source, helper, expected) in [
        (
            "def bad(f: Type -> Type) -> Type:\n  match (\n    f\n  ):\n",
            None,
            "Error:\n- expected : a datatype\n- observed : @_:Type -> Type\nLocation: bad\n1 | def bad(f: Type -> Type) -> Type:\n2>|   match (\n3 |     f\n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some("def bad(f: Type -> Type) -> Type:\n  match (\n    f\n  ):\n"),
            "Error:\n- expected : a datatype\n- observed : @_:Type -> Type\nLocation: helper.bad\n1 | def bad(f: Type -> Type) -> Type:\n2>|   match (\n3 |     f\n",
        ),
        (
            "def bad() -> Type:\n  \\{}\n",
            None,
            "Error:\n- expected : Type\n- observed : non-inferrable term '\\{}'\nLocation: bad\n1 | def bad() -> Type:\n2>|   \\{}\n3 | \n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad() -> Type:\n  \\{Z: Z{}}\n",
            None,
            "Error:\n- expected : Type\n- observed : non-inferrable term '\\{Z: Z{}}'\nLocation: bad\n4 | def bad() -> Type:\n5>|   \\{Z: Z{}}\n6 | \n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ntype Other is Data:\n  Other.new{}\ndef bad(n: N) -> N:\n  match n:\n    case Other.new{}:\n      Z{}\n",
            None,
            "Error:\n- expected : a constructor of N (missing, or already matched)\n- observed : \\{Other.new: Z{}}\nLocation: bad\n7 |   match n:\n8>|     case Other.new{}:\n9 |       Z{}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ntype Other is Data:\n  Other.new{}\ndef bad(n: N) -> N:\n  match n:\n    case S{\n      Other.new{}\n    }:\n      Z{}\n",
            None,
            "Error:\n- expected : a constructor of N (missing, or already matched)\n- observed : \\{Other.new: Z{}}\nLocation: bad\n 8 |     case S{\n 9>|       Other.new{}\n10 |     }:\n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type N is Data:\n  Z{}\n  S{p: N}\ntype Other is Data:\n  Other.new{}\ndef bad(n: N) -> N:\n  match n:\n    case S{\n      Other.new{}\n    }:\n      Z{}\n",
            ),
            "Error:\n- expected : a constructor of helper.N (missing, or already matched)\n- observed : \\{helper.Other.new: helper.Z{}}\nLocation: helper.bad\n 8 |     case S{\n 9>|       Other.new{}\n10 |     }:\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ntype Other is Data:\n  Other.new{}\ndef bad(~f: Type -> Type, n: N) -> N:\n  match n:\n    case Other.new{}:\n      Z{}\ndef main() -> N: bad(~(x => x), Z{})\n",
            None,
            "Error:\n- expected : a constructor of N (missing, or already matched)\n- observed : \\{Other.new: Z{}}\nContext:\n- f : @_:Type -> Type\nLocation: bad~0\n7 |   match n:\n8>|     case Other.new{}:\n9 |       Z{}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad() -> N -> N:\n  \\{Z: Z{};\n    Z: Z{};\n    S: p => p}\n",
            None,
            "Error:\n- expected : a constructor of N (missing, or already matched)\n- observed : \\{Z: Z{}; S: (p => p)}\nLocation: bad\n5 |   \\{Z: Z{};\n6>|     Z: Z{};\n7 |     S: p => p}\n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type N is Data:\n  Z{}\n  S{p: N}\ndef bad() -> N -> N:\n  \\{Z: Z{};\n    Z: Z{};\n    S: p => p}\n",
            ),
            "Error:\n- expected : a constructor of helper.N (missing, or already matched)\n- observed : \\{helper.Z: helper.Z{}; helper.S: (p => p)}\nLocation: helper.bad\n5 |   \\{Z: Z{};\n6>|     Z: Z{};\n7 |     S: p => p}\n",
        ),
        (
            "type N is Data:\r\n  Z{}\r\n  S{p: N}\r\ntype Other is Data:\r\n  Other.new{}\r\ndef bad(n: N) -> N:\r\n  match n:\r\n    case S{\r\n      Other.new{}\r\n    }:\r\n      Z{}\r\n",
            None,
            "Error:\n- expected : a constructor of N (missing, or already matched)\n- observed : \\{Other.new: Z{}}\nLocation: bad\n 8 |     case S{\r\n 9>|       Other.new{}\r\n10 |     }:\r\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ntype Other is Data:\n  Other.new{}\ndef bad(n: N) -> N:\n  match n:\n    case S{\n      Other.new{}\n    }:\n      Z{}",
            None,
            "Error:\n- expected : a constructor of N (missing, or already matched)\n- observed : \\{Other.new: Z{}}\nLocation: bad\n 8 |     case S{\n 9>|       Other.new{}\n10 |     }:\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(-n: N) -> N:\n  match n:\n    case Z{}: Z{}\n    case S{p}: p\n",
            None,
            "Error:\n- message  : a live scrutinee (a - scrutinee matches only in a dead region)\nLocation: bad\n4 | def bad(-n: N) -> N:\n5>|   match n:\n6 |     case Z{}: Z{}\n",
        ),
        (
            "import helper.bend as H\ndef main() -> Type: Type\n",
            Some(
                "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(-n: N) -> N:\n  match n:\n    case Z{}: Z{}\n    case S{p}: p\n",
            ),
            "Error:\n- message  : a live scrutinee (a - scrutinee matches only in a dead region)\nLocation: helper.bad\n4 | def bad(-n: N) -> N:\n5>|   match n:\n6 |     case Z{}: Z{}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(~f: Type -> Type, -n: N) -> N:\n  match n:\n    case Z{}: Z{}\n    case S{p}: p\ndef main() -> N: bad(~(x => x), Z{})\n",
            None,
            "Error:\n- message  : a live scrutinee (a - scrutinee matches only in a dead region)\nContext:\n- f : @_:Type -> Type\nLocation: bad~0\n4 | def bad(~f: Type -> Type, -n: N) -> N:\n5>|   match n:\n6 |     case Z{}: Z{}\n",
        ),
        (
            "type N is Data:\n  Z{}\n  S{p: N}\ndef bad(x: N) -> @-n:N -> N:\n  \\{Z: x; S: p => p}\n",
            None,
            "Error:\n- message  : a live scrutinee (a - scrutinee matches only in a dead region)\nContext:\n- x : N\nLocation: bad\n4 | def bad(x: N) -> @-n:N -> N:\n5>|   \\{Z: x; S: p => p}\n6 | \n",
        ),
    ] {
        let fixture = Fixture::new(source);
        if let Some(helper) = helper {
            std::fs::write(fixture.0.join("helper.bend"), helper).unwrap();
        }
        assert_diagnostic(&fixture.source(), expected);
    }
}
