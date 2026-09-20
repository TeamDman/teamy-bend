// SPDX-License-Identifier: MPL-2.0
//! Connect sealed channel values and pure transitions to the native scheduler.

use super::super::channels::Answer;
use super::super::channels::Handle;
use super::super::channels::Outcome;
use super::BuiltinForeign;
use super::KernelError;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;

impl Machine<'_> {
    pub(super) fn channel_request(
        &mut self,
        builtin: BuiltinForeign,
        arguments: &[ThunkId],
        continuation: ThunkId,
    ) -> Result<Option<ThunkId>, KernelError> {
        let transition = match (builtin, arguments) {
            (BuiltinForeign::ChanNew, [_, room]) => {
                let room = self.read_u32(*room)?;
                let handle = self.channels.open(room)?;
                let answer = self.allocate(Thunk::Ready(Value::Channel(handle)))?;
                return Ok(Some(
                    self.allocate(Thunk::Application(continuation, answer))?,
                ));
            }
            (BuiltinForeign::ChanSend, [_, channel, payload]) => {
                let handle = self.read_channel(*channel)?;
                self.channels.send(handle, *payload, continuation)?
            }
            (BuiltinForeign::ChanRecv, [_, channel]) => {
                let handle = self.read_channel(*channel)?;
                self.channels.recv(handle, continuation)?
            }
            (BuiltinForeign::ChanClose, [_, channel]) => {
                let handle = self.read_channel(*channel)?;
                self.channels.close(handle)
            }
            _ => return Err(KernelError::new("channel request has an invalid arity")),
        };
        let mut roots = Vec::new();
        transition.visit_roots(|id| {
            roots.push(id);
            Ok(())
        })?;
        self.with_roots(&roots, |machine| {
            // Wakes are enqueued in order. The current task continues until it
            // suspends or finishes; a successful send/recv is not a yield.
            for wake in transition.wakes {
                machine.tick()?;
                let answer = machine.channel_answer(wake.answer)?;
                let action = machine.allocate(Thunk::Application(wake.continuation, answer))?;
                machine.scheduler.resume(action)?;
            }
            match transition.outcome {
                Outcome::Ready(answer) => {
                    let answer = machine.channel_answer(answer)?;
                    Ok(Some(
                        machine.allocate(Thunk::Application(continuation, answer))?,
                    ))
                }
                Outcome::Parked => Ok(None),
            }
        })
    }

    fn read_channel(&mut self, thunk: ThunkId) -> Result<Handle, KernelError> {
        match self.force(thunk)? {
            Value::Channel(handle) => Ok(handle),
            Value::Request { .. } => Err(KernelError::new(
                "runtime fail-stop: a foreign effect request escaped the IO driver",
            )),
            _ => Err(KernelError::new("expected a sealed native channel handle")),
        }
    }

    fn channel_answer(&mut self, answer: Answer) -> Result<ThunkId, KernelError> {
        let (name, fields) = match answer {
            Answer::Bool(true) => ("True", Vec::new()),
            Answer::Bool(false) => ("False", Vec::new()),
            Answer::Some(payload) => ("Some", vec![payload]),
            Answer::None => ("None", Vec::new()),
            Answer::Unit => ("Unit", Vec::new()),
        };
        self.allocate(Thunk::Ready(Value::Constructor {
            name: name.into(),
            fields,
        }))
    }
}
