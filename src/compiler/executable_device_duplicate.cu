/* SPDX-License-Identifier: Apache-2.0
 * Value ownership derived from Bend 2.0.5 comp.ts and the shared value runtime,
 * Copyright 2026 HigherOrderCO. Persistent CUDA traversal: TeamDman.
 * See NOTICE and licenses/Apache-2.0.txt. Include after device primitives. */
#define TB_DEVICE_DUPLICATE_STATE_WORDS 4
#define TB_DEVICE_DUPLICATE_MAGIC UINT64_C(0x445550)
enum {
  TB_DEVICE_DUPLICATE_ENTER, TB_DEVICE_DUPLICATE_START,
  TB_DEVICE_DUPLICATE_SLICE, TB_DEVICE_DUPLICATE_CHILD_WRAPPED,
  TB_DEVICE_DUPLICATE_ROOT_WRAPPED, TB_DEVICE_DUPLICATE_RETAINED,
  TB_DEVICE_DUPLICATE_CLONE_RESERVED, TB_DEVICE_DUPLICATE_CLONE_COMPLETE,
  TB_DEVICE_DUPLICATE_COMPLETE, TB_DEVICE_DUPLICATE_FAST
};
#ifndef TB_DEVICE_DUPLICATE_OBSERVE
#define TB_DEVICE_DUPLICATE_OBSERVE(event, state, frame, work) ((void)0)
#endif
enum { TB_DUP_NONE, TB_DUP_CORPUS, TB_DUP_SCRATCH };
enum { TB_DUP_VALUE = 1, TB_DUP_SEAL = 2 };
enum {
  TB_DUP_READ, TB_DUP_WRAP_ROOT, TB_DUP_ALLOC_CLONE, TB_DUP_INIT_CLONE,
  TB_DUP_COPY_CAPTURE, TB_DUP_CLONE_DONE,
  TB_DUP_SEAL_ENTER, TB_DUP_SEAL_SCAN, TB_DUP_WRAP_CHILD, TB_DUP_DONE
};
struct TBDupRef { u64 arena, base, index; };
struct TBDupWork {
  u64 previous, kind, phase;
  TBDupRef owner, destination;
  Term value, output;
  u64 source, target, index, count, cls, cursor, state_index, before;
};
INLINE TBDupWork *tb_device_dup_frame(u64 offset) {
  return (TBDupWork *)tb_device_pointer(offset, sizeof(TBDupWork));
}
INLINE bool tb_device_dup_same_ref(TBDupRef a, TBDupRef b) {
  return a.arena == b.arena && a.base == b.base && a.index == b.index;
}
/* Scratch references carry the allocation base as well as the interior index.
 * Looking for a scratch header immediately before an interior cell is invalid. */
