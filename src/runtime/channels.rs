// SPDX-License-Identifier: Apache-2.0
//! Native channel transitions ported from Bend 2.0.5 comp.ts and effs/chan_*.c.
//! Copyright 2026 `HigherOrderCO`. Rust implementation and bounds: `TeamDman`.
//! See NOTICE and licenses/Apache-2.0.txt.

use super::ARENA_LIMIT;
use super::ThunkId;
use crate::kernel::KernelError;
use std::collections::VecDeque;

/// Kept inside a sealed runtime value; source programs cannot forge handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Handle {
    index: usize,
    generation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Answer {
    Bool(bool),
    Some(ThunkId),
    None,
    Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Outcome {
    Ready(Answer),
    Parked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Wake {
    pub(super) continuation: ThunkId,
    pub(super) answer: Answer,
}

/// Wakes retain registration order. Applying continuations belongs to the
/// scheduler: this module never evaluates payloads or runs a resumed task.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Transition {
    pub(super) outcome: Outcome,
    pub(super) wakes: Vec<Wake>,
}

impl Transition {
    fn ready(answer: Answer) -> Self {
        Self {
            outcome: Outcome::Ready(answer),
            wakes: Vec::new(),
        }
    }

    fn parked() -> Self {
        Self {
            outcome: Outcome::Parked,
            wakes: Vec::new(),
        }
    }

    fn waking(answer: Answer, continuation: ThunkId, wake_answer: Answer) -> Self {
        Self {
            outcome: Outcome::Ready(answer),
            wakes: vec![Wake {
                continuation,
                answer: wake_answer,
            }],
        }
    }

    /// These roots have left the channel table. Keep them alive until all
    /// returned answers and wake applications have been committed to the VM.
    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        if let Outcome::Ready(Answer::Some(payload)) = self.outcome {
            visit(payload)?;
        }
        for wake in &self.wakes {
            visit(wake.continuation)?;
            if let Answer::Some(payload) = wake.answer {
                visit(payload)?;
            }
        }
        Ok(())
    }
}

enum Waiter {
    Sender {
        continuation: ThunkId,
        payload: ThunkId,
    },
    Receiver {
        continuation: ThunkId,
    },
}

struct Channel {
    room: u32,
    closed: bool,
    buffer: VecDeque<ThunkId>,
    wait: VecDeque<Waiter>,
}

struct Row {
    generation: u32,
    channel: Option<Channel>,
}

#[derive(Default)]
pub(super) struct State {
    rows: Vec<Row>,
    free: Vec<usize>,
    payloads: usize,
    waiters: usize,
}

/// Avoid retaining a large historical allocation in each otherwise idle row.
/// Along with aggregate entry bounds this keeps table memory bounded linearly.
fn compact<T>(queue: &mut VecDeque<T>) {
    if queue.capacity() > queue.len().saturating_mul(4).max(4) {
        queue.shrink_to_fit();
    }
}

impl State {
    pub(super) fn open(&mut self, room: u32) -> Result<Handle, KernelError> {
        let index = if let Some(index) = self.free.pop() {
            index
        } else {
            if self.rows.len() >= ARENA_LIMIT {
                return Err(KernelError::new("IO channel table budget exhausted"));
            }
            self.rows.push(Row {
                generation: 0,
                channel: None,
            });
            self.rows.len() - 1
        };
        let row = &mut self.rows[index];
        row.generation = row
            .generation
            .checked_add(1)
            .ok_or_else(|| KernelError::new("IO channel generation exhausted"))?;
        row.channel = Some(Channel {
            room,
            closed: false,
            buffer: VecDeque::new(),
            wait: VecDeque::new(),
        });
        Ok(Handle {
            index,
            generation: row.generation,
        })
    }

    fn at(&self, handle: Handle) -> Option<&Channel> {
        let row = self.rows.get(handle.index)?;
        (row.generation == handle.generation)
            .then_some(row.channel.as_ref())
            .flatten()
    }

    fn release(&mut self, index: usize) {
        let row = &mut self.rows[index];
        row.channel = None;
        // Never revive a stale handle after generation wraparound. Retired
        // entries still count towards the table budget.
        if row.generation < u32::MAX {
            self.free.push(index);
        }
    }

