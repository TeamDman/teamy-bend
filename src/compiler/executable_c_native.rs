// SPDX-License-Identifier: Apache-2.0
// Numeric operations derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.

use super::Body;
use super::CompileError;
use super::DefinitionBody;
use super::ExecutableProgram;
use super::Expression;
use super::ExpressionKind;
use super::Generator;
use super::Quant;
use super::Scope;
use super::TermRef;
use super::specialize_type;
use crate::syntax::executable::NumericIntrinsic;

pub(super) fn optimized(name: &str) -> bool {
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
            | "Array.new"
            | "Array.set"
            | "Array.get"
            | "Array.swap"
            | "Array.size"
            | "Array.clone"
    )
}

enum DirectCall {
    Native,
    Numeric(NumericIntrinsic),
}

/// Decide before consuming any operand: a partial application still creates a
/// callable, and an unrelated definition with a builtin's name is ordinary code.
fn direct_call(
    program: &ExecutableProgram,
    name: &str,
    arguments: &[(&Expression, Quant)],
) -> Option<DirectCall> {
    let definition = program.definitions.get(name)?;
    if !program.base_names.contains(name)
        || definition.parameters.len() != arguments.len()
        || definition
            .parameters
            .iter()
            .zip(arguments)
            .any(|(parameter, (_, quant))| parameter.quant != *quant)
    {
        return None;
    }
    match &definition.body {
        DefinitionBody::Numeric(intrinsic)
            if arguments.iter().filter(|(_, q)| *q != Quant::None).count() == intrinsic.arity() =>
        {
            Some(DirectCall::Numeric(*intrinsic))
        }
        DefinitionBody::Ordinary(_) if optimized(name) => Some(DirectCall::Native),
        _ => None,
    }
}

