// SPDX-License-Identifier: MPL-2.0
//! Compact executable naturals. Strict proofs retain ordinary constructors.
//! Fast paths replace exact checked Base bodies, never user-defined lookalikes.

use super::Frame;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;
use super::packed::Wrapper;
use crate::kernel::AdtDecl;
use crate::kernel::Declaration;
use crate::kernel::DefDecl;
use crate::kernel::KernelError;
use crate::kernel::Term;
use crate::kernel::TermRef;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

/// Bend's native `NAT_IMM`, shared with its JS executable arithmetic bound.
pub(super) const MAX: u64 = (1 << 48) - 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Operation {
    Sub,
    Cmp,
    Show,
    FromU32,
    ToU32,
}

impl Operation {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Sub => "Nat.sub",
            Self::Cmp => "Nat.cmp",
            Self::Show => "Nat.show",
            Self::FromU32 => "U32.to_nat",
            Self::ToU32 => "U32.from_nat",
        }
    }

    const fn arity(self) -> usize {
        match self {
            Self::Sub | Self::Cmp => 2,
            Self::Show | Self::FromU32 | Self::ToU32 => 1,
        }
    }
}

thread_local! {
    static BASE: (BTreeMap<String, DefDecl>, BTreeMap<String, AdtDecl>) = {
        let book = crate::syntax::parse(include_str!("../syntax/base.bend"))
            .expect("bundled checked Base parses");
        let mut definitions = BTreeMap::new();
        let mut datatypes = BTreeMap::new();
        for declaration in book.declarations {
            match declaration {
                Declaration::Def(definition) => {
                    definitions.insert(definition.name.clone(), definition);
                }
                Declaration::Adt(datatype) => {
                    datatypes.insert(datatype.name.clone(), datatype);
                }
            }
        }
        (definitions, datatypes)
    };
}

/// Loader origin is necessary but not sufficient: also compare the checked
/// body, type, and every transitively referenced Base definition/type, modulo
/// binder identities. No host fast path can bypass a modified helper.
pub(super) fn checked_optimizations(
    definitions: &BTreeMap<String, DefDecl>,
    datatypes: &BTreeMap<String, AdtDecl>,
    origins: &BTreeSet<String>,
) -> BTreeSet<Operation> {
    if !super::packed::checked_layouts(datatypes, origins) {
        return BTreeSet::new();
    }
    BASE.with(|(base_definitions, base_datatypes)| {
        [
            Operation::Sub,
            Operation::Cmp,
            Operation::Show,
            Operation::FromU32,
            Operation::ToU32,
        ]
        .into_iter()
        .filter(|operation| {
            let mut pending = vec![operation.name().to_owned()];
            let mut checked = BTreeSet::new();
            while let Some(name) = pending.pop() {
                if !checked.insert(name.clone()) {
                    continue;
                }
                if !origins.contains(&name) {
                    return false;
                }
                let mut shape = Shape::default();
                let matches = if let Some(expected) = base_definitions.get(&name) {
                    definitions
                        .get(&name)
                        .is_some_and(|actual| shape.definition(actual, expected))
                } else if let Some(expected) = base_datatypes.get(&name) {
                    datatypes
                        .get(&name)
                        .is_some_and(|actual| shape.datatype(actual, expected))
                } else {
                    false
                };
                if !matches {
                    return false;
                }
                pending.extend(shape.references);
            }
            true
        })
        .collect()
    })
}

#[derive(Default)]
struct Shape {
    bindings: Vec<(usize, usize)>,
    references: BTreeSet<String>,
}

impl Shape {
    fn definition(&mut self, actual: &DefDecl, expected: &DefDecl) -> bool {
        if actual.name != expected.name
            || actual.foreign
            || actual.unsafe_
            || actual.parameters.len() != expected.parameters.len()
        {
            return false;
        }
        for (a, b) in actual.parameters.iter().zip(&expected.parameters) {
            if a.quant != b.quant || !self.term(&a.ty, &b.ty) {
                return false;
            }
            self.bindings.push((a.id, b.id));
        }
        self.bindings.clear();
        self.term(&actual.ty, &expected.ty)
            && match (&actual.body, &expected.body) {
                (Some(a), Some(b)) => self.term(a, b),
                _ => false,
            }
    }