    pub(super) fn send(
        &mut self,
        handle: Handle,
        payload: ThunkId,
        continuation: ThunkId,
    ) -> Result<Transition, KernelError> {
        let Some(channel) = self.at(handle) else {
            return Ok(Transition::ready(Answer::Bool(false)));
        };
        if channel.closed {
            return Ok(Transition::ready(Answer::Bool(false)));
        }
        let has_receiver = matches!(channel.wait.front(), Some(Waiter::Receiver { .. }));
        let can_buffer = channel.buffer.len() < channel.room as usize;
        if !has_receiver {
            if self.payloads >= ARENA_LIMIT {
                return Err(KernelError::new("IO channel payload budget exhausted"));
            }
            if !can_buffer && self.waiters >= ARENA_LIMIT {
                return Err(KernelError::new("IO channel waiter budget exhausted"));
            }
        }
        let channel = self.rows[handle.index]
            .channel
            .as_mut()
            .ok_or_else(|| KernelError::new("invalid native channel row"))?;
        if has_receiver {
            let Some(Waiter::Receiver { continuation }) = channel.wait.pop_front() else {
                return Err(KernelError::new("invalid native channel receiver"));
            };
            self.waiters -= 1;
            compact(&mut channel.wait);
            return Ok(Transition::waking(
                Answer::Bool(true),
                continuation,
                Answer::Some(payload),
            ));
        }
        self.payloads += 1;
        if can_buffer {
            channel.buffer.push_back(payload);
            return Ok(Transition::ready(Answer::Bool(true)));
        }
        channel.wait.push_back(Waiter::Sender {
            continuation,
            payload,
        });
        self.waiters += 1;
        Ok(Transition::parked())
    }

    pub(super) fn recv(
        &mut self,
        handle: Handle,
        continuation: ThunkId,
    ) -> Result<Transition, KernelError> {
        let Some(channel) = self.at(handle) else {
            return Ok(Transition::ready(Answer::None));
        };
        let has_sender = matches!(channel.wait.front(), Some(Waiter::Sender { .. }));
        if channel.buffer.is_empty()
            && !has_sender
            && !channel.closed
            && self.waiters >= ARENA_LIMIT
        {
            return Err(KernelError::new("IO channel waiter budget exhausted"));
        }
        let channel = self.rows[handle.index]
            .channel
            .as_mut()
            .ok_or_else(|| KernelError::new("invalid native channel row"))?;
        if let Some(payload) = channel.buffer.pop_front() {
            self.payloads -= 1;
            let mut transition = Transition::ready(Answer::Some(payload));
            if let Some(waiter) = channel.wait.pop_front() {
                let Waiter::Sender {
                    continuation,
                    payload,
                } = waiter
                else {
                    return Err(KernelError::new("invalid native buffered channel waiter"));
                };
                channel.buffer.push_back(payload);
                self.waiters -= 1;
                transition.wakes.push(Wake {
                    continuation,
                    answer: Answer::Bool(true),
                });
            }
            compact(&mut channel.buffer);
            compact(&mut channel.wait);
            if channel.closed && channel.buffer.is_empty() {
                self.release(handle.index);
            }
            return Ok(transition);
        }
        if has_sender {
            let Some(Waiter::Sender {
                continuation,
                payload,
            }) = channel.wait.pop_front()
            else {
                return Err(KernelError::new("invalid native channel sender"));
            };
            self.payloads -= 1;
            self.waiters -= 1;
            compact(&mut channel.wait);
            return Ok(Transition::waking(
                Answer::Some(payload),
                continuation,
                Answer::Bool(true),
            ));
        }
        if channel.closed {
            self.release(handle.index);
            return Ok(Transition::ready(Answer::None));
        }
        channel.wait.push_back(Waiter::Receiver { continuation });
        self.waiters += 1;
        Ok(Transition::parked())
    }

    pub(super) fn close(&mut self, handle: Handle) -> Transition {
        let mut transition = Transition::ready(Answer::Unit);
        if self.at(handle).is_none_or(|channel| channel.closed) {
            return transition;
        }
        let Some(channel) = self.rows[handle.index].channel.as_mut() else {
            return transition;
        };
        channel.closed = true;
        for waiter in channel.wait.drain(..) {
            self.waiters -= 1;
            let wake = match waiter {
                Waiter::Sender { continuation, .. } => {
                    self.payloads -= 1;
                    Wake {
                        continuation,
                        answer: Answer::Bool(false),
                    }
                }
                Waiter::Receiver { continuation } => Wake {
                    continuation,
                    answer: Answer::None,
                },
            };
            transition.wakes.push(wake);
        }
        compact(&mut channel.wait);
        if channel.buffer.is_empty() {
            self.release(handle.index);
        }
        transition
    }

    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        for row in &self.rows {
            let Some(channel) = &row.channel else {
                continue;
            };
            for payload in &channel.buffer {
                visit(*payload)?;
            }
            for waiter in &channel.wait {
                match waiter {
                    Waiter::Sender {
                        continuation,
                        payload,
                    } => {
                        visit(*continuation)?;
                        visit(*payload)?;
                    }
                    Waiter::Receiver { continuation } => visit(*continuation)?,
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "channels/tests.rs"]
mod tests;
