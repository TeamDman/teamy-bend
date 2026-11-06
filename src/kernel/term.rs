// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use std::fmt;
use std::rc::Rc;

/// Reference-counted immutable core term.
pub type TermRef = Rc<Term>;

/// Erased, affine, or reusable demand.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Quant {
    None,
    Lone,
    Many,
}

impl Quant {
    pub(crate) const fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, q) | (q, Self::None) => q,
            _ => Self::Many,
        }
    }
    pub(crate) const fn demand(self, outer: Self) -> Self {
        if matches!(self, Self::None) {
            Self::None
        } else {
            outer
        }
    }
    pub(crate) const fn product(self, other: Self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Lone => other,
            Self::Many => other.add(other),
        }
    }
}

/// A named binder. IDs must be unique across a complete parsed book.
#[derive(Clone, Debug)]
pub struct Binder {
    pub quant: Quant,
    pub name: String,
    pub id: usize,
    pub ty: TermRef,
}

/// One parallel let binding.
#[derive(Clone, Debug)]
pub struct LetBinding {
    pub quant: Quant,
    pub name: String,
    pub id: usize,
    pub value: TermRef,
}

/// Bend's first-order core syntax. Surface matches elaborate to `Mat`/`Efq`.
#[derive(Clone, Debug)]
pub enum Term {
    Var {
        name: String,
        id: usize,
    },
    Ref(String),
    /// A named reference with upstream's `!` scheduling annotation. The marker
    /// survives compilation but has no effect on proof equality or evaluation.
    GpuRef(String),
    Typ(TermRef),
    Qnt,
    Qua(Quant),
    Min(TermRef, TermRef),
    All {
        quant: Quant,
        name: String,
        id: usize,
        domain: TermRef,
        body: TermRef,
    },
    Lam {
        name: String,
        id: usize,
        body: TermRef,
    },
    App(TermRef, TermRef),
    Adt {
        name: String,
        args: Vec<TermRef>,
        excluded: Vec<String>,
    },
    Ctr {
        name: String,
        args: Vec<TermRef>,
    },
    Mat {
        constructor: String,
        arm: TermRef,
        fallback: TermRef,
    },
    Efq,
    Eql {
        left: TermRef,
        right: TermRef,
        ty: TermRef,
    },
    Rfl,
    Rwt {
        evidence: TermRef,
        motive: TermRef,
        body: TermRef,
    },
    Hole(String),
    Ann(TermRef, TermRef),
    Let {
        bindings: Vec<LetBinding>,
        body: TermRef,
    },
}

/// An algebraic datatype declaration.
#[derive(Clone, Debug)]
pub struct AdtDecl {
    pub name: String,
    pub parameters: Vec<Binder>,
    pub kind: TermRef,
    pub constructors: Vec<ConstructorDecl>,
}
/// Constructor fields are in a telescope following the datatype parameters.
#[derive(Clone, Debug)]
pub struct ConstructorDecl {
    pub name: String,
    pub fields: Vec<Binder>,
}
/// A law (`body == None`) or definition; `ty` and `body` contain full telescopes.
#[derive(Clone, Debug)]
pub struct DefDecl {
    pub name: String,
    /// Typed declaration telescope. For a parsed law completion, this retains
    /// the law's provisional telescope until checking resolves the actual list.
    pub parameters: Vec<Binder>,
    /// Actual names and IDs supplied by an ordinary `def` completing a law.
    /// Types and quantities come exclusively from the checked function type;
    /// aliases need not expose a syntactic telescope during parsing. Checked
    /// definitions contain canonical `parameters` and clear this field.
    pub completion_parameters: Option<Vec<(String, usize)>>,
    pub ty: TermRef,
    pub body: Option<TermRef>,
    pub unsafe_: bool,
    pub foreign: bool,
}
impl DefDecl {
    /// Source arity, excluding binders of a returned function.
    #[must_use]
    pub fn declared_arity(&self) -> usize {
        self.completion_parameters
            .as_ref()
            .map_or(self.parameters.len(), Vec::len)
    }
}
/// Source-order declaration events, including a law and its later proof.
#[derive(Clone, Debug)]
pub enum Declaration {
    Adt(AdtDecl),
    Def(DefDecl),
}
/// Complete source-order module graph.
#[derive(Clone, Debug, Default)]
pub struct Book {
    pub declarations: Vec<Declaration>,
}

/// Allocate an immutable syntax node.
#[must_use]
pub fn term(value: Term) -> TermRef {
    Rc::new(value)
}

/// Release an owned term graph without recursively dropping uniquely owned
/// children. Shared children remain alive until their other owners release them.
///
/// This is an explicit cleanup operation: ordinary `Rc<Term>` destruction is
/// unchanged and can still recurse. Callers holding another deep root must also
/// release that root iteratively. Temporary storage is proportional to pending
/// graph edges, and no semantic validation or resource budget is bypassed.
pub fn release_term_iteratively(root: TermRef) {
    // A shared root cannot be destroyed by releasing this owner; avoid
    // allocating a work list on that common path.
    if Rc::strong_count(&root) > 1 {
        return;
    }
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        let Ok(value) = Rc::try_unwrap(value) else {
            continue;
        };
        match value {
            Term::Var { .. }
            | Term::Ref(_)
            | Term::GpuRef(_)
            | Term::Qnt
            | Term::Qua(_)
            | Term::Efq
            | Term::Rfl
            | Term::Hole(_) => {}
            Term::Typ(value) | Term::Lam { body: value, .. } => pending.push(value),
            Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => pending.extend([a, b]),
            Term::All { domain, body, .. } => pending.extend([domain, body]),
            Term::Adt { args, .. } | Term::Ctr { args, .. } => pending.extend(args),
            Term::Mat { arm, fallback, .. } => pending.extend([arm, fallback]),
            Term::Eql { left, right, ty } => pending.extend([left, right, ty]),
            Term::Rwt {
                evidence,
                motive,
                body,
            } => pending.extend([evidence, motive, body]),
            Term::Let { bindings, body } => {
                pending.extend(bindings.into_iter().map(|binding| binding.value));
                pending.push(body);
            }
        }
    }
}

