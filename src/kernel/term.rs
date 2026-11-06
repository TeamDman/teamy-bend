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
#[derive(Clone)]
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

// Keep the compact structural spelling while traversing children iteratively.
// Alternate pretty-debug formatting uses the same compact layout.
impl fmt::Debug for Term {
    #[expect(
        clippy::too_many_lines,
        reason = "All term variants use one explicit debug traversal."
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        enum Task<'a> {
            Term(&'a Term),
            Text(&'static str),
            Str(&'a str),
            Number(usize),
            Quant(Quant),
            Strings(&'a [String]),
        }
        use Task::Number;
        use Task::Quant as Q;
        use Task::Str;
        use Task::Term as Node;
        use Task::Text;
        let mut tasks = vec![Node(self)];
        while let Some(task) = tasks.pop() {
            match task {
                Text(s) => f.write_str(s)?,
                Str(s) => write!(f, "{s:?}")?,
                Number(n) => write!(f, "{n}")?,
                Q(q) => write!(f, "{q:?}")?,
                Task::Strings(v) => write!(f, "{v:?}")?,
                Node(value) => {
                    let mut out = Vec::new();
                    match value {
                        Term::Var { name, id } => out.extend([
                            Text("Var { name: "),
                            Str(name),
                            Text(", id: "),
                            Number(*id),
                            Text(" }"),
                        ]),
                        Term::Ref(s) => out.extend([Text("Ref("), Str(s), Text(")")]),
                        Term::GpuRef(s) => out.extend([Text("GpuRef("), Str(s), Text(")")]),
                        Term::Typ(a) => out.extend([Text("Typ("), Node(a), Text(")")]),
                        Term::Qnt => out.push(Text("Qnt")),
                        Term::Qua(q) => out.extend([Text("Qua("), Q(*q), Text(")")]),
                        Term::Min(a, b) => {
                            out.extend([Text("Min("), Node(a), Text(", "), Node(b), Text(")")]);
                        }
                        Term::All {
                            quant,
                            name,
                            id,
                            domain,
                            body,
                        } => out.extend([
                            Text("All { quant: "),
                            Q(*quant),
                            Text(", name: "),
                            Str(name),
                            Text(", id: "),
                            Number(*id),
                            Text(", domain: "),
                            Node(domain),
                            Text(", body: "),
                            Node(body),
                            Text(" }"),
                        ]),
                        Term::Lam { name, id, body } => out.extend([
                            Text("Lam { name: "),
                            Str(name),
                            Text(", id: "),
                            Number(*id),
                            Text(", body: "),
                            Node(body),
                            Text(" }"),
                        ]),
                        Term::App(a, b) => {
                            out.extend([Text("App("), Node(a), Text(", "), Node(b), Text(")")]);
                        }
                        Term::Adt {
                            name,
                            args,
                            excluded,
                        } => {
                            out.extend([Text("Adt { name: "), Str(name), Text(", args: [")]);
                            for (i, arg) in args.iter().enumerate() {
                                if i != 0 {
                                    out.push(Text(", "));
                                }
                                out.push(Node(arg));
                            }
                            out.extend([
                                Text("], excluded: "),
                                Task::Strings(excluded),
                                Text(" }"),
                            ]);
                        }
                        Term::Ctr { name, args } => {
                            out.extend([Text("Ctr { name: "), Str(name), Text(", args: [")]);
                            for (i, arg) in args.iter().enumerate() {
                                if i != 0 {
                                    out.push(Text(", "));
                                }
                                out.push(Node(arg));
                            }
                            out.push(Text("] }"));
                        }
                        Term::Mat {
                            constructor,
                            arm,
                            fallback,
                        } => out.extend([
                            Text("Mat { constructor: "),
                            Str(constructor),
                            Text(", arm: "),
                            Node(arm),
                            Text(", fallback: "),
                            Node(fallback),
                            Text(" }"),
                        ]),
                        Term::Efq => out.push(Text("Efq")),
                        Term::Eql { left, right, ty } => out.extend([
                            Text("Eql { left: "),
                            Node(left),
                            Text(", right: "),
                            Node(right),
                            Text(", ty: "),
                            Node(ty),
                            Text(" }"),
                        ]),
                        Term::Rfl => out.push(Text("Rfl")),
                        Term::Rwt {
                            evidence,
                            motive,
                            body,
                        } => out.extend([
                            Text("Rwt { evidence: "),
                            Node(evidence),
                            Text(", motive: "),
                            Node(motive),
                            Text(", body: "),
                            Node(body),
                            Text(" }"),
                        ]),
                        Term::Hole(s) => out.extend([Text("Hole("), Str(s), Text(")")]),
                        Term::Ann(a, b) => {
                            out.extend([Text("Ann("), Node(a), Text(", "), Node(b), Text(")")]);
                        }
                        Term::Let { bindings, body } => {
                            out.push(Text("Let { bindings: ["));
                            for (i, b) in bindings.iter().enumerate() {
                                if i != 0 {
                                    out.push(Text(", "));
                                }
                                out.extend([
                                    Text("LetBinding { quant: "),
                                    Q(b.quant),
                                    Text(", name: "),
                                    Str(&b.name),
                                    Text(", id: "),
                                    Number(b.id),
                                    Text(", value: "),
                                    Node(&b.value),
                                    Text(" }"),
                                ]);
                            }
                            out.extend([Text("], body: "), Node(body), Text(" }")]);
                        }
                    }
                    tasks.extend(out.into_iter().rev());
                }
            }
        }
        Ok(())
    }
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

// An owned Term implements Drop; callers extracting fields must borrow or clone
// them instead of moving them directly out of the enum.
fn is_leaf(value: &Term) -> bool {
    matches!(
        value,
        Term::Var { .. }
            | Term::Ref(_)
            | Term::GpuRef(_)
            | Term::Qnt
            | Term::Qua(_)
            | Term::Efq
            | Term::Rfl
            | Term::Hole(_)
    )
}

fn has_nonleaf_child(value: &Term) -> bool {
    let nonleaf = |child: &TermRef| !is_leaf(child);
    match value {
        Term::Var { .. }
        | Term::Ref(_)
        | Term::GpuRef(_)
        | Term::Qnt
        | Term::Qua(_)
        | Term::Efq
        | Term::Rfl
        | Term::Hole(_) => false,
        Term::Typ(child) | Term::Lam { body: child, .. } => nonleaf(child),
        Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => nonleaf(a) || nonleaf(b),
        Term::All { domain, body, .. } => nonleaf(domain) || nonleaf(body),
        Term::Adt { args, .. } | Term::Ctr { args, .. } => args.iter().any(nonleaf),
        Term::Mat { arm, fallback, .. } => nonleaf(arm) || nonleaf(fallback),
        Term::Eql { left, right, ty } => nonleaf(left) || nonleaf(right) || nonleaf(ty),
        Term::Rwt {
            evidence,
            motive,
            body,
        } => nonleaf(evidence) || nonleaf(motive) || nonleaf(body),
        Term::Let { bindings, body } => bindings.iter().any(|b| nonleaf(&b.value)) || nonleaf(body),
    }
}

fn take_edges(value: &mut Term, empty: &TermRef, pending: &mut Vec<TermRef>) {
    // Reserve before detaching any edge. Cleanup cannot return an allocation
    // error, and unwinding with a partially detached graph can itself overflow
    // the host stack. Match Rust's ordinary allocation-failure behavior and
    // terminate rather than continue with an unbounded recursive cleanup.
    let edges = match &*value {
        Term::Var { .. }
        | Term::Ref(_)
        | Term::GpuRef(_)
        | Term::Qnt
        | Term::Qua(_)
        | Term::Efq
        | Term::Rfl
        | Term::Hole(_) => 0,
        Term::Typ(_) | Term::Lam { .. } => 1,
        Term::Min(..) | Term::App(..) | Term::Ann(..) | Term::All { .. } | Term::Mat { .. } => 2,
        Term::Eql { .. } | Term::Rwt { .. } => 3,
        Term::Adt { args, .. } | Term::Ctr { args, .. } => args.len(),
        Term::Let { bindings, .. } => match bindings.len().checked_add(1) {
            Some(edges) => edges,
            None => std::process::abort(),
        },
    };
    if pending.try_reserve(edges).is_err() {
        std::process::abort();
    }
    let mut take = |edge: &mut TermRef| pending.push(std::mem::replace(edge, Rc::clone(empty)));
    match value {
        Term::Var { .. }
        | Term::Ref(_)
        | Term::GpuRef(_)
        | Term::Qnt
        | Term::Qua(_)
        | Term::Efq
        | Term::Rfl
        | Term::Hole(_) => {}
        Term::Typ(child) | Term::Lam { body: child, .. } => take(child),
        Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => {
            take(a);
            take(b);
        }
        Term::All { domain, body, .. } => {
            take(domain);
            take(body);
        }
        Term::Adt { args, .. } | Term::Ctr { args, .. } => pending.append(args),
        Term::Mat { arm, fallback, .. } => {
            take(arm);
            take(fallback);
        }
        Term::Eql { left, right, ty } => {
            take(left);
            take(right);
            take(ty);
        }
        Term::Rwt {
            evidence,
            motive,
            body,
        } => {
            take(evidence);
            take(motive);
            take(body);
        }
        Term::Let { bindings, body } => {
            for binding in bindings {
                take(&mut binding.value);
            }
            take(body);
        }
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        // Leaf-only children can be dropped by Rust without recursive chains.
        // This also makes destruction of already-drained nodes allocation-free.
        if !has_nonleaf_child(self) {
            return;
        }
        let empty = Rc::new(Term::Rfl);
        let mut pending = Vec::new();
        take_edges(self, &empty, &mut pending);
        while let Some(value) = pending.pop() {
            if let Ok(mut value) = Rc::try_unwrap(value) {
                take_edges(&mut value, &empty, &mut pending);
            }
        }
    }
}

/// Release an owned graph. Ordinary last-owner drop is also iterative.
pub fn release_term_iteratively(root: TermRef) {
    drop(root);
}

// A single intermediate result needs the same unwind protection as a pending
// vector, without allocating a vector for every provenance-aware substitution.
pub(super) struct PendingTerm(Option<TermRef>);

impl PendingTerm {
    pub(super) fn new(value: TermRef) -> Self {
        Self(Some(value))
    }

    pub(super) fn get(&self) -> &TermRef {
        self.0.as_ref().expect("pending term exists")
    }

    pub(super) fn take(mut self) -> TermRef {
        self.0.take().expect("pending term exists")
    }
}

impl Drop for PendingTerm {
    fn drop(&mut self) {
        if let Some(value) = self.0.take() {
            release_term_iteratively(value);
        }
    }
}

// Intermediate owned results are kept together until a later sibling succeeds.
// Successful callers take the vector out, leaving the guard empty.
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

/// Capture-free substitution for terms whose binder IDs are globally unique.
/// Traversal uses explicit heap frames; ordinary destruction of the returned
/// `Rc<Term>` also releases its uniquely owned graph iteratively.
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
        iterative_display(f, self.value, self.scope)
    }
}

#[derive(Clone, Copy)]
enum PrintContext {
    Root,
    Branch,
    Argument,
    Infix,
    Head,
    ConsHead,
}

enum PrintTask<'a> {
    Render(&'a Term, PrintContext),
    Raw(&'a Term),
    Text(&'a str),
    Owned(String),
    Escaped(u32, char),
    Push(usize, &'a str),
    Pop(usize),
}

fn group(value: &Term, context: PrintContext, diagnostic: bool) -> bool {
    match context {
        PrintContext::Root => false,
        PrintContext::Branch => matches!(
            value,
            Term::Lam { .. } | Term::Let { .. } | Term::Rwt { .. }
        ),
        PrintContext::Argument => matches!(value, Term::Let { .. }),
        PrintContext::Infix => {
            matches!(value, Term::All { .. })
                || (diagnostic && matches!(value, Term::Min(..)))
                || open_nat(value)
                || open_list(value)
                || refined_datatype(value)
        }
        PrintContext::Head => {
            matches!(
                value,
                Term::Lam { .. } | Term::Let { .. } | Term::Rwt { .. } | Term::All { .. }
            ) || (diagnostic && matches!(value, Term::Min(..)))
                || open_nat(value)
                || open_list(value)
                || refined_datatype(value)
        }
        PrintContext::ConsHead => {
            matches!(
                value,
                Term::Lam { .. }
                    | Term::Let { .. }
                    | Term::Rwt { .. }
                    | Term::All { .. }
                    | Term::Min(..)
            ) || open_nat(value)
                || open_list(value)
                || refined_datatype(value)
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "One explicit stack implements all existing term-show branches."
)]
fn iterative_display<'a>(
    f: &mut fmt::Formatter<'_>,
    value: &'a Term,
    scope: PrintScope<'a>,
) -> fmt::Result {
    use PrintContext::Argument;
    use PrintContext::Branch;
    use PrintContext::ConsHead;
    use PrintContext::Head;
    use PrintContext::Infix;
    use PrintContext::Root;
    use PrintTask::Escaped;
    use PrintTask::Owned;
    use PrintTask::Pop;
    use PrintTask::Push;
    use PrintTask::Raw;
    use PrintTask::Render;
    use PrintTask::Text;
    let mut binders = scope.binders.to_vec();
    let mut tasks = vec![Render(value, Root)];
    while let Some(task) = tasks.pop() {
        match task {
            Text(text) => f.write_str(text)?,
            Owned(text) => f.write_str(&text)?,
            Escaped(value, quote) => write_character(f, value, quote)?,
            Push(id, name) => binders.push((id, name)),
            Pop(n) => binders.truncate(binders.len() - n),
            Render(value, context) => {
                if group(value, context, scope.diagnostic) {
                    f.write_str("(")?;
                    tasks.push(Text(")"));
                }
                tasks.push(Raw(value));
            }
            Raw(value) => {
                let mut sequence = Vec::new();
                match value {
                    Term::Var { name, id } => {
                        f.write_str(name)?;
                        if let Some(level) = binders.iter().position(|(bound, _)| bound == id)
                            && binders.iter().rposition(|(_, bound)| *bound == name) != Some(level)
                        {
                            write!(f, "^{level}")?;
                        }
                    }
                    Term::Ref(name) | Term::GpuRef(name) => {
                        f.write_str(name)?;
                        if binders.iter().any(|(_, bound)| *bound == name) {
                            f.write_str("^")?;
                        }
                        if matches!(value, Term::GpuRef(_)) {
                            f.write_str("!")?;
                        }
                    }
                    Term::Typ(g) => match g.as_ref() {
                        Term::Qua(Quant::Lone) => f.write_str("Type")?,
                        Term::Qua(Quant::Many) => f.write_str("Data")?,
                        _ => sequence.extend([Text("Kind("), Render(g, Root), Text(")")]),
                    },
                    Term::Qnt => f.write_str("Quant")?,
                    Term::Qua(q) => f.write_str(match q {
                        Quant::None => "&0",
                        Quant::Lone => "&1",
                        Quant::Many => "&2",
                    })?,
                    Term::Min(a, b) => {
                        if !scope.diagnostic {
                            sequence.push(Text("("));
                        }
                        sequence.extend([Render(a, Infix), Text(" <&> "), Render(b, Infix)]);
                        if !scope.diagnostic {
                            sequence.push(Text(")"));
                        }
                    }
                    Term::All {
                        quant,
                        name,
                        id,
                        domain,
                        body,
                    } => {
                        sequence.push(Owned(format!("@{quant}{name}:")));
                        sequence.extend([
                            Render(domain, Infix),
                            Text(" -> "),
                            Push(*id, name),
                            Render(body, Root),
                            Pop(1),
                        ]);
                    }
                    Term::Lam { name, id, body } => sequence.extend([
                        Text(name),
                        Text(" => "),
                        Push(*id, name),
                        Render(body, Root),
                        Pop(1),
                    ]),
                    Term::App(..) => {
                        let mut head = value;
                        let mut args = Vec::new();
                        while let Term::App(fun, arg) = head {
                            args.push(arg.as_ref());
                            head = fun;
                        }
                        sequence.push(Render(head, Head));
                        sequence.push(Text("("));
                        for (i, arg) in args.into_iter().rev().enumerate() {
                            if i != 0 {
                                sequence.push(Text(", "));
                            }
                            sequence.push(Render(arg, Argument));
                        }
                        sequence.push(Text(")"));
                    }
                    Term::Adt {
                        name,
                        args,
                        excluded,
                    } => {
                        sequence.push(Text(name));
                        if !args.is_empty() || !excluded.is_empty() {
                            sequence.push(Text("<"));
                            for (i, arg) in args.iter().enumerate() {
                                if i != 0 {
                                    sequence.push(Text(", "));
                                }
                                sequence.push(Render(arg, Root));
                            }
                            sequence.push(Text(">"));
                        }
                        for name in excluded {
                            sequence.extend([Text(" - "), Text(name), Text("{}")]);
                        }
                    }
                    Term::Ctr { name, args } => {
                        if let Some((count, tail)) = nat_spine(value) {
                            sequence.push(Owned(format!("{count}n")));
                            if !empty_constructor(tail, "Zero") {
                                sequence.extend([Text("+"), Render(tail, Branch)]);
                            }
                        } else if let Some(ch) = literal_character(value) {
                            sequence.extend([Text("'"), Escaped(ch, '\''), Text("'")]);
                        } else if let Some((prefix, quoted_tail)) = string_plan(value) {
                            let mut tail = value;
                            for _ in 0..prefix {
                                let Term::Ctr { args, .. } = tail else {
                                    unreachable!("validated string")
                                };
                                let [head, next] = args.as_slice() else {
                                    unreachable!("validated string arity")
                                };
                                sequence.extend([Text("SCon{"), Render(head, Root), Text(", ")]);
                                tail = next;
                            }
                            if quoted_tail {
                                sequence.push(Text("\""));
                                while let Term::Ctr { name, args } = tail {
                                    if name == "SNil" {
                                        break;
                                    }
                                    let [head, next] = args.as_slice() else {
                                        unreachable!("validated string tail")
                                    };
                                    sequence.push(Escaped(
                                        literal_character(head).expect("validated character"),
                                        '"',
                                    ));
                                    tail = next;
                                }
                                sequence.push(Text("\""));
                            } else {
                                sequence.push(Render(tail, Root));
                            }
                            for _ in 0..prefix {
                                sequence.push(Text("}"));
                            }
                        } else if complete_list(value) {
                            sequence.push(Text("["));
                            let mut tail = value;
                            let mut first = true;
                            while let Term::Ctr { args, .. } = tail {
                                let [head, next] = args.as_slice() else { break };
                                if !first {
                                    sequence.push(Text(", "));
                                }
                                sequence.push(Render(head, Argument));
                                tail = next;
                                first = false;
                            }
                            sequence.push(Text("]"));
                        } else if open_list(value) {
                            let mut tail = value;
                            while let Term::Ctr { name, args } = tail {
                                if name != "Con" || args.len() != 2 {
                                    break;
                                }
                                sequence.extend([Render(&args[0], ConsHead), Text(" <> ")]);
                                tail = &args[1];
                            }
                            sequence.push(Render(tail, Branch));
                        } else if name == "Tuple" && args.len() == 2 {
                            sequence.push(Text("("));
                            let mut tail = value;
                            while let Term::Ctr { name, args } = tail {
                                if name != "Tuple" {
                                    break;
                                }
                                let [head, next] = args.as_slice() else { break };
                                sequence.extend([Render(head, Argument), Text(", ")]);
                                tail = next;
                            }
                            sequence.extend([Render(tail, Argument), Text(")")]);
                        } else if matches!(name.as_str(), "ALeaf" | "ANode")
                            && let Some(elements) = array_elements(value)
                        {
                            sequence.push(Text("["));
                            for (i, element) in elements.into_iter().enumerate() {
                                if i != 0 {
                                    sequence.push(Text(", "));
                                }
                                sequence.push(Render(element, Argument));
                            }
                            sequence.push(Text("]"));
                        } else if name == "U32"
                            && let [word] = args.as_slice()
                            && let Some(number) = literal_word(word)
                        {
                            sequence.push(Owned(number.to_string()));
                        } else {
                            sequence.extend([Text(name), Text("{")]);
                            for (i, arg) in args.iter().enumerate() {
                                if i != 0 {
                                    sequence.push(Text(", "));
                                }
                                sequence.push(Render(arg, Root));
                            }
                            sequence.push(Text("}"));
                        }
                    }
                    Term::Mat { .. } => {
                        sequence.push(Text("\\{"));
                        let mut current = value;
                        let mut first = true;
                        while let Term::Mat {
                            constructor,
                            arm,
                            fallback,
                        } = current
                        {
                            if !first {
                                sequence.push(Text("; "));
                            }
                            sequence.extend([Text(constructor), Text(": "), Render(arm, Branch)]);
                            current = fallback;
                            first = false;
                        }
                        if !matches!(current, Term::Efq) {
                            sequence.extend([Text("; "), Render(current, Branch)]);
                        }
                        sequence.push(Text("}"));
                    }
                    Term::Efq => f.write_str("\\{}")?,
                    Term::Eql { left, right, ty } => sequence.extend([
                        Text("{"),
                        Render(left, Branch),
                        Text(" == "),
                        Render(right, Branch),
                        Text(" : "),
                        Render(ty, Branch),
                        Text("}"),
                    ]),
                    Term::Rfl => f.write_str("{==}")?,
                    Term::Rwt {
                        evidence,
                        motive,
                        body,
                    } => {
                        sequence.push(Text("%"));
                        let outer = without_annotations(motive);
                        let inner = if let Term::Lam { body, .. } = outer {
                            without_annotations(body)
                        } else {
                            outer
                        };
                        let (printed, binds) = if let (
                            Term::Lam {
                                name: endpoint,
                                id: endpoint_id,
                                ..
                            },
                            Term::Lam { name, id, body },
                        ) = (outer, inner)
                        {
                            if !name.is_empty() {
                                sequence.extend([Text(name), Text("@")]);
                            }
                            (
                                body.as_ref(),
                                Some((*endpoint_id, endpoint.as_str(), *id, name.as_str())),
                            )
                        } else {
                            (motive.as_ref(), None)
                        };
                        sequence.extend([Render(evidence, Branch), Text(" : ")]);
                        if let Some((a, an, b, bn)) = binds {
                            sequence.extend([
                                Push(a, an),
                                Push(b, bn),
                                Render(printed, Branch),
                                Pop(2),
                            ]);
                        } else {
                            sequence.push(Render(printed, Branch));
                        }
                        sequence.extend([Text("; "), Render(body, Root)]);
                    }
                    Term::Hole(name) => sequence.extend([Text("?"), Text(name)]),
                    Term::Ann(x, t) => sequence.extend([
                        Text("{"),
                        Render(x, Root),
                        Text(" : "),
                        Render(t, Root),
                        Text("}"),
                    ]),
                    Term::Let { bindings, body } => {
                        for b in bindings {
                            sequence.extend([
                                Owned(b.quant.to_string()),
                                Text(&b.name),
                                Text(" = "),
                                Render(&b.value, Root),
                                Text("; "),
                            ]);
                        }
                        for b in bindings {
                            sequence.push(Push(b.id, &b.name));
                        }
                        sequence.push(Render(body, Root));
                        sequence.push(Pop(bindings.len()));
                    }
                }
                tasks.extend(sequence.into_iter().rev());
            }
        }
    }
    Ok(())
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

fn without_annotations(mut value: &Term) -> &Term {
    while let Term::Ann(inner, _) = value {
        value = inner;
    }
    value
}

#[cfg(test)]
mod iterative_display_tests {
    use super::*;

    #[test]
    fn deep_display_and_formatter_error_are_stack_safe() {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let root = (0..20_000).fold(term(Term::Rfl), |child, _| term(Term::Typ(child)));
                let weak = Rc::downgrade(&root);
                let rendered = root.to_string();
                assert!(rendered.starts_with("Kind(Kind("));
                assert!(rendered.ends_with(")"));
                assert_eq!(rendered.matches("Kind(").count(), 20_000);
                let debug = format!("{root:?}");
                assert_eq!(debug.matches("Typ(").count(), 20_000);
                struct FailAfter(usize);
                impl fmt::Write for FailAfter {
                    fn write_str(&mut self, _: &str) -> fmt::Result {
                        self.0 = self.0.checked_sub(1).ok_or(fmt::Error)?;
                        Ok(())
                    }
                }
                assert!(fmt::write(&mut FailAfter(12), format_args!("{root}")).is_err());
                drop(root);
                assert!(weak.upgrade().is_none());

                let root = (0..10_000).fold(term(Term::Rfl), |arm, i| {
                    term(Term::Mat {
                        constructor: format!("C{i}"),
                        arm,
                        fallback: term(Term::Efq),
                    })
                });
                let weak = Rc::downgrade(&root);
                let rendered = root.to_string();
                assert!(rendered.starts_with("\\{C9999: "));
                assert!(rendered.contains("C0: {==}"));
                drop(root);
                assert!(weak.upgrade().is_none());
            })
            .unwrap()
            .join()
            .unwrap();
    }
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
