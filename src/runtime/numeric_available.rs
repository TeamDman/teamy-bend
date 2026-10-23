// SPDX-License-Identifier: MPL-2.0
//! Propagate already available U32 values without demanding suspended arguments.

use super::EnvId;
use super::Machine;
use super::Thunk;
use super::Value;
use super::packed;
use super::packed::Wrapper;
use crate::kernel::KernelError;
use crate::kernel::Term;
use crate::kernel::TermRef;
use std::rc::Rc;

// An unsuccessful inspection must stay cheap, even for a cyclic/long alias or
// environment graph. This is an optimization cutoff, not an evaluation limit.
const INSPECTION_LIMIT: usize = 32;

impl Machine<'_> {
    // A bound variable already denotes its target thunk. Sharing that identity
    // avoids retaining another environment without evaluating the target.
    pub(super) fn available_binding(
        &mut self,
        mut environment: EnvId,
        binder: usize,
    ) -> Result<Option<super::ThunkId>, KernelError> {
        for _ in 0..INSPECTION_LIMIT {
            self.tick()?;
            let Some(scope) = self.environments.get(environment).and_then(Option::as_ref) else {
                return Ok(None);
            };
            if let Some((_, target)) = scope.bindings.iter().find(|(id, _)| *id == binder) {
                return Ok(Some(*target));
            }
            let Some(parent) = scope.parent else {
                return Ok(None);
            };
            environment = parent;
        }
        Ok(None)
    }

    pub(super) fn available_numeric(
        &mut self,
        expression: &TermRef,
        environment: EnvId,
    ) -> Result<Option<u32>, KernelError> {
        if !self.program.packed {
            return Ok(None);
        }
        if matches!(expression.as_ref(), Term::Ctr { name, .. } if name == "U32") {
            return self.available_word(expression, environment);
        }
        let Term::App(function, last) = expression.as_ref() else {
            return Ok(None);
        };
        let (head, arguments): (_, &[_]) = match function.as_ref() {
            Term::App(head, first) => (head, &[first, last]),
            _ => (function, &[last]),
        };
        let (Term::Ref(name) | Term::GpuRef(name)) = head.as_ref() else {
            return Ok(None);
        };
        let Some(operation) = self
            .program
            .optimizations
            .iter()
            .find(|operation| operation.name() == name && operation.arity() == arguments.len())
            .copied()
        else {
            return Ok(None);
        };
        let mut words = [0; 2];
        for (index, argument) in arguments.iter().enumerate() {
            let Some(bits) = self.available_word(argument, environment)? else {
                return Ok(None);
            };
            words[index] = bits;
        }
        Ok(Some(operation.result(&words)))
    }

    fn available_word(
        &mut self,
        expression: &TermRef,
        mut environment: EnvId,
    ) -> Result<Option<u32>, KernelError> {
        let mut expression = Rc::clone(expression);
        let mut thunk = None;
        let mut binder = None;
        for _ in 0..INSPECTION_LIMIT {
            self.tick()?;
            if let Some(id) = binder {
                let Some(scope) = self.environments.get(environment).and_then(Option::as_ref)
                else {
                    return Ok(None);
                };
                if let Some((_, target)) = scope.bindings.iter().find(|(name, _)| *name == id) {
                    thunk = Some(*target);
                    binder = None;
                } else if let Some(parent) = scope.parent {
                    environment = parent;
                    continue;
                } else {
                    return Ok(None);
                }
            }
            if let Some(id) = thunk.take() {
                match self.arena.get(id).and_then(Option::as_ref) {
                    Some(Thunk::Ready(Value::PackedWord {
                        wrapper: Wrapper::U32,
                        bits,
                    })) => {
                        return Ok(Some(*bits));
                    }
                    Some(Thunk::Alias(target)) => {
                        thunk = Some(*target);
                        continue;
                    }
                    Some(Thunk::Expression(value, scope)) => {
                        expression = Rc::clone(value);
                        environment = *scope;
                    }
                    // No forcing, unfolding, constructor-field decoding or
                    // mutation: retain the original lazy path on uncertainty.
                    _ => return Ok(None),
                }
            }
            match expression.as_ref() {
                Term::Var { id, .. } => binder = Some(*id),
                Term::Ann(body, _) | Term::Rwt { body, .. } => expression = Rc::clone(body),
                Term::Ctr { name, args } => {
                    return Ok(match packed::literal(name, args) {
                        Some(Value::PackedWord {
                            wrapper: Wrapper::U32,
                            bits,
                        }) => Some(bits),
                        _ => None,
                    });
                }
                _ => return Ok(None),
            }
        }
        Ok(None)
    }
}