// Intermediate owned results must not recursively destroy a deep graph when a
// later sibling fails or an operation unwinds. Successful callers take the
// vector out, leaving the guard empty. This does not change `Rc<Term>` drop.
#[derive(Default)]
pub(super) struct PendingTerms(Vec<TermRef>);

impl std::ops::Deref for PendingTerms {
    type Target = Vec<TermRef>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PendingTerms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for PendingTerms {
    fn drop(&mut self) {
        while let Some(value) = self.0.pop() {
            release_term_iteratively(value);
        }
    }
}

pub(super) fn try_map_terms<E>(
    values: &[TermRef],
    mut transform: impl FnMut(&TermRef) -> Result<TermRef, E>,
) -> Result<Vec<TermRef>, E> {
    let mut pending = PendingTerms::default();
    for value in values {
        pending.push(transform(value)?);
    }
    Ok(std::mem::take(&mut pending.0))
}

#[cfg(test)]
thread_local! {
    static SUBSTITUTION_TEST_FAULT: std::cell::Cell<*const Term> = const {
        std::cell::Cell::new(std::ptr::null())
    };
    static SUBSTITUTION_TEST_RESULT: std::cell::RefCell<Option<std::rc::Weak<Term>>> = const {
        std::cell::RefCell::new(None)
    };
}
/// Form the dependent function telescope over a list of binders.
#[must_use]
pub fn arrows(binders: &[Binder], mut result: TermRef) -> TermRef {
    for b in binders.iter().rev() {
        result = term(Term::All {
            quant: b.quant,
            name: b.name.clone(),
            id: b.id,
            domain: Rc::clone(&b.ty),
            body: result,
        });
    }
    result
}
/// Form lambda abstractions in the order of a function's parameters.
#[must_use]
pub fn lambdas(binders: &[Binder], mut body: TermRef) -> TermRef {
    for b in binders.iter().rev() {
        body = term(Term::Lam {
            name: b.name.clone(),
            id: b.id,
            body,
        });
    }
    body
}

// Keep loader provenance when a constructor is unchanged. Mapped fields that
// actually change produce a new, untracked node instead of a guessed origin.
pub(crate) fn map_constructor(
    value: &TermRef,
    name: &str,
    args: &[TermRef],
    transform: impl FnMut(&TermRef) -> TermRef,
) -> TermRef {
    let mapped: Vec<_> = args.iter().map(transform).collect();
    if args
        .iter()
        .zip(&mapped)
        .all(|(old, new)| Rc::ptr_eq(old, new))
    {
        Rc::clone(value)
    } else {
        term(Term::Ctr {
            name: name.into(),
            args: mapped,
        })
    }
}

/// Capture-free substitution for terms whose binder IDs are globally unique.
/// Traversal uses explicit heap frames; the returned `Rc<Term>` still has
/// ordinary recursive destruction unless explicitly released iteratively.
///
/// # Panics
/// Panics if an internal traversal-frame invariant is violated.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "Every core term variant must preserve its substitution and binder rules."
)]
pub fn substitute(root: &TermRef, target: usize, replacement: &TermRef) -> TermRef {
    enum Task<'a> {
        Visit(&'a TermRef),
        Build(&'a TermRef),
    }
    let mut tasks = vec![Task::Visit(root)];
    let mut results = PendingTerms::default();
    #[cfg(test)]
    let inject_fault = SUBSTITUTION_TEST_FAULT.with(|fault| !fault.get().is_null());
    while let Some(task) = tasks.pop() {
        #[cfg(test)]
        if inject_fault
            && let Task::Visit(node) = &task
            && SUBSTITUTION_TEST_FAULT.with(|fault| Rc::as_ptr(node) == fault.get())
        {
            SUBSTITUTION_TEST_RESULT.with(|result| {
                *result.borrow_mut() = Some(Rc::downgrade(results.last().unwrap()));
            });
            SUBSTITUTION_TEST_FAULT.with(|fault| fault.set(std::ptr::null()));
            panic!("deliberate substitution unwind");
        }
        match task {
            Task::Visit(node) => {
                match node.as_ref() {
                    Term::Var { id, .. } if *id == target => {
                        results.push(Rc::clone(replacement));
                        continue;
                    }
                    Term::Var { .. }
                    | Term::Ref(_)
                    | Term::GpuRef(_)
                    | Term::Qnt
                    | Term::Qua(_)
                    | Term::Efq
                    | Term::Rfl
                    | Term::Hole(_) => {
                        results.push(Rc::clone(node));
                        continue;
                    }
                    Term::Lam { id, .. } if *id == target => {
                        results.push(Rc::clone(node));
                        continue;
                    }
                    Term::Typ(_)
                    | Term::Min(..)
                    | Term::All { .. }
                    | Term::Lam { .. }
                    | Term::App(..)
                    | Term::Adt { .. }
                    | Term::Ctr { .. }
                    | Term::Mat { .. }
                    | Term::Eql { .. }
                    | Term::Rwt { .. }
                    | Term::Ann(..)
                    | Term::Let { .. } => {}
                }
                tasks.push(Task::Build(node));
                match node.as_ref() {
                    Term::Typ(a) | Term::Lam { body: a, .. } => tasks.push(Task::Visit(a)),
                    Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => {
                        tasks.push(Task::Visit(b));
                        tasks.push(Task::Visit(a));
                    }
                    Term::All {
                        id, domain, body, ..
                    } => {
                        if *id != target {
                            tasks.push(Task::Visit(body));
                        }
                        tasks.push(Task::Visit(domain));
                    }
                    Term::Adt { args, .. } | Term::Ctr { args, .. } => {
                        tasks.extend(args.iter().rev().map(Task::Visit));
                    }
                    Term::Mat { arm, fallback, .. } => {
                        tasks.push(Task::Visit(fallback));
                        tasks.push(Task::Visit(arm));
                    }
                    Term::Eql { left, right, ty } => {
                        tasks.push(Task::Visit(ty));
                        tasks.push(Task::Visit(right));
                        tasks.push(Task::Visit(left));
                    }
                    Term::Rwt {
                        evidence,
                        motive,
                        body,
                    } => {
                        tasks.push(Task::Visit(body));
                        tasks.push(Task::Visit(motive));
                        tasks.push(Task::Visit(evidence));
                    }
                    Term::Let { bindings, body } => {
                        if !bindings.iter().any(|b| b.id == target) {
                            tasks.push(Task::Visit(body));
                        }
                        tasks.extend(bindings.iter().rev().map(|b| Task::Visit(&b.value)));
                    }
                    Term::Var { .. }
                    | Term::Ref(_)
                    | Term::GpuRef(_)
                    | Term::Qnt
                    | Term::Qua(_)
                    | Term::Efq
                    | Term::Rfl
                    | Term::Hole(_) => unreachable!("leaf handled before scheduling"),
                }
            }
            Task::Build(node) => {
                let mut pop = || results.pop().unwrap();
                let next = match node.as_ref() {
                    Term::Typ(_) => term(Term::Typ(pop())),
                    Term::Min(..) => {
                        let b = pop();
                        let a = pop();
                        term(Term::Min(a, b))
                    }
                    Term::All {
                        quant,
                        name,
                        id,
                        body,
                        ..
                    } => {
                        let b = if *id == target {
                            Rc::clone(body)
                        } else {
                            pop()
                        };
                        let a = pop();
                        term(Term::All {
                            quant: *quant,
                            name: name.clone(),
                            id: *id,
                            domain: a,
                            body: b,
                        })
                    }
                    Term::Lam { name, id, .. } => term(Term::Lam {
                        name: name.clone(),
                        id: *id,
                        body: pop(),
                    }),
                    Term::App(a, b) => {
                        let y = pop();
                        let x = pop();
                        if Rc::ptr_eq(a, &x) && Rc::ptr_eq(b, &y) {
                            Rc::clone(node)
                        } else {
                            term(Term::App(x, y))
                        }
                    }
                    Term::Adt {
                        name,
                        args,
                        excluded,
                    } => {
                        let start = results.len() - args.len();
                        let out = results.split_off(start);
                        term(Term::Adt {
                            name: name.clone(),
                            args: out,
                            excluded: excluded.clone(),
                        })
                    }
                    Term::Ctr { name, args } => {
                        let start = results.len() - args.len();
                        let out = results.split_off(start);
                        if args.iter().zip(&out).all(|(a, b)| Rc::ptr_eq(a, b)) {
                            Rc::clone(node)
                        } else {
                            term(Term::Ctr {
                                name: name.clone(),
                                args: out,
                            })
                        }
                    }
                    Term::Mat { constructor, .. } => {
                        let fallback = pop();
                        let arm = pop();
                        term(Term::Mat {
                            constructor: constructor.clone(),
                            arm,
                            fallback,
                        })
                    }
                    Term::Eql { .. } => {
                        let ty = pop();
                        let right = pop();
                        let left = pop();
                        term(Term::Eql { left, right, ty })
                    }
                    Term::Rwt { .. } => {
                        let body = pop();
                        let motive = pop();
                        let evidence = pop();
                        term(Term::Rwt {
                            evidence,
                            motive,
                            body,
                        })
                    }
                    Term::Ann(..) => {
                        let ty = pop();
                        let value = pop();
                        term(Term::Ann(value, ty))
                    }
                    Term::Let { bindings, body } => {
                        let body = if bindings.iter().any(|b| b.id == target) {
                            Rc::clone(body)
                        } else {
                            pop()
                        };
                        let start = results.len() - bindings.len();
                        let mapped = results.split_off(start);
                        let bindings = bindings
                            .iter()
                            .zip(mapped)
                            .map(|(b, value)| LetBinding {
                                quant: b.quant,
                                name: b.name.clone(),
                                id: b.id,
                                value,
                            })
                            .collect();
                        term(Term::Let { bindings, body })
                    }
                    Term::Var { .. }
                    | Term::Ref(_)
                    | Term::GpuRef(_)
                    | Term::Qnt
                    | Term::Qua(_)
                    | Term::Efq
                    | Term::Rfl
                    | Term::Hole(_) => unreachable!("leaf cannot have a build frame"),
                };
                results.push(next);
            }
        }
    }
    assert_eq!(results.len(), 1);
    results.pop().unwrap()
}

pub(crate) fn inspect(value: &TermRef) -> Result<usize, super::KernelError> {
    inspect_input(value, false).map(|input| input.highest)
}

pub(super) struct InspectedInput {
    pub highest: usize,
    pub hole: Option<String>,
}

pub(super) fn inspect_for_check(value: &TermRef) -> Result<InspectedInput, super::KernelError> {
    inspect_input(value, true)
}

fn inspect_input(value: &TermRef, defer_holes: bool) -> Result<InspectedInput, super::KernelError> {
    let mut stack = vec![(value, 0_usize)];
    let mut highest = 0;
    let mut hole = None;
    let mut remaining = 2_000_000_usize;
    while let Some((value, depth)) = stack.pop() {
        if depth > 128 {
            return Err(super::KernelError::new(
                "kernel input nesting limit exhausted; proof not accepted",
            ));
        }
        remaining = remaining.checked_sub(1).ok_or_else(|| {
            super::KernelError::new("kernel input size limit exhausted; proof not accepted")
        })?;
        let next = depth + 1;
        match value.as_ref() {
            Term::Hole(name) => {
                if !defer_holes {
                    return Err(super::KernelError::new(format!(
                        "unfinished proof hole ?{name}"
                    )));
                }
                if hole.is_none() {
                    hole = Some(name.clone());
                }
            }
            Term::Var { id, .. } => highest = highest.max(*id),
            Term::All {
                id, domain, body, ..
            } => {
                highest = highest.max(*id);
                stack.extend([(domain, next), (body, next)]);
            }
            Term::Lam { id, body, .. } => {
                highest = highest.max(*id);
                stack.push((body, next));
            }
            Term::Typ(x) => stack.push((x, next)),
            Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => {
                stack.extend([(a, next), (b, next)]);
            }
            Term::Adt { args, .. } | Term::Ctr { args, .. } => {
                stack.extend(args.iter().map(|a| (a, next)));
            }
            Term::Mat { arm, fallback, .. } => stack.extend([(arm, next), (fallback, next)]),
            Term::Eql { left, right, ty } => {
                stack.extend([(left, next), (right, next), (ty, next)]);
            }
            Term::Rwt {
                evidence,
                motive,
                body,
            } => stack.extend([(evidence, next), (motive, next), (body, next)]),
            Term::Let { bindings, body } => {
                stack.push((body, next));
                for b in bindings {
                    highest = highest.max(b.id);
                    stack.push((&b.value, next));
                }
            }
            _ => (),
        }
    }
    Ok(InspectedInput { highest, hole })
}

impl fmt::Display for Quant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "-",
            Self::Lone => "",
            Self::Many => "+",
        })
    }
}

