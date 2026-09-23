/* SPDX-License-Identifier: Apache-2.0
 * Shared CPU/CUDA value bridges derived from Bend 2.0.5, Copyright 2026
 * HigherOrderCO. Changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt. */
OUTLINE TB_NOINLINE Term tb_c_construct(const Env *e, u32 cid, u32 count, const Term *fields, bool packed, const Term *box_mask) {
  Term value = tb_construct(*e, cid, count, fields, packed);
  if (!packed && box_mask != NULL) {
    Loc at = term_peek(*e, value);
    for (u32 index = 0; index < count; ++index)
      if (box_mask[index] == 0) tb_mark_raw(*e, at + index, 1);
  }
  return value;
}
OUTLINE TB_NOINLINE void tb_c_fields(const Env *e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  tb_fields(*e, value, cid, count, packed, out);
}
OUTLINE TB_NOINLINE void tb_c_borrow_fields(const Env *e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  tb_borrow_fields(*e, value, cid, count, packed, out);
}
OUTLINE TB_NOINLINE Term tb_c_duplicate(const Env *e, Term *value) { return tb_duplicate(*e, value); }
OUTLINE TB_NOINLINE void tb_c_drop(const Env *e, Term value) { term_drop(*e, value); }
OUTLINE TB_NOINLINE Loc tb_c_peek(const Env *e, Term value) { return term_peek(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_tail_apply(const Env *e, Term closure, Term argument) {
  return tb_tail_apply(*e, closure, argument);
}
OUTLINE TB_NOINLINE Term tb_c_word_task(const Env *e, u32 fid, u32 count,
    const Term *words, const Term *owned) {
  return tb_word_task(*e, fid, count, words, owned);
}
/* A join owns held words followed by each child's complete result range.
 * Only the first word temporarily owns the child task; the rest remain holes.
 * Completion replaces the range and its exact ownership mask together. */
OUTLINE TB_NOINLINE Term tb_c_word_join(const Env *e, u32 fid, u32 held_count,
    const Term *held, const Term *held_owned, u32 children, const Term *tasks) {
  u32 arity = held_count;
  if (children < 2 || children > 255 || held_count > 255)
    err_fail("invalid generated word fork layout");
  for (u32 index = 0; index < children; ++index) {
    Loc tail = task_tail(tasks[index]);
    u32 width = fid_result_width((Fid)term_aux(tasks[index]));
    if (e->mem[tail] != TERM_HOLE || e->mem[tail + 1] != 0 || width > 255 - arity)
      err_fail("invalid generated word fork child");
    arity += width;
  }
  if (fid_arity(fid) != arity) err_fail("invalid generated word fork arity");
  Loc at = task_node(*e, fid, TERM_HOLE, 0, children);
  Term join = term_tsk(fid, at);
  for (u32 index = 0; index < held_count; ++index) {
    Term owned = held_owned == NULL ? 1 : held_owned[index];
    if (owned > 1) err_fail("invalid segment ownership mask");
    e->mem[at + index] = held[index];
    if (owned == 0) tb_mark_raw(*e, at + index, 1);
  }
  u32 destination = held_count;
  for (u32 index = 0; index < children; ++index) {
    Loc tail = task_tail(tasks[index]);
    if (e->mem[tail] != TERM_HOLE || e->mem[tail + 1] != 0)
      err_fail("duplicate generated word fork child");
    e->mem[tail] = join;
    e->mem[tail + 1] = (u64)destination << 32;
    e->mem[at + destination] = tasks[index];
    tb_mark_owned(*e, at + destination, 1);
    destination += fid_result_width((Fid)term_aux(tasks[index]));
  }
  return join;
}
#if defined(__CUDA_ARCH__)
INLINE Loc tb_device_task_node_reserved(const Env *e, Fid fid, Term continuation,
    u32 index, u32 remaining, Loc at);
#endif
/* Build a word-lowered fork from one flat argument vector. CUDA first
 * validates the full layout and reserves the parent plus each child's exact
 * task class as one allocator transaction. The host keeps the established
 * per-child construction path and uses caller-owned frame storage for tasks. */
OUTLINE TB_NOINLINE Term tb_c_word_join_build(const Env *e, u32 fid,
    u32 held_count, const Term *held, const Term *held_owned, u32 children,
    const Term *child_fids, u32 argument_count, const Term *arguments,
    const Term *argument_owned, Term *tasks) {
  if (children < 2 || children > 255 || held_count > 255
      || child_fids == NULL || tasks == NULL
      || (held_count != 0 && held == NULL))
    err_fail("invalid generated word fork layout");
  u32 arity = held_count, expected_arguments = 0;
  for (u32 index = 0; index < held_count; ++index)
    if (held_owned != NULL && held_owned[index] > 1)
      err_fail("invalid segment ownership mask");
  for (u32 index = 0; index < children; ++index) {
    if (child_fids[index] >= UINT32_C(65536))
      err_fail("invalid generated word fork child");
    Fid child_fid = (Fid)child_fids[index];
    u32 child_arity = fid_arity(child_fid);
    u32 width = fid_result_width(child_fid);
    if (child_arity > UINT32_MAX - expected_arguments || width == 0
        || width > 255 - arity)
      err_fail("invalid generated word fork child");
    expected_arguments += child_arity;
    arity += width;
  }
  if (expected_arguments != argument_count
      || (argument_count != 0 && (arguments == NULL || argument_owned == NULL))
      || fid_arity((Fid)fid) != arity)
    err_fail("invalid generated word fork arity");
#if defined(__CUDA_ARCH__)
  for (u32 index = 0; index < argument_count; ++index) {
    if (argument_owned[index] > 1)
      err_fail("invalid task ownership mask");
    if (argument_owned[index] != 0 && arguments[index] == TERM_HOLE)
      err_fail("foreign task argument is missing");
    if (argument_owned[index] != 0 && term_tag(arguments[index]) == TAG_TSK)
      err_fail("runnable task contains a pending task");
  }
  Loc ticket = tb_device_corpus_reserve_task_children(e,
      cls_fit(arity + 2), child_fids, children);
  Loc parent = tb_device_task_node_reserved(e, (Fid)fid, TERM_HOLE, 0,
      children, tb_device_corpus_ticket_take(e, &ticket));
  Term join = term_tsk((Fid)fid, parent);
  for (u32 index = 0; index < held_count; ++index) {
    e->mem[parent + index] = held[index];
    if (held_owned != NULL && held_owned[index] == 0)
      tb_mark_raw(*e, parent + index, 1);
  }
  u32 argument = 0, destination = held_count;
  for (u32 index = 0; index < children; ++index) {
    Fid child_fid = (Fid)child_fids[index];
    u32 child_arity = fid_arity(child_fid);
    u32 width = fid_result_width(child_fid);
    Loc child = tb_device_task_node_reserved(e, child_fid, TERM_HOLE, 0, 0,
        tb_device_corpus_ticket_take(e, &ticket));
    for (u32 word = 0; word < child_arity; ++word) {
      e->mem[child + word] = arguments[argument + word];
      if (argument_owned[argument + word] == 0)
        tb_mark_raw(*e, child + word, 1);
    }
    Loc tail = child + child_arity;
    if (e->mem[tail] != TERM_HOLE || e->mem[tail + 1] != 0)
      err_fail("duplicate generated word fork child");
    e->mem[tail] = join;
    e->mem[tail + 1] = (u64)destination << 32;
    e->mem[parent + destination] = term_tsk(child_fid, child);
    tb_mark_owned(*e, parent + destination, 1);
    argument += child_arity;
    destination += width;
  }
  if (ticket != 0) err_fail("invalid heap reservation");
  return join;
#else
  u32 argument = 0;
  for (u32 index = 0; index < children; ++index) {
    Fid child_fid = (Fid)child_fids[index];
    u32 child_arity = fid_arity(child_fid);
    tasks[index] = tb_word_task(*e, child_fid, child_arity,
        child_arity == 0 ? NULL : arguments + argument,
        child_arity == 0 ? NULL : argument_owned + argument);
    argument += child_arity;
  }
  return tb_c_word_join(e, fid, held_count, held, held_owned, children, tasks);
#endif
}
#if defined(__CUDA_ARCH__)
INLINE Loc tb_device_task_node_reserved(const Env *e, Fid fid, Term continuation,
    u32 index, u32 remaining, Loc at) {
  u32 arity = fid_arity(fid);
  if ((continuation == TERM_HOLE && index != 0)
      || (continuation != TERM_HOLE && (term_tag(continuation) != TAG_TSK || term_rfc(continuation)))
      || remaining > arity)
    err_fail("foreign task continuation is unsupported");
  tb_device_corpus_initialize_reserved(e, at, cls_fit(arity + 2));
  for (u32 i = 0; i < arity; ++i) e->mem[at + i] = TERM_HOLE;
  e->mem[at + arity] = continuation;
  e->mem[at + arity + 1] = ((u64)index << 32) | remaining;
  tb_mark_raw(*e, at + arity, 2);
  return at;
}
#endif
OUTLINE TB_NOINLINE Term tb_c_join(const Env *e, u32 fid, u32 held_count, const Term *held,
    u32 children, const Term *applications) {
  Loc at;
  Term join;
  if (children < 2 || held_count > 254 || children > 254 - held_count
      || fid_arity(fid) != held_count + children + 1)
    err_fail("invalid generated fork layout");
#if defined(__CUDA_ARCH__)
  u32 parent_arity = fid_arity(fid), child_arity = fid_arity(FID_CLO_APPLY);
  Loc ticket = tb_device_corpus_reserve_pair(e, cls_fit(parent_arity + 2),
      cls_fit(child_arity + 2), children);
  at = tb_device_task_node_reserved(e, (Fid)fid, TERM_HOLE, 0, children,
      tb_device_corpus_ticket_take(e, &ticket));
#else
  at = task_node(*e, fid, TERM_HOLE, 0, children);
#endif
  join = term_tsk(fid, at);
  if (held_count != 0) memcpy(e->mem + at, held, held_count * sizeof(Term));
  e->mem[at + held_count + children] = 0;
  for (u32 index = 0; index < children; ++index) {
    u32 destination = held_count + index;
#if defined(__CUDA_ARCH__)
    Loc child = tb_device_task_node_reserved(e, FID_CLO_APPLY, join, destination, 0,
        tb_device_corpus_ticket_take(e, &ticket));
#else
    Loc child = task_node(*e, FID_CLO_APPLY, join, destination, 0);
#endif
    e->mem[child] = applications[2 * index];
    e->mem[child + 1] = applications[2 * index + 1];
    e->mem[at + destination] = term_tsk(FID_CLO_APPLY, child);
  }
#if defined(__CUDA_ARCH__)
  if (ticket != 0) err_fail("invalid heap reservation");
#endif
  return join;
}
OUTLINE TB_NOINLINE Term tb_c_closure(const Env *e, u32 fid, u32 count, const Term *captures) {
  return tb_closure(*e, fid, count, captures);
}
OUTLINE TB_NOINLINE Term tb_c_word(const Env *e, u32 value) { return tb_word(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_term_word(const Env *e, Term value) { return term_word(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_nat_chk(const Env *e, Nat value) { return nat_chk(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_nat_mul(const Env *e, Nat left, Nat right) { return nat_mul(*e, left, right); }
OUTLINE TB_NOINLINE Term tb_c_blk_node(const Env *e, Term left, Term right) { return blk_node(*e, left, right); }
OUTLINE TB_NOINLINE Term tb_c_blk_new(const Env *e, bool array, Nat depth, u32 lgs, u32 count, Term *values, const Term *box_mask) {
  return tb_blk_new_masked(*e, array, depth, lgs, count, values, box_mask);
}
OUTLINE TB_NOINLINE Term tb_c_blk_half(const Env *e, Term value, u32 high) { return blk_half(*e, value, high); }
OUTLINE TB_NOINLINE Term tb_c_blk_copy(const Env *e, Term value) { return blk_copy(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_blk_unique(const Env *e, Term value) { return blk_unique(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_blk_keep(const Env *e, Loc at) { return blk_keep(*e, at); }
OUTLINE TB_NOINLINE void tb_c_blk_free(const Env *e, Term value) { blk_free(*e, value); }
OUTLINE TB_NOINLINE void tb_c_blk_write(const Env *e, bool array, Loc at, u32 index, Term value, bool owned, bool discard) {
  if (array && discard && tb_cell_owned(*e, at + index)) term_drop(*e, e->mem[at + index]);
  blk_write(e->mem, array, at, index, value);
  if (array) {
    if (owned) tb_mark_owned(*e, at + index, 1);
    else tb_mark_raw(*e, at + index, 1);
  }
}
