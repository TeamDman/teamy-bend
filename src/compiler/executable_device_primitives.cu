/* SPDX-License-Identifier: Apache-2.0
 * Allocation and array layout derived from Bend 2.0.5 comp.ts and the shared
 * value runtime, Copyright 2026 HigherOrderCO. Resumable CUDA implementation:
 * TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * Pending corpus blocks are privately owned
 * by persistent generated frames until every initialization/fill slice ends.
 * Include after executable_device_tasks.cu and the shared array/value core. */
#ifndef BEND_GPU_PRIMITIVE_QUANTUM
#define BEND_GPU_PRIMITIVE_QUANTUM 1024
#endif
#if BEND_GPU_PRIMITIVE_QUANTUM < 1 || BEND_GPU_PRIMITIVE_QUANTUM > 4096
#error BEND_GPU_PRIMITIVE_QUANTUM must be between 1 and 4096
#endif
#define TB_DEVICE_ARRAY_NEW_STATE_WORDS 8
/* phase, source, destination, cursor, word count, physical class, block tag,
 * array flag, and whether the current copy reads a still-shared source. */
#define TB_DEVICE_ARRAY_COPY_STATE_WORDS 9
enum {
  TB_DEVICE_ARRAY_COPY_ENTER = 0, TB_DEVICE_ARRAY_COPY_START = 1,
  TB_DEVICE_ARRAY_COPY_RESERVED = 2, TB_DEVICE_ARRAY_COPY_SLICE = 3,
  TB_DEVICE_ARRAY_COPY_YIELD = 4, TB_DEVICE_ARRAY_COPY_COMPLETE = 5
};
enum {
  TB_DEVICE_ARRAY_COPY_IDLE = 0, TB_DEVICE_ARRAY_COPY_COW_INIT = 1,
  TB_DEVICE_ARRAY_COPY_COW_VALUES = 2, TB_DEVICE_ARRAY_COPY_CLONE_INIT = 3,
  TB_DEVICE_ARRAY_COPY_CLONE_VALUES = 4
};
enum {
  TB_DEVICE_ARRAY_COPY_FLAG_COW = 1, TB_DEVICE_ARRAY_COPY_FLAG_DEST_INITIALIZED = 2
};
enum {
  TB_DEVICE_ARRAY_NEW_ENTER = 0, TB_DEVICE_ARRAY_NEW_RESERVED = 1,
  TB_DEVICE_ARRAY_NEW_SLICE = 2, TB_DEVICE_ARRAY_NEW_COMPLETE = 3
};
#define TB_DEVICE_PAYLOAD_STATE_WORDS 6
enum {
  TB_DEVICE_PAYLOAD_ENTER = 0, TB_DEVICE_PAYLOAD_CTR_RESERVED = 1,
  TB_DEVICE_PAYLOAD_CLO_RESERVED = 2, TB_DEVICE_PAYLOAD_SLICE = 3,
  TB_DEVICE_PAYLOAD_COMPLETE = 4
};
/* Optional test observation points run outside locks. A production observer
 * must not alter operands, state, or corpus ownership. */
#ifndef TB_DEVICE_ARRAY_NEW_OBSERVE
#define TB_DEVICE_ARRAY_NEW_OBSERVE(event, state, work) ((void)0)
#endif
#ifndef TB_DEVICE_ARRAY_COPY_OBSERVE
#define TB_DEVICE_ARRAY_COPY_OBSERVE(event, state, work) ((void)0)
#endif
#ifndef TB_DEVICE_PAYLOAD_OBSERVE
#define TB_DEVICE_PAYLOAD_OBSERVE(event, state, work, fields, mask, count, closure) ((void)0)
#endif

OUTLINE bool tb_device_duplicate(const Env *e, TBCallFrame *call,
    Term *owner, Term *state, Term *result);

/* Reserve without touching the block's potentially large payload/metadata.
 * Device storage currently backs the complete capacity; tb_heap_commit stays
 * before allocator mutation so a later backing-request protocol can suspend
 * here without retrying consumed work. A failed invocation discards its arena
 * rather than attempting to free a partially initialized private block. */