INLINE Term *tb_device_dup_pointer(Env e, TBDupRef ref) {
  if (ref.arena == TB_DUP_SCRATCH) {
    if (ref.index >= SIZE_MAX / sizeof(Term)) err_fail("invalid duplicate scratch reference");
    return (Term *)tb_device_pointer(ref.base, (size_t)(ref.index + 1) * sizeof(Term)) + ref.index;
  }
  if (ref.arena != TB_DUP_CORPUS || ref.base > LOC_MASK || ref.index > LOC_MASK - ref.base)
    err_fail("invalid duplicate corpus reference");
  tb_vm_acquire();
  tb_span_locked(e, ref.base, 1);
  if ((tb_heap_meta[ref.base] & LOC_MASK) != ref.base)
    err_fail("invalid duplicate allocation base");
  tb_span_locked(e, ref.base, ref.index + 1);
  tb_vm_release();
  return e.mem + ref.base + ref.index;
}
INLINE TBDupRef tb_device_dup_root_ref(TBCallFrame *frame, Term *cell, u64 count) {
  if (frame == NULL || frame->values == NULL || cell == NULL || count == 0)
    err_fail("missing duplicate frame storage");
  u64 base = tb_device_offset(frame->values);
  (void)tb_device_pointer(base, 0);
  u64 address = (u64)cell, start = (u64)frame->values;
  if (address < start || (address - start) % sizeof(Term) != 0)
    err_fail("invalid duplicate frame storage");
  u64 index = (address - start) / sizeof(Term), words = tb_device_scratch[base - 1];
  if (count > words || index > words - count)
    err_fail("duplicate cell is outside retained frame");
  TBDupRef ref = {TB_DUP_SCRATCH, base, index}; return ref;
}
INLINE void tb_device_dup_store(Env e, TBDupRef ref, Term value) {
  Term *destination = tb_device_dup_pointer(e, ref);
  if (ref.arena == TB_DUP_CORPUS && tb_cell_sealed(e, ref.base))
    err_fail("mutation of sealed duplicate owner");
  *destination = value;
}
INLINE void tb_device_dup_rewrite(Env e, TBDupRef ref, Term original, Term value) {
  if (*tb_device_dup_pointer(e, ref) != original)
    err_fail("duplicate owner changed during traversal");
  tb_device_dup_store(e, ref, value);
}
INLINE u64 tb_device_dup_push(u64 previous, u64 kind, TBDupRef owner,
    TBDupRef destination, Term value) {
  TBDupWork *frame = (TBDupWork *)io_mem(tb_host_calloc(1, sizeof(TBDupWork)));
  frame->previous = previous; frame->kind = kind;
  frame->phase = kind == TB_DUP_SEAL ? TB_DUP_SEAL_ENTER : TB_DUP_READ;
  frame->owner = owner; frame->destination = destination; frame->value = value;
  return tb_device_offset(frame);
}
INLINE void tb_device_dup_shape(Env e, TBDupWork *frame) {
  u32 tag = (u32)term_tag(frame->value), aux = (u32)term_aux(frame->value);
  if (tag == TAG_CTR) {
    frame->count = cid_arity(aux); frame->cls = cls_fit((u32)frame->count);
  } else if (tag == TAG_CLO) {
    frame->count = tb_closure_captures[aux];
    if (aux < 2 || frame->count == 0) err_fail("invalid closure sealing");
    frame->cls = cls_fit((u32)frame->count);
  } else if (tag == TAG_ARR || tag == TAG_BUF) {
    if (aux > 17) err_fail("invalid array layout");
    frame->cls = tag == TAG_ARR ? aux : aux == 0 ? 0 : aux - 1;
    frame->count = tag == TAG_ARR ? UINT64_C(1) << frame->cls : 0;
  } else err_fail("native value cannot use a reference-count cell");
  frame->source = term_loc(frame->value);
  tb_allocation(e, frame->source, (Cls)frame->cls);
}
/* Class-zero count cells are bounded. Nothing can suspend between allocating,
 * initializing, and recording their owner; later backing waits precede reserve. */
INLINE Term tb_device_dup_wrap(const Env *e, Term value, u32 count) {
  if (term_rfc(value) || (term_tag(value) != TAG_CTR && term_tag(value) != TAG_ARR
      && term_tag(value) != TAG_BUF) || !tb_cell_sealed(*e, term_loc(value)))
    err_fail("invalid duplicate shared payload");
  Loc cell = tb_device_corpus_reserve(e, 0);
  e->mem[cell] = (term_loc(value) << 24) | count;
  tb_heap_meta[cell] = tb_meta(cell, 0); /* Count cells contain raw bits. */
  return (value & ~LOC_MASK) | RFC_BIT | cell;
}
INLINE void tb_device_dup_result(Env e, TBDupWork *frame, Term value) {
  frame->output = value;
  if (frame->previous != 0) tb_device_dup_store(e, frame->destination, value);
  frame->phase = TB_DUP_DONE;
}

/* Four state cells: operation tag, top record offset, root record offset and
 * retained values allocation base. Frames retain offsets, never lane pointers.
 * Each state-machine transition or initialized word is one work unit. Logical
 * ticks remain at the original seal scan and duplicate capture traversal. */
