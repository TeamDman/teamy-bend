// SPDX-License-Identifier: MPL-2.0
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-numeric-{}-{}.bend",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, source).expect("write numeric fixture");
        Self(path)
    }

    fn checked(&self) -> ExecutableBook {
        check_executable(&load_executable(&self.0).expect("parse numeric fixture"))
            .expect("check numeric fixture")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

const HELPERS: &str = r"import Base
def make(bits: U32) -> F32:
  match bits:
    case U32{word}: F32{word}
def bu(x: Bool) -> U32:
  match x:
    case False{}: 0
    case True{}: 1
";

fn output(source: &str) -> String {
    let fixture = Fixture::new(source);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .expect("run numeric fixture"),
        0
    );
    assert!(stderr.is_empty());
    String::from_utf8(stdout).expect("numeric ASCII output")
}

#[test]
fn core_float_operations_match_independent_binary32_witnesses() {
    // Decimal bit patterns also compared with the upstream JS executable lane.
    // Raw signaling NaN preservation is intentionally a separate native test.
    let cases = [
        ("F32.bits(U32.to_f32(4294967295))", "1333788672"),
        ("F32.bits(F32.add(16777216.0, 1.0))", "1266679808"),
        ("F32.bits(F32.sub(0.0, 1.0))", "3212836864"),
        ("F32.bits(F32.mul(1.75, 2.0))", "1080033280"),
        ("F32.bits(F32.div(1.0, 3.0))", "1051372203"),
        ("F32.bits(F32.mod(F32.neg(5.5), 2.0))", "3217031168"),
        ("F32.bits(F32.neg(0.0))", "2147483648"),
        ("F32.bits(F32.abs(F32.neg(0.0)))", "0"),
        ("F32.bits(F32.abs(F32.neg(1.5)))", "1069547520"),
        ("F32.to_u32(3.75)", "3"),
        ("F32.to_u32(0.75)", "0"),
        ("F32.to_u32(F32.neg(3.75))", "0"),
        ("F32.to_u32(4294967296.0)", "0"),
        ("F32.to_u32(4294967040.0)", "4294967040"),
        ("F32.to_u32(F32.div(0.0, 0.0))", "0"),
        ("F32.to_u32(F32.div(1.0, 0.0))", "0"),
        ("F32.bits(F32.div(1.0, 0.0))", "2139095040"),
        ("F32.bits(F32.div(F32.neg(1.0), 0.0))", "4286578688"),
        ("bu(F32.is_eq(F32.neg(0.0), 0.0))", "1"),
        ("bu(F32.is_eq(F32.div(0.0, 0.0), 0.0))", "0"),
        ("bu(F32.is_ne(F32.div(0.0, 0.0), 0.0))", "1"),
        ("bu(F32.is_lt(F32.div(0.0, 0.0), 0.0))", "0"),
        ("bu(F32.is_le(F32.div(0.0, 0.0), 0.0))", "0"),
        ("bu(F32.is_gt(F32.div(0.0, 0.0), 0.0))", "0"),
        ("bu(F32.is_ge(F32.div(0.0, 0.0), 0.0))", "0"),
        ("bu(F32.is_lt(F32.neg(1.0), 0.0))", "1"),
        ("bu(F32.is_le(1.0, 1.0))", "1"),
        ("bu(F32.is_gt(F32.div(1.0, 0.0), 4294967296.0))", "1"),
        ("bu(F32.is_ge(1.0, 1.0))", "1"),
        ("F32.bits(make(2143289345))", "2143289345"),
        ("F32.bits(F32.add(make(1), make(1)))", "2"),
        ("F32.bits(F32.mul(make(1), 0.5))", "0"),
        ("F32.bits(F32.mod(F32.neg(4.0), 2.0))", "2147483648"),
    ];
    for (expression, value) in cases {
        let source =
            format!("{HELPERS}\ndef main() -> IO(Unit): IO.print(U32.show({expression}))\n");
        assert_eq!(output(&source), format!("{value}\n"), "{expression}");
    }
}

