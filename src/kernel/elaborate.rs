// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5's typing rules, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Typed lowering of already checked executable terms.
//!
//! This is not another acceptance path. Only `ExecutableBook` supplies the
//! checked engine and loader-owned metadata. The traversal reconstructs types
//! needed by code generation; it grants no proof or evaluation capability and
//! never changes affine, descent, or foreign-contract checking.

use super::AdtDecl;
use super::Binder;
use super::ConstructorDecl;
use super::ExecutableEntry;
use super::KernelError;
use super::Quant;
use super::Term;
use super::TermRef;
use super::arrows;
use super::check::Engine;
use super::reduce::apply;
use super::substitute;
use super::term;
use crate::syntax::executable::ForeignDefinition;
use crate::syntax::executable::NumericIntrinsic;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

type Context = BTreeMap<usize, Binder>;

/// A compiler input, not a strict proof certificate.
#[derive(Clone, Debug)]
pub(crate) struct ExecutableProgram {
    pub(crate) entry: ExecutableEntry,
    pub(crate) definitions: BTreeMap<String, Definition>,
    pub(crate) datatypes: BTreeMap<String, AdtDecl>,
    pub(crate) constructor_tags: BTreeMap<String, String>,
    pub(crate) base_names: BTreeSet<String>,
    types: Engine,
}