INLINE Loc tb_device_corpus_reserve(const Env *e, Cls cls) {
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL || cls >= NCLS_ALL)
    err_fail("invalid heap allocation");
  u64 size = UINT64_C(1) << cls;
  tb_vm_acquire();
  Loc at = tb_free_lists[cls];
  if (at != 0) {
    if (at < HEAP_OFF || at >= tb_bump || size > tb_bump - at
        || tb_heap_meta[at] != (tb_meta(at, cls) | TB_META_FREE))
      err_fail("invalid native free list");
  } else {
    at = tb_bump;
    if (at < HEAP_OFF || size > tb_capacity || at > tb_capacity - size)
      err_fail("VM allocation budget exhausted");
  }
  if (size > tb_capacity || tb_live_words > tb_capacity - size
      || tb_live_blocks == UINT64_MAX)
    err_fail("invalid native heap accounting");
  tb_heap_commit(at + size);
  if (tb_free_lists[cls] != 0) tb_free_lists[cls] = e->mem[at];
  else tb_bump = at + size;
  tb_live_words += size; ++tb_live_blocks;
  tb_vm_release();
  return at;
}

/* Atomically reserve one block followed by either a repeated class or the
 * classes implied by a run of child function identifiers. Reserved offsets
 * form a private linked ticket in request order, avoiding a lane-local offsets
 * array even for a wide or heterogeneous task join. */
INLINE Loc tb_device_corpus_reserve_bundle(const Env *e, Cls first_cls,
    Cls repeated_cls, const Term *fids, u32 tail_count) {
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL
      || first_cls >= NCLS_ALL || tail_count == UINT32_MAX
      || (fids == NULL && repeated_cls >= NCLS_ALL))
    err_fail("invalid heap reservation");
  u32 requested[NCLS_ALL] = {0}, reused[NCLS_ALL] = {0}, consumed[NCLS_ALL] = {0};
  u32 count = tail_count + 1;
  ++requested[first_cls];
  u64 total_words = UINT64_C(1) << first_cls;
  for (u32 index = 0; index < tail_count; ++index) {
    Cls cls = repeated_cls;
    if (fids != NULL) {
      if (fids[index] >= UINT32_C(65536)) err_fail("invalid heap reservation");
      cls = cls_fit(fid_arity((Fid)fids[index]) + 2);
    }
    if (cls >= NCLS_ALL || requested[cls] == UINT32_MAX)
      err_fail("invalid heap reservation");
    ++requested[cls];
    total_words += UINT64_C(1) << cls;
  }
  u64 fresh_words = 0;
  tb_vm_acquire();
  for (u32 cls = 0; cls < NCLS_ALL; ++cls) {
    Loc at = tb_free_lists[cls];
    u64 size = UINT64_C(1) << cls;
    while (at != 0 && reused[cls] < requested[cls]) {
      if (at < HEAP_OFF || at >= tb_bump || size > tb_bump - at
          || tb_heap_meta[at] != (tb_meta(at, (Cls)cls) | TB_META_FREE))
        err_fail("invalid native free list");
      Loc next = e->mem[at];
      if (next != 0 && (next < HEAP_OFF || next >= tb_bump))
        err_fail("invalid native free list");
      at = next;
      ++reused[cls];
    }
    fresh_words += (u64)(requested[cls] - reused[cls]) * size;
  }
  if (tb_bump < HEAP_OFF || tb_bump > tb_capacity
      || fresh_words > tb_capacity - tb_bump
      || total_words > tb_capacity)
    err_fail("VM allocation budget exhausted");
  if (tb_live_words > tb_capacity - total_words
      || count > UINT64_MAX - tb_live_blocks)
    err_fail("invalid native heap accounting");
  Loc next_bump = tb_bump + fresh_words;
  if (fresh_words != 0) tb_heap_commit(next_bump);

  Loc head = 0, previous = 0;
  for (u32 index = 0; index < count; ++index) {
    Cls cls = first_cls;
    if (index != 0) {
      cls = repeated_cls;
      if (fids != NULL) cls = cls_fit(fid_arity((Fid)fids[index - 1]) + 2);
    }
    Loc at;
    if (consumed[cls] < reused[cls]) {
      at = tb_free_lists[cls];
      if (at == 0) err_fail("invalid native free list");
      tb_free_lists[cls] = e->mem[at];
      ++consumed[cls];
    } else {
      at = tb_bump;
      tb_bump += UINT64_C(1) << cls;
    }
    if (previous == 0) head = at;
    else e->mem[previous] = at;
    previous = at;
  }
  if (previous != 0) e->mem[previous] = 0;
  if (tb_bump != next_bump) err_fail("invalid native heap accounting");
  tb_live_words += total_words;
  tb_live_blocks += count;
  tb_vm_release();
  return head;
}

