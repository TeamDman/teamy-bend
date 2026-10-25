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
use std::ops::ControlFlow;
use std::rc::Rc;
use std::rc::Weak;

// Demand-local sharing, analogous to upstream term_cell/VAR fills. Immutable
// input nodes never change. Weak keys prevent allocator-address reuse; cached
// constructor heads retain raw fields, so observation cannot force those fields.
// The table exists only during one normalize call, while aliases are stable.
#[derive(Clone, Debug, Default)]
pub(super) struct NormalizationMemo {
    heads: std::collections::BTreeMap<usize, (Weak<Term>, TermRef)>,
}

impl NormalizationMemo {
    fn get(&self, value: &TermRef) -> Option<TermRef> {
        if !matches!(value.as_ref(), Term::App(..)) {
            return None;
        }
        self.heads
            .get(&(Rc::as_ptr(value) as usize))
            .map(|(_, result)| Rc::clone(result))
    }

    fn insert(&mut self, value: &TermRef, result: &TermRef) {
        if matches!(value.as_ref(), Term::App(..)) && matches!(result.as_ref(), Term::Ctr { .. }) {
            self.heads.insert(
                Rc::as_ptr(value) as usize,
                (Rc::downgrade(value), Rc::clone(result)),
            );
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Comparison {
    Equal,
    Fits,
}

enum PatternArgument {
    Value(TermRef),
    EndFields,
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
        // Even a shared demand must pay its step and pass the nesting guard.
        let cached = self
            .normalization_memo
            .as_ref()
            .and_then(|memo| memo.get(value));
        let result = cached.map_or_else(
            || {
                let result = self.whnf_inner(value);
                if let (Some(memo), Ok(result)) = (&mut self.normalization_memo, &result) {
                    memo.insert(value, result);
                }
                result
            },
            Ok,
        );
        self.depth -= 1;
        result
    }

    fn whnf_inner(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        let mut value = Rc::clone(value);
        loop {
            match value.as_ref() {
                Term::Var { id, .. } => {
                    if let Some(replacement) = self.aliases.get(id).cloned() {
                        value = replacement;
                    } else {
                        return Ok(value);
                    }
                }
                Term::Ann(x, _) => value = Rc::clone(x),
                Term::Let { bindings, body } => {
                    // Bindings are simultaneous: values use only the outer scope.
                    let mut result = Rc::clone(body);
                    for b in bindings {
                        result = substitute(&result, b.id, &b.value);
                    }
                    value = result;
                }
                Term::Min(a, b) => {
                    let a = self.whnf(a)?;
                    match a.as_ref() {
                        Term::Qua(Quant::None) => return Ok(a),
                        Term::Qua(Quant::Many) => {
                            value = Rc::clone(b);
                            self.step()?;
                            continue;
                        }
                        _ => (),
                    }
                    let b = self.whnf(b)?;
                    return match (a.as_ref(), b.as_ref()) {
                        (_, Term::Qua(Quant::None)) | (Term::Qua(_), Term::Qua(Quant::Lone)) => {
                            Ok(b)
                        }
                        (_, Term::Qua(Quant::Many)) => Ok(a),
                        _ => Ok(term(Term::Min(a, b))),
                    };
                }
                Term::Rwt { evidence, body, .. } => {
                    if matches!(self.whnf(evidence)?.as_ref(), Term::Rfl) {
                        value = Rc::clone(body);
                    } else {
                        return Ok(value);
                    }
                }
                Term::Ref(name) | Term::GpuRef(name) if self.adts.contains_key(name) => {
                    let declaration = &self.adts[name];
                    if declaration.parameters.is_empty() {
                        let result = term(Term::Adt {
                            name: name.clone(),
                            args: vec![],
                            excluded: vec![],
                        });
                        if let Some(origins) = &self.origins {
                            origins.transformed(&value, &result);
                        }
                        return Ok(result);
                    }
                    return Ok(value);
                }
                Term::Ref(_) | Term::GpuRef(_) | Term::App(_, _) => {
                    match self.whnf_spine(&value)? {
                        ControlFlow::Break(result) => return Ok(result),
                        ControlFlow::Continue(next) => value = next,
                    }
                }
                _ => return Ok(value),
            }
            // A tail reduction has no pending work requiring a depth frame.
            // Charge exactly the step that its former recursive whnf call used.
            self.step()?;
        }
    }

    fn whnf_spine(
        &mut self,
        value: &TermRef,
    ) -> Result<ControlFlow<TermRef, TermRef>, KernelError> {
        let (head, args) = spine(value);
        if let Term::Ref(name) | Term::GpuRef(name) = head.as_ref() {
            let definition = self.defs.get(name).cloned();
            if let Some(def) = definition
                && args.len() >= def.parameters.len()
                && let Some(body) = def.body
            {
                let body = self.freshen(&body)?;
                return self.named_patterns(value, body, args, def.parameters.len());
            }
            return Ok(ControlFlow::Break(Rc::clone(value)));
        }
        if args.is_empty() {
            return Ok(ControlFlow::Break(Rc::clone(value)));
        }
        let mut result = self.whnf(&head)?;
        let mut progressed = false;
        for arg in args {
            match self.consume(&result, &arg, false)? {
                Some(next) => {
                    result = next;
                    progressed = true;
                }
                None => result = term(Term::App(result, arg)),
            }
        }
        // Reducing a computed head can reveal a partial named call. Retrying
        // its rebuilt spine lets these outer arguments saturate it. The direct
        // named path above still stops on underapplication or blocked matching.
        // Keep every existing consume charge and pay the usual tail retry step.
        let mut rebuilt_head = &result;
        while let Term::App(function, _) = rebuilt_head.as_ref() {
            rebuilt_head = function;
        }
        if progressed || matches!(rebuilt_head.as_ref(), Term::Ref(_) | Term::GpuRef(_)) {
            Ok(ControlFlow::Continue(result))
        } else {
            Ok(ControlFlow::Break(result))
        }
    }

    fn named_patterns(
        &mut self,
        original: &TermRef,
        body: TermRef,
        mut args: Vec<TermRef>,
        arity: usize,
    ) -> Result<ControlFlow<TermRef, TermRef>, KernelError> {
        let extra = args.split_off(arity);
        let pending = args.into_iter().rev().map(PatternArgument::Value).collect();
        // Field applications have pending work just like the former recursive
        // whnf(apply(arm, fields)). Restore only these owned depth frames on
        // return/error; fuel, fresh identities and aliases are never reset.
        let depth = self.depth;
        let result = self.named_patterns_inner(original, body, pending, extra);
        self.depth = depth;
        result
    }

    fn named_patterns_inner(
        &mut self,
        original: &TermRef,
        mut body: TermRef,
        mut pending: Vec<PatternArgument>,
        extra: Vec<TermRef>,
    ) -> Result<ControlFlow<TermRef, TermRef>, KernelError> {
        while let Some(argument) = pending.pop() {
            let PatternArgument::Value(argument) = argument else {
                self.depth -= 1;
                continue;
            };
            self.step()?;
            match body.as_ref() {
                Term::Lam { id, body: next, .. } => {
                    body = substitute(next, *id, &argument);
                }
                Term::Mat {
                    constructor,
                    arm,
                    fallback,
                } => {
                    let evaluated = self.whnf(&argument)?;
                    let Term::Ctr { name, args } = evaluated.as_ref() else {
                        return Ok(ControlFlow::Break(Rc::clone(original)));
                    };
                    if name == constructor {
                        self.step()?;
                        self.enter()?;
                        pending.push(PatternArgument::EndFields);
                        pending.extend(
                            args.iter()
                                .rev()
                                .map(|x| PatternArgument::Value(Rc::clone(x))),
                        );
                        body = Rc::clone(arm);
                    } else {
                        pending.push(PatternArgument::Value(evaluated));
                        body = Rc::clone(fallback);
                    }
                }
                Term::Efq => return Ok(ControlFlow::Break(Rc::clone(original))),
                Term::Ann(value, _) => {
                    pending.push(PatternArgument::Value(argument));
                    body = Rc::clone(value);
                }
                Term::Let {
                    bindings,
                    body: next,
                } => {
                    let mut next = Rc::clone(next);
                    for binding in bindings {
                        next = substitute(&next, binding.id, &binding.value);
                    }
                    pending.push(PatternArgument::Value(argument));
                    body = next;
                }
                Term::Rwt {
                    evidence,
                    body: next,
                    ..
                } if matches!(self.whnf(evidence)?.as_ref(), Term::Rfl) => {
                    pending.push(PatternArgument::Value(argument));
                    body = Rc::clone(next);
                }
                _ => {
                    // A new call/computation ends this definition's pattern
                    // prefix. Pass every remaining field/parameter together so
                    // an underapplied callee can become saturated normally.
                    let args = std::iter::once(argument)
                        .chain(pending.into_iter().rev().filter_map(|arg| match arg {
                            PatternArgument::Value(value) => Some(value),
                            PatternArgument::EndFields => None,
                        }))
                        .chain(extra);
                    return Ok(ControlFlow::Continue(apply(body, args)));
                }
            }
        }
        Ok(ControlFlow::Continue(apply(body, extra)))
    }

    fn consume(
        &mut self,
        function: &TermRef,
        arg: &TermRef,
        residual: bool,
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
                        self.consume(fallback, &evaluated, true)
                    }
                } else {
                    Ok(None)
                }
            }
            // An impossible residual arm retains the original match graph.
            Term::Efq => Ok(None),
            // A residual value is an argument to a named or open fallback.
            // Supply it before testing whether a named call is saturated.
            _ if residual => Ok(Some(term(Term::App(function, Rc::clone(arg))))),
            _ => Ok(None),
        }
    }

    pub(crate) fn normalize(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        self.step()?;
        self.enter()?;
        let owns_memo = self.normalization_memo.is_none();
        if owns_memo {
            self.normalization_memo = Some(NormalizationMemo::default());
        }
        let result = self.normalize_inner(value);
        if owns_memo {
            self.normalization_memo = None;
        }
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
            (Term::Ref(a) | Term::GpuRef(a), Term::Ref(b) | Term::GpuRef(b)) => Ok(a == b),
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
