// SPDX-License-Identifier: MPL-2.0

use super::*;

fn ready(answer: Answer) -> Transition {
    Transition::ready(answer)
}

fn roots(state: &State) -> Vec<ThunkId> {
    let mut roots = Vec::new();
    state
        .visit_roots(|id| {
            roots.push(id);
            Ok(())
        })
        .unwrap();
    roots
}

#[test]
fn buffered_values_and_waiting_senders_are_fifo() {
    let mut state = State::default();
    let handle = state.open(2).unwrap();
    for payload in [11, 12] {
        assert_eq!(
            state.send(handle, payload, 90).unwrap(),
            ready(Answer::Bool(true))
        );
    }
    assert_eq!(state.send(handle, 13, 31).unwrap(), Transition::parked());
    assert_eq!(state.send(handle, 14, 32).unwrap(), Transition::parked());
    for (payload, continuation) in [(11, 31), (12, 32)] {
        assert_eq!(
            state.recv(handle, 90).unwrap(),
            Transition::waking(Answer::Some(payload), continuation, Answer::Bool(true))
        );
    }
    assert_eq!((state.payloads, state.waiters), (2, 0));
    for payload in [13, 14] {
        assert_eq!(
            state.recv(handle, 90).unwrap(),
            ready(Answer::Some(payload))
        );
    }
    assert_eq!((state.payloads, state.waiters), (0, 0));
    assert_eq!(state.recv(handle, 91).unwrap(), Transition::parked());
}

#[test]
fn rendezvous_queues_both_directions_and_zero_is_a_real_payload() {
    let mut state = State::default();
    let handle = state.open(0).unwrap();
    for continuation in [21, 22] {
        assert_eq!(
            state.recv(handle, continuation).unwrap(),
            Transition::parked()
        );
    }
    for (payload, continuation) in [(0, 21), (7, 22)] {
        assert_eq!(
            state.send(handle, payload, 90).unwrap(),
            Transition::waking(Answer::Bool(true), continuation, Answer::Some(payload))
        );
    }
    for (payload, continuation) in [(0, 31), (8, 32)] {
        assert_eq!(
            state.send(handle, payload, continuation).unwrap(),
            Transition::parked()
        );
    }
    for (payload, continuation) in [(0, 31), (8, 32)] {
        assert_eq!(
            state.recv(handle, 90).unwrap(),
            Transition::waking(Answer::Some(payload), continuation, Answer::Bool(true))
        );
    }
    assert_eq!((state.payloads, state.waiters), (0, 0));
}

#[test]
fn close_wakes_senders_false_without_appending_their_payloads_to_drain() {
    let mut state = State::default();
    let handle = state.open(2).unwrap();
    state.send(handle, 11, 90).unwrap();
    state.send(handle, 12, 90).unwrap();
    state.send(handle, 13, 31).unwrap();
    state.send(handle, 14, 32).unwrap();
    assert_eq!(
        state.close(handle),
        Transition {
            outcome: Outcome::Ready(Answer::Unit),
            wakes: vec![
                Wake {
                    continuation: 31,
                    answer: Answer::Bool(false)
                },
                Wake {
                    continuation: 32,
                    answer: Answer::Bool(false)
                },
            ],
        }
    );
    assert_eq!(state.close(handle), ready(Answer::Unit));
    assert_eq!(roots(&state), [11, 12]);
    assert_eq!(
        state.send(handle, 15, 90).unwrap(),
        ready(Answer::Bool(false))
    );
    assert_eq!(state.recv(handle, 90).unwrap(), ready(Answer::Some(11)));
    assert!(state.at(handle).is_some());
    assert_eq!(state.recv(handle, 90).unwrap(), ready(Answer::Some(12)));
    assert!(state.at(handle).is_none());
    assert_eq!(state.recv(handle, 90).unwrap(), ready(Answer::None));
    assert_eq!((state.payloads, state.waiters), (0, 0));
}

#[test]
fn close_wakes_receivers_none_in_registration_order() {
    let mut state = State::default();
    let handle = state.open(5).unwrap();
    state.recv(handle, 21).unwrap();
    state.recv(handle, 22).unwrap();
    assert_eq!(
        state.close(handle),
        Transition {
            outcome: Outcome::Ready(Answer::Unit),
            wakes: vec![
                Wake {
                    continuation: 21,
                    answer: Answer::None
                },
                Wake {
                    continuation: 22,
                    answer: Answer::None
                },
            ],
        }
    );
    assert_eq!(state.close(handle), ready(Answer::Unit));
    assert!(state.at(handle).is_none());
    assert!(roots(&state).is_empty());
    assert_eq!((state.payloads, state.waiters), (0, 0));
}