INLINE Loc tb_device_corpus_reserve_pair(const Env *e, Cls first_cls,
    Cls repeated_cls, u32 repeated_count) {
  return tb_device_corpus_reserve_bundle(e, first_cls, repeated_cls, NULL,
      repeated_count);
}
INLINE Loc tb_device_corpus_reserve_task_children(const Env *e, Cls parent_cls,
    const Term *child_fids, u32 children) {
  return tb_device_corpus_reserve_bundle(e, parent_cls, 0, child_fids, children);
}

/* Advance a private reservation ticket before its current block is zeroed. */
INLINE Loc tb_device_corpus_ticket_take(const Env *e, Loc *ticket) {
  if (e == NULL || e->mem != tb_memory || ticket == NULL
      || *ticket < HEAP_OFF || *ticket >= tb_bump)
    err_fail("invalid heap reservation");
  Loc at = *ticket, next = e->mem[at];
  if (next != 0 && (next < HEAP_OFF || next >= tb_bump))
    err_fail("invalid heap reservation");
  *ticket = next;
  return at;
}

/* Copy-on-write and clone use the same bounded block copier. Shared sources
 * are never rewritten: their owned fields are duplicated from a persistent
 * frame slot. For a unique source, duplicate rewrites are published back to
 * its cells, matching blk_copy's synchronous ownership transfer. */
