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
    pub parameters: Vec<Binder>,
    pub ty: TermRef,
    pub body: Option<TermRef>,
    pub unsafe_: bool,
    pub foreign: bool,
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

/// Capture-free substitution for terms whose binder IDs are globally unique.
#[must_use]
pub fn substitute(value: &TermRef, target: usize, replacement: &TermRef) -> TermRef {
    let sub = |x: &TermRef| substitute(x, target, replacement);
    match value.as_ref() {
        Term::Var { id, .. } if *id == target => Rc::clone(replacement),
        Term::Var { .. }
        | Term::Ref(_)
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
        Term::App(f, x) => term(Term::App(sub(f), sub(x))),
        Term::Adt {
            name,
            args,
            excluded,
        } => term(Term::Adt {
            name: name.clone(),
            args: args.iter().map(sub).collect(),
            excluded: excluded.clone(),
        }),
        Term::Ctr { name, args } => term(Term::Ctr {
            name: name.clone(),
            args: args.iter().map(sub).collect(),
        }),
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

pub(crate) fn inspect(value: &TermRef) -> Result<usize, super::KernelError> {
    let mut stack = vec![(value, 0_usize)];
    let mut highest = 0;
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
                return Err(super::KernelError::new(format!(
                    "unfinished proof hole ?{name}"
                )));
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
    Ok(highest)
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

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var { name, .. } | Self::Ref(name) => f.write_str(name),
            Self::Typ(g) => match g.as_ref() {
                Self::Qua(Quant::Lone) => f.write_str("Type"),
                Self::Qua(Quant::Many) => f.write_str("Data"),
                _ => write!(f, "Kind({g})"),
            },
            Self::Qnt => f.write_str("Quant"),
            Self::Qua(q) => write!(
                f,
                "&{}",
                match q {
                    Quant::None => 0,
                    Quant::Lone => 1,
                    Quant::Many => 2,
                }
            ),
            Self::Min(a, b) => write!(f, "({a} <&> {b})"),
            Self::All {
                quant,
                name,
                domain,
                body,
                ..
            } => write!(f, "@{quant}{name}:{domain} -> {body}"),
            Self::Lam { name, body, .. } => write!(f, "{name} => {body}"),
            Self::App(a, b) => write!(f, "{a}({b})"),
            Self::Adt {
                name,
                args,
                excluded,
            } => {
                f.write_str(name)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    list(f, args)?;
                    f.write_str(">")?;
                }
                if !excluded.is_empty() {
                    write!(f, " - [{}]", excluded.join(", "))?;
                }
                Ok(())
            }
            Self::Ctr { name, args } => {
                write!(f, "{name}{{")?;
                list(f, args)?;
                f.write_str("}")
            }
            Self::Mat {
                constructor,
                arm,
                fallback,
            } => write!(f, "\\{{{constructor}: {arm}; {fallback}}}"),
            Self::Efq => f.write_str("\\{}"),
            Self::Eql { left, right, ty } => write!(f, "{{{left} == {right} : {ty}}}"),
            Self::Rfl => f.write_str("{==}"),
            Self::Rwt {
                evidence,
                motive,
                body,
            } => write!(f, "%{evidence}: {motive}; {body}"),
            Self::Hole(name) => write!(f, "?{name}"),
            Self::Ann(x, t) => write!(f, "{{{x} : {t}}}"),
            Self::Let { bindings, body } => {
                for b in bindings {
                    write!(f, "{}{} = {}; ", b.quant, b.name, b.value)?;
                }
                write!(f, "{body}")
            }
        }
    }
}

fn list(f: &mut fmt::Formatter<'_>, values: &[TermRef]) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{x}")?;
    }
    Ok(())
}