// Display-only lexical context. Semantic IDs remain unchanged; the suffix is
// the binder's level in this printed term, matching pinned term_show.
struct ScopedTerm<'a> {
    value: &'a Term,
    scope: PrintScope<'a>,
}

#[derive(Clone, Copy)]
struct PrintScope<'a> {
    binders: &'a [(usize, &'a str)],
    diagnostic: bool,
}

impl PrintScope<'_> {
    fn with_binders<'a>(self, binders: &'a [(usize, &'a str)]) -> PrintScope<'a> {
        PrintScope {
            binders,
            diagnostic: self.diagnostic,
        }
    }
}

pub(super) fn scoped<'a>(
    value: &'a Term,
    binders: &'a [(usize, &'a str)],
) -> impl fmt::Display + 'a {
    within(
        value,
        PrintScope {
            binders,
            diagnostic: false,
        },
    )
}

// Pinned diagnostic expr_show uses term_show at precedence -1. Keep the public
// Term Display (including internal KernelError messages) independent of this
// source-facing presentation correction. No terms or scopes are transformed.
pub(super) fn diagnostic<'a>(
    value: &'a Term,
    binders: &'a [(usize, &'a str)],
) -> impl fmt::Display + 'a {
    within(
        value,
        PrintScope {
            binders,
            diagnostic: true,
        },
    )
}

