// SPDX-License-Identifier: MPL-2.0
//! Call-by-need execution of checked bodies and execution contracts. Proof
//! checking never uses this runtime. An explicit continuation stack bounds evaluation
//! without recursive host calls, and each invocation owns its complete arena.

use crate::kernel::AdtDecl;
use crate::kernel::DefDecl;
use crate::kernel::KernelError;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::term;
use crate::syntax::executable::ForeignDefinition;
use crate::syntax::executable::NumericIntrinsic;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

mod channels;
mod executable;
mod file_handles;
mod gc;
mod host_files;
mod host_jobs;
mod host_network;
mod nat;
mod network;
mod numeric;
mod packed;
mod runtime_clock;
mod scheduler;

type ThunkId = usize;
type EnvId = usize;
const STEPS: usize = 2_000_000;
const ARENA_LIMIT: usize = 131_072;
const FRAME_LIMIT: usize = 4_096;
const OUTPUT_DEPTH: usize = 96;
const OUTPUT_NODES: usize = 16_384;

/// Only constructed by a proof or execution checker; stored syntax is immutable.
#[derive(Clone, Debug)]
pub(crate) struct Program {
    definitions: Rc<BTreeMap<String, DefDecl>>,
    datatypes: Rc<BTreeMap<String, AdtDecl>>,
    foreign: Rc<BTreeMap<String, ForeignDefinition>>,
    numeric: Rc<BTreeMap<String, NumericIntrinsic>>,
    optimizations: Rc<BTreeSet<numeric::PureOptimization>>,
    nat_optimizations: Rc<BTreeSet<nat::Operation>>,
    packed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_limits_reject_before_growing_their_arenas() {
        let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
        let mut machine = Machine::new(&program);
        machine.remaining = 0;
        machine.tick().expect_err("step budget is enforced");
        machine
            .arena
            .resize(ARENA_LIMIT, Some(Thunk::Ready(Value::Erased)));
        machine
            .allocate(Thunk::Ready(Value::Erased))
            .expect_err("thunk arena is bounded");
        machine.environments.resize_with(ARENA_LIMIT, || {
            Some(Environment {
                parent: None,
                bindings: Vec::new(),
            })
        });
        machine
            .environment(0, Vec::new())
            .expect_err("environment arena is bounded");
        let mut frames = (0..FRAME_LIMIT).map(Frame::Update).collect();
        Machine::push(&mut frames, Frame::Update(0)).expect_err("continuations are bounded");
    }
}

impl Program {
    pub(crate) fn from_checked(
        definitions: &Rc<BTreeMap<String, DefDecl>>,
        datatypes: &Rc<BTreeMap<String, AdtDecl>>,
    ) -> Self {
        Self {
            definitions: Rc::clone(definitions),
            datatypes: Rc::clone(datatypes),
            foreign: Rc::new(BTreeMap::new()),
            numeric: Rc::new(BTreeMap::new()),
            optimizations: Rc::new(BTreeSet::new()),
            nat_optimizations: Rc::new(BTreeSet::new()),
            packed: false,
        }
    }

    pub(crate) fn evaluate(&self, name: &str, args: &[TermRef]) -> Result<TermRef, KernelError> {
        let mut machine = Machine::new(self);
        let mut entry = machine.reference(name)?;
        for argument in args {
            let argument = machine.expression(Rc::clone(argument), 0)?;
            entry = machine.allocate(Thunk::Application(entry, argument))?;
        }
        machine.materialize(entry, 0)
    }
}

#[derive(Clone)]
enum Value {
    Channel(channels::Handle),
    File(file_handles::Handle),
    Socket(network::Handle),
    Listener(network::Handle),
    PackedNat(u64),
    PackedWord {
        wrapper: packed::Wrapper,
        bits: u32,
    },
    PackedBits {
        bits: u32,
        width: u8,
    },
    Constructor {
        name: String,
        fields: Vec<ThunkId>,
    },
    Closure {
        binder: usize,
        body: TermRef,
        environment: EnvId,
    },
    Match {
        constructor: String,
        arm: ThunkId,
        fallback: ThunkId,
    },
    Foreign {
        name: String,
        arguments: Vec<ThunkId>,
    },
    Numeric {
        operation: numeric::NumericOperation,
        arguments: Vec<ThunkId>,
    },
    Natural {
        operation: nat::Operation,
        arguments: Vec<ThunkId>,
    },
    Request {
        name: String,
        arguments: Vec<ThunkId>,
        continuation: ThunkId,
    },
    EmitContinuation,
    Impossible,
    Erased,
}

#[derive(Clone)]
enum Thunk {
    Expression(TermRef, EnvId),
    Application(ThunkId, ThunkId),
    Evaluating,
    Ready(Value),
}

