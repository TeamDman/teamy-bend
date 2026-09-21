/* SPDX-License-Identifier: Apache-2.0
 * Task layout and delivery derived from Bend 2.0.5 comp.ts, Copyright 2026
 * HigherOrderCO. Bounded sequential executor: TeamDman. See NOTICE and
 * licenses/Apache-2.0.txt. Boxed closure callbacks and flat word segments
 * share the dispatcher without interpreting raw words as task controls. */
typedef struct TBTaskRun TBTaskRun;
typedef struct TBTaskContext TBTaskContext;
struct TBTaskRecord {
  TBTaskRecord *hash_next, *adopt_next, *parent;
  TBTaskContext *context;
  TBTaskRun *boundary;
  Term task;
  u32 arity, index, remaining, width;
  u64 reserved[4], delivered[4];
};
struct TBTaskRun {
  TBTaskRun *next;
  TBTaskRecord *target;
  TBCallFrame *current, *pending;
  Term closure, argument;
  TBOutcome result;
  Term *captures;
  u32 fid, state, expected, tail_result;
};
struct TBTaskContext {
  TBTaskContext *previous;
  Corpus memory;
  TBTaskRun *head, *tail;
  Term *root;
  Term *root_owned;
  u32 root_count, records;
  bool done;
};
static TBTaskContext *tb_task_current;
static void tb_task_context_reset(void) { tb_task_current = NULL; }
enum { TB_TASK_APPLY, TB_TASK_VALUE, TB_TASK_DIRECT };

/* A completed packet owns its words. Its metadata shares the same tracked
 * allocation, and moving the packet never duplicates the contained owners. */
OUTLINE TBOutcome tb_task_packet(const Term *words, const Term *owned, u32 count) {
  if (count > 255 || (count != 0 && words == NULL)) err_fail("invalid task result width");
  Term *storage = count == 0 ? NULL : (Term *)io_mem(tb_host_malloc(2 * count * sizeof(Term)));
  for (u32 i = 0; i < count; ++i) {
    if (owned != NULL && owned[i] > 1) err_fail("invalid task ownership mask");
    storage[i] = words[i]; storage[count + i] = owned == NULL ? 1 : owned[i];
  }
  return tb_segment_words(storage, count == 0 ? NULL : storage + count, count);
}
INLINE void tb_task_packet_free(TBOutcome packet) { tb_host_free((void *)packet.words); }

/* Unlike upstream's shared corpus header, this private root is delimited per
 * foreign reentry. Nested corpus_eval cannot overwrite an outer root result. */
INLINE bool root_done(Corpus memory) {
  if (tb_task_current == NULL || tb_task_current->memory != memory)
    err_fail("task root context is unavailable");
  return tb_task_current->done;
}
INLINE u32 root_take(Corpus memory, Term *values) {
  TBTaskContext *context = tb_task_current;
  if (!root_done(memory)) err_fail("task root result is missing");
  u32 count = context->root_count;
  if (count != 0) memcpy(values, context->root, count * sizeof(Term));
  tb_host_free(context->root); context->root = NULL; context->root_owned = NULL;
  context->root_count = 0; context->done = false;
  return count;
}
INLINE Term task_deliver(Corpus memory, Term continuation, u32 index, Term *values, u32 count) {
  if (memory != tb_memory || count > 255 || (count != 0 && values == NULL))
    err_fail("invalid task result width");
  if (continuation == TERM_HOLE) {
    TBTaskContext *context = tb_task_current;
    if (context == NULL || context->memory != memory || context->done)
      err_fail("invalid task root delivery");
    /* Upstream root delivery ignores index. Canonical task-node construction
     * separately requires a zero root index for this bounded executor. */
    TBOutcome packet = tb_task_packet(values, NULL, count);
    context->root = (Term *)packet.words; context->root_owned = (Term *)packet.owned;
    context->root_count = count; context->done = true;
    return 0;
  }
  Loc tail = task_tail(continuation), at = term_loc(continuation);
  u32 arity = fid_arity((Fid)term_aux(continuation));
  u32 remaining = (u32)memory[tail + 1];
  if (index > arity || count > arity - index || remaining == 0)
    err_fail("invalid task result destination");
  for (u32 i = 0; i < count; ++i)
    if (memory[at + index + i] != TERM_HOLE) err_fail("duplicate task result delivery");
  /* Slot ownership belongs to the receiving layout; do not reinterpret raw
   * W64 result bits or overwrite its exact per-cell ownership metadata. */
  for (u32 i = 0; i < count; ++i) memory[at + index + i] = values[i];
  memory[tail + 1] -= 1;
  return remaining == 1 ? continuation : 0;
}