fn within<'a>(value: &'a Term, scope: PrintScope<'a>) -> ScopedTerm<'a> {
    ScopedTerm { value, scope }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        scoped(self, &[]).fmt(f)
    }
}

impl fmt::Display for ScopedTerm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = self.scope;
        match self.value {
            Term::Var { name, id } => {
                f.write_str(name)?;
                if let Some(level) = scope.binders.iter().position(|(bound, _)| bound == id)
                    && scope.binders.iter().rposition(|(_, bound)| *bound == name) != Some(level)
                {
                    write!(f, "^{level}")?;
                }
                Ok(())
            }
            Term::Ref(name) => reference(f, name, false, scope),
            Term::GpuRef(name) => reference(f, name, true, scope),
            Term::Typ(g) => match g.as_ref() {
                Term::Qua(Quant::Lone) => f.write_str("Type"),
                Term::Qua(Quant::Many) => f.write_str("Data"),
                _ => write!(f, "Kind({})", within(g, scope)),
            },
            Term::Qnt => f.write_str("Quant"),
            Term::Qua(q) => write!(
                f,
                "&{}",
                match q {
                    Quant::None => 0,
                    Quant::Lone => 1,
                    Quant::Many => 2,
                }
            ),
            Term::Min(a, b) => {
                if !scope.diagnostic {
                    f.write_str("(")?;
                }
                infix_operand(f, a, scope)?;
                f.write_str(" <&> ")?;
                infix_operand(f, b, scope)?;
                if !scope.diagnostic {
                    f.write_str(")")?;
                }
                Ok(())
            }
            Term::All {
                quant,
                name,
                domain,
                body,
                id,
            } => {
                write!(f, "@{quant}{name}:")?;
                infix_operand(f, domain, scope)?;
                let mut body_scope = scope.binders.to_vec();
                body_scope.push((*id, name));
                write!(f, " -> {}", within(body, scope.with_binders(&body_scope)))
            }
            Term::Lam { name, id, body } => {
                let mut body_scope = scope.binders.to_vec();
                body_scope.push((*id, name));
                write!(
                    f,
                    "{name} => {}",
                    within(body, scope.with_binders(&body_scope))
                )
            }
            Term::App(..) => application(f, self.value, scope),
            Term::Adt {
                name,
                args,
                excluded,
            } => {
                f.write_str(name)?;
                if !args.is_empty() || !excluded.is_empty() {
                    f.write_str("<")?;
                    list(f, args, scope)?;
                    f.write_str(">")?;
                }
                for constructor in excluded {
                    write!(f, " - {constructor}{{}}")?;
                }
                Ok(())
            }
            Term::Ctr { name, args } => constructor(f, self.value, name, args, scope),
            Term::Mat { .. } => match_term(f, self.value, scope),
            Term::Efq => f.write_str("\\{}"),
            Term::Eql { left, right, ty } => equation(f, left, right, ty, scope),
            Term::Rfl => f.write_str("{==}"),
            Term::Rwt {
                evidence,
                motive,
                body,
            } => rewrite(f, evidence, motive, body, scope),
            Term::Hole(name) => write!(f, "?{name}"),
            Term::Ann(x, t) => write!(f, "{{{} : {}}}", within(x, scope), within(t, scope)),
            Term::Let { bindings, body } => {
                let mut body_scope = scope.binders.to_vec();
                for b in bindings {
                    write!(f, "{}{} = {}; ", b.quant, b.name, within(&b.value, scope))?;
                    body_scope.push((b.id, &b.name));
                }
                write!(f, "{}", within(body, scope.with_binders(&body_scope)))
            }
        }
    }
}

