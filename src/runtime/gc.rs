// SPDX-License-Identifier: MPL-2.0
//! Bounded, nonmoving reclamation at committed evaluator transitions only.

use super::ARENA_LIMIT;
use super::EnvId;
use super::Frame;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;
use crate::kernel::KernelError;

// Leave room for a transition's allocations, without promising that an
// arbitrarily large constructor/let can fit between two safe points.
const RESERVE: usize = 8_192;
const COLLECTION_DEBT: usize = 8_192;

#[derive(Default)]
pub(super) struct State {
    pub(super) free_thunks: Vec<ThunkId>,
    pub(super) free_environments: Vec<EnvId>,
    roots: Vec<ThunkId>,
    allocation_debt: usize,
    #[cfg(test)]
    pub(super) collections: usize,
}

impl State {
    pub(super) fn allocated(&mut self) {
        self.allocation_debt = self.allocation_debt.saturating_add(1);
    }

    #[cfg(test)]
    fn record_collection(&mut self) {
        self.collections += 1;
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum Mode {
    Automatic,
    EverySafePoint,
    Disabled,
}

pub(super) fn charge(
    remaining: &mut usize,
    cancelled: Option<&dyn Fn() -> bool>,
) -> Result<(), KernelError> {
    if cancelled.is_some_and(|cancelled| cancelled()) {
        return Err(KernelError::new("execution cancelled"));
    }
    *remaining = remaining
        .checked_sub(1)
        .ok_or_else(|| KernelError::new("data runtime step budget exhausted"))?;
    Ok(())
}

enum Node {
    Thunk(ThunkId),
    Environment(EnvId),
}

struct Marks<'a> {
    thunks: Vec<bool>,
    environments: Vec<bool>,
    pending: Vec<Node>,
    remaining: &'a mut usize,
    cancelled: Option<&'a dyn Fn() -> bool>,
}

impl Marks<'_> {
    fn charge(&mut self) -> Result<(), KernelError> {
        charge(self.remaining, self.cancelled)
    }

    fn thunk(&mut self, id: ThunkId) -> Result<(), KernelError> {
        self.charge()?;
        let marked = self
            .thunks
            .get_mut(id)
            .ok_or_else(|| KernelError::new("invalid runtime thunk root"))?;
        if !*marked {
            *marked = true;
            self.pending.push(Node::Thunk(id));
        }
        Ok(())
    }

    fn environment(&mut self, id: EnvId) -> Result<(), KernelError> {
        self.charge()?;
        let marked = self
            .environments
            .get_mut(id)
            .ok_or_else(|| KernelError::new("invalid runtime environment root"))?;
        if !*marked {
            *marked = true;
            self.pending.push(Node::Environment(id));
        }
        Ok(())
    }

    fn value(&mut self, value: &Value) -> Result<(), KernelError> {
        match value {
            Value::Constructor { fields, .. } => {
                for id in fields {
                    self.thunk(*id)?;
                }
            }
            Value::Closure { environment, .. } => self.environment(*environment)?,
            Value::Match { arm, fallback, .. } => {
                self.thunk(*arm)?;
                self.thunk(*fallback)?;
            }
            Value::Foreign { arguments, .. }
            | Value::Numeric { arguments, .. }
            | Value::Natural { arguments, .. } => {
                for id in arguments {
                    self.thunk(*id)?;
                }
            }
            Value::Request {
                arguments,
                continuation,
                ..
            } => {
                for id in arguments {
                    self.thunk(*id)?;
                }
                self.thunk(*continuation)?;
            }
            Value::Channel(_)
            | Value::File(_)
            | Value::Socket(_)
            | Value::Listener(_)
            | Value::PackedNat(_)
            | Value::PackedWord { .. }
            | Value::PackedBits { .. }
            | Value::EmitContinuation
            | Value::Impossible
            | Value::Erased => {}
        }
        Ok(())
    }

    fn frame(&mut self, frame: &Frame) -> Result<(), KernelError> {
        match frame {
            Frame::Update(id) | Frame::Apply(id) => self.thunk(*id)?,
            Frame::Numeric(frame) => frame.visit_roots(|id| self.thunk(id))?,
            Frame::Natural(frame) => frame.visit_roots(|id| self.thunk(id))?,
            Frame::Match {
                arm,
                fallback,
                argument,
                ..
            } => {
                self.thunk(*arm)?;
                self.thunk(*fallback)?;
                self.thunk(*argument)?;
            }
        }
        Ok(())
    }
}

