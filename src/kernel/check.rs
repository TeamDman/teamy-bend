// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use super::AdtDecl;
use super::Binder;
use super::Book;
use super::ConstructorDecl;
use super::Declaration;
use super::DefDecl;
use super::Quant;
use super::Term;
use super::TermRef;
use super::arrows;
use super::lambdas;
use super::reduce::Comparison;
use super::reduce::apply;
use super::reduce::spine;
use super::substitute;
use super::term;
use super::term::inspect;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

type Uses = BTreeMap<usize, Quant>;
type Context = BTreeMap<usize, Binder>;

/// A rejected or resource-limited proof check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelError {
    pub message: String,
}
impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl Error for KernelError {}
impl KernelError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Definitions can be evaluated only after the complete book passes checking.
#[derive(Clone, Debug)]
pub struct CheckedBook {
    engine: Engine,
}

impl CheckedBook {
    /// Look up a checked definition's complete dependent function type.
    #[must_use]
    pub fn definition_type(&self, name: &str) -> Option<&TermRef> {
        self.engine.defs.get(name).map(|d| &d.ty)
    }

    /// Evaluate a completely applied checked definition with checked arguments.
    ///
    /// # Errors
    /// Rejects unknown definitions, incorrect arguments, and exhausted limits.
    pub fn evaluate(&self, name: &str, args: &[TermRef]) -> Result<TermRef, KernelError> {
        let mut engine = self.engine.clone();
        engine.reset();
        for arg in args {
            let highest = inspect(arg)?;
            engine.fresh_id = engine.fresh_id.max(
                highest
                    .checked_add(1)
                    .ok_or_else(|| KernelError::new("binder identifier space exhausted"))?,
            );
        }
        let def = engine
            .defs
            .get(name)
            .cloned()
            .ok_or_else(|| KernelError::new(format!("undefined definition {name}")))?;
        if args.len() != def.parameters.len() {
            return Err(KernelError::new(format!(
                "{name} needs {} arguments; got {}",
                def.parameters.len(),
                args.len()
            )));
        }
        let mut ty = Rc::clone(&def.ty);
        let lhs = Lhs::default();
        for arg in args {
            let t = engine.whnf(&ty)?;
            let Term::All {
                quant,
                id,
                domain,
                body,
                ..
            } = t.as_ref()
            else {
                return Err(KernelError::new("argument supplied to a non-function"));
            };
            engine.check(
                &lhs,
                arg,
                quant.demand(Quant::Lone),
                domain,
                &Context::new(),
            )?;
            ty = substitute(body, *id, arg);
        }
        engine.normalize(&apply(
            term(Term::Ref(name.into())),
            args.iter().map(Rc::clone),
        ))
    }

