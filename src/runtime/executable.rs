// SPDX-License-Identifier: MPL-2.0
//! Native execution of checked executable contracts. Requests remain private
//! machine values and are never reduced or accepted by the proof kernel.

use super::Machine;
use super::Program;
use super::Thunk;
use super::ThunkId;
use super::Value;
use super::packed::Wrapper;
use super::runtime_clock::Clock;
use super::runtime_clock::MAX_WAIT_NANOS;
use super::runtime_clock::NANOS_PER_MILLI;
use super::runtime_clock::SystemClock;
use crate::kernel::AdtDecl;
use crate::kernel::DefDecl;
use crate::kernel::KernelError;
use crate::syntax::executable::BuiltinForeign;
use crate::syntax::executable::ForeignDefinition;
use crate::syntax::executable::NumericIntrinsic;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::io::Write;
use std::rc::Rc;

pub(super) const TEXT_BYTES: usize = 8 * 1024 * 1024;

mod channel_io;
#[cfg(test)]
mod channel_tests;
mod file_io;
#[cfg(test)]
mod file_tests;
#[cfg(test)]
mod scheduler_tests;

enum TaskOutcome {
    Complete,
    Parked,
    Halt(u32),
}

impl Program {
    pub(crate) fn from_executable(
        definitions: &Rc<BTreeMap<String, DefDecl>>,
        datatypes: &Rc<BTreeMap<String, AdtDecl>>,
        foreign: &BTreeMap<String, ForeignDefinition>,
        numeric: &BTreeMap<String, NumericIntrinsic>,
        base_names: &BTreeSet<String>,
    ) -> Self {
        Self {
            definitions: Rc::clone(definitions),
            datatypes: Rc::clone(datatypes),
            foreign: Rc::new(foreign.clone()),
            numeric: Rc::new(numeric.clone()),
            optimizations: Rc::new(super::numeric::checked_optimizations(
                definitions,
                base_names,
            )),
            packed: super::packed::checked_layouts(datatypes, base_names),
            nat_optimizations: Rc::new(super::nat::checked_optimizations(
                definitions,
                datatypes,
                base_names,
            )),
        }
    }

    pub(crate) fn run_io(
        &self,
        name: &str,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<u32, KernelError> {
        let mut machine = Machine::new(self);
        machine.cancelled = Some(cancelled);
        machine.tick()?;
        let entry = machine.reference(name)?;
        let current = machine.io_action(entry)?;
        machine.drive_io(current, stdout, stderr)
    }
}

impl Machine<'_> {
    fn io_action(&mut self, entry: ThunkId) -> Result<ThunkId, KernelError> {
        // IO(A) retains its erased R lambda. Supply erased R and an Emit
        // continuation without manufacturing proof evidence or forcing payloads.
        let result_type = self.allocate(Thunk::Ready(Value::Erased))?;
        let continuation = self.allocate(Thunk::Ready(Value::EmitContinuation))?;
        let action = self.allocate(Thunk::Application(entry, result_type))?;
        self.allocate(Thunk::Application(action, continuation))
    }