impl Generator<'_> {
    /// A direct primitive produces its value immediately, including in tail
    /// position. Other applications may return task control to the dispatcher.
    pub(super) fn direct_native_application(&self, expression: &Expression) -> bool {
        let mut head = expression;
        let mut arguments = Vec::new();
        while let ExpressionKind::Apply {
            function,
            argument,
            quant,
        } = &head.kind
        {
            arguments.push((argument.as_ref(), *quant));
            head = function;
        }
        let ExpressionKind::Definition(name) = &head.kind else {
            return false;
        };
        arguments.reverse();
        direct_call(self.program, name, &arguments).is_some()
    }

    /// Upstream `emit_intr` evaluates its live operands in order before emitting
    /// the primitive. Keep that order and the existing helpers' representations,
    /// ownership, Array layout checks and numeric edge behavior.
    pub(super) fn direct_native(
        &mut self,
        name: &str,
        arguments: &[(&Expression, Quant)],
        scope: &mut Scope,
        output: &mut Body,
    ) -> Result<Option<String>, CompileError> {
        let Some(call) = direct_call(self.program, name, arguments) else {
            return Ok(None);
        };
        let mut values = Vec::new();
        for (argument, quant) in arguments {
            if *quant != Quant::None {
                let value = self.expression(argument, scope, output, false)?;
                values.push(self.hold(output, &value)?);
            }
        }
        // Removing closure dispatch must retain an aggregate step-budget and
        // cancellation boundary for each executed primitive.
        output.push_str("  tb_tick();\n");
        let value = match call {
            DirectCall::Native => {
                let substitutions = self.instantiation(name, arguments);
                let ty = specialize_type(&self.program.definitions[name].ty, &substitutions);
                self.native(name, &ty, &values, output)?
            }
            DirectCall::Numeric(intrinsic) => self.numeric(intrinsic, &values, output)?,
        };
        Ok(Some(value))
    }

    pub(super) fn native(
        &mut self,
        name: &str,
        ty: &TermRef,
        arguments: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        if name.starts_with("Array.") {
            return self.array_operation(name, ty, arguments, output);
        }
        let a = &arguments[0];
        let b = arguments.get(1).map_or("0", String::as_str);
        let expression = match name {
            "U32.add" | "U32.sub" | "U32.mul" | "U32.and" | "U32.or" | "U32.xor" => {
                let operator = match name {
                    "U32.add" => "+",
                    "U32.sub" => "-",
                    "U32.mul" => "*",
                    "U32.and" => "&",
                    "U32.or" => "|",
                    _ => "^",
                };
                format!("(u32)((u32){a} {operator} (u32){b})")
            }
            "U32.div" => format!("((u32){b} == 0 ? 0 : (u32){a} / (u32){b})"),
            "U32.mod" => format!("((u32){b} == 0 ? (u32){a} : (u32){a} % (u32){b})"),
            "U32.not" => format!("(u32)~(u32){a}"),
            "U32.inc" => format!("(u32)((u32){a} + 1u)"),
            "U32.shl" => format!("(u32)((u32){a} << 1)"),
            "U32.shr" => format!("((u32){a} >> 1)"),
            "U32.shln" => format!("({b} >= 32 ? 0 : (u32)((u32){a} << (u32){b}))"),
            "U32.shrn" => format!("({b} >= 32 ? 0 : (u32){a} >> (u32){b})"),
            "U32.is_eq" | "U32.is_ne" | "U32.is_lt" | "U32.is_le" | "U32.is_gt" | "U32.is_ge" => {
                let operator = match name {
                    "U32.is_eq" => "==",
                    "U32.is_ne" => "!=",
                    "U32.is_lt" => "<",
                    "U32.is_le" => "<=",
                    "U32.is_gt" => ">",
                    _ => ">=",
                };
                format!("term_pak((u32){a} {operator} (u32){b} ? CID_TRUE : CID_FALSE, 0)")
            }
            "U32.is_zero" => format!("term_pak((u32){a} == 0 ? CID_TRUE : CID_FALSE, 0)"),
            "U32.cmp" | "Nat.cmp" => {
                format!("term_pak({a} < {b} ? CID_LT : {a} == {b} ? CID_EQ : CID_GT, 0)")
            }
            "U32.to_nat" | "U32.from_nat" => format!("(u32){a}"),
            "Nat.add" => format!("tb_c_nat_chk(e, {a} + {b})"),
            "Nat.sub" => format!("({a} < {b} ? 0 : {a} - {b})"),
            "Nat.mul" => format!("tb_c_nat_mul(e, {a}, {b})"),
            "Nat.double" => format!("tb_c_nat_chk(e, {a} + {a})"),
            "Nat.is_lt" => format!("term_pak({a} < {b} ? CID_TRUE : CID_FALSE, 0)"),
            "Nat.divmod" => {
                return self.construct(
                    "Tuple",
                    &[
                        format!("({b} == 0 ? 0 : {a} / {b})"),
                        format!("({b} == 0 ? {a} : {a} % {b})"),
                    ],
                    output,
                );
            }
            "Bool.or" | "Bool.xor" => {
                let operator = if name == "Bool.or" { "||" } else { "!=" };
                format!(
                    "term_pak(((term_aux({a}) == CID_TRUE) {operator} (term_aux({b}) == CID_TRUE)) ? CID_TRUE : CID_FALSE, 0)"
                )
            }
            _ => {
                return Err(CompileError::new(format!(
                    "unknown native C operation {name}"
                )));
            }
        };
        self.hold(output, &expression)
    }

    pub(super) fn numeric(
        &mut self,
        intrinsic: NumericIntrinsic,
        arguments: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        use NumericIntrinsic as N;
        if matches!(intrinsic, N::Show | N::Read) {
            output.dependencies.host_operations.insert(
                if intrinsic == N::Show {
                    "F32.show"
                } else {
                    "F32.read"
                }
                .to_owned(),
            );
        }
        let a = &arguments[0];
        let b = arguments.get(1).map_or("0", String::as_str);
        let left = format!("f32_unbox({a})");
        let right = format!("f32_unbox({b})");
        let expression = match intrinsic {
            N::U32ToF32 => format!("f32_rewrap((f32)(u32){a})"),
            N::F32ToU32 => format!("({left} >= 0.0f && {left} < 4294967296.0f ? (u32){left} : 0)"),
            N::Add | N::Sub | N::Mul | N::Div => {
                let operator = match intrinsic {
                    N::Add => "+",
                    N::Sub => "-",
                    N::Mul => "*",
                    _ => "/",
                };
                format!("f32_rewrap({left} {operator} {right})")
            }
            N::Mod => format!("f32_rewrap((f32)fmod({left}, {right}))"),
            N::Pow | N::Atan2 => format!(
                "f32_rewrap((f32){}({left}, {right}))",
                if intrinsic == N::Pow { "pow" } else { "atan2" }
            ),
            N::Neg => format!("f32_rewrap(-{left})"),
            N::Bits => format!("(u32){a}"),
            N::Show => format!("tb_c_f32_show(e, {a})"),
            N::Read => format!("tb_c_f32_read(e, {a})"),
            N::IsEq | N::IsNe | N::IsLt | N::IsLe | N::IsGt | N::IsGe => {
                let operator = match intrinsic {
                    N::IsEq => "==",
                    N::IsNe => "!=",
                    N::IsLt => "<",
                    N::IsLe => "<=",
                    N::IsGt => ">",
                    _ => ">=",
                };
                format!("term_pak({left} {operator} {right} ? CID_TRUE : CID_FALSE, 0)")
            }
            _ => {
                let function = match intrinsic {
                    N::Abs => "fabs",
                    N::Sqrt => "sqrt",
                    N::Exp => "exp",
                    N::Log => "log",
                    N::Log2 => "log2",
                    N::Log10 => "log10",
                    N::Sin => "sin",
                    N::Cos => "cos",
                    N::Tan => "tan",
                    N::Asin => "asin",
                    N::Acos => "acos",
                    N::Atan => "atan",
                    N::Sinh => "sinh",
                    N::Cosh => "cosh",
                    N::Tanh => "tanh",
                    N::Floor => "floor",
                    N::Ceil => "ceil",
                    N::Trunc => "trunc",
                    _ => return Err(CompileError::new("unknown executable C numeric operation")),
                };
                format!("f32_rewrap((f32){function}({left}))")
            }
        };
        self.hold(output, &expression)
    }
}