    fn datatype(&mut self, actual: &AdtDecl, expected: &AdtDecl) -> bool {
        if actual.name != expected.name
            || actual.parameters.len() != expected.parameters.len()
            || actual.constructors.len() != expected.constructors.len()
        {
            return false;
        }
        for (a, b) in actual.parameters.iter().zip(&expected.parameters) {
            if a.quant != b.quant || !self.term(&a.ty, &b.ty) {
                return false;
            }
            self.bindings.push((a.id, b.id));
        }
        if !self.term(&actual.kind, &expected.kind) {
            return false;
        }
        for (a, b) in actual.constructors.iter().zip(&expected.constructors) {
            if a.name != b.name || a.fields.len() != b.fields.len() {
                return false;
            }
            self.bindings.truncate(actual.parameters.len());
            for (a, b) in a.fields.iter().zip(&b.fields) {
                if a.quant != b.quant || !self.term(&a.ty, &b.ty) {
                    return false;
                }
                self.bindings.push((a.id, b.id));
            }
        }
        true
    }

    fn terms(&mut self, actual: &[TermRef], expected: &[TermRef]) -> bool {
        actual.len() == expected.len() && actual.iter().zip(expected).all(|(a, b)| self.term(a, b))
    }

    // These are syntax comparisons, not normalization or kernel equivalence.
    // New or unsupported syntax disables an optimization instead of guessing.
    #[expect(
        clippy::too_many_lines,
        reason = "one structural comparison makes binding scopes and dependencies auditable"
    )]
    fn term(&mut self, actual: &TermRef, expected: &TermRef) -> bool {
        match (actual.as_ref(), expected.as_ref()) {
            (
                Term::Var { id: actual_id, .. },
                Term::Var {
                    id: expected_id, ..
                },
            ) => self
                .bindings
                .iter()
                .rev()
                .find(|(id, _)| id == actual_id)
                .is_some_and(|(_, id)| id == expected_id),
            (Term::Ref(actual_name), Term::Ref(expected_name)) => {
                self.references.insert(expected_name.clone());
                actual_name == expected_name
            }
            (Term::Typ(actual_kind), Term::Typ(expected_kind)) => {
                self.term(actual_kind, expected_kind)
            }
            (Term::Qua(actual_quant), Term::Qua(expected_quant)) => actual_quant == expected_quant,
            (Term::Qnt, Term::Qnt) | (Term::Efq, Term::Efq) | (Term::Rfl, Term::Rfl) => true,
            (
                Term::App(actual_first, actual_second),
                Term::App(expected_first, expected_second),
            )
            | (
                Term::Ann(actual_first, actual_second),
                Term::Ann(expected_first, expected_second),
            )
            | (
                Term::Min(actual_first, actual_second),
                Term::Min(expected_first, expected_second),
            ) => {
                self.term(actual_first, expected_first) && self.term(actual_second, expected_second)
            }
            (
                Term::Ctr {
                    name: actual_name,
                    args: actual_args,
                },
                Term::Ctr {
                    name: expected_name,
                    args: expected_args,
                },
            ) => actual_name == expected_name && self.terms(actual_args, expected_args),
            (
                Term::Adt {
                    name: actual_name,
                    args: actual_args,
                    excluded: actual_excluded,
                },
                Term::Adt {
                    name: expected_name,
                    args: expected_args,
                    excluded: expected_excluded,
                },
            ) => {
                self.references.insert(expected_name.clone());
                actual_name == expected_name
                    && actual_excluded == expected_excluded
                    && self.terms(actual_args, expected_args)
            }
            (
                Term::All {
                    quant: actual_quant,
                    id: actual_id,
                    domain: actual_domain,
                    body: actual_body,
                    ..
                },
                Term::All {
                    quant: expected_quant,
                    id: expected_id,
                    domain: expected_domain,
                    body: expected_body,
                    ..
                },
            ) => {
                if actual_quant != expected_quant || !self.term(actual_domain, expected_domain) {
                    return false;
                }
                self.bindings.push((*actual_id, *expected_id));
                let result = self.term(actual_body, expected_body);
                self.bindings.pop();
                result
            }
            (
                Term::Lam {
                    id: actual_id,
                    body: actual_body,
                    ..
                },
                Term::Lam {
                    id: expected_id,
                    body: expected_body,
                    ..
                },
            ) => {
                self.bindings.push((*actual_id, *expected_id));
                let result = self.term(actual_body, expected_body);
                self.bindings.pop();
                result
            }
            (
                Term::Mat {
                    constructor: actual_constructor,
                    arm: actual_arm,
                    fallback: actual_fallback,
                },
                Term::Mat {
                    constructor: expected_constructor,
                    arm: expected_arm,
                    fallback: expected_fallback,
                },
            ) => {
                actual_constructor == expected_constructor
                    && self.term(actual_arm, expected_arm)
                    && self.term(actual_fallback, expected_fallback)
            }
            (
                Term::Let {
                    bindings: actual_bindings,
                    body: actual_body,
                },
                Term::Let {
                    bindings: expected_bindings,
                    body: expected_body,
                },
            ) => {
                if actual_bindings.len() != expected_bindings.len()
                    || !actual_bindings
                        .iter()
                        .zip(expected_bindings)
                        .all(|(actual, expected)| {
                            actual.quant == expected.quant
                                && self.term(&actual.value, &expected.value)
                        })
                {
                    return false;
                }
                let start = self.bindings.len();
                self.bindings.extend(
                    actual_bindings
                        .iter()
                        .zip(expected_bindings)
                        .map(|(actual, expected)| (actual.id, expected.id)),
                );
                let result = self.term(actual_body, expected_body);
                self.bindings.truncate(start);
                result
            }
            (
                Term::Eql {
                    left: actual_first,
                    right: actual_second,
                    ty: actual_third,
                },
                Term::Eql {
                    left: expected_first,
                    right: expected_second,
                    ty: expected_third,
                },
            )
            | (
                Term::Rwt {
                    evidence: actual_first,
                    motive: actual_second,
                    body: actual_third,
                },
                Term::Rwt {
                    evidence: expected_first,
                    motive: expected_second,
                    body: expected_third,
                },
            ) => {
                self.term(actual_first, expected_first)
                    && self.term(actual_second, expected_second)
                    && self.term(actual_third, expected_third)
            }
            _ => false,
        }
    }
}

