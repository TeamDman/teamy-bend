/* SPDX-License-Identifier: Apache-2.0
 * Task/word protocol derived from Bend 2.0.5 comp.ts and the team's CPU port.
 * Copyright 2026 HigherOrderCO. CUDA offset storage and bounded launch phases:
 * TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * Include executable_device_control.h immediately before this source. */
enum { TB_OUTCOME_WORDS = 0, TB_OUTCOME_TASK = 1, TB_OUTCOME_CALL = 2 };
struct TBOutcome { Term task; const Term *words, *owned; u32 count, pending; };
struct TBCallFrame {
  size_t pc, destination;
  u32 expected, tail_result;
  bool waiting;
  Term *values;
  const Term *captures;
  Term argument;
  Term *result;
};
INLINE TBOutcome tb_segment_task(Term task) {
  TBOutcome result = {task, NULL, NULL, 0, TB_OUTCOME_TASK}; return result;
}
INLINE TBOutcome tb_segment_words(const Term *words, const Term *owned, u32 count) {
  TBOutcome result = {0, words, owned, count, TB_OUTCOME_WORDS}; return result;
}
INLINE TBOutcome tb_segment_call(Fid fid, u32 count, const Term *words, const Term *owned) {
  TBOutcome result = {fid, words, owned, count, TB_OUTCOME_CALL}; return result;
}
OUTLINE TBOutcome tb_device_resume(Fid fid, const Env *e, TBCallFrame *frame);
enum { TB_RESULT_NONE = 0, TB_RESULT_RAW64 = 1, TB_RESULT_BOX64 = 3,
       TB_RESULT_RAW32 = 5, TB_RESULT_BOX32 = 7 };
INLINE u32 tb_result_compose(u32 outer, u32 inner) {
  if ((outer != 0 && ((outer & 1u) == 0 || outer > 7))
      || (inner != 0 && ((inner & 1u) == 0 || inner > 7)))
    err_fail("invalid scalar tail result adapter");
  return outer == 0 ? inner : (outer & 3u) | ((outer | inner) & 4u);
}
INLINE Term tb_tail_result(TBCallFrame *frame, Term task, u32 mode) {
  if (frame == NULL || frame->waiting || mode == TB_RESULT_NONE)
    err_fail("invalid scalar tail result adapter");
  frame->tail_result = tb_result_compose(frame->tail_result, mode); return task;
}

/* These records are persistent. They contain offsets and words only; the
 * pointer-bearing callback view above is reconstructed on a lane's stack. */
struct TBDeviceFrame {
  u64 captures, values, parent;
  Term argument, result;
  u32 fid, slots, pc, destination, expected, tail_result, saved_result, waiting;
};
enum { TB_DEVICE_RUN_READY, TB_DEVICE_RUN_WORDS, TB_DEVICE_RUN_GRAPH };
struct TBDeviceRun {
  u64 next, target, current, pending, packet;
  Term task;
  u32 expected, tail_result, packet_count, event;
};
struct TBDeviceRecord {
  u64 hash_next, adopt_next, parent, boundary;
  Term task;
  u32 arity, index, remaining, width;
  u64 reserved[4], delivered[4];
};
static __device__ TBDeviceControl *tb_device_control;