    /// Names of all checked definitions, in lexical order.
    pub fn definition_names(&self) -> impl Iterator<Item = &str> {
        self.engine.defs.keys().map(String::as_str)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Engine {
    pub(crate) defs: BTreeMap<String, DefDecl>,
    pub(crate) adts: BTreeMap<String, AdtDecl>,
    pub(crate) aliases: BTreeMap<usize, TermRef>,
    fresh_id: usize,
    fuel: usize,
    pub(crate) depth: usize,
}

impl Engine {
    fn reset(&mut self) {
        self.fuel = 2_000_000;
        self.depth = 0;
        self.aliases.clear();
    }
    pub(crate) fn step(&mut self) -> Result<(), KernelError> {
        self.fuel = self.fuel.checked_sub(1).ok_or_else(|| {
            KernelError::new("kernel evaluation budget exhausted; proof not accepted")
        })?;
        Ok(())
    }
    pub(crate) fn enter(&mut self) -> Result<(), KernelError> {
        if self.depth >= 128 {
            return Err(KernelError::new(
                "kernel nesting limit exhausted; proof not accepted",
            ));
        }
        self.depth += 1;
        Ok(())
    }
    pub(crate) fn fresh(&mut self, name: &str) -> Result<TermRef, KernelError> {
        let id = self.fresh_id;
        self.fresh_id = self
            .fresh_id
            .checked_add(1)
            .ok_or_else(|| KernelError::new("binder identifier space exhausted"))?;
        Ok(term(Term::Var {
            name: name.into(),
            id,
        }))
    }
}

#[derive(Clone, Debug)]
struct Lhs {
    name: String,
    equation: TermRef,
    remaining: usize,
    quants: Vec<Quant>,
}
impl Default for Lhs {
    fn default() -> Self {
        Self {
            name: String::new(),
            equation: term(Term::Ref(String::new())),
            remaining: 0,
            quants: vec![],
        }
    }
}

fn kind(q: Quant) -> TermRef {
    term(Term::Typ(term(Term::Qua(q))))
}
fn usage(id: usize, q: Quant) -> Uses {
    BTreeMap::from([(id, q)])
}
fn add(mut a: Uses, b: Uses) -> Uses {
    for (id, q) in b {
        let old = a.get(&id).copied().unwrap_or(Quant::None);
        a.insert(id, old.add(q));
    }
    a
}
fn join(mut a: Uses, b: Uses) -> Uses {
    for (id, q) in b {
        let old = a.get(&id).copied().unwrap_or(Quant::None);
        a.insert(id, old.max(q));
    }
    a
}
fn check_usage(binder: &Binder, uses: &mut Uses) -> Result<(), KernelError> {
    let observed = uses.remove(&binder.id).unwrap_or(Quant::None);
    if observed > binder.quant {
        Err(KernelError::new(format!(
            "binder {} permits {:?} use, observed {:?}",
            binder.name, binder.quant, observed
        )))
    } else {
        Ok(())
    }
}
fn mismatch(expected: &TermRef, observed: &TermRef) -> KernelError {
    KernelError::new(format!(
        "type mismatch: expected {expected}; observed {observed}"
    ))
}

/// Check every declaration and require every law to have a proof.
///
/// # Errors
/// Rejects ill-typed programs, incomplete laws, holes, unsafe/foreign assumptions,
/// invalid resource use or recursion, and exhausted checking limits.
pub fn check_book(book: &Book) -> Result<CheckedBook, KernelError> {
    let mut highest = 0;
    for declaration in &book.declarations {
        match declaration {
            Declaration::Def(d) => {
                highest = highest.max(inspect(&d.ty)?);
                if let Some(b) = &d.body {
                    highest = highest.max(inspect(b)?);
                }
            }
            Declaration::Adt(d) => {
                highest = highest.max(inspect(&d.kind)?);
                for b in d
                    .parameters
                    .iter()
                    .chain(d.constructors.iter().flat_map(|c| &c.fields))
                {
                    highest = highest.max(b.id).max(inspect(&b.ty)?);
                }
            }
        }
    }
    let mut engine = Engine {
        defs: BTreeMap::new(),
        adts: BTreeMap::new(),
        aliases: BTreeMap::new(),
        fresh_id: highest
            .checked_add(1)
            .ok_or_else(|| KernelError::new("binder identifier space exhausted"))?,
        fuel: 2_000_000,
        depth: 0,
    };
    for declaration in &book.declarations {
        engine.fuel = 2_000_000;
        let result = match declaration {
            Declaration::Adt(d) => engine.validate_adt(d),
            Declaration::Def(d) => engine.validate_def(d),
        };
        if let Err(error) = result {
            let name = match declaration {
                Declaration::Adt(d) => &d.name,
                Declaration::Def(d) => &d.name,
            };
            return Err(KernelError::new(format!("{name}: {error}")));
        }
    }
    let open: Vec<_> = engine
        .defs
        .values()
        .filter(|d| d.body.is_none())
        .map(|d| d.name.as_str())
        .collect();
    if !open.is_empty() {
        return Err(KernelError::new(format!(
            "unfilled laws: {}",
            open.join(", ")
        )));
    }
    Ok(CheckedBook { engine })
}

impl Engine {
    fn validate_def(&mut self, definition: &DefDecl) -> Result<(), KernelError> {
        if definition.unsafe_ || definition.foreign {
            return Err(KernelError::new(
                "strict proof checking rejects unsafe and foreign definitions",
            ));
        }
        if self.adts.contains_key(&definition.name) {
            return Err(KernelError::new("definition collides with a datatype"));
        }
        if let Some(previous) = self.defs.get(&definition.name).cloned() {
            if previous.body.is_some() || definition.body.is_none() {
                return Err(KernelError::new("duplicate declaration"));
            }
            if previous.parameters.len() != definition.parameters.len()
                || !self.compare(Comparison::Equal, &previous.ty, &definition.ty)?
            {
                return Err(KernelError::new("proof does not preserve its law's type"));
            }
        }
        let mut pending = definition.clone();
        pending.body = None;
        self.defs.insert(definition.name.clone(), pending);
        let lhs = Lhs {
            name: definition.name.clone(),
            equation: term(Term::Ref(definition.name.clone())),
            remaining: 0,
            quants: vec![],
        };
        self.check(
            &lhs,
            &definition.ty,
            Quant::None,
            &kind(Quant::Lone),
            &Context::new(),
        )?;
        let mut telescope = Rc::clone(&definition.ty);
        let mut quants = Vec::new();
        for _ in &definition.parameters {
            let t = self.whnf(&telescope)?;
            let Term::All { quant, body, .. } = t.as_ref() else {
                return Err(KernelError::new(
                    "parameter list exceeds the function telescope",
                ));
            };
            quants.push(*quant);
            telescope = Rc::clone(body);
        }
        if let Some(body) = &definition.body {
            let lhs = Lhs {
                remaining: definition.parameters.len(),
                quants,
                ..lhs
            };
            self.check(&lhs, body, Quant::Lone, &definition.ty, &Context::new())?;
        }
        self.defs
            .insert(definition.name.clone(), definition.clone());
        Ok(())
    }

    fn validate_adt(&mut self, declaration: &AdtDecl) -> Result<(), KernelError> {
        if self.adts.contains_key(&declaration.name) || self.defs.contains_key(&declaration.name) {
            return Err(KernelError::new("duplicate datatype"));
        }
        for constructor in &declaration.constructors {
            if self.constructor_family(&constructor.name).is_some()
                || declaration
                    .constructors
                    .iter()
                    .filter(|c| c.name == constructor.name)
                    .count()
                    != 1
            {
                return Err(KernelError::new(format!(
                    "duplicate constructor {}",
                    constructor.name
                )));
            }
        }
        self.adts
            .insert(declaration.name.clone(), declaration.clone());
        let lhs = Lhs::default();
        self.check(
            &lhs,
            &arrows(&declaration.parameters, Rc::clone(&declaration.kind)),
            Quant::None,
            &kind(Quant::Lone),
            &Context::new(),
        )?;
        let declared_kind = self.whnf(&declaration.kind)?;
        if !matches!(declared_kind.as_ref(), Term::Typ(_)) {
            return Err(KernelError::new(
                "datatype declaration must end in Kind(g), Type, or Data",
            ));
        }
        for constructor in &declaration.constructors {
            let mut context = Context::new();
            for (i, binder) in declaration
                .parameters
                .iter()
                .chain(&constructor.fields)
                .enumerate()
            {
                let expected = if i >= declaration.parameters.len() && binder.quant == Quant::Lone {
                    Rc::clone(&declaration.kind)
                } else {
                    kind(binder.quant)
                };
                self.check(&lhs, &binder.ty, Quant::None, &expected, &context)?;
                if context.insert(binder.id, binder.clone()).is_some() {
                    return Err(KernelError::new("duplicate binder identifier"));
                }
            }
        }
        Ok(())
    }

    fn constructor_family(&self, name: &str) -> Option<String> {
        self.adts
            .values()
            .find(|a| a.constructors.iter().any(|c| c.name == name))
            .map(|a| a.name.clone())
    }

    fn adt(&self, value: &TermRef) -> Result<AdtDecl, KernelError> {
        let Term::Adt {
            name,
            args,
            excluded,
        } = value.as_ref()
        else {
            return Err(KernelError::new(format!(
                "expected a datatype; observed {value}"
            )));
        };
        let mut adt = self
            .adts
            .get(name)
            .cloned()
            .ok_or_else(|| KernelError::new(format!("undefined datatype {name}")))?;
        if args.len() != adt.parameters.len() {
            return Err(KernelError::new(format!(
                "{name} needs {} type parameters",
                adt.parameters.len()
            )));
        }
        if excluded
            .iter()
            .any(|n| !adt.constructors.iter().any(|c| &c.name == n))
        {
            return Err(KernelError::new("unknown excluded constructor"));
        }
        adt.constructors.retain(|c| !excluded.contains(&c.name));
        Ok(adt)
    }

    fn fields(adt: &AdtDecl, constructor: &ConstructorDecl, args: &[TermRef]) -> Vec<Binder> {
        constructor
            .fields
            .iter()
            .map(|field| {
                let mut result = field.clone();
                for (parameter, arg) in adt.parameters.iter().zip(args) {
                    result.ty = substitute(&result.ty, parameter.id, arg);
                }
                result
            })
            .collect()
    }

    fn infer(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        context: &Context,
        arguments: &[TermRef],
    ) -> Result<(TermRef, Uses), KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.infer_inner(lhs, value, demand, context, arguments);
        self.depth -= 1;
        result
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Keeping the bidirectional inference rules in one exhaustive match makes the trusted kernel auditable."
    )]
    fn infer_inner(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        context: &Context,
        arguments: &[TermRef],
    ) -> Result<(TermRef, Uses), KernelError> {
        match value.as_ref() {
            Term::Var { id, .. } => {
                let b = context
                    .get(id)
                    .ok_or_else(|| KernelError::new(format!("unbound variable {value}")))?;
                Ok((Rc::clone(&b.ty), usage(*id, demand)))
            }
            Term::Ref(name) => {
                if let Some(adt) = self.adts.get(name) {
                    if !adt.parameters.is_empty() {
                        return Err(KernelError::new(
                            "a parameterized datatype needs <...> arguments",
                        ));
                    }
                    return Ok((Rc::clone(&adt.kind), Uses::new()));
                }
                let def = self
                    .defs
                    .get(name)
                    .cloned()
                    .ok_or_else(|| KernelError::new(format!("undefined name {name}")))?;
                if demand != Quant::None {
                    if name == &lhs.name {
                        let (_, columns) = spine(&lhs.equation);
                        let mut order = std::cmp::Ordering::Equal;
                        for ((arg, col), quant) in arguments.iter().zip(&columns).zip(&lhs.quants) {
                            if order != std::cmp::Ordering::Equal {
                                break;
                            }
                            order = descend(*quant, arg, col);
                        }
                        if order != std::cmp::Ordering::Less {
                            return Err(KernelError::new(
                                "recursive self-call must decrease structurally, left to right",
                            ));
                        }
                    } else if def.body.is_none() {
                        return Err(KernelError::new(format!(
                            "unfilled law {name} cannot be used as live evidence"
                        )));
                    }
                }
                Ok((def.ty, Uses::new()))
            }
            Term::Typ(g) => {
                self.check(lhs, g, Quant::None, &term(Term::Qnt), context)?;
                Ok((kind(Quant::Lone), Uses::new()))
            }
            Term::Qnt => Ok((kind(Quant::Lone), Uses::new())),
            Term::Qua(_) => Ok((term(Term::Qnt), Uses::new())),
            Term::Min(a, b) => {
                let a = self.check(lhs, a, demand, &term(Term::Qnt), context)?;
                let b = self.check(lhs, b, demand, &term(Term::Qnt), context)?;
                Ok((term(Term::Qnt), add(a, b)))
            }
            Term::All {
                quant,
                name,
                id,
                domain,
                body,
            } => {
                self.check(lhs, domain, Quant::None, &kind(*quant), context)?;
                let mut context = context.clone();
                if context
                    .insert(
                        *id,
                        Binder {
                            quant: *quant,
                            name: name.clone(),
                            id: *id,
                            ty: Rc::clone(domain),
                        },
                    )
                    .is_some()
                {
                    return Err(KernelError::new("duplicate binder identifier"));
                }
                self.check(lhs, body, Quant::None, &kind(Quant::Lone), &context)?;
                Ok((kind(Quant::Lone), Uses::new()))
            }
            Term::App(f, x) => {
                if let Term::Lam { id, body, .. } = f.as_ref() {
                    return self.infer(lhs, &substitute(body, *id, x), demand, context, arguments);
                }
                let mut args = vec![Rc::clone(x)];
                args.extend(arguments.iter().map(Rc::clone));
                let (ft, fu) = self.infer(lhs, f, demand, context, &args)?;
                let ft = self.whnf(&ft)?;
                let Term::All {
                    quant,
                    id,
                    domain,
                    body,
                    ..
                } = ft.as_ref()
                else {
                    return Err(KernelError::new(format!(
                        "expected a function type; observed {ft}"
                    )));
                };
                let xu = self.check(lhs, x, quant.demand(demand), domain, context)?;
                Ok((substitute(body, *id, x), add(fu, xu)))
            }
            Term::Adt { name, args, .. } => {
                let adt = self.adt(value)?;
                let telescope = arrows(&adt.parameters, Rc::clone(&adt.kind));
                let (ty, uses) = self.check_arguments(lhs, &telescope, args, demand, context)?;
                if name != &adt.name {
                    return Err(KernelError::new("datatype name mismatch"));
                }
                Ok((ty, uses))
            }
            Term::Eql { left, right, ty } => {
                self.check(lhs, ty, Quant::None, &kind(Quant::Lone), context)?;
                self.check(lhs, left, Quant::None, ty, context)?;
                self.check(lhs, right, Quant::None, ty, context)?;
                Ok((kind(Quant::Many), Uses::new()))
            }
            Term::Ann(x, ty) => {
                self.check(lhs, ty, Quant::None, &kind(Quant::Lone), context)?;
                let uses = self.check(lhs, x, demand, ty, context)?;
                Ok((Rc::clone(ty), uses))
            }
            Term::Hole(name) => Err(KernelError::new(format!("unfinished proof hole ?{name}"))),
            _ => Err(KernelError::new(format!(
                "term needs an expected type or annotation: {value}"
            ))),
        }
    }

