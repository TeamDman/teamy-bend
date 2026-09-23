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
enum {
  TB_DEVICE_ARRAY_NEW_ENTER = 0, TB_DEVICE_ARRAY_NEW_RESERVED = 1,
  TB_DEVICE_ARRAY_NEW_SLICE = 2, TB_DEVICE_ARRAY_NEW_COMPLETE = 3
};
/* Optional test observation points run outside locks. A production observer
 * must not alter operands, state, or corpus ownership. */
#ifndef TB_DEVICE_ARRAY_NEW_OBSERVE
#define TB_DEVICE_ARRAY_NEW_OBSERVE(event, state, work) ((void)0)
#endif

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
 * stride, value count, array flag. Phase 1 initializes physical words and their
 * metadata; phase 2 fills logical elements; phase 3 is complete. The caller
 * keeps operands and these cells in its persistent generated frame. */
OUTLINE bool tb_device_array_new_raw(const Env *e, bool array, Nat depth,
    u32 lgs, u32 count, const Term *raw_values, Term *state, Term *result) {
  tb_device_check_cancelled();
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL || state == NULL
      || result == NULL || (count != 0 && raw_values == NULL))
    err_fail("invalid device array primitive arguments");
  TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_ENTER, state, 0);
  if (depth > 17 || lgs > 17 || depth + lgs > 17 || count > (1u << lgs))
    err_fail("array budget exhausted");
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
      Term value = column < count ? raw_values[column] : 0;
      if (array) e->mem[at + cursor] = value;
      else {
        u32 shift = ((u32)cursor & 1u) * 32;
        u64 *word = e->mem + at + cursor / 2;
        *word = (*word & ~(UINT64_C(0xffffffff) << shift)) | ((u64)(u32)value << shift);
      }
      ++cursor; ++work;
      if (cursor == elements) state[0] = 3;
    }
  }
  state[2] = cursor;
  bool done = state[0] == 3;
  tb_device_lock();
  if (work == 0 || work > UINT64_MAX - tb_device_control->primitive_progress
      || (!done && tb_device_control->primitive_yields == UINT64_MAX)
      || tb_device_control->primitive_live == 0)
    err_fail("device primitive progress overflow");
  tb_device_control->primitive_progress += work;
  if (done) --tb_device_control->primitive_live;
  else ++tb_device_control->primitive_yields;
  tb_device_unlock();
  TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_SLICE, state, work);
  if (!done) return false;

  TB_DEVICE_ARRAY_NEW_OBSERVE(TB_DEVICE_ARRAY_NEW_COMPLETE, state, work);
  *result = term_blk(array, logical, at);
  memset(state, 0, TB_DEVICE_ARRAY_NEW_STATE_WORDS * sizeof(Term));
  return true;
}
