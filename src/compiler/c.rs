// SPDX-License-Identifier: MPL-2.0

use super::CompileError;
use crate::kernel::Book;
use crate::kernel::Declaration;
use crate::kernel::DefDecl;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::check_book;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write;

const RUNTIME: &str = include_str!("c_runtime.c");
const TABLE_MARKER: &str = "/* BEND_GENERATED_TABLES */";
const MAX_NODES: usize = 250_000;

/// Compile a complete checked book into a standalone portable C11 program.
///
/// The entry must have no declared arguments. The emitted program uses a bounded
/// lazy runtime with closures, memoized constructor fields and simultaneous let
/// bindings. It prints the same constructor JSON and explicit erased marker as
/// the JavaScript backend. A function result or exhausted runtime budget is an
/// error. All owned runtime allocations are released before the program exits.
///
/// This baseline backend emits immutable instructions and their runtime; it
/// performs no upstream optimizations, executes no source during generation,
/// and does not invoke a C compiler. Input definitions retain their licenses.
///
/// # Errors
/// Rejects incomplete or invalid books, unknown or non-closed entries, unknown
/// references and programs exceeding the code-generation instruction budget.
pub fn compile_c(book: &Book, entry: &str) -> Result<String, CompileError> {
    check_book(book).map_err(|error| CompileError::new(error.to_string()))?;
    let mut definitions = BTreeMap::<&str, &DefDecl>::new();
    let mut datatypes = BTreeSet::new();
    for declaration in &book.declarations {
        match declaration {
            Declaration::Def(definition) => {
                definitions.insert(definition.name.as_str(), definition);
            }
            Declaration::Adt(datatype) => {
                datatypes.insert(datatype.name.as_str());
            }
        }
    }
    let selected = definitions
        .get(entry)
        .ok_or_else(|| CompileError::new(format!("undefined entry point {entry}")))?;
    if !selected.parameters.is_empty() {
        return Err(CompileError::new(format!(
            "standalone entry {entry} requires {} arguments; define a closed entry instead",
            selected.parameters.len()
        )));
    }
    let indices = definitions
        .keys()
        .enumerate()
        .map(|(index, name)| (*name, index))
        .collect();
    let mut generator = Generator {
        indices,
        datatypes,
        nodes: Vec::new(),
        links: Vec::new(),
    };
    let mut roots = Vec::new();
    for definition in definitions.values() {
        let body = definition
            .body
            .as_ref()
            .ok_or_else(|| CompileError::new(format!("unfilled definition {}", definition.name)))?;
        roots.push(generator.lower(body)?);
    }
    let mut tables = String::new();
    writeln!(tables, "#define BEND_DEFINITION_COUNT {}", roots.len()).unwrap();
    writeln!(
        tables,
        "#define BEND_ENTRY_INDEX {}",
        generator.indices[entry]
    )
    .unwrap();
    writeln!(tables, "static const Expr program[] = {{").unwrap();
    for node in generator.nodes {
        writeln!(tables, "  {node},").unwrap();
    }
    writeln!(tables, "}};\nstatic const uint64_t links[] = {{").unwrap();
    if generator.links.is_empty() {
        generator.links.push(0);
    }
    for link in generator.links {
        writeln!(tables, "  UINT64_C({link}),").unwrap();
    }
    writeln!(tables, "}};\nstatic const size_t definition_roots[] = {{").unwrap();
    for root in roots {
        writeln!(tables, "  {root},").unwrap();
    }
    writeln!(tables, "}};").unwrap();
    Ok(RUNTIME.replace(TABLE_MARKER, &tables))
}

struct Generator<'book> {
    indices: BTreeMap<&'book str, usize>,
    datatypes: BTreeSet<&'book str>,
    nodes: Vec<String>,
    links: Vec<usize>,
}

impl Generator<'_> {
    fn lower(&mut self, value: &TermRef) -> Result<usize, CompileError> {
        let node = match value.as_ref() {
            Term::Var { id, .. } => format!("{{N_VAR, 0, 0, UINT64_C({id}), 0, 0, NULL, 0}}"),
            Term::Ref(name) => match self.indices.get(name.as_str()) {
                Some(index) => format!("{{N_REF, {index}, 0, 0, 0, 0, NULL, 0}}"),
                None if self.datatypes.contains(name.as_str()) => erased_node(),
                None => {
                    return Err(CompileError::new(format!(
                        "undefined compiled reference {name}"
                    )));
                }
            },
            Term::Lam { id, body, .. } => {
                let body = self.lower(body)?;
                format!("{{N_LAM, {body}, 0, UINT64_C({id}), 0, 0, NULL, 0}}")
            }
            Term::App(function, argument) => {
                let function = self.lower(function)?;
                let argument = self.lower(argument)?;
                format!("{{N_APP, {function}, {argument}, 0, 0, 0, NULL, 0}}")
            }
            Term::Ctr { name, args } => {
                let fields = args
                    .iter()
                    .map(|arg| self.lower(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let start = self.links.len();
                self.links.extend(fields);
                format!(
                    "{{N_CTR, 0, 0, 0, {start}, {}, {}, {}}}",
                    args.len(),
                    c_string(name),
                    name.len()
                )
            }
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => {
                let arm = self.lower(arm)?;
                let fallback = self.lower(fallback)?;
                format!(
                    "{{N_MAT, {arm}, {fallback}, 0, 0, 0, {}, {}}}",
                    c_string(constructor),
                    constructor.len()
                )
            }
            Term::Efq => "{N_EFQ, 0, 0, 0, 0, 0, NULL, 0}".to_owned(),
            Term::Let { bindings, body } => {
                let mut entries = Vec::new();
                for binding in bindings {
                    entries.push(binding.id);
                    entries.push(self.lower(&binding.value)?);
                }
                let body = self.lower(body)?;
                let start = self.links.len();
                self.links.extend(entries);
                format!(
                    "{{N_LET, {body}, 0, 0, {start}, {}, NULL, 0}}",
                    bindings.len()
                )
            }
            Term::Ann(value, _) | Term::Rwt { body: value, .. } => return self.lower(value),
            Term::Typ(_)
            | Term::Qnt
            | Term::Qua(_)
            | Term::Min(_, _)
            | Term::All { .. }
            | Term::Adt { .. }
            | Term::Eql { .. }
            | Term::Rfl => erased_node(),
            Term::Hole(name) => {
                return Err(CompileError::new(format!("unfinished proof hole ?{name}")));
            }
        };
        if self.nodes.len() >= MAX_NODES {
            return Err(CompileError::new("C instruction budget exhausted"));
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        Ok(index)
    }
}

fn erased_node() -> String {
    "{N_ERASED, 0, 0, 0, 0, 0, NULL, 0}".to_owned()
}

// Fixed-width octal escapes preserve every UTF-8 byte, including embedded NUL,
// without source-name interpolation, greedy hex escapes or locale dependence.
fn c_string(value: &str) -> String {
    let mut output = String::from("\"");
    for byte in value.bytes() {
        write!(output, "\\{byte:03o}").unwrap();
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::c_string;

    #[test]
    fn c_strings_encode_all_bytes_without_literal_source_names() {
        assert_eq!(c_string("\"\\\n\0é"), "\"\\042\\134\\012\\000\\303\\251\"");
    }
}
