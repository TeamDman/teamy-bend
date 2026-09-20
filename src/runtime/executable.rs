// SPDX-License-Identifier: MPL-2.0
//! Console execution of checked executable contracts. Requests remain private
//! machine values and are never reduced or accepted by the proof kernel.

use super::Machine;
use super::Program;
use super::Thunk;
use super::ThunkId;
use super::Value;
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
        // IO(A) keeps its erased R lambda in this machine. The driver supplies
        // an erased value followed by the terminal continuation; it never
        // manufactures a kernel proof or evaluates the result as evidence.
        let result_type = machine.allocate(Thunk::Ready(Value::Erased))?;
        let continuation = machine.allocate(Thunk::Ready(Value::EmitContinuation))?;
        let action = machine.allocate(Thunk::Application(entry, result_type))?;
        let mut current = machine.allocate(Thunk::Application(action, continuation))?;
        loop {
            machine.tick()?;
            match machine.force(current)? {
                Value::Constructor { name, fields } if name == "Emit" && fields.len() == 1 => {
                    // Successful IO discards its result, including IO(U32).
                    machine.flush(stdout, "stdout")?;
                    return Ok(0);
                }
                Value::Constructor { name, fields } if name == "Halt" && fields.len() == 2 => {
                    let code = machine.read_u32(fields[0])?;
                    let message = machine.read_text(fields[1])?;
                    machine.flush(stdout, "stdout")?;
                    machine.write_bytes(stderr, &message, "stderr")?;
                    machine.write_bytes(stderr, b"\n", "stderr")?;
                    machine.flush(stderr, "stderr")?;
                    return Ok(code);
                }
                Value::Request {
                    name,
                    arguments,
                    continuation,
                } => {
                    let answer = machine.console_request(&name, &arguments, stdout, stderr)?;
                    current = machine.allocate(Thunk::Application(continuation, answer))?;
                }
                _ => return Err(KernelError::new("main did not produce an IO operation")),
            }
        }
    }
}

impl Machine<'_> {
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
            BuiltinForeign::Spawn | BuiltinForeign::Sleep | BuiltinForeign::Now => {
                return Err(KernelError::new(format!(
                    "native execution does not support the scheduler builtin {name}; compile to executable JavaScript"
                )));
            }
            BuiltinForeign::ChanNew
            | BuiltinForeign::ChanSend
            | BuiltinForeign::ChanRecv
            | BuiltinForeign::ChanClose => {
                return Err(KernelError::new(format!(
                    "native execution does not support the channel builtin {name}; compile to executable JavaScript"
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
        match self.force(thunk)? {
            Value::Constructor { name, fields } => Ok((name, fields)),
            Value::Request { .. } => Err(KernelError::new(
                "runtime fail-stop: a foreign effect request escaped the IO driver",
            )),
            _ => Err(KernelError::new("console argument is not constructor data")),
        }
    }

    fn read_u32(&mut self, thunk: ThunkId) -> Result<u32, KernelError> {
        let (name, fields) = self.constructor(thunk)?;
        let [word] = fields.as_slice() else {
            return Err(KernelError::new("expected a U32 value"));
        };
        let mut word = *word;
        if name != "U32" {
            return Err(KernelError::new("expected a U32 value"));
        }
        let mut result = 0;
        for bit in 0..32 {
            let (name, fields) = self.constructor(word)?;
            let [head, tail] = fields.as_slice() else {
                return Err(KernelError::new("expected a 32-bit Word"));
            };
            if name != "WCon" {
                return Err(KernelError::new("expected a 32-bit Word"));
            }
            let (name, fields) = self.constructor(*head)?;
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
                    let (name, fields) = self.constructor(*head)?;
                    let [code] = fields.as_slice() else {
                        return Err(KernelError::new("expected a Char value"));
                    };
                    if name != "Chr" {
                        return Err(KernelError::new("expected a Char value"));
                    }
                    let scalar = char::from_u32(self.read_u32(*code)?).ok_or_else(|| {
                        KernelError::new("console text contains an invalid Unicode scalar")
                    })?;
                    let mut bytes = [0; 4];
                    let encoded = scalar.encode_utf8(&mut bytes).as_bytes();
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
mod channel_tests {
    use super::BuiltinForeign;
    use super::ForeignDefinition;
    use super::Machine;
    use super::Program;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    #[test]
    fn every_channel_dispatch_refuses_before_argument_decoding() {
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
                .expect_err("native channels are not implemented")
                .to_string();
            assert!(
                error.contains(&format!("channel builtin {name}")),
                "{error}"
            );
            assert!(stdout.is_empty() && stderr.is_empty());
        }
    }
}