struct Environment {
    parent: Option<EnvId>,
    bindings: Vec<(usize, ThunkId)>,
}

enum Frame {
    Update(ThunkId),
    Apply(ThunkId),
    Numeric(numeric::NumericFrame),
    Natural(nat::Evaluation),
    Match {
        constructor: String,
        arm: ThunkId,
        fallback: ThunkId,
        argument: ThunkId,
    },
}

struct Machine<'program> {
    program: &'program Program,
    arena: Vec<Option<Thunk>>,
    environments: Vec<Option<Environment>>,
    gc: gc::State,
    scheduler: scheduler::State,
    channels: channels::State,
    files: file_handles::State,
    jobs: host_jobs::State,
    network: network::State,
    #[cfg(test)]
    gc_mode: gc::Mode,
    globals: BTreeMap<String, ThunkId>,
    remaining: usize,
    output_nodes: usize,
    cancelled: Option<&'program dyn Fn() -> bool>,
}

impl<'program> Machine<'program> {
    fn new(program: &'program Program) -> Self {
        Self {
            program,
            arena: Vec::new(),
            environments: vec![Some(Environment {
                parent: None,
                bindings: Vec::new(),
            })],
            gc: gc::State::default(),
            scheduler: scheduler::State::default(),
            channels: channels::State::default(),
            files: file_handles::State::default(),
            jobs: host_jobs::State::default(),
            network: network::State::default(),
            #[cfg(test)]
            gc_mode: gc::Mode::Automatic,
            globals: BTreeMap::new(),
            remaining: STEPS,
            output_nodes: 0,
            cancelled: None,
        }
    }

    fn tick(&mut self) -> Result<(), KernelError> {
        gc::charge(&mut self.remaining, self.cancelled)
    }

    fn allocate(&mut self, thunk: Thunk) -> Result<ThunkId, KernelError> {
        if let Some(id) = self.gc.free_thunks.pop() {
            self.arena[id] = Some(thunk);
            self.gc.allocated();
            return Ok(id);
        }
        if self.arena.len() >= ARENA_LIMIT {
            return Err(KernelError::new("data runtime thunk budget exhausted"));
        }
        let id = self.arena.len();
        self.arena.push(Some(thunk));
        self.gc.allocated();
        Ok(id)
    }

    fn expression(
        &mut self,
        expression: TermRef,
        environment: EnvId,
    ) -> Result<ThunkId, KernelError> {
        self.allocate(Thunk::Expression(expression, environment))
    }

    fn reference(&mut self, name: &str) -> Result<ThunkId, KernelError> {
        if let Some(id) = self.globals.get(name) {
            return Ok(*id);
        }
        let id = if let Some(intrinsic) = self.program.numeric.get(name) {
            self.allocate(Thunk::Ready(Value::Numeric {
                operation: numeric::NumericOperation::Intrinsic(*intrinsic),
                arguments: Vec::new(),
            }))?
        } else if let Some(optimization) = self
            .program
            .optimizations
            .iter()
            .find(|operation| operation.name() == name)
        {
            self.allocate(Thunk::Ready(Value::Numeric {
                operation: numeric::NumericOperation::Optimized(*optimization),
                arguments: Vec::new(),
            }))?
        } else if let Some(operation) = self
            .program
            .nat_optimizations
            .iter()
            .find(|operation| operation.name() == name)
        {
            self.allocate(Thunk::Ready(Value::Natural {
                operation: *operation,
                arguments: Vec::new(),
            }))?
        } else if self.program.foreign.contains_key(name) {
            self.allocate(Thunk::Ready(Value::Foreign {
                name: name.into(),
                arguments: Vec::new(),
            }))?
        } else if let Some(definition) = self.program.definitions.get(name) {
            let body = definition
                .body
                .as_ref()
                .ok_or_else(|| KernelError::new("unchecked open definition reached runtime"))?;
            self.expression(Rc::clone(body), 0)?
        } else if self.program.datatypes.contains_key(name) {
            self.allocate(Thunk::Ready(Value::Erased))?
        } else {
            return Err(KernelError::new(format!(
                "undefined runtime reference {name}"
            )));
        };
        self.globals.insert(name.into(), id);
        Ok(id)
    }

    fn environment(
        &mut self,
        parent: EnvId,
        bindings: Vec<(usize, ThunkId)>,
    ) -> Result<EnvId, KernelError> {
        let environment = Some(Environment {
            parent: Some(parent),
            bindings,
        });
        if let Some(id) = self.gc.free_environments.pop() {
            self.environments[id] = environment;
            self.gc.allocated();
            return Ok(id);
        }
        if self.environments.len() >= ARENA_LIMIT {
            return Err(KernelError::new(
                "data runtime environment budget exhausted",
            ));
        }
        let id = self.environments.len();
        self.environments.push(environment);
        self.gc.allocated();
        Ok(id)
    }

