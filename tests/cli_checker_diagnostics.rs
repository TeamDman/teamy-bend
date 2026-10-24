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
