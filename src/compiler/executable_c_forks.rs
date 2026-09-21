// SPDX-License-Identifier: Apache-2.0
// Call classification derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Identify the calls which upstream keeps together in a parallel let.

use super::native;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::kernel::elaborate::Expression;
use crate::kernel::elaborate::ExpressionKind;
use std::rc::Rc;

/// Follow upstream `term_spine`: a saturated definition, an over-application,
/// or a live application of a variable is a call. Partial applications and
/// native intrinsics are values, even when their source uses application syntax.
///
/// This only selects a lowering strategy for already checked expressions. If
/// compiler-only type exposure cannot recover a definition's telescope, retain
/// its existing sequential lowering rather than inventing a call arity.
pub(super) fn is_call(program: &ExecutableProgram, expression: &Expression) -> bool {
    let mut head = expression;
    let mut supplied = 0;
    while let ExpressionKind::Apply {
        function, quant, ..
    } = &head.kind
    {
        supplied += usize::from(*quant != Quant::None);
        head = function;
    }
    match &head.kind {
        ExpressionKind::Variable(_) => supplied > 0,
        ExpressionKind::Definition(name) => {
            required_arguments(program, name).is_some_and(|required| supplied >= required)
        }
        _ => false,
    }
}

pub(super) fn required_arguments(program: &ExecutableProgram, name: &str) -> Option<usize> {
    let definition = program.definitions.get(name)?;
    // The loader's bundled-origin set is authoritative: an ordinary user
    // definition with the same short name is not an intrinsic.
    if program.base_names.contains(name) && native::optimized(name) {
        return None;
    }
    match &definition.body {
        DefinitionBody::Numeric(_) | DefinitionBody::OpaqueType(_) => None,
        DefinitionBody::Foreign(_) => Some(
            definition
                .parameters
                .iter()
                .filter(|parameter| parameter.quant != Quant::None)
                .count()
                + 1,
        ),
        DefinitionBody::Ordinary(body) => {
            let declared = definition.parameters.len();
            let extra = raised_arguments(body, i64::try_from(declared).ok()?)?;
            let limit = declared.checked_add(extra)?;
            let mut ty = Rc::clone(&definition.ty);
            let mut live = 0;
            for index in 0..limit {
                let exposed = program.expose_type(&ty).ok()?;
                let Term::All { quant, body, .. } = exposed.as_ref() else {
                    // Raising stops at the actual type telescope, including
                    // when an impossible match supplied the upstream sentinel.
                    return (index >= declared).then_some(live);
                };
                live += usize::from(*quant != Quant::None);
                ty = Rc::clone(body);
            }
            Some(live)
        }
    }
}

/// Upstream `def_raise` counts declared and returned binders before erasure.
/// A match arm replaces the scrutinee with its complete constructor telescope;
/// both branches must expose the same additional prefix. A leading let stops
/// raising, so a call which computes and then returns a closure remains a call.
pub(super) fn raised_arguments(expression: &Expression, remaining: i64) -> Option<usize> {
    match &expression.kind {
        ExpressionKind::Lambda { body, .. } => {
            if remaining > 0 {
                raised_arguments(body, remaining - 1)
            } else {
                raised_arguments(body, 0)?.checked_add(1)
            }
        }
        ExpressionKind::Match {
            fields,
            arm,
            fallback,
            ..
        } => {
            let fields = i64::try_from(fields.len()).ok()?;
            let arm_remaining = remaining.checked_sub(1)?.checked_add(fields)?;
            Some(raised_arguments(arm, arm_remaining)?.min(raised_arguments(fallback, remaining)?))
        }
        // This is the literal upstream sentinel; the type telescope caps it.
        ExpressionKind::Absurd { .. } => Some(99),
        _ => Some(0),
    }
}