impl ExecutableProgram {
    /// Compiler-only weak-head type inspection; this grants no proof token.
    pub(crate) fn expose_type(&self, ty: &TermRef) -> Result<TermRef, KernelError> {
        let mut engine = self.types.clone();
        engine.reset();
        engine.whnf(ty)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Definition {
    pub(crate) ty: TermRef,
    /// The actual declared telescope, including erased parameters. A returned
    /// closure's binders are in `ty` and the expression, not in this list.
    pub(crate) parameters: Vec<Binder>,
    pub(crate) body: DefinitionBody,
}

#[derive(Clone, Debug)]
pub(crate) enum DefinitionBody {
    Ordinary(Expression),
    Foreign(ForeignDefinition),
    Numeric(NumericIntrinsic),
}

#[derive(Clone, Debug)]
pub(crate) struct Expression {
    pub(crate) ty: TermRef,
    pub(crate) kind: ExpressionKind,
}

#[derive(Clone, Debug)]
pub(crate) enum ExpressionKind {
    /// A live type/proof becomes null; an erased argument or field is omitted.
    /// The surrounding binder quantity distinguishes those two cases.
    Erased,
    Variable(usize),
    Definition(String),
    Lambda {
        parameter: Binder,
        body: Box<Expression>,
    },
    Apply {
        function: Box<Expression>,
        argument: Box<Expression>,
        quant: Quant,
    },
    Constructor {
        owner: String,
        name: String,
        fields: Vec<Field>,
    },
    /// A match is a function over its scrutinee. The arm is a function over
    /// `fields` (including erased fields); fallback consumes the residual ADT.
    Match {
        parameter: Binder,
        owner: String,
        constructor: String,
        fields: Vec<Binder>,
        arm: Box<Expression>,
        fallback: Box<Expression>,
    },
    /// Applying an impossible branch must fail at runtime, never produce data.
    Absurd {
        parameter: Binder,
        owner: String,
    },
    /// Values are simultaneous: each sees the outer scope, the body sees all.
    Let {
        bindings: Vec<Field>,
        body: Box<Expression>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub(crate) binder: Binder,
    pub(crate) value: Expression,
}

pub(super) fn lower(
    engine: &Engine,
    foreign: &BTreeMap<String, ForeignDefinition>,
    numeric: &BTreeMap<String, NumericIntrinsic>,
    constructor_tags: &BTreeMap<String, String>,
    base_names: &BTreeSet<String>,
    entry: ExecutableEntry,
) -> Result<ExecutableProgram, KernelError> {
    let mut lowerer = Lowerer {
        engine: engine.clone(),
        references: BTreeSet::new(),
    };
    if entry != ExecutableEntry::Missing {
        lowerer.references.insert("main".into());
    }
    let mut definitions = BTreeMap::new();
    while let Some(name) = lowerer.references.pop_first() {
        if definitions.contains_key(&name) {
            continue;
        }
        lowerer.engine.reset();
        let definition = engine
            .defs
            .get(&name)
            .ok_or_else(|| error(format!("undefined reachable definition {name}")))?;
        let parameters = lowerer.parameters(&definition.ty, &definition.parameters)?;
        let body = if let Some(intrinsic) = numeric.get(&name) {
            DefinitionBody::Numeric(*intrinsic)
        } else if let Some(metadata) = foreign.get(&name) {
            DefinitionBody::Foreign(metadata.clone())
        } else {
            let body = definition
                .body
                .as_ref()
                .ok_or_else(|| error(format!("reachable definition {name} has no body")))?;
            DefinitionBody::Ordinary(
                lowerer
                    .expression(body, &definition.ty, Quant::Lone, &Context::new())
                    .map_err(|failure| error(format!("{name}: {}", failure.message)))?,
            )
        };
        definitions.insert(
            name,
            Definition {
                ty: Rc::clone(&definition.ty),
                parameters,
                body,
            },
        );
    }
    Ok(ExecutableProgram {
        entry,
        definitions,
        datatypes: engine.adts.as_ref().clone(),
        constructor_tags: constructor_tags.clone(),
        base_names: base_names.clone(),
        types: engine.clone(),
    })
}

fn error(message: impl Into<String>) -> KernelError {
    KernelError::new(format!("executable lowering: {}", message.into()))
}

fn kind(quant: Quant) -> TermRef {
    term(Term::Typ(term(Term::Qua(quant))))
}

fn variable(binder: &Binder) -> TermRef {
    term(Term::Var {
        name: binder.name.clone(),
        id: binder.id,
    })
}

#[derive(Debug)]
struct Lowerer {
    engine: Engine,
    references: BTreeSet<String>,
}

impl Lowerer {
    fn parameters(
        &mut self,
        ty: &TermRef,
        declared: &[Binder],
    ) -> Result<Vec<Binder>, KernelError> {
        let mut telescope = Rc::clone(ty);
        let mut parameters = Vec::with_capacity(declared.len());
        for parameter in declared {
            let (mut binder, body) = self.function(&telescope)?;
            let old_id = binder.id;
            binder.id = parameter.id;
            binder.name.clone_from(&parameter.name);
            telescope = substitute(&body, old_id, &variable(&binder));
            parameters.push(binder);
        }
        Ok(parameters)
    }

    fn function(&mut self, ty: &TermRef) -> Result<(Binder, TermRef), KernelError> {
        let ty = self.engine.whnf(ty)?;
        match ty.as_ref() {
            Term::All {
                quant,
                name,
                id,
                domain,
                body,
            } => Ok((
                Binder {
                    quant: *quant,
                    name: name.clone(),
                    id: *id,
                    ty: Rc::clone(domain),
                },
                Rc::clone(body),
            )),
            _ => Err(error(format!("expected function type; observed {ty}"))),
        }
    }

    fn datatype(&mut self, ty: &TermRef) -> Result<(AdtDecl, TermRef), KernelError> {
        let normalized = self.engine.whnf(ty)?;
        let Term::Adt {
            name,
            args,
            excluded,
        } = normalized.as_ref()
        else {
            return Err(error(format!("expected datatype; observed {normalized}")));
        };
        let mut declaration = self
            .engine
            .adts
            .get(name)
            .cloned()
            .ok_or_else(|| error(format!("undefined datatype {name}")))?;
        if args.len() != declaration.parameters.len() {
            return Err(error("datatype argument count changed after checking"));
        }
        declaration
            .constructors
            .retain(|constructor| !excluded.contains(&constructor.name));
        Ok((declaration, normalized))
    }

    fn fields(
        datatype: &AdtDecl,
        constructor: &ConstructorDecl,
        parameters: &[TermRef],
    ) -> Vec<Binder> {
        constructor
            .fields
            .iter()
            .map(|field| {
                let mut field = field.clone();
                for (parameter, argument) in datatype.parameters.iter().zip(parameters) {
                    field.ty = substitute(&field.ty, parameter.id, argument);
                }
                field
            })
            .collect()
    }

    /// Reconstruct an inferred type without rechecking usage or recursion. This
    /// function is private and receives terms only from a checked executable.
    fn infer(&mut self, value: &TermRef, context: &Context) -> Result<TermRef, KernelError> {
        self.engine.step()?;
        self.engine.enter()?;
        let result = self.infer_inner(value, context);
        self.engine.depth -= 1;
        result
    }

    fn infer_inner(&mut self, value: &TermRef, context: &Context) -> Result<TermRef, KernelError> {
        match value.as_ref() {
            Term::Var { id, .. } => context
                .get(id)
                .map(|binder| Rc::clone(&binder.ty))
                .ok_or_else(|| error(format!("unbound variable {value}"))),
            Term::Ref(name) => self.engine.adts.get(name).map_or_else(
                || {
                    self.engine
                        .defs
                        .get(name)
                        .map(|definition| Rc::clone(&definition.ty))
                        .ok_or_else(|| error(format!("undefined reference {name}")))
                },
                |datatype| Ok(Rc::clone(&datatype.kind)),
            ),
            Term::Typ(_) | Term::Qnt | Term::All { .. } => Ok(kind(Quant::Lone)),
            Term::Qua(_) | Term::Min(_, _) => Ok(term(Term::Qnt)),
            Term::Adt { name, args, .. } => {
                let datatype = self
                    .engine
                    .adts
                    .get(name)
                    .ok_or_else(|| error(format!("undefined datatype {name}")))?;
                let mut result = Rc::clone(&datatype.kind);
                for (parameter, argument) in datatype.parameters.iter().zip(args) {
                    result = substitute(&result, parameter.id, argument);
                }
                Ok(result)
            }
            Term::Eql { .. } => Ok(kind(Quant::Many)),
            Term::Ann(_, ty) => Ok(Rc::clone(ty)),
            Term::App(function, argument) => {
                if let Term::Lam { id, body, .. } = function.as_ref() {
                    return self.infer(&substitute(body, *id, argument), context);
                }
                let function_type = self.infer(function, context)?;
                let (parameter, body) = self.function(&function_type)?;
                Ok(substitute(&body, parameter.id, argument))
            }
            _ => Err(error(format!(
                "expression requires an expected type: {value}"
            ))),
        }
    }

    fn expression(
        &mut self,
        value: &TermRef,
        ty: &TermRef,
        demand: Quant,
        context: &Context,
    ) -> Result<Expression, KernelError> {
        self.engine.step()?;
        self.engine.enter()?;
        let result = self.expression_inner(value, ty, demand, context);
        self.engine.depth -= 1;
        result.map(|kind| Expression {
            ty: Rc::clone(ty),
            kind,
        })
    }

    #[expect(
        clippy::too_many_lines,
        reason = "An exhaustive term traversal makes executable erasure and expected-type propagation reviewable together."
    )]
    fn expression_inner(
        &mut self,
        value: &TermRef,
        ty: &TermRef,
        demand: Quant,
        context: &Context,
    ) -> Result<ExpressionKind, KernelError> {
        if demand == Quant::None
            || matches!(
                self.engine.whnf(ty)?.as_ref(),
                Term::Typ(_) | Term::Qnt | Term::Eql { .. }
            )
        {
            return Ok(ExpressionKind::Erased);
        }
        match value.as_ref() {
            Term::Var { id, .. } => {
                if !context.contains_key(id) {
                    return Err(error(format!("unbound variable {value}")));
                }
                Ok(ExpressionKind::Variable(*id))
            }
            Term::Ref(name) => {
                if !self.engine.defs.contains_key(name) {
                    return Err(error(format!("unbound runtime definition {name}")));
                }
                self.references.insert(name.clone());
                Ok(ExpressionKind::Definition(name.clone()))
            }
            Term::Lam { name, id, body } => {
                let (mut parameter, result_type) = self.function(ty)?;
                let type_id = parameter.id;
                parameter.name.clone_from(name);
                parameter.id = *id;
                let result_type = substitute(&result_type, type_id, &variable(&parameter));
                let mut inner = context.clone();
                if inner.insert(*id, parameter.clone()).is_some() {
                    return Err(error("duplicate lambda binder after checking"));
                }
                Ok(ExpressionKind::Lambda {
                    body: Box::new(self.expression(body, &result_type, demand, &inner)?),
                    parameter,
                })
            }
            Term::App(function, argument) => {
                // The checker uses this same beta substitution to infer an
                // unannotated lambda application (notably template fills).
                if let Term::Lam { id, body, .. } = function.as_ref() {
                    return self
                        .expression(&substitute(body, *id, argument), ty, demand, context)
                        .map(|expression| expression.kind);
                }
                let function_type = self.infer(function, context)?;
                let (parameter, _) = self.function(&function_type)?;
                Ok(ExpressionKind::Apply {
                    function: Box::new(self.expression(
                        function,
                        &function_type,
                        demand,
                        context,
                    )?),
                    argument: Box::new(self.expression(
                        argument,
                        &parameter.ty,
                        parameter.quant.demand(demand),
                        context,
                    )?),
                    quant: parameter.quant,
                })
            }
            Term::Ctr { name, args } => self.constructor(name, args, ty, demand, context),
            Term::Mat { .. } | Term::Efq => self.match_expression(value, ty, demand, context),
            Term::Let { bindings, body } => {
                let mut inner = context.clone();
                let mut lowered = Vec::with_capacity(bindings.len());
                for binding in bindings {
                    let ty = self.infer(&binding.value, context)?;
                    let binder = Binder {
                        quant: binding.quant,
                        name: binding.name.clone(),
                        id: binding.id,
                        ty,
                    };
                    let value = self.expression(
                        &binding.value,
                        &binder.ty,
                        binder.quant.demand(demand),
                        context,
                    )?;
                    if inner.insert(binder.id, binder.clone()).is_some() {
                        return Err(error("duplicate let binder after checking"));
                    }
                    lowered.push(Field { binder, value });
                }
                for binding in bindings {
                    self.engine
                        .aliases
                        .insert(binding.id, Rc::clone(&binding.value));
                }
                let result = self.expression(body, ty, demand, &inner);
                for binding in bindings {
                    self.engine.aliases.remove(&binding.id);
                }
                Ok(ExpressionKind::Let {
                    bindings: lowered,
                    body: Box::new(result?),
                })
            }
            Term::Ann(body, annotation) => self
                .expression(body, annotation, demand, context)
                .map(|expression| expression.kind),
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                let evidence_type = self.infer(evidence, context)?;
                let evidence_type = self.engine.whnf(&evidence_type)?;
                let Term::Eql { left, .. } = evidence_type.as_ref() else {
                    return Err(error("rewrite evidence has no equality type"));
                };
                let premise = apply(Rc::clone(motive), [Rc::clone(left), term(Term::Rfl)]);
                self.expression(body, &premise, demand, context)
                    .map(|expression| expression.kind)
            }
            Term::Hole(name) => Err(error(format!("unexpected checked hole ?{name}"))),
            Term::Typ(_)
            | Term::Qnt
            | Term::Qua(_)
            | Term::Min(_, _)
            | Term::All { .. }
            | Term::Adt { .. }
            | Term::Eql { .. }
            | Term::Rfl => Err(error("non-runtime term retained at a live data type")),
        }
    }

