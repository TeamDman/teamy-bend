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

#[derive(Clone, Debug)]
pub(super) enum Pattern {
    Variable(Variable),
    Constructor(String, Vec<Pattern>, usize),
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

#[derive(Clone, Debug)]
pub(super) enum Body {
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

impl Body {
    pub fn with_reusable(self, bindings: &[(Variable, TermRef)]) -> Self {
        match self {
            Self::Local {
                patterns,
                quant,
                values,
                body,
            } if matches!(patterns.first(), Some(Pattern::Constructor(_, _, _))) => Self::Local {
                patterns,
                quant,
                values,
                body: Box::new(body.with_reusable(bindings)),
            },
            Self::Match { scrutinees, rows } => Self::Match {
                scrutinees,
                rows: rows
                    .into_iter()
                    .map(|row| Row {
                        patterns: row.patterns,
                        body: row.body.with_reusable(bindings),
                    })
                    .collect(),
            },
            body => bindings
                .iter()
                .rev()
                .fold(body, |body, (binder, value)| Self::Local {
                    patterns: vec![Pattern::Variable(binder.clone())],
                    quant: Quant::Many,
                    values: vec![SurfaceTerm::new(Rc::clone(value), None)],
                    body: Box::new(body),
                }),
        }
    }

    fn replace(&self, id: usize, replacement: &TermRef, origins: Option<&TermOrigins>) -> Self {
        match self {
            Self::Reply(value) => Self::Reply(value.replace(id, replacement, origins)),
            Self::Local {
                patterns,
                quant,
                values,
                body,
            } => Self::Local {
                patterns: patterns.clone(),
                quant: *quant,
                values: values
                    .iter()
                    .map(|v| v.replace(id, replacement, origins))
                    .collect(),
                body: Box::new(body.replace(id, replacement, origins)),
            },
            Self::Match { scrutinees, rows } => Self::Match {
                scrutinees: scrutinees
                    .iter()
                    .map(|v| v.replace(id, replacement, origins))
                    .collect(),
                rows: rows
                    .iter()
                    .map(|r| Row {
                        patterns: r.patterns.clone(),
                        body: r.body.replace(id, replacement, origins),
                    })
                    .collect(),
            },
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
    body: &Body,
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    origins: Option<&TermOrigins>,
) -> Result<Occurrence, FlattenError> {
    flatten_at(body, vars, fresh, display_fresh, 0, origins).map(SurfaceTerm::into_occurrence)
}

fn flatten_at(
    body: &Body,
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
) -> Result<SurfaceTerm, FlattenError> {
    if depth >= 64 {
        return Err("pattern compilation exceeds the resource limit of 64 levels".into());
    }
    match body {
        Body::Reply(value) => Ok(vars
            .iter()
            .rev()
            .fold(value.clone(), |v, b| v.lambda(b, origins))),
        Body::Match { scrutinees, rows } => flatten_match(
            scrutinees,
            rows,
            vars,
            fresh,
            display_fresh,
            depth + 1,
            origins,
        ),
        Body::Local {
            patterns,
            quant,
            values,
            body,
        } => {
            if let [Pattern::Constructor(_, _, _)] = patterns.as_slice() {
                return flatten_match(
                    values,
                    &[Row {
                        patterns: patterns.clone(),
                        body: *body.clone(),
                    }],
                    vars,
                    fresh,
                    display_fresh,
                    depth + 1,
                    origins,
                );
            }
            let bindings = patterns
                .iter()
                .map(|p| match p {
                    Pattern::Variable(v) => Ok(v.clone()),
                    Pattern::Constructor(_, _, _) => {
                        Err("a parallel let can bind only names".to_owned())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut tail = flatten_at(body, &bindings, fresh, display_fresh, depth + 1, origins)?;
            for binder in &bindings {
                match tail.value.as_ref() {
                    Term::Lam { body, .. } => {
                        let origin = origins.and_then(|o| o.child_source(&tail.value, 0, body));
                        let mut next = SurfaceTerm::new(Rc::clone(body), origins);
                        next.explicit_source =
                            origins.and_then(|o| o.occurrence_source(&tail.value, 0));
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
                    .zip(values)
                    .map(|(b, v)| LetBinding {
                        quant: *quant,
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
                    std::iter::once(tail.explicit_source)
                        .chain(values.iter().map(|v| v.explicit_source)),
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
    }
}

struct MatchState {
    scrutinees: Vec<SurfaceTerm>,
    rows: Vec<Row>,
    vars: Vec<Variable>,
}

enum MatchStep {
    Done(SurfaceTerm),
    Nullary {
        constructor: String,
        constructor_origin: usize,
        next: MatchState,
        fallback: MatchState,
    },
}

fn flatten_match(
    scrutinees: &[SurfaceTerm],
    rows: &[Row],
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
) -> Result<SurfaceTerm, FlattenError> {
    // A nullary constructor consumes a column without introducing field
    // binders. Continue that arm iteratively: a fixed-width word has one Bool
    // column per bit, which need not double its Rust recursion depth. Retain
    // bounded pending work and compile fallbacks only after their arms, keeping
    // the original term, fresh-name sequence and first-error ordering.
    let mut state: Option<MatchState> = None;
    let mut pending: Vec<(String, usize, MatchState)> = Vec::new();
    loop {
        if depth + pending.len() >= 64 {
            return Err("pattern compilation exceeds the resource limit of 64 levels".into());
        }
        let (scrutinees, rows, vars) = state.as_ref().map_or((scrutinees, rows, vars), |s| {
            (
                s.scrutinees.as_slice(),
                s.rows.as_slice(),
                s.vars.as_slice(),
            )
        });
        match flatten_match_step(scrutinees, rows, vars, fresh, display_fresh, depth, origins)? {
            MatchStep::Done(mut value) => {
                for (constructor, constructor_origin, fallback) in pending.into_iter().rev() {
                    let site = fallback.scrutinees[0].match_site;
                    let fallback = flatten_match(
                        &fallback.scrutinees,
                        &fallback.rows,
                        &fallback.vars,
                        fresh,
                        display_fresh,
                        depth + 1,
                        origins,
                    )?;
                    value = SurfaceTerm::matched(
                        constructor,
                        value,
                        fallback,
                        Some(site),
                        Some(constructor_origin),
                        origins,
                    );
                }
                return Ok(value);
            }
            MatchStep::Nullary {
                constructor,
                constructor_origin,
                next,
                fallback,
            } => {
                pending.push((constructor, constructor_origin, fallback));
                state = Some(next);
            }
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Direct translation of upstream's first-row pattern compilation algorithm"
)]
fn flatten_match_step(
    scrutinees: &[SurfaceTerm],
    rows: &[Row],
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
) -> Result<MatchStep, FlattenError> {
    if depth >= 64 {
        return Err("pattern compilation exceeds the resource limit of 64 levels".into());
    }
    if scrutinees.is_empty() {
        return rows.first().map_or_else(
            || Err("a match needs a case to return".into()),
            |r| {
                flatten_at(&r.body, vars, fresh, display_fresh, depth + 1, origins)
                    .map(MatchStep::Done)
            },
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
        return flatten_match(
            &scrutinees[1..],
            &rows,
            vars,
            fresh,
            display_fresh,
            depth + 1,
            origins,
        )
        .map(MatchStep::Done);
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
        let arm = flatten_match(
            &next_scrutinees,
            &positive,
            &next_vars,
            fresh,
            display_fresh,
            depth + 1,
            origins,
        )?;
        let fallback = flatten_match(
            scrutinees,
            &negative,
            vars,
            fresh,
            display_fresh,
            depth + 1,
            origins,
        )?;
        Ok(MatchStep::Done(SurfaceTerm::matched(
            name.clone(),
            arm,
            fallback,
            Some(scrutinees[0].match_site),
            Some(constructor_origin),
            origins,
        )))
    } else {
        Ok(MatchStep::Done(
            flatten_match(
                scrutinees,
                rows,
                &vars[1..],
                fresh,
                display_fresh,
                depth + 1,
                origins,
            )?
            .lambda(first, origins),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
