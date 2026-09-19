// SPDX-License-Identifier: MPL-2.0

use crate::kernel::Book;
use crate::kernel::Declaration;
use crate::kernel::DefDecl;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::check_book;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Write;
use std::fmt::{self};

/// A failed proof check or an unsupported standalone entry point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileError {
    message: String,
}

impl CompileError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CompileError {}

/// Compile a checked, argument-free definition to a standalone Node.js program.
///
/// The generated program prints one JSON object with a `value` property.
/// Constructors are represented as `{"constructor":"Name","fields":[...]}`;
/// erased type, quantity and proof values become `{"erased":true}`. Functions
/// work internally, but a function in the final result is a runtime error.
///
/// Every definition is checked before code generation. The backend preserves
/// lazy function application, constructor fields, simultaneous lets and curried
/// pattern matching. Equality rewrites erase to their checked bodies. No source
/// name is inserted as a JavaScript identifier or unescaped string literal.
///
/// This function does not run Node.js or evaluate the selected entry point.
/// Input definitions retain their original licenses; compiling does not apply
/// the generator's license to user programs.
///
/// # Errors
/// Returns an error for an invalid or incomplete book, an unknown entry, an
/// entry requiring arguments, or a reference outside the checked book.
pub fn compile_javascript(book: &Book, entry: &str) -> Result<String, CompileError> {
    check_book(book).map_err(|error| CompileError::new(error.to_string()))?;
    let mut definitions = BTreeMap::<&str, &DefDecl>::new();
    let mut datatypes = BTreeSet::new();
    for declaration in &book.declarations {
        match declaration {
            Declaration::Def(definition) => {
                definitions.insert(&definition.name, definition);
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
    let generator = Generator { indices, datatypes };
    let mut source = String::from(RUNTIME);
    for (index, definition) in definitions.values().enumerate() {
        let body = definition
            .body
            .as_ref()
            .ok_or_else(|| CompileError::new(format!("unfilled definition {}", definition.name)))?;
        let expression = generator.expression(body)?;
        writeln!(source, "definitions[{index}] = lazy(() => {expression});")
            .map_err(|error| CompileError::new(error.to_string()))?;
    }
    let selected_index = generator.indices[entry];
    writeln!(
        source,
        "try {{\n  process.stdout.write(JSON.stringify({{value: materialize(definitions[{selected_index}])}}) + '\\n');\n}} catch (error) {{\n  process.stderr.write('teamy-bend generated program: ' + String(error.message ?? error) + '\\n');\n  process.exitCode = 1;\n}}"
    )
    .map_err(|error| CompileError::new(error.to_string()))?;
    Ok(source)
}

struct Generator<'book> {
    indices: BTreeMap<&'book str, usize>,
    datatypes: BTreeSet<&'book str>,
}

impl Generator<'_> {
    fn expression(&self, value: &TermRef) -> Result<String, CompileError> {
        let result = match value.as_ref() {
            Term::Var { id, .. } => format!("v{id}"),
            Term::Ref(name) => match self.indices.get(name.as_str()) {
                Some(index) => format!("definitions[{index}]"),
                None if self.datatypes.contains(name.as_str()) => "erased".to_owned(),
                None => {
                    return Err(CompileError::new(format!(
                        "undefined compiled reference {name}"
                    )));
                }
            },
            Term::Lam { id, body, .. } => {
                format!("lazy(() => (v{id}) => {})", self.expression(body)?)
            }
            Term::App(function, argument) => format!(
                "lazy(() => apply({}, {}))",
                self.expression(function)?,
                self.expression(argument)?
            ),
            Term::Ctr { name, args } => format!(
                "lazy(() => ({{constructor: {}, fields: [{}]}}))",
                javascript_string(name),
                self.expressions(args)?
            ),
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => format!(
                "lazy(() => (scrutinee) => matchConstructor(scrutinee, {}, {}, {}))",
                javascript_string(constructor),
                self.expression(arm)?,
                self.expression(fallback)?
            ),
            Term::Efq => {
                "lazy(() => () => { throw new Error('entered an impossible match branch'); })"
                    .to_owned()
            }
            Term::Let { bindings, body } => {
                let parameters = bindings
                    .iter()
                    .map(|binding| format!("v{}", binding.id))
                    .collect::<Vec<_>>()
                    .join(", ");
                let values = bindings
                    .iter()
                    .map(|binding| self.expression(&binding.value))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!(
                    "lazy(() => (({parameters}) => {})({values}))",
                    self.expression(body)?
                )
            }
            Term::Ann(value, _) => self.expression(value)?,
            Term::Rwt { body, .. } => self.expression(body)?,
            Term::Typ(_)
            | Term::Qnt
            | Term::Qua(_)
            | Term::Min(_, _)
            | Term::All { .. }
            | Term::Adt { .. }
            | Term::Eql { .. }
            | Term::Rfl => "erased".to_owned(),
            Term::Hole(name) => {
                return Err(CompileError::new(format!("unfinished proof hole ?{name}")));
            }
        };
        Ok(result)
    }

    fn expressions(&self, values: &[TermRef]) -> Result<String, CompileError> {
        Ok(values
            .iter()
            .map(|value| self.expression(value))
            .collect::<Result<Vec<_>, _>>()?
            .join(", "))
    }
}

fn javascript_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            character if character <= '\u{001f}' => {
                // Writing to a String cannot fail.
                write!(escaped, "\\u{:04x}", u32::from(character))
                    .expect("writing to a string cannot fail");
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

const RUNTIME: &str = r"// Generated by teamy-bend. Runtime support: MPL-2.0.
// Input definitions retain their original licenses; generation does not relicense them.
'use strict';
const thunkMarker = Symbol('Bend thunk');
const erased = Object.freeze({erased: true});
const definitions = [];
let remainingSteps = 2000000;
function tick() {
  if (--remainingSteps < 0) throw new Error('evaluation budget exhausted');
}
function lazy(run) {
  return {[thunkMarker]: true, state: 0, run, value: undefined};
}
function force(value) {
  while (value && value[thunkMarker]) {
    tick();
    if (value.state === 1) throw new Error('cyclic evaluation');
    if (value.state === 0) {
      value.state = 1;
      value.value = value.run();
      value.run = undefined;
      value.state = 2;
    }
    value = value.value;
  }
  return value;
}
function apply(fn, argument) {
  tick();
  const callable = force(fn);
  if (typeof callable !== 'function') throw new Error('application of a non-function');
  return callable(argument);
}
function matchConstructor(scrutinee, name, arm, fallback) {
  tick();
  const value = force(scrutinee);
  if (!value || typeof value.constructor !== 'string' || !Array.isArray(value.fields)) {
    throw new Error('pattern match expected constructor data');
  }
  if (value.constructor !== name) return apply(fallback, scrutinee);
  let result = arm;
  for (const field of value.fields) result = apply(result, field);
  return result;
}
function materialize(thunk, depth = 0) {
  tick();
  if (depth > 1024) throw new Error('result nesting limit exhausted');
  const value = force(thunk);
  if (value === erased) return {erased: true};
  if (typeof value === 'function') throw new Error('standalone entry must return data, not a function');
  if (!value || typeof value.constructor !== 'string' || !Array.isArray(value.fields)) {
    throw new Error('standalone entry produced unsupported runtime data');
  }
  return {constructor: value.constructor, fields: value.fields.map(field => materialize(field, depth + 1))};
}
";

#[cfg(test)]
mod tests {
    use super::javascript_string;

    #[test]
    fn constructor_names_cannot_break_javascript_string_literals() {
        assert_eq!(
            javascript_string("\"; throw Error('injected'); //\\\n\u{2028}\u{0000}"),
            "\"\\\"; throw Error('injected'); //\\\\\\n\\u2028\\u0000\""
        );
    }
}
