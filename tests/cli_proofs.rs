// SPDX-License-Identifier: MPL-2.0
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-cli-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir(&path).expect("create fixture directory");
        std::fs::write(path.join("main.bend"), source).expect("write source fixture");
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

fn invoke(command: &str, source: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
        .args(["--output-format", "json", command])
        .arg(source)
        .args(args)
        .output()
        .expect("run native CLI")
}

#[test]
fn checked_example_evaluates_and_batch_preserves_row_order() {
    #[derive(facet::Facet)]
    struct Report {
        results: Vec<u64>,
    }

    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/laws.bend");
    let checked = invoke("check", &source, &[]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert!(String::from_utf8_lossy(&checked.stdout).contains("\"checked\": true"));
    let fixture = Fixture::new("");
    let rows = fixture.0.join("rows.json");
    std::fs::write(&rows, "[[3],[0],[8],[1]]").expect("write batch");
    let output = invoke(
        "batch",
        &source,
        &[
            "--entry",
            "identity",
            "--args-json",
            rows.to_str().expect("path"),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Report = facet_json::from_slice(&output.stdout).expect("valid JSON report");
    assert_eq!(report.results, vec![3, 0, 8, 1]);
}

#[test]
fn native_checker_accepts_source_induction_and_equality_rewrite() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/induction.bend");
    let result = invoke("check", &source, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn false_reflexivity_is_rejected_by_check_and_eval() {
    let fixture = Fixture::new(&format!(
        "{NAT}\nlaw false_law:\n  {{0n == 1n : Nat}}\ndef false_law():\n  {{==}}\ndef main() -> Nat:\n  0n\n"
    ));
    for command in ["check", "eval"] {
        let result = invoke(command, &fixture.source(), &[]);
        assert!(!result.status.success(), "{command} accepted a false proof");
        assert!(result.stdout.is_empty(), "failure emitted a success report");
    }
}

#[test]
fn incomplete_and_unsafe_proofs_fail_closed() {
    for declaration in [
        "law unfinished:\n  Nat\n",
        "def unfinished() -> Nat:\n  ?TODO\n",
        "@unsafe\ndef diverge(n: Nat) -> Nat:\n  diverge(n)\n",
        "def diverge(n: Nat) -> Nat:\n  diverge(n)\n",
    ] {
        let fixture = Fixture::new(&format!("{NAT}\n{declaration}"));
        let result = invoke("check", &fixture.source(), &[]);
        assert!(!result.status.success(), "accepted {declaration}");
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn native_cli_reaches_nesting_limit_without_stack_overflow() {
    // A direct Nat128 body is below the parser's expansion cap. On Windows,
    // checking it used to overflow the debug CLI's main stack before the
    // kernel could return its normal nesting-limit error.
    let fixture = Fixture::new(&format!("{NAT}\ndef main() -> Nat:\n  128n\n"));
    for command in ["check", "eval"] {
        let result = invoke(command, &fixture.source(), &[]);
        assert_eq!(
            result.status.code(),
            Some(1),
            "{command} crashed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stderr).contains("kernel nesting limit exhausted"));
        assert!(result.stdout.is_empty(), "failure emitted a success report");
    }

    let supported = Fixture::new(&format!("{NAT}\ndef main() -> Nat:\n  120n\n"));
    let result = invoke("check", &supported.source(), &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn malformed_batch_data_and_wrong_arity_are_errors() {
    let fixture = Fixture::new(&format!("{NAT}\ndef identity(n: Nat) -> Nat:\n  n\n"));
    let rows = fixture.0.join("rows.json");
    for data in ["[[1.5]]", "[[-1]]", "[[4097]]", "[[0,1]]", "[[]]", "{}"] {
        std::fs::write(&rows, data).expect("write batch");
        let result = invoke(
            "batch",
            &fixture.source(),
            &[
                "--entry",
                "identity",
                "--args-json",
                rows.to_str().expect("path"),
            ],
        );
        assert!(!result.status.success(), "accepted invalid input {data}");
        assert!(result.stdout.is_empty());
    }
}
