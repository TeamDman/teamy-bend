// SPDX-License-Identifier: MPL-2.0
//! Foreign source assembly for the execution-only JavaScript backend.

use super::CompileError;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::kernel::elaborate::Expression;
use crate::kernel::elaborate::ExpressionKind;
use crate::syntax::executable::BuiltinForeign;
use crate::syntax::executable::ForeignTarget;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::fmt::Write;
use std::fs;

const DRIVER: &str = include_str!("executable_io.js");

pub(super) struct ForeignAssembly {
    pub(super) source: String,
    pub(super) indices: BTreeMap<String, usize>,
}

/// Read selected source files without executing them. All selected modules share
/// one private lexical scope, matching upstream's canonical-file deduplication.
pub(super) fn assemble(program: &ExecutableProgram) -> Result<ForeignAssembly, CompileError> {
    let mut indices = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut imports = String::new();
    let mut entries = Vec::new();
    for name in reference_order(program) {
        let definition = &program.definitions[name];
        let DefinitionBody::Foreign(foreign) = &definition.body else {
            continue;
        };
        indices.insert(name.to_owned(), entries.len());
        if let Some(builtin) = foreign.builtin {
            let function = match builtin {
                BuiltinForeign::Print => ("$tbPrint", "undefined"),
                BuiltinForeign::Write => ("$tbWrite", "undefined"),
                BuiltinForeign::PrintErr => ("$tbPrintErr", "undefined"),
                BuiltinForeign::Spawn => ("$tbSpawn", "undefined"),
                BuiltinForeign::Sleep => ("$tbSleep", "$tbSleepNeed"),
                BuiltinForeign::Now => ("$tbNow", "undefined"),
                BuiltinForeign::ChanNew => ("$tbChanNew", "undefined"),
                BuiltinForeign::ChanSend => ("$tbChanSend", "undefined"),
                BuiltinForeign::ChanRecv => ("$tbChanRecv", "undefined"),
                BuiltinForeign::ChanClose => ("$tbChanClose", "undefined"),
            };
            // Resolve these outside the foreign lexical scope: a companion
            // file declaring the same name cannot replace bundled contracts.
            entries.push((Some(function), String::new()));
            continue;
        }
        let import = foreign
            .imports
            .iter()
            .find(|import| import.target == ForeignTarget::JavaScript)
            .ok_or_else(|| {
                CompileError::new(format!(
                    "foreign definition {name} has no JavaScript import"
                ))
            })?;
        let path = fs::canonicalize(&import.path).map_err(|error| {
            CompileError::new(format!(
                "cannot resolve JavaScript import for {name}: {error}"
            ))
        })?;
        if seen.insert(path.clone()) {
            let source = fs::read_to_string(&path).map_err(|error| {
                CompileError::new(format!("cannot read JavaScript import for {name}: {error}"))
            })?;
            imports.push_str(&source);
            // A final line comment in one file cannot swallow the next file.
            imports.push('\n');
        }
        let symbol = &foreign.local_symbol;
        if !symbol.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            || !symbol
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(CompileError::new(format!(
                "invalid JavaScript foreign symbol for {name}"
            )));
        }
        entries.push((None, format!(
            "{{run:typeof {symbol}==='function'?{symbol}:undefined,need:typeof {symbol}_need==='function'?{symbol}_need:undefined}}"
        )));
    }
    let mut source = String::from(DRIVER);
    source.push_str("\nconst $tbForeign = (() => {\n");
    source.push_str(&imports);
    source.push_str("return [\n");
    for (builtin, expression) in &entries {
        writeln!(
            source,
            "{},",
            if builtin.is_some() {
                "null"
            } else {
                expression
            }
        )
        .unwrap();
    }
    source.push_str("];\n})();\n");
    for (index, (builtin, _)) in entries.iter().enumerate() {
        if let Some((function, need)) = builtin {
            writeln!(
                source,
                "$tbForeign[{index}] = {{run:{function},need:{need}}};"
            )
            .unwrap();
        }
    }
    Ok(ForeignAssembly { source, indices })
}

/// Upstream assembles foreign files in breadth-first live-reference order from
/// main. File initialization can depend on earlier files, so sorting definition
/// names would change program behavior even though canonical dedup still works.
fn reference_order(program: &ExecutableProgram) -> Vec<&str> {
    let mut queue = VecDeque::from(["main"]);
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    while let Some(name) = queue.pop_front() {
        if !seen.insert(name) {
            continue;
        }
        let Some(definition) = program.definitions.get(name) else {
            continue;
        };
        ordered.push(name);
        if let DefinitionBody::Ordinary(expression) = &definition.body {
            let mut pending = vec![expression];
            while let Some(expression) = pending.pop() {
                push_references(expression, &mut pending, &mut queue);
            }
        }
    }
    ordered
}

fn push_references<'a>(
    expression: &'a Expression,
    pending: &mut Vec<&'a Expression>,
    queue: &mut VecDeque<&'a str>,
) {
    match &expression.kind {
        ExpressionKind::Definition(name) => queue.push_back(name),
        ExpressionKind::Lambda { body, .. } => pending.push(body),
        ExpressionKind::Apply {
            function, argument, ..
        } => {
            pending.push(argument);
            pending.push(function);
        }
        ExpressionKind::Constructor { fields, .. } => {
            pending.extend(fields.iter().rev().map(|field| &field.value));
        }
        ExpressionKind::Match { arm, fallback, .. } => {
            pending.push(fallback);
            pending.push(arm);
        }
        ExpressionKind::Let { bindings, body } => {
            pending.push(body);
            pending.extend(bindings.iter().rev().map(|field| &field.value));
        }
        ExpressionKind::Erased | ExpressionKind::Variable(_) | ExpressionKind::Absurd { .. } => {}
    }
}

#[cfg(test)]
#[path = "executable_foreign_tests.rs"]
mod tests;