    fn constructor(
        &mut self,
        name: &str,
        arguments: &[TermRef],
        ty: &TermRef,
        demand: Quant,
        context: &Context,
    ) -> Result<ExpressionKind, KernelError> {
        let (datatype, normalized) = self.datatype(ty)?;
        let Term::Adt { args, .. } = normalized.as_ref() else {
            unreachable!()
        };
        let constructor = datatype
            .constructors
            .iter()
            .find(|constructor| constructor.name == name)
            .ok_or_else(|| {
                error(format!(
                    "constructor {name} is absent from {}",
                    datatype.name
                ))
            })?;
        if arguments.len() != constructor.fields.len() {
            return Err(error("constructor field count changed after checking"));
        }
        let fields = Self::fields(&datatype, constructor, args);
        let mut substitutions = Vec::new();
        let mut lowered = Vec::with_capacity(fields.len());
        for (mut binder, argument) in fields.into_iter().zip(arguments) {
            for (id, replacement) in &substitutions {
                binder.ty = substitute(&binder.ty, *id, replacement);
            }
            let value =
                self.expression(argument, &binder.ty, binder.quant.demand(demand), context)?;
            substitutions.push((binder.id, Rc::clone(argument)));
            lowered.push(Field { binder, value });
        }
        Ok(ExpressionKind::Constructor {
            owner: datatype.name,
            name: name.into(),
            fields: lowered,
        })
    }

