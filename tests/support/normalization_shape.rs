// SPDX-License-Identifier: MPL-2.0
//! A deliberately bounded structural comparison vocabulary for this family.
//! It performs no evaluation, reordering, text replacement or eta conversion.
use facet::Facet;
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
}

pub fn shape(value: &TermRef) -> Result<Shape, &'static str> {
    fn visit(value: &TermRef, scope: &mut Vec<usize>, depth: usize) -> Result<Shape, &'static str> {
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
            Term::Ref(name) => Shape::Reference(name.clone()),
            Term::Lam { id, body, .. } => {
                scope.push(*id);
                let body = visit(body, scope, depth + 1)?;
                scope.pop();
                Shape::Lambda(Box::new(body))
            }
            Term::App(function, argument) => Shape::Application(
                Box::new(visit(function, scope, depth + 1)?),
                Box::new(visit(argument, scope, depth + 1)?),
            ),
            Term::Ctr { name, args } => Shape::Constructor(
                name.clone(),
                args.iter()
                    .map(|arg| visit(arg, scope, depth + 1))
                    .collect::<Result<_, _>>()?,
            ),
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => Shape::Match(
                constructor.clone(),
                Box::new(visit(arm, scope, depth + 1)?),
                Box::new(visit(fallback, scope, depth + 1)?),
            ),
            Term::Efq => Shape::EmptyMatch,
            _ => return Err("term outside the qualified comparison vocabulary"),
        })
    }
    visit(value, &mut Vec::new(), 0)
}