INLINE u64 tb_device_offset(const void *memory) {
  if (memory == NULL) return 0;
  u64 address = (u64)memory, base = (u64)tb_device_scratch;
  if (address < base || (address - base) % sizeof(Term) != 0)
    err_fail("invalid device scratch pointer");
  u64 offset = (address - base) / sizeof(Term);
  if (offset < 4 || offset >= tb_device_state->scratch_bump)
    err_fail("invalid device scratch offset");
  return offset;
}
INLINE void *tb_device_pointer(u64 offset, size_t bytes) {
  if (offset == 0) {
    if (bytes != 0) err_fail("missing device scratch span");
    return NULL;
  }
  if (bytes > SIZE_MAX - 7 || offset < 4 || offset >= tb_device_state->scratch_bump)
    err_fail("invalid device scratch span");
  u64 at = offset - 2, header = tb_device_scratch[at];
  u32 cls = (u32)(header & ~TB_SCRATCH_ACTIVE);
  if (cls < 1 || cls >= NCLS_ALL || header != (TB_SCRATCH_ACTIVE | cls)
      || ((u64)bytes + 7) / 8 > tb_device_scratch[at + 1]
      || (UINT64_C(1) << cls) > tb_device_state->scratch_bump - at)
    err_fail("invalid device scratch allocation");
  return tb_device_scratch + offset;
}
INLINE TBDeviceFrame *tb_device_frame(u64 at) {
  return (TBDeviceFrame *)tb_device_pointer(at, sizeof(TBDeviceFrame));
}
INLINE TBDeviceRun *tb_device_run(u64 at) {
  return (TBDeviceRun *)tb_device_pointer(at, sizeof(TBDeviceRun));
}
INLINE TBDeviceRecord *tb_device_record(u64 at) {
  return (TBDeviceRecord *)tb_device_pointer(at, sizeof(TBDeviceRecord));
}
INLINE void tb_device_lock(void) {
  for (;;) {
    tb_device_check_cancelled();
    if (atomicCAS(&tb_device_control->lock, 0u, 1u) == 0) break;
    __nanosleep(64);
  }
  __threadfence();
}
INLINE void tb_device_unlock(void) {
  __threadfence(); atomicExch(&tb_device_control->lock, 0u);
}
INLINE void tb_device_count_frame(bool allocate) {
  tb_device_lock();
  if (allocate) {
    if (tb_device_control->live_frames >= tb_device_control->frame_limit)
      err_fail("generated continuation budget exhausted");
    ++tb_device_control->live_frames;
  } else {
    if (tb_device_control->live_frames == 0) err_fail("unbalanced generated continuation");
    --tb_device_control->live_frames;
  }
  tb_device_unlock();
}
OUTLINE Term *tb_frame_push(size_t slots) {
  if (slots > SIZE_MAX / sizeof(Term)) err_fail("generated frame allocation overflow");
  u64 lane = (u64)blockIdx.x * blockDim.x + threadIdx.x;
  if (lane >= tb_device_control->helper_lanes) err_fail("invalid device helper lane");
  u32 *depths = (u32 *)tb_device_pointer(tb_device_control->helper_depths,
    (size_t)tb_device_control->helper_lanes * sizeof(u32));
  if (depths[lane] >= tb_device_control->helper_limit)
    err_fail("generated frame depth budget exhausted");
  u32 depth = ++depths[lane];
  atomicAdd(&tb_device_control->helper_live, 1ull);
  atomicAdd(&tb_device_control->helper_calls, 1ull);
  atomicMax(&tb_device_control->helper_peak, depth);
  return slots == 0 ? NULL : (Term *)io_mem(tb_host_calloc(slots, sizeof(Term)));
}
OUTLINE void tb_frame_pop(Term *values) {
  u64 lane = (u64)blockIdx.x * blockDim.x + threadIdx.x;
  if (lane >= tb_device_control->helper_lanes) err_fail("invalid device helper lane");
  u32 *depths = (u32 *)tb_device_pointer(tb_device_control->helper_depths,
    (size_t)tb_device_control->helper_lanes * sizeof(u32));
  if (depths[lane] == 0) err_fail("unbalanced generated frame");
  tb_host_free(values);
  --depths[lane]; atomicAdd(&tb_device_control->helper_live, UINT64_MAX);
}
INLINE u64 tb_device_copy(const Term *words, u32 count) {
  if (count == 0) return 0;
  if (words == NULL) err_fail("missing device word span");
  Term *copy = (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
  memcpy(copy, words, count * sizeof(Term)); return tb_device_offset(copy);
}
OUTLINE u64 tb_device_frame_new(Fid fid, const Term *words, Term argument) {
  u32 kind = tb_device_fid_kind(fid), count;
  if (fid != FID_IO_EMIT && kind != 1 && kind != 2) err_fail("unregistered device function");
  count = fid == FID_IO_EMIT ? 0 : kind == 2 ? fid_arity(fid) : tb_closure_captures[fid];
  tb_device_count_frame(true);
  TBDeviceFrame *frame = (TBDeviceFrame *)io_mem(tb_host_calloc(1, sizeof(*frame)));
  frame->captures = tb_device_copy(words, count);
  frame->slots = fid == FID_IO_EMIT ? 0 : tb_device_fid_slots(fid);
  frame->values = frame->slots == 0 ? 0 : tb_device_offset(
    io_mem(tb_host_calloc(frame->slots, sizeof(Term))));
  frame->fid = fid; frame->argument = argument; frame->expected = 1;
  return tb_device_offset(frame);
}
OUTLINE void tb_device_frame_free(u64 at) {
  TBDeviceFrame *frame = tb_device_frame(at);
  tb_host_free(tb_device_pointer(frame->captures, 0));
  tb_host_free(tb_device_pointer(frame->values, 0));
  tb_host_free(frame); tb_device_count_frame(false);
}
INLINE bool tb_device_bit(const u64 *bits, u32 at) {
  return (bits[at >> 6] & (UINT64_C(1) << (at & 63))) != 0;
}
INLINE void tb_device_set(u64 *bits, u32 at) { bits[at >> 6] |= UINT64_C(1) << (at & 63); }
INLINE void tb_device_arguments(const Term *words, const Term *owned, u32 count) {
  if (count != 0 && words == NULL) err_fail("invalid direct segment arguments");
  for (u32 i = 0; i < count; ++i) {
    if (owned != NULL && owned[i] > 1) err_fail("invalid task ownership mask");
    if (owned != NULL && owned[i] == 0) continue;
    if (words[i] == TERM_HOLE) err_fail("foreign task argument is missing");
    if (term_tag(words[i]) == TAG_TSK) err_fail("runnable task contains a pending task");
  }
}
INLINE void tb_device_payload(Env e, Loc at, u32 count) {
  for (u32 i = 0; i < count; ++i) {
    if (!tb_cell_owned(e, at + i)) continue;
    tb_device_arguments(e.mem + at + i, NULL, 1);
  }
}
INLINE Loc task_node(Env e, Fid fid, Term continuation, u32 index, u32 remaining) {
  u32 arity = fid_arity(fid);
  if ((continuation == TERM_HOLE && index != 0)
      || (continuation != TERM_HOLE && (term_tag(continuation) != TAG_TSK || term_rfc(continuation))))
    err_fail("foreign task continuation is unsupported");
  if (remaining > arity) err_fail("invalid task dependency count");
  Loc at = heap_alloc(e, cls_fit(arity + 2));
  for (u32 i = 0; i < arity; ++i) e.mem[at + i] = TERM_HOLE;
  e.mem[at + arity] = continuation;
  e.mem[at + arity + 1] = ((u64)index << 32) | remaining;
  tb_mark_raw(e, at + arity, 2); return at;
}
INLINE Loc task_tail(Term task) {
  Env e = {tb_memory, NULL};
  if (term_tag(task) != TAG_TSK || term_rfc(task)) err_fail("foreign task is unsupported");
  u32 arity = fid_arity((Fid)term_aux(task));
  tb_allocation(e, term_loc(task), cls_fit(arity + 2)); return term_loc(task) + arity;
}
INLINE Term tb_word_task(Env e, Fid fid, u32 count, const Term *words, const Term *owned) {
  if (count != fid_arity(fid) || (count != 0 && words == NULL)) err_fail("invalid word task payload");
  Loc at = task_node(e, fid, TERM_HOLE, 0, 0);
  for (u32 i = 0; i < count; ++i) {
    if (owned != NULL && owned[i] > 1) err_fail("invalid task ownership mask");
    e.mem[at + i] = words[i];
    if (owned != NULL && owned[i] == 0) tb_mark_raw(e, at + i, 1);
  }
  return term_tsk(fid, at);
}
INLINE Term tb_tail_apply(Env e, Term closure, Term argument) {
  Term words[2] = {closure, argument}; return tb_word_task(e, FID_CLO_APPLY, 2, words, NULL);
}
INLINE Term tb_closure(Env e, u32 fid, u32 count, const Term *captures) {
  if ((fid != FID_IO_EMIT && tb_device_fid_kind(fid) != 1)
      || count != (fid == FID_IO_EMIT ? 0 : tb_closure_captures[fid])
      || (count != 0 && captures == NULL)) err_fail("unregistered closure");
  Loc at = 0;
  if (count != 0) { at = heap_alloc(e, cls_fit(count)); memcpy(e.mem + at, captures, count * sizeof(Term)); }
  return term_clo(fid, at);
}

INLINE u64 tb_device_packet(TBOutcome outcome) {
  if (outcome.count == 0 || outcome.count > 255 || outcome.words == NULL)
    err_fail("invalid task result width");
  Term *packet = (Term *)io_mem(tb_host_malloc(2 * outcome.count * sizeof(Term)));
  for (u32 i = 0; i < outcome.count; ++i) {
    if (outcome.owned != NULL && outcome.owned[i] > 1) err_fail("invalid task ownership mask");
    packet[i] = outcome.words[i]; packet[outcome.count + i] = outcome.owned == NULL ? 1 : outcome.owned[i];
  }
  return tb_device_offset(packet);
}
INLINE void tb_device_enqueue(u64 offset, bool complete) {
  TBDeviceRun *run = tb_device_run(offset); run->next = 0;
  tb_device_lock();
  u64 tail = complete ? tb_device_control->completed_tail : tb_device_control->ready_tail;
  if (tail != 0) tb_device_run(tail)->next = offset;
  else if (complete) tb_device_control->completed_head = offset;
  else tb_device_control->ready_head = offset;
  if (complete) { tb_device_control->completed_tail = offset; ++tb_device_control->completed_count; }
  else { tb_device_control->ready_tail = offset; ++tb_device_control->ready_count; }
  tb_device_unlock();
}
INLINE u64 tb_device_dequeue(bool complete) {
  tb_device_lock();
  u64 offset = complete ? tb_device_control->completed_head : tb_device_control->ready_head;
  if (offset != 0) {
    TBDeviceRun *run = tb_device_run(offset);
    if (complete) {
      tb_device_control->completed_head = run->next;
      if (run->next == 0) tb_device_control->completed_tail = 0;
      --tb_device_control->completed_count;
    } else {
      tb_device_control->ready_head = run->next;
      if (run->next == 0) tb_device_control->ready_tail = 0;
      --tb_device_control->ready_count;
    }
    run->next = 0;
  }
  tb_device_unlock(); return offset;
}
INLINE u32 tb_device_hash(Loc at) {
  u64 mixed = at * UINT64_C(11400714819323198485); return (u32)((mixed ^ (mixed >> 32)) & 1023);
}
OUTLINE u64 tb_device_record_new(Term task) {
  Loc tail = task_tail(task), at = term_loc(task);
  u32 hash = tb_device_hash(at);
  for (u64 link = tb_device_control->table[hash]; link != 0; link = tb_device_record(link)->hash_next)
    if (term_loc(tb_device_record(link)->task) == at) err_fail("duplicate or cyclic task graph");
  if (tb_device_control->live_records >= tb_device_control->task_limit) err_fail("task budget exhausted");
  TBDeviceRecord *record = (TBDeviceRecord *)io_mem(tb_host_calloc(1, sizeof(*record)));
  record->task = task; record->arity = fid_arity((Fid)term_aux(task));
  record->width = fid_result_width((Fid)term_aux(task));
  record->remaining = (u32)tb_memory[tail + 1]; record->index = (u32)(tb_memory[tail + 1] >> 32);
  if (record->remaining > record->arity) err_fail("invalid task dependency count");
  record->hash_next = tb_device_control->table[hash];
  u64 offset = tb_device_offset(record); tb_device_control->table[hash] = offset;
  ++tb_device_control->live_records;
  if (record->remaining != 0) ++tb_device_control->forks;
  return offset;
}
INLINE void tb_device_unindex(u64 offset) {
  TBDeviceRecord *record = tb_device_record(offset);
  u64 *link = &tb_device_control->table[tb_device_hash(term_loc(record->task))];
  while (*link != 0 && *link != offset) link = &tb_device_record(*link)->hash_next;
  if (*link != offset) err_fail("task ownership registry mismatch");
  *link = record->hash_next; record->hash_next = 0;
}
OUTLINE u64 tb_device_closure_frame(Env e, Term closure, Term argument) {
  if (term_tag(closure) != TAG_CLO || term_rfc(closure)) err_fail("application of a non-function");
  Fid fid = (Fid)term_aux(closure);
  if (fid != FID_IO_EMIT && tb_device_fid_kind(fid) != 1) err_fail("unregistered device closure");
  u32 count = fid == FID_IO_EMIT ? 0 : tb_closure_captures[fid];
  Loc at = term_loc(closure);
  if (count != 0) tb_allocation(e, at, cls_fit(count));
  else if (at != 0) err_fail("invalid empty closure payload");
  u64 frame = tb_device_frame_new(fid, count == 0 ? NULL : e.mem + at, argument);
  if (count != 0) heap_free(e, cls_fit(count), at);
  return frame;
}
OUTLINE u64 tb_device_task_frame(Env e, Term task) {
  Fid fid = (Fid)term_aux(task);
  Loc at = term_loc(task); u32 arity = fid_arity(fid);
  tb_device_payload(e, at, arity);
  u64 frame;
  if (fid == FID_CLO_APPLY) frame = tb_device_closure_frame(e, e.mem[at], e.mem[at + 1]);
  else if (fid == FID_IO_EMIT) frame = tb_device_frame_new(fid, NULL, e.mem[at]);
  else if (tb_device_fid_kind(fid) == 2) frame = tb_device_frame_new(fid, e.mem + at, 0);
  else if (tb_device_fid_kind(fid) == 1)
    frame = tb_device_frame_new(fid, e.mem + at, e.mem[at + arity - 1]);
  else err_fail("unregistered device task");
  heap_free(e, cls_fit(arity + 2), at); return frame;
}
OUTLINE void tb_device_start(Env e, u64 offset) {
  TBDeviceRecord *record = tb_device_record(offset);
  if (record->remaining != 0) err_fail("task started before its dependencies");
  if (tb_device_control->live_runs >= tb_device_control->run_limit) err_fail("device runnable budget exhausted");
  TBDeviceRun *run = (TBDeviceRun *)io_mem(tb_host_calloc(1, sizeof(*run)));
  run->target = offset; run->expected = record->width;
  tb_device_unindex(offset);
  run->current = tb_device_task_frame(e, record->task);
  ++tb_device_control->live_runs; tb_device_enqueue(tb_device_offset(run), false);
}

INLINE void tb_device_waiting(const TBDeviceFrame *frame) {
  if (frame->pc == 0 || frame->expected == 0 || frame->expected > 255
      || frame->destination > frame->slots || frame->expected > frame->slots - frame->destination)
    err_fail("invalid generated continuation destination");
}
/* One lane exclusively owns this run. All persisted state is restored before
 * yielding; outcomes borrow only until this dispatcher copies their vectors. */
OUTLINE void tb_device_advance(Env e, TBDeviceRun *run, u32 quantum) {
  run->event = TB_DEVICE_RUN_READY;
  for (u32 turn = 0; turn < quantum; ++turn) {
    tb_tick();
    if (run->current != 0) {
      u64 current = run->current;
      TBDeviceFrame *frame = tb_device_frame(current);
      if (frame->waiting || (frame->fid != FID_IO_EMIT && tb_device_fid_kind(frame->fid) == 0))
        err_fail("invalid generated continuation state");
      u32 count = frame->fid == FID_IO_EMIT ? 0
        : tb_device_fid_kind(frame->fid) == 2 ? fid_arity(frame->fid) : tb_closure_captures[frame->fid];
      TBCallFrame view;
      view.pc = frame->pc; view.destination = frame->destination;
      view.expected = frame->expected; view.tail_result = frame->tail_result;
      view.waiting = false; view.argument = frame->argument;
      /* Every borrowed result pointer targets persistent scratch storage.
       * Mixing a lane-local scalar with global typed-result pointers causes
       * NVRTC to infer a local address space for otherwise global outcomes. */
      frame->result = 0; view.result = &frame->result;
      view.captures = (const Term *)tb_device_pointer(frame->captures, count * sizeof(Term));
      view.values = (Term *)tb_device_pointer(frame->values, frame->slots * sizeof(Term));
      atomicAdd(&tb_device_control->dispatches, 1ull);
      TBOutcome outcome;
      if (frame->fid == FID_IO_EMIT) {
        if (cid_arity(CID_EMIT) != 1) err_fail("Emit representation unavailable");
        Loc at = heap_alloc(e, 0); e.mem[at] = view.argument;
        *view.result = term_ctr(CID_EMIT, at);
        outcome = tb_segment_words(view.result, NULL, 1);
      } else outcome = tb_device_resume(frame->fid, &e, &view);
      if (outcome.pending > TB_OUTCOME_CALL) err_fail("invalid segment outcome tag");
      if (view.pc > UINT32_MAX || view.destination > UINT32_MAX)
        err_fail("invalid generated continuation destination");
      frame->pc = (u32)view.pc; frame->destination = (u32)view.destination;
      frame->expected = view.expected; frame->tail_result = view.tail_result;
      frame->waiting = view.waiting; frame->argument = view.argument;
      if (frame->tail_result != TB_RESULT_NONE) {
        if (frame->waiting || outcome.pending == TB_OUTCOME_WORDS || fid_result_width(frame->fid) != 1)
          err_fail("invalid scalar tail result adapter");
        run->tail_result = tb_result_compose(run->tail_result, frame->tail_result);
      }
      if (frame->waiting) {
        tb_device_waiting(frame);
        if (outcome.pending == TB_OUTCOME_WORDS) err_fail("foreign task is unsupported");
      }
      if (outcome.pending == TB_OUTCOME_CALL) {
        if (outcome.task >= 65536 || tb_device_fid_kind((Fid)outcome.task) != 2)
          err_fail("invalid direct segment call");
        Fid fid = (Fid)outcome.task;
        if (outcome.count != fid_arity(fid)) err_fail("invalid direct segment arguments");
        u32 expected = frame->waiting ? frame->expected : run->expected;
        if (expected != fid_result_width(fid)) err_fail("task result width mismatch");
        tb_device_arguments(outcome.words, outcome.owned, outcome.count);
        atomicAdd(&tb_device_control->direct_calls, 1ull);
        if (!frame->waiting && fid == frame->fid) {
          Term *captures = (Term *)tb_device_pointer(frame->captures, count * sizeof(Term));
          if (count != 0) memmove(captures, outcome.words, count * sizeof(Term));
          if (frame->slots != 0) memset(view.values, 0, frame->slots * sizeof(Term));
          frame->pc = 0; frame->destination = 0; frame->expected = 1;
          frame->tail_result = 0; frame->argument = 0; frame->saved_result = 0; frame->parent = 0;
          atomicAdd(&tb_device_control->reused_calls, 1ull);
        } else {
          /* Copy before dropping the current frame, including when words and
           * ownership borrow overlapping regions of its scratch or captures. */
          u64 payload = tb_device_copy(outcome.words, outcome.count);
          if (frame->waiting) {
            run->expected = frame->expected;
            frame->saved_result = run->tail_result; run->tail_result = 0;
            frame->parent = run->pending; run->pending = current;
          } else tb_device_frame_free(current);
          run->current = tb_device_frame_new(fid,
            (const Term *)tb_device_pointer(payload, outcome.count * sizeof(Term)), 0);
          tb_host_free(tb_device_pointer(payload, 0));
        }
        continue;
      }
      if (outcome.pending == TB_OUTCOME_WORDS) {
        if (outcome.count != fid_result_width(frame->fid)) err_fail("task result width mismatch");
        run->packet = tb_device_packet(outcome); run->packet_count = outcome.count;
      }
      if (frame->waiting) {
        run->expected = frame->expected;
        frame->saved_result = run->tail_result; run->tail_result = 0;
        frame->parent = run->pending; run->pending = current;
      } else tb_device_frame_free(current);
      run->current = 0;
      if (outcome.pending == TB_OUTCOME_TASK) {
        Term task = outcome.task;
        Loc tail = task_tail(task);
        if (e.mem[tail] == TERM_HOLE && e.mem[tail + 1] == 0) {
          if (run->expected != fid_result_width((Fid)term_aux(task))) err_fail("task result width mismatch");
          run->current = tb_device_task_frame(e, task); continue;
        }
        run->task = task; run->event = TB_DEVICE_RUN_GRAPH; return;
      }
    }
    if (run->packet == 0 || run->packet_count != run->expected) err_fail("task result width mismatch");
    Term *packet = (Term *)tb_device_pointer(run->packet, 2 * run->packet_count * sizeof(Term));
    if (run->tail_result != 0) {
      if (run->packet_count != 1) err_fail("invalid scalar tail result adapter");
      u32 mode = tb_result_compose(0, run->tail_result);
      if ((mode & 4u) != 0) packet[0] = (u32)packet[0];
      packet[1] = (mode & 2u) != 0; run->tail_result = 0;
    }
    if (run->pending != 0) {
      TBDeviceFrame *frame = tb_device_frame(run->pending);
      tb_device_waiting(frame);
      if (!frame->waiting || frame->expected != run->packet_count) err_fail("invalid generated continuation destination");
      Term *values = (Term *)tb_device_pointer(frame->values, frame->slots * sizeof(Term));
      memcpy(values + frame->destination, packet, run->packet_count * sizeof(Term));
      run->current = run->pending; run->pending = frame->parent; frame->parent = 0;
      frame->waiting = 0; run->expected = fid_result_width(frame->fid);
      run->tail_result = frame->saved_result; frame->saved_result = 0;
      tb_host_free(packet); run->packet = 0; run->packet_count = 0; continue;
    }
    run->event = TB_DEVICE_RUN_WORDS; return;
  }
}

/* Graph indexing/adoption/delivery run only in the commit kernel, after all
 * lanes in the preceding work kernel have returned. Each commit unit handles
 * one record with at most 255 argument cells; the cursor survives kernel ends. */
OUTLINE void tb_device_adopt(Term task, u64 boundary, bool entry) {
  if (tb_device_control->adopt_head != 0) err_fail("nested device graph adoption");
  u64 root = tb_device_record_new(task);
  TBDeviceRecord *record = tb_device_record(root);
  if (tb_memory[task_tail(task)] != TERM_HOLE || record->index != 0)
    err_fail("foreign task continuation is unsupported");
  if (entry && record->remaining != 0) err_fail("corpus_eval requires a ready task");
  if (boundary != 0 && tb_device_run(boundary)->expected != record->width)
    err_fail("task result width mismatch");
  record->boundary = boundary;
  tb_device_control->adopt_head = root; tb_device_control->adopt_tail = root;
  tb_device_control->adopt_cursor = root; tb_device_control->adopt_boundary = boundary;
  tb_device_control->adopt_phase = 0;
}
OUTLINE void tb_device_adopt_step(Env e) {
  u64 offset = tb_device_control->adopt_cursor;
  if (offset == 0) err_fail("missing device adoption cursor");
  TBDeviceRecord *record = tb_device_record(offset);
  Loc at = term_loc(record->task);
  if (tb_device_control->adopt_phase == 0) {
    if (record->remaining == 0) tb_device_payload(e, at, record->arity);
    else {
      u32 children = 0;
      for (u32 i = 0; i < record->arity; ++i) {
        if (tb_device_bit(record->reserved, i) || !tb_cell_owned(e, at + i)) continue;
        Term value = e.mem[at + i];
        if (value == TERM_HOLE) err_fail("foreign task argument is missing");
        if (term_tag(value) == TAG_TSK) {
          u64 child_offset = tb_device_record_new(value);
          TBDeviceRecord *child = tb_device_record(child_offset);
          if (e.mem[task_tail(value)] != record->task || child->index != i)
            err_fail("task child destination mismatch");
          if (child->width > record->arity - i) err_fail("task result span is out of bounds");
          for (u32 j = 0; j < child->width; ++j) {
            if (tb_device_bit(record->reserved, i + j) || (j != 0 && e.mem[at + i + j] != TERM_HOLE))
              err_fail("task result spans overlap");
            tb_device_set(record->reserved, i + j);
          }
          child->parent = offset;
          tb_device_record(tb_device_control->adopt_tail)->adopt_next = child_offset;
          tb_device_control->adopt_tail = child_offset; ++children;
        }
      }
      if (children != record->remaining) err_fail("task dependency count mismatch");
    }
  } else if (tb_device_control->adopt_phase == 1) {
    for (u32 i = 0; i < record->arity; ++i)
      if (tb_device_bit(record->reserved, i)) e.mem[at + i] = TERM_HOLE;
  } else if (tb_device_control->adopt_phase != 2) err_fail("invalid device adoption phase");
  u64 next = record->adopt_next;
  if (tb_device_control->adopt_phase == 2 && record->remaining == 0) tb_device_start(e, offset);
  tb_device_control->adopt_cursor = next;
  if (next == 0) {
    if (++tb_device_control->adopt_phase < 3) tb_device_control->adopt_cursor = tb_device_control->adopt_head;
    else {
      tb_device_control->adopt_head = 0; tb_device_control->adopt_tail = 0;
      tb_device_control->adopt_boundary = 0; tb_device_control->adopt_phase = 0;
    }
  }
}
OUTLINE void tb_device_finish(Env e, u64 offset) {
  TBDeviceRun *run = tb_device_run(offset);
  TBDeviceRecord *record = tb_device_record(run->target);
  u32 count = run->packet_count;
  if (count != run->expected || count != record->width) err_fail("task result width mismatch");
  Term *packet = (Term *)tb_device_pointer(run->packet, 2 * count * sizeof(Term));
  if (record->parent != 0) {
    u64 parent_offset = record->parent;
    TBDeviceRecord *parent = tb_device_record(parent_offset);
    u32 index = record->index;
    Loc at = term_loc(parent->task), tail = task_tail(parent->task);
    if (index > parent->arity || count > parent->arity - index || parent->remaining == 0
        || (u32)e.mem[tail + 1] != parent->remaining) err_fail("duplicate task result delivery");
    for (u32 i = 0; i < count; ++i) {
      if (!tb_device_bit(parent->reserved, index + i) || tb_device_bit(parent->delivered, index + i)
          || e.mem[at + index + i] != TERM_HOLE) err_fail("duplicate task result delivery");
      e.mem[at + index + i] = packet[i];
      if (packet[count + i] == 0) tb_mark_raw(e, at + index + i, 1);
      else tb_mark_owned(e, at + index + i, 1);
      tb_device_set(parent->delivered, index + i);
    }
    --parent->remaining; --e.mem[tail + 1];
    if (parent->remaining == 0) tb_device_start(e, parent_offset);
  } else if (record->boundary != 0) {
    TBDeviceRun *boundary = tb_device_run(record->boundary);
    if (boundary->expected != count || boundary->packet != 0) err_fail("task result width mismatch");
    boundary->packet = run->packet; boundary->packet_count = count; boundary->event = TB_DEVICE_RUN_READY;
    tb_device_enqueue(record->boundary, false); run->packet = 0;
  } else {
    if (tb_device_control->done || count != tb_device_control->root_count)
      err_fail("invalid task root delivery");
    memcpy(tb_device_control->root_words, packet, count * sizeof(Term));
    memcpy(tb_device_control->root_owned, packet + count, count * sizeof(Term));
    tb_device_control->done = 1;
  }
  if (run->packet != 0) tb_host_free(packet);
  if (tb_device_control->live_records == 0 || tb_device_control->live_runs == 0)
    err_fail("unbalanced device task ownership");
  --tb_device_control->live_records; --tb_device_control->live_runs;
  tb_host_free(record); tb_host_free(run);
}

extern "C" __global__ void tb_device_tasks_begin(TBDeviceControl *control, Term root) {
  if (blockIdx.x != 0 || threadIdx.x != 0) return;
  tb_device_check_cancelled();
  tb_device_control = control;
  u32 run_limit = control->run_limit, task_limit = control->task_limit, frame_limit = control->frame_limit;
  u32 helper_lanes = control->helper_lanes, helper_limit = control->helper_limit;
  if (control->live_runs != 0 || control->live_records != 0 || control->live_frames != 0
      || control->helper_live != 0 || control->helper_depths != 0
      || tb_device_state->scratch_live != 0) err_fail("device invocation retained active work");
  if (helper_lanes == 0) err_fail("invalid device helper lane count");
  memset(control, 0, sizeof(*control));
  control->run_limit = run_limit == 0 ? 65536 : run_limit;
  control->task_limit = task_limit == 0 ? 65536 : task_limit;
  control->frame_limit = frame_limit == 0 ? 65536 : frame_limit;
  control->helper_lanes = helper_lanes; control->helper_limit = helper_limit;
  control->helper_depths = tb_device_offset(io_mem(tb_host_calloc(helper_lanes, sizeof(u32))));
  control->root_count = fid_result_width((Fid)term_aux(root));
  tb_device_adopt(root, 0, true);
}
extern "C" __global__ void tb_device_tasks_step(TBDeviceControl *control, u32 quantum) {
  tb_device_check_cancelled();
  if (control != tb_device_control || quantum == 0 || quantum > 4096)
    err_fail("invalid device dispatch boundary");
  if (control->done || control->adopt_head != 0) return;
  u64 offset = tb_device_dequeue(false);
  if (offset == 0) return;
  u64 helper_lane = (u64)blockIdx.x * blockDim.x + threadIdx.x;
  if (helper_lane >= control->helper_lanes) err_fail("invalid device helper lane");
  u32 *helper_depths = (u32 *)tb_device_pointer(control->helper_depths,
    (size_t)control->helper_lanes * sizeof(u32));
  if (helper_depths[helper_lane] != 0) err_fail("device helper frame retained across dispatch");
  u32 lane = (blockIdx.x * blockDim.x + threadIdx.x) & 255u;
  atomicOr(&control->lanes_seen[lane >> 6], UINT64_C(1) << (lane & 63));
  u32 active = atomicAdd(&control->active_lanes, 1u) + 1;
  atomicMax(&control->peak_lanes, active);
  Env e = {tb_memory, NULL};
  tb_device_advance(e, tb_device_run(offset), quantum);
  if (helper_depths[helper_lane] != 0) err_fail("device helper frame retained across dispatch");
  atomicSub(&control->active_lanes, 1u);
  tb_device_enqueue(offset, true);
}
extern "C" __global__ void tb_device_tasks_commit(TBDeviceControl *control, u32 quantum) {
  if (blockIdx.x != 0 || threadIdx.x != 0) return;
  tb_device_check_cancelled();
  if (control != tb_device_control || quantum == 0 || quantum > 4096 || control->active_lanes != 0)
    err_fail("invalid device commit boundary");
  ++control->launches;
  if (control->helper_live != 0) err_fail("device helper frame retained across dispatch");
  Env e = {tb_memory, NULL};
  for (u32 turn = 0; turn < quantum && !control->done; ++turn) {
    tb_tick();
    if (control->adopt_head != 0) { tb_device_adopt_step(e); continue; }
    u64 offset = tb_device_dequeue(true);
    if (offset == 0) break;
    TBDeviceRun *run = tb_device_run(offset);
    if (run->event == TB_DEVICE_RUN_READY) tb_device_enqueue(offset, false);
    else if (run->event == TB_DEVICE_RUN_WORDS) tb_device_finish(e, offset);
    else if (run->event == TB_DEVICE_RUN_GRAPH) {
      Term task = run->task; run->task = 0; tb_device_adopt(task, offset, false);
    } else err_fail("invalid device task event");
  }
  if (control->done) {
    if (control->live_runs != 0 || control->live_records != 0 || control->live_frames != 0
        || control->ready_count != 0 || control->completed_count != 0 || control->adopt_head != 0)
      err_fail("device root completed with pending work");
    tb_host_free(tb_device_pointer(control->helper_depths, 0));
    control->helper_depths = 0;
  } else if (control->ready_count == 0 && control->completed_count == 0 && control->adopt_head == 0)
    err_fail("incomplete task graph");
}