    fn variable(&mut self, mut environment: EnvId, binder: usize) -> Result<ThunkId, KernelError> {
        loop {
            self.tick()?;
            let current = self
                .environments
                .get(environment)
                .and_then(Option::as_ref)
                .ok_or_else(|| KernelError::new("invalid runtime environment reference"))?;
            if let Some((_, thunk)) = current.bindings.iter().find(|(id, _)| *id == binder) {
                return Ok(*thunk);
            }
            environment = current
                .parent
                .ok_or_else(|| KernelError::new("unbound runtime variable"))?;
        }
    }

    fn push(frames: &mut Vec<Frame>, frame: Frame) -> Result<(), KernelError> {
        if frames.len() >= FRAME_LIMIT {
            return Err(KernelError::new(
                "data runtime continuation depth exhausted",
            ));
        }
        frames.push(frame);
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the explicit evaluation machine keeps every continuation transition visible together"
    )]
    fn force(&mut self, mut current: ThunkId) -> Result<Value, KernelError> {
        let mut frames = Vec::new();
        'evaluate: loop {
            self.collection_safe_point(current, &frames)?;
            self.tick()?;
            let value = match self
                .arena
                .get(current)
                .and_then(Clone::clone)
                .ok_or_else(|| KernelError::new("invalid runtime thunk reference"))?
            {
                Thunk::Ready(value) => value,
                Thunk::Evaluating => {
                    return Err(KernelError::new("cyclic data runtime evaluation"));
                }
                Thunk::Application(function, argument) => {
                    self.arena[current] = Some(Thunk::Evaluating);
                    Self::push(&mut frames, Frame::Update(current))?;
                    Self::push(&mut frames, Frame::Apply(argument))?;
                    current = function;
                    continue;
                }
                Thunk::Expression(expression, environment) => {
                    self.arena[current] = Some(Thunk::Evaluating);
                    Self::push(&mut frames, Frame::Update(current))?;
                    match expression.as_ref() {
                        Term::Var { id, .. } => {
                            current = self.variable(environment, *id)?;
                            continue;
                        }
                        Term::Ref(name) | Term::GpuRef(name) => {
                            current = self.reference(name)?;
                            continue;
                        }
                        Term::Lam { id, body, .. } => Value::Closure {
                            binder: *id,
                            body: Rc::clone(body),
                            environment,
                        },
                        Term::App(function, argument) => {
                            let argument = self.expression(Rc::clone(argument), environment)?;
                            Self::push(&mut frames, Frame::Apply(argument))?;
                            current = self.expression(Rc::clone(function), environment)?;
                            continue;
                        }
                        Term::Ctr { name, args } => {
                            if let Some(value) = self
                                .program
                                .packed
                                .then(|| {
                                    packed::literal(name, args).or_else(|| nat::literal(name, args))
                                })
                                .flatten()
                            {
                                value
                            } else {
                                let fields = args
                                    .iter()
                                    .map(|arg| self.expression(Rc::clone(arg), environment))
                                    .collect::<Result<_, _>>()?;
                                Value::Constructor {
                                    name: name.clone(),
                                    fields,
                                }
                            }
                        }
                        Term::Mat {
                            constructor,
                            arm,
                            fallback,
                        } => Value::Match {
                            constructor: constructor.clone(),
                            arm: self.expression(Rc::clone(arm), environment)?,
                            fallback: self.expression(Rc::clone(fallback), environment)?,
                        },
                        Term::Let { bindings, body } => {
                            // Every RHS sees the original environment: lets are simultaneous.
                            let bindings = bindings
                                .iter()
                                .map(|binding| {
                                    Ok((
                                        binding.id,
                                        self.expression(Rc::clone(&binding.value), environment)?,
                                    ))
                                })
                                .collect::<Result<_, KernelError>>()?;
                            let environment = self.environment(environment, bindings)?;
                            current = self.expression(Rc::clone(body), environment)?;
                            continue;
                        }
                        Term::Ann(body, _) | Term::Rwt { body, .. } => {
                            current = self.expression(Rc::clone(body), environment)?;
                            continue;
                        }
                        Term::Efq => Value::Impossible,
                        Term::Typ(_)
                        | Term::Qnt
                        | Term::Qua(_)
                        | Term::Min(_, _)
                        | Term::All { .. }
                        | Term::Adt { .. }
                        | Term::Eql { .. }
                        | Term::Rfl => Value::Erased,
                        Term::Hole(_) => {
                            return Err(KernelError::new("unchecked hole reached data runtime"));
                        }
                    }
                }
            };
            loop {
                self.tick()?;
                match frames.pop() {
                    None => return Ok(value),
                    Some(Frame::Update(id)) => self.arena[id] = Some(Thunk::Ready(value.clone())),
                    Some(Frame::Numeric(frame)) => {
                        current = self.numeric_step(frame, value, &mut frames)?;
                        continue 'evaluate;
                    }
                    Some(Frame::Natural(frame)) => {
                        current = self.nat_step(frame, value, &mut frames)?;
                        continue 'evaluate;
                    }
                    Some(Frame::Apply(argument)) => match value {
                        Value::Closure {
                            binder,
                            body,
                            environment,
                        } => {
                            let environment =
                                self.environment(environment, vec![(binder, argument)])?;
                            current = self.expression(body, environment)?;
                            continue 'evaluate;
                        }
                        Value::Match {
                            constructor,
                            arm,
                            fallback,
                        } => {
                            Self::push(
                                &mut frames,
                                Frame::Match {
                                    constructor,
                                    arm,
                                    fallback,
                                    argument,
                                },
                            )?;
                            current = argument;
                            continue 'evaluate;
                        }
                        Value::Impossible => {
                            return Err(KernelError::new(
                                "entered an impossible runtime match branch",
                            ));
                        }
                        Value::Foreign { name, arguments } => {
                            current = self.apply_foreign(name, arguments, argument)?;
                            continue 'evaluate;
                        }
                        Value::Numeric {
                            operation,
                            arguments,
                        } => {
                            current =
                                self.apply_numeric(operation, arguments, argument, &mut frames)?;
                            continue 'evaluate;
                        }
                        Value::Natural {
                            operation,
                            arguments,
                        } => {
                            current =
                                self.apply_nat(operation, arguments, argument, &mut frames)?;
                            continue 'evaluate;
                        }
                        Value::EmitContinuation => {
                            current = self.allocate(Thunk::Ready(Value::Constructor {
                                name: "Emit".into(),
                                fields: vec![argument],
                            }))?;
                            continue 'evaluate;
                        }
                        _ => return Err(KernelError::new("runtime application of a non-function")),
                    },
                    Some(Frame::Match {
                        constructor,
                        arm,
                        fallback,
                        argument,
                    }) => {
                        if matches!(value, Value::Request { .. }) {
                            return Err(KernelError::new(
                                "runtime fail-stop: a foreign effect request cannot be matched as constructor data",
                            ));
                        }
                        let (name, fields) = self.constructor_value(value)?;
                        if name == constructor {
                            current = arm;
                            for field in fields {
                                current = self.allocate(Thunk::Application(current, field))?;
                            }
                        } else {
                            current = self.allocate(Thunk::Application(fallback, argument))?;
                        }
                        continue 'evaluate;
                    }
                }
            }
        }
    }

    fn materialize(&mut self, thunk: ThunkId, depth: usize) -> Result<TermRef, KernelError> {
        self.tick()?;
        if depth > OUTPUT_DEPTH || self.output_nodes >= OUTPUT_NODES {
            return Err(KernelError::new(
                "data runtime output depth or node budget exhausted",
            ));
        }
        self.output_nodes += 1;
        match self.force(thunk)? {
            value @ (Value::Constructor { .. }
            | Value::PackedNat(_)
            | Value::PackedWord { .. }
            | Value::PackedBits { .. }) => {
                let (name, fields) = self.constructor_value(value)?;
                // Packed views allocate fresh fields which are not children of
                // the original thunk. Keep every sibling across nested force.
                let args = self.with_roots(&fields, |machine| {
                    fields
                        .iter()
                        .map(|field| machine.materialize(*field, depth + 1))
                        .collect::<Result<_, _>>()
                })?;
                Ok(term(Term::Ctr { name, args }))
            }
            Value::Erased => Err(KernelError::new(
                "data runtime result contains an erased type or proof",
            )),
            Value::Request { .. } => Err(KernelError::new(
                "data runtime cannot materialize a foreign effect request",
            )),
            Value::Channel(_) => Err(KernelError::new(
                "data runtime cannot materialize an opaque channel handle",
            )),
            Value::File(_) => Err(KernelError::new(
                "data runtime cannot materialize an opaque file handle",
            )),
            Value::Socket(_) | Value::Listener(_) => Err(KernelError::new(
                "data runtime cannot materialize an opaque network handle",
            )),
            Value::Closure { .. }
            | Value::Match { .. }
            | Value::Foreign { .. }
            | Value::Numeric { .. }
            | Value::Natural { .. }
            | Value::EmitContinuation
            | Value::Impossible => Err(KernelError::new("data runtime result contains a function")),
        }
    }
}