OUTLINE bool tb_device_array_clone_raw(const Env *e, TBCallFrame *call,
    Term *owner, Term *result, Term *duplicate_owner, Term *duplicate_state,
    Term *duplicate_result, Term *state) {
  tb_device_check_cancelled();
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL || call == NULL
      || owner == NULL || result == NULL || duplicate_owner == NULL
      || duplicate_state == NULL || duplicate_result == NULL || state == NULL
      || owner == result || duplicate_owner == result || owner == duplicate_owner)
    err_fail("invalid device array clone arguments");
  TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_ENTER, state, 0);
  bool resumed = state[0] != TB_DEVICE_ARRAY_COPY_IDLE;
  if (state[0] == TB_DEVICE_ARRAY_COPY_IDLE) {
    for (u32 cell = 1; cell < TB_DEVICE_ARRAY_COPY_STATE_WORDS; ++cell)
      if (state[cell] != 0) err_fail("invalid device array clone state");
    if (*result != 0 || *duplicate_owner != 0 || *duplicate_result != 0
        || duplicate_state[0] != 0 || duplicate_state[1] != 0
        || duplicate_state[2] != 0 || duplicate_state[3] != 0)
      err_fail("invalid device array clone frame");
    Term block = *owner;
    u32 tag = (u32)term_tag(block);
    Cls logical = blk_cls(block);
    if ((tag != TAG_ARR && tag != TAG_BUF) || logical > 17)
      err_fail("array expected");
    Cls physical = tag == TAG_ARR ? logical : buf_wcls(logical);
    Loc source = term_peek(*e, block);
    tb_allocation(*e, source, physical);
    state[1] = source;
    state[4] = UINT64_C(1) << physical;
    state[5] = physical;
    state[6] = block & ~(RFC_BIT | LOC_MASK);
    state[7] = tag == TAG_ARR;
    state[8] = 0;
    if (term_rfc(block)) {
      if (rfc_claim_unique(*e, term_loc(block))) {
        tb_set_sealed(*e, source, false);
        *owner = state[6] | source;
      } else {
        state[0] = TB_DEVICE_ARRAY_COPY_COW_INIT;
        state[8] = TB_DEVICE_ARRAY_COPY_FLAG_COW;
      }
    }
    if (state[0] == TB_DEVICE_ARRAY_COPY_IDLE)
      state[0] = TB_DEVICE_ARRAY_COPY_CLONE_INIT;
    state[2] = tb_device_corpus_reserve(e, physical);
    tb_device_lock();
    if (tb_device_control->primitive_starts == UINT64_MAX
        || tb_device_control->primitive_live == UINT32_MAX)
      err_fail("device primitive progress overflow");
    ++tb_device_control->primitive_starts;
    ++tb_device_control->primitive_live;
    tb_device_unlock();
    TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_START, state, 0);
    TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_RESERVED, state, 0);
  } else if ((state[0] != TB_DEVICE_ARRAY_COPY_COW_INIT
          && state[0] != TB_DEVICE_ARRAY_COPY_COW_VALUES
          && state[0] != TB_DEVICE_ARRAY_COPY_CLONE_INIT
          && state[0] != TB_DEVICE_ARRAY_COPY_CLONE_VALUES)
      || state[1] < HEAP_OFF || state[1] >= tb_bump || state[2] < HEAP_OFF
      || state[2] >= tb_bump || state[4] == 0 || state[5] >= NCLS_ALL
      || state[4] != (UINT64_C(1) << state[5]) || state[6] == 0
      || (state[7] != 0 && state[7] != 1)
      || (state[8] & ~(TB_DEVICE_ARRAY_COPY_FLAG_COW
          | TB_DEVICE_ARRAY_COPY_FLAG_DEST_INITIALIZED)) != 0
      || state[3] >= state[4]
      || ((state[0] == TB_DEVICE_ARRAY_COPY_COW_INIT
              || state[0] == TB_DEVICE_ARRAY_COPY_COW_VALUES)
          != ((state[8] & TB_DEVICE_ARRAY_COPY_FLAG_COW) != 0))
      || ((state[0] == TB_DEVICE_ARRAY_COPY_COW_VALUES
              || state[0] == TB_DEVICE_ARRAY_COPY_CLONE_VALUES)
          && (state[8] & TB_DEVICE_ARRAY_COPY_FLAG_DEST_INITIALIZED) == 0)
      || (state[7] != (u64)(term_tag(state[6]) == TAG_ARR))
      || (term_tag(state[6]) != TAG_ARR && term_tag(state[6]) != TAG_BUF)
      || term_aux(state[6]) > 17)
    err_fail("invalid device array clone state");
  if (resumed) {
    tb_allocation(*e, state[1], (Cls)state[5]);
    if ((state[8] & TB_DEVICE_ARRAY_COPY_FLAG_DEST_INITIALIZED) != 0)
      tb_allocation(*e, state[2], (Cls)state[5]);
    if (state[0] == TB_DEVICE_ARRAY_COPY_COW_INIT
        || state[0] == TB_DEVICE_ARRAY_COPY_COW_VALUES) {
      if (!term_rfc(*owner) || term_peek(*e, *owner) != state[1]
          || (*owner & ~(RFC_BIT | LOC_MASK)) != state[6])
        err_fail("shared array owner changed during copy");
    } else if (term_rfc(*owner) || term_loc(*owner) != state[1]
        || (*owner & ~(RFC_BIT | LOC_MASK)) != state[6]) {
      err_fail("unique array owner changed during copy");
    }
  }

  u64 work = 0;
  bool duplicate_waiting = false;
  while (work < BEND_GPU_PRIMITIVE_QUANTUM) {
    tb_device_check_cancelled();
    if (state[0] == TB_DEVICE_ARRAY_COPY_COW_INIT
        || state[0] == TB_DEVICE_ARRAY_COPY_CLONE_INIT) {
      e->mem[state[2] + state[3]] = 0;
      tb_heap_meta[state[2] + state[3]] = tb_meta(state[2], (Cls)state[5]);
      state[8] |= TB_DEVICE_ARRAY_COPY_FLAG_DEST_INITIALIZED;
      ++state[3];
      ++work;
      if (state[3] == state[4]) {
        state[3] = 0;
        state[0] = state[0] == TB_DEVICE_ARRAY_COPY_COW_INIT
            ? TB_DEVICE_ARRAY_COPY_COW_VALUES : TB_DEVICE_ARRAY_COPY_CLONE_VALUES;
      }
    } else if (state[0] == TB_DEVICE_ARRAY_COPY_COW_VALUES
        || state[0] == TB_DEVICE_ARRAY_COPY_CLONE_VALUES) {
      bool cow = state[0] == TB_DEVICE_ARRAY_COPY_COW_VALUES;
      Loc from = state[1], to = state[2], index = state[3];
      Loc count = state[4];
      if (index >= count) err_fail("invalid device array clone cursor");
      bool owned = state[7] != 0 && tb_cell_owned(*e, from + index);
      if (owned) {
        if (duplicate_state[0] == 0)
          *duplicate_owner = e->mem[from + index];
        if (!tb_device_duplicate(e, call, duplicate_owner, duplicate_state,
            duplicate_result)) {
          duplicate_waiting = true;
          break;
        }
        Term value = *duplicate_result;
        *duplicate_result = 0;
        if (!cow) {
          if (tb_cell_sealed(*e, from))
            err_fail("unique array copy source is sealed");
          e->mem[from + index] = *duplicate_owner;
        }
        *duplicate_owner = 0;
        e->mem[to + index] = value;
        tb_heap_meta[to + index] = tb_meta(to, (Cls)state[5]) | TB_META_OWNED;
      } else {
        e->mem[to + index] = e->mem[from + index];
        tb_heap_meta[to + index] = tb_meta(to, (Cls)state[5]);
      }
      ++state[3];
      ++work;
      if (state[3] == count) {
        if (cow) {
          Term previous = *owner;
          *owner = state[6] | to;
          term_drop(*e, previous);
          state[1] = to;
          state[2] = tb_device_corpus_reserve(e, (Cls)state[5]);
          state[3] = 0;
          state[0] = TB_DEVICE_ARRAY_COPY_CLONE_INIT;
          state[6] = *owner & ~(RFC_BIT | LOC_MASK);
          state[8] = 0;
          TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_RESERVED, state, work);
        } else {
          *result = state[6] | to;
          state[0] = TB_DEVICE_ARRAY_COPY_IDLE;
          break;
        }
      }
    } else {
      err_fail("invalid device array clone phase");
    }
  }

  bool done = state[0] == TB_DEVICE_ARRAY_COPY_IDLE;
  tb_device_lock();
  if ((work == 0 && !duplicate_waiting)
      || work > UINT64_MAX - tb_device_control->primitive_progress
      || (!done && !duplicate_waiting
          && tb_device_control->primitive_yields == UINT64_MAX)
      || tb_device_control->primitive_live == 0)
    err_fail("device primitive progress overflow");
  tb_device_control->primitive_progress += work;
  if (done) --tb_device_control->primitive_live;
  else if (!duplicate_waiting) ++tb_device_control->primitive_yields;
  tb_device_unlock();
  TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_SLICE, state, work);
  if (!done) {
    if (!duplicate_waiting)
      TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_YIELD, state, work);
    return false;
  }
  TB_DEVICE_ARRAY_COPY_OBSERVE(TB_DEVICE_ARRAY_COPY_COMPLETE, state, work);
  memset(state, 0, TB_DEVICE_ARRAY_COPY_STATE_WORDS * sizeof(Term));
  return true;
}

