// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use super::LetBinding;
use super::Term;
use super::TermRef;
use super::check::Engine;
use super::check::KernelError;
use super::substitute;
use super::term;
use std::rc::Rc;

impl Engine {
    /// Each unfolding owns fresh binder identities, including recursive instances.
    pub(crate) fn freshen(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.freshen_inner(value);
        self.depth -= 1;
        result
    }

    #[expect(
        clippy::too_many_lines,
        reason = "An exhaustive AST traversal keeps every binding and child term visible for capture-avoidance review."
    )]
    fn freshen_inner(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        match value.as_ref() {
            Term::All {
                quant,
                name,
                id,
                domain,
                body,
            } => {
                let x = self.fresh(name)?;
                let Term::Var { id: fresh_id, .. } = x.as_ref() else {
                    unreachable!()
                };
                let domain = self.freshen(domain)?;
                let body = self.freshen(&substitute(body, *id, &x))?;
                Ok(term(Term::All {
                    quant: *quant,
                    name: name.clone(),
                    id: *fresh_id,
                    domain,
                    body,
                }))
            }
            Term::Lam { name, id, body } => {
                let x = self.fresh(name)?;
                let Term::Var { id: fresh_id, .. } = x.as_ref() else {
                    unreachable!()
                };
                let body = self.freshen(&substitute(body, *id, &x))?;
                Ok(term(Term::Lam {
                    name: name.clone(),
                    id: *fresh_id,
                    body,
                }))
            }
            Term::Let { bindings, body } => {
                let mut body = Rc::clone(body);
                let mut fresh_bindings = Vec::new();
                for binding in bindings {
                    let x = self.fresh(&binding.name)?;
                    let Term::Var { id, .. } = x.as_ref() else {
                        unreachable!()
                    };
                    body = substitute(&body, binding.id, &x);
                    fresh_bindings.push(LetBinding {
                        quant: binding.quant,
                        name: binding.name.clone(),
                        id: *id,
                        value: self.freshen(&binding.value)?,
                    });
                }
                let body = self.freshen(&body)?;
                Ok(term(Term::Let {
                    bindings: fresh_bindings,
                    body,
                }))
            }
            Term::Typ(g) => self.freshen(g).map(|g| term(Term::Typ(g))),
            Term::Min(a, b) => {
                let a = self.freshen(a)?;
                let b = self.freshen(b)?;
                Ok(term(Term::Min(a, b)))
            }
            Term::App(f, x) => {
                let f = self.freshen(f)?;
                let x = self.freshen(x)?;
                Ok(term(Term::App(f, x)))
            }
            Term::Ann(x, t) => {
                let x = self.freshen(x)?;
                let t = self.freshen(t)?;
                Ok(term(Term::Ann(x, t)))
            }
            Term::Adt {
                name,
                args,
                excluded,
            } => {
                let args = args
                    .iter()
                    .map(|x| self.freshen(x))
                    .collect::<Result<_, _>>()?;
                Ok(term(Term::Adt {
                    name: name.clone(),
                    args,
                    excluded: excluded.clone(),
                }))
            }
            Term::Ctr { name, args } => {
                let args = args
                    .iter()
                    .map(|x| self.freshen(x))
                    .collect::<Result<_, _>>()?;
                Ok(term(Term::Ctr {
                    name: name.clone(),
                    args,
                }))
            }
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => {
                let arm = self.freshen(arm)?;
                let fallback = self.freshen(fallback)?;
                Ok(term(Term::Mat {
                    constructor: constructor.clone(),
                    arm,
                    fallback,
                }))
            }
            Term::Eql { left, right, ty } => {
                let left = self.freshen(left)?;
                let right = self.freshen(right)?;
                let ty = self.freshen(ty)?;
                Ok(term(Term::Eql { left, right, ty }))
            }
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                let evidence = self.freshen(evidence)?;
                let motive = self.freshen(motive)?;
                let body = self.freshen(body)?;
                Ok(term(Term::Rwt {
                    evidence,
                    motive,
                    body,
                }))
            }
            _ => Ok(Rc::clone(value)),
        }
    }
}
