// SPDX-License-Identifier: MPL-2.0
use super::output::CliOutput;
use crate::kernel::KernelError;
use crate::syntax::diagnostics::DiagnosticOrigins;
use crate::syntax::diagnostics::LoadDiagnosticContent;
use eyre::Context;
use eyre::eyre;
use std::fmt::Write as _;
use std::io::Write;
use teamy_cancellation::CancellationToken;

pub(super) fn reject(
    error: &KernelError,
    origins: &DiagnosticOrigins,
    cancellation: &CancellationToken,
) -> eyre::Result<CliOutput> {
    emit(error, origins, cancellation, &mut std::io::stderr().lock())
}

fn emit(
    error: &KernelError,
    origins: &DiagnosticOrigins,
    cancellation: &CancellationToken,
    stderr: &mut impl Write,
) -> eyre::Result<CliOutput> {
    cancellation.bail_if_cancelled()?;
    if let Some(count) = error.unfilled_law_count() {
        tracing::debug!(internal_error = %error, "Bend check rejected");
        let suffix = if count == 1 { "" } else { "s" };
        return write_diagnostic(
            &format!(
                "Error: {count} TODO{suffix} found.\nThe code is incomplete, and not a valid proof yet.\n"
            ),
            stderr,
        );
    }
    let Some(report) = error.checker_diagnostic() else {
        return Err(eyre!("{error}"));
    };
    let excerpt = diagnostic_excerpt(report, origins);
    let Some(excerpt) = excerpt else {
        // Programmatically constructed or untracked terms keep the ordinary
        // host diagnostic. Never guess a source line from an error string.
        return Err(eyre!("{error}"));
    };
    let mut diagnostic = report.observed.as_ref().map_or_else(
        || format!("Error:\n- message  : {}", report.expected),
        |observed| {
            format!(
                "Error:\n- expected : {}\n- observed : {observed}",
                report.expected
            )
        },
    );
    if !report.context.is_empty() {
        diagnostic.push_str("\nContext:");
        let width = report
            .context
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        for (name, ty) in &report.context {
            write!(diagnostic, "\n- {name:width$} : {ty}").expect("String formatting");
        }
    }
    write!(diagnostic, "\nLocation: {}\n{excerpt}\n", report.definition)
        .expect("String formatting");
    tracing::debug!(internal_error = %error, "Bend check rejected");
    write_diagnostic(&diagnostic, stderr)
}

fn diagnostic_excerpt(
    report: &crate::kernel::CheckerDiagnostic,
    origins: &DiagnosticOrigins,
) -> Option<String> {
    report
        .preferred_source
        .and_then(|source| origins.excerpt(source))
        .or_else(|| {
            report.match_site.map_or_else(
                || origins.excerpt(report.term_identity),
                |site| origins.match_excerpt(site),
            )
        })
}

pub(super) fn reject_load(
    failure: &crate::syntax::diagnostics::DiagnosticLoadError,
    cancellation: &CancellationToken,
) -> eyre::Result<CliOutput> {
    emit_load(failure, cancellation, &mut std::io::stderr().lock())
}

fn emit_load(
    failure: &crate::syntax::diagnostics::DiagnosticLoadError,
    cancellation: &CancellationToken,
    stderr: &mut impl Write,
) -> eyre::Result<CliOutput> {
    cancellation.bail_if_cancelled()?;
    let Some(report) = &failure.diagnostic else {
        return Err(eyre!("{}", failure.error));
    };
    tracing::debug!(internal_error = %failure.error, "Bend source load rejected");
    let content = match &report.content {
        LoadDiagnosticContent::Message(message) => format!("- message  : {message}"),
        LoadDiagnosticContent::ExpectedObserved { expected, observed } => {
            format!("- expected : {expected}\n- observed : {observed}")
        }
    };
    write_diagnostic(
        &format!("Error:\n{content}\nLocation:\n{}\n", report.excerpt),
        stderr,
    )
}

