// SPDX-License-Identifier: MPL-2.0
//! A deliberately bounded structural comparison vocabulary for this family.
//! It performs no evaluation, reordering, text replacement or eta conversion.
use facet::Facet;
use std::collections::BTreeSet;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;

#[derive(Debug, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum Shape {
    Bound(usize),
    Reference(String),
    Lambda(Box<Self>),
    Application(Box<Self>, Box<Self>),
    Constructor(String, Vec<Self>),
    Match(String, Box<Self>, Box<Self>),
    EmptyMatch,
    Datatype(String, Vec<Self>, Vec<String>),
    Equation(Box<Self>, Box<Self>, Box<Self>),
    Reflexivity,
    Rewrite(Box<Self>, Box<Self>, Box<Self>),
}

pub fn shape(value: &TermRef) -> Result<Shape, &'static str> {
    shape_with_datatypes(value, &BTreeSet::new())
}

/// Resolve only source-declared, parameterless datatype names in parsed oracle
/// text. Keep arguments, exclusions, and all rewrite fields explicit; no reduction.
pub fn shape_in_book(value: &TermRef, book: &Book) -> Result<Shape, &'static str> {
    let datatypes: BTreeSet<_> = book
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Adt(adt) if adt.parameters.is_empty() => Some(adt.name.as_str()),
            _ => None,
        })
        .collect();
    if datatypes.is_empty() {
        shape(value)
    } else {
        shape_with_datatypes(value, &datatypes)
    }
}

fn shape_with_datatypes(
    value: &TermRef,
    datatypes: &BTreeSet<&str>,
) -> Result<Shape, &'static str> {
    fn visit(
        value: &TermRef,
        scope: &mut Vec<usize>,
        depth: usize,
        datatypes: &BTreeSet<&str>,
    ) -> Result<Shape, &'static str> {
        if depth > 128 {
            return Err("comparison depth exhausted");
        }
        Ok(match value.as_ref() {
            Term::Var { id, .. } => Shape::Bound(
                scope
                    .iter()
                    .rev()
                    .position(|bound| bound == id)
                    .ok_or("unbound comparison variable")?,
            ),
            Term::Ref(name) if datatypes.contains(name.as_str()) => {
                Shape::Datatype(name.clone(), Vec::new(), Vec::new())
            }
            Term::Ref(name) => Shape::Reference(name.clone()),
            Term::Lam { id, body, .. } => {
                scope.push(*id);
                let body = visit(body, scope, depth + 1, datatypes)?;
                scope.pop();
                Shape::Lambda(Box::new(body))
            }
            Term::App(function, argument) => Shape::Application(
                Box::new(visit(function, scope, depth + 1, datatypes)?),
                Box::new(visit(argument, scope, depth + 1, datatypes)?),
            ),
            Term::Ctr { name, args } => Shape::Constructor(
                name.clone(),
                args.iter()
                    .map(|arg| visit(arg, scope, depth + 1, datatypes))
                    .collect::<Result<_, _>>()?,
            ),
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => Shape::Match(
                constructor.clone(),
                Box::new(visit(arm, scope, depth + 1, datatypes)?),
                Box::new(visit(fallback, scope, depth + 1, datatypes)?),
            ),
            Term::Efq => Shape::EmptyMatch,
            Term::Adt {
                name,
                args,
                excluded,
            } => Shape::Datatype(
                name.clone(),
                args.iter()
                    .map(|arg| visit(arg, scope, depth + 1, datatypes))
                    .collect::<Result<_, _>>()?,
                excluded.clone(),
            ),
            Term::Eql { left, right, ty } => Shape::Equation(
                Box::new(visit(left, scope, depth + 1, datatypes)?),
                Box::new(visit(right, scope, depth + 1, datatypes)?),
                Box::new(visit(ty, scope, depth + 1, datatypes)?),
            ),
            Term::Rfl => Shape::Reflexivity,
            Term::Rwt {
                evidence,
                motive,
                body,
            } => Shape::Rewrite(
                Box::new(visit(evidence, scope, depth + 1, datatypes)?),
                Box::new(visit(motive, scope, depth + 1, datatypes)?),
                Box::new(visit(body, scope, depth + 1, datatypes)?),
            ),
            _ => return Err("term outside the qualified comparison vocabulary"),
        })
    }
    visit(value, &mut Vec::new(), 0, datatypes)
}