/* Complete a previously reserved block without changing allocator counters.
 * The free-list marker (if reused) is still present until this private block is
 * initialized. No other lane can discover it after the reservation commit. */
INLINE void tb_device_corpus_initialize_reserved(const Env *e, Loc at, Cls cls) {
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL || cls >= NCLS_ALL)
    err_fail("invalid heap reservation");
  u64 size = UINT64_C(1) << cls;
  if (at < HEAP_OFF || at >= tb_bump || size > tb_bump - at)
    err_fail("invalid heap reservation");
  u64 metadata = tb_heap_meta[at];
  if (metadata != 0 && metadata != (tb_meta(at, cls) | TB_META_FREE))
    err_fail("invalid heap reservation");
  memset(e->mem + at, 0, (size_t)size * sizeof(Term));
  for (u64 index = 0; index < size; ++index)
    tb_heap_meta[at + index] = tb_meta(at, cls) | TB_META_OWNED;
}

/* State cells: phase, allocation offset, cursor, logical class, physical class,
 * stride, value count, array flag. Phase 1 initializes physical words and
 * metadata; phase 2 fills logical elements; phase 3 is complete. Boxed array
 * elements may suspend through the nested duplicate state before one slot is
 * published. All operands and state cells are rooted in the generated frame. */
OUTLINE bool tb_device_array_new_raw(const Env *e, TBCallFrame *call, bool array,
    Nat depth, u32 lgs, u32 count, Term *raw_values, const Term *box_mask,
    Term *duplicate_state, Term *duplicate_result, Term *state, Term *result) {
  tb_device_check_cancelled();
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL || state == NULL
      || result == NULL || (count != 0 && raw_values == NULL)
      || ((box_mask == NULL) != (duplicate_state == NULL))
      || ((box_mask == NULL) != (duplicate_result == NULL))
      || (box_mask != NULL && (!array || call == NULL)))
    err_fail("invalid device array primitive arguments");
  TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_ENTER, state, 0);
  if (depth > 17 || lgs > 17 || depth + lgs > 17 || count > (1u << lgs))
    err_fail("array budget exhausted");
  if (box_mask != NULL)
    for (u32 column = 0; column < count; ++column)
      if (box_mask[column] > 1) err_fail("invalid array ownership mask");
  Cls logical = (Cls)depth + lgs, physical = array ? logical : buf_wcls(logical);
  u64 words = UINT64_C(1) << physical, elements = UINT64_C(1) << logical;
  u32 stride = 1u << lgs;
  if (state[0] == 0) {
    for (u32 cell = 1; cell < TB_DEVICE_ARRAY_NEW_STATE_WORDS; ++cell)
      if (state[cell] != 0) err_fail("invalid device array primitive state");
    Loc at = tb_device_corpus_reserve(e, physical);
    state[0] = 1; state[1] = at; state[2] = 0;
    state[3] = logical; state[4] = physical; state[5] = stride;
    state[6] = count; state[7] = array;
    tb_device_lock();
    if (tb_device_control->primitive_starts == UINT64_MAX
        || tb_device_control->primitive_live == UINT32_MAX)
      err_fail("device primitive progress overflow");
    ++tb_device_control->primitive_starts; ++tb_device_control->primitive_live;
    tb_device_unlock();
    TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_RESERVED, state, 0);
  } else if ((state[0] != 1 && state[0] != 2) || state[3] != logical
      || state[4] != physical || state[5] != stride || state[6] != count
      || state[7] != (u64)array || state[1] < HEAP_OFF
      || words > tb_capacity || state[1] > tb_capacity - words
      || state[2] >= (state[0] == 1 ? words : elements))
    err_fail("invalid device array primitive state");

  Loc at = state[1];
  u64 cursor = state[2], work = 0;
  bool duplicate_waiting = false;
  u64 metadata = tb_meta(at, physical) | (array ? 0 : TB_META_OWNED);
  while (work < BEND_GPU_PRIMITIVE_QUANTUM && state[0] != 3) {
    tb_device_check_cancelled();
    if (state[0] == 1) {
      e->mem[at + cursor] = 0;
      tb_heap_meta[at + cursor] = metadata;
      ++cursor; ++work;
      if (cursor == words) { state[0] = 2; cursor = 0; }
    } else {
      u32 column = (u32)cursor & (stride - 1);
      bool owned = array && column < count && box_mask != NULL
          && box_mask[column] != 0;
      bool needs_duplicate = owned && depth != 0
          && (cursor >> lgs) + 1 < (UINT64_C(1) << (u32)depth);
      Term value = column < count ? raw_values[column] : 0;
      if (needs_duplicate) {
        if (duplicate_state == NULL || duplicate_result == NULL)
          err_fail("missing boxed array duplication state");
        if (!tb_device_duplicate(e, call, raw_values + column,
            duplicate_state, duplicate_result)) {
          duplicate_waiting = true;
          break;
        }
        value = *duplicate_result;
        *duplicate_result = 0;
      }
      if (array) e->mem[at + cursor] = value;
      else {
        u32 shift = ((u32)cursor & 1u) * 32;
        u64 *word = e->mem + at + cursor / 2;
        *word = (*word & ~(UINT64_C(0xffffffff) << shift)) | ((u64)(u32)value << shift);
      }
      if (array)
        tb_heap_meta[at + cursor] = tb_meta(at, physical)
            | (owned ? TB_META_OWNED : 0);
      ++cursor; ++work;
      if (needs_duplicate) break;
      if (cursor == elements) state[0] = 3;
    }
  }
  state[2] = cursor;
  bool done = state[0] == 3;
  tb_device_lock();
  if ((work == 0 && !duplicate_waiting)
      || work > UINT64_MAX - tb_device_control->primitive_progress
      || (!done && !duplicate_waiting
          && tb_device_control->primitive_yields == UINT64_MAX)
      || tb_device_control->primitive_live == 0)
    err_fail("device primitive progress overflow");
  tb_device_control->primitive_progress += work;
  if (done) --tb_device_control->primitive_live;
  else {
    /* A pending duplicate owns the yield count for this task requeue. Counting
     * a second outer yield here could make yields exceed measured work. */
    if (!duplicate_waiting) ++tb_device_control->primitive_yields;
  }
  tb_device_unlock();
  TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_SLICE, state, work);
  if (!done) return false;

  TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_COMPLETE, state, work);
  *result = term_blk(array, logical, at);
  memset(state, 0, TB_DEVICE_ARRAY_NEW_STATE_WORDS * sizeof(Term));
  return true;
}