fn reference(
    f: &mut fmt::Formatter<'_>,
    name: &str,
    gpu: bool,
    scope: PrintScope<'_>,
) -> fmt::Result {
    f.write_str(name)?;
    if scope.binders.iter().any(|(_, bound)| *bound == name) {
        f.write_str("^")?;
    }
    if gpu {
        f.write_str("!")?;
    }
    Ok(())
}

fn match_term(f: &mut fmt::Formatter<'_>, value: &Term, scope: PrintScope<'_>) -> fmt::Result {
    // Upstream term_show prints a consecutive fallback chain as
    // ordered arms in one match. An empty final fallback is implicit.
    f.write_str("\\{")?;
    let mut current = value;
    let mut first = true;
    while let Term::Mat {
        constructor,
        arm,
        fallback,
    } = current
    {
        if !first {
            f.write_str("; ")?;
        }
        write!(f, "{constructor}: ")?;
        match_branch(f, arm, scope)?;
        current = fallback;
        first = false;
    }
    if !matches!(current, Term::Efq) {
        f.write_str("; ")?;
        match_branch(f, current, scope)?;
    }
    f.write_str("}")
}

fn equation(
    f: &mut fmt::Formatter<'_>,
    left: &Term,
    right: &Term,
    ty: &Term,
    scope: PrintScope<'_>,
) -> fmt::Result {
    f.write_str("{")?;
    match_branch(f, left, scope)?;
    f.write_str(" == ")?;
    match_branch(f, right, scope)?;
    f.write_str(" : ")?;
    match_branch(f, ty, scope)?;
    f.write_str("}")
}

fn constructor(
    f: &mut fmt::Formatter<'_>,
    value: &Term,
    name: &str,
    args: &[TermRef],
    scope: PrintScope<'_>,
) -> fmt::Result {
    if let Some((count, tail)) = nat_spine(value) {
        write!(f, "{count}n")?;
        if !empty_constructor(tail, "Zero") {
            f.write_str("+")?;
            match_branch(f, tail, scope)?;
        }
        return Ok(());
    }
    if let Some(character) = literal_character(value) {
        f.write_str("'")?;
        write_character(f, character, '\'')?;
        return f.write_str("'");
    }
    if let Some((prefix, quoted_tail)) = string_plan(value) {
        return print_string(f, value, scope, prefix, quoted_tail);
    }
    if complete_list(value) {
        return list_literal(f, value, scope);
    }
    if open_list(value) {
        return cons_chain(f, value, scope);
    }
    if name == "Tuple" && args.len() == 2 {
        return tuple_literal(f, value, scope);
    }
    if matches!(name, "ALeaf" | "ANode")
        && let Some(elements) = array_elements(value)
    {
        f.write_str("[")?;
        for (index, element) in elements.into_iter().enumerate() {
            if index != 0 {
                f.write_str(", ")?;
            }
            argument(f, element, scope)?;
        }
        return f.write_str("]");
    }
    if name == "U32"
        && let [word] = args
        && let Some(value) = literal_word(word)
    {
        return write!(f, "{value}");
    }
    write!(f, "{name}{{")?;
    list(f, args, scope)?;
    f.write_str("}")
}

// Match term_show structurally: a partial string is never evaluated or
// silently truncated, and a character's U32 may contain annotations.
fn literal_character(value: &Term) -> Option<u32> {
    let Term::Ctr { name, args } = value else {
        return None;
    };
    if name != "Chr" {
        return None;
    }
    let [number] = args.as_slice() else {
        return None;
    };
    let mut number = number;
    while let Term::Ann(inner, _) = number.as_ref() {
        number = inner;
    }
    let Term::Ctr { name, args } = number.as_ref() else {
        return None;
    };
    if name != "U32" {
        return None;
    }
    let [word] = args.as_slice() else {
        return None;
    };
    literal_word(word)
}

// Find the longest literal suffix in one pass. A partially open string must
// retain its constructors; repeatedly rescanning each suffix would be quadratic.
fn string_plan(mut value: &Term) -> Option<(usize, bool)> {
    let mut length = 0;
    let mut last_nonliteral = 0;
    loop {
        match value {
            Term::Ctr { name, args } if name == "SNil" && args.is_empty() => {
                return Some((last_nonliteral, true));
            }
            Term::Ctr { name, args } if name == "SCon" && args.len() == 2 => {
                length += 1;
                if literal_character(&args[0]).is_none() {
                    last_nonliteral = length;
                }
                value = &args[1];
            }
            _ => return (length != 0).then_some((length, false)),
        }
    }
}

fn print_string(
    f: &mut fmt::Formatter<'_>,
    mut value: &Term,
    scope: PrintScope<'_>,
    prefix: usize,
    quoted_tail: bool,
) -> fmt::Result {
    for _ in 0..prefix {
        let Term::Ctr { args, .. } = value else {
            unreachable!("validated string prefix")
        };
        let [head, tail] = args.as_slice() else {
            unreachable!("validated string arity")
        };
        write!(f, "SCon{{{}, ", within(head, scope))?;
        value = tail;
    }
    if quoted_tail {
        f.write_str("\"")?;
        while let Term::Ctr { name, args } = value {
            if name == "SNil" {
                break;
            }
            let [head, tail] = args.as_slice() else {
                unreachable!("validated string tail")
            };
            write_character(
                f,
                literal_character(head).expect("validated character"),
                '"',
            )?;
            value = tail;
        }
        f.write_str("\"")?;
    } else {
        write!(f, "{}", within(value, scope))?;
    }
    for _ in 0..prefix {
        f.write_str("}")?;
    }
    Ok(())
}

