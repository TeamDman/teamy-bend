// SPDX-License-Identifier: Apache-2.0
//! Cooperative queue/timer semantics ported from Bend 2.0.5 comp.ts.
//! Copyright 2026 `HigherOrderCO`. Rust implementation and bounds: `TeamDman`.
//! See NOTICE and licenses/Apache-2.0.txt.

use super::ARENA_LIMIT;
use super::ThunkId;
use crate::kernel::KernelError;
use std::collections::VecDeque;

struct Timer {
    deadline: u64,
    action: ThunkId,
}

#[derive(Default)]
pub(super) struct State {
    ready: VecDeque<ThunkId>,
    timers: Vec<Timer>,
    live: usize,
}

impl State {
    pub(super) fn spawn(&mut self, action: ThunkId) -> Result<(), KernelError> {
        if self.live >= ARENA_LIMIT {
            return Err(KernelError::new("IO pending task budget exhausted"));
        }
        self.live += 1;
        self.ready.push_back(action);
        Ok(())
    }

    pub(super) fn next(&mut self) -> Option<ThunkId> {
        self.ready.pop_front()
    }

    /// A channel wake resumes an existing task without creating another one.
    pub(super) fn resume(&mut self, action: ThunkId) -> Result<(), KernelError> {
        if self.ready.len() + self.timers.len() >= self.live {
            return Err(KernelError::new("invalid native scheduler wake count"));
        }
        self.ready.push_back(action);
        Ok(())
    }

    pub(super) fn complete(&mut self) -> Result<(), KernelError> {
        self.live = self
            .live
            .checked_sub(1)
            .ok_or_else(|| KernelError::new("invalid native scheduler task count"))?;
        Ok(())
    }

    pub(super) const fn finished(&self) -> bool {
        self.live == 0
    }

    pub(super) fn sleep(&mut self, deadline: u64, action: ThunkId) -> Result<(), KernelError> {
        if self.timers.len() >= self.live || self.ready.len() + self.timers.len() >= ARENA_LIMIT {
            return Err(KernelError::new("IO pending task budget exhausted"));
        }
        self.timers.push(Timer { deadline, action });
        Ok(())
    }

    /// Only called when ready is empty. Preserve registration order among all
    /// overdue timers, even if their deadlines were registered out of order.
    pub(super) fn wake(&mut self, now: u64) {
        self.timers.retain(|timer| {
            if timer.deadline <= now {
                self.ready.push_back(timer.action);
                false
            } else {
                true
            }
        });
    }

    pub(super) fn next_deadline(&self) -> Option<u64> {
        self.timers.iter().map(|timer| timer.deadline).min()
    }

    pub(super) fn has_ready(&self) -> bool {
        !self.ready.is_empty()
    }

    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        for action in &self.ready {
            visit(*action)?;
        }
        for timer in &self.timers {
            visit(timer.action)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overdue_timers_follow_registration_order_not_deadline_order() {
        let mut scheduler = State::default();
        for id in 1..=3 {
            scheduler.spawn(id).unwrap();
        }
        assert_eq!(scheduler.next(), Some(1));
        scheduler.sleep(30, 4).unwrap();
        assert_eq!(scheduler.next(), Some(2));
        scheduler.sleep(10, 5).unwrap();
        assert_eq!(scheduler.next(), Some(3));
        scheduler.complete().unwrap();
        assert_eq!(scheduler.next_deadline(), Some(10));
        scheduler.wake(30);
        assert_eq!(scheduler.next(), Some(4));
        assert_eq!(scheduler.next(), Some(5));
        scheduler.complete().unwrap();
        scheduler.complete().unwrap();
        assert!(scheduler.finished());
    }

    #[test]
    fn live_queue_and_timer_roots_are_visited_and_errors_propagate() {
        let mut scheduler = State::default();
        scheduler.spawn(3).unwrap();
        scheduler.spawn(4).unwrap();
        assert_eq!(scheduler.next(), Some(3));
        scheduler.sleep(100, 5).unwrap();
        let mut roots = Vec::new();
        scheduler
            .visit_roots(|id| {
                roots.push(id);
                Ok(())
            })
            .unwrap();
        assert_eq!(roots, [4, 5]);
        assert!(
            scheduler
                .visit_roots(|_| Err(KernelError::new("cancelled")))
                .is_err()
        );
    }

    #[test]
    fn task_limit_is_reusable_after_completion() {
        let mut scheduler = State {
            live: ARENA_LIMIT,
            ..State::default()
        };
        assert!(scheduler.spawn(1).is_err());
        scheduler.complete().unwrap();
        scheduler.spawn(1).unwrap();
        assert_eq!(scheduler.next(), Some(1));
    }
}