    fn check_arguments(
        &mut self,
        lhs: &Lhs,
        telescope: &TermRef,
        args: &[TermRef],
        demand: Quant,
        context: &Context,
    ) -> Result<(TermRef, Uses), KernelError> {
        let mut tel = Rc::clone(telescope);
        let mut uses = Uses::new();
        for arg in args {
            let t = self.whnf(&tel)?;
            let Term::All {
                quant,
                id,
                domain,
                body,
                ..
            } = t.as_ref()
            else {
                return Err(KernelError::new("too many arguments for telescope"));
            };
            uses = add(
                uses,
                self.check(lhs, arg, quant.demand(demand), domain, context)?,
            );
            tel = substitute(body, *id, arg);
        }
        Ok((tel, uses))
    }

    fn check(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        ty: &TermRef,
        context: &Context,
    ) -> Result<Uses, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.check_inner(lhs, value, demand, ty, context);
        self.depth -= 1;
        result
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Keeping the bidirectional checking rules in one exhaustive match makes the trusted kernel auditable."
    )]
    fn check_inner(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        ty: &TermRef,
        context: &Context,
    ) -> Result<Uses, KernelError> {
        match value.as_ref() {
            Term::Hole(name) => {
                return Err(KernelError::new(format!("unfinished proof hole ?{name}")));
            }
            Term::Lam { name, id, body } => {
                let t = self.whnf(ty)?;
                let Term::All {
                    quant,
                    id: tid,
                    domain,
                    body: codomain,
                    ..
                } = t.as_ref()
                else {
                    return Err(mismatch(ty, value));
                };
                let x = self.fresh(name)?;
                let Term::Var { id: xid, .. } = x.as_ref() else {
                    unreachable!()
                };
                let binder = Binder {
                    quant: *quant,
                    name: name.clone(),
                    id: *xid,
                    ty: Rc::clone(domain),
                };
                let mut next = context.clone();
                next.insert(*xid, binder.clone());
                let mut lhs = lhs.clone();
                if lhs.remaining > 0 {
                    lhs.equation = apply(lhs.equation, [Rc::clone(&x)]);
                    lhs.remaining -= 1;
                }
                let mut uses = self.check(
                    &lhs,
                    &substitute(body, *id, &x),
                    demand,
                    &substitute(codomain, *tid, &x),
                    &next,
                )?;
                check_usage(&binder, &mut uses)?;
                return Ok(uses);
            }
            Term::Let { bindings, body } => {
                let mut next = context.clone();
                let mut uses = Uses::new();
                let mut binders = Vec::new();
                for b in bindings {
                    if next.contains_key(&b.id) {
                        return Err(KernelError::new("duplicate let binder identifier"));
                    }
                    let (t, u) = self.infer(lhs, &b.value, b.quant.demand(demand), context, &[])?;
                    self.check(lhs, &t, Quant::None, &kind(b.quant), context)?;
                    let binder = Binder {
                        quant: b.quant,
                        name: b.name.clone(),
                        id: b.id,
                        ty: t,
                    };
                    next.insert(b.id, binder.clone());
                    binders.push(binder);
                    uses = add(uses, u);
                }
                for b in bindings {
                    self.aliases.insert(b.id, Rc::clone(&b.value));
                }
                let result = self.check(lhs, body, demand, ty, &next);
                for b in bindings {
                    self.aliases.remove(&b.id);
                }
                let mut body_uses = result?;
                for b in &binders {
                    check_usage(b, &mut body_uses)?;
                }
                return Ok(add(uses, body_uses));
            }
            Term::Ctr { name, args } => {
                let t = self.whnf(ty)?;
                let adt = self.adt(&t)?;
                let constructor = adt
                    .constructors
                    .iter()
                    .find(|c| &c.name == name)
                    .ok_or_else(|| {
                        KernelError::new(format!(
                            "{name} is not a remaining constructor of {}",
                            adt.name
                        ))
                    })?;
                if constructor.fields.len() != args.len() {
                    return Err(KernelError::new(format!(
                        "{name} needs {} fields; got {}",
                        constructor.fields.len(),
                        args.len()
                    )));
                }
                let Term::Adt {
                    args: parameters, ..
                } = t.as_ref()
                else {
                    unreachable!()
                };
                let telescope = arrows(&Self::fields(&adt, constructor, parameters), Rc::clone(&t));
                return self
                    .check_arguments(lhs, &telescope, args, demand, context)
                    .map(|(_, u)| u);
            }
            Term::Mat { .. } | Term::Efq => {
                return self.check_match(lhs, value, demand, ty, context);
            }
            Term::Rfl => {
                let t = self.whnf(ty)?;
                let Term::Eql { left, right, .. } = t.as_ref() else {
                    return Err(mismatch(ty, value));
                };
                if !self.compare(Comparison::Equal, left, right)? {
                    return Err(KernelError::new(format!(
                        "reflexivity requires equal endpoints: {left} != {right}"
                    )));
                }
                return Ok(Uses::new());
            }
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                let (et, eu) = self.infer(lhs, evidence, demand, context, &[])?;
                let et = self.whnf(&et)?;
                let Term::Eql {
                    left,
                    right,
                    ty: domain,
                } = et.as_ref()
                else {
                    return Err(KernelError::new("rewrite evidence must have equality type"));
                };
                let x = self.fresh("_")?;
                let e = self.fresh("equation")?;
                let Term::Var { id: xi, .. } = x.as_ref() else {
                    unreachable!()
                };
                let Term::Var { id: ei, .. } = e.as_ref() else {
                    unreachable!()
                };
                let mt = arrows(
                    &[
                        Binder {
                            quant: Quant::Lone,
                            name: "_".into(),
                            id: *xi,
                            ty: Rc::clone(domain),
                        },
                        Binder {
                            quant: Quant::Lone,
                            name: "equation".into(),
                            id: *ei,
                            ty: term(Term::Eql {
                                left: Rc::clone(left),
                                right: x,
                                ty: Rc::clone(domain),
                            }),
                        },
                    ],
                    kind(Quant::Lone),
                );
                self.check(lhs, motive, Quant::None, &mt, context)?;
                let goal = apply(Rc::clone(motive), [Rc::clone(right), Rc::clone(evidence)]);
                if !self.compare(Comparison::Fits, &goal, ty)? {
                    return Err(mismatch(ty, &goal));
                }
                let premise = apply(Rc::clone(motive), [Rc::clone(left), term(Term::Rfl)]);
                let bu = self.check(lhs, body, demand, &premise, context)?;
                return Ok(add(eu, bu));
            }
            _ => (),
        }
        let (observed, uses) = self.infer(lhs, value, demand, context, &[])?;
        if self.compare(Comparison::Fits, &observed, ty)? {
            Ok(uses)
        } else {
            Err(mismatch(ty, &observed))
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "The match rule constructs arm and residual goals together to keep quantity and termination transitions visible."
    )]
    fn check_match(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        ty: &TermRef,
        context: &Context,
    ) -> Result<Uses, KernelError> {
        let t = self.whnf(ty)?;
        let Term::All {
            quant,
            name,
            id,
            domain,
            body,
        } = t.as_ref()
        else {
            return Err(mismatch(ty, value));
        };
        if demand != Quant::None && *quant == Quant::None {
            return Err(KernelError::new(
                "an erased scrutinee cannot determine live evidence",
            ));
        }
        let dt = self.whnf(domain)?;
        let adt = self.adt(&dt)?;
        if matches!(value.as_ref(), Term::Efq) {
            if adt.constructors.is_empty() || self.context_dead(context)? {
                return Ok(Uses::new());
            }
            return Err(KernelError::new(format!(
                "missing cases for {}",
                adt.constructors
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        let Term::Mat {
            constructor,
            arm,
            fallback,
        } = value.as_ref()
        else {
            unreachable!()
        };
        let ctr = adt
            .constructors
            .iter()
            .find(|c| &c.name == constructor)
            .ok_or_else(|| {
                KernelError::new(format!(
                    "constructor {constructor} missing or already matched"
                ))
            })?;
        let Term::Adt { args, excluded, .. } = dt.as_ref() else {
            unreachable!()
        };
        let fields = Self::fields(&adt, ctr, args);
        let mut fresh_fields = Vec::new();
        let mut substitutions = Vec::new();
        let mut field_values = Vec::new();
        for field in fields {
            let x = self.fresh(&field.name)?;
            let Term::Var { id: xid, .. } = x.as_ref() else {
                unreachable!()
            };
            let mut domain = field.ty;
            for (old, replacement) in &substitutions {
                domain = substitute(&domain, *old, replacement);
            }
            fresh_fields.push(Binder {
                quant: field.quant.product(*quant),
                name: field.name,
                id: *xid,
                ty: domain,
            });
            substitutions.push((field.id, Rc::clone(&x)));
            field_values.push(x);
        }
        let constructor_value = term(Term::Ctr {
            name: constructor.clone(),
            args: field_values,
        });
        let arm_goal = arrows(&fresh_fields, substitute(body, *id, &constructor_value));
        let mut arm_lhs = lhs.clone();
        if lhs.remaining > 0 {
            arm_lhs.equation = lambdas(
                &fresh_fields,
                apply(Rc::clone(&lhs.equation), [constructor_value]),
            );
            arm_lhs.remaining = lhs.remaining - 1 + fresh_fields.len();
        }
        let arm_uses = self.check(&arm_lhs, arm, demand, &arm_goal, context)?;
        let mut excluded = excluded.clone();
        excluded.push(constructor.clone());
        let residual = term(Term::All {
            quant: *quant,
            name: name.clone(),
            id: *id,
            domain: term(Term::Adt {
                name: adt.name,
                args: args.iter().map(Rc::clone).collect(),
                excluded,
            }),
            body: Rc::clone(body),
        });
        let fallback_uses = self.check(lhs, fallback, demand, &residual, context)?;
        Ok(join(arm_uses, fallback_uses))
    }

    fn context_dead(&mut self, context: &Context) -> Result<bool, KernelError> {
        for b in context.values() {
            if b.quant != Quant::None {
                let t = self.whnf(&b.ty)?;
                if matches!(t.as_ref(), Term::Adt { .. }) && self.adt(&t)?.constructors.is_empty() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

fn strip(value: &TermRef) -> &TermRef {
    if let Term::Ann(x, _) = value.as_ref() {
        strip(x)
    } else {
        value
    }
}
fn descend(quant: Quant, arg: &TermRef, column: &TermRef) -> std::cmp::Ordering {
    use std::cmp::Ordering::Equal;
    use std::cmp::Ordering::Greater;
    use std::cmp::Ordering::Less;
    if quant == Quant::None {
        return Equal;
    }
    let a = strip(arg);
    let p = strip(column);
    match p.as_ref() {
        Term::Var { id: pi, .. } => {
            if matches!(a.as_ref(),Term::Var{id:ai,..}if ai==pi) {
                Equal
            } else {
                Greater
            }
        }
        Term::Ctr { name: pn, args: px } => {
            if let Term::Ctr { name: an, args: ax } = a.as_ref()
                && an == pn
                && ax.len() == px.len()
            {
                let mut order = Equal;
                for (x, y) in ax.iter().zip(px) {
                    let field = descend(Quant::Lone, x, y);
                    if field != Equal {
                        order = field;
                    }
                    if order == Greater {
                        break;
                    }
                }
                if order != Greater {
                    return order;
                }
            }
            if px.iter().any(|p| descend(Quant::Lone, a, p) != Greater) {
                Less
            } else {
                Greater
            }
        }
        _ => Greater,
    }
}