fn write_character(f: &mut fmt::Formatter<'_>, value: u32, quote: char) -> fmt::Result {
    match value {
        10 => f.write_str("\\n"),
        9 => f.write_str("\\t"),
        13 => f.write_str("\\r"),
        0 => f.write_str("\\0"),
        92 => f.write_str("\\\\"),
        39 if quote == '\'' => f.write_str("\\'"),
        34 if quote == '"' => f.write_str("\\\""),
        1..=31 | 127 | 0xd800..=0xdfff | 0x0011_0000..=u32::MAX => {
            write!(f, "\\u{{{value:x}}}")
        }
        _ => write!(f, "{}", char::from_u32(value).expect("valid scalar")),
    }
}

// Pinned term_show flattens only complete ALeaf/ANode trees. Inspect syntax
// without evaluating leaves or crossing annotations, and validate the whole
// tree before writing so a partial tree retains its constructor spelling.
fn array_elements(value: &Term) -> Option<Vec<&Term>> {
    let mut pending = vec![value];
    let mut elements = Vec::new();
    while let Some(value) = pending.pop() {
        match value {
            Term::Ctr { name, args } if name == "ALeaf" && args.len() == 1 => {
                elements.push(args[0].as_ref());
            }
            Term::Ctr { name, args } if name == "ANode" && args.len() == 2 => {
                pending.push(args[1].as_ref());
                pending.push(args[0].as_ref());
            }
            _ => return None,
        }
    }
    Some(elements)
}

// Walk existing syntax only. Pinned term_show retains an open tail as NUMBER n+T,
// using precedence 1 for T. An annotation or malformed constructor ends the spine;
// displaying never evaluates a field or treats an open tail as Zero.
fn nat_spine(mut value: &Term) -> Option<(usize, &Term)> {
    let mut count = 0_usize;
    while let Term::Ctr { name, args } = value {
        if name != "Succ" {
            break;
        }
        let [tail] = args.as_slice() else {
            break;
        };
        count = count.checked_add(1)?;
        value = tail;
    }
    (count != 0 || empty_constructor(value, "Zero")).then_some((count, value))
}

fn empty_constructor(value: &Term, expected: &str) -> bool {
    matches!(value, Term::Ctr { name, args } if name == expected && args.is_empty())
}

fn open_nat(value: &Term) -> bool {
    nat_spine(value).is_some_and(|(_, tail)| !empty_constructor(tail, "Zero"))
}

fn infix_operand(f: &mut fmt::Formatter<'_>, value: &Term, scope: PrintScope<'_>) -> fmt::Result {
    // Arrow domains and quantitative-meet operands use upstream precedence 2.
    if matches!(value, Term::All { .. })
        || (scope.diagnostic && matches!(value, Term::Min(..)))
        || open_nat(value)
        || open_list(value)
        || refined_datatype(value)
    {
        write!(f, "({})", within(value, scope))
    } else {
        write!(f, "{}", within(value, scope))
    }
}

fn refined_datatype(value: &Term) -> bool {
    // Pinned ADT exclusions have precedence 1. Inspect syntax only: printing
    // cannot evaluate parameters or change the ordered semantic exclusion list.
    matches!(value, Term::Adt { excluded, .. } if !excluded.is_empty())
}

fn complete_list(mut value: &Term) -> bool {
    while let Term::Ctr { name, args } = value {
        if name != "Con" {
            break;
        }
        let [_, tail] = args.as_slice() else {
            break;
        };
        value = tail;
    }
    empty_constructor(value, "Nil")
}

// A non-Nil tail remains explicit. This inspects syntax only; constructor
// fields are never evaluated to decide whether list sugar is applicable.
fn open_list(value: &Term) -> bool {
    matches!(value, Term::Ctr { name, args } if name == "Con" && args.len() == 2)
        && !complete_list(value)
}

fn cons_chain(f: &mut fmt::Formatter<'_>, mut value: &Term, scope: PrintScope<'_>) -> fmt::Result {
    while let Term::Ctr { name, args } = value {
        if name != "Con" || args.len() != 2 {
            break;
        }
        let head = args[0].as_ref();
        if matches!(
            head,
            Term::Lam { .. }
                | Term::Let { .. }
                | Term::Rwt { .. }
                | Term::All { .. }
                | Term::Min(_, _)
        ) || open_nat(head)
            || open_list(head)
            || refined_datatype(head)
        {
            write!(f, "({})", within(head, scope))?;
        } else {
            write!(f, "{}", within(head, scope))?;
        }
        f.write_str(" <> ")?;
        value = &args[1];
    }
    match_branch(f, value, scope)
}

fn list_literal(
    f: &mut fmt::Formatter<'_>,
    mut value: &Term,
    scope: PrintScope<'_>,
) -> fmt::Result {
    f.write_str("[")?;
    let mut first = true;
    while let Term::Ctr { args, .. } = value {
        let [head, tail] = args.as_slice() else {
            break;
        };
        if !first {
            f.write_str(", ")?;
        }
        argument(f, head, scope)?;
        value = tail;
        first = false;
    }
    f.write_str("]")
}

fn tuple_literal(
    f: &mut fmt::Formatter<'_>,
    mut value: &Term,
    scope: PrintScope<'_>,
) -> fmt::Result {
    f.write_str("(")?;
    while let Term::Ctr { name, args } = value {
        if name != "Tuple" {
            break;
        }
        let [head, tail] = args.as_slice() else {
            break;
        };
        argument(f, head, scope)?;
        f.write_str(", ")?;
        value = tail;
    }
    argument(f, value, scope)?;
    f.write_str(")")
}

// Recognize the pinned term_show numeric spelling only for complete structural
// words. A partial, malformed or differently named constructor keeps its syntax;
// printing never evaluates a field or drops a nonliteral tail.
fn literal_word(mut word: &TermRef) -> Option<u32> {
    fn strip(mut value: &TermRef) -> &Term {
        while let Term::Ann(inner, _) = value.as_ref() {
            value = inner;
        }
        value.as_ref()
    }
    let mut value = 0_u32;
    for bit in 0..32 {
        let Term::Ctr { name, args } = strip(word) else {
            return None;
        };
        let [head, tail] = args.as_slice() else {
            return None;
        };
        if name != "WCon" {
            return None;
        }
        let Term::Ctr { name, args } = strip(head) else {
            return None;
        };
        if !args.is_empty() || !matches!(name.as_str(), "True" | "False") {
            return None;
        }
        if name == "True" {
            value |= 1 << bit;
        }
        word = tail;
    }
    matches!(strip(word), Term::Ctr { name, args } if name == "WNil" && args.is_empty())
        .then_some(value)
}

