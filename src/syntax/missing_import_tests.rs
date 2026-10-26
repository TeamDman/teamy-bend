// SPDX-License-Identifier: MPL-2.0
use super::DiagnosticOrigins;
use super::ImportSite;
use super::LoadDiagnosticContent;
use std::path::Path;
use std::path::PathBuf;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-missing-import-diagnostics")
}

struct Scratch(PathBuf);
impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("teamy-bend-missing-{label}-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, source).unwrap();
        path
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

fn tracked_failures(path: &Path) -> [super::DiagnosticLoadError; 2] {
    [
        crate::syntax::load_with_diagnostics(path).err().unwrap(),
        crate::syntax::load_executable_with_diagnostics(path)
            .err()
            .unwrap(),
    ]
}

#[test]
fn missing_imports_preserve_public_errors_and_exact_importer_source() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "../../tests/fixtures/cpu-missing-import-diagnostics/negative-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let path = root().join(&case);
        let plain = crate::syntax::load(&path).unwrap_err();
        assert_eq!((plain.line, plain.column), (1, 1));
        assert_eq!(
            plain.to_string(),
            crate::syntax::load_executable(&path)
                .unwrap_err()
                .to_string()
        );
        for failure in tracked_failures(&path) {
            assert_eq!(plain.to_string(), failure.error.to_string(), "{case}");
            let report = failure.diagnostic.unwrap();
            let LoadDiagnosticContent::Message(message) = report.content else {
                panic!("expected a typed missing-import report")
            };
            assert_eq!(
                format!(
                    "Error:\n- message  : {message}\nLocation:\n{}\n",
                    report.excerpt
                ),
                std::fs::read_to_string(path.with_extension("stderr")).unwrap(),
                "{case}"
            );
        }
    }
}

#[test]
fn missing_import_spelling_stays_lexical_and_ambiguous_paths_keep_fallback() {
    for (from, relative, expected) in [
        (
            "/authored/link/main.bend",
            "missing.bend",
            Some("/authored/link/missing.bend"),
        ),
        (
            "main.bend",
            "child/missing.bend",
            Some("child/missing.bend"),
        ),
        (
            "C:\\source\\main.bend",
            "/abs/missing.bend",
            Some("/abs/missing.bend"),
        ),
        ("C:\\source\\main.bend", "missing.bend", None),
        ("C:/source/main.bend", "missing.bend", None),
        ("/authored/main.bend", "C:/missing.bend", None),
        ("/authored/main.bend", "C:missing.bend", None),
        ("/authored/./main.bend", "missing.bend", None),
        ("/authored/main.bend", "./missing.bend", None),
        ("/authored/main.bend", "child/../missing.bend", None),
        ("/authored/main.bend", "child//missing.bend", None),
        ("/authored/main.bend", "child\\missing.bend", None),
    ] {
        let source = format!("import Base\nimport {relative} as D\n");
        let mut origins = DiagnosticOrigins::default();
        origins.missing_import(&ImportSite {
            source: &source,
            line: 2,
            from: Path::new(from),
            relative,
        });
        if let Some(expected) = expected {
            let report = origins.load_failure.unwrap();
            assert!(
                matches!(report.content, LoadDiagnosticContent::Message(message) if message == format!("no such file: {expected}"))
            );
            assert!(report.excerpt.starts_with("1 | import Base\n2>| import "));
        } else {
            assert!(origins.load_failure.is_none());
        }
        assert!(origins.sources.is_empty());
    }
}

#[test]
fn missing_root_and_other_filesystem_failures_keep_unformatted_errors() {
    let scratch = Scratch::new("filesystem");
    let absent = scratch.0.join("absent.bend");
    let directory = scratch.0.join("directory.bend");
    std::fs::create_dir(&directory).unwrap();
    let importer = scratch.write("directory-importer.bend", "import directory.bend as D\n");
    let file = scratch.write("ordinary.bend", "");
    let not_directory = scratch.write(
        "file-importer.bend",
        "import ordinary.bend/child.bend as D\n",
    );
    assert!(file.is_file());
    for path in [absent, importer, not_directory] {
        let plain = crate::syntax::load(&path).unwrap_err();
        for failure in tracked_failures(&path) {
            assert_eq!(plain.to_string(), failure.error.to_string());
            assert!(failure.diagnostic.is_none(), "{}", path.display());
        }
    }
}

#[test]
fn missing_import_precedes_depth_guard_but_present_import_still_hits_64() {
    let scratch = Scratch::new("depth");
    for index in 0..64 {
        scratch.write(
            &format!("f{index}.bend"),
            &format!("import f{}.bend as Next\n", index + 1),
        );
    }
    let path = scratch.0.join("f0.bend");
    for failure in tracked_failures(&path) {
        assert!(failure.error.source.ends_with("f64.bend"));
        assert!(!failure.error.message.contains("nesting"));
        if super::simple_import_path(path.to_str().unwrap()) {
            let report = failure.diagnostic.unwrap();
            assert!(
                matches!(report.content, LoadDiagnosticContent::Message(message) if message.ends_with("/f64.bend"))
            );
            assert!(report.excerpt.starts_with("1>| import f64.bend as Next\n"));
        } else {
            assert!(failure.diagnostic.is_none());
        }
    }
    scratch.write("f64.bend", "");
    for failure in tracked_failures(&path) {
        assert_eq!(
            failure.error.message,
            "import nesting exceeds the loader resource limit of 64"
        );
        assert!(failure.diagnostic.is_none());
    }
}

#[test]
fn existing_cycles_and_ambiguous_import_paths_keep_their_errors() {
    for case in [
        "controls/cycle/main.bend",
        "controls/dot.bend",
        "controls/parent.bend",
        "controls/double.bend",
        "controls/hash.bend",
        "controls/bom.bend",
    ] {
        let path = root().join(case);
        let plain = crate::syntax::load(&path).unwrap_err();
        for failure in tracked_failures(&path) {
            assert_eq!(plain.to_string(), failure.error.to_string());
            assert!(failure.diagnostic.is_none(), "{case}");
        }
        if case.contains("cycle") {
            assert_eq!(plain.message, "import cycle");
        }
    }
}

#[cfg(unix)]
#[test]
fn missing_dangling_symlink_keeps_the_link_name_and_immediate_importer() {
    use std::os::unix::fs::symlink;
    let scratch = Scratch::new("symlink");
    symlink("missing-target.bend", scratch.0.join("link.bend")).unwrap();
    let child = scratch.write("child.bend", "# child\nimport link.bend as D\n");
    let parent = scratch.write("main.bend", "import child.bend as C\n");
    for failure in tracked_failures(&parent) {
        let report = failure.diagnostic.unwrap();
        assert!(
            matches!(report.content, LoadDiagnosticContent::Message(message) if message == format!("no such file: {}", child.parent().unwrap().join("link.bend").display()))
        );
        assert_eq!(
            report.excerpt,
            "1 | # child\n2>| import link.bend as D\n3 | "
        );
    }
}
