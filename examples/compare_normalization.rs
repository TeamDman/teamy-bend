// SPDX-License-Identifier: MPL-2.0
//! Compare a checked definition's core structure with captured interpreter output.
#[path = "../tests/support/normalization_shape.rs"]
mod normalization_shape;

use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::io::Write;
use std::path::Path;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

#[derive(Facet)]
struct Report {
    structural_match: bool,
    exact_text_match: bool,
    port_value: String,
    upstream_value: String,
    port_shape: Shape,
    upstream_shape: Shape,
}

fn main() -> eyre::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        eyre::bail!("usage: compare_normalization <source.bend> <oracle-stdout>");
    }
    let book = load(Path::new(&args[0]))?;
    let checked = check_book(&book)?;
    let value = checked.normalize_definition("main")?;
    let raw = std::fs::read_to_string(&args[1])?;
    let upstream = raw
        .strip_suffix('\n')
        .ok_or_else(|| eyre::eyre!("oracle output needs a final newline"))?;
    let port_shape = shape_in_book(&value, &book).map_err(eyre::Report::msg)?;
    let upstream_shape = shape_in_book(&parse_term(upstream)?, &book).map_err(eyre::Report::msg)?;
    let port_value = value.to_string();
    let report = Report {
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
