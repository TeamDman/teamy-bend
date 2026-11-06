// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use crate::kernel::LetBinding;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::origins::MatchSite;
use crate::kernel::origins::TermOrigins;
use crate::kernel::origins::substitute_with_origins;
use crate::kernel::term;
use std::rc::Rc;

// A parsed occurrence carries diagnostic-only syntax information beside the
// exact shared core value. In particular, macro identity still uses its Rc.
#[derive(Clone, Debug)]
pub(super) struct Occurrence {
    pub value: TermRef,
    pub source: Option<usize>,
    pub bare_name: bool,
}

impl From<TermRef> for Occurrence {
    fn from(value: TermRef) -> Self {
        Self {
            value,
            source: None,
            bare_name: false,
        }
    }
}

impl std::ops::Deref for Occurrence {
    type Target = TermRef;
    fn deref(&self) -> &TermRef {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub(super) struct Variable {
    pub name: String,
    pub id: usize,
}

impl Variable {
    pub fn term(&self) -> TermRef {
        term(Term::Var {
            name: self.name.clone(),
            id: self.id,
        })
    }
    fn lambda(&self, body: TermRef) -> TermRef {
        term(Term::Lam {
            name: self.name.clone(),
            id: self.id,
            body,
        })
    }
}

#[derive(Debug)]
pub(super) enum Pattern {
    Variable(Variable),
    Constructor(String, Vec<Pattern>, usize),
}

// Pattern trees can be much deeper than the syntax that names them: string
// patterns expand characters into word constructors. Clone and destruction
// use explicit work instead of putting every constructor on the host stack.
impl Clone for Pattern {
    fn clone(&self) -> Self {
        enum Work<'a> {
            Visit(&'a Pattern),
            Constructor(&'a str, usize, usize),
        }

        let mut pending = vec![Work::Visit(self)];
        let mut values = Vec::<Pattern>::new();
        while let Some(work) = pending.pop() {
            match work {
                Work::Visit(Self::Variable(variable)) => {
                    values.push(Self::Variable(variable.clone()));
                }
                Work::Visit(Self::Constructor(name, fields, origin)) => {
                    pending.push(Work::Constructor(name, fields.len(), *origin));
                    pending.extend(fields.iter().rev().map(Work::Visit));
                }
                Work::Constructor(name, count, origin) => {
                    let start = values.len() - count;
                    let fields = values.split_off(start);
                    values.push(Self::Constructor(name.to_owned(), fields, origin));
                }
            }
        }
        values.pop().expect("pattern clone has one root")
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        let Self::Constructor(_, fields, _) = self else {
            return;
        };
        let mut pending = std::mem::take(fields);
        while let Some(mut value) = pending.pop() {
            if let Self::Constructor(_, fields, _) = &mut value {
                pending.append(fields);
            }
        }
    }
}

impl Pattern {
    fn term(&self) -> TermRef {
        match self {
            Self::Variable(v) => v.term(),
            Self::Constructor(name, fields, _) => term(Term::Ctr {
                name: name.clone(),
                args: fields.iter().map(Self::term).collect(),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Row {
    pub patterns: Vec<Pattern>,
    pub body: Body,
}

// A surface occurrence keeps its own source anchor when substitution shares a
// replacement across several uses. This is discarded with the surface body;
// it adds no core term edge, identity, evaluator work, or serialized field.
#[derive(Clone, Debug)]
pub(super) struct SurfaceTerm {
    value: TermRef,
    origin: usize,
    match_site: MatchSite,
    explicit_source: Option<usize>,
    bare_name: bool,
}

impl SurfaceTerm {
    pub(super) fn new(value: TermRef, origins: Option<&TermOrigins>) -> Self {
        let origin = origins.map_or_else(|| Rc::as_ptr(&value) as usize, |o| o.source(&value));
        Self {
            value,
            origin,
            match_site: MatchSite::Expression(origin),
            explicit_source: None,
            bare_name: false,
        }
    }

    pub(super) fn occurrence(value: Occurrence, origins: Option<&TermOrigins>) -> Self {
        let mut result = Self::new(value.value, origins);
        if let Some(source) = value.source {
            result.origin = source;
            result.match_site = MatchSite::Expression(source);
        }
        result.explicit_source = value.source;
        result.bare_name = value.bare_name;
        result
    }

    pub(super) fn into_occurrence(self) -> Occurrence {
        Occurrence {
            value: self.value,
            source: self.explicit_source,
            bare_name: self.bare_name,
        }
    }

    pub(super) fn statement_scrutinee(mut self) -> Self {
        self.match_site = MatchSite::Statement(self.origin);
        self
    }

    fn at_match(self, site: MatchSite, origins: Option<&TermOrigins>) -> Self {
        if let Some(origins) = origins {
            origins.surface_match(&self.value, site);
        }
        self
    }

    fn parent(value: TermRef, children: [Option<usize>; 3], origins: Option<&TermOrigins>) -> Self {
        if let Some(origins) = origins {
            origins.surface_children(&value, children);
        }
        Self::new(value, origins)
    }

    fn lambda(self, binder: &Variable, origins: Option<&TermOrigins>) -> Self {
        let value = binder.lambda(self.value);
        if let Some(origins) = origins {
            origins.binder_node(binder.id, &value);
            origins.occurrence_children(&value, [self.explicit_source]);
        }
        Self::parent(value, [Some(self.origin), None, None], origins)
    }

    fn matched(
        constructor: String,
        arm: Self,
        fallback: Self,
        site: Option<MatchSite>,
        constructor_origin: Option<usize>,
        origins: Option<&TermOrigins>,
    ) -> Self {
        let children = [Some(arm.origin), Some(fallback.origin), None];
        let value = term(Term::Mat {
            constructor,
            arm: arm.value,
            fallback: fallback.value,
        });
        if let Some(origins) = origins {
            origins.occurrence_children(&value, [arm.explicit_source, fallback.explicit_source]);
        }
        if let (Some(origins), Some(source)) = (origins, constructor_origin) {
            origins.match_constructor(&value, source);
        }
        let result = Self::parent(value, children, origins);
        if let Some(site) = site {
            result.at_match(site, origins)
        } else {
            result
        }
    }

    fn replace(&self, id: usize, replacement: &TermRef, origins: Option<&TermOrigins>) -> Self {
        Self {
            value: substitute_with_origins(&self.value, id, replacement, origins),
            origin: self.origin,
            match_site: self.match_site,
            explicit_source: self.explicit_source,
            bare_name: self.bare_name,
        }
    }
}

#[derive(Debug)]
pub(super) enum Body {
    // Internal placeholder while moving owned subtrees between work frames.
    Empty,
    Reply(SurfaceTerm),
    Local {
        patterns: Vec<Pattern>,
        quant: Quant,
        values: Vec<SurfaceTerm>,
        body: Box<Self>,
    },
    Match {
        scrutinees: Vec<SurfaceTerm>,
        rows: Vec<Row>,
    },
}

// Partially transformed values can own unique deep terms if a later child
// unwinds. Successful construction takes the vector out of this guard.
#[derive(Default)]
struct PendingSurface(Vec<SurfaceTerm>);

impl PendingSurface {
    fn take(&mut self) -> Vec<SurfaceTerm> {
        std::mem::take(&mut self.0)
    }
}

impl Drop for PendingSurface {
    fn drop(&mut self) {
        for value in std::mem::take(&mut self.0) {
            crate::kernel::release_term_iteratively(value.value);
        }
    }
}

impl Clone for Body {
    fn clone(&self) -> Self {
        self.map_terms(Clone::clone)
    }
}

impl Drop for Body {
    fn drop(&mut self) {
        fn drain(body: &mut Body, pending: &mut Vec<Body>) {
            match body {
                Body::Empty => {}
                Body::Reply(value) => {
                    // A shared root cannot be destroyed by this field's drop.
                    // For the last owner, replace it before iterative release.
                    let leaf = matches!(
                        value.value.as_ref(),
                        Term::Var { .. }
                            | Term::Ref(_)
                            | Term::GpuRef(_)
                            | Term::Qnt
                            | Term::Qua(_)
                            | Term::Efq
                            | Term::Rfl
                            | Term::Hole(_)
                    );
                    if !leaf && Rc::strong_count(&value.value) == 1 {
                        let root = std::mem::replace(&mut value.value, term(Term::Rfl));
                        crate::kernel::release_term_iteratively(root);
                    }
                }
                Body::Local { values, body, .. } => {
                    for value in std::mem::take(values) {
                        crate::kernel::release_term_iteratively(value.value);
                    }
                    if !matches!(body.as_ref(), Body::Empty) {
                        pending.push(std::mem::replace(body.as_mut(), Body::Empty));
                    }
                }
                Body::Match { scrutinees, rows } => {
                    for value in std::mem::take(scrutinees) {
                        crate::kernel::release_term_iteratively(value.value);
                    }
                    for row in std::mem::take(rows) {
                        pending.push(row.body);
                    }
                }
            }
        }

        let mut pending = Vec::new();
        drain(self, &mut pending);
        while let Some(mut body) = pending.pop() {
            drain(&mut body, &mut pending);
        }
    }
}

impl Body {
    pub fn with_reusable(mut self, bindings: &[(Variable, TermRef)]) -> Self {
        if bindings.is_empty() {
            return self;
        }
        let mut pending = vec![&mut self];
        while let Some(body) = pending.pop() {
            let descends = matches!(body, Self::Local { patterns, .. } if matches!(patterns.first(), Some(Pattern::Constructor(_, _, _))));
            if descends {
                let Self::Local { body, .. } = body else {
                    unreachable!()
                };
                pending.push(body.as_mut());
                continue;
            }
            if matches!(body, Self::Match { .. }) {
                let Self::Match { rows, .. } = body else {
                    unreachable!()
                };
                pending.extend(rows.iter_mut().rev().map(|row| &mut row.body));
                continue;
            }
            let original = std::mem::replace(body, Self::Empty);
            *body = bindings
                .iter()
                .rev()
                .fold(original, |body, (binder, value)| Self::Local {
                    patterns: vec![Pattern::Variable(binder.clone())],
                    quant: Quant::Many,
                    values: vec![SurfaceTerm::new(Rc::clone(value), None)],
                    body: Box::new(body),
                });
        }
        self
    }

    fn replace(&self, id: usize, replacement: &TermRef, origins: Option<&TermOrigins>) -> Self {
        self.map_terms(|value| value.replace(id, replacement, origins))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "The explicit frames keep ordered body traversal and reconstruction together."
    )]
    fn map_terms(&self, mut transform: impl FnMut(&SurfaceTerm) -> SurfaceTerm) -> Self {
        enum Frame<'a> {
            Local {
                patterns: Vec<Pattern>,
                quant: Quant,
                values: PendingSurface,
            },
            Match {
                scrutinees: PendingSurface,
                rows: &'a [Row],
                next: usize,
                completed: Vec<Row>,
                patterns: Vec<Pattern>,
            },
        }

        let mut frames = Vec::<Frame<'_>>::new();
        let mut current = self;
        'visit: loop {
            let mut result = match current {
                Self::Empty => Self::Empty,
                Self::Reply(value) => Self::Reply(transform(value)),
                Self::Local {
                    patterns,
                    quant,
                    values,
                    body,
                } => {
                    let patterns = patterns.clone();
                    let mut mapped = PendingSurface::default();
                    for value in values {
                        mapped.0.push(transform(value));
                    }
                    frames.push(Frame::Local {
                        patterns,
                        quant: *quant,
                        values: mapped,
                    });
                    current = body;
                    continue 'visit;
                }
                Self::Match { scrutinees, rows } => {
                    let mut mapped = PendingSurface::default();
                    for value in scrutinees {
                        mapped.0.push(transform(value));
                    }
                    if let Some(first) = rows.first() {
                        frames.push(Frame::Match {
                            scrutinees: mapped,
                            rows,
                            next: 1,
                            completed: Vec::new(),
                            patterns: first.patterns.clone(),
                        });
                        current = &first.body;
                        continue 'visit;
                    }
                    Self::Match {
                        scrutinees: mapped.take(),
                        rows: Vec::new(),
                    }
                }
            };
            loop {
                match frames.pop() {
                    None => return result,
                    Some(Frame::Local {
                        patterns,
                        quant,
                        mut values,
                    }) => {
                        result = Self::Local {
                            patterns,
                            quant,
                            values: values.take(),
                            body: Box::new(result),
                        };
                    }
                    Some(Frame::Match {
                        mut scrutinees,
                        rows,
                        next,
                        mut completed,
                        patterns,
                    }) => {
                        completed.push(Row {
                            patterns,
                            body: result,
                        });
                        if let Some(row) = rows.get(next) {
                            frames.push(Frame::Match {
                                scrutinees,
                                rows,
                                next: next + 1,
                                completed,
                                patterns: row.patterns.clone(),
                            });
                            current = &row.body;
                            continue 'visit;
                        }
                        result = Self::Match {
                            scrutinees: scrutinees.take(),
                            rows: completed,
                        };
                    }
                }
            }
        }
    }
}

pub(super) enum FlattenError {
    Other(String),
    LocalBinding { binder: usize },
    ComputedScrutinee { origin: usize },
    VariableScrutinee { origin: usize },
    ConstructorScrutinee { origin: usize },
}

impl FlattenError {
    pub(super) fn message(&self) -> &str {
        match self {
            Self::Other(message) => message,
            Self::LocalBinding { .. } => {
                "a match cannot scrutinize a local binding; pass it to a definition"
            }
            Self::ComputedScrutinee { .. }
            | Self::VariableScrutinee { .. }
            | Self::ConstructorScrutinee { .. } => {
                "match scrutinees must be parameters or fields, in binder order"
            }
        }
    }
}

impl From<String> for FlattenError {
    fn from(message: String) -> Self {
        Self::Other(message)
    }
}

impl From<&str> for FlattenError {
    fn from(message: &str) -> Self {
        Self::Other(message.into())
    }
}

pub(super) fn flatten_occurrence(
    body: Body,
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    origins: Option<&TermOrigins>,
) -> Result<Occurrence, FlattenError> {
    flatten_driver(
        Current::Body {
            body,
            vars: vars.to_vec(),
            depth: 0,
        },
        fresh,
        display_fresh,
        origins,
        Some(64),
    )
    .map(SurfaceTerm::into_occurrence)
}

struct LocalFrame {
    vars: Vec<Variable>,
    bindings: Vec<Variable>,
    quant: Quant,
    values: Vec<SurfaceTerm>,
}

fn finish_local(
    frame: LocalFrame,
    mut tail: SurfaceTerm,
    origins: Option<&TermOrigins>,
) -> Result<SurfaceTerm, FlattenError> {
    let LocalFrame {
        vars,
        bindings,
        quant,
        values,
    } = frame;
    for binder in &bindings {
        match tail.value.as_ref() {
            Term::Lam { body, .. } => {
                let origin = origins.and_then(|o| o.child_source(&tail.value, 0, body));
                let mut next = SurfaceTerm::new(Rc::clone(body), origins);
                next.explicit_source = origins.and_then(|o| o.occurrence_source(&tail.value, 0));
                if let Some(origin) = origin {
                    next.origin = origin;
                }
                tail = next;
            }
            _ => {
                return Err(FlattenError::LocalBinding { binder: binder.id });
            }
        }
    }
    let value = term(Term::Let {
        bindings: bindings
            .iter()
            .zip(&values)
            .map(|(b, v)| LetBinding {
                quant,
                name: b.name.clone(),
                id: b.id,
                value: Rc::clone(&v.value),
            })
            .collect(),
        body: tail.value,
    });
    if let Some(origins) = origins {
        origins.occurrence_children(
            &value,
            std::iter::once(tail.explicit_source).chain(values.iter().map(|v| v.explicit_source)),
        );
    }
    if let Some(origins) = origins
        && let Some(binder) = bindings.first()
    {
        origins.binder_node(binder.id, &value);
    }
    let value = SurfaceTerm::parent(value, [Some(tail.origin), None, None], origins);
    Ok(vars.iter().rev().fold(value, |v, b| v.lambda(b, origins)))
}

struct MatchState {
    scrutinees: Vec<SurfaceTerm>,
    rows: Vec<Row>,
    vars: Vec<Variable>,
}

enum MatchStep {
    Done(SurfaceTerm),
    Body {
        depth: usize,
    },
    Continue {
        next: MatchState,
        wrap: Option<Variable>,
    },
    SkipBinder(Variable),
    Nullary {
        constructor: String,
        constructor_origin: usize,
        next: MatchState,
        fallback: MatchState,
    },
    Constructor {
        constructor: String,
        constructor_origin: usize,
        next: MatchState,
        fallback: MatchState,
    },
}

// A suspended fallback owns the arm already compiled. On an error or
// unwind, release that partial term without recursively destroying its graph.
struct SavedFallback<'a> {
    parent: Option<MatchCall<'a>>,
    constructor: String,
    constructor_origin: usize,
    site: MatchSite,
    arm: Option<SurfaceTerm>,
}

impl Drop for SavedFallback<'_> {
    fn drop(&mut self) {
        if let Some(arm) = self.arm.take() {
            crate::kernel::release_term_iteratively(arm.value);
        }
    }
}

struct SavedArm<'a> {
    parent: MatchCall<'a>,
    constructor: String,
    constructor_origin: usize,
    site: MatchSite,
    fallback: MatchState,
}

enum SavedMatch<'a> {
    Arm(SavedArm<'a>),
    Fallback(SavedFallback<'a>),
    Continue {
        parent: MatchCall<'a>,
        wrap: Option<Variable>,
    },
    BodyReturn(MatchCall<'a>),
    Local(LocalFrame),
}

struct MatchCall<'a> {
    initial: Option<(&'a [SurfaceTerm], &'a [Row], &'a [Variable])>,
    state: Option<MatchState>,
    pending: Vec<(String, usize, MatchState)>,
    depth: usize,
}

enum Current<'a> {
    Match(MatchCall<'a>),
    Body {
        body: Body,
        vars: Vec<Variable>,
        depth: usize,
    },
}

#[cfg(test)]
fn flatten_match(
    scrutinees: &[SurfaceTerm],
    rows: &[Row],
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
) -> Result<SurfaceTerm, FlattenError> {
    flatten_match_bounded(
        scrutinees,
        rows,
        vars,
        fresh,
        display_fresh,
        depth,
        origins,
        64,
    )
}

#[cfg(test)]
#[expect(
    clippy::too_many_arguments,
    reason = "The match frame carries the existing traversal inputs and bound."
)]
fn flatten_match_bounded(
    scrutinees: &[SurfaceTerm],
    rows: &[Row],
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
    limit: usize,
) -> Result<SurfaceTerm, FlattenError> {
    flatten_driver(
        Current::Match(MatchCall {
            initial: Some((scrutinees, rows, vars)),
            state: None,
            pending: Vec::new(),
            depth,
        }),
        fresh,
        display_fresh,
        origins,
        Some(limit),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "The shared driver preserves body and match continuation order."
)]
fn flatten_driver<'a>(
    mut current: Current<'a>,
    fresh: &mut usize,
    display_fresh: &mut usize,
    origins: Option<&TermOrigins>,
    limit: Option<usize>,
) -> Result<SurfaceTerm, FlattenError> {
    // Production currently retains this bound while recursive kernel checking
    // can overflow on newly admitted deep patterns. Tests can exercise the
    // explicit driver without that bound.
    let mut saved: Vec<SavedMatch<'a>> = Vec::new();
    'compile: loop {
        let (mut value, mut parent) = match current {
            Current::Body {
                mut body,
                vars,
                depth,
            } => {
                if limit.is_some_and(|limit| depth >= limit) {
                    return Err(
                        "pattern compilation exceeds the resource limit of 64 levels".into(),
                    );
                }
                match &mut body {
                    Body::Empty => unreachable!("body work placeholder escaped"),
                    Body::Reply(value) => (
                        vars.iter()
                            .rev()
                            .fold(value.clone(), |v, binder| v.lambda(binder, origins)),
                        None,
                    ),
                    Body::Match { scrutinees, rows } => {
                        let state = MatchState {
                            scrutinees: std::mem::take(scrutinees),
                            rows: std::mem::take(rows),
                            vars,
                        };
                        current = Current::Match(MatchCall {
                            initial: None,
                            state: Some(state),
                            pending: Vec::new(),
                            depth: depth + 1,
                        });
                        continue 'compile;
                    }
                    Body::Local {
                        patterns,
                        quant,
                        values,
                        body: inner,
                    } => {
                        if let [Pattern::Constructor(_, _, _)] = patterns.as_slice() {
                            let row = Row {
                                patterns: std::mem::take(patterns),
                                body: std::mem::replace(inner.as_mut(), Body::Empty),
                            };
                            let state = MatchState {
                                scrutinees: std::mem::take(values),
                                rows: vec![row],
                                vars,
                            };
                            current = Current::Match(MatchCall {
                                initial: None,
                                state: Some(state),
                                pending: Vec::new(),
                                depth: depth + 1,
                            });
                            continue 'compile;
                        }
                        let bindings = patterns
                            .iter()
                            .map(|pattern| match pattern {
                                Pattern::Variable(v) => Ok(v.clone()),
                                Pattern::Constructor(_, _, _) => {
                                    Err("a parallel let can bind only names".to_owned())
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let child = std::mem::replace(inner.as_mut(), Body::Empty);
                        saved.push(SavedMatch::Local(LocalFrame {
                            vars,
                            bindings: bindings.clone(),
                            quant: *quant,
                            values: std::mem::take(values),
                        }));
                        current = Current::Body {
                            body: child,
                            vars: bindings,
                            depth: depth + 1,
                        };
                        continue 'compile;
                    }
                }
            }
            Current::Match(mut call) => {
                if limit.is_some_and(|limit| call.depth.saturating_add(call.pending.len()) >= limit)
                {
                    return Err(
                        "pattern compilation exceeds the resource limit of 64 levels".into(),
                    );
                }
                let (scrutinees, rows, vars) = if let Some(state) = &call.state {
                    (
                        state.scrutinees.as_slice(),
                        state.rows.as_slice(),
                        state.vars.as_slice(),
                    )
                } else {
                    call.initial.expect("initial or owned match state")
                };
                match flatten_match_step(
                    scrutinees,
                    rows,
                    vars,
                    fresh,
                    display_fresh,
                    call.depth,
                    limit,
                    origins,
                )? {
                    MatchStep::Done(value) => (value, Some(call)),
                    MatchStep::Body { depth } => {
                        let (body, vars) = if let Some(state) = &mut call.state {
                            (
                                std::mem::replace(&mut state.rows[0].body, Body::Empty),
                                state.vars.clone(),
                            )
                        } else {
                            let (_, rows, vars) = call.initial.expect("initial match state");
                            (rows[0].body.clone(), vars.to_vec())
                        };
                        saved.push(SavedMatch::BodyReturn(call));
                        current = Current::Body { body, vars, depth };
                        continue 'compile;
                    }
                    MatchStep::Continue { next, wrap } => {
                        let child = MatchCall {
                            initial: None,
                            state: Some(next),
                            pending: Vec::new(),
                            depth: call.depth + 1,
                        };
                        saved.push(SavedMatch::Continue { parent: call, wrap });
                        current = Current::Match(child);
                        continue 'compile;
                    }
                    MatchStep::SkipBinder(binder) => {
                        let mut next = call.state.take().unwrap_or_else(|| {
                            let (scrutinees, rows, vars) =
                                call.initial.expect("initial match state");
                            MatchState {
                                scrutinees: scrutinees.to_vec(),
                                rows: rows.to_vec(),
                                vars: vars.to_vec(),
                            }
                        });
                        next.vars.remove(0);
                        let child = MatchCall {
                            initial: None,
                            state: Some(next),
                            pending: Vec::new(),
                            depth: call.depth + 1,
                        };
                        saved.push(SavedMatch::Continue {
                            parent: call,
                            wrap: Some(binder),
                        });
                        current = Current::Match(child);
                        continue 'compile;
                    }
                    MatchStep::Nullary {
                        constructor,
                        constructor_origin,
                        next,
                        fallback,
                    } => {
                        call.pending
                            .push((constructor, constructor_origin, fallback));
                        call.state = Some(next);
                        current = Current::Match(call);
                        continue 'compile;
                    }
                    MatchStep::Constructor {
                        constructor,
                        constructor_origin,
                        next,
                        fallback,
                    } => {
                        let site = fallback.scrutinees[0].match_site;
                        let child = MatchCall {
                            initial: None,
                            state: Some(next),
                            pending: Vec::new(),
                            depth: call.depth + 1,
                        };
                        saved.push(SavedMatch::Arm(SavedArm {
                            parent: call,
                            constructor,
                            constructor_origin,
                            site,
                            fallback,
                        }));
                        current = Current::Match(child);
                        continue 'compile;
                    }
                }
            }
        };
        loop {
            if let Some(mut call) = parent.take()
                && let Some((constructor, constructor_origin, fallback)) = call.pending.pop()
            {
                let site = fallback.scrutinees[0].match_site;
                let next = MatchCall {
                    initial: None,
                    state: Some(fallback),
                    pending: Vec::new(),
                    depth: call.depth + 1,
                };
                saved.push(SavedMatch::Fallback(SavedFallback {
                    parent: Some(call),
                    constructor,
                    constructor_origin,
                    site,
                    arm: Some(value),
                }));
                current = Current::Match(next);
                continue 'compile;
            }
            match saved.pop() {
                None => return Ok(value),
                Some(SavedMatch::Continue { parent: call, wrap }) => {
                    if let Some(binder) = wrap {
                        value = value.lambda(&binder, origins);
                    }
                    parent = Some(call);
                }
                Some(SavedMatch::Fallback(mut previous)) => {
                    value = SurfaceTerm::matched(
                        std::mem::take(&mut previous.constructor),
                        previous.arm.take().expect("saved match arm"),
                        value,
                        Some(previous.site),
                        Some(previous.constructor_origin),
                        origins,
                    );
                    parent = Some(previous.parent.take().expect("saved parent call"));
                }
                Some(SavedMatch::Arm(previous)) => {
                    let next = MatchCall {
                        initial: None,
                        state: Some(previous.fallback),
                        pending: Vec::new(),
                        depth: previous.parent.depth + 1,
                    };
                    saved.push(SavedMatch::Fallback(SavedFallback {
                        parent: Some(previous.parent),
                        constructor: previous.constructor,
                        constructor_origin: previous.constructor_origin,
                        site: previous.site,
                        arm: Some(value),
                    }));
                    current = Current::Match(next);
                    continue 'compile;
                }
                Some(SavedMatch::BodyReturn(call)) => parent = Some(call),
                Some(SavedMatch::Local(frame)) => {
                    value = finish_local(frame, value, origins)?;
                    parent = None;
                }
            }
        }
    }
}

#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "Direct translation of upstream's first-row pattern compilation algorithm"
)]
fn flatten_match_step(
    scrutinees: &[SurfaceTerm],
    rows: &[Row],
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    limit: Option<usize>,
    origins: Option<&TermOrigins>,
) -> Result<MatchStep, FlattenError> {
    if limit.is_some_and(|limit| depth >= limit) {
        return Err("pattern compilation exceeds the resource limit of 64 levels".into());
    }
    if scrutinees.is_empty() {
        return rows.first().map_or_else(
            || Err("a match needs a case to return".into()),
            |_| Ok(MatchStep::Body { depth: depth + 1 }),
        );
    }
    let Some(first) = vars.first() else {
        let origin = scrutinees[0].origin;
        return Err(match scrutinees[0].value.as_ref() {
            Term::Var { .. } | Term::Ref(_) | Term::GpuRef(_) => {
                FlattenError::VariableScrutinee { origin }
            }
            Term::Ctr { .. } => FlattenError::ConstructorScrutinee { origin },
            _ => FlattenError::ComputedScrutinee { origin },
        });
    };
    let constructor = rows.iter().find_map(|row| match &row.patterns[0] {
        Pattern::Constructor(name, fields, origin) => Some((name, fields, *origin)),
        Pattern::Variable(_) => None,
    });
    let binder = vars
        .iter()
        .find(|v| matches!(scrutinees[0].value.as_ref(),Term::Var {id,..} if *id == v.id));
    if let Some(v) = binder.filter(|_| constructor.is_none() && !rows.is_empty()) {
        let rows = rows
            .iter()
            .map(|r| {
                let Pattern::Variable(p) = &r.patterns[0] else {
                    unreachable!("constructor-free column")
                };
                Row {
                    patterns: r.patterns[1..].to_vec(),
                    body: r.body.replace(p.id, &v.term(), origins),
                }
            })
            .collect::<Vec<_>>();
        return Ok(MatchStep::Continue {
            next: MatchState {
                scrutinees: scrutinees[1..].to_vec(),
                rows,
                vars: vars.to_vec(),
            },
            wrap: None,
        });
    }
    if binder.is_some_and(|v| v.id == first.id) {
        let Some((name, fields, constructor_origin)) = constructor else {
            return Ok(MatchStep::Done(
                SurfaceTerm::new(term(Term::Efq), origins)
                    .at_match(scrutinees[0].match_site, origins),
            ));
        };
        let field_vars = fields
            .iter()
            .map(|p| match p {
                Pattern::Variable(v) => v.clone(),
                Pattern::Constructor(_, _, source) => {
                    let id = *fresh;
                    *fresh += 1;
                    let display = *display_fresh;
                    *display_fresh += 1;
                    if let Some(origins) = origins {
                        origins.binder_source(id, *source);
                    }
                    Variable {
                        name: format!("_{display}"),
                        id,
                    }
                }
            })
            .collect::<Vec<_>>();
        let value = Pattern::Constructor(
            name.clone(),
            field_vars.iter().cloned().map(Pattern::Variable).collect(),
            constructor_origin,
        )
        .term();
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for row in rows {
            match &row.patterns[0] {
                Pattern::Constructor(n, ps, _) if n == name => {
                    if ps.len() != fields.len() {
                        return Err(
                            format!("constructor {name} has the wrong number of fields").into()
                        );
                    }
                    let mut patterns = ps.clone();
                    patterns.extend_from_slice(&row.patterns[1..]);
                    positive.push(Row {
                        patterns,
                        body: row.body.replace(first.id, &value, origins),
                    });
                }
                Pattern::Constructor(_, _, _) => negative.push(row.clone()),
                Pattern::Variable(v) => {
                    let mut patterns = field_vars
                        .iter()
                        .cloned()
                        .map(Pattern::Variable)
                        .collect::<Vec<_>>();
                    patterns.extend_from_slice(&row.patterns[1..]);
                    positive.push(Row {
                        patterns,
                        body: row
                            .body
                            .replace(v.id, &first.term(), origins)
                            .replace(first.id, &value, origins),
                    });
                    negative.push(row.clone());
                }
            }
        }
        let mut next_scrutinees = field_vars
            .iter()
            .map(|v| {
                let mut field = SurfaceTerm::new(v.term(), None);
                field.match_site = scrutinees[0].match_site;
                field
            })
            .collect::<Vec<_>>();
        next_scrutinees.extend_from_slice(&scrutinees[1..]);
        let mut next_vars = field_vars;
        next_vars.extend_from_slice(&vars[1..]);
        if fields.is_empty() {
            return Ok(MatchStep::Nullary {
                constructor: name.clone(),
                constructor_origin,
                next: MatchState {
                    scrutinees: next_scrutinees,
                    rows: positive,
                    vars: next_vars,
                },
                fallback: MatchState {
                    scrutinees: scrutinees.to_vec(),
                    rows: negative,
                    vars: vars.to_vec(),
                },
            });
        }
        Ok(MatchStep::Constructor {
            constructor: name.clone(),
            constructor_origin,
            next: MatchState {
                scrutinees: next_scrutinees,
                rows: positive,
                vars: next_vars,
            },
            fallback: MatchState {
                scrutinees: scrutinees.to_vec(),
                rows: negative,
                vars: vars.to_vec(),
            },
        })
    } else {
        Ok(MatchStep::SkipBinder(first.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_columns_and_skipped_binders_preserve_result_and_guard() {
        on_small_stack(|| {
            for count in [1, 30, 60] {
                let mut fresh = count;
                let vars = (0..count)
                    .map(|id| Variable {
                        name: format!("v{id}"),
                        id,
                    })
                    .collect::<Vec<_>>();
                let scrutinees = vars
                    .iter()
                    .map(|v| SurfaceTerm::new(v.term(), None))
                    .collect::<Vec<_>>();
                let rows = vec![Row {
                    patterns: vars.iter().cloned().map(Pattern::Variable).collect(),
                    body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
                }];
                let result = flatten_match(&scrutinees, &rows, &vars, &mut fresh, &mut 0, 0, None)
                    .unwrap_or_else(|error| panic!("variable column: {}", error.message()));
                let mut cursor = &result.value;
                for expected in &vars {
                    let Term::Lam { id, body, .. } = cursor.as_ref() else {
                        panic!("missing variable lambda")
                    };
                    assert_eq!(*id, expected.id);
                    cursor = body;
                }
                assert!(matches!(cursor.as_ref(), Term::Rfl));

                let rows = vec![Row {
                    patterns: vec![Pattern::Constructor("C".into(), vec![], 0)],
                    body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
                }];
                let last = vec![SurfaceTerm::new(vars.last().unwrap().term(), None)];
                let result = flatten_match(&last, &rows, &vars, &mut fresh, &mut 0, 0, None)
                    .unwrap_or_else(|error| panic!("skipped binder: {}", error.message()));
                let mut cursor = &result.value;
                for expected in &vars[..count - 1] {
                    let Term::Lam { id, body, .. } = cursor.as_ref() else {
                        panic!("missing skipped lambda")
                    };
                    assert_eq!(*id, expected.id);
                    cursor = body;
                }
                assert!(matches!(cursor.as_ref(), Term::Mat { .. }));
            }
            let vars = (0..63)
                .map(|id| Variable {
                    name: format!("v{id}"),
                    id,
                })
                .collect::<Vec<_>>();
            let scrutinees = vars
                .iter()
                .map(|v| SurfaceTerm::new(v.term(), None))
                .collect::<Vec<_>>();
            let rows = vec![Row {
                patterns: vars.iter().cloned().map(Pattern::Variable).collect(),
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            }];
            let Err(error) = flatten_match(&scrutinees, &rows, &vars, &mut 63, &mut 0, 0, None)
            else {
                panic!("variable-column depth guard must remain");
            };
            assert_eq!(
                error.message(),
                "pattern compilation exceeds the resource limit of 64 levels"
            );

            let rows = vec![Row {
                patterns: vec![Pattern::Constructor("C".into(), vec![], 0)],
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            }];
            let vars = (0..64)
                .map(|id| Variable {
                    name: format!("v{id}"),
                    id,
                })
                .collect::<Vec<_>>();
            let last = vec![SurfaceTerm::new(vars.last().unwrap().term(), None)];
            let Err(error) = flatten_match(&last, &rows, &vars, &mut 64, &mut 0, 0, None) else {
                panic!("skipped-binder depth guard must remain");
            };
            assert_eq!(
                error.message(),
                "pattern compilation exceeds the resource limit of 64 levels"
            );
        });
    }

    fn on_small_stack(f: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(f)
            .expect("spawn small-stack test")
            .join()
            .expect("small-stack test completed");
    }

    fn deep_term(depth: usize, leaf: TermRef) -> TermRef {
        (0..depth).fold(leaf, |value, _| term(Term::Typ(value)))
    }

    fn deep_body(depth: usize, leaf: TermRef, constructor_pattern: bool) -> Body {
        let mut body = Body::Reply(SurfaceTerm::new(leaf, None));
        for _ in 0..depth {
            let patterns = if constructor_pattern {
                vec![Pattern::Constructor("C".into(), vec![], 0)]
            } else {
                Vec::new()
            };
            body = Body::Local {
                patterns,
                quant: Quant::Lone,
                values: Vec::new(),
                body: Box::new(body),
            };
        }
        body
    }

    #[test]
    fn unified_body_driver_handles_deep_success_error_and_unwind() {
        on_small_stack(|| {
            let leaf = term(Term::Rfl);
            let weak = Rc::downgrade(&leaf);
            let body = deep_body(10_000, leaf, false);
            let result = flatten_driver(
                Current::Body {
                    body,
                    vars: vec![],
                    depth: 0,
                },
                &mut 0,
                &mut 0,
                None,
                Some(10_010),
            )
            .unwrap_or_else(|e| panic!("deep local: {}", e.message()));
            assert!(weak.upgrade().is_some());
            drop(result);
            assert!(weak.upgrade().is_none());

            let leaf = term(Term::Rfl);
            let weak = Rc::downgrade(&leaf);
            let body = deep_body(10_000, leaf, false);
            let Err(error) = flatten_driver(
                Current::Body {
                    body,
                    vars: vec![],
                    depth: 0,
                },
                &mut 0,
                &mut 0,
                None,
                Some(5_000),
            ) else {
                panic!("depth guard")
            };
            assert_eq!(
                error.message(),
                "pattern compilation exceeds the resource limit of 64 levels"
            );
            assert!(weak.upgrade().is_none());

            let retained = (0..10_000).fold(term(Term::Rfl), |child, _| term(Term::Typ(child)));
            let weak = Rc::downgrade(&retained);
            let body = deep_body(10_000, Rc::clone(&retained), false);
            let Err(_error) = flatten_driver(
                Current::Body {
                    body,
                    vars: vec![],
                    depth: 0,
                },
                &mut 0,
                &mut 0,
                None,
                Some(5_000),
            ) else {
                panic!("depth guard")
            };
            assert!(weak.upgrade().is_some());
            drop(retained);
            assert!(weak.upgrade().is_none());

            let anchor = term(Term::Rfl);
            let weak = Rc::downgrade(&anchor);
            let var = Variable {
                name: "x".into(),
                id: 1,
            };
            let mut body = Body::Empty;
            for _ in 0..3_000 {
                body = Body::Local {
                    patterns: vec![Pattern::Variable(var.clone())],
                    quant: Quant::Many,
                    values: vec![SurfaceTerm::new(Rc::clone(&anchor), None)],
                    body: Box::new(body),
                };
            }
            drop(anchor);
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = flatten_driver(
                    Current::Body {
                        body,
                        vars: vec![],
                        depth: 0,
                    },
                    &mut 0,
                    &mut 0,
                    None,
                    Some(4_000),
                );
            }));
            assert!(outcome.is_err(), "invalid body placeholder must unwind");
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn unified_driver_handles_nested_matches_on_small_stack() {
        on_small_stack(|| {
            let x = Variable {
                name: "x".into(),
                id: 7,
            };
            let leaf = term(Term::Rfl);
            let weak = Rc::downgrade(&leaf);
            let mut body = Body::Reply(SurfaceTerm::new(leaf, None));
            for _ in 0..128 {
                body = Body::Match {
                    scrutinees: vec![SurfaceTerm::new(x.term(), None)],
                    rows: vec![Row {
                        patterns: vec![Pattern::Variable(x.clone())],
                        body,
                    }],
                };
            }
            let result = flatten_driver(
                Current::Body {
                    body,
                    vars: vec![x],
                    depth: 0,
                },
                &mut 8,
                &mut 0,
                None,
                Some(512),
            )
            .unwrap_or_else(|error| panic!("nested match: {}", error.message()));
            assert!(matches!(result.value.as_ref(), Term::Lam { .. }));
            assert!(weak.upgrade().is_some());
            drop(result);
            assert!(weak.upgrade().is_none());
        });
    }

    fn body_leaf(mut body: &Body, depth: usize) -> &SurfaceTerm {
        for _ in 0..depth {
            let Body::Local { body: next, .. } = body else {
                panic!("missing body frame")
            };
            body = next;
        }
        let Body::Reply(value) = body else {
            panic!("missing reply")
        };
        value
    }

    #[test]
    fn deep_bodies_clone_replace_and_drop_automatically() {
        on_small_stack(|| {
            let input = deep_term(
                20_000,
                term(Term::Var {
                    name: "x".into(),
                    id: 7,
                }),
            );
            let input_weak = Rc::downgrade(&input);
            let body = deep_body(20_000, input, false);
            let cloned = body.clone();
            assert!(Rc::ptr_eq(
                &body_leaf(&body, 20_000).value,
                &body_leaf(&cloned, 20_000).value
            ));
            let replacement = term(Term::Rfl);
            let changed = body.replace(7, &replacement, None);
            let changed_weak = Rc::downgrade(&body_leaf(&changed, 20_000).value);
            let mut cursor = &body_leaf(&changed, 20_000).value;
            for _ in 0..20_000 {
                let Term::Typ(next) = cursor.as_ref() else {
                    panic!("missing substituted term")
                };
                cursor = next;
            }
            assert!(Rc::ptr_eq(cursor, &replacement));
            drop(body);
            assert!(input_weak.upgrade().is_some());
            drop(cloned);
            assert!(input_weak.upgrade().is_none());
            drop(changed);
            assert!(changed_weak.upgrade().is_none());
            assert_eq!(Rc::strong_count(&replacement), 1);
        });
    }

    #[test]
    fn deep_bodies_drop_automatically_on_error_and_unwind() {
        on_small_stack(|| {
            fn fail(_body: Body) -> Result<(), ()> {
                Err(())
            }
            let input = deep_term(20_000, term(Term::Rfl));
            let weak = Rc::downgrade(&input);
            fail(deep_body(100_000, input, false)).expect_err("injected error");
            assert!(weak.upgrade().is_none());
            let input = deep_term(20_000, term(Term::Rfl));
            let weak = Rc::downgrade(&input);
            let outcome = std::panic::catch_unwind(|| {
                let _body = deep_body(100_000, input, false);
                panic!("injected unwind");
            });
            let _panic = outcome.expect_err("injected unwind must be caught");
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn deep_reusable_binding_traversal_is_iterative() {
        on_small_stack(|| {
            let binder = Variable {
                name: "z".into(),
                id: 11,
            };
            let replacement = term(Term::Rfl);
            let mut body = deep_body(20_000, term(Term::Rfl), true)
                .with_reusable(&[(binder, Rc::clone(&replacement))]);
            let mut cursor = &mut body;
            for _ in 0..20_000 {
                let Body::Local { body, .. } = cursor else {
                    panic!("missing constructor body")
                };
                cursor = body;
            }
            let Body::Local {
                patterns,
                quant,
                values,
                body: tail,
            } = cursor
            else {
                panic!("missing reusable binding")
            };
            assert!(matches!(
                &patterns[0],
                Pattern::Variable(Variable { id: 11, .. })
            ));
            assert_eq!(*quant, Quant::Many);
            assert!(Rc::ptr_eq(&values[0].value, &replacement));
            assert!(matches!(tail.as_ref(), Body::Reply(_)));
            drop(body);
            assert_eq!(Rc::strong_count(&replacement), 1);
        });
    }

    #[test]
    fn reusable_bindings_preserve_match_row_and_binding_order() {
        let value_a = term(Term::Rfl);
        let value_b = term(Term::Qnt);
        let bindings = vec![
            (
                Variable {
                    name: "a".into(),
                    id: 1,
                },
                Rc::clone(&value_a),
            ),
            (
                Variable {
                    name: "b".into(),
                    id: 2,
                },
                Rc::clone(&value_b),
            ),
        ];
        let body = Body::Match {
            scrutinees: Vec::new(),
            rows: vec![
                Row {
                    patterns: vec![],
                    body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
                },
                Row {
                    patterns: vec![],
                    body: Body::Local {
                        patterns: vec![Pattern::Constructor("C".into(), vec![], 0)],
                        quant: Quant::Lone,
                        values: vec![],
                        body: Box::new(Body::Reply(SurfaceTerm::new(term(Term::Qnt), None))),
                    },
                },
            ],
        }
        .with_reusable(&bindings);
        let Body::Match { rows, .. } = &body else {
            panic!("match preserved")
        };
        for (index, row) in rows.iter().enumerate() {
            let mut cursor = &row.body;
            if index == 1 {
                let Body::Local { patterns, body, .. } = cursor else {
                    panic!("constructor local")
                };
                assert!(matches!(&patterns[0], Pattern::Constructor(name, _, _) if name == "C"));
                cursor = body;
            }
            for (id, expected) in [(1, &value_a), (2, &value_b)] {
                let Body::Local {
                    patterns,
                    quant,
                    values,
                    body,
                } = cursor
                else {
                    panic!("reusable local")
                };
                assert!(
                    matches!(&patterns[0], Pattern::Variable(Variable { id: actual, .. }) if *actual == id)
                );
                assert_eq!(*quant, Quant::Many);
                assert!(Rc::ptr_eq(&values[0].value, expected));
                cursor = body;
            }
            assert!(matches!(cursor, Body::Reply(_)));
        }
    }

    #[test]
    fn body_mapping_preserves_order_and_cleans_partial_values_on_unwind() {
        on_small_stack(|| {
            let value = |id| {
                SurfaceTerm::new(
                    term(Term::Var {
                        name: format!("v{id}"),
                        id,
                    }),
                    None,
                )
            };
            let body = Body::Match {
                scrutinees: vec![value(0), value(1)],
                rows: vec![
                    Row {
                        patterns: vec![],
                        body: Body::Local {
                            patterns: vec![],
                            quant: Quant::Lone,
                            values: vec![value(2)],
                            body: Box::new(Body::Reply(value(3))),
                        },
                    },
                    Row {
                        patterns: vec![],
                        body: Body::Reply(value(4)),
                    },
                ],
            };
            let mut seen = Vec::new();
            let mapped = body.map_terms(|v| {
                let Term::Var { id, .. } = v.value.as_ref() else {
                    panic!("expected variable")
                };
                seen.push(*id);
                v.clone()
            });
            assert_eq!(seen, [0, 1, 2, 3, 4]);
            drop(mapped);

            let mut calls = 0;
            let mut weak = std::rc::Weak::new();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = body.map_terms(|_| {
                    calls += 1;
                    if calls == 1 {
                        let root = deep_term(20_000, term(Term::Rfl));
                        weak = Rc::downgrade(&root);
                        SurfaceTerm::new(root, None)
                    } else {
                        panic!("injected sibling failure")
                    }
                });
            }));
            let _panic = outcome.expect_err("injected mapping panic must be caught");
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn body_drop_preserves_shared_term_owners() {
        on_small_stack(|| {
            let root = deep_term(20_000, term(Term::Rfl));
            let weak = Rc::downgrade(&root);
            let body = Body::Match {
                scrutinees: vec![SurfaceTerm::new(Rc::clone(&root), None)],
                rows: vec![Row {
                    patterns: vec![],
                    body: Body::Reply(SurfaceTerm::new(root, None)),
                }],
            };
            drop(body);
            assert!(weak.upgrade().is_none());

            let root = deep_term(20_000, term(Term::Rfl));
            let weak = Rc::downgrade(&root);
            let retained = Rc::clone(&root);
            let body = Body::Reply(SurfaceTerm::new(root, None));
            drop(body);
            assert_eq!(Rc::strong_count(&retained), 1);
            assert!(weak.upgrade().is_some());
            // The remaining public Rc owner still requires explicit release.
            crate::kernel::release_term_iteratively(retained);
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn body_drop_releases_all_composite_term_families() {
        on_small_stack(|| {
            for family in 0..12 {
                let child = deep_term(20_000, term(Term::Rfl));
                let weak = Rc::downgrade(&child);
                let root = term(match family {
                    0 => Term::Typ(child),
                    1 => Term::Min(Rc::clone(&child), child),
                    2 => Term::All {
                        quant: Quant::Lone,
                        name: "x".into(),
                        id: 1,
                        domain: Rc::clone(&child),
                        body: child,
                    },
                    3 => Term::Lam {
                        name: "x".into(),
                        id: 1,
                        body: child,
                    },
                    4 => Term::App(Rc::clone(&child), child),
                    5 => Term::Adt {
                        name: "T".into(),
                        args: vec![Rc::clone(&child), child],
                        excluded: vec![],
                    },
                    6 => Term::Ctr {
                        name: "C".into(),
                        args: vec![Rc::clone(&child), child],
                    },
                    7 => Term::Mat {
                        constructor: "C".into(),
                        arm: Rc::clone(&child),
                        fallback: child,
                    },
                    8 => Term::Eql {
                        left: Rc::clone(&child),
                        right: Rc::clone(&child),
                        ty: child,
                    },
                    9 => Term::Rwt {
                        evidence: Rc::clone(&child),
                        motive: Rc::clone(&child),
                        body: child,
                    },
                    10 => Term::Ann(Rc::clone(&child), child),
                    11 => Term::Let {
                        bindings: vec![LetBinding {
                            quant: Quant::Lone,
                            name: "x".into(),
                            id: 1,
                            value: Rc::clone(&child),
                        }],
                        body: child,
                    },
                    _ => unreachable!(),
                });
                drop(Body::Reply(SurfaceTerm::new(root, None)));
                assert!(weak.upgrade().is_none(), "family {family} remained alive");
            }
        });
    }

    fn nullary_alternatives(count: usize) -> (Vec<SurfaceTerm>, Vec<Row>, Vec<Variable>) {
        let variable = Variable {
            name: "x".into(),
            id: 0,
        };
        let scrutinees = vec![SurfaceTerm::new(variable.term(), None)];
        let rows = (0..count)
            .map(|i| Row {
                patterns: vec![Pattern::Constructor(format!("C{i}"), vec![], 0)],
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            })
            .collect();
        (scrutinees, rows, vec![variable])
    }

    fn deep_pattern(depth: usize, branching: bool) -> Pattern {
        let mut value = Pattern::Variable(Variable {
            name: "leaf".into(),
            id: 7,
        });
        for index in 0..depth {
            let fields = if branching {
                vec![
                    Pattern::Variable(Variable {
                        name: "side".into(),
                        id: index,
                    }),
                    value,
                ]
            } else {
                vec![value]
            };
            value = Pattern::Constructor("C".into(), fields, index);
        }
        value
    }

    #[test]
    fn deep_patterns_clone_and_drop_automatically_on_small_stack() {
        on_small_stack(|| {
            let original = deep_pattern(100_000, false);
            let copy = original.clone();
            let mut current = &copy;
            for index in (0..100_000).rev() {
                let Pattern::Constructor(name, fields, origin) = current else {
                    panic!("missing cloned constructor")
                };
                assert_eq!(name, "C");
                assert_eq!(*origin, index);
                assert_eq!(fields.len(), 1);
                current = &fields[0];
            }
            assert!(
                matches!(current, Pattern::Variable(Variable { name, id: 7 }) if name == "leaf")
            );
            drop(original);
            drop(copy);
            // A deep, branching tree exercises pending-sibling storage as well.
            drop(deep_pattern(8_192, true));
        });
    }

    #[test]
    fn deep_patterns_drop_automatically_on_error_and_unwind() {
        on_small_stack(|| {
            fn fail(_value: Pattern) -> Result<(), ()> {
                Err(())
            }
            fail(deep_pattern(100_000, false)).expect_err("injected error");
            let outcome = std::panic::catch_unwind(|| {
                let _value = deep_pattern(100_000, false);
                panic!("injected unwind");
            });
            let _panic = outcome.expect_err("injected unwind must be caught");
        });
    }

    #[test]
    fn nested_field_patterns_compile_on_small_stack() {
        on_small_stack(|| {
            let x = Variable {
                name: "x".into(),
                id: 0,
            };
            let y = Variable {
                name: "y".into(),
                id: 1,
            };
            let mut pattern = Pattern::Variable(y);
            for _ in 0..40 {
                pattern = Pattern::Constructor("C".into(), vec![pattern], 0);
            }
            let scrutinees = vec![SurfaceTerm::new(x.term(), None)];
            let vars = vec![x];
            let rows = vec![Row {
                patterns: vec![pattern],
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            }];
            let result = flatten_match(&scrutinees, &rows, &vars, &mut 2, &mut 0, 0, None)
                .unwrap_or_else(|error| panic!("unexpected error: {}", error.message()));
            let mut cursor = &result.value;
            for _ in 0..40 {
                let Term::Mat {
                    constructor,
                    arm,
                    fallback,
                } = cursor.as_ref()
                else {
                    panic!("missing field constructor")
                };
                assert_eq!(constructor, "C");
                assert!(matches!(fallback.as_ref(), Term::Efq));
                cursor = arm;
            }
            assert!(matches!(cursor.as_ref(), Term::Lam { .. }));
            crate::kernel::release_term_iteratively(result.value);
        });
    }

    #[test]
    fn unbounded_driver_compiles_deep_field_patterns_on_small_stack() {
        on_small_stack(|| {
            let x = Variable {
                name: "x".into(),
                id: 0,
            };
            let mut pattern = Pattern::Variable(Variable {
                name: "y".into(),
                id: 1,
            });
            for _ in 0..512 {
                pattern = Pattern::Constructor("C".into(), vec![pattern], 0);
            }
            let scrutinees = vec![SurfaceTerm::new(x.term(), None)];
            let vars = vec![x];
            let leaf = term(Term::Rfl);
            let weak = Rc::downgrade(&leaf);
            let rows = vec![Row {
                patterns: vec![pattern],
                body: Body::Reply(SurfaceTerm::new(leaf, None)),
            }];
            let result = flatten_driver(
                Current::Match(MatchCall {
                    initial: Some((&scrutinees, &rows, &vars)),
                    state: None,
                    pending: Vec::new(),
                    depth: 0,
                }),
                &mut 2,
                &mut 0,
                None,
                None,
            )
            .unwrap_or_else(|error| panic!("deep field: {}", error.message()));
            let mut cursor = &result.value;
            for _ in 0..512 {
                let Term::Mat { arm, fallback, .. } = cursor.as_ref() else {
                    panic!("match")
                };
                assert!(matches!(fallback.as_ref(), Term::Efq));
                cursor = arm;
            }
            assert!(matches!(cursor.as_ref(), Term::Lam { .. }));
            drop(rows);
            assert!(weak.upgrade().is_some());
            drop(result);
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn nested_field_patterns_keep_the_existing_depth_guard() {
        for (count, should_pass) in [(61, true), (62, false)] {
            let x = Variable {
                name: "x".into(),
                id: 0,
            };
            let mut pattern = Pattern::Variable(Variable {
                name: "y".into(),
                id: 1,
            });
            for _ in 0..count {
                pattern = Pattern::Constructor("C".into(), vec![pattern], 0);
            }
            let scrutinees = vec![SurfaceTerm::new(x.term(), None)];
            let vars = vec![x];
            let rows = vec![Row {
                patterns: vec![pattern],
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            }];
            let result = flatten_match(&scrutinees, &rows, &vars, &mut 2, &mut 0, 0, None);
            match (result, should_pass) {
                (Ok(value), true) => crate::kernel::release_term_iteratively(value.value),
                (Err(error), false) => assert_eq!(
                    error.message(),
                    "pattern compilation exceeds the resource limit of 64 levels"
                ),
                _ => panic!("field-pattern depth boundary changed at {count}"),
            }
        }
    }

    #[test]
    fn field_branch_errors_preserve_order_and_release_shared_arm() {
        on_small_stack(|| {
            let (scrutinees, mut rows, vars) = nullary_alternatives(2);
            let shared = term(Term::Rfl);
            rows[0].patterns[0] = Pattern::Constructor(
                "C0".into(),
                vec![Pattern::Variable(Variable {
                    name: "y".into(),
                    id: 1,
                })],
                0,
            );
            rows[0].body = Body::Reply(SurfaceTerm::new(Rc::clone(&shared), None));
            rows.push(Row {
                patterns: vec![Pattern::Constructor(
                    "C1".into(),
                    vec![Pattern::Variable(Variable {
                        name: "z".into(),
                        id: 2,
                    })],
                    0,
                )],
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            });
            let before = Rc::strong_count(&shared);
            let Err(error) = flatten_match(&scrutinees, &rows, &vars, &mut 3, &mut 0, 0, None)
            else {
                panic!("fallback constructor arity must fail");
            };
            assert_eq!(
                error.message(),
                "constructor C1 has the wrong number of fields"
            );
            assert_eq!(Rc::strong_count(&shared), before);

            rows[0].body = Body::Local {
                patterns: vec![
                    Pattern::Variable(vars[0].clone()),
                    Pattern::Constructor("Bad".into(), vec![], 0),
                ],
                quant: Quant::Lone,
                values: vec![],
                body: Box::new(Body::Reply(SurfaceTerm::new(term(Term::Rfl), None))),
            };
            let Err(error) = flatten_match(&scrutinees, &rows, &vars, &mut 3, &mut 0, 0, None)
            else {
                panic!("invalid positive arm must fail first");
            };
            assert_eq!(error.message(), "a parallel let can bind only names");
        });
    }

    #[test]
    fn nullary_fallbacks_compile_on_small_stack_and_keep_depth_boundary() {
        on_small_stack(|| {
            for count in [48, 63] {
                let (scrutinees, rows, vars) = nullary_alternatives(count);
                let result = flatten_match(&scrutinees, &rows, &vars, &mut 1, &mut 0, 0, None)
                    .unwrap_or_else(|error| panic!("unexpected error: {}", error.message()));
                let mut cursor = &result.value;
                for i in 0..count {
                    let Term::Mat {
                        constructor,
                        arm,
                        fallback,
                    } = cursor.as_ref()
                    else {
                        panic!("missing alternative {i}");
                    };
                    assert_eq!(constructor, &format!("C{i}"));
                    assert!(matches!(arm.as_ref(), Term::Rfl));
                    cursor = fallback;
                }
                assert!(matches!(cursor.as_ref(), Term::Efq));
                crate::kernel::release_term_iteratively(result.value);
            }
            let (scrutinees, rows, vars) = nullary_alternatives(64);
            let Err(error) = flatten_match(&scrutinees, &rows, &vars, &mut 1, &mut 0, 0, None)
            else {
                panic!("existing depth limit must still apply");
            };
            assert_eq!(
                error.message(),
                "pattern compilation exceeds the resource limit of 64 levels"
            );
        });
    }

    #[test]
    fn many_pending_nullary_columns_use_explicit_frames() {
        on_small_stack(|| {
            let vars = (0..256)
                .map(|id| Variable {
                    name: format!("v{id}"),
                    id,
                })
                .collect::<Vec<_>>();
            let scrutinees = vars
                .iter()
                .map(|v| SurfaceTerm::new(v.term(), None))
                .collect::<Vec<_>>();
            let rows = vec![Row {
                patterns: (0..256)
                    .map(|_| Pattern::Constructor("C".into(), vec![], 0))
                    .collect(),
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            }];
            // This private test raises only this driver's bound; public callers still use 64,
            // and the remaining recursive calls in this case stay at depth one.
            let result =
                flatten_match_bounded(&scrutinees, &rows, &vars, &mut 256, &mut 0, 0, None, 512)
                    .unwrap_or_else(|error| panic!("unexpected error: {}", error.message()));
            let mut cursor = &result.value;
            for _ in 0..256 {
                let Term::Mat {
                    constructor,
                    arm,
                    fallback,
                } = cursor.as_ref()
                else {
                    panic!("missing column")
                };
                assert_eq!(constructor, "C");
                assert!(matches!(fallback.as_ref(), Term::Efq));
                cursor = arm;
            }
            assert!(matches!(cursor.as_ref(), Term::Rfl));
            crate::kernel::release_term_iteratively(result.value);
        });
    }

    #[test]
    fn suspended_fallback_releases_deep_arm_on_unwind() {
        on_small_stack(|| {
            let mut deep = term(Term::Rfl);
            for _ in 0..20_000 {
                deep = term(Term::Typ(deep));
            }
            let weak = Rc::downgrade(&deep);
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _frame = SavedMatch::Fallback(SavedFallback {
                    parent: None,
                    constructor: "C".into(),
                    constructor_origin: 0,
                    site: MatchSite::Expression(0),
                    arm: Some(SurfaceTerm::new(deep, None)),
                });
                panic!("exercise unwind cleanup");
            }));
            let _panic = outcome.expect_err("the injected panic must be caught");
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn fallback_error_preserves_order_and_shared_arm_ownership() {
        on_small_stack(|| {
            let (scrutinees, mut rows, vars) = nullary_alternatives(2);
            let shared = term(Term::Rfl);
            rows[0].body = Body::Reply(SurfaceTerm::new(Rc::clone(&shared), None));
            rows.push(Row {
                patterns: vec![Pattern::Constructor(
                    "C1".into(),
                    vec![Pattern::Variable(Variable {
                        name: "y".into(),
                        id: 1,
                    })],
                    0,
                )],
                body: Body::Reply(SurfaceTerm::new(term(Term::Rfl), None)),
            });
            let before = Rc::strong_count(&shared);
            let Err(error) = flatten_match(&scrutinees, &rows, &vars, &mut 2, &mut 0, 0, None)
            else {
                panic!("inconsistent constructor arity must fail");
            };
            assert_eq!(
                error.message(),
                "constructor C1 has the wrong number of fields"
            );
            assert_eq!(Rc::strong_count(&shared), before);

            rows[0].body = Body::Local {
                patterns: vec![
                    Pattern::Variable(vars[0].clone()),
                    Pattern::Constructor("Bad".into(), vec![], 0),
                ],
                quant: Quant::Lone,
                values: vec![],
                body: Box::new(Body::Reply(SurfaceTerm::new(term(Term::Rfl), None))),
            };
            let Err(error) = flatten_match(&scrutinees, &rows, &vars, &mut 2, &mut 0, 0, None)
            else {
                panic!("invalid positive arm must fail before its fallback");
            };
            assert_eq!(error.message(), "a parallel let can bind only names");
        });
    }

    #[test]
    fn suspended_fallback_keeps_shared_deep_arm_alive() {
        on_small_stack(|| {
            let mut shared = term(Term::Rfl);
            for _ in 0..20_000 {
                shared = term(Term::Typ(shared));
            }
            let weak = Rc::downgrade(&shared);
            let saved = SavedFallback {
                parent: None,
                constructor: "C".into(),
                constructor_origin: 0,
                site: MatchSite::Expression(0),
                arm: Some(SurfaceTerm::new(Rc::clone(&shared), None)),
            };
            drop(saved);
            assert_eq!(Rc::strong_count(&shared), 1);
            assert!(weak.upgrade().is_some());
            crate::kernel::release_term_iteratively(shared);
            assert!(weak.upgrade().is_none());
        });
    }

    #[test]
    fn generated_match_edges_preserve_distinct_reply_origins_and_shared_values() {
        let origins = TermOrigins::default();
        let variable = Variable {
            name: "x".into(),
            id: 0,
        };
        let first = variable.term();
        let second = variable.term();
        origins.track(&first);
        origins.track(&second);
        let first = SurfaceTerm::new(first, Some(&origins));
        let second = SurfaceTerm::new(second, Some(&origins));
        let replacement = term(Term::Ctr {
            name: "Zero".into(),
            args: vec![],
        });
        let a = first.replace(0, &replacement, Some(&origins));
        let b = second.replace(0, &replacement, Some(&origins));
        let parent = SurfaceTerm::matched("Zero".into(), a, b, None, None, Some(&origins));
        let Term::Mat { arm, fallback, .. } = parent.value.as_ref() else {
            panic!("match")
        };
        assert!(Rc::ptr_eq(arm, &replacement));
        assert!(Rc::ptr_eq(fallback, &replacement));
        assert_eq!(Rc::strong_count(&replacement), 3);
        assert_eq!(
            origins.child_source(&parent.value, 0, arm),
            Some(first.origin)
        );
        assert_eq!(
            origins.child_source(&parent.value, 1, fallback),
            Some(second.origin)
        );
        assert_ne!(first.origin, second.origin);
    }

    #[test]
    fn scrutinee_substitution_retains_occurrences_without_copying_replacements() {
        let variable = Variable {
            name: "x".into(),
            id: 0,
        };
        let first = variable.term();
        let second = variable.term();
        let origins = TermOrigins::default();
        origins.track(&first);
        origins.track(&second);
        let first = SurfaceTerm::new(first, Some(&origins));
        let second = SurfaceTerm::new(second, Some(&origins));
        let replacement = term(Term::Ctr {
            name: "Zero".into(),
            args: vec![],
        });
        let a = first.replace(0, &replacement, Some(&origins));
        let b = second.replace(0, &replacement, Some(&origins));
        assert!(Rc::ptr_eq(&a.value, &replacement));
        assert!(Rc::ptr_eq(&b.value, &replacement));
        assert_eq!(Rc::strong_count(&replacement), 3);
        assert_eq!(a.origin, first.origin);
        assert_eq!(b.origin, second.origin);
        assert_ne!(a.origin, b.origin);
        let plain = first.replace(0, &replacement, None);
        assert!(Rc::ptr_eq(&plain.value, &replacement));
        assert_eq!(plain.origin, first.origin);
    }
}