    fn drive_io(
        &mut self,
        current: ThunkId,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> Result<u32, KernelError> {
        self.drive_io_with_clock(current, stdout, stderr, &mut SystemClock)
    }

    fn drive_io_with_clock(
        &mut self,
        current: ThunkId,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        clock: &mut dyn Clock,
    ) -> Result<u32, KernelError> {
        self.scheduler.spawn(current)?;
        let result = self.drive_tasks(stdout, stderr, clock);
        // Every exit discards pending tasks, timers, channel values and waits.
        self.scheduler = super::scheduler::State::default();
        self.channels = super::channels::State::default();
        self.jobs = super::host_jobs::State::default();
        self.files = super::file_handles::State::default();
        result
    }

    fn drive_tasks(
        &mut self,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        clock: &mut dyn Clock,
    ) -> Result<u32, KernelError> {
        loop {
            self.tick()?;
            if let Some(current) = self.scheduler.next() {
                match self.run_task(current, stdout, stderr, clock)? {
                    TaskOutcome::Complete => self.scheduler.complete()?,
                    TaskOutcome::Parked => {}
                    TaskOutcome::Halt(code) => return Ok(code),
                }
                continue;
            }
            if self.scheduler.finished() {
                self.flush(stdout, "stdout")?;
                return Ok(0);
            }
            self.wait_for_task(stdout, clock)?;
        }
    }

    fn wait_for_task(
        &mut self,
        stdout: &mut dyn Write,
        clock: &mut dyn Clock,
    ) -> Result<(), KernelError> {
        // Upstream flushes before entering its poller, including zero-time
        // waits. Buffered output must be visible while every task is parked.
        self.flush(stdout, "stdout")?;
        loop {
            // Waiting does not consume evaluation steps. In particular, the
            // full U32 sleep range must not exhaust the budget merely because
            // the host adapter polls cancellation every 100 milliseconds.
            if self.cancelled.is_some_and(|cancelled| cancelled()) {
                return Err(KernelError::new("execution cancelled"));
            }
            // Upstream takes completed host jobs before waking due timers,
            // only after the runnable queue has drained.
            self.resume_host_jobs()?;
            let now = clock.now()?;
            self.scheduler.wake(now);
            if self.scheduler.has_ready() {
                return Ok(());
            }
            let wait = self
                .scheduler
                .next_deadline()
                .map(|deadline| deadline.saturating_sub(now).min(MAX_WAIT_NANOS));
            if self.jobs.has_pending() {
                clock.wait_for_work(&mut self.jobs, wait.unwrap_or(MAX_WAIT_NANOS))?;
            } else if let Some(wait) = wait {
                clock.wait(wait)?;
            } else {
                return Err(KernelError::new(
                    "native IO scheduler deadlock: no runnable task, timer or host job",
                ));
            }
        }
    }

    fn run_task(
        &mut self,
        mut current: ThunkId,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        clock: &mut dyn Clock,
    ) -> Result<TaskOutcome, KernelError> {
        loop {
            self.tick()?;
            match self.force(current)? {
                Value::Constructor { name, fields } if name == "Emit" && fields.len() == 1 => {
                    // Successful IO discards its result, including IO(U32).
                    return Ok(TaskOutcome::Complete);
                }
                Value::Constructor { name, fields } if name == "Halt" && fields.len() == 2 => {
                    let (code, message) = self.with_roots(&fields, |machine| {
                        let code = machine.read_u32(fields[0])?;
                        let message = machine.read_text(fields[1])?;
                        Ok((code, message))
                    })?;
                    self.flush(stdout, "stdout")?;
                    self.write_bytes(stderr, &message, "stderr")?;
                    self.write_bytes(stderr, b"\n", "stderr")?;
                    self.flush(stderr, "stderr")?;
                    return Ok(TaskOutcome::Halt(code));
                }
                Value::Request {
                    name,
                    arguments,
                    continuation,
                } => {
                    let mut roots = arguments.clone();
                    roots.push(continuation);
                    let next = self.with_roots(&roots, |machine| {
                        machine.io_request(&name, &arguments, continuation, stdout, stderr, clock)
                    })?;
                    if let Some(next) = next {
                        current = next;
                    } else {
                        return Ok(TaskOutcome::Parked);
                    }
                }
                _ => return Err(KernelError::new("main did not produce an IO operation")),
            }
        }
    }

    fn unit(&mut self) -> Result<ThunkId, KernelError> {
        self.allocate(Thunk::Ready(Value::Constructor {
            name: "Unit".into(),
            fields: Vec::new(),
        }))
    }

    fn io_request(
        &mut self,
        name: &str,
        arguments: &[ThunkId],
        continuation: ThunkId,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        clock: &mut dyn Clock,
    ) -> Result<Option<ThunkId>, KernelError> {
        let builtin = self
            .program
            .foreign
            .get(name)
            .and_then(|foreign| foreign.builtin);
        let answer = match builtin {
            Some(
                builtin @ (BuiltinForeign::GetEnv
                | BuiltinForeign::FileOpen
                | BuiltinForeign::FileRead
                | BuiltinForeign::FileReadBytes
                | BuiltinForeign::FileWrite
                | BuiltinForeign::FileClose),
            ) => return self.file_request(builtin, arguments, continuation),
            Some(
                builtin @ (BuiltinForeign::ChanNew
                | BuiltinForeign::ChanSend
                | BuiltinForeign::ChanRecv
                | BuiltinForeign::ChanClose),
            ) => return self.channel_request(builtin, arguments, continuation),
            Some(BuiltinForeign::Spawn) => {
                let [_, action] = arguments else {
                    return Err(KernelError::new("spawn request has an invalid arity"));
                };
                let action = self.io_action(*action)?;
                self.scheduler.spawn(action)?;
                self.unit()?
            }
            Some(BuiltinForeign::Sleep) => {
                let [milliseconds] = arguments else {
                    return Err(KernelError::new("sleep request has an invalid arity"));
                };
                let milliseconds = self.read_u32(*milliseconds)?;
                let deadline = clock
                    .now()?
                    .checked_add(u64::from(milliseconds) * NANOS_PER_MILLI)
                    .ok_or_else(|| KernelError::new("native timer deadline overflow"))?;
                let unit = self.unit()?;
                let action = self.allocate(Thunk::Application(continuation, unit))?;
                self.scheduler.sleep(deadline, action)?;
                return Ok(None);
            }
            Some(BuiltinForeign::Now) => {
                if !arguments.is_empty() {
                    return Err(KernelError::new("clock request has an invalid arity"));
                }
                self.nat(clock.now()? / NANOS_PER_MILLI)?
            }
            _ => self.console_request(name, arguments, stdout, stderr)?,
        };
        Ok(Some(
            self.allocate(Thunk::Application(continuation, answer))?,
        ))
    }

    pub(super) fn apply_foreign(
        &mut self,
        name: String,
        mut arguments: Vec<ThunkId>,
        argument: ThunkId,
    ) -> Result<ThunkId, KernelError> {
        let declared = self
            .program
            .foreign
            .get(&name)
            .ok_or_else(|| KernelError::new("foreign declaration metadata is absent"))?
            .declared_arity;
        arguments.push(argument);
        let value = if arguments.len() == declared + 2 {
            let continuation = arguments[declared + 1];
            // Arguments include the source parameters, then erased R and k.
            // Erased source parameters remain represented until host dispatch.
            arguments.truncate(declared);
            Value::Request {
                name,
                arguments,
                continuation,
            }
        } else {
            Value::Foreign { name, arguments }
        };
        self.allocate(Thunk::Ready(value))
    }

    fn console_request(
        &mut self,
        name: &str,
        arguments: &[ThunkId],
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> Result<ThunkId, KernelError> {
        let descriptor = self
            .program
            .foreign
            .get(name)
            .ok_or_else(|| KernelError::new("foreign request metadata is absent"))?;
        let builtin = descriptor.builtin.ok_or_else(|| {
            KernelError::new(format!(
                "native execution does not support the foreign implementation of {name}"
            ))
        })?;
        match builtin {
            BuiltinForeign::GetEnv
            | BuiltinForeign::FileOpen
            | BuiltinForeign::FileRead
            | BuiltinForeign::FileReadBytes
            | BuiltinForeign::FileWrite
            | BuiltinForeign::FileClose => {
                return Err(KernelError::new(format!(
                    "file/environment builtin {name} reached console-only dispatch"
                )));
            }
            BuiltinForeign::Spawn | BuiltinForeign::Sleep | BuiltinForeign::Now => {
                return Err(KernelError::new(format!(
                    "scheduler builtin {name} reached console-only dispatch"
                )));
            }
            BuiltinForeign::ChanNew
            | BuiltinForeign::ChanSend
            | BuiltinForeign::ChanRecv
            | BuiltinForeign::ChanClose => {
                return Err(KernelError::new(format!(
                    "channel builtin {name} reached console-only dispatch"
                )));
            }
            BuiltinForeign::Print | BuiltinForeign::Write | BuiltinForeign::PrintErr => {}
        }
        let [text] = arguments else {
            return Err(KernelError::new("console request has an invalid arity"));
        };
        let bytes = self.read_text(*text)?;
        if builtin == BuiltinForeign::PrintErr {
            self.flush(stdout, "stdout")?;
            self.write_bytes(stderr, &bytes, "stderr")?;
            self.write_bytes(stderr, b"\n", "stderr")?;
            self.flush(stderr, "stderr")?;
        } else {
            self.write_bytes(stdout, &bytes, "stdout")?;
            if builtin == BuiltinForeign::Print {
                self.write_bytes(stdout, b"\n", "stdout")?;
            }
        }
        self.allocate(Thunk::Ready(Value::Constructor {
            name: "Unit".into(),
            fields: Vec::new(),
        }))
    }

    fn constructor(&mut self, thunk: ThunkId) -> Result<(String, Vec<ThunkId>), KernelError> {
        let value = self.force(thunk)?;
        self.console_constructor(value)
    }

    fn console_constructor(&mut self, value: Value) -> Result<(String, Vec<ThunkId>), KernelError> {
        match value {
            Value::Constructor { name, fields } => Ok((name, fields)),
            Value::PackedWord { .. } | Value::PackedBits { .. } | Value::PackedNat(_) => {
                self.constructor_value(value)
            }
            Value::Request { .. } => Err(KernelError::new(
                "runtime fail-stop: a foreign effect request escaped the IO driver",
            )),
            _ => Err(KernelError::new("console argument is not constructor data")),
        }
    }

    fn read_u32(&mut self, thunk: ThunkId) -> Result<u32, KernelError> {
        let value = self.force(thunk)?;
        if let Value::PackedWord { wrapper, bits } = value {
            return if wrapper == Wrapper::U32 {
                Ok(bits)
            } else {
                Err(KernelError::new("expected a U32 value"))
            };
        }
        let (name, fields) = self.console_constructor(value)?;
        let [word] = fields.as_slice() else {
            return Err(KernelError::new("expected a U32 value"));
        };
        let mut word = *word;
        if name != "U32" {
            return Err(KernelError::new("expected a U32 value"));
        }
        let mut result = 0;
        for bit in 0..32 {
            let value = self.force(word)?;
            if let Value::PackedBits { bits, width } = value {
                if u32::from(width) != 32 - bit || !super::packed::valid_bits(bits, width) {
                    return Err(KernelError::new("expected exactly 32 Word bits"));
                }
                return Ok(result | (bits << bit));
            }
            let (name, fields) = self.console_constructor(value)?;
            let [head, tail] = fields.as_slice() else {
                return Err(KernelError::new("expected a 32-bit Word"));
            };
            if name != "WCon" {
                return Err(KernelError::new("expected a 32-bit Word"));
            }
            let (name, fields) = self.with_roots(&[*tail], |machine| machine.constructor(*head))?;
            match (name.as_str(), fields.is_empty()) {
                ("False", true) => {}
                ("True", true) => result |= 1 << bit,
                _ => return Err(KernelError::new("expected a Boolean Word bit")),
            }
            word = *tail;
        }
        let (name, fields) = self.constructor(word)?;
        if name != "WNil" || !fields.is_empty() {
            return Err(KernelError::new("expected exactly 32 Word bits"));
        }
        Ok(result)
    }

    fn read_text(&mut self, mut current: ThunkId) -> Result<Vec<u8>, KernelError> {
        let mut result = Vec::new();
        loop {
            let (name, fields) = self.constructor(current)?;
            match (name.as_str(), fields.as_slice()) {
                ("SNil", []) => return Ok(result),
                ("SCon", [head, tail]) => {
                    let scalar = self.with_roots(&[*tail], |machine| {
                        let (name, fields) = machine.constructor(*head)?;
                        let [code] = fields.as_slice() else {
                            return Err(KernelError::new("expected a Char value"));
                        };
                        if name != "Chr" {
                            return Err(KernelError::new("expected a Char value"));
                        }
                        machine.read_u32(*code)
                    })?;
                    let (bytes, length) = super::host_files::encode_native_char(scalar);
                    let encoded = &bytes[..length];
                    if result.len() + encoded.len() > TEXT_BYTES {
                        return Err(KernelError::new("console text byte budget exhausted"));
                    }
                    result.extend_from_slice(encoded);
                    current = *tail;
                }
                _ => return Err(KernelError::new("expected a String value")),
            }
        }
    }

    fn write_bytes(
        &mut self,
        writer: &mut dyn Write,
        mut bytes: &[u8],
        stream: &str,
    ) -> Result<(), KernelError> {
        while !bytes.is_empty() {
            self.tick()?;
            match writer.write(bytes) {
                Ok(0) => return Err(KernelError::new(format!("zero-length write on {stream}"))),
                Ok(count) => bytes = &bytes[count..],
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => {
                    return Err(KernelError::new(format!("cannot write {stream}: {error}")));
                }
            }
        }
        Ok(())
    }

    fn flush(&mut self, writer: &mut dyn Write, stream: &str) -> Result<(), KernelError> {
        self.tick()?;
        writer
            .flush()
            .map_err(|error| KernelError::new(format!("cannot flush {stream}: {error}")))
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::BuiltinForeign;
    use super::ForeignDefinition;
    use super::Machine;
    use super::Program;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    #[test]
    fn console_only_dispatch_rejects_misrouted_channels_before_argument_decoding() {
        for (name, builtin) in [
            ("Chan.new", BuiltinForeign::ChanNew),
            ("Chan.send", BuiltinForeign::ChanSend),
            ("Chan.recv", BuiltinForeign::ChanRecv),
            ("Chan.close", BuiltinForeign::ChanClose),
        ] {
            let foreign = BTreeMap::from([(
                name.into(),
                ForeignDefinition {
                    imports: vec![],
                    local_symbol: String::new(),
                    declared_arity: 0,
                    parameters: vec![],
                    builtin: Some(builtin),
                },
            )]);
            let program = Program::from_executable(
                &Rc::new(BTreeMap::new()),
                &Rc::new(BTreeMap::new()),
                &foreign,
                &BTreeMap::new(),
                &BTreeSet::new(),
            );
            let mut machine = Machine::new(&program);
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let error = machine
                .console_request(name, &[], &mut stdout, &mut stderr)
                .expect_err("channels must use the scheduler-aware dispatcher")
                .to_string();
            assert!(
                error.contains(&format!("channel builtin {name}")),
                "{error}"
            );
            assert!(stdout.is_empty() && stderr.is_empty());
        }
    }
}

#[cfg(test)]
mod word_tests {
    use super::*;
    use crate::syntax::parse_term;

    fn ready(machine: &mut Machine<'_>, value: Value) -> ThunkId {
        machine.allocate(Thunk::Ready(value)).unwrap()
    }

    fn constructor(machine: &mut Machine<'_>, name: &str, fields: Vec<ThunkId>) -> ThunkId {
        ready(
            machine,
            Value::Constructor {
                name: name.into(),
                fields,
            },
        )
    }

    #[test]
    fn console_reads_packed_mixed_and_ordinary_words_without_expanding_packed_tails() {
        let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
        let mut machine = Machine::new(&program);
        for bits in [0, 1, 0x8000_0000, u32::MAX] {
            let word = ready(
                &mut machine,
                Value::PackedWord {
                    wrapper: Wrapper::U32,
                    bits,
                },
            );
            let allocated = machine.arena.len();
            assert_eq!(machine.read_u32(word).unwrap(), bits);
            assert_eq!(machine.arena.len(), allocated);
        }

        let tail = ready(
            &mut machine,
            Value::PackedBits {
                bits: 0x4000_0000,
                width: 31,
            },
        );
        let head = constructor(&mut machine, "True", vec![]);
        let word = constructor(&mut machine, "WCon", vec![head, tail]);
        let wrapper = constructor(&mut machine, "U32", vec![word]);
        let allocated = machine.arena.len();
        assert_eq!(machine.read_u32(wrapper).unwrap(), 0x8000_0001);
        assert_eq!(machine.arena.len(), allocated);

        let tail = ready(
            &mut machine,
            Value::PackedBits {
                bits: u32::MAX,
                width: 32,
            },
        );
        let wrapper = constructor(&mut machine, "U32", vec![tail]);
        assert_eq!(machine.read_u32(wrapper).unwrap(), u32::MAX);

        let ordinary = machine
            .expression(parse_term("305419896").unwrap(), 0)
            .unwrap();
        assert_eq!(machine.read_u32(ordinary).unwrap(), 0x1234_5678);
    }

    #[test]
    fn console_word_decoding_rejects_wrong_wrappers_invalid_tails_and_effect_requests() {
        let program = Program::from_checked(&Rc::new(BTreeMap::new()), &Rc::new(BTreeMap::new()));
        let mut machine = Machine::new(&program);
        let wrong = ready(
            &mut machine,
            Value::PackedWord {
                wrapper: Wrapper::F32,
                bits: 0,
            },
        );
        assert!(
            machine
                .read_u32(wrong)
                .unwrap_err()
                .to_string()
                .contains("expected a U32")
        );
        for (bits, width) in [(0, 31), (0, 33)] {
            let tail = ready(&mut machine, Value::PackedBits { bits, width });
            let wrapper = constructor(&mut machine, "U32", vec![tail]);
            assert!(
                machine
                    .read_u32(wrapper)
                    .unwrap_err()
                    .to_string()
                    .contains("exactly 32 Word bits")
            );
        }
        let tail = ready(
            &mut machine,
            Value::PackedBits {
                bits: 0x8000_0000,
                width: 31,
            },
        );
        let head = constructor(&mut machine, "False", vec![]);
        let word = constructor(&mut machine, "WCon", vec![head, tail]);
        let wrapper = constructor(&mut machine, "U32", vec![word]);
        assert!(
            machine
                .read_u32(wrapper)
                .unwrap_err()
                .to_string()
                .contains("exactly 32 Word bits")
        );

        let request = ready(
            &mut machine,
            Value::Request {
                name: "blocked".into(),
                arguments: vec![],
                continuation: 0,
            },
        );
        let wrapper = constructor(&mut machine, "U32", vec![request]);
        for value in [request, wrapper] {
            assert!(
                machine
                    .read_u32(value)
                    .unwrap_err()
                    .to_string()
                    .contains("foreign effect request escaped")
            );
        }
    }
}

#[cfg(test)]
mod collection_tests {
    use super::*;
    use crate::kernel::Term;
    use crate::kernel::term;
    use crate::syntax::parse_term;
    use std::cell::Cell;
    use std::cell::RefCell;

    type Events = Rc<RefCell<Vec<(&'static str, Vec<u8>)>>>;

    struct Writer {
        stream: &'static str,
        events: Events,
    }

    impl Write for Writer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.events.borrow_mut().push((self.stream, buf.to_vec()));
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn program() -> Program {
        let foreign = [
            ("IO.print", BuiltinForeign::Print),
            ("IO.write", BuiltinForeign::Write),
            ("IO.print_err", BuiltinForeign::PrintErr),
        ]
        .into_iter()
        .map(|(name, builtin)| {
            (
                name.into(),
                ForeignDefinition {
                    imports: vec![],
                    local_symbol: name.into(),
                    declared_arity: 1,
                    parameters: vec!["text".into()],
                    builtin: Some(builtin),
                },
            )
        })
        .collect();
        Program::from_executable(
            &Rc::new(BTreeMap::new()),
            &Rc::new(BTreeMap::new()),
            &foreign,
            &BTreeMap::new(),
            &BTreeSet::new(),
        )
    }

    fn request(machine: &mut Machine<'_>, name: &str, text: &str, next: ThunkId) -> ThunkId {
        let argument = machine
            .expression(parse_term(&format!("{text:?}")).unwrap(), 0)
            .unwrap();
        let environment = machine.environment(0, vec![(1, next)]).unwrap();
        let continuation = machine
            .allocate(Thunk::Ready(Value::Closure {
                binder: 0,
                body: term(Term::Var {
                    id: 1,
                    name: "next".into(),
                }),
                environment,
            }))
            .unwrap();
        machine
            .allocate(Thunk::Ready(Value::Request {
                name: name.into(),
                arguments: vec![argument],
                continuation,
            }))
            .unwrap()
    }

    #[test]
    fn collection_preserves_console_order_request_continuations_and_halt_fields() {
        let program = program();
        let mut machine = Machine::new(&program);
        machine.gc_mode = super::super::gc::Mode::EverySafePoint;
        // Ordinary Word and String syntax exercises both saved-tail scopes.
        let code = machine
            .expression(parse_term("4294967295").unwrap(), 0)
            .unwrap();
        let message = machine
            .expression(parse_term("\"halt\"").unwrap(), 0)
            .unwrap();
        let halt = machine
            .allocate(Thunk::Ready(Value::Constructor {
                name: "Halt".into(),
                fields: vec![code, message],
            }))
            .unwrap();
        let last = request(&mut machine, "IO.print", "last", halt);
        let middle = request(&mut machine, "IO.print_err", "middle", last);
        let first = request(&mut machine, "IO.write", "first", middle);
        let events = Events::default();
        let mut stdout = Writer {
            stream: "stdout",
            events: Rc::clone(&events),
        };
        let mut stderr = Writer {
            stream: "stderr",
            events: Rc::clone(&events),
        };
        assert_eq!(
            machine.drive_io(first, &mut stdout, &mut stderr).unwrap(),
            u32::MAX
        );
        assert_eq!(
            *events.borrow(),
            vec![
                ("stdout", b"first".to_vec()),
                ("stderr", b"middle".to_vec()),
                ("stderr", b"\n".to_vec()),
                ("stdout", b"last".to_vec()),
                ("stdout", b"\n".to_vec()),
                ("stderr", b"halt".to_vec()),
                ("stderr", b"\n".to_vec()),
            ]
        );
        assert!(machine.gc.collections > 1);
    }

    #[test]
    fn cancellation_during_collected_request_decoding_precedes_console_output() {
        let program = program();
        let mut machine = Machine::new(&program);
        machine.gc_mode = super::super::gc::Mode::EverySafePoint;
        let terminal = machine
            .expression(parse_term("Emit{Unit{}}").unwrap(), 0)
            .unwrap();
        let request = request(&mut machine, "IO.print", "pending output", terminal);
        let checks = Cell::new(0);
        let cancelled = || {
            let checks_now = checks.get() + 1;
            checks.set(checks_now);
            checks_now >= 200
        };
        machine.cancelled = Some(&cancelled);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = machine
            .drive_io(request, &mut stdout, &mut stderr)
            .unwrap_err()
            .to_string();
        assert!(error.contains("execution cancelled"), "{error}");
        assert!(stdout.is_empty() && stderr.is_empty());
        assert!(machine.gc.collections > 0);
    }

    #[test]
    fn collection_does_not_make_a_private_request_matchable_as_constructor_data() {
        let program = program();
        let mut machine = Machine::new(&program);
        machine.gc_mode = super::super::gc::Mode::EverySafePoint;
        let terminal = machine
            .expression(parse_term("Emit{Unit{}}").unwrap(), 0)
            .unwrap();
        let request = request(&mut machine, "IO.print", "must not execute", terminal);
        let impossible = machine.allocate(Thunk::Ready(Value::Impossible)).unwrap();
        let matcher = machine
            .allocate(Thunk::Ready(Value::Match {
                constructor: "Emit".into(),
                arm: impossible,
                fallback: impossible,
            }))
            .unwrap();
        let call = machine
            .allocate(Thunk::Application(matcher, request))
            .unwrap();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = machine
            .drive_io(call, &mut stdout, &mut stderr)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("foreign effect request cannot be matched"),
            "{error}"
        );
        assert!(stdout.is_empty() && stderr.is_empty());
        assert!(machine.gc.collections > 0);
    }
}