/* Constructors and captured closures each own one exact allocation class.
 * Reserve it before exposing copied fields, then initialize the payload and
 * ownership map in bounded slices. State and operands live in the generated
 * continuation frame. */
OUTLINE bool tb_device_payload_raw(const Env *e, u32 id, u32 count,
    const Term *fields, const Term *box_mask, bool closure, Term *state,
    Term *result) {
  tb_device_check_cancelled();
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL || state == NULL
      || result == NULL || (count != 0 && fields == NULL))
    err_fail("invalid device payload arguments");
  if (closure) {
    if ((id != FID_IO_EMIT && tb_device_fid_kind(id) != 1)
        || count != (id == FID_IO_EMIT ? 0 : tb_closure_captures[id])
        || box_mask != NULL)
      err_fail("invalid device closure captures");
  } else if (id >= BEND_CID_COUNT || count != cid_arity(id)) {
    err_fail("invalid device constructor layout");
  }
  for (u32 index = 0; index < count; ++index)
    if (box_mask != NULL && box_mask[index] > 1)
      err_fail("invalid constructor ownership mask");
  TB_DEVICE_PAYLOAD_OBSERVE(TB_DEVICE_PAYLOAD_ENTER, state, 0, fields,
      box_mask, count, closure);
  Cls cls = cls_fit(count);
  u64 words = UINT64_C(1) << cls;
  if (state[0] == 0) {
    for (u32 cell = 1; cell < TB_DEVICE_PAYLOAD_STATE_WORDS; ++cell)
      if (state[cell] != 0) err_fail("invalid device payload state");
    Loc at = tb_device_corpus_reserve(e, cls);
    state[0] = closure ? TB_DEVICE_PAYLOAD_CLO_RESERVED
                       : TB_DEVICE_PAYLOAD_CTR_RESERVED;
    state[1] = at; state[2] = 0; state[3] = id; state[4] = count;
    state[5] = cls;
    tb_device_lock();
    if (tb_device_control->primitive_starts == UINT64_MAX
        || tb_device_control->primitive_live == UINT32_MAX)
      err_fail("device primitive progress overflow");
    ++tb_device_control->primitive_starts; ++tb_device_control->primitive_live;
    tb_device_unlock();
    TB_DEVICE_PAYLOAD_OBSERVE(state[0], state, 0, fields, box_mask, count,
        closure);
  } else if (state[0] != (closure ? TB_DEVICE_PAYLOAD_CLO_RESERVED
                                  : TB_DEVICE_PAYLOAD_CTR_RESERVED)
      || state[1] < HEAP_OFF || state[1] > LOC_MASK
      || state[1] >= tb_bump || words > tb_capacity || state[1] > tb_capacity - words
      || state[2] >= words || state[3] != id || state[4] != count
      || state[5] != cls
      || (state[2] != 0 && (tb_heap_meta[state[1]]
          & ~(TB_META_OWNED | TB_META_SEALED)) != tb_meta(state[1], cls)))
    err_fail("invalid device payload state");

  Loc at = state[1];
  u64 cursor = state[2], work = 0;
  while (work < BEND_GPU_PRIMITIVE_QUANTUM && cursor < words) {
    tb_device_check_cancelled();
    Term value = cursor < count ? fields[cursor] : 0;
    bool owned = closure || cursor >= count || box_mask == NULL || box_mask[cursor] != 0;
    e->mem[at + cursor] = value;
    tb_heap_meta[at + cursor] = tb_meta(at, cls) | (owned ? TB_META_OWNED : 0);
    ++cursor; ++work;
  }
  state[2] = cursor;
  bool done = cursor == words;
  tb_device_lock();
  if (work == 0 || work > UINT64_MAX - tb_device_control->primitive_progress
      || (!done && tb_device_control->primitive_yields == UINT64_MAX)
      || tb_device_control->primitive_live == 0)
    err_fail("device primitive progress overflow");
  tb_device_control->primitive_progress += work;
  if (done) --tb_device_control->primitive_live;
  else ++tb_device_control->primitive_yields;
  tb_device_unlock();
  TB_DEVICE_PAYLOAD_OBSERVE(TB_DEVICE_PAYLOAD_SLICE, state, work, fields,
      box_mask, count, closure);
  if (!done) return false;

  state[0] = TB_DEVICE_PAYLOAD_COMPLETE;
  *result = closure ? term_clo(id, at) : term_ctr(id, at);
  TB_DEVICE_PAYLOAD_OBSERVE(TB_DEVICE_PAYLOAD_COMPLETE, state, work, fields,
      box_mask, count, closure);
  memset(state, 0, TB_DEVICE_PAYLOAD_STATE_WORDS * sizeof(Term));
  return true;
}

OUTLINE bool tb_device_construct_raw(const Env *e, u32 cid, u32 count,
    const Term *fields, const Term *box_mask, Term *state, Term *result) {
  return tb_device_payload_raw(e, cid, count, fields, box_mask, false, state,
      result);
}

OUTLINE bool tb_device_closure_raw(const Env *e, u32 fid, u32 count,
    const Term *captures, Term *state, Term *result) {
  return tb_device_payload_raw(e, fid, count, captures, NULL, true, state,
      result);
}