fn application(f: &mut fmt::Formatter<'_>, value: &Term, scope: PrintScope<'_>) -> fmt::Result {
    // Pinned term_show un-applies only the function spine, then prints its
    // arguments in source order. An annotation is a boundary, not another App.
    let mut head = value;
    let mut arguments = Vec::new();
    while let Term::App(function, argument) = head {
        arguments.push(argument.as_ref());
        head = function;
    }
    // The head uses precedence 2; lambda/let/rewrite/arrow heads need grouping.
    if matches!(
        head,
        Term::Lam { .. } | Term::Let { .. } | Term::Rwt { .. } | Term::All { .. }
    ) || (scope.diagnostic && matches!(head, Term::Min(..)))
        || open_nat(head)
        || open_list(head)
        || refined_datatype(head)
    {
        write!(f, "({})", within(head, scope))?;
    } else {
        write!(f, "{}", within(head, scope))?;
    }
    f.write_str("(")?;
    for (index, value) in arguments.into_iter().rev().enumerate() {
        if index != 0 {
            f.write_str(", ")?;
        }
        argument(f, value, scope)?;
    }
    f.write_str(")")
}

fn argument(f: &mut fmt::Formatter<'_>, value: &Term, scope: PrintScope<'_>) -> fmt::Result {
    // Arguments and literal sequence elements use upstream precedence 0.
    if matches!(value, Term::Let { .. }) {
        write!(f, "({})", within(value, scope))
    } else {
        write!(f, "{}", within(value, scope))
    }
}

fn rewrite(
    f: &mut fmt::Formatter<'_>,
    evidence: &Term,
    motive: &Term,
    body: &Term,
    scope: PrintScope<'_>,
) -> fmt::Result {
    // A surface rewrite stores two motive binders: the endpoint `_`, then the
    // optional equation name. Print under them without evaluating the motive.
    f.write_str("%")?;
    let outer = without_annotations(motive);
    let inner = if let Term::Lam { body, .. } = outer {
        without_annotations(body)
    } else {
        outer
    };
    let mut motive_scope = scope.binders.to_vec();
    let printed_motive = if let (
        Term::Lam {
            name: endpoint,
            id: endpoint_id,
            ..
        },
        Term::Lam { name, id, body },
    ) = (outer, inner)
    {
        motive_scope.push((*endpoint_id, endpoint));
        motive_scope.push((*id, name));
        if !name.is_empty() {
            write!(f, "{name}@")?;
        }
        body.as_ref()
    } else {
        motive
    };
    match_branch(f, evidence, scope)?;
    f.write_str(" : ")?;
    match_branch(f, printed_motive, scope.with_binders(&motive_scope))?;
    write!(f, "; {}", within(body, scope))
}

fn without_annotations(mut value: &Term) -> &Term {
    while let Term::Ann(inner, _) = value {
        value = inner;
    }
    value
}

fn match_branch(f: &mut fmt::Formatter<'_>, value: &Term, scope: PrintScope<'_>) -> fmt::Result {
    // Match branches use upstream precedence 1. These low-precedence forms
    // need parentheses; nested matches retain their own explicit delimiters.
    if matches!(
        value,
        Term::Lam { .. } | Term::Let { .. } | Term::Rwt { .. }
    ) {
        write!(f, "({})", within(value, scope))
    } else {
        write!(f, "{}", within(value, scope))
    }
}

fn list(f: &mut fmt::Formatter<'_>, values: &[TermRef], scope: PrintScope<'_>) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{}", within(x, scope))?;
    }
    Ok(())
}

#[cfg(test)]
mod inspection_tests {
    use super::*;

    #[test]
    fn deferred_holes_keep_input_limits_and_identifier_discovery() {
        let hole = term(Term::Hole("pending".into()));
        let value = term(Term::App(
            term(Term::Var {
                name: "later".into(),
                id: 987,
            }),
            Rc::clone(&hole),
        ));
        let checked = inspect_for_check(&value).unwrap();
        assert_eq!(checked.highest, 987);
        assert_eq!(checked.hole.as_deref(), Some("pending"));
        assert!(
            inspect(&value)
                .unwrap_err()
                .to_string()
                .contains("unfinished proof hole")
        );

        let boundary = (0..128).fold(Rc::clone(&hole), |body, _| term(Term::Typ(body)));
        assert_eq!(
            inspect_for_check(&boundary).unwrap().hole.as_deref(),
            Some("pending")
        );
        let deep = term(Term::Typ(boundary));
        assert!(
            inspect_for_check(&deep)
                .err()
                .unwrap()
                .to_string()
                .contains("input nesting limit")
        );

        let wide = term(Term::Ctr {
            name: "Wide".into(),
            args: vec![hole; 2_000_000],
        });
        assert!(
            inspect_for_check(&wide)
                .err()
                .unwrap()
                .to_string()
                .contains("input size limit")
        );
    }
}

#[cfg(test)]
mod automatic_cleanup_tests {
    use super::*;

    fn deep(depth: usize) -> TermRef {
        let mut value = term(Term::Rfl);
        for _ in 0..depth {
            value = term(Term::Typ(value));
        }
        value
    }

    struct Cleanup(Option<TermRef>);

    impl Drop for Cleanup {
        fn drop(&mut self) {
            if let Some(value) = self.0.take() {
                release_term_iteratively(value);
            }
        }
    }

