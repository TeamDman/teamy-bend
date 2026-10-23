// SPDX-License-Identifier: MPL-2.0
//! Compare a checked definition's core structure with captured interpreter output.
#[path = "../tests/support/normalization_shape.rs"]
mod normalization_shape;

use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::io::Write;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::ExecutableEntry;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;
use teamy_bend::syntax::parse_term;

#[derive(Facet)]
struct Report {
    core_term_observed: bool,
    structural_match: bool,
    exact_text_match: bool,
    port_value: String,
    upstream_value: String,
    port_shape: Shape,
    upstream_shape: Shape,
}

fn main() -> eyre::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let executable_function = args.len() == 3 && args[2] == "--executable-function";
    if args.len() != 2 && !executable_function {
        eyre::bail!(
            "usage: compare_normalization <source.bend> <oracle-stdout> [--executable-function]"
        );
    }
    let path = Path::new(&args[0]);
    let (value, book, port_value) = if executable_function {
        // Executable contracts cannot become proof certificates. Observe the
        // existing pure-main writer, then compare its parsed output; a separate
        // in-crate regression checks the actual normalized core for these inputs.
        let checked = check_executable(&load_executable(path)?)?;
        if checked.entry_kind()? != ExecutableEntry::Pure {
            eyre::bail!("executable-function comparison requires a pure main");
        }
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let status = checked.run_main(&mut stdout, &mut stderr, &|| false)?;
        if status != 0 || !stderr.is_empty() {
            eyre::bail!("unexpected executable-function status or stderr");
        }
        let text = String::from_utf8(stdout)?;
        let text = text
            .strip_suffix('\n')
            .ok_or_else(|| eyre::eyre!("missing result newline"))?;
        let value = parse_term(text)?;
        if !matches!(value.as_ref(), Term::Lam { .. }) {
            eyre::bail!("executable-function comparison requires a function result");
        }
        (value, None, text.to_owned())
    } else {
        let book = load(path)?;
        let value = check_book(&book)?.normalize_definition("main")?;
        let port_value = value.to_string();
        (value, Some(book), port_value)
    };
    let raw = std::fs::read_to_string(&args[1])?;
    let upstream = raw
        .strip_suffix('\n')
        .ok_or_else(|| eyre::eyre!("oracle output needs a final newline"))?;
    let shape = |value| match &book {
        Some(book) => shape_in_book(value, book),
        None => normalization_shape::shape(value),
    };
    let port_shape = shape(&value).map_err(eyre::Report::msg)?;
    let upstream_shape = shape(&parse_term(upstream)?).map_err(eyre::Report::msg)?;
    let report = Report {
        core_term_observed: !executable_function,
        structural_match: port_shape == upstream_shape,
        exact_text_match: port_value == upstream,
        port_value,
        upstream_value: upstream.to_owned(),
        port_shape,
        upstream_shape,
    };
    writeln!(
        std::io::stdout().lock(),
        "{}",
        facet_json::to_string_pretty(&report)?
    )?;
    if !report.structural_match {
        eyre::bail!("normalized structures differ");
    }
    Ok(())
}
