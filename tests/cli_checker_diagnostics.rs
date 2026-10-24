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