    fn match_expression(
        &mut self,
        value: &TermRef,
        ty: &TermRef,
        demand: Quant,
        context: &Context,
    ) -> Result<ExpressionKind, KernelError> {
        let (parameter, codomain) = self.function(ty)?;
        let (datatype, normalized) = self.datatype(&parameter.ty)?;
        if matches!(value.as_ref(), Term::Efq) {
            return Ok(ExpressionKind::Absurd {
                parameter,
                owner: datatype.name,
            });
        }
        let Term::Mat {
            constructor,
            arm,
            fallback,
        } = value.as_ref()
        else {
            return Err(error("unexpected match representation"));
        };
        let Term::Adt { args, excluded, .. } = normalized.as_ref() else {
            unreachable!()
        };
        let declaration = datatype
            .constructors
            .iter()
            .find(|field| field.name == *constructor)
            .ok_or_else(|| error(format!("constructor {constructor} is not a remaining case")))?;
        let mut substitutions = Vec::new();
        let mut fields = Vec::new();
        let mut values = Vec::new();
        for mut field in Self::fields(&datatype, declaration, args) {
            let fresh = self.engine.fresh(&field.name)?;
            let Term::Var { id, .. } = fresh.as_ref() else {
                unreachable!()
            };
            for (old, replacement) in &substitutions {
                field.ty = substitute(&field.ty, *old, replacement);
            }
            substitutions.push((field.id, Rc::clone(&fresh)));
            field.id = *id;
            field.quant = field.quant.product(parameter.quant);
            fields.push(field);
            values.push(fresh);
        }
        let constructed = term(Term::Ctr {
            name: constructor.clone(),
            args: values,
        });
        let arm_type = arrows(&fields, substitute(&codomain, parameter.id, &constructed));
        let arm = self.expression(arm, &arm_type, demand, context)?;
        let mut excluded = excluded.clone();
        excluded.push(constructor.clone());
        let mut residual = parameter.clone();
        residual.ty = term(Term::Adt {
            name: datatype.name.clone(),
            args: args.clone(),
            excluded,
        });
        let fallback_type = arrows(&[residual], codomain);
        let fallback = self.expression(fallback, &fallback_type, demand, context)?;
        Ok(ExpressionKind::Match {
            parameter,
            owner: datatype.name,
            constructor: constructor.clone(),
            fields,
            arm: Box::new(arm),
            fallback: Box::new(fallback),
        })
    }
}

#[cfg(test)]
#[path = "elaborate_tests.rs"]
mod tests;
