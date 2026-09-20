// SPDX-License-Identifier: Apache-2.0
// Native JS representation derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Executable code generation uses checked runtime contracts, never proof tokens.

use super::CompileError;
use super::executable_foreign;
use crate::kernel::ExecutableBook;
use crate::kernel::ExecutableEntry;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::kernel::elaborate::Expression;
use crate::kernel::elaborate::ExpressionKind;
use crate::kernel::substitute;
use crate::syntax::executable::NumericIntrinsic;
use crate::syntax::executable::OpaqueType;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::rc::Rc;

/// Compile a checked executable into a standalone `CommonJS` Node.js program.
///
/// IO uses native JavaScript values, curried callbacks, and the synchronous
/// foreign-import protocol. Foreign source is read and embedded, never executed
/// during compilation. Runtime contracts do not become strict proof evidence.
/// Pure printable entries use Bend's textual value notation; a missing entry
/// reports successful checking. The existing strict data compiler is unchanged.
///
/// # Errors
/// Rejects unsupported pure result types, unavailable reachable foreign imports,
/// unsupported code generation and exhausted type-lowering resource limits.
pub fn compile_executable_javascript(book: &ExecutableBook) -> Result<String, CompileError> {
    let program = book
        .lower_for_compilation()
        .map_err(|error| CompileError::new(error.to_string()))?;
    let foreign = executable_foreign::assemble(&program)?;
    let indices = program
        .definitions
        .keys()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();
    let generator = Generator {
        program: &program,
        indices,
    };
    let mut source = String::from(include_str!("executable_core.js"));
    source.push_str(&foreign.source);
    source.push_str("\nconst $tbDefinitions = [];\n");
    for (name, definition) in &program.definitions {
        let arity = definition
            .parameters
            .iter()
            .filter(|p| p.quant != Quant::None)
            .count();
        let body = match &definition.body {
            DefinitionBody::OpaqueType(
                OpaqueType::Chan | OpaqueType::File | OpaqueType::Socket | OpaqueType::Listener,
            ) => "null".into(),
            DefinitionBody::Numeric(intrinsic) => native_function(numeric_name(*intrinsic), arity),
            DefinitionBody::Foreign(_) => {
                let index = foreign
                    .indices
                    .get(name)
                    .ok_or_else(|| CompileError::new(format!("foreign assembly omitted {name}")))?;
                let arguments = (0..arity)
                    .map(|index| format!("a{index}"))
                    .collect::<Vec<_>>();
                let mut parameters = arguments.clone();
                parameters.push("k".into());
                format!(
                    "$tb.curry({}, ({}) => $tbMakeRequest({index}, [{}], k))",
                    arity + 1,
                    parameters.join(","),
                    arguments.join(",")
                )
            }
            DefinitionBody::Ordinary(body) => {
                if program.base_names.contains(name) && optimized(name) {
                    native_function(name, arity)
                } else {
                    generator.expression(body, true)?
                }
            }
        };
        writeln!(
            source,
            "$tbDefinitions[{}] = () => {{ $tbTick(); return {body}; }};",
            generator.indices[name.as_str()]
        )
        .expect("writing to a string cannot fail");
    }
    let (main, is_io, printer) = match program.entry {
        ExecutableEntry::Missing => ("null".into(), false, "null".into()),
        ExecutableEntry::Io => (
            format!("$tbDefinitions[{}]", generator.indices["main"]),
            true,
            "null".into(),
        ),
        ExecutableEntry::Pure => {
            let mut printer = Printer {
                program: &program,
                nodes: Vec::new(),
                seen: BTreeMap::new(),
            };
            let root = printer.node(&program.definitions["main"].ty, 0)?;
            let nodes = printer.nodes.join(",\n");
            writeln!(source, "const $tbPrintTypes = [{nodes}];")
                .expect("writing to a string cannot fail");
            (
                format!("$tbDefinitions[{}]", generator.indices["main"]),
                false,
                format!("value => $tb.show($tbPrintTypes,{root},value)"),
            )
        }
    };
    writeln!(
        source,
        "process.exitCode = $tbRunMain({main}, {is_io}, {printer});"
    )
    .expect("writing to a string cannot fail");
    Ok(source)
}

