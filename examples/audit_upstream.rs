// SPDX-License-Identifier: MPL-2.0
//! Audit check compatibility with a separate, unchanged upstream checkout.
//! A matching rejection is not proof of matching semantics; diagnostics and
//! execution outputs are deliberately reported as not compared by this tool.
use facet::Facet;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Default, Facet)]
struct Category {
    fixtures: usize,
    expected_success_checked: usize,
    expected_success_rejected: usize,
    expected_failure_rejected: usize,
    expected_failure_checked: usize,
    abnormal_exit: usize,
}

#[derive(Debug, Facet)]
struct Finding {
    fixture: String,
    outcome: String,
    detail: String,
}

#[derive(Debug, Facet)]
struct Audit {
    comparison: String,
    categories: BTreeMap<String, Category>,
    findings: Vec<Finding>,
}

fn collect(root: &Path, paths: &mut Vec<PathBuf>) -> eyre::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        // Do not traverse symlinks outside the supplied reference.
        if entry.file_type()?.is_dir() {
            collect(&entry.path(), paths)?;
        } else if entry.file_type()?.is_file()
            && entry.path().extension().is_some_and(|s| s == "bend")
        {
            paths.push(entry.path());
        }
    }
    Ok(())
}

fn main() -> eyre::Result<()> {
    let reference = std::env::args_os().nth(1).ok_or_else(|| {
        eyre::eyre!("usage: cargo run --example audit_upstream -- <bend-reference>")
    })?;
    let tests = PathBuf::from(reference).join("tests");
    let mut files = Vec::new();
    collect(&tests, &mut files)?;
    files.sort();
    let default_binary = std::env::current_exe()?
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| eyre::eyre!("cannot resolve build directory"))?
        .join(format!("teamy-bend{}", std::env::consts::EXE_SUFFIX));
    let binary = std::env::args_os()
        .nth(2)
        .map_or(default_binary, PathBuf::from);
    if !binary.is_file() {
        eyre::bail!("run cargo build before the fixture audit");
    }
    let mut audit = Audit {
        comparison: "check acceptance only; expected runtime failures may check successfully; exact diagnostics and runtime output are not compared".to_owned(),
        categories: BTreeMap::new(), findings: Vec::new(),
    };
    for path in files {
        let source = std::fs::read_to_string(&path)?;
        if !source.lines().any(|line| line.starts_with("#|")) {
            continue;
        }
        let relative = path.strip_prefix(&tests)?;
        eprintln!("checking {}", relative.display());
        let category = relative
            .components()
            .next()
            .ok_or_else(|| eyre::eyre!("missing category"))?
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let expected_failure = source.lines().any(|line| {
            line.strip_prefix("#|exit ")
                .is_some_and(|code| code.trim() != "0")
        });
        // A parser/kernel crash in one untrusted fixture must not lose the
        // remaining compatibility evidence. Each fixture uses the real CLI.
        let output = Command::new(&binary)
            .args(["--output-format", "json", "check"])
            .arg(&path)
            .output()?;
        let abnormal = !output.status.success() && output.status.code() != Some(1);
        let outcome = if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).into_owned())
        };
        let counts = audit.categories.entry(category).or_default();
        counts.fixtures += 1;
        if abnormal {
            counts.abnormal_exit += 1;
        }
        if !abnormal {
            match (&outcome, expected_failure) {
                (Ok(()), false) => counts.expected_success_checked += 1,
                (Ok(()), true) => counts.expected_failure_checked += 1,
                (Err(_), false) => counts.expected_success_rejected += 1,
                (Err(_), true) => counts.expected_failure_rejected += 1,
            }
        }
        if outcome.is_ok() == expected_failure || abnormal {
            audit.findings.push(Finding {
                fixture: relative.to_string_lossy().replace('\\', "/"),
                outcome: if abnormal {
                    "abnormal-exit"
                } else if expected_failure {
                    "expected-failure-checked"
                } else {
                    "expected-success-rejected"
                }
                .to_owned(),
                detail: outcome.err().unwrap_or_else(|| {
                    "checked; inspect whether expected failure is runtime-only".to_owned()
                }),
            });
        }
    }
    writeln!(
        std::io::stdout().lock(),
        "{}",
        facet_json::to_string_pretty(&audit)?
    )?;
    Ok(())
}