INLINE u32 tb_task_hash(Loc at) {
  u64 mixed = at * UINT64_C(11400714819323198485);
  return (u32)((mixed ^ (mixed >> 32)) & 1023);
}
OUTLINE TBTaskRecord *tb_task_record(TBTaskContext *context, Term task) {
  Loc tail = task_tail(task), at = term_loc(task);
  u32 hash = tb_task_hash(at);
  TBTaskRecord *record;
  for (record = tb_task_table[hash]; record != NULL; record = record->hash_next)
    if (term_loc(record->task) == at) err_fail("duplicate or cyclic task graph");
  if (tb_tasks == UINT32_MAX || tb_tasks >= BEND_MAX_TASKS) err_fail("task budget exhausted");
  record = (TBTaskRecord *)io_mem(tb_host_calloc(1, sizeof(*record)));
  record->task = task; record->context = context;
  record->arity = fid_arity((Fid)term_aux(task));
  record->width = fid_result_width((Fid)term_aux(task));
  record->remaining = (u32)context->memory[tail + 1];
  record->index = (u32)(context->memory[tail + 1] >> 32);
  if (record->remaining > record->arity) err_fail("invalid task dependency count");
  record->hash_next = tb_task_table[hash]; tb_task_table[hash] = record;
  ++tb_tasks; ++context->records;
  if (tb_tasks > tb_task_peak) tb_task_peak = tb_tasks;
  if (record->remaining != 0) {
    if (tb_task_joins == UINT64_MAX) err_fail("task join counter exhausted");
    ++tb_task_joins;
  }
  return record;
}
OUTLINE void tb_task_unindex(TBTaskRecord *record) {
  TBTaskRecord **link = &tb_task_table[tb_task_hash(term_loc(record->task))];
  while (*link != NULL && *link != record) link = &(*link)->hash_next;
  if (*link != record) err_fail("task ownership registry mismatch");
  *link = record->hash_next; record->hash_next = NULL;
}
OUTLINE void tb_task_record_free(TBTaskRecord *record) {
  if (tb_tasks == 0 || record->context->records == 0) err_fail("unbalanced task ownership");
  --tb_tasks; --record->context->records;
  tb_host_free(record);
}
INLINE void tb_task_enqueue(TBTaskContext *context, TBTaskRun *run) {
  run->next = NULL;
  if (context->tail != NULL) context->tail->next = run; else context->head = run;
  context->tail = run;
}
INLINE bool tb_task_bit(const u64 *bits, u32 index) {
  return (bits[index >> 6] & (UINT64_C(1) << (index & 63))) != 0;
}
INLINE void tb_task_set(u64 *bits, u32 index) {
  bits[index >> 6] |= UINT64_C(1) << (index & 63);
}
OUTLINE void tb_task_payload_ready(Env e, Loc at, u32 arity) {
  for (u32 i = 0; i < arity; ++i) {
    if (!tb_cell_owned(e, at + i)) continue;
    if (e.mem[at + i] == TERM_HOLE) err_fail("foreign task argument is missing");
    if (term_tag(e.mem[at + i]) == TAG_TSK) err_fail("runnable task contains a pending task");
  }
}
OUTLINE void tb_task_arguments(TBTaskRecord *record) {
  Env e = {record->context->memory, NULL};
  tb_task_payload_ready(e, term_loc(record->task), record->arity);
}
OUTLINE void tb_task_segment_start(Env e, Fid fid, Loc at, TBTaskRun *run) {
  u32 count = fid_arity(fid);
  Term *payload = count == 0 ? NULL : (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
  if (count != 0) memcpy(payload, e.mem + at, count * sizeof(Term));
  run->current = tb_call_frame_owned(fid, payload, 0);
  if (tb_segment_calls == UINT64_MAX) err_fail("segment call counter exhausted");
  ++tb_segment_calls;
}
/* Starting a node transfers all boxed arguments into a run before releasing
 * its shell. Unindex first: the allocator may immediately reuse that address. */
OUTLINE void tb_task_start(Env e, TBTaskRecord *record) {
  Loc at = term_loc(record->task);
  u32 fid = (u32)term_aux(record->task);
  TBTaskRun *run;
  if (record->remaining != 0) err_fail("task started before its dependencies");
  tb_task_arguments(record);
  run = (TBTaskRun *)io_mem(tb_host_calloc(1, sizeof(*run)));
  run->target = record; run->expected = record->width;
  if (tb_segment_functions[fid] != NULL) {
    tb_task_segment_start(e, fid, at, run);
  } else if (fid == FID_CLO_APPLY) {
    run->closure = e.mem[at]; run->argument = e.mem[at + 1];
  } else if (fid == FID_IO_EMIT) {
    run->closure = term_clo(FID_IO_EMIT, 0); run->argument = e.mem[at];
  } else {
    u32 count = record->arity - 1;
    run->fid = fid; run->state = TB_TASK_DIRECT; run->argument = e.mem[at + count];
    run->captures = count == 0 ? NULL : (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
    if (count != 0) memcpy(run->captures, e.mem + at, count * sizeof(Term));
  }
  tb_task_unindex(record);
  heap_free(e, cls_fit(record->arity + 2), at);
  tb_task_enqueue(record->context, run);
}
OUTLINE void tb_task_root(TBTaskRecord *record, TBTaskRun *boundary) {
  Corpus memory = record->context->memory;
  if (memory[task_tail(record->task)] != TERM_HOLE || record->index != 0)
    err_fail("foreign task continuation is unsupported");
  if (boundary != NULL && boundary->expected != record->width) err_fail("task result width mismatch");
  record->boundary = boundary;
}
/* Adoption is a separate, iterative validation pass. No callback runs and no
 * child is detached until every edge and the dependency counts are checked. */
OUTLINE void tb_task_adopt(Env e, TBTaskContext *context, Term task, TBTaskRun *boundary, bool entry) {
  TBTaskRecord *head = tb_task_record(context, task), *last = head;
  Loc tail = task_tail(task);
  if (entry && head->remaining != 0) err_fail("corpus_eval requires a ready task");
  if (e.mem[tail] != TERM_HOLE) {
    /* A standalone ready child may own a closed chain of one-child
     * continuations. Siblings scheduled elsewhere are not guessed or run. */
    TBTaskRecord *child = head;
    if (head->remaining != 0) err_fail("incomplete external task graph");
    tb_task_arguments(head);
    while (e.mem[task_tail(child->task)] != TERM_HOLE) {
      Term parent_term = e.mem[task_tail(child->task)];
      TBTaskRecord *parent = tb_task_record(context, parent_term);
      Loc at = term_loc(parent_term);
      tb_tick();
      if (parent->remaining != 1 || child->index > parent->arity
          || child->width > parent->arity - child->index)
        err_fail("incomplete external task graph");
      for (u32 i = 0; i < parent->arity; ++i) {
        bool destination = i >= child->index && i - child->index < child->width;
        if (destination) {
          if (e.mem[at + i] != TERM_HOLE) err_fail("incomplete external task graph");
          tb_task_set(parent->reserved, i);
        } else if (tb_cell_owned(e, at + i)
            && (e.mem[at + i] == TERM_HOLE || term_tag(e.mem[at + i]) == TAG_TSK))
          err_fail("incomplete external task graph");
      }
      child->parent = parent;
      last->adopt_next = parent; last = parent; child = parent;
    }
    tb_task_root(last, boundary);
    tb_task_start(e, head);
    return;
  }
  tb_task_root(head, boundary);
  for (TBTaskRecord *record = head; record != NULL; record = record->adopt_next) {
    Loc at = term_loc(record->task);
    u32 children = 0;
    tb_tick();
    if (record->remaining == 0) { tb_task_arguments(record); continue; }
    for (u32 i = 0; i < record->arity; ++i) {
      if (tb_task_bit(record->reserved, i) || !tb_cell_owned(e, at + i)) continue;
      Term value = e.mem[at + i];
      if (value == TERM_HOLE) err_fail("foreign task argument is missing");
      if (term_tag(value) == TAG_TSK) {
        TBTaskRecord *child = tb_task_record(context, value);
        if (e.mem[task_tail(value)] != record->task || child->index != i)
          err_fail("task child destination mismatch");
        if (child->width > record->arity - i) err_fail("task result span is out of bounds");
        for (u32 j = 0; j < child->width; ++j) {
          if (tb_task_bit(record->reserved, i + j)
              || (j != 0 && e.mem[at + i + j] != TERM_HOLE))
            err_fail("task result spans overlap");
          tb_task_set(record->reserved, i + j);
        }
        child->parent = record;
        last->adopt_next = child; last = child; ++children;
      }
    }
    if (children != record->remaining) err_fail("task dependency count mismatch");
  }
  /* The adoption list owns record pointers, so parent slots can now become
   * empty result destinations without losing any queued child. */
  for (TBTaskRecord *record = head; record != NULL; record = record->adopt_next) {
    Loc at = term_loc(record->task);
    for (u32 i = 0; i < record->arity; ++i)
      if (tb_task_bit(record->reserved, i)) e.mem[at + i] = TERM_HOLE;
  }
  for (TBTaskRecord *record = head; record != NULL; record = record->adopt_next)
    if (record->remaining == 0) tb_task_start(e, record);
}
OUTLINE void tb_task_finish(Env e, TBTaskContext *context, TBTaskRun *run, TBOutcome packet) {
  TBTaskRecord *record = run->target;
  if (packet.count != run->expected || (record != NULL && packet.count != record->width))
    err_fail("task result width mismatch");
  if (record == NULL) {
    (void)task_deliver(e.mem, TERM_HOLE, 0, (Term *)packet.words, packet.count);
    memcpy(context->root_owned, packet.owned, packet.count * sizeof(Term));
  } else if (record->parent != NULL) {
    TBTaskRecord *parent = record->parent;
    u32 index = record->index;
    if (index > parent->arity || packet.count > parent->arity - index || parent->remaining == 0
        || (u32)e.mem[task_tail(parent->task) + 1] != parent->remaining)
      err_fail("duplicate task result delivery");
    for (u32 i = 0; i < packet.count; ++i)
      if (!tb_task_bit(parent->reserved, index + i) || tb_task_bit(parent->delivered, index + i))
        err_fail("duplicate task result delivery");
    Term ready = task_deliver(e.mem, parent->task, index, (Term *)packet.words, packet.count);
    for (u32 i = 0; i < packet.count; ++i) {
      Loc cell = term_loc(parent->task) + index + i;
      if (packet.owned[i] != 0) tb_mark_owned(e, cell, 1); else tb_mark_raw(e, cell, 1);
      tb_task_set(parent->delivered, index + i);
    }
    --parent->remaining;
    if (ready != 0) tb_task_start(e, parent);
  } else if (record->boundary != NULL) {
    if (packet.count != record->boundary->expected) err_fail("task result width mismatch");
    record->boundary->result = packet; record->boundary->state = TB_TASK_VALUE;
    tb_task_enqueue(context, record->boundary);
    packet.words = NULL;
  } else {
    (void)task_deliver(e.mem, TERM_HOLE, 0, (Term *)packet.words, packet.count);
    memcpy(context->root_owned, packet.owned, packet.count * sizeof(Term));
  }
  tb_task_packet_free(packet);
  if (record != NULL) tb_task_record_free(record);
  tb_host_free(run);
}
/* Task controls and private generated continuations share one iterative
 * dispatcher. A fork parks its existing run behind a private root boundary;
 * each child has an independent pending-frame stack and completion target. */
OUTLINE TBOutcome tb_task_execute(Env e, Term closure, Term argument, Term input, bool task_entry) {
  TBTaskContext *context;
  TBOutcome answer;
  if (++tb_depth > BEND_MAX_DEPTH) err_fail("call depth budget exhausted");
  context = (TBTaskContext *)io_mem(tb_host_calloc(1, sizeof(*context)));
  context->memory = e.mem; context->previous = tb_task_current; tb_task_current = context;
  if (task_entry) tb_task_adopt(e, context, input, NULL, true);
  else {
    TBTaskRun *run = (TBTaskRun *)io_mem(tb_host_calloc(1, sizeof(*run)));
    run->closure = closure; run->argument = argument; run->expected = 1;
    tb_task_enqueue(context, run);
  }
  while (context->head != NULL) {
    TBTaskRun *run = context->head;
    context->head = run->next;
    if (context->head == NULL) context->tail = NULL;
    for (;;) {
      TBOutcome outcome;
      tb_tick();
      if (run->state == TB_TASK_VALUE) {
        outcome = run->result; run->result.words = NULL; run->state = TB_TASK_APPLY;
      } else if (run->current != NULL) {
        TBCallFrame *current = run->current;
        if (current->waiting || current->fid >= 65536
            || (tb_resume_functions[current->fid] == NULL && tb_segment_functions[current->fid] == NULL))
          err_fail("invalid generated continuation state");
        if (tb_segment_functions[current->fid] != NULL) {
          outcome = tb_segment_functions[current->fid](&e, current);
          if (!outcome.pending) {
            if (outcome.count != fid_result_width(current->fid)) err_fail("task result width mismatch");
            if (tb_segment_result_words > UINT64_MAX - outcome.count) err_fail("segment result counter exhausted");
            tb_segment_result_words += outcome.count;
            outcome = tb_task_packet(outcome.words, outcome.owned, outcome.count);
            if (outcome.count > 1) {
              if (tb_segment_multiword_results == UINT64_MAX) err_fail("segment result counter exhausted");
              ++tb_segment_multiword_results;
            }
          }
        } else {
          Term result = tb_resume_functions[current->fid](&e, current);
          outcome = term_tag(result) == TAG_TSK ? tb_segment_task(result) : tb_task_packet(&result, NULL, 1);
        }
        if (current->tail_result != TB_RESULT_NONE) {
          if (current->waiting || !outcome.pending || fid_result_width(current->fid) != 1)
            err_fail("invalid scalar tail result adapter");
          run->tail_result = tb_result_compose(run->tail_result, current->tail_result);
        }
        if (current->waiting) {
          if (current->pc == 0 || current->expected == 0 || current->expected > 255
              || current->destination > current->slots || current->expected > current->slots - current->destination)
            err_fail("invalid generated continuation destination");
          if (!outcome.pending || term_tag(outcome.task) != TAG_TSK) err_fail("foreign task is unsupported");
          run->expected = current->expected;
          current->saved_result = run->tail_result; run->tail_result = TB_RESULT_NONE;
          current->parent = run->pending; run->pending = current; run->current = NULL;
        } else { tb_call_frame_free(current); run->current = NULL; }
      } else {
        u32 fid, count;
        Term result;
        Term *captures = NULL;
        if (run->state == TB_TASK_DIRECT) {
          fid = run->fid; captures = run->captures; run->captures = NULL;
          run->state = TB_TASK_APPLY;
          if (tb_resume_functions[fid] != NULL) {
            run->current = tb_call_frame_owned(fid, captures, run->argument); continue;
          }
        } else {
          if (run->expected != 1) err_fail("task result width mismatch");
          if (term_tag(run->closure) != TAG_CLO) err_fail("application of a non-function");
          if (term_rfc(run->closure)) err_fail("reference-counted closure application is unsupported");
          fid = (u32)term_aux(run->closure);
          if (fid != FID_IO_EMIT) {
            if (tb_closure_functions[fid] == NULL) err_fail("unregistered closure application");
            if (tb_resume_functions[fid] != NULL) {
              run->current = tb_call_frame(e, run->closure, run->argument); continue;
            }
            count = tb_closure_captures[fid];
            if (count != 0) {
              Loc at = term_peek(e, run->closure); tb_allocation(e, at, cls_fit(count));
              captures = (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
              memcpy(captures, e.mem + at, count * sizeof(Term));
              heap_free(e, cls_fit(count), at);
            }
          }
        }
        if (fid == FID_IO_EMIT) {
          if (cid_arity(CID_EMIT) != 1) err_fail("Emit representation unavailable");
          Loc at = heap_alloc(e, 0); e.mem[at] = run->argument; result = term_ctr(CID_EMIT, at);
        } else result = tb_closure_functions[fid](e, captures, run->argument);
        tb_host_free(captures);
        outcome = term_tag(result) == TAG_TSK ? tb_segment_task(result) : tb_task_packet(&result, NULL, 1);
      }
      if (outcome.pending) {
        Term task = outcome.task;
        Loc tail = task_tail(task);
        Fid fid = (Fid)term_aux(task);
        if (tb_segment_functions[fid] != NULL && e.mem[tail] == TERM_HOLE && e.mem[tail + 1] == 0) {
          Loc at = term_loc(task);
          u32 arity = fid_arity(fid);
          if (run->expected != fid_result_width(fid)) err_fail("task result width mismatch");
          tb_task_payload_ready(e, at, arity);
          /* Ready word calls reuse this run. A tail jump cannot accumulate
           * parked root boundaries; a suspended caller stays in pending. */
          tb_task_segment_start(e, fid, at, run);
          heap_free(e, cls_fit(arity + 2), at);
          continue;
        }
        if (term_aux(task) == FID_CLO_APPLY && e.mem[tail] == TERM_HOLE && e.mem[tail + 1] == 0) {
          if (run->expected != 1) err_fail("task result width mismatch");
          tb_task_take(e, task, &run->closure, &run->argument); continue;
        }
        tb_task_adopt(e, context, task, run, false);
        break;
      }
      if (outcome.count != run->expected) err_fail("task result width mismatch");
      if (run->tail_result != TB_RESULT_NONE) {
        if (outcome.count != 1) err_fail("invalid scalar tail result adapter");
        /* Only completed packets are adapted. The run retains its adapter
         * while a returned graph executes behind its private boundary. */
        u32 mode = tb_result_compose(TB_RESULT_NONE, run->tail_result);
        if ((mode & 4u) != 0) ((Term *)outcome.words)[0] = (u32)outcome.words[0];
        ((Term *)outcome.owned)[0] = (mode & 2u) != 0;
        run->tail_result = TB_RESULT_NONE;
      }
      if (run->pending != NULL) {
        TBCallFrame *current = run->pending;
        run->pending = current->parent; current->parent = NULL;
        if (!current->waiting || current->pc == 0 || current->expected != outcome.count
            || current->destination > current->slots || outcome.count > current->slots - current->destination)
          err_fail("invalid generated continuation destination");
        memcpy(current->values + current->destination, outcome.words, outcome.count * sizeof(Term));
        current->waiting = false; run->expected = fid_result_width(current->fid);
        run->tail_result = current->saved_result; current->saved_result = TB_RESULT_NONE;
        tb_task_packet_free(outcome);
        run->current = current; continue;
      }
      tb_task_finish(e, context, run, outcome);
      break;
    }
  }
  if (!context->done || context->records != 0) err_fail("incomplete task graph");
  answer = tb_segment_words(context->root, context->root_owned, context->root_count);
  context->root = NULL; context->root_owned = NULL;
  tb_task_current = context->previous; tb_host_free(context); --tb_depth;
  return answer;
}
OUTLINE Term tb_apply(Env e, Term closure, Term argument) {
  TBOutcome packet = tb_task_execute(e, closure, argument, 0, false);
  if (packet.count != 1) err_fail("boxed task result width is unsupported");
  Term result = packet.words[0]; tb_task_packet_free(packet); return result;
}
OUTLINE Term corpus_eval(Corpus memory, Term task) {
  Env e = {memory, NULL};
  TBOutcome packet = tb_task_execute(e, 0, 0, task, true);
  if (packet.count != 1) err_fail("boxed task result width is unsupported");
  Term result = packet.words[0]; tb_task_packet_free(packet); return result;
}
OUTLINE u32 corpus_eval_words(Corpus memory, Term task, Term *words, Term *owned, u32 capacity) {
  Env e = {memory, NULL};
  TBOutcome packet = tb_task_execute(e, 0, 0, task, true);
  if (packet.count > capacity || (packet.count != 0 && (words == NULL || owned == NULL)))
    err_fail("task result buffer is too small");
  if (packet.count != 0) {
    memcpy(words, packet.words, packet.count * sizeof(Term));
    memcpy(owned, packet.owned, packet.count * sizeof(Term));
  }
  u32 count = packet.count; tb_task_packet_free(packet); return count;
}