fn native_function(name: &str, arity: usize) -> String {
    let args = (0..arity)
        .map(|index| format!("a{index}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "$tb.curry({arity}, ({args}) => $tb.native({},[{args}]))",
        quote(name)
    )
}

const fn numeric_name(intrinsic: NumericIntrinsic) -> &'static str {
    match intrinsic {
        NumericIntrinsic::U32ToF32 => "U32.to_f32",
        NumericIntrinsic::F32ToU32 => "F32.to_u32",
        NumericIntrinsic::Add => "F32.add",
        NumericIntrinsic::Sub => "F32.sub",
        NumericIntrinsic::Mul => "F32.mul",
        NumericIntrinsic::Div => "F32.div",
        NumericIntrinsic::Mod => "F32.mod",
        NumericIntrinsic::Neg => "F32.neg",
        NumericIntrinsic::Abs => "F32.abs",
        NumericIntrinsic::Bits => "F32.bits",
        NumericIntrinsic::IsEq => "F32.is_eq",
        NumericIntrinsic::IsNe => "F32.is_ne",
        NumericIntrinsic::IsLt => "F32.is_lt",
        NumericIntrinsic::IsLe => "F32.is_le",
        NumericIntrinsic::IsGt => "F32.is_gt",
        NumericIntrinsic::IsGe => "F32.is_ge",
        NumericIntrinsic::Pow => "F32.pow",
        NumericIntrinsic::Atan2 => "F32.atan2",
        NumericIntrinsic::Sqrt => "F32.sqrt",
        NumericIntrinsic::Exp => "F32.exp",
        NumericIntrinsic::Log => "F32.log",
        NumericIntrinsic::Log2 => "F32.log2",
        NumericIntrinsic::Log10 => "F32.log10",
        NumericIntrinsic::Sin => "F32.sin",
        NumericIntrinsic::Cos => "F32.cos",
        NumericIntrinsic::Tan => "F32.tan",
        NumericIntrinsic::Asin => "F32.asin",
        NumericIntrinsic::Acos => "F32.acos",
        NumericIntrinsic::Atan => "F32.atan",
        NumericIntrinsic::Sinh => "F32.sinh",
        NumericIntrinsic::Cosh => "F32.cosh",
        NumericIntrinsic::Tanh => "F32.tanh",
        NumericIntrinsic::Floor => "F32.floor",
        NumericIntrinsic::Ceil => "F32.ceil",
        NumericIntrinsic::Trunc => "F32.trunc",
        NumericIntrinsic::Show => "F32.show",
        NumericIntrinsic::Read => "F32.read",
    }
}

fn optimized(name: &str) -> bool {
    matches!(
        name,
        "U32.add"
            | "U32.sub"
            | "U32.mul"
            | "U32.div"
            | "U32.mod"
            | "U32.and"
            | "U32.or"
            | "U32.xor"
            | "U32.not"
            | "U32.inc"
            | "U32.shl"
            | "U32.shr"
            | "U32.shln"
            | "U32.shrn"
            | "U32.is_eq"
            | "U32.is_ne"
            | "U32.is_lt"
            | "U32.is_le"
            | "U32.is_gt"
            | "U32.is_ge"
            | "U32.is_zero"
            | "U32.cmp"
            | "U32.to_nat"
            | "U32.from_nat"
            | "Nat.add"
            | "Nat.sub"
            | "Nat.mul"
            | "Nat.double"
            | "Nat.cmp"
            | "Nat.is_lt"
            | "Nat.divmod"
            | "Bool.or"
            | "Bool.xor"
            | "String.append"
            | "Array.new"
            | "Array.set"
            | "Array.get"
            | "Array.swap"
            | "Array.size"
            | "Array.clone"
    )
}

fn native_owner<'a>(program: &ExecutableProgram, name: &'a str) -> &'a str {
    if program.base_names.contains(name)
        && matches!(
            name,
            "Nat" | "Bool" | "U32" | "F32" | "Char" | "String" | "Array"
        )
    {
        name
    } else {
        ""
    }
}

struct Generator<'a> {
    program: &'a ExecutableProgram,
    indices: BTreeMap<&'a str, usize>,
}