#[test]
fn native_raw_float_bits_and_sign_operations_preserve_nan_payloads() {
    for (expression, expected) in [
        ("make(2139095041)", "2139095041"),
        ("F32.neg(make(2139095041))", "4286578689"),
        ("F32.abs(make(4286578689))", "2139095041"),
    ] {
        assert_eq!(
            output(&format!(
                "{HELPERS}\ndef main() -> IO(Unit): IO.print(U32.show(F32.bits({expression})))\n"
            )),
            format!("{expected}\n")
        );
    }
}

#[test]
fn numeric_functions_can_be_partial_higher_order_and_remain_opaque_to_proofs() {
    let fixture = Fixture::new(
        r"import Base
def apply(f: F32 -> F32, x: F32) -> F32: f(x)
def main() -> IO(Unit): IO.print(U32.show(F32.to_u32(apply(F32.add(2.0), 3.0))))

law reflexive_opaque:
  for a: F32
  {F32.add(a, 0.0) == F32.add(a, 0.0) : F32}
def reflexive_opaque(a): {==}

law copied_builtin:
  for a: F32
  F32
def copied_builtin(a): F32.mul(a, 2.0)

law erased_symbol:
  {F32.add(1.0, 2.0) == F32.add(1.0, 2.0) : F32}
def erased_symbol(): {==}
",
    );
    let checked = fixture.checked();
    assert_eq!(checked.numeric_names().count(), 37);
    assert_eq!(checked.foreign_names().count(), 27);
    let mut stdout = Vec::new();
    checked
        .run_main(&mut stdout, &mut Vec::new(), &|| false)
        .unwrap();
    assert_eq!(stdout, b"5\n");
    let strict = load(&fixture.0).expect("strict syntax loads ordinary numeric references");
    check_book(&strict).expect_err("strict Base does not grant numeric runtime assumptions");

    for (body, diagnostic) in [
        (
            "def false_reduction() -> {F32.add(1.0, 2.0) == 3.0 : F32}: {==}",
            "reflexivity",
        ),
        ("def bad_operand() -> F32: F32.add(1, 2.0)", "constructor"),
        ("def F32.add(a, b): a", "numeric executable declaration"),
        ("law extra: F32", "unfilled laws"),
        (
            "def erased_operand(-x: F32) -> F32: F32.add(x, 1.0)",
            "permits None use, observed Lone",
        ),
    ] {
        let fixture = Fixture::new(&format!("import Base\n{body}\n"));
        let source = load_executable(&fixture.0).expect("negative numeric fixture parses");
        let error = check_executable(&source)
            .expect_err("numeric checking fails closed")
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
}

#[test]
fn untrusted_names_cannot_acquire_numeric_implementations() {
    let fixture = Fixture::new(
        r"type F32 is Data: F32{}
law F32.add:
  for a: F32
  for b: F32
  F32
def main() -> F32: F32.add(F32{}, F32{})
",
    );
    let source = load_executable(&fixture.0).expect("ordinary unfilled law parses");
    let error = check_executable(&source)
        .expect_err("numeric spelling grants no origin")
        .to_string();
    assert!(error.contains("unfilled law F32.add"), "{error}");
    assert_eq!(
        output(
            "type F32 is Data: F32{}\ndef F32.add(a: F32, b: F32) -> F32: a\ndef main() -> F32: F32.add(F32{}, F32{})\n"
        ),
        "F32{}\n"
    );
}

#[test]
fn pure_main_keeps_numeric_applications_opaque_in_the_proof_normalizer() {
    let printed = output("import Base\ndef main() -> F32: F32.add(1.0, 2.0)\n");
    assert!(printed.contains("F32.add"), "{printed}");
    assert!(printed.ends_with('\n'));
    assert_eq!(
        output(
            "import Base\ndef main() -> IO(Unit): IO.print(U32.show(F32.to_u32(F32.add(1.0, 2.0))))\n"
        ),
        "3\n"
    );
}

#[test]
fn nested_numeric_calls_use_bounded_continuations_and_observe_cancellation() {
    let helper = "import Base\ndef count(n: Nat, x: F32) -> F32:\n  match n:\n    case Zero{}: x\n    case Succ{p}: F32.add(count(p, x), 1.0)\n";
    assert_eq!(
        output(&format!(
            "{helper}def main() -> IO(Unit): IO.print(U32.show(F32.to_u32(count(40n, 2.0))))\n"
        )),
        "42\n"
    );
    let fixture = Fixture::new(&format!(
        "{helper}def main() -> IO(Unit): IO.print(U32.show(F32.to_u32(count(U32.to_nat(5000), 0.0))))\n"
    ));
    let checked = fixture.checked();
    let error = checked
        .run_main(&mut Vec::new(), &mut Vec::new(), &|| false)
        .expect_err("deep nested numeric calls have a bounded frame stack")
        .to_string();
    assert!(error.contains("continuation depth exhausted"), "{error}");
    let ticks = std::cell::Cell::new(0);
    let error = checked
        .run_main(&mut Vec::new(), &mut Vec::new(), &|| {
            ticks.set(ticks.get() + 1);
            ticks.get() > 100
        })
        .expect_err("numeric argument evaluation is cancellable")
        .to_string();
    assert!(error.contains("cancelled"), "{error}");
}

#[test]
fn ordinary_u32_add_wrapping_matches_checked_reduction_at_boundaries() {
    for (left, right, expected) in [
        (0_u32, 0_u32, 0_u32),
        (4_294_967_295_u32, 1, 0),
        (4_294_967_295, 4_294_967_295, 4_294_967_294),
        (2_147_483_647, 1, 2_147_483_648),
        (2_147_483_648, 2_147_483_648, 0),
        (65_535, 1, 65_536),
        (2_863_311_530, 1_431_655_765, 4_294_967_295),
        (20, 22, 42),
    ] {
        let proof = Fixture::new(&format!(
            "import Base\n law witness: {{U32.add({left}, {right}) == {expected} : U32}}\ndef witness(): {{==}}\n"
        ));
        check_book(&load(&proof.0).expect("strict wrapping fixture parses"))
            .expect("ordinary checked U32.add reduction verifies the expected bits");
        assert_eq!(
            output(&format!(
                "import Base\ndef main() -> IO(Unit): IO.print(U32.show(U32.add({left}, {right})))\n"
            )),
            format!("{expected}\n")
        );
    }
}

#[test]
fn optimized_add_retains_partial_application_and_is_not_a_numeric_assumption() {
    let fixture = Fixture::new(
        r"import Base
def apply(f: U32 -> U32, x: U32) -> U32: f(x)
def main() -> IO(Unit): IO.print(U32.show(apply(U32.add(4294967295), 2)))
",
    );
    let checked = fixture.checked();
    assert!(!checked.numeric_names().any(|name| name == "U32.add"));
    assert!(!checked.foreign_names().any(|name| name == "U32.add"));
    let mut stdout = Vec::new();
    assert_eq!(
        checked
            .run_main(&mut stdout, &mut Vec::new(), &|| false)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"1\n");
}

#[test]
fn upstream_float_comparison_digest_and_decimal_formatting_fit_existing_budgets() {
    // This exact upstream workload used to exhaust the unchanged thunk arena
    // when its repeated ordinary U32 additions were followed by U32.show.
    assert_eq!(
        output(include_str!("fixtures/float_compare.bend")),
        "1999985\n"
    );
}

#[test]
fn remaining_float_math_matches_independent_c_binary32_witnesses() {
    // Captured from the fixed upstream C expressions: double math, then a float
    // cast. Decimal bit witnesses make rounding and signed-zero changes visible.
    for (expression, expected) in [
        ("F32.sqrt(2.0)", 1_068_827_891_u32),
        ("F32.exp(1.0)", 1_076_754_516),
        ("F32.log(2.0)", 1_060_205_080),
        ("F32.log2(3.0)", 1_070_260_237),
        ("F32.log10(2.0)", 1_050_288_283),
        ("F32.sin(1.0)", 1_062_693_540),
        ("F32.cos(1.0)", 1_057_640_768),
        ("F32.tan(1.0)", 1_070_029_091),
        ("F32.asin(0.5)", 1_057_360_530),
        ("F32.acos(0.5)", 1_065_749_138),
        ("F32.atan(1.0)", 1_061_752_795),
        ("F32.sinh(1.0)", 1_066_822_910),
        ("F32.cosh(1.0)", 1_069_908_907),
        ("F32.tanh(1.0)", 1_061_353_430),
        ("F32.floor(F32.neg(1.25))", 3_221_225_472),
        ("F32.ceil(F32.neg(0.25))", 2_147_483_648),
        ("F32.trunc(F32.neg(0.75))", 2_147_483_648),
        ("F32.pow(2.0, 0.5)", 1_068_827_891),
        ("F32.atan2(F32.neg(0.0), F32.neg(1.0))", 3_226_013_659),
        ("F32.pow(F32.neg(2.0), 3.0)", 3_238_002_688),
        ("F32.sqrt(F32.neg(0.0))", 2_147_483_648),
        ("F32.exp(100.0)", 2_139_095_040),
        ("F32.exp(F32.neg(104.0))", 0),
        ("F32.log(0.0)", 4_286_578_688),
        ("F32.sinh(F32.neg(0.0))", 2_147_483_648),
        ("F32.tanh(make(2139095040))", 1_065_353_216),
        ("F32.atan2(0.0, F32.neg(0.0))", 1_078_530_011),
        ("F32.floor(F32.neg(0.0))", 2_147_483_648),
    ] {
        assert_eq!(
            output(&format!(
                "{HELPERS}def main() -> IO(Unit): IO.print(U32.show(F32.bits({expression})))\n"
            )),
            format!("{expected}\n"),
            "{expression}"
        );
    }
    // Domain failures are NaNs; a portable test must not demand a particular
    // libm NaN payload or sign for arithmetic results.
    for expression in [
        "F32.pow(F32.neg(2.0), 0.5)",
        "F32.sqrt(F32.neg(1.0))",
        "F32.log(F32.neg(1.0))",
        "F32.sin(make(2139095040))",
        "F32.cos(make(2139095040))",
        "F32.asin(2.0)",
        "F32.acos(F32.neg(2.0))",
    ] {
        assert_eq!(
            output(&format!(
                "{HELPERS}def main() -> IO(Unit): IO.print(F32.show({expression}))\n"
            )),
            "nan\n"
        );
    }
}

const READ_HELPER: &str = r#"def show_read(x: Maybe<&2, F32>) -> String:
  match x:
    case None{}: "none"
    case Some{value}: U32.show(F32.bits(value))
def from_read(x: Maybe<&2, F32>) -> F32:
  match x:
    case None{}: 0.0
    case Some{value}: value
"#;

#[test]
fn native_float_text_executes_through_strings_and_maybe_constructors() {
    for (source_text, expected) in [
        (r#""""#, "none"),
        (r#""\0""#, "0"),
        (r#""\0tail""#, "0"),
        (r#""1.5\0tail""#, "1069547520"),
        (r#""\t\n 1.5""#, "1069547520"),
        (r#""1.5 ""#, "none"),
        (r#""1.5\n""#, "none"),
        (r#""0x1.8p+1""#, "1077936128"),
        (r#""0x1.000002p-150""#, "1"),
        (r#""0x1p""#, "none"),
        (r#""-0""#, "2147483648"),
        (r#""inf""#, "2139095040"),
        (r#""nan(a-b)""#, "none"),
        (r#""\u{a0}1""#, "none"),
    ] {
        assert_eq!(
            output(&format!(
                "{HELPERS}{READ_HELPER}def main() -> IO(Unit): IO.print(show_read(F32.read({source_text})))\n"
            )),
            format!("{expected}\n"),
            "{source_text}"
        );
    }
    for (expression, expected) in [
        ("F32.neg(0.0)", "-0"),
        ("make(1)", "1e-45"),
        ("make(1621981420)", "100000000000000000000"),
        ("make(1649989415)", "1e+21"),
        ("make(2139095039)", "3.4028235e+38"),
        ("F32.hypot(3.0, 4.0)", "5"),
        ("F32.round(F32.neg(1.5))", "-1"),
        ("F32.clamp(3.0, 0.0, 2.0)", "2"),
        ("F32.lerp(2.0, 4.0, 0.25)", "2.5"),
    ] {
        assert_eq!(
            output(&format!(
                "{HELPERS}def main() -> IO(Unit): IO.print(F32.show({expression}))\n"
            )),
            format!("{expected}\n"),
            "{expression}"
        );
    }
}

#[test]
fn extended_intrinsics_keep_higher_order_calls_and_proof_opacity() {
    assert_eq!(
        output(&format!(
            r#"{HELPERS}{READ_HELPER}
def apply(f: F32 -> F32, x: F32) -> F32: f(x)
def render(f: (@+x: F32 -> String), +x: F32) -> String: f(x)
def parse_with(f: String -> Maybe<&2, F32>, s: String) -> F32: from_read(f(s))
def main() -> IO(Unit):
  IO.print(render(F32.show, apply(F32.pow(2.0), parse_with(F32.read, "3"))))
"#
        )),
        "8\n"
    );
    for body in [
        "def false_sqrt() -> {F32.sqrt(4.0) == 2.0 : F32}: {==}",
        r#"def false_show() -> {F32.show(1.0) == "1" : String}: {==}"#,
        r#"def false_read() -> {F32.read("1") == Some{1.0} : Maybe<&2, F32>}: {==}"#,
    ] {
        let fixture = Fixture::new(&format!("import Base\n{body}\n"));
        let error = check_executable(&load_executable(&fixture.0).unwrap())
            .expect_err("runtime contracts do not establish equality proofs")
            .to_string();
        assert!(error.contains("reflexivity"), "{error}");
    }
    for (body, diagnostic) in [
        ("def F32.sqrt(a): a", "numeric executable declaration"),
        (
            "def erased_text(-x: String) -> Maybe<&2, F32>: F32.read(x)",
            "permits None use",
        ),
        (
            "def erased_float(-x: F32) -> String: F32.show(x)",
            "permits None use",
        ),
    ] {
        let fixture = Fixture::new(&format!("import Base\n{body}\n"));
        let error = check_executable(&load_executable(&fixture.0).unwrap())
            .expect_err("extended contracts obey ordinary fill and resource rules")
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
}

#[test]
fn nested_float_text_uses_bounded_continuations_and_cancellable_decoding() {
    let helper = format!(
        "{HELPERS}{READ_HELPER}def nest(n: Nat, +x: F32) -> String:\n  match n:\n    case Zero{{}}: F32.show(x)\n    case Succ{{p}}: F32.show(from_read(F32.read(nest(p, x))))\n"
    );
    assert_eq!(
        output(&format!(
            "{helper}def main() -> IO(Unit): IO.print(nest(20n, 1.5))\n"
        )),
        "1.5\n"
    );
    let fixture = Fixture::new(&format!(
        "{helper}def main() -> IO(Unit): IO.print(nest(U32.to_nat(5000), 1.5))\n"
    ));
    let error = fixture
        .checked()
        .run_main(&mut Vec::new(), &mut Vec::new(), &|| false)
        .expect_err("nested text calls must stay on the bounded continuation stack")
        .to_string();
    assert!(error.contains("continuation depth exhausted"), "{error}");
    let fixture = Fixture::new(&format!(
        r#"{HELPERS}{READ_HELPER}def main() -> IO(Unit): IO.print(show_read(F32.read(String.repeat("0", U32.to_nat(1000)))))"#
    ));
    let ticks = std::cell::Cell::new(0);
    let error = fixture
        .checked()
        .run_main(&mut Vec::new(), &mut Vec::new(), &|| {
            ticks.set(ticks.get() + 1);
            ticks.get() > 1000
        })
        .expect_err("numeric String traversal remains cancellable")
        .to_string();
    assert!(error.contains("cancelled"), "{error}");
}
