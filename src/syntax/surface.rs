// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use crate::kernel::LetBinding;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::origins::TermOrigins;
use crate::kernel::origins::substitute_with_origins;
use crate::kernel::term;
use std::rc::Rc;

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
    Constructor(String, Vec<Pattern>),
}

impl Pattern {
    fn term(&self) -> TermRef {
        match self {
            Self::Variable(v) => v.term(),
            Self::Constructor(name, fields) => term(Term::Ctr {
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

// A match occurrence keeps its own source anchor when substitution shares a
// replacement across several uses. This is discarded with the surface body;
// it adds no core term edge, identity, evaluator work, or serialized field.
#[derive(Clone, Debug)]
pub(super) struct Scrutinee {
    value: TermRef,
    origin: usize,
}

impl Scrutinee {
    pub(super) fn new(value: TermRef, origins: Option<&TermOrigins>) -> Self {
        let origin = origins.map_or_else(|| Rc::as_ptr(&value) as usize, |o| o.source(&value));
        Self { value, origin }
    }

    fn replace(&self, id: usize, replacement: &TermRef, origins: Option<&TermOrigins>) -> Self {
        Self {
            value: substitute_with_origins(&self.value, id, replacement, origins),
            origin: self.origin,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Body {
    Reply(TermRef),
    Local {
        patterns: Vec<Pattern>,
        quant: Quant,
        values: Vec<Scrutinee>,
        body: Box<Self>,
    },
    Match {
        scrutinees: Vec<Scrutinee>,
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
            } if matches!(patterns.first(), Some(Pattern::Constructor(_, _))) => Self::Local {
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
                    values: vec![Scrutinee::new(Rc::clone(value), None)],
                    body: Box::new(body),
                }),
        }
    }

    fn replace(&self, id: usize, replacement: &TermRef, origins: Option<&TermOrigins>) -> Self {
        let sub = |t: &TermRef| substitute_with_origins(t, id, replacement, origins);
        match self {
            Self::Reply(value) => Self::Reply(sub(value)),
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

pub(super) fn flatten(
    body: &Body,
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    origins: Option<&TermOrigins>,
) -> Result<TermRef, FlattenError> {
    flatten_at(body, vars, fresh, display_fresh, 0, origins)
}

fn flatten_at(
    body: &Body,
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
) -> Result<TermRef, FlattenError> {
    if depth >= 64 {
        return Err("pattern compilation exceeds the resource limit of 64 levels".into());
    }
    match body {
        Body::Reply(value) => Ok(vars.iter().rev().fold(Rc::clone(value), |v, b| b.lambda(v))),
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
            if let [Pattern::Constructor(_, _)] = patterns.as_slice() {
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
                    Pattern::Constructor(_, _) => {
                        Err("a parallel let can bind only names".to_owned())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut tail = flatten_at(body, &bindings, fresh, display_fresh, depth + 1, origins)?;
            for binder in &bindings {
                match tail.as_ref() {
                    Term::Lam { body, .. } => tail = Rc::clone(body),
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
                body: tail,
            });
            Ok(vars.iter().rev().fold(value, |v, b| b.lambda(v)))
        }
    }
}

struct MatchState {
    scrutinees: Vec<Scrutinee>,
    rows: Vec<Row>,
    vars: Vec<Variable>,
}

enum MatchStep {
    Done(TermRef),
    Nullary {
        constructor: String,
        next: MatchState,
        fallback: MatchState,
    },
}

fn flatten_match(
    scrutinees: &[Scrutinee],
    rows: &[Row],
    vars: &[Variable],
    fresh: &mut usize,
    display_fresh: &mut usize,
    depth: usize,
    origins: Option<&TermOrigins>,
) -> Result<TermRef, FlattenError> {
    // A nullary constructor consumes a column without introducing field
    // binders. Continue that arm iteratively: a fixed-width word has one Bool
    // column per bit, which need not double its Rust recursion depth. Retain
    // bounded pending work and compile fallbacks only after their arms, keeping
    // the original term, fresh-name sequence and first-error ordering.
    let mut state: Option<MatchState> = None;
    let mut pending: Vec<(String, MatchState)> = Vec::new();
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
                for (constructor, fallback) in pending.into_iter().rev() {
                    let fallback = flatten_match(
                        &fallback.scrutinees,
                        &fallback.rows,
                        &fallback.vars,
                        fresh,
                        display_fresh,
                        depth + 1,
                        origins,
                    )?;
                    value = term(Term::Mat {
                        constructor,
                        arm: value,
                        fallback,
                    });
                }
                return Ok(value);
            }
            MatchStep::Nullary {
                constructor,
                next,
                fallback,
            } => {
                pending.push((constructor, fallback));
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
    scrutinees: &[Scrutinee],
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
        Pattern::Constructor(name, fields) => Some((name, fields)),
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
        let Some((name, fields)) = constructor else {
            return Ok(MatchStep::Done(term(Term::Efq)));
        };
        let field_vars = fields
            .iter()
            .map(|p| match p {
                Pattern::Variable(v) => v.clone(),
                Pattern::Constructor(_, _) => {
                    let id = *fresh;
                    *fresh += 1;
                    let display = *display_fresh;
                    *display_fresh += 1;
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
        )
        .term();
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for row in rows {
            match &row.patterns[0] {
                Pattern::Constructor(n, ps) if n == name => {
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
                Pattern::Constructor(_, _) => negative.push(row.clone()),
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
            .map(|v| Scrutinee::new(v.term(), None))
            .collect::<Vec<_>>();
        next_scrutinees.extend_from_slice(&scrutinees[1..]);
        let mut next_vars = field_vars;
        next_vars.extend_from_slice(&vars[1..]);
        if fields.is_empty() {
            return Ok(MatchStep::Nullary {
                constructor: name.clone(),
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
        Ok(MatchStep::Done(term(Term::Mat {
            constructor: name.clone(),
            arm,
            fallback,
        })))
    } else {
        Ok(MatchStep::Done(first.lambda(flatten_match(
            scrutinees,
            rows,
            &vars[1..],
            fresh,
            display_fresh,
            depth + 1,
            origins,
        )?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let first = Scrutinee::new(first, Some(&origins));
        let second = Scrutinee::new(second, Some(&origins));
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