impl Generator<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "The complete typed IR traversal keeps quantity erasure and tail positions explicit."
    )]
    fn expression(&self, expression: &Expression, tail: bool) -> Result<String, CompileError> {
        Ok(match &expression.kind {
            ExpressionKind::Erased => "null".into(),
            ExpressionKind::Variable(id) => format!("v{id}"),
            ExpressionKind::Definition(name) => {
                let index = self.indices.get(name.as_str()).ok_or_else(|| {
                    CompileError::new(format!("missing reachable definition {name}"))
                })?;
                if tail {
                    format!("$tb.jump($tbDefinitions[{index}],[])")
                } else {
                    format!("$tb.run($tbDefinitions[{index}]())")
                }
            }
            ExpressionKind::Lambda { parameter, body } => {
                if parameter.quant == Quant::None {
                    self.expression(body, tail)?
                } else {
                    format!(
                        "$tb.closure(v{} => {})",
                        parameter.id,
                        self.expression(body, true)?
                    )
                }
            }
            ExpressionKind::Apply { .. } => {
                let mut arguments = Vec::new();
                let mut head = expression;
                while let ExpressionKind::Apply {
                    function,
                    argument,
                    quant,
                } = &head.kind
                {
                    if *quant != Quant::None {
                        arguments.push(self.expression(argument, false)?);
                    }
                    head = function;
                }
                arguments.reverse();
                if arguments.is_empty() {
                    self.expression(head, tail)?
                } else {
                    format!(
                        "$tb.apply({},[{}],{tail})",
                        self.expression(head, false)?,
                        arguments.join(",")
                    )
                }
            }
            ExpressionKind::Constructor {
                owner,
                name,
                fields,
            } => {
                let native = native_owner(self.program, owner);
                if matches!(native, "U32" | "F32")
                    && fields.len() == 1
                    && let Some(word) = self.word_literal(&fields[0].value)
                {
                    return Ok(if native == "U32" {
                        word.to_string()
                    } else {
                        format!("$tb.fromBits({word})")
                    });
                }
                let tag = self.tag(name)?;
                let live = fields
                    .iter()
                    .filter(|field| field.binder.quant != Quant::None)
                    .collect::<Vec<_>>();
                let names = live
                    .iter()
                    .map(|field| quote(&field.binder.name))
                    .collect::<Vec<_>>()
                    .join(",");
                let values = live
                    .iter()
                    .map(|field| self.expression(&field.value, false))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(",");
                format!(
                    "$tb.construct({},{},[{names}],[{values}])",
                    quote(native),
                    quote(tag)
                )
            }
            ExpressionKind::Match {
                parameter,
                owner,
                constructor,
                fields,
                arm,
                fallback,
            } => {
                if parameter.quant == Quant::None {
                    return Err(CompileError::new(format!(
                        "cannot generate a live match over an erased scrutinee at {}",
                        expression.ty
                    )));
                }
                let scrutinee = format!("v{}", parameter.id);
                let normalized = self
                    .program
                    .expose_type(&parameter.ty)
                    .map_err(|error| CompileError::new(error.to_string()))?;
                let excluded = match normalized.as_ref() {
                    Term::Adt { excluded, .. } => excluded.len(),
                    _ => 0,
                };
                let remaining = self.program.datatypes[owner]
                    .constructors
                    .len()
                    .saturating_sub(excluded);
                let native = native_owner(self.program, owner);
                let condition = if remaining == 1 {
                    "true".into()
                } else {
                    format!(
                        "$tb.matches({},{},{scrutinee})",
                        quote(native),
                        quote(self.tag(constructor)?)
                    )
                };
                let live_fields = fields
                    .iter()
                    .filter(|field| field.quant != Quant::None)
                    .enumerate()
                    .map(|(index, field)| {
                        format!(
                            "$tb.field({},{},{scrutinee},{index},{})",
                            quote(native),
                            quote(self.tag(constructor).expect("known constructor")),
                            quote(&field.name)
                        )
                    })
                    .collect::<Vec<_>>();
                let branch = if live_fields.is_empty() {
                    self.expression(arm, true)?
                } else {
                    format!(
                        "$tb.apply({},[{}],true)",
                        self.expression(arm, false)?,
                        live_fields.join(",")
                    )
                };
                let fallback = format!(
                    "$tb.apply({},[{scrutinee}],true)",
                    self.expression(fallback, false)?
                );
                format!(
                    "$tb.closure({scrutinee} => {{ if ($tbIsRequest({scrutinee})) throw {scrutinee}; if ({condition}) return {branch}; return {fallback}; }})"
                )
            }
            ExpressionKind::Absurd { parameter, owner } => {
                if !self.program.datatypes.contains_key(owner) {
                    return Err(CompileError::new(format!(
                        "missing impossible-branch datatype {owner}"
                    )));
                }
                if parameter.quant == Quant::None {
                    "(()=>{throw new Error('entered an impossible erased match');})()".into()
                } else {
                    format!(
                        "$tb.closure(v{} => {{ if ($tbIsRequest(v{})) throw v{}; throw new Error('entered an impossible match'); }})",
                        parameter.id, parameter.id, parameter.id
                    )
                }
            }
            ExpressionKind::Let { bindings, body } => {
                let mut result = String::from("(()=>{");
                for binding in bindings
                    .iter()
                    .filter(|binding| binding.binder.quant != Quant::None)
                {
                    write!(
                        result,
                        "const v{}={};",
                        binding.binder.id,
                        self.expression(&binding.value, false)?
                    )
                    .expect("writing to a string cannot fail");
                }
                write!(result, "return {};}})()", self.expression(body, tail)?)
                    .expect("writing to a string cannot fail");
                result
            }
        })
    }

    fn tag(&self, name: &str) -> Result<&str, CompileError> {
        self.program
            .constructor_tags
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| {
                CompileError::new(format!("constructor {name} has no loader-owned local tag"))
            })
    }

    /// Fold only literal bits from actual bundled Word/Bool constructors. User
    /// datatypes with these spellings retain their ordinary constructor objects.
    fn word_literal(&self, mut expression: &Expression) -> Option<u32> {
        let mut result = 0;
        for bit in 0..32 {
            let ExpressionKind::Constructor {
                owner,
                name,
                fields,
            } = &expression.kind
            else {
                return None;
            };
            if owner != "Word.Con"
                || name != "WCon"
                || fields.len() != 2
                || !self.program.base_names.contains(owner)
            {
                return None;
            }
            let ExpressionKind::Constructor {
                owner,
                name,
                fields: bool_fields,
            } = &fields[0].value.kind
            else {
                return None;
            };
            if native_owner(self.program, owner) != "Bool" || !bool_fields.is_empty() {
                return None;
            }
            match name.as_str() {
                "True" => result |= 1 << bit,
                "False" => (),
                _ => return None,
            }
            expression = &fields[1].value;
        }
        match &expression.kind {
            ExpressionKind::Constructor {
                owner,
                name,
                fields,
            } if owner == "Word.Nil"
                && name == "WNil"
                && fields.is_empty()
                && self.program.base_names.contains(owner) =>
            {
                Some(result)
            }
            _ => None,
        }
    }
}