fn write_diagnostic(diagnostic: &str, stderr: &mut impl Write) -> eyre::Result<CliOutput> {
    stderr
        .write_all(diagnostic.as_bytes())
        .wrap_err("cannot write Bend checker diagnostic")?;
    stderr
        .flush()
        .wrap_err("cannot flush Bend checker diagnostic")?;
    Ok(CliOutput::exit_status(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::check_book_with_origins;
    use crate::syntax::load_with_diagnostics;
    use std::io;

    fn rejected(name: &str) -> (KernelError, DiagnosticOrigins) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let (book, origins) = load_with_diagnostics(path).unwrap();
        (
            check_book_with_origins(&book, Some(origins.tracker.clone())).unwrap_err(),
            origins,
        )
    }

    #[test]
    fn final_open_law_summary_preserves_cancellation_writes_and_unclassified_errors() {
        struct Writer(bool);
        impl Write for Writer {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.0 {
                    Err(io::Error::other("write failed"))
                } else {
                    Ok(bytes.len())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("flush failed"))
            }
        }
        for file in [
            "check/axiom_warn.bend",
            "controls/multiple.bend",
            "controls/imported/main.bend",
        ] {
            let (error, origins) = rejected(&format!("tests/fixtures/cpu-final-todo/{file}"));
            assert!(error.checker_diagnostic().is_none());
            let token = CancellationToken::new();
            token.request_cancel("stop summary");
            let mut bytes = Vec::new();
            assert!(
                emit(&error, &origins, &token, &mut bytes)
                    .unwrap_err()
                    .to_string()
                    .contains("stop summary")
            );
            assert!(bytes.is_empty());
            for fail_write in [true, false] {
                let failure = emit(
                    &error,
                    &origins,
                    &CancellationToken::new(),
                    &mut Writer(fail_write),
                )
                .unwrap_err();
                assert!(failure.to_string().contains(if fail_write {
                    "cannot write"
                } else {
                    "cannot flush"
                }));
            }
            // Global inventories have no source span, even for programmatic books.
            let output = emit(
                &error,
                &DiagnosticOrigins::default(),
                &CancellationToken::new(),
                &mut bytes,
            )
            .unwrap();
            assert_eq!(output.exit_code(), 1);
            let count = error.unfilled_law_count().unwrap();
            let expected = format!(
                "Error: {count} TODO{} found.\nThe code is incomplete, and not a valid proof yet.\n",
                if count == 1 { "" } else { "s" }
            );
            assert_eq!(bytes, expected.as_bytes());
            bytes.clear();
            let ordinary = KernelError::new(&error.message);
            assert_eq!(
                emit(&ordinary, &origins, &CancellationToken::new(), &mut bytes)
                    .unwrap_err()
                    .to_string(),
                error.to_string()
            );
            assert!(bytes.is_empty());
        }
    }

    #[test]
    fn unsourced_let_keeps_the_original_diagnostic_excerpt() {
        use crate::kernel::Declaration;
        use crate::kernel::Term;
        use crate::kernel::term;
        use std::rc::Rc;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-let-kind-locations/direct-let.bend");
        let (mut book, origins) = load_with_diagnostics(path).unwrap();
        let before = format!("{book:?}");
        let definition = book
            .declarations
            .iter_mut()
            .find_map(|item| match item {
                Declaration::Def(d) if d.name == "bad" => Some(d),
                _ => None,
            })
            .unwrap();
        let Term::Lam { name, id, body } = definition.body.as_ref().unwrap().as_ref() else {
            panic!("outer function")
        };
        let Term::Let {
            bindings,
            body: continuation,
        } = body.as_ref()
        else {
            panic!("let")
        };
        let untracked = term(Term::Let {
            bindings: bindings.clone(),
            body: Rc::clone(continuation),
        });
        let identity = Rc::as_ptr(&untracked) as usize;
        definition.body = Some(term(Term::Lam {
            name: name.clone(),
            id: *id,
            body: untracked,
        }));
        assert_eq!(format!("{book:?}"), before);
        assert!(origins.excerpt(identity).is_none());
        let error = check_book_with_origins(&book, Some(origins.tracker.clone())).unwrap_err();
        let report = error.checker_diagnostic().unwrap();
        assert!(report.preferred_source.is_some());
        assert!(origins.excerpt(report.preferred_source.unwrap()).is_none());
        let mut bytes = Vec::new();
        let result = emit(&error, &origins, &CancellationToken::new(), &mut bytes).unwrap();
        assert_eq!(result.exit_code(), 1);
        let expected: String = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-let-kind-locations/direct-let-unsourced.stderr.json"
        ))
        .unwrap();
        assert_eq!(bytes, expected.as_bytes());
    }

    #[test]
    fn cancellation_and_untracked_errors_never_emit_a_partial_language_report() {
        for name in [
            "tests/fixtures/cpu-contextual-proof-diagnostics/check/default_arm_family.bend",
            "tests/fixtures/cpu-contextual-proof-diagnostics/check/neq_reflexivity.bend",
            "tests/fixtures/cpu-let-kind-locations/reusable-field.bend",
            "tests/fixtures/cpu-let-kind-locations/inference-first.bend",
            "tests/fixtures/cpu-comptime-type/comptime/wrong_type.bend",
            "tests/fixtures/cpu-comptime-type/negative/imported.bend",
            "tests/fixtures/cpu-comptime-type/negative/distinct_uses.bend",
            "tests/fixtures/cpu-match-guards/match_not_datatype.bend",
            "tests/fixtures/cpu-match-guards/error_window_match_repeated.bend",
            "tests/fixtures/cpu-match-guards/erased_match_ctor.bend",
            "tests/fixtures/cpu-match-failures/empty_match.bend",
            "tests/fixtures/cpu-match-failures/error_window_match_missing.bend",
            "tests/fixtures/cpu-matched-values/error_window_matched_scrutinee.bend",
            "tests/fixtures/cpu-checking/false_brace.bend",
            "tests/fixtures/cpu-constructors/nat_literal_import.bend",
            "tests/fixtures/cpu-plus-binders/plus_binder_term.bend",
            "tests/fixtures/cpu-rewrite-checking/rewrite_ctor_brace.bend",
            "tests/fixtures/cpu-rewrite-proof/rewrite_non_equation.bend",
            "tests/fixtures/cpu-rewrite-proof/error_window_rewrite_proof.bend",
            "tests/fixtures/cpu-rewrite-proof/reject_rewrite.bend",
        ] {
            let (error, origins) = rejected(name);
            let cancellation = CancellationToken::new();
            cancellation.request_cancel("stop diagnostic");
            let mut bytes = Vec::new();
            assert!(
                emit(&error, &origins, &cancellation, &mut bytes)
                    .unwrap_err()
                    .to_string()
                    .contains("stop diagnostic")
            );
            assert!(bytes.is_empty());
            let fallback = emit(
                &error,
                &DiagnosticOrigins::default(),
                &CancellationToken::new(),
                &mut bytes,
            )
            .unwrap_err();
            assert_eq!(fallback.to_string(), error.to_string());
            assert!(bytes.is_empty());
            let ordinary = KernelError::new("ordinary type error");
            assert_eq!(
                emit(&ordinary, &origins, &CancellationToken::new(), &mut bytes)
                    .unwrap_err()
                    .to_string(),
                "ordinary type error"
            );
            assert!(bytes.is_empty());
        }
    }

    #[test]
    fn pattern_load_boundary_preserves_cancellation_writer_errors_and_fallback() {
        struct Writer(bool);
        impl Write for Writer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                if self.0 {
                    Err(io::Error::other("write failed"))
                } else {
                    Ok(buf.len())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("flush failed"))
            }
        }
        for (file, excerpt) in [
            (
                "cpu-comptime-rebind/comptime/err_rebind.bend",
                "10>|   f = Nat.add(x)",
            ),
            (
                "cpu-comptime-rebind/negative/imported.bend",
                "4>| def main() -> Nat: D.bad(~D.inc, 10n)(1n)",
            ),
            (
                "cpu-comptime-rebind/negative/nested.bend",
                "6>|   Wrap{f} = x",
            ),
            (
                "cpu-untyped-binder/comptime/err_untyped.bend",
                "4>| def app(~f, x: Nat) -> Nat:",
            ),
            (
                "cpu-untyped-binder/negative/imported.bend",
                "3>| def bad(~n, x: U32) -> U32: x",
            ),
            (
                "cpu-unknown-template/negative/template_missing.bend",
                "4>| def main() -> U32: D.missing(~9)",
            ),
            (
                "cpu-unknown-template/negative/imported.bend",
                "4>| def broken() -> U32: missing(~9)",
            ),
            (
                "cpu-duplicate-declaration/check/data_table_checked_000.bend",
                "11>| law String.eq:",
            ),
            (
                "cpu-duplicate-declaration/controls/imported/main.bend",
                "4>| law value: U32",
            ),
            (
                "cpu-duplicate-constructor/check/book_prefix_arity.bend",
                "12>|   K6{a: Un6, b: Un6}",
            ),
            (
                "cpu-duplicate-constructor/controls/imported/main.bend",
                "5>|   Dup{}",
            ),
            (
                "cpu-matched-values/arity_mismatch_000.bend",
                "19>|     case Zw{}, x, Zw{}:",
            ),
            (
                "cpu-scrutinee-origins/error_window_substituted_scrutinee.bend",
                "12>|       match y:",
            ),
            (
                "cpu-scrutinee-origins/consumed_column_error_001.bend",
                "19>|       match a:",
            ),
            (
                "cpu-computed-scrutinee/motive_suffix_join.bend",
                "14>|   match Bool.not(n):",
            ),
            (
                "cpu-constructors/nested-pattern.bend",
                "7>|       Missing{}",
            ),
            (
                "cpu-rewrite-checking/dead_motive_syntax.bend",
                "13>|   %[h => {y == h : U32}] e {=}",
            ),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(file);
            let mut error = load_with_diagnostics(path)
                .err()
                .expect("pattern load rejects");
            let mut output = Vec::new();
            let token = CancellationToken::new();
            token.request_cancel("stop load diagnostic");
            assert!(
                emit_load(&error, &token, &mut output)
                    .unwrap_err()
                    .to_string()
                    .contains("stop load diagnostic")
            );
            assert!(output.is_empty());
            for fail_write in [true, false] {
                let failure = emit_load(&error, &CancellationToken::new(), &mut Writer(fail_write))
                    .unwrap_err();
                assert!(failure.to_string().contains(if fail_write {
                    "cannot write"
                } else {
                    "cannot flush"
                }));
            }
            let result = emit_load(&error, &CancellationToken::new(), &mut output).unwrap();
            assert_eq!(result.exit_code(), 1);
            assert!(String::from_utf8(output).unwrap().contains(excerpt));
            error.diagnostic = None;
            let mut bytes = Vec::new();
            assert_eq!(
                emit_load(&error, &CancellationToken::new(), &mut bytes)
                    .unwrap_err()
                    .to_string(),
                error.error.to_string()
            );
            assert!(bytes.is_empty());
        }
    }

    #[test]
    fn diagnostic_write_and_flush_failures_remain_host_errors() {
        struct Writer {
            fail_write: bool,
            flushed: bool,
        }
        impl Write for Writer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                if self.fail_write {
                    Err(io::Error::other("write failed"))
                } else {
                    Ok(buf.len())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                self.flushed = true;
                Err(io::Error::other("flush failed"))
            }
        }
        for name in [
            "tests/fixtures/cpu-contextual-proof-diagnostics/check/default_arm_family.bend",
            "tests/fixtures/cpu-contextual-proof-diagnostics/check/neq_reflexivity.bend",
            "tests/fixtures/cpu-comptime-type/comptime/wrong_type.bend",
            "tests/fixtures/cpu-comptime-type/negative/imported.bend",
            "tests/fixtures/cpu-comptime-type/negative/distinct_uses.bend",
            "tests/fixtures/cpu-match-guards/match_not_datatype.bend",
            "tests/fixtures/cpu-match-guards/error_window_match_repeated.bend",
            "tests/fixtures/cpu-match-guards/erased_match_ctor.bend",
            "tests/fixtures/cpu-checking/false_brace.bend",
            "tests/fixtures/cpu-constructors/nat_literal_import.bend",
            "tests/fixtures/cpu-plus-binders/plus_binder_term.bend",
            "tests/fixtures/cpu-rewrite-checking/rewrite_ctor_brace.bend",
            "tests/fixtures/cpu-rewrite-proof/rewrite_non_equation.bend",
            "tests/fixtures/cpu-rewrite-proof/error_window_rewrite_proof.bend",
            "tests/fixtures/cpu-rewrite-proof/reject_rewrite.bend",
        ] {
            let (error, origins) = rejected(name);
            for fail_write in [true, false] {
                let mut writer = Writer {
                    fail_write,
                    flushed: false,
                };
                let failure =
                    emit(&error, &origins, &CancellationToken::new(), &mut writer).unwrap_err();
                assert!(failure.to_string().contains(if fail_write {
                    "cannot write"
                } else {
                    "cannot flush"
                }));
                assert_eq!(writer.flushed, !fail_write);
            }
            let output =
                emit(&error, &origins, &CancellationToken::new(), &mut Vec::new()).unwrap();
            assert_eq!(output.exit_code(), 1);
        }
    }
}
