// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use super::Quant;
use super::Term;
use super::TermRef;
use super::check::Engine;
use super::check::KernelError;
use super::substitute;
use super::term;
use std::rc::Rc;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Comparison {
    Equal,
    Fits,
}

pub(crate) fn spine(value: &TermRef) -> (TermRef, Vec<TermRef>) {
    let mut head = Rc::clone(value);
    let mut args = Vec::new();
    while let Term::App(f, x) = head.as_ref() {
        args.push(Rc::clone(x));
        head = Rc::clone(f);
    }
    args.reverse();
    (head, args)
}

pub(crate) fn apply(mut f: TermRef, args: impl IntoIterator<Item = TermRef>) -> TermRef {
    for x in args {
        f = match f.as_ref() {
            Term::Lam { id, body, .. } => substitute(body, *id, &x),
            _ => term(Term::App(f, x)),
        };
    }
    f
}

impl Engine {
    pub(crate) fn whnf(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.whnf_inner(value);
        self.depth -= 1;
        result
    }

    fn whnf_inner(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        match value.as_ref() {
            Term::Var { id, .. } => {
                if let Some(replacement) = self.aliases.get(id).cloned() {
                    self.whnf(&replacement)
                } else {
                    Ok(Rc::clone(value))
                }
            }
            Term::Ann(x, _) => self.whnf(x),
            Term::Let { bindings, body } => {
                // Bindings are simultaneous: values use only the outer scope.
                let mut result = Rc::clone(body);
                for b in bindings {
                    result = substitute(&result, b.id, &b.value);
                }
                self.whnf(&result)
            }
            Term::Min(a, b) => {
                let a = self.whnf(a)?;
                match a.as_ref() {
                    Term::Qua(Quant::None) => return Ok(a),
                    Term::Qua(Quant::Many) => return self.whnf(b),
                    _ => (),
                }
                let b = self.whnf(b)?;
                match (a.as_ref(), b.as_ref()) {
                    (_, Term::Qua(Quant::None)) | (Term::Qua(_), Term::Qua(Quant::Lone)) => Ok(b),
                    (_, Term::Qua(Quant::Many)) => Ok(a),
                    _ => Ok(term(Term::Min(a, b))),
                }
            }
            Term::Rwt { evidence, body, .. } => {
                if matches!(self.whnf(evidence)?.as_ref(), Term::Rfl) {
                    self.whnf(body)
                } else {
                    Ok(Rc::clone(value))
                }
            }
            Term::Ref(name) if self.adts.contains_key(name) => {
                let declaration = &self.adts[name];
                if declaration.parameters.is_empty() {
                    Ok(term(Term::Adt {
                        name: name.clone(),
                        args: vec![],
                        excluded: vec![],
                    }))
                } else {
                    Ok(Rc::clone(value))
                }
            }
            Term::Ref(_) | Term::App(_, _) => self.whnf_spine(value),
            _ => Ok(Rc::clone(value)),
        }
    }

    fn whnf_spine(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        let (head, args) = spine(value);
        if let Term::Ref(name) = head.as_ref() {
            let definition = self.defs.get(name).cloned();
            if let Some(def) = definition
                && args.len() >= def.parameters.len()
                && let Some(body) = def.body
            {
                let mut body = self.freshen(&body)?;
                for arg in args.iter().take(def.parameters.len()) {
                    let Some(next) = self.consume(&body, arg)? else {
                        return Ok(Rc::clone(value));
                    };
                    body = next;
                }
                let result = apply(body, args.into_iter().skip(def.parameters.len()));
                return self.whnf(&result);
            }
            return Ok(Rc::clone(value));
        }
        if args.is_empty() {
            return Ok(Rc::clone(value));
        }
        let mut result = self.whnf(&head)?;
        let mut progressed = false;
        for arg in args {
            match self.consume(&result, &arg)? {
                Some(next) => {
                    result = next;
                    progressed = true;
                }
                None => result = term(Term::App(result, arg)),
            }
        }
        if progressed {
            self.whnf(&result)
        } else {
            Ok(result)
        }
    }