    #[test]
    fn substitution_partial_result_releases_automatically_on_unwind() {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let trigger = term(Term::Hole("fault marker".into()));
                let source = Cleanup(Some(term(Term::Ctr {
                    name: "Pair".into(),
                    args: vec![deep(20_000), Rc::clone(&trigger)],
                })));
                let replacement = term(Term::Rfl);
                SUBSTITUTION_TEST_FAULT.with(|fault| fault.set(Rc::as_ptr(&trigger)));
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    substitute(source.0.as_ref().unwrap(), 999, &replacement)
                }));
                if let Ok(value) = result {
                    release_term_iteratively(value);
                    panic!("fault must occur after the first child");
                }
                let output = SUBSTITUTION_TEST_RESULT.with(|value| value.borrow_mut().take());
                assert!(
                    output.unwrap().upgrade().is_none(),
                    "partially built output must be released"
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn mapped_partial_results_release_on_error_and_unwind() {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let inputs = [term(Term::Rfl), term(Term::Rfl), term(Term::Rfl)];
                for panics in [false, true] {
                    let mut value = Cleanup(Some(deep(20_000)));
                    let weak = Rc::downgrade(value.0.as_ref().unwrap());
                    let mut calls = 0;
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        try_map_terms(&inputs, |_| {
                            calls += 1;
                            if calls == 1 {
                                return Ok(value.0.take().unwrap());
                            }
                            assert!(!panics, "deliberate mapping failure");
                            Err("second child fails")
                        })
                    }));
                    if panics {
                        match result {
                            Err(_) => {}
                            Ok(Err(error)) => panic!("expected panic, got error: {error}"),
                            Ok(Ok(output)) => {
                                for term in output {
                                    release_term_iteratively(term);
                                }
                                panic!("expected panic");
                            }
                        }
                    } else {
                        match result.unwrap() {
                            Err(error) => assert_eq!(error, "second child fails"),
                            Ok(output) => {
                                for term in output {
                                    release_term_iteratively(term);
                                }
                                panic!("expected second-child failure");
                            }
                        }
                    }
                    assert_eq!(calls, 2, "the later child must not run");
                    assert!(weak.upgrade().is_none(), "partial output must be released");
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn mapped_success_keeps_order_and_reference_identity() {
        let first = term(Term::Ref("first".into()));
        let second = term(Term::Ref("second".into()));
        let inputs = [Rc::clone(&first), Rc::clone(&second)];
        let output = try_map_terms(&inputs, |value| Ok::<_, ()>(Rc::clone(value))).unwrap();
        assert!(Rc::ptr_eq(&output[0], &first));
        assert!(Rc::ptr_eq(&output[1], &second));
        assert_eq!(Rc::strong_count(&first), 3);
        assert_eq!(Rc::strong_count(&second), 3);
    }

    #[test]
    fn mapped_failure_preserves_other_shared_owners() {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let original = Cleanup(Some(deep(20_000)));
                let retained = Cleanup(Some(Rc::clone(original.0.as_ref().unwrap())));
                let weak = Rc::downgrade(original.0.as_ref().unwrap());
                let inputs = [term(Term::Rfl), term(Term::Rfl)];
                let mut calls = 0;
                let result = try_map_terms(&inputs, |_| {
                    calls += 1;
                    if calls == 1 {
                        Ok(Rc::clone(original.0.as_ref().unwrap()))
                    } else {
                        Err("second child fails")
                    }
                });
                assert!(matches!(result, Err("second child fails")));
                assert_eq!(Rc::strong_count(original.0.as_ref().unwrap()), 2);
                drop(retained);
                assert!(weak.upgrade().is_some());
                drop(original);
                assert!(weak.upgrade().is_none());
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod diagnostic_precedence_tests {
    use super::*;

    #[derive(facet::Facet)]
    struct Case {
        source: String,
        expected: String,
        legacy: String,
    }

    #[test]
    fn diagnostic_meet_precedence_preserves_graph_and_public_display() {
        let cases: Vec<Case> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-meet-diagnostic-precedence/printer-cases.json"
        ))
        .unwrap();
        assert_eq!(cases.len(), 17);
        for case in cases {
            let value = crate::syntax::parse_term(&case.source).unwrap();
            let identity = Rc::as_ptr(&value);
            let owners = Rc::strong_count(&value);
            let structure = format!("{value:?}");
            for _ in 0..2 {
                assert_eq!(
                    diagnostic(&value, &[]).to_string(),
                    case.expected,
                    "{}",
                    case.source
                );
                assert_eq!(value.to_string(), case.legacy, "{}", case.source);
                assert_eq!(Rc::as_ptr(&value), identity);
                assert_eq!(Rc::strong_count(&value), owners);
                assert_eq!(format!("{value:?}"), structure);
            }
        }
    }

    #[test]
    fn diagnostic_meet_scope_preserves_shadowed_binders_and_guard_boundaries() {
        let first = term(Term::Var {
            name: "q".into(),
            id: 7,
        });
        let second = term(Term::Var {
            name: "q".into(),
            id: 19,
        });
        let meet = term(Term::Min(Rc::clone(&first), Rc::clone(&second)));
        let scope = [(7, "q"), (19, "q")];
        assert_eq!(diagnostic(&meet, &scope).to_string(), "q^0 <&> q");
        assert_eq!(scoped(&meet, &scope).to_string(), "(q^0 <&> q)");
        let body = term(Term::Lam {
            name: "q".into(),
            id: 23,
            body: Rc::clone(&meet),
        });
        assert_eq!(diagnostic(&body, &scope).to_string(), "q => q^0 <&> q^1");
        assert_eq!(scoped(&body, &scope).to_string(), "q => (q^0 <&> q^1)");
        let boundary = (0..127).fold(meet, |body, _| term(Term::Typ(body)));
        assert!(inspect(&boundary).is_ok());
        let original = format!("{boundary:?}");
        assert!(
            diagnostic(&boundary, &scope)
                .to_string()
                .contains("q^0 <&> q")
        );
        assert_eq!(format!("{boundary:?}"), original);
        assert!(
            inspect(&term(Term::Typ(boundary)))
                .unwrap_err()
                .to_string()
                .contains("input nesting limit")
        );
        assert_eq!(Rc::strong_count(&first), 2);
        assert_eq!(Rc::strong_count(&second), 2);
    }
}