#[test]
fn row_reuse_cannot_reopen_a_stale_handle_and_generation_never_wraps() {
    let mut state = State::default();
    let old = state.open(0).unwrap();
    state.close(old);
    let new = state.open(1).unwrap();
    assert_eq!(old.index, new.index);
    assert_ne!(old.generation, new.generation);
    assert_eq!(state.send(old, 7, 8).unwrap(), ready(Answer::Bool(false)));
    assert_eq!(state.recv(old, 8).unwrap(), ready(Answer::None));
    assert_eq!(state.close(old), ready(Answer::Unit));
    assert_eq!(state.send(new, 7, 8).unwrap(), ready(Answer::Bool(true)));
    assert_eq!(state.recv(new, 8).unwrap(), ready(Answer::Some(7)));
    state.close(new);
    state.rows[new.index].generation = u32::MAX - 1;
    let final_generation = state.open(0).unwrap();
    assert_eq!(final_generation.generation, u32::MAX);
    state.close(final_generation);
    let separate = state.open(0).unwrap();
    assert_ne!(separate.index, final_generation.index);
    assert_eq!(
        state.recv(final_generation, 8).unwrap(),
        ready(Answer::None)
    );
}

#[test]
fn room_is_full_u32_without_upfront_allocation_and_payload_limit_is_reusable() {
    let mut state = State::default();
    let handle = state.open(u32::MAX).unwrap();
    assert_eq!(state.at(handle).unwrap().buffer.capacity(), 0);
    for payload in 0..ARENA_LIMIT {
        state.send(handle, payload, 90).unwrap();
    }
    let error = state.send(handle, ARENA_LIMIT, 90).unwrap_err();
    assert!(error.to_string().contains("payload budget"));
    assert_eq!(state.payloads, ARENA_LIMIT);
    assert_eq!(state.recv(handle, 90).unwrap(), ready(Answer::Some(0)));
    state.send(handle, ARENA_LIMIT, 90).unwrap();
    for payload in 1..=ARENA_LIMIT {
        assert_eq!(
            state.recv(handle, 90).unwrap(),
            ready(Answer::Some(payload))
        );
    }
    assert_eq!(state.payloads, 0);
    assert!(state.at(handle).unwrap().buffer.capacity() <= 4);
}

#[test]
fn waiter_budget_is_aggregate_and_reusable_after_wake() {
    let mut state = State::default();
    let receiver = state.open(0).unwrap();
    let sender = state.open(0).unwrap();
    for continuation in 0..ARENA_LIMIT {
        state.recv(receiver, continuation).unwrap();
    }
    assert!(
        state
            .recv(sender, 99)
            .unwrap_err()
            .to_string()
            .contains("waiter budget")
    );
    assert!(
        state
            .send(sender, 7, 99)
            .unwrap_err()
            .to_string()
            .contains("waiter budget")
    );
    assert_eq!((state.payloads, state.waiters), (0, ARENA_LIMIT));
    assert_eq!(
        state.send(receiver, 7, 99).unwrap(),
        Transition::waking(Answer::Bool(true), 0, Answer::Some(7))
    );
    state.send(sender, 8, 99).unwrap();
    assert_eq!((state.payloads, state.waiters), (1, ARENA_LIMIT));
    state.close(receiver);
    state.close(sender);
    assert_eq!((state.payloads, state.waiters), (0, 0));
}

#[test]
fn table_limit_is_reusable_and_capacity_does_not_count_as_payload_retention() {
    let mut state = State::default();
    let first = state.open(u32::MAX).unwrap();
    for _ in 1..ARENA_LIMIT {
        state.open(u32::MAX).unwrap();
    }
    assert!(
        state
            .open(0)
            .unwrap_err()
            .to_string()
            .contains("table budget")
    );
    state.close(first);
    let reused = state.open(0).unwrap();
    assert_eq!(reused.index, first.index);
    assert_eq!((state.payloads, state.waiters), (0, 0));
}

#[test]
fn live_and_detached_transition_roots_are_complete_and_propagate_errors() {
    let mut state = State::default();
    let buffered = state.open(1).unwrap();
    let receiving = state.open(0).unwrap();
    state.send(buffered, 11, 90).unwrap();
    state.send(buffered, 12, 31).unwrap();
    state.recv(receiving, 21).unwrap();
    assert_eq!(roots(&state), [11, 31, 12, 21]);
    assert!(
        state
            .visit_roots(|_| Err(KernelError::new("cancelled")))
            .is_err()
    );
    let transition = state.recv(buffered, 90).unwrap();
    let mut detached = Vec::new();
    transition
        .visit_roots(|id| {
            detached.push(id);
            Ok(())
        })
        .unwrap();
    assert_eq!(detached, [11, 31]);
    assert_eq!(roots(&state), [12, 21]);
    let transition = state.send(receiving, 13, 90).unwrap();
    detached.clear();
    transition
        .visit_roots(|id| {
            detached.push(id);
            Ok(())
        })
        .unwrap();
    assert_eq!(detached, [21, 13]);
    assert!(
        transition
            .visit_roots(|_| Err(KernelError::new("cancelled")))
            .is_err()
    );
    state = State::default();
    assert!(roots(&state).is_empty());
}
