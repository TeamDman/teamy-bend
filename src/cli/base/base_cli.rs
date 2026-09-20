// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Context;
use facet::Facet;
use figue as args;
use std::io::Write;
use teamy_cancellation::CancellationToken;

const SOURCE: &str = include_str!("../../syntax/base.bend");

/// Print the bundled Base source, its types, or a declaration and its subnames.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct BaseArgs {
    /// Optional exact name or namespace, such as List or Nat.add.
    #[facet(args::positional, default)]
    pub name: Option<String>,
    /// Print datatype declarations and laws whose result is a kind.
    #[facet(args::named, default)]
    pub types: bool,
}

impl BaseArgs {
    /// Print source text independently of the structured output setting.
    ///
    /// # Errors
    /// Returns an error for conflicting selectors, unknown names, cancellation,
    /// or an output failure.
    pub fn invoke(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        if self.types && self.name.is_some() {
            eyre::bail!("base accepts either --types or a name");
        }
        let source = if self.types || self.name.is_some() {
            select_source(self.name.as_deref(), self.types)?
        } else {
            SOURCE.to_owned()
        };
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(source.as_bytes())
            .wrap_err("cannot write Base source")?;
        stdout.flush().wrap_err("cannot flush Base source")?;
        Ok(CliOutput::none())
    }
}

fn select_source(name: Option<&str>, types: bool) -> eyre::Result<String> {
    let mut blocks = Vec::new();
    let mut start = 0;
    let mut offset = 0;
    for line in SOURCE.split_inclusive('\n') {
        if offset > 0
            && ["type ", "law ", "def ", "@"]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        {
            blocks.push(&SOURCE[start..offset]);
            start = offset;
        }
        offset += line.len();
    }
    blocks.push(&SOURCE[start..]);
    let mut selected = Vec::new();
    for block in blocks {
        let Some((kind, declared)) = block.lines().find_map(|line| {
            let (kind, tail) = line.split_once(' ')?;
            if !["type", "law", "def"].contains(&kind) {
                return None;
            }
            let declared = tail
                .split(|c: char| c.is_whitespace() || "(<:".contains(c))
                .next()?;
            Some((kind, declared))
        }) else {
            continue;
        };
        let trimmed = block.trim_end_matches(|c: char| c.is_whitespace());
        let content = trimmed.lines().collect::<Vec<_>>();
        let end = content
            .iter()
            .rposition(|line| !line.is_empty() && !line.starts_with('#'));
        let Some(end) = end else { continue };
        let last = content[end].trim();
        let matches = if types {
            kind == "type"
                || (kind == "law"
                    && (matches!(last, "Type" | "Data")
                        || (last.starts_with("Kind(") && last.ends_with(')'))))
        } else {
            name.is_some_and(|name| {
                declared == name
                    || declared
                        .strip_prefix(name)
                        .is_some_and(|tail| tail.starts_with('.'))
            })
        };
        if matches {
            selected.push(content[..=end].join("\n"));
        }
    }
    if selected.is_empty() {
        eyre::bail!("Base has no {}", name.unwrap_or("type declarations"));
    }
    Ok(selected.join("\n\n") + "\n")
}