    fn consume(
        &mut self,
        function: &TermRef,
        arg: &TermRef,
    ) -> Result<Option<TermRef>, KernelError> {
        let function = self.whnf(function)?;
        match function.as_ref() {
            Term::Lam { id, body, .. } => Ok(Some(substitute(body, *id, arg))),
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => {
                let evaluated = self.whnf(arg)?;
                if let Term::Ctr { name, args } = evaluated.as_ref() {
                    if name == constructor {
                        self.whnf(&apply(Rc::clone(arm), args.iter().map(Rc::clone)))
                            .map(Some)
                    } else {
                        self.consume(fallback, &evaluated)
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn normalize(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.normalize_inner(value);
        self.depth -= 1;
        result
    }

    #[expect(
        clippy::too_many_lines,
        reason = "An exhaustive AST traversal makes complete normalization coverage directly reviewable."
    )]
    fn normalize_inner(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        let v = self.whnf(value)?;
        match v.as_ref() {
            Term::Typ(g) => self.normalize(g).map(|g| term(Term::Typ(g))),
            Term::Min(a, b) => {
                let a = self.normalize(a)?;
                let b = self.normalize(b)?;
                Ok(term(Term::Min(a, b)))
            }
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
                let domain = self.normalize(domain)?;
                let body = self.normalize(&substitute(body, *id, &x))?;
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
                let body = self.normalize(&substitute(body, *id, &x))?;
                Ok(term(Term::Lam {
                    name: name.clone(),
                    id: *fresh_id,
                    body,
                }))
            }
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => {
                let arm = self.normalize(arm)?;
                let fallback = self.normalize(fallback)?;
                Ok(term(Term::Mat {
                    constructor: constructor.clone(),
                    arm,
                    fallback,
                }))
            }
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                let evidence = self.normalize(evidence)?;
                let motive = self.normalize(motive)?;
                let body = self.normalize(body)?;
                Ok(term(Term::Rwt {
                    evidence,
                    motive,
                    body,
                }))
            }
            Term::Ctr { name, args } => {
                let args = args
                    .iter()
                    .map(|a| self.normalize(a))
                    .collect::<Result<_, _>>()?;
                Ok(term(Term::Ctr {
                    name: name.clone(),
                    args,
                }))
            }
            Term::Adt {
                name,
                args,
                excluded,
            } => {
                let args = args
                    .iter()
                    .map(|a| self.normalize(a))
                    .collect::<Result<_, _>>()?;
                Ok(term(Term::Adt {
                    name: name.clone(),
                    args,
                    excluded: excluded.clone(),
                }))
            }
            Term::App(f, x) => {
                let f = self.normalize(f)?;
                let x = self.normalize(x)?;
                Ok(term(Term::App(f, x)))
            }
            Term::Eql { left, right, ty } => {
                let left = self.normalize(left)?;
                let right = self.normalize(right)?;
                let ty = self.normalize(ty)?;
                Ok(term(Term::Eql { left, right, ty }))
            }
            _ => Ok(v),
        }
    }

    pub(crate) fn compare(
        &mut self,
        mode: Comparison,
        left: &TermRef,
        right: &TermRef,
    ) -> Result<bool, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.compare_inner(mode, left, right);
        self.depth -= 1;
        result
    }
    #[expect(
        clippy::too_many_lines,
        clippy::many_single_char_names,
        reason = "Conversion follows the reference kernel's exhaustive mathematical cases, using paired term variables."
    )]
    fn compare_inner(
        &mut self,
        mode: Comparison,
        left: &TermRef,
        right: &TermRef,
    ) -> Result<bool, KernelError> {
        if Rc::ptr_eq(left, right) {
            return Ok(true);
        }
        let a = self.whnf(left)?;
        let b = self.whnf(right)?;
        if Rc::ptr_eq(&a, &b) {
            return Ok(true);
        }
        if matches!(a.as_ref(), Term::Lam { .. }) || matches!(b.as_ref(), Term::Lam { .. }) {
            let x = self.fresh("eta")?;
            return self.compare(mode, &apply(a, [Rc::clone(&x)]), &apply(b, [x]));
        }
        match (a.as_ref(), b.as_ref()) {
            (Term::Var { id: a, .. }, Term::Var { id: b, .. }) => Ok(a == b),
            (Term::Ref(a), Term::Ref(b)) => Ok(a == b),
            (Term::Typ(g), Term::Typ(h)) => {
                if mode == Comparison::Equal {
                    return self.compare(mode, g, h);
                }
                let g = self.whnf(g)?;
                let h = self.whnf(h)?;
                if matches!(g.as_ref(), Term::Qua(Quant::Many))
                    || matches!(h.as_ref(), Term::Qua(Quant::None | Quant::Lone))
                {
                    return Ok(true);
                }
                if let Term::Min(x, y) = g.as_ref() {
                    return Ok(self.compare(mode, &term(Term::Typ(Rc::clone(x))), &b)?
                        && self.compare(mode, &term(Term::Typ(Rc::clone(y))), &b)?);
                }
                if let Term::Min(x, y) = h.as_ref() {
                    return Ok(self.compare(mode, &a, &term(Term::Typ(Rc::clone(x))))?
                        || self.compare(mode, &a, &term(Term::Typ(Rc::clone(y))))?);
                }
                self.compare(mode, &g, &h)
            }
            (Term::Qnt, Term::Qnt) | (Term::Rfl, Term::Rfl) | (Term::Efq, Term::Efq) => Ok(true),
            (Term::Qua(a), Term::Qua(b)) => Ok(a == b),
            (
                Term::All {
                    quant: aq,
                    id: ai,
                    domain: ad,
                    body: ab,
                    ..
                },
                Term::All {
                    quant: bq,
                    id: bi,
                    domain: bd,
                    body: bb,
                    ..
                },
            ) => {
                if aq != bq || !self.compare(mode, bd, ad)? {
                    return Ok(false);
                }
                let x = self.fresh("cmp")?;
                self.compare(mode, &substitute(ab, *ai, &x), &substitute(bb, *bi, &x))
            }
            (Term::Min(a, b), Term::Min(c, d)) | (Term::App(a, b), Term::App(c, d)) => Ok(self
                .compare(Comparison::Equal, a, c)?
                && self.compare(Comparison::Equal, b, d)?),
            (
                Term::Adt {
                    name: an,
                    args: ax,
                    excluded: ar,
                },
                Term::Adt {
                    name: bn,
                    args: bx,
                    excluded: br,
                },
            ) => {
                if an != bn
                    || (mode == Comparison::Equal && ar.len() != br.len())
                    || !br.iter().all(|x| ar.contains(x))
                {
                    return Ok(false);
                }
                self.compare_list(ax, bx)
            }
            (Term::Ctr { name: an, args: ax }, Term::Ctr { name: bn, args: bx }) => {
                Ok(an == bn && self.compare_list(ax, bx)?)
            }
            (
                Term::Mat {
                    constructor: a,
                    arm: ah,
                    fallback: am,
                },
                Term::Mat {
                    constructor: b,
                    arm: bh,
                    fallback: bm,
                },
            ) => Ok(a == b
                && self.compare(Comparison::Equal, ah, bh)?
                && self.compare(Comparison::Equal, am, bm)?),
            (
                Term::Eql {
                    left: a,
                    right: b,
                    ty: t,
                },
                Term::Eql {
                    left: c,
                    right: d,
                    ty: u,
                },
            ) => Ok(self.compare(Comparison::Equal, a, c)?
                && self.compare(Comparison::Equal, b, d)?
                && self.compare(Comparison::Equal, t, u)?),
            (
                Term::Rwt {
                    evidence: a,
                    motive: b,
                    body: c,
                },
                Term::Rwt {
                    evidence: d,
                    motive: e,
                    body: f,
                },
            ) => Ok(self.compare(Comparison::Equal, a, d)?
                && self.compare(Comparison::Equal, b, e)?
                && self.compare(Comparison::Equal, c, f)?),
            _ => Ok(false),
        }
    }
    fn compare_list(&mut self, a: &[TermRef], b: &[TermRef]) -> Result<bool, KernelError> {
        if a.len() != b.len() {
            return Ok(false);
        }
        for (x, y) in a.iter().zip(b) {
            if !self.compare(Comparison::Equal, x, y)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
