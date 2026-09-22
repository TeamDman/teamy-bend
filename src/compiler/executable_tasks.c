/* SPDX-License-Identifier: Apache-2.0
 * Task layout and delivery derived from Bend 2.0.5 comp.ts, Copyright 2026
 * HigherOrderCO. Bounded CPU executor: TeamDman. See NOTICE and
 * licenses/Apache-2.0.txt. Boxed closure callbacks and flat word segments
 * share the dispatcher without interpreting raw words as task controls. */
typedef struct TBTaskRun TBTaskRun;
#ifdef TB_GPU_ENABLED
static bool tb_gpu_cpu_only(void);
#endif
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
  u32 fid, state, expected, tail_result, event;
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
static TB_THREAD_LOCAL TBTaskContext *tb_task_current;
static void tb_task_context_reset(void) { tb_task_current = NULL; }
enum { TB_TASK_APPLY, TB_TASK_VALUE, TB_TASK_DIRECT, TB_TASK_GPU };
enum { TB_CPU_FINISH, TB_CPU_GRAPH, TB_CPU_COORDINATOR };
#ifndef TB_CPU_RESUME_ENTER
#define TB_CPU_RESUME_ENTER(fid, frame) ((void)0)
#endif
#ifndef TB_CPU_RESUME_LEAVE
#define TB_CPU_RESUME_LEAVE(fid, frame) ((void)0)
#endif
#ifndef TB_DIRECT_CALL
#define TB_DIRECT_CALL(fid, reused) ((void)0)
#endif

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
  tb_counter_add(&tb_segment_calls, 1, "segment call counter exhausted");
}
OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {
  Loc at = term_loc(task);
  u32 fid = (u32)term_aux(task), arity = fid_arity(fid);
  run->state = TB_TASK_APPLY;
  if (tb_segment_functions[fid] != NULL) {
    tb_task_segment_start(e, fid, at, run);
  } else if (fid == FID_CLO_APPLY) {
    run->closure = e.mem[at]; run->argument = e.mem[at + 1];
  } else if (fid == FID_IO_EMIT) {
    run->closure = term_clo(FID_IO_EMIT, 0); run->argument = e.mem[at];
  } else {
    u32 count = arity - 1;
    run->fid = fid; run->state = TB_TASK_DIRECT; run->argument = e.mem[at + count];
    run->captures = count == 0 ? NULL : (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
    if (count != 0) memcpy(run->captures, e.mem + at, count * sizeof(Term));
  }
  heap_free(e, cls_fit(arity + 2), at);
}
/* Starting a node transfers all boxed arguments into a run before releasing
 * its shell. Unindex first: the allocator may immediately reuse that address.
 * A CUDA root retains its shell, but its host destination remains parked in
 * the record; no device link ever points into a host continuation. */
OUTLINE void tb_task_start(Env e, TBTaskRecord *record) {
  TBTaskRun *run;
  if (record->remaining != 0) err_fail("task started before its dependencies");
  tb_task_arguments(record);
  run = (TBTaskRun *)io_mem(tb_host_calloc(1, sizeof(*run)));
  run->target = record; run->expected = record->width;
  tb_task_unindex(record);
#ifdef TB_GPU_ENABLED
  if (tb_gpu_marked((Fid)term_aux(record->task)) && !tb_gpu_cpu_only()) {
    Loc tail = task_tail(record->task);
    e.mem[tail] = TERM_HOLE; e.mem[tail + 1] = 0;
    run->state = TB_TASK_GPU; run->result = tb_segment_task(record->task);
  } else
#endif
  tb_task_start_cpu(e, record->task, run);
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
/* Only the compiler opts generated entries into worker execution. Public
 * foreign registration alone never certifies a callback as parallel-safe. */
INLINE bool tb_task_worker_safe(const TBTaskRun *run) {
  u32 fid;
  if (run->state == TB_TASK_GPU) return false;
  if (run->state == TB_TASK_VALUE) return true;
  if (run->current != NULL) fid = run->current->fid;
  else if (run->state == TB_TASK_DIRECT) fid = run->fid;
  else {
    if (term_tag(run->closure) != TAG_CLO) return false;
    fid = (u32)term_aux(run->closure);
    if (fid == FID_IO_EMIT) return true;
  }
  return fid < 65536 && tb_parallel_functions[fid];
}
/* Validate the full plain ID before narrowing it. A direct call's words and
 * mask can alias the source frame, so complete validation before moving or
 * releasing any of that storage. Raw words retain all bits, including values
 * that would look like holes or tasks if treated as boxed Terms. */
OUTLINE Fid tb_task_call_validate(TBOutcome outcome, u32 expected) {
  if (outcome.task >= 65536 || tb_segment_functions[(u32)outcome.task] == NULL)
    err_fail("invalid direct segment call");
  Fid fid = (Fid)outcome.task;
  if (outcome.count != tb_segment_arities[fid] || (outcome.count != 0 && outcome.words == NULL))
    err_fail("invalid direct segment arguments");
  if (expected != tb_segment_widths[fid]) err_fail("task result width mismatch");
  for (u32 i = 0; i < outcome.count; ++i) {
    if (outcome.owned != NULL && outcome.owned[i] > 1) err_fail("invalid task ownership mask");
    if (outcome.owned != NULL && outcome.owned[i] == 0) continue;
    if (outcome.words[i] == TERM_HOLE) err_fail("foreign task argument is missing");
    if (term_tag(outcome.words[i]) == TAG_TSK) err_fail("runnable task contains a pending task");
  }
  return fid;
}
/* Advance one exclusively owned run. Graph adoption and result delivery are
 * coordinator operations; neither happens inside a worker. Recheck safety at
 * every call boundary, including dynamic closures and ready tail calls. */
OUTLINE u32 tb_task_advance(Env e, TBTaskRun *run, bool worker) {
    for (;;) {
      TBOutcome outcome;
      if (worker && !tb_task_worker_safe(run)) return TB_CPU_COORDINATOR;
      tb_tick();
#ifdef TB_GPU_ENABLED
      if (run->state == TB_TASK_GPU) {
        Term root = run->result.task;
        outcome = tb_gpu_execute(e, root);
        run->state = TB_TASK_APPLY;
        if (outcome.pending == TB_OUTCOME_TASK) {
          if (outcome.task != root) err_fail("invalid GPU fallback root");
          tb_task_start_cpu(e, root, run);
          continue;
        }
      } else
#endif
      if (run->state == TB_TASK_VALUE) {
        outcome = run->result; run->result.words = NULL; run->state = TB_TASK_APPLY;
      } else if (run->current != NULL) {
        TBCallFrame *current = run->current;
        if (current->waiting || current->fid >= 65536
            || (tb_resume_functions[current->fid] == NULL && tb_segment_functions[current->fid] == NULL))
          err_fail("invalid generated continuation state");
        if (tb_segment_functions[current->fid] != NULL) {
          if (worker) TB_CPU_RESUME_ENTER(current->fid, current);
          outcome = tb_segment_functions[current->fid](&e, current);
          if (worker) TB_CPU_RESUME_LEAVE(current->fid, current);
          if (outcome.pending > TB_OUTCOME_CALL) err_fail("invalid segment outcome tag");
          if (outcome.pending == TB_OUTCOME_WORDS) {
            if (outcome.count != fid_result_width(current->fid)) err_fail("task result width mismatch");
            tb_counter_add(&tb_segment_result_words, outcome.count, "segment result counter exhausted");
            outcome = tb_task_packet(outcome.words, outcome.owned, outcome.count);
            if (outcome.count > 1) {
              tb_counter_add(&tb_segment_multiword_results, 1, "segment result counter exhausted");
            }
          }
        } else {
          if (worker) TB_CPU_RESUME_ENTER(current->fid, current);
          Term result = tb_resume_functions[current->fid](&e, current);
          if (worker) TB_CPU_RESUME_LEAVE(current->fid, current);
          outcome = term_tag(result) == TAG_TSK ? tb_segment_task(result) : tb_task_packet(&result, NULL, 1);
        }
        if (current->tail_result != TB_RESULT_NONE) {
          if (current->waiting || outcome.pending == TB_OUTCOME_WORDS || fid_result_width(current->fid) != 1)
            err_fail("invalid scalar tail result adapter");
          run->tail_result = tb_result_compose(run->tail_result, current->tail_result);
        }
        if (current->waiting) {
          if (current->pc == 0 || current->expected == 0 || current->expected > 255
              || current->destination > current->slots || current->expected > current->slots - current->destination)
            err_fail("invalid generated continuation destination");
          if (outcome.pending != TB_OUTCOME_CALL
              && (outcome.pending != TB_OUTCOME_TASK || term_tag(outcome.task) != TAG_TSK))
            err_fail("foreign task is unsupported");
        }
        if (outcome.pending == TB_OUTCOME_CALL) {
          Fid fid = tb_task_call_validate(outcome, current->waiting ? current->expected : run->expected);
          bool reuse = !current->waiting && fid == current->fid;
          Term *captures;
          if (reuse) {
            /* Both input spans may point inside captures. Copy first, then
             * clear scratch and control fields without inspecting stale owners. */
            captures = (Term *)current->captures;
            if (outcome.count != 0) memmove(captures, outcome.words, outcome.count * sizeof(Term));
            if (current->slots != 0) memset(current->values, 0, current->slots * sizeof(Term));
            current->pc = 0; current->destination = 0; current->expected = 1;
            current->tail_result = TB_RESULT_NONE; current->argument = 0;
            current->saved_result = TB_RESULT_NONE; current->parent = NULL;
          } else {
            captures = outcome.count == 0 ? NULL : (Term *)io_mem(tb_host_malloc(outcome.count * sizeof(Term)));
            if (outcome.count != 0) memcpy(captures, outcome.words, outcome.count * sizeof(Term));
            if (current->waiting) {
              run->expected = current->expected;
              current->saved_result = run->tail_result; run->tail_result = TB_RESULT_NONE;
              current->parent = run->pending; run->pending = current;
            } else tb_call_frame_free(current);
            run->current = tb_call_frame_owned(fid, captures, 0);
          }
          tb_counter_add(&tb_segment_calls, 1, "segment call counter exhausted");
          TB_DIRECT_CALL(fid, reuse);
          continue;
        }
        if (current->waiting) {
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
      if (outcome.pending != TB_OUTCOME_WORDS && outcome.pending != TB_OUTCOME_TASK)
        err_fail("invalid segment outcome tag");
      if (outcome.pending == TB_OUTCOME_TASK) {
        Term task = outcome.task;
        Loc tail = task_tail(task);
        Fid fid = (Fid)term_aux(task);
        if (tb_segment_functions[fid] != NULL && e.mem[tail] == TERM_HOLE && e.mem[tail + 1] == 0
#ifdef TB_GPU_ENABLED
            && (!tb_gpu_marked(fid) || tb_gpu_cpu_only())
#endif
        ) {
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
        run->result = outcome;
        return TB_CPU_GRAPH;
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
      run->result = outcome;
      return TB_CPU_FINISH;
    }
}
/* The persistent CPU pool moves uniquely owned runs through mutex-published
 * queues. Only the coordinator changes graph records or starts foreign code.
 * Threads are created lazily to match the ready frontier, up to the selected
 * CPU count. A single worker selection uses the coordinator without a pool. */
#ifndef BEND_CPU_WORKERS
#define BEND_CPU_WORKERS 0u
#endif
#ifndef BEND_CPU_TEST_SPAWN_FAIL_AFTER
#define BEND_CPU_TEST_SPAWN_FAIL_AFTER UINT32_MAX
#endif
#define TB_CPU_LIMIT 128u
typedef struct {
  TBHost *host;
  TBTaskRun *head, *tail, *completed_head, *completed_tail;
  u32 desired, created;
  bool stop, failed;
  char error[256];
#ifdef _WIN32
  HANDLE threads[TB_CPU_LIMIT];
#else
  pthread_t threads[TB_CPU_LIMIT];
#endif
} TBCpuPool;
static TBCpuPool tb_cpu;
static TBMutex tb_cpu_mutex = TB_MUTEX_INIT;
static u32 tb_cpu_pending; /* Coordinator-owned; workers publish only queues. */
static u32 tb_cpu_live_workers; /* Protected by tb_cpu_mutex until joined. */
static TB_NORETURN void tb_cpu_cancel_error(void) {
  char error[256];
  if (tb_worker_failure_capture) err_fail("CPU execution cancelled");
  tb_lock(&tb_cpu_mutex);
  (void)snprintf(error, sizeof(error), "%s", tb_cpu.failed ? tb_cpu.error : "CPU execution cancelled");
  tb_unlock(&tb_cpu_mutex);
  err_fail(error);
}
#ifdef _WIN32
static CONDITION_VARIABLE tb_cpu_changed = CONDITION_VARIABLE_INIT;
INLINE bool tb_cpu_wait_locked(void) {
  return SleepConditionVariableSRW(&tb_cpu_changed, &tb_cpu_mutex, INFINITE, 0) != 0;
}
INLINE void tb_cpu_notify_locked(void) { WakeAllConditionVariable(&tb_cpu_changed); }
#else
static pthread_cond_t tb_cpu_changed = PTHREAD_COND_INITIALIZER;
INLINE bool tb_cpu_wait_locked(void) {
  return pthread_cond_wait(&tb_cpu_changed, &tb_cpu_mutex) == 0;
}
INLINE void tb_cpu_notify_locked(void) { (void)pthread_cond_broadcast(&tb_cpu_changed); }
#endif
INLINE void tb_cpu_set_active(bool active) {
  tb_vm_acquire(); tb_cpu_active = active; tb_vm_release();
}
static void tb_cpu_initialize(void) {
  u32 desired = BEND_CPU_WORKERS;
  if (tb_cpu.created != 0 || tb_cpu_live_workers != 0) err_fail("CPU pool is already running");
  memset(&tb_cpu, 0, sizeof(tb_cpu)); tb_cpu_pending = 0;
  tb_cpu.host = tb_host_current;
  if (desired == 0) {
#ifdef _WIN32
    desired = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
#else
    long available = sysconf(_SC_NPROCESSORS_ONLN);
    desired = available < 1 ? 1 : available > TB_CPU_LIMIT ? TB_CPU_LIMIT : (u32)available;
#endif
    if (desired == 0) desired = 1;
    if (desired > TB_CPU_LIMIT) desired = TB_CPU_LIMIT;
  }
  if (desired > TB_CPU_LIMIT) err_fail("CPU worker count exceeds 128");
  tb_cpu.desired = desired;
  tb_cpu_set_active(false);
}
#ifdef _WIN32
static unsigned __stdcall tb_cpu_worker(void *opaque)
#else
static void *tb_cpu_worker(void *opaque)
#endif
{
  TBHost *host = (TBHost *)opaque;
  jmp_buf guard;
  tb_host_current = host; tb_failure_guard = &guard;
  tb_worker_failure_capture = true; tb_failure_message[0] = '\0';
  tb_lock(&tb_cpu_mutex); ++tb_cpu_live_workers; tb_unlock(&tb_cpu_mutex);
  if (setjmp(guard) == 0) {
    for (;;) {
      TBTaskRun *run;
      tb_lock(&tb_cpu_mutex);
      while (tb_cpu.head == NULL && !tb_cpu.stop) {
        if (!tb_cpu_wait_locked()) {
          tb_unlock(&tb_cpu_mutex); err_fail("CPU worker wait failed");
        }
      }
      if (tb_cpu.stop) { tb_unlock(&tb_cpu_mutex); break; }
      run = tb_cpu.head; tb_cpu.head = run->next;
      if (tb_cpu.head == NULL) tb_cpu.tail = NULL;
      tb_unlock(&tb_cpu_mutex);
      Env e = {tb_memory, NULL};
      run->event = tb_task_advance(e, run, true);
      run->next = NULL;
      tb_lock(&tb_cpu_mutex);
      if (tb_cpu.completed_tail != NULL) tb_cpu.completed_tail->next = run;
      else tb_cpu.completed_head = run;
      tb_cpu.completed_tail = run;
      tb_cpu_notify_locked(); tb_unlock(&tb_cpu_mutex);
    }
  } else {
    tb_lock(&tb_cpu_mutex);
    if (!tb_cpu.failed) {
      tb_cpu.failed = true;
      (void)snprintf(tb_cpu.error, sizeof(tb_cpu.error), "%s", tb_failure_message);
    }
    tb_cpu.stop = true; tb_cpu_notify_locked(); tb_unlock(&tb_cpu_mutex);
    tb_vm_cancel();
  }
  tb_lock(&tb_cpu_mutex); --tb_cpu_live_workers;
  tb_cpu_notify_locked(); tb_unlock(&tb_cpu_mutex);
  tb_failure_guard = NULL; tb_host_current = NULL; tb_worker_failure_capture = false;
#ifdef _WIN32
  return 0;
#else
  return NULL;
#endif
}
OUTLINE void tb_cpu_grow(u32 frontier) {
  if (frontier > tb_cpu.desired) frontier = tb_cpu.desired;
  while (tb_cpu.created < frontier) {
    if (tb_cpu.created >= BEND_CPU_TEST_SPAWN_FAIL_AFTER) err_fail("CPU worker creation failed");
#ifdef _WIN32
    uintptr_t thread = _beginthreadex(NULL, 0, tb_cpu_worker, tb_cpu.host, 0, NULL);
    if (thread == 0) err_fail("CPU worker creation failed");
    tb_cpu.threads[tb_cpu.created] = (HANDLE)thread;
#else
    if (pthread_create(&tb_cpu.threads[tb_cpu.created], NULL, tb_cpu_worker, tb_cpu.host) != 0)
      err_fail("CPU worker creation failed");
#endif
    ++tb_cpu.created;
  }
}
OUTLINE void tb_cpu_submit(TBTaskRun *run, u32 frontier) {
  tb_cpu_grow(frontier);
  if (tb_cpu_pending == UINT32_MAX) err_fail("CPU pending count exhausted");
  ++tb_cpu_pending; tb_cpu_set_active(true);
  run->next = NULL;
  tb_lock(&tb_cpu_mutex);
  if (tb_cpu.tail != NULL) tb_cpu.tail->next = run; else tb_cpu.head = run;
  tb_cpu.tail = run;
  tb_cpu_notify_locked(); tb_unlock(&tb_cpu_mutex);
}
OUTLINE TBTaskRun *tb_cpu_complete(void) {
  TBTaskRun *run;
  char error[256];
  tb_lock(&tb_cpu_mutex);
  while (tb_cpu.completed_head == NULL && !tb_cpu.failed) {
    if (!tb_cpu_wait_locked()) {
      tb_unlock(&tb_cpu_mutex); err_fail("CPU coordinator wait failed");
    }
  }
  if (tb_cpu.failed) {
    (void)snprintf(error, sizeof(error), "%s", tb_cpu.error);
    tb_unlock(&tb_cpu_mutex); err_fail(error);
  }
  run = tb_cpu.completed_head; tb_cpu.completed_head = run->next;
  if (tb_cpu.completed_head == NULL) tb_cpu.completed_tail = NULL;
  tb_unlock(&tb_cpu_mutex);
  if (tb_cpu_pending == 0) err_fail("unbalanced CPU completion");
  --tb_cpu_pending;
  if (tb_cpu_pending == 0) tb_cpu_set_active(false);
  return run;
}
static void tb_cpu_shutdown(void) {
  tb_lock(&tb_cpu_mutex); tb_cpu.stop = true;
  tb_cpu_notify_locked(); tb_unlock(&tb_cpu_mutex);
  if (tb_cpu_pending != 0) tb_vm_cancel();
  for (u32 index = 0; index < tb_cpu.created; ++index) {
#ifdef _WIN32
    if (WaitForSingleObject(tb_cpu.threads[index], INFINITE) != WAIT_OBJECT_0) {
      (void)fputs("teamy-bend executable C: CPU worker join failed\n", stderr); exit(1);
    }
    (void)CloseHandle(tb_cpu.threads[index]);
#else
    if (pthread_join(tb_cpu.threads[index], NULL) != 0) {
      (void)fputs("teamy-bend executable C: CPU worker join failed\n", stderr); exit(1);
    }
#endif
  }
  tb_cpu.created = 0; tb_cpu_pending = 0; tb_cpu_set_active(false);
  /* Trusted initializers may evaluate before the next pool initialization.
   * Keep those evaluations sequential; no stale host or stopped pool survives. */
  tb_cpu.desired = 0; tb_cpu.host = NULL;
}
OUTLINE void tb_task_accept_event(Env e, TBTaskContext *context, TBTaskRun *run) {
  switch (run->event) {
    case TB_CPU_FINISH: tb_task_finish(e, context, run, run->result); return;
    case TB_CPU_GRAPH: tb_task_adopt(e, context, run->result.task, run, false); return;
    case TB_CPU_COORDINATOR: tb_task_enqueue(context, run); return;
    default: err_fail("invalid CPU task event");
  }
}
/* Every nested foreign reentry starts with an idle pool. Its private root and
 * graph registry can therefore run to completion without waiting on a caller
 * occupying a worker. Foreign callbacks never race generated worker bodies. */
OUTLINE TBOutcome tb_task_execute(Env e, Term closure, Term argument, Term input, bool task_entry) {
  TBTaskContext *context;
  TBOutcome answer;
  if (tb_worker_failure_capture) err_fail("CPU worker entered a foreign dispatcher");
  if (++tb_depth > BEND_MAX_DEPTH) err_fail("call depth budget exhausted");
  context = (TBTaskContext *)io_mem(tb_host_calloc(1, sizeof(*context)));
  context->memory = e.mem; context->previous = tb_task_current; tb_task_current = context;
  if (task_entry) tb_task_adopt(e, context, input, NULL, true);
  else {
    TBTaskRun *run = (TBTaskRun *)io_mem(tb_host_calloc(1, sizeof(*run)));
    run->closure = closure; run->argument = argument; run->expected = 1;
    tb_task_enqueue(context, run);
  }
  while (context->head != NULL || tb_cpu_pending != 0) {
    if (context->head == NULL) {
      tb_task_accept_event(e, context, tb_cpu_complete());
      continue;
    }
    TBTaskRun *run = context->head;
    bool parallel = tb_cpu.desired > 1 && (tb_cpu_pending != 0 || run->next != NULL)
        && tb_task_worker_safe(run);
    context->head = run->next;
    if (context->head == NULL) context->tail = NULL;
    if (parallel) {
      u32 frontier = tb_cpu_pending + 1;
      for (TBTaskRun *ready = context->head; ready != NULL && frontier < tb_cpu.desired; ready = ready->next)
        ++frontier;
      tb_cpu_submit(run, frontier);
    } else {
      while (tb_cpu_pending != 0) tb_task_accept_event(e, context, tb_cpu_complete());
      run->event = tb_task_advance(e, run, false);
      tb_task_accept_event(e, context, run);
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
