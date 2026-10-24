// SPDX-License-Identifier: MPL-2.0
//! Executable erasure and by-value construction, separate from proof-data evaluation.

use super::Frame;
use super::Machine;
use super::Program;
use super::Thunk;
use super::ThunkId;
use super::Value;
use crate::kernel::KernelError;
use crate::kernel::LetBinding;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::kernel::elaborate::Expression;
use crate::kernel::elaborate::ExpressionKind;
use crate::kernel::term;
use std::rc::Rc;

impl Program {
    /// The shared checked lowering removes erased computations before native
    /// argument demand. Keep their slots: native IO adapters use source arities.
    /// Optimization identities were checked against the original sealed bodies.
    pub(crate) fn with_executable_demand(&self, lowered: &ExecutableProgram) -> Self {
        let mut definitions = self.definitions.as_ref().clone();
        for (name, definition) in &lowered.definitions {
            if let DefinitionBody::Ordinary(body) = &definition.body {
                definitions
                    .get_mut(name)
                    .expect("lowering retains checked definitions")
                    .body = Some(executable_term(body));
            }
        }
        Self {
            definitions: Rc::new(definitions),
            strict: true,
            ..self.clone()
        }
    }
}

fn executable_term(expression: &Expression) -> TermRef {
    term(match &expression.kind {
        ExpressionKind::Erased => Term::Rfl,
        ExpressionKind::Variable(id) => Term::Var {
            name: String::new(),
            id: *id,
        },
        ExpressionKind::Definition(name) => Term::Ref(name.clone()),
        ExpressionKind::Lambda { parameter, body } => Term::Lam {
            name: parameter.name.clone(),
            id: parameter.id,
            body: executable_term(body),
        },
        ExpressionKind::Apply {
            function, argument, ..
        } => Term::App(executable_term(function), executable_term(argument)),
        ExpressionKind::Constructor { name, fields, .. } => Term::Ctr {
            name: name.clone(),
            args: fields
                .iter()
                .map(|field| executable_term(&field.value))
                .collect(),
        },
        ExpressionKind::Match {
            constructor,
            arm,
            fallback,
            ..
        } => Term::Mat {
            constructor: constructor.clone(),
            arm: executable_term(arm),
            fallback: executable_term(fallback),
        },
        ExpressionKind::Absurd { .. } => Term::Efq,
        ExpressionKind::Let { bindings, body } => Term::Let {
            bindings: bindings
                .iter()
                .map(|field| LetBinding {
                    quant: field.binder.quant,
                    name: field.binder.name.clone(),
                    id: field.binder.id,
                    value: executable_term(&field.value),
                })
                .collect(),
            body: executable_term(body),
        },
    })
}

impl Machine<'_> {
    /// Tail-modulo-constructor evaluation: previous fields are already values,
    /// and only the last field remains. Store the unfinished result in its
    /// existing arena slot instead of one continuation per list element. It
    /// remains a black hole until completion, so no partial value can escape.
    /// Non-tail computation still uses `FRAME_LIMIT`; result storage still uses
    /// `ARENA_LIMIT`, and every completion charges the unchanged work budget.
    pub(super) fn compress_constructor_tail(
        &mut self,
        frames: &mut Vec<Frame>,
    ) -> Result<(), KernelError> {
        let length = frames.len();
        if length < 3
            || !matches!(frames[length - 1], Frame::Update(_))
            || !matches!(&frames[length - 2], Frame::Constructor { fields, next, .. } if *next == fields.len())
            || !matches!(frames[length - 3], Frame::Update(_))
        {
            return Ok(());
        }
        self.tick()?;
        let child = frames.pop().expect("inspected update frame");
        let Some(Frame::Constructor { name, fields, .. }) = frames.pop() else {
            unreachable!()
        };
        let Some(Frame::Update(root)) = frames.pop() else {
            unreachable!()
        };
        let parent = matches!(frames.last(), Some(Frame::CompleteConstructorChain(_))).then(|| {
            let Some(Frame::CompleteConstructorChain(parent)) = frames.pop() else {
                unreachable!()
            };
            parent
        });
        self.arena[root] = Some(Thunk::PendingConstructor {
            name,
            fields,
            parent,
        });
        Self::push(frames, Frame::CompleteConstructorChain(root))?;
        Self::push(frames, child)
    }

    pub(super) fn complete_constructor_chain(
        &mut self,
        mut head: ThunkId,
        mut value: Value,
    ) -> Result<Value, KernelError> {
        loop {
            self.tick()?;
            let Some(Thunk::PendingConstructor {
                name,
                fields,
                parent,
            }) = self.arena[head].take()
            else {
                return Err(KernelError::new("invalid pending constructor chain"));
            };
            value = self.strict_constructor(name, fields, &value)?;
            self.arena[head] = Some(Thunk::Ready(value.clone()));
            let Some(parent) = parent else {
                return Ok(value);
            };
            head = parent;
        }
    }

    /// All live fields have completed left-to-right on the bounded machine
    /// stack. Seal ordinary successors too, before a match can observe a head.
    pub(super) fn strict_constructor(
        &self,
        name: String,
        fields: Vec<ThunkId>,
        last: &Value,
    ) -> Result<Value, KernelError> {
        if self.program.packed && name == "Succ" && fields.len() == 1 {
            let Value::PackedNat(tail) = last else {
                return Err(KernelError::new("native successor has a non-Nat field"));
            };
            let value = tail
                .checked_add(1)
                .filter(|value| *value <= super::nat::MAX)
                .ok_or_else(|| KernelError::nat_overflow("native Nat exceeds the 48-bit bound"))?;
            return Ok(Value::PackedNat(value));
        }
        Ok(Value::Constructor { name, fields })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::DefDecl;
    use crate::runtime::gc;
    use crate::syntax::parse_term;
    use std::collections::BTreeMap;

    #[test]
    fn strict_constructor_recursion_keeps_frame_work_and_cancellation_limits() {
        let body = parse_term("n => Box{repeat(n), n}").unwrap();
        let definition = DefDecl {
            completion_parameters: None,
            name: "repeat".into(),
            parameters: vec![],
            ty: term(Term::Rfl),
            body: Some(body),
            unsafe_: true,
            foreign: false,
        };
        let mut program = Program::from_checked(
            &Rc::new(BTreeMap::from([("repeat".into(), definition)])),
            &Rc::new(BTreeMap::new()),
        );
        program.strict = true;
        for (budget, cancelled, expected) in [
            (super::super::STEPS, false, "continuation depth exhausted"),
            (100, false, "step budget exhausted"),
            (super::super::STEPS, true, "execution cancelled"),
        ] {
            let mut machine = Machine::new(&program);
            machine.gc_mode = gc::Mode::Disabled;
            let function = machine.reference("repeat").unwrap();
            let argument = machine.allocate(Thunk::Ready(Value::Erased)).unwrap();
            let entry = machine
                .allocate(Thunk::Application(function, argument))
                .unwrap();
            machine.remaining = budget;
            let cancellation = || cancelled;
            machine.cancelled = Some(&cancellation);
            let error = machine
                .force(entry)
                .err()
                .expect("bounded strict evaluation refuses recursion");
            assert!(error.to_string().contains(expected), "{error}");
            assert!(machine.arena.len() < super::super::ARENA_LIMIT);
        }
    }
}