struct Printer<'a> {
    program: &'a ExecutableProgram,
    nodes: Vec<String>,
    seen: BTreeMap<String, usize>,
}

impl Printer<'_> {
    fn node(&mut self, ty: &TermRef, depth: usize) -> Result<usize, CompileError> {
        if depth > 96 || self.nodes.len() >= 256 {
            return Err(CompileError::new("pure main printer type budget exhausted"));
        }
        let ty = self
            .program
            .expose_type(ty)
            .map_err(|error| CompileError::new(error.to_string()))?;
        let key = format!("{ty:?}");
        if let Some(index) = self.seen.get(&key) {
            return Ok(*index);
        }
        let index = self.nodes.len();
        self.seen.insert(key, index);
        self.nodes.push("null".into());
        let description = match ty.as_ref() {
            Term::Eql { .. } => "{kind:'proof'}".into(),
            Term::Adt { name, args, .. } => {
                let native = native_owner(self.program, name);
                if native == "Array" {
                    let element = self.node(&args[0], depth + 1)?;
                    format!("{{kind:'Array',element:{element}}}")
                } else if !native.is_empty() && native != "Bool" {
                    format!("{{kind:{}}}", quote(native))
                } else {
                    let datatype = self.program.datatypes[name].clone();
                    let mut variants = Vec::new();
                    for constructor in &datatype.constructors {
                        let mut fields = Vec::new();
                        for field in &constructor.fields {
                            if field.quant == Quant::None {
                                return Err(CompileError::new(
                                    "pure main with erased constructor fields cannot be printed",
                                ));
                            }
                            let mut ty = Rc::clone(&field.ty);
                            for (parameter, argument) in datatype.parameters.iter().zip(args) {
                                ty = substitute(&ty, parameter.id, argument);
                            }
                            let field_type = self.node(&ty, depth + 1)?;
                            fields.push(format!("[{},{}]", quote(&field.name), field_type));
                        }
                        let tag = self
                            .program
                            .constructor_tags
                            .get(&constructor.name)
                            .ok_or_else(|| {
                                CompileError::new("pure printer constructor has no local tag")
                            })?;
                        variants.push(format!(
                            "{{tag:{},fields:[{}]}}",
                            quote(tag),
                            fields.join(",")
                        ));
                    }
                    format!("{{kind:'data',variants:[{}]}}", variants.join(","))
                }
            }
            _ => {
                return Err(CompileError::new(format!(
                    "pure main type {ty} cannot be printed (function, type or dependent field)"
                )));
            }
        };
        self.nodes[index] = description;
        Ok(index)
    }
}

pub(super) fn quote(value: &str) -> String {
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
                write!(escaped, "\\u{:04x}", u32::from(character))
                    .expect("writing to a string cannot fail");
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}