impl Machine<'_> {
    /// Host-side callers must retain every ID used after a nested `force`.
    /// The root stack is restored on success and ordinary error propagation.
    pub(super) fn with_roots<T>(
        &mut self,
        ids: &[ThunkId],
        action: impl FnOnce(&mut Self) -> Result<T, KernelError>,
    ) -> Result<T, KernelError> {
        let start = self.gc.roots.len();
        if ids.len() > ARENA_LIMIT - start {
            return Err(KernelError::new(
                "data runtime caller root budget exhausted",
            ));
        }
        self.gc.roots.extend_from_slice(ids);
        let result = action(self);
        self.gc.roots.truncate(start);
        result
    }

    pub(super) fn collection_safe_point(
        &mut self,
        current: ThunkId,
        frames: &[Frame],
    ) -> Result<(), KernelError> {
        let thunk_room = ARENA_LIMIT - self.arena.len() + self.gc.free_thunks.len();
        let environment_room =
            ARENA_LIMIT - self.environments.len() + self.gc.free_environments.len();
        let collect = self.gc.allocation_debt >= COLLECTION_DEBT
            && (thunk_room <= RESERVE || environment_room <= RESERVE);
        #[cfg(test)]
        let collect = match self.gc_mode {
            Mode::Automatic => collect,
            Mode::EverySafePoint => true,
            Mode::Disabled => false,
        };
        if collect {
            self.collect(current, frames)?;
        }
        Ok(())
    }

    fn collect(&mut self, current: ThunkId, frames: &[Frame]) -> Result<(), KernelError> {
        // Mark on enqueue: each allocated slot can enter this iterative work
        // list at most once. Its storage is bounded by the two arena caps.
        let mut marks = Marks {
            thunks: vec![false; self.arena.len()],
            environments: vec![false; self.environments.len()],
            pending: Vec::new(),
            remaining: &mut self.remaining,
            cancelled: self.cancelled,
        };
        marks.thunk(current)?;
        marks.environment(0)?;
        for frame in frames {
            marks.frame(frame)?;
        }
        for id in self.globals.values().chain(self.gc.roots.iter()) {
            marks.thunk(*id)?;
        }
        self.scheduler.visit_roots(|id| marks.thunk(id))?;
        self.channels.visit_roots(|id| marks.thunk(id))?;
        self.jobs.visit_roots(|id| marks.thunk(id))?;
        self.network.visit_roots(|id| marks.thunk(id))?;
        while let Some(node) = marks.pending.pop() {
            marks.charge()?;
            match node {
                Node::Thunk(id) => {
                    let thunk = self.arena[id]
                        .as_ref()
                        .ok_or_else(|| KernelError::new("vacant runtime thunk root"))?;
                    match thunk {
                        Thunk::Expression(_, environment) => marks.environment(*environment)?,
                        Thunk::Application(function, argument) => {
                            marks.thunk(*function)?;
                            marks.thunk(*argument)?;
                        }
                        Thunk::Ready(value) => marks.value(value)?,
                        Thunk::Evaluating => {}
                    }
                }
                Node::Environment(id) => {
                    let environment = self.environments[id]
                        .as_ref()
                        .ok_or_else(|| KernelError::new("vacant runtime environment root"))?;
                    if let Some(parent) = environment.parent {
                        marks.environment(parent)?;
                    }
                    for (_, id) in &environment.bindings {
                        marks.thunk(*id)?;
                    }
                }
            }
        }
        let Marks {
            thunks,
            environments,
            ..
        } = marks;
        self.gc.free_thunks.clear();
        for (id, marked) in thunks.into_iter().enumerate() {
            self.tick()?;
            if !marked {
                self.arena[id] = None;
                self.gc.free_thunks.push(id);
            }
        }
        self.gc.free_environments.clear();
        for (id, marked) in environments.into_iter().enumerate() {
            self.tick()?;
            if !marked {
                self.environments[id] = None;
                self.gc.free_environments.push(id);
            }
        }
        self.gc.allocation_debt = 0;
        #[cfg(test)]
        self.gc.record_collection();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::Term;
    use crate::kernel::term;
    use crate::runtime::Program;
    use crate::runtime::numeric::NumericOperation;
    use crate::runtime::packed::Wrapper;
    use crate::syntax::executable::NumericIntrinsic;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    fn program() -> Program {
        Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()))
    }

    fn erased(machine: &mut Machine<'_>) -> ThunkId {
        machine.allocate(Thunk::Ready(Value::Erased)).unwrap()
    }

    #[test]
    fn traces_every_value_environment_and_continuation_edge_without_forcing() {
        let program = program();
        let mut machine = Machine::new(&program);
        let mut required = Vec::new();
        for _ in 0..12 {
            required.push(erased(&mut machine));
        }
        let parent = machine.environment(0, vec![(1, required[0])]).unwrap();
        let environment = machine.environment(parent, vec![(2, required[1])]).unwrap();
        let expression = machine.expression(term(Term::Efq), environment).unwrap();
        let application = machine
            .allocate(Thunk::Application(required[2], required[3]))
            .unwrap();
        let values = [
            Value::Closure {
                binder: 0,
                body: term(Term::Efq),
                environment,
            },
            Value::Match {
                constructor: "Ignored".into(),
                arm: required[4],
                fallback: required[5],
            },
            Value::Foreign {
                name: "unexecuted".into(),
                arguments: vec![required[6]],
            },
            Value::Numeric {
                operation: NumericOperation::Intrinsic(NumericIntrinsic::Add),
                arguments: vec![required[7]],
            },
            Value::Request {
                name: "unexecuted".into(),
                arguments: vec![required[8]],
                continuation: required[9],
            },
            Value::PackedWord {
                wrapper: Wrapper::F32,
                bits: 0x7f81_2345,
            },
            Value::PackedBits { bits: 1, width: 1 },
            Value::EmitContinuation,
            Value::Impossible,
        ];
        let mut children = vec![expression, application];
        for value in values {
            children.push(machine.allocate(Thunk::Ready(value)).unwrap());
        }
        let root = machine
            .allocate(Thunk::Ready(Value::Constructor {
                name: "Root".into(),
                fields: children.clone(),
            }))
            .unwrap();
        let updating = machine.allocate(Thunk::Evaluating).unwrap();
        let frames = [
            Frame::Update(updating),
            Frame::Apply(required[10]),
            Frame::Match {
                constructor: "Ignored".into(),
                arm: required[11],
                fallback: root,
                argument: root,
            },
        ];
        let global = erased(&mut machine);
        machine.globals.insert("global".into(), global);
        let caller = erased(&mut machine);
        let garbage = erased(&mut machine);
        let dead_environment = machine.environment(0, Vec::new()).unwrap();
        machine
            .with_roots(&[caller], |machine| machine.collect(root, &frames))
            .unwrap();
        for id in required
            .into_iter()
            .chain(children)
            .chain([root, updating, global, caller])
        {
            assert!(machine.arena[id].is_some(), "lost live thunk {id}");
        }
        assert!(matches!(machine.arena[updating], Some(Thunk::Evaluating)));
        assert!(machine.environments[parent].is_some());
        assert!(machine.environments[environment].is_some());
        assert!(machine.arena[garbage].is_none());
        assert!(machine.environments[dead_environment].is_none());
        assert_eq!(
            erased(&mut machine),
            garbage,
            "reuse a vacant slot without moving live IDs"
        );
        assert_eq!(
            machine.environment(0, Vec::new()).unwrap(),
            dead_environment
        );
    }

    #[test]
    fn scoped_roots_restore_nested_success_and_failure_and_reject_growth() {
        let program = program();
        let mut machine = Machine::new(&program);
        let root = erased(&mut machine);
        machine
            .with_roots(&[root], |machine| {
                assert_eq!(machine.gc.roots, [root]);
                machine
                    .with_roots(&[root], |machine| {
                        assert_eq!(machine.gc.roots, [root, root]);
                        Err::<(), _>(KernelError::new("sentinel"))
                    })
                    .expect_err("nested failure must restore its scope");
                assert_eq!(machine.gc.roots, [root]);
                Ok(())
            })
            .unwrap();
        assert!(machine.gc.roots.is_empty());
        machine
            .with_roots(&vec![root; ARENA_LIMIT], |machine| {
                machine
                    .with_roots(&[root], |_| Ok(()))
                    .expect_err("root stack is bounded");
                Ok(())
            })
            .unwrap();
        assert!(machine.gc.roots.is_empty());
    }

    #[test]
    fn automatic_collection_reclaims_both_arenas_and_avoids_futile_repetition() {
        let program = program();
        let mut machine = Machine::new(&program);
        let root = erased(&mut machine);
        for _ in 1..ARENA_LIMIT - RESERVE {
            erased(&mut machine);
            machine.environment(0, Vec::new()).unwrap();
        }
        machine.collection_safe_point(root, &[]).unwrap();
        assert_eq!(machine.gc.collections, 1);
        assert_eq!(machine.gc.free_thunks.len(), ARENA_LIMIT - RESERVE - 1);
        assert_eq!(
            machine.gc.free_environments.len(),
            ARENA_LIMIT - RESERVE - 1
        );
        let thunk_len = machine.arena.len();
        let environment_len = machine.environments.len();
        for _ in 0..COLLECTION_DEBT {
            erased(&mut machine);
            machine.environment(0, Vec::new()).unwrap();
        }
        machine.collection_safe_point(root, &[]).unwrap();
        assert_eq!(
            machine.gc.collections, 1,
            "plenty of space requires no collection"
        );
        assert_eq!(machine.arena.len(), thunk_len);
        assert_eq!(machine.environments.len(), environment_len);
    }

    #[test]
    fn a_fully_live_graph_keeps_both_arena_caps_and_does_not_collect_repeatedly() {
        let program = program();
        let mut machine = Machine::new(&program);
        let root = erased(&mut machine);
        let mut parent = 0;
        for _ in 1..ARENA_LIMIT {
            let child = erased(&mut machine);
            parent = machine.environment(parent, vec![(0, child)]).unwrap();
        }
        machine.arena[root] = Some(Thunk::Ready(Value::Closure {
            binder: 0,
            body: term(Term::Efq),
            environment: parent,
        }));
        machine.collection_safe_point(root, &[]).unwrap();
        assert!(machine.gc.free_thunks.is_empty());
        assert!(machine.gc.free_environments.is_empty());
        assert_eq!(machine.gc.collections, 1);
        machine.collection_safe_point(root, &[]).unwrap();
        assert_eq!(machine.gc.collections, 1);
        machine
            .allocate(Thunk::Ready(Value::Erased))
            .expect_err("all thunk slots are live");
        machine
            .environment(0, Vec::new())
            .expect_err("all environment slots are live");
    }

    #[test]
    fn mark_cycles_are_bounded_and_invalid_roots_steps_and_cancellation_fail_closed() {
        let program = program();
        let mut machine = Machine::new(&program);
        let root = erased(&mut machine);
        machine.arena[root] = Some(Thunk::Application(root, root));
        machine.collect(root, &[]).unwrap();
        assert!(machine.arena[root].is_some());
        machine
            .collect(ARENA_LIMIT, &[])
            .expect_err("invalid root must not index an arena");
        machine.remaining = 0;
        assert!(
            machine
                .collect(root, &[])
                .unwrap_err()
                .to_string()
                .contains("step budget")
        );
        machine.remaining = super::super::STEPS;
        machine.cancelled = Some(&|| true);
        assert!(
            machine
                .collect(root, &[])
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
    }

    #[test]
    fn interruption_during_either_sweep_keeps_live_ids_and_free_lists_consistent() {
        use std::cell::Cell;
        use std::collections::BTreeSet;

        let program = program();
        let mut saw_partial_thunk_sweep = false;
        let mut saw_partial_environment_sweep = false;
        for cancellation in [false, true] {
            for cutoff in 0..40 {
                let calls = Cell::new(0);
                let cancelled = || {
                    calls.set(calls.get() + 1);
                    calls.get() > cutoff
                };
                let mut machine = Machine::new(&program);
                let dead = erased(&mut machine);
                let live = erased(&mut machine);
                let dead_environment = machine.environment(0, Vec::new()).unwrap();
                let live_environment = machine.environment(0, vec![(0, live)]).unwrap();
                let root = machine
                    .allocate(Thunk::Ready(Value::Closure {
                        binder: 0,
                        body: term(Term::Efq),
                        environment: live_environment,
                    }))
                    .unwrap();
                if cancellation {
                    machine.cancelled = Some(&cancelled);
                } else {
                    machine.remaining = cutoff;
                }
                let result = machine.collect(root, &[]);
                assert!(machine.arena[root].is_some());
                assert!(machine.arena[live].is_some());
                assert!(machine.environments[0].is_some());
                assert!(machine.environments[live_environment].is_some());
                for id in &machine.gc.free_thunks {
                    assert!(machine.arena[*id].is_none());
                }
                for id in &machine.gc.free_environments {
                    assert!(machine.environments[*id].is_none());
                }
                assert_eq!(
                    machine.gc.free_thunks.iter().collect::<BTreeSet<_>>().len(),
                    machine.gc.free_thunks.len()
                );
                assert_eq!(
                    machine
                        .gc
                        .free_environments
                        .iter()
                        .collect::<BTreeSet<_>>()
                        .len(),
                    machine.gc.free_environments.len()
                );
                if let Err(error) = result {
                    let expected = if cancellation {
                        "cancelled"
                    } else {
                        "step budget"
                    };
                    assert!(error.to_string().contains(expected), "{error}");
                    saw_partial_thunk_sweep |= machine.arena[dead].is_none();
                    saw_partial_environment_sweep |=
                        machine.environments[dead_environment].is_none();
                    // Even an interrupted collection leaves a safe arena. A
                    // later caller cannot reuse a live slot or duplicate a
                    // vacant entry when the free lists are rebuilt.
                    machine.cancelled = None;
                    machine.remaining = super::super::STEPS;
                    machine.collect(root, &[]).unwrap();
                    assert_eq!(erased(&mut machine), dead);
                    assert_eq!(
                        machine.environment(0, Vec::new()).unwrap(),
                        dead_environment
                    );
                }
            }
        }
        assert!(saw_partial_thunk_sweep);
        assert!(saw_partial_environment_sweep);
    }

    #[test]
    fn every_safe_point_matches_disabled_collection_for_shared_lets_and_packed_views() {
        use crate::kernel::check_book;
        use crate::syntax::parse;
        let book = parse("type Bit is Data: Off{} On{}\ntype Pair is Data: Pair{a: Bit, b: Bit}\ndef id(a: Bit) -> Bit: a\ndef main() -> Pair:\n  +a = id(On{})\n  Pair{a, a}\n").unwrap();
        check_book(&book).unwrap();
        let mut definitions = BTreeMap::new();
        let mut datatypes = BTreeMap::new();
        for declaration in book.declarations {
            match declaration {
                crate::kernel::Declaration::Def(definition) => {
                    definitions.insert(definition.name.clone(), definition);
                }
                crate::kernel::Declaration::Adt(datatype) => {
                    datatypes.insert(datatype.name.clone(), datatype);
                }
            }
        }
        let program = Program::from_checked(&Rc::new(definitions), &Rc::new(datatypes));
        let mut outputs = Vec::new();
        for mode in [Mode::Disabled, Mode::EverySafePoint] {
            let mut machine = Machine::new(&program);
            machine.gc_mode = mode;
            let root = machine.reference("main").unwrap();
            outputs.push(machine.materialize(root, 0).unwrap().to_string());
        }
        assert_eq!(outputs, ["Pair{On{}, On{}}", "Pair{On{}, On{}}"]);

        let mut outputs = Vec::new();
        for mode in [Mode::Disabled, Mode::EverySafePoint] {
            let mut machine = Machine::new(&program);
            machine.gc_mode = mode;
            let root = machine
                .allocate(Thunk::Ready(Value::PackedWord {
                    wrapper: Wrapper::F32,
                    bits: 0x7f81_2345,
                }))
                .unwrap();
            outputs.push(machine.materialize(root, 0).unwrap().to_string());
            if matches!(mode, Mode::EverySafePoint) {
                assert!(machine.gc.collections >= 66);
            }
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}