OUTLINE bool tb_device_duplicate(const Env *e, TBCallFrame *call,
    Term *owner, Term *state, Term *result) {
  tb_device_check_cancelled();
  if (e == NULL || e->mem != tb_memory || tb_heap_meta == NULL)
    err_fail("invalid duplicate environment");
  TBDupRef owner_ref = tb_device_dup_root_ref(call, owner, 1);
  TBDupRef result_ref = tb_device_dup_root_ref(call, result, 1);
  TBDupRef state_ref = tb_device_dup_root_ref(call, state, TB_DEVICE_DUPLICATE_STATE_WORDS);
  if (owner_ref.index == result_ref.index
      || (owner_ref.index >= state_ref.index && owner_ref.index - state_ref.index < TB_DEVICE_DUPLICATE_STATE_WORDS)
      || (result_ref.index >= state_ref.index && result_ref.index - state_ref.index < TB_DEVICE_DUPLICATE_STATE_WORDS))
    err_fail("overlapping duplicate frame storage");
  TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_ENTER, state, NULL, 0);
  if (state[0] == 0) {
    if (state[1] != 0 || state[2] != 0 || state[3] != 0)
      err_fail("invalid duplicate operation state");
    if (term_triv(*owner)) {
      *result = *owner;
      TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_FAST, state, NULL, 0);
      return true;
    }
    if (term_rfc(*owner)) {
      *result = term_keep(*e, *owner);
      TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_FAST, state, NULL, 0);
      return true;
    }
    u64 root = tb_device_dup_push(0, TB_DUP_VALUE, owner_ref, result_ref, *owner);
    tb_device_dup_frame(root)->state_index = state_ref.index;
    state[0] = TB_DEVICE_DUPLICATE_MAGIC; state[1] = root; state[2] = root; state[3] = owner_ref.base;
    tb_device_lock();
    if (tb_device_control->primitive_starts == UINT64_MAX || tb_device_control->primitive_live == UINT32_MAX)
      err_fail("device primitive progress overflow");
    ++tb_device_control->primitive_starts; ++tb_device_control->primitive_live;
    tb_device_unlock();
    TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_START, state, tb_device_dup_frame(root), 0);
  }
  if (state[0] != TB_DEVICE_DUPLICATE_MAGIC || state[1] == 0 || state[2] == 0
      || state[3] != owner_ref.base)
    err_fail("invalid duplicate operation state");
  TBDupWork *root = tb_device_dup_frame(state[2]);
  if (root->previous != 0 || root->kind != TB_DUP_VALUE || root->state_index != state_ref.index
      || !tb_device_dup_same_ref(root->owner, owner_ref)
      || !tb_device_dup_same_ref(root->destination, result_ref))
    err_fail("duplicate operation storage changed");
  u64 work = 0;
  bool done = false;
  while (work < BEND_GPU_PRIMITIVE_QUANTUM && !done) {
    tb_device_check_cancelled();
    TBDupWork *frame = tb_device_dup_frame(state[1]);
    if (frame->kind != TB_DUP_VALUE && frame->kind != TB_DUP_SEAL)
      err_fail("invalid duplicate work kind");
    ++work;
    switch (frame->phase) {
      case TB_DUP_READ: {
        Term value = *tb_device_dup_pointer(*e, frame->owner);
        if (value != frame->value) err_fail("duplicate owner changed during traversal");
        if (term_triv(value)) tb_device_dup_result(*e, frame, value);
        else if (term_rfc(value)) {
          frame->before = rfc_view(*e, term_loc(value)) & RFC_CNT;
          Term kept = term_keep(*e, value);
          tb_device_dup_result(*e, frame, kept);
          TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_RETAINED, state, frame, work);
        } else {
          tb_device_dup_shape(*e, frame);
          frame->phase = term_tag(value) == TAG_CLO ? TB_DUP_ALLOC_CLONE : TB_DUP_WRAP_ROOT;
          TBDupRef none = {TB_DUP_NONE, 0, 0};
          state[1] = tb_device_dup_push(state[1], TB_DUP_SEAL, none, none, value);
        }
        break;
      }
      case TB_DUP_WRAP_ROOT: {
        Term kept = tb_device_dup_wrap(e, frame->value, 2);
        tb_device_dup_rewrite(*e, frame->owner, frame->value, kept);
        tb_device_dup_result(*e, frame, kept);
        TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_ROOT_WRAPPED, state, frame, work);
        break;
      }
      case TB_DUP_ALLOC_CLONE:
        frame->target = tb_device_corpus_reserve(e, (Cls)frame->cls);
        frame->cursor = 0; frame->phase = TB_DUP_INIT_CLONE;
        TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_CLONE_RESERVED, state, frame, work);
        break;
      case TB_DUP_INIT_CLONE:
        if (frame->cls >= NCLS_ALL || frame->cursor >= (UINT64_C(1) << frame->cls))
          err_fail("invalid duplicate allocation cursor");
        e->mem[frame->target + frame->cursor] = 0;
        tb_heap_meta[frame->target + frame->cursor] = tb_meta(frame->target, (Cls)frame->cls) | TB_META_OWNED;
        if (++frame->cursor == (UINT64_C(1) << frame->cls)) frame->phase = TB_DUP_COPY_CAPTURE;
        break;
      case TB_DUP_COPY_CAPTURE:
        if (frame->index > frame->count) err_fail("invalid duplicate capture cursor");
        if (frame->index == frame->count) frame->phase = TB_DUP_CLONE_DONE;
        else {
          tb_tick();
          TBDupRef source = {TB_DUP_CORPUS, frame->source, frame->index};
          TBDupRef target = {TB_DUP_CORPUS, frame->target, frame->index};
          ++frame->index;
          state[1] = tb_device_dup_push(state[1], TB_DUP_VALUE, source, target,
            *tb_device_dup_pointer(*e, source));
        }
        break;
      case TB_DUP_CLONE_DONE:
        tb_device_dup_result(*e, frame, term_clo(term_aux(frame->value), frame->target));
        TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_CLONE_COMPLETE, state, frame, work);
        break;
      case TB_DUP_SEAL_ENTER:
        if (term_triv(frame->value) || term_rfc(frame->value)) frame->phase = TB_DUP_DONE;
        else {
          tb_device_dup_shape(*e, frame);
          if (!tb_cell_sealed(*e, frame->source)) frame->phase = TB_DUP_SEAL_SCAN;
          else frame->phase = frame->owner.arena != TB_DUP_NONE && term_tag(frame->value) != TAG_CLO
            ? TB_DUP_WRAP_CHILD : TB_DUP_DONE;
        }
        break;
      case TB_DUP_SEAL_SCAN:
        tb_tick();
        if (frame->index > frame->count) err_fail("invalid duplicate seal cursor");
        if (frame->index == frame->count) {
          tb_set_sealed(*e, frame->source, true);
          frame->phase = frame->owner.arena != TB_DUP_NONE && term_tag(frame->value) != TAG_CLO
            ? TB_DUP_WRAP_CHILD : TB_DUP_DONE;
        } else {
          TBDupRef child = {TB_DUP_CORPUS, frame->source, frame->index++};
          if (tb_cell_owned(*e, child.base + child.index)) {
            TBDupRef none = {TB_DUP_NONE, 0, 0};
            state[1] = tb_device_dup_push(state[1], TB_DUP_SEAL, child, none,
              *tb_device_dup_pointer(*e, child));
          }
        }
        break;
      case TB_DUP_WRAP_CHILD: {
        Term wrapped = tb_device_dup_wrap(e, frame->value, 1);
        tb_device_dup_rewrite(*e, frame->owner, frame->value, wrapped);
        frame->output = wrapped; frame->phase = TB_DUP_DONE;
        TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_CHILD_WRAPPED, state, frame, work);
        break;
      }
      case TB_DUP_DONE:
        if (frame->previous == 0) {
          if (state[1] != state[2] || frame->kind != TB_DUP_VALUE)
            err_fail("invalid duplicate root completion");
          done = true;
        } else {
          u64 previous = frame->previous;
          tb_host_free(frame); state[1] = previous;
        }
        break;
      default: err_fail("invalid duplicate work phase");
    }
  }
  tb_device_lock();
  if (work == 0 || work > UINT64_MAX - tb_device_control->primitive_progress
      || (!done && tb_device_control->primitive_yields == UINT64_MAX)
      || tb_device_control->primitive_live == 0)
    err_fail("device primitive progress overflow");
  tb_device_control->primitive_progress += work;
  if (done) --tb_device_control->primitive_live;
  else ++tb_device_control->primitive_yields;
  tb_device_unlock();
  TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_SLICE, state, tb_device_dup_frame(state[1]), work);
  if (!done) return false;
  TB_DEVICE_DUPLICATE_OBSERVE(TB_DEVICE_DUPLICATE_COMPLETE, state, root, work);
  *result = root->output;
  tb_host_free(root);
  memset(state, 0, TB_DEVICE_DUPLICATE_STATE_WORDS * sizeof(Term));
  return true;
}