/// Read only complete literal syntax, and bound the speculative scan. No
/// variable, reference or source field is evaluated to make a compact value.
pub(super) fn literal(name: &str, fields: &[TermRef]) -> Option<Value> {
    let mut name = name;
    let mut fields = fields;
    let mut value = 0;
    for _ in 0..=super::FRAME_LIMIT {
        match (name, fields) {
            ("Zero", []) => return Some(Value::PackedNat(value)),
            ("Succ", [tail]) => {
                let Term::Ctr { name: next, args } = tail.as_ref() else {
                    return None;
                };
                name = next;
                fields = args;
                value += 1;
            }
            _ => return None,
        }
    }
    None
}

pub(super) struct Evaluation {
    operation: Operation,
    arguments: Vec<ThunkId>,
    values: Vec<u64>,
    prefix: u64,
}

impl Evaluation {
    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        for argument in &self.arguments {
            visit(*argument)?;
        }
        Ok(())
    }
}

impl Machine<'_> {
    pub(super) fn nat(&mut self, value: u64) -> Result<ThunkId, KernelError> {
        if !self.program.packed {
            return Err(KernelError::new("native Nat requires sealed Base layouts"));
        }
        if value > MAX {
            return Err(KernelError::new("native Nat exceeds the 48-bit bound"));
        }
        self.allocate(Thunk::Ready(Value::PackedNat(value)))
    }

    pub(super) fn apply_nat(
        &mut self,
        operation: Operation,
        mut arguments: Vec<ThunkId>,
        argument: ThunkId,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        arguments.push(argument);
        if arguments.len() < operation.arity() {
            return self.allocate(Thunk::Ready(Value::Natural {
                operation,
                arguments,
            }));
        }
        if arguments.len() != operation.arity() {
            return Err(KernelError::new("native Nat operation has invalid arity"));
        }
        let first = arguments[0];
        Self::push(
            frames,
            Frame::Natural(Evaluation {
                operation,
                arguments,
                values: vec![],
                prefix: 0,
            }),
        )?;
        Ok(first)
    }

    pub(super) fn nat_step(
        &mut self,
        mut state: Evaluation,
        value: Value,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        let value = match (state.operation, value) {
            (
                Operation::FromU32,
                Value::PackedWord {
                    wrapper: Wrapper::U32,
                    bits,
                },
            ) => u64::from(bits),
            (operation, Value::PackedNat(value)) if operation != Operation::FromU32 => value
                .checked_add(state.prefix)
                .filter(|value| *value <= MAX)
                .ok_or_else(|| KernelError::new("native Nat exceeds the 48-bit bound"))?,
            // Both bodies consume the complete natural before yielding their
            // result. Decode a reconstructed unary prefix on the bounded
            // machine stack, stopping as soon as its compact tail is found.
            (Operation::Show | Operation::ToU32, Value::Constructor { name, fields })
                if name == "Zero" && fields.is_empty() =>
            {
                state.prefix
            }
            (Operation::Show | Operation::ToU32, Value::Constructor { name, fields })
                if name == "Succ" && fields.len() == 1 =>
            {
                state.prefix = state
                    .prefix
                    .checked_add(1)
                    .filter(|value| *value <= MAX)
                    .ok_or_else(|| KernelError::new("native Nat exceeds the 48-bit bound"))?;
                let next = fields[0];
                Self::push(frames, Frame::Natural(state))?;
                return Ok(next);
            }
            _ => return self.ordinary_nat(state.operation, &state.arguments),
        };
        state.values.push(value);
        if state.values.len() < state.arguments.len() {
            let next = state.arguments[state.values.len()];
            Self::push(frames, Frame::Natural(state))?;
            return Ok(next);
        }
        match state.operation {
            Operation::Sub => self.nat(state.values[0].saturating_sub(state.values[1])),
            Operation::Cmp => self.ready_constructor(
                match state.values[0].cmp(&state.values[1]) {
                    std::cmp::Ordering::Less => "LT",
                    std::cmp::Ordering::Equal => "EQ",
                    std::cmp::Ordering::Greater => "GT",
                },
                vec![],
            ),
            Operation::FromU32 => self.nat(value),
            Operation::ToU32 => self.numeric_word_value(
                "U32",
                u32::try_from(value & u64::from(u32::MAX)).expect("masked to U32"),
            ),
            Operation::Show => {
                let mut result = self.ready_constructor("SNil", vec![])?;
                for byte in value.to_string().bytes().rev() {
                    self.tick()?;
                    let code = self.numeric_word_value("U32", u32::from(byte))?;
                    let character = self.ready_constructor("Chr", vec![code])?;
                    result = self.ready_constructor("SCon", vec![character, result])?;
                }
                Ok(result)
            }
        }
    }

    fn ordinary_nat(
        &mut self,
        operation: Operation,
        arguments: &[ThunkId],
    ) -> Result<ThunkId, KernelError> {
        let body = self
            .program
            .definitions
            .get(operation.name())
            .and_then(|definition| definition.body.as_ref())
            .map(Rc::clone)
            .ok_or_else(|| KernelError::new("native Nat optimization has no checked body"))?;
        let mut function = self.expression(body, 0)?;
        for argument in arguments {
            function = self.allocate(Thunk::Application(function, *argument))?;
        }
        Ok(function)
    }
}

#[cfg(test)]
#[path = "nat_tests.rs"]
mod tests;
