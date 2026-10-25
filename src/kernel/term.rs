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
#[must_use]
pub fn substitute(value: &TermRef, target: usize, replacement: &TermRef) -> TermRef {
    let sub = |x: &TermRef| substitute(x, target, replacement);
    match value.as_ref() {
        Term::Var { id, .. } if *id == target => Rc::clone(replacement),
        Term::Var { .. }
        | Term::Ref(_)
        | Term::GpuRef(_)
        | Term::Qnt
        | Term::Qua(_)
        | Term::Efq
        | Term::Rfl
        | Term::Hole(_) => Rc::clone(value),
        Term::Typ(g) => term(Term::Typ(sub(g))),
        Term::Min(a, b) => term(Term::Min(sub(a), sub(b))),
        Term::All {
            quant,
            name,
            id,
            domain,
            body,
        } => term(Term::All {
            quant: *quant,
            name: name.clone(),
            id: *id,
            domain: sub(domain),
            body: if *id == target {
                Rc::clone(body)
            } else {
                sub(body)
            },
        }),
        Term::Lam { name, id, body } => {
            if *id == target {
                Rc::clone(value)
            } else {
                term(Term::Lam {
                    name: name.clone(),
                    id: *id,
                    body: sub(body),
                })
            }
        }
        Term::App(f, x) => map_application(value, f, x, sub),
        Term::Adt {
            name,
            args,
            excluded,
        } => term(Term::Adt {
            name: name.clone(),
            args: args.iter().map(sub).collect(),
            excluded: excluded.clone(),
        }),
        Term::Ctr { name, args } => map_constructor(value, name, args, sub),
        Term::Mat {
            constructor,
            arm,
            fallback,
        } => term(Term::Mat {
            constructor: constructor.clone(),
            arm: sub(arm),
            fallback: sub(fallback),
        }),
        Term::Eql { left, right, ty } => term(Term::Eql {
            left: sub(left),
            right: sub(right),
            ty: sub(ty),
        }),
        Term::Rwt {
            evidence,
            motive,
            body,
        } => term(Term::Rwt {
            evidence: sub(evidence),
            motive: sub(motive),
            body: sub(body),
        }),
        Term::Ann(x, t) => term(Term::Ann(sub(x), sub(t))),
        Term::Let { bindings, body } => term(Term::Let {
            bindings: bindings
                .iter()
                .map(|b| LetBinding {
                    quant: b.quant,
                    name: b.name.clone(),
                    id: b.id,
                    value: sub(&b.value),
                })
                .collect(),
            body: if bindings.iter().any(|b| b.id == target) {
                Rc::clone(body)
            } else {
                sub(body)
            },
        }),
    }
}

fn map_application(
    value: &TermRef,
    f: &TermRef,
    x: &TermRef,
    mut transform: impl FnMut(&TermRef) -> TermRef,
) -> TermRef {
    // An unrelated binder must not copy an already shared computation.
    let next_f = transform(f);
    let next_x = transform(x);
    if Rc::ptr_eq(f, &next_f) && Rc::ptr_eq(x, &next_x) {
        Rc::clone(value)
    } else {
        term(Term::App(next_f, next_x))
    }
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
