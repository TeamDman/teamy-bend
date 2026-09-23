/* SPDX-License-Identifier: Apache-2.0
 * Native values derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
 * Shared CPU/device implementation: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * Backend supplies allocation state, synchronization and bounded scratch storage. */
INLINE Cls cls_fit(u32 count) {
  Cls cls = 0;
  while ((UINT64_C(1) << cls) < count) ++cls;
  return cls;
}
#define TB_META_OWNED (UINT64_C(1) << 63)
#define TB_META_FREE (UINT64_C(1) << 62)
#define TB_META_SEALED (UINT64_C(1) << 61)
INLINE u64 tb_meta(Loc at, Cls cls) { return at | ((u64)(cls + 1) << 40); }
INLINE Cls tb_meta_class(u64 metadata) { return (Cls)((metadata >> 40) & 63) - 1; }
INLINE Loc heap_alloc_locked(Env e, Cls cls) {
  Loc at;
  u64 size;
  if (e.mem != tb_memory || tb_heap_meta == NULL || cls >= NCLS_ALL) err_fail("invalid heap allocation");
  size = UINT64_C(1) << cls;
  at = tb_free_lists[cls];
  if (at != 0) {
    if (at >= tb_bump || tb_heap_meta[at] != (tb_meta(at, cls) | TB_META_FREE))
      err_fail("invalid native free list");
    tb_free_lists[cls] = e.mem[at];
  } else {
    at = tb_bump;
    if (size > tb_capacity || at > tb_capacity - size) err_fail("VM allocation budget exhausted");
    tb_heap_commit(at + size);
    tb_bump += size;
  }
  memset(e.mem + at, 0, (size_t)size * sizeof(Term));
  for (u64 index = 0; index < size; ++index) tb_heap_meta[at + index] = tb_meta(at, cls) | TB_META_OWNED;
  tb_live_words += size; ++tb_live_blocks;
  return at;
}
INLINE Loc heap_alloc(Env e, Cls cls) {
  Loc at; tb_vm_acquire(); at = heap_alloc_locked(e, cls); tb_vm_release(); return at;
}
INLINE void tb_span_locked(Env e, Loc at, u64 count) {
  u64 metadata, size; Loc owner; Cls cls;
  if (e.mem != tb_memory || tb_heap_meta == NULL || at < HEAP_OFF || at >= tb_bump)
    err_fail("invalid native heap span");
  metadata = tb_heap_meta[at]; owner = metadata & LOC_MASK; cls = tb_meta_class(metadata);
  if (metadata == 0 || (metadata & TB_META_FREE) != 0 || owner < HEAP_OFF || owner > at || cls >= NCLS_ALL)
    err_fail("invalid native heap span");
  size = UINT64_C(1) << cls;
  if (at - owner >= size || count > size - (at - owner)) err_fail("invalid native heap span");
}
INLINE void tb_span(Env e, Loc at, u64 count) {
  tb_vm_acquire(); tb_span_locked(e, at, count); tb_vm_release();
}
INLINE void tb_allocation_locked(Env e, Loc at, Cls cls) {
  tb_span_locked(e, at, 1);
  if (cls >= NCLS_ALL || (tb_heap_meta[at] & ~(TB_META_OWNED | TB_META_SEALED)) != tb_meta(at, cls))
    err_fail("native allocation class mismatch");
}
INLINE void tb_allocation(Env e, Loc at, Cls cls) {
  tb_vm_acquire(); tb_allocation_locked(e, at, cls); tb_vm_release();
}
INLINE bool tb_cell_owned(Env e, Loc at) {
  bool owned;
  tb_vm_acquire(); tb_span_locked(e, at, 1); owned = (tb_heap_meta[at] & TB_META_OWNED) != 0;
  tb_vm_release(); return owned;
}
INLINE bool tb_cell_sealed(Env e, Loc at) {
  bool sealed;
  tb_vm_acquire(); tb_span_locked(e, at, 1);
  sealed = (tb_heap_meta[tb_heap_meta[at] & LOC_MASK] & TB_META_SEALED) != 0;
  tb_vm_release(); return sealed;
}
INLINE void tb_set_sealed(Env e, Loc at, bool sealed) {
  tb_vm_acquire(); tb_span_locked(e, at, 1);
  if ((tb_heap_meta[at] & LOC_MASK) != at) err_fail("invalid sealed allocation");
  if (sealed) tb_heap_meta[at] |= TB_META_SEALED;
  else tb_heap_meta[at] &= ~TB_META_SEALED;
  tb_vm_release();
}
INLINE void tb_mark_raw(Env e, Loc at, u64 count) {
  tb_vm_acquire(); tb_span_locked(e, at, count);
  if ((tb_heap_meta[tb_heap_meta[at] & LOC_MASK] & TB_META_SEALED) != 0)
    err_fail("mutation of sealed native layout");
  for (u64 index = 0; index < count; ++index) tb_heap_meta[at + index] &= ~TB_META_OWNED;
  tb_vm_release();
}
INLINE void tb_mark_owned(Env e, Loc at, u64 count) {
  tb_vm_acquire(); tb_span_locked(e, at, count);
  if ((tb_heap_meta[tb_heap_meta[at] & LOC_MASK] & TB_META_SEALED) != 0)
    err_fail("mutation of sealed native layout");
  for (u64 index = 0; index < count; ++index) tb_heap_meta[at + index] |= TB_META_OWNED;
  tb_vm_release();
}
INLINE void heap_free_locked(Env e, Cls cls, Loc at) {
  tb_allocation_locked(e, at, cls);
  memset(tb_heap_meta + at, 0, ((size_t)1 << cls) * sizeof(u64));
  tb_heap_meta[at] = tb_meta(at, cls) | TB_META_FREE;
  e.mem[at] = tb_free_lists[cls]; tb_free_lists[cls] = at;
  tb_live_words -= UINT64_C(1) << cls; --tb_live_blocks;
}
INLINE void heap_free(Env e, Cls cls, Loc at) {
  tb_vm_acquire(); heap_free_locked(e, cls, at); tb_vm_release();
}
INLINE void spare_free(Env e, Cls cls, Loc at) { if (at != 0) heap_free(e, cls, at); }
INLINE Term term_make(u64 tag, u64 aux, u64 loc) {
  if (tag > 127 || aux > 65535 || loc > LOC_MASK) err_fail("native term fields overflow");
  return (tag << 56) | (aux << 40) | loc;
}
#define term_pak(cid,loc) term_make(TAG_PAK,(cid),(loc))
#define term_ctr(cid,loc) term_make(TAG_CTR,(cid),(loc))
#define term_clo(fid,loc) term_make(TAG_CLO,(fid),(loc))
#define term_buf(cls,loc) term_make(TAG_BUF,(cls),(loc))
#define term_tsk(fid,loc) term_make(TAG_TSK,(fid),(loc))
INLINE u64 term_tag(Term term) { return (term >> 56) & 127; }
INLINE u64 term_aux(Term term) { return (term >> 40) & 65535; }
INLINE Loc term_loc(Term term) { return term & LOC_MASK; }
INLINE bool term_rfc(Term term) { return (term & RFC_BIT) != 0; }
INLINE bool term_triv(Term term) {
  return term_tag(term) <= TAG_PAK || term == TERM_HOLE
      || (term_tag(term) == TAG_CLO && term_loc(term) == 0 && !term_rfc(term));
}
INLINE u32 cid_arity(u32 cid) {
  if (cid >= BEND_CID_COUNT) err_fail("unknown constructor id");
  return CID_ARITY_T[cid];
}
#define RFC_CNT UINT32_C(0xffffff)
INLINE u64 rfc_view_locked(Env e, Loc cell) {
  u64 value;
  tb_allocation_locked(e, cell, 0); value = e.mem[cell];
  if ((value & RFC_CNT) == 0 || (value & RFC_CNT) == RFC_CNT) err_fail("invalid native reference count");
  tb_span_locked(e, value >> 24, 1);
  if ((tb_heap_meta[value >> 24] & TB_META_SEALED) == 0) err_fail("unsealed shared native payload");
  return value;
}
INLINE u64 rfc_view(Env e, Loc cell) {
  u64 value; tb_vm_acquire(); value = rfc_view_locked(e, cell); tb_vm_release(); return value;
}
INLINE void rfc_bump(Env e, Loc cell, u32 amount) {
  u64 value;
  tb_vm_acquire(); value = rfc_view_locked(e, cell);
  if (amount >= RFC_CNT - (u32)(value & RFC_CNT)) err_fail("native reference count overflow");
  e.mem[cell] = value + amount;
  tb_vm_release();
}
/* Claiming the final reference and freeing its count cell is one transaction.
 * A retained owner keeps the payload alive while shared readers clone fields. */
INLINE Loc rfc_release(Env e, Loc cell) {
  u64 value; Loc unique = 0;
  tb_vm_acquire(); value = rfc_view_locked(e, cell);
  if ((value & RFC_CNT) == 1) { unique = value >> 24; heap_free_locked(e, 0, cell); }
  else e.mem[cell] = value - 1;
  tb_vm_release(); return unique;
}
INLINE bool rfc_claim_unique(Env e, Loc cell) {
  bool unique;
  tb_vm_acquire(); unique = (rfc_view_locked(e, cell) & RFC_CNT) == 1;
  if (unique) heap_free_locked(e, 0, cell);
  tb_vm_release(); return unique;
}
OUTLINE void tb_seal_children(Env e, Term value);
INLINE Term rfc_wrap_sealed(Env e, Term term, u32 count) {
  Loc cell;
  if (term_rfc(term) || (term_tag(term) != TAG_CTR && term_tag(term) != TAG_ARR && term_tag(term) != TAG_BUF))
    err_fail("native value cannot use a reference-count cell");
  if (count == 0 || count >= RFC_CNT) err_fail("native reference count overflow");
  tb_span(e, term_loc(term), 1);
  if (!tb_cell_sealed(e, term_loc(term))) err_fail("unsealed shared native payload");
  cell = heap_alloc(e, 0); e.mem[cell] = (term_loc(term) << 24) | count;
  tb_mark_raw(e, cell, 1);
  return (term & ~LOC_MASK) | RFC_BIT | cell;
}
OUTLINE Term rfc_wrap(Env e, Term term, u32 count) {
  if (term_rfc(term) || (term_tag(term) != TAG_CTR && term_tag(term) != TAG_ARR && term_tag(term) != TAG_BUF))
    err_fail("native value cannot use a reference-count cell");
  if (count == 0 || count >= RFC_CNT) err_fail("native reference count overflow");
  tb_seal_children(e, term);
  return rfc_wrap_sealed(e, term, count);
}
INLINE Loc term_peek(Env e, Term term) {
  Loc at = term_rfc(term) ? rfc_view(e, term_loc(term)) >> 24 : term_loc(term);
  tb_span(e, at, 1); return at;
}
INLINE Term rfc_seal(Env e, Term term) {
  return term_tag(term) == TAG_CTR && !term_rfc(term) ? rfc_wrap(e, term, 1) : term;
}
INLINE Term term_keep(Env e, Term term) {
  if (term_triv(term)) return term;
  if (term_tag(term) == TAG_CLO || term_tag(term) == TAG_TSK)
    err_fail("native value cannot use a reference-count cell");
  if (term_rfc(term)) { rfc_bump(e, term_loc(term), 1); return term; }
  return rfc_wrap(e, term, 2);
}
/* Every published shared node has stable child words. Prepare descendants in
 * unique storage before exposing a second owner; subsequent shared reads only
 * retain cells or clone closure shells. Exact raw masks exclude arbitrary W64
 * tag bits from traversal. This explicit stack also bounds native stack use. */
typedef struct TBSealFrame TBSealFrame;
struct TBSealFrame { TBSealFrame *previous; Term value; Term *owner; Loc at; u32 index, count; };
OUTLINE void tb_seal_children(Env e, Term value) {
  TBSealFrame *stack = NULL;
  Term *owner = NULL;
  for (;;) {
    if (!term_triv(value) && !term_rfc(value)) {
      u32 tag = (u32)term_tag(value), aux = (u32)term_aux(value), count = 0;
      Cls cls;
      Loc at = term_loc(value);
      if (tag == TAG_CTR) { count = cid_arity(aux); cls = cls_fit(count); }
      else if (tag == TAG_CLO) {
        count = tb_closure_captures[aux];
        if (aux < 2 || count == 0) err_fail("invalid closure sealing");
        cls = cls_fit(count);
      } else if (tag == TAG_ARR || tag == TAG_BUF) {
        if (aux > 17) err_fail("invalid array layout");
        cls = tag == TAG_ARR ? aux : aux == 0 ? 0 : aux - 1;
        if (tag == TAG_ARR) count = 1u << cls;
      } else err_fail("native value cannot use a reference-count cell");
      tb_allocation(e, at, cls);
      if (!tb_cell_sealed(e, at)) {
        TBSealFrame *frame = (TBSealFrame *)io_mem(tb_host_calloc(1, sizeof(*frame)));
        frame->previous = stack; frame->value = value; frame->owner = owner; frame->at = at; frame->count = count;
        stack = frame;
      } else if (owner != NULL && tag != TAG_CLO) *owner = rfc_wrap_sealed(e, value, 1);
    }
    for (;;) {
      if (stack == NULL) return;
      tb_tick();
      if (stack->index < stack->count) {
        Loc cell = stack->at + stack->index++;
        if (!tb_cell_owned(e, cell)) continue;
        value = e.mem[cell]; owner = e.mem + cell;
        break;
      } else {
        TBSealFrame *frame = stack;
        tb_set_sealed(e, frame->at, true);
        if (frame->owner != NULL && term_tag(frame->value) != TAG_CLO)
          *frame->owner = rfc_wrap_sealed(e, frame->value, 1);
        stack = frame->previous; tb_host_free(frame);
      }
    }
  }
}
/* Closures remain affine shells. Sealed captured descendants permit recursive
 * structural duplication without ever rewriting a published capture. */
typedef struct TBDuplicateFrame TBDuplicateFrame;
struct TBDuplicateFrame { TBDuplicateFrame *previous; Loc source, target; u32 index, count; };
OUTLINE Term tb_duplicate(Env e, Term *owner) {
  Term result;
  Term *destination = &result;
  TBDuplicateFrame *stack = NULL;
  for (;;) {
    Term value = *owner;
    if (term_tag(value) == TAG_CLO && term_loc(value) != 0) {
      u32 fid = (u32)term_aux(value), count = tb_closure_captures[fid];
      Loc source, target;
      TBDuplicateFrame *frame;
      if (term_rfc(value) || fid < 2 || count == 0) err_fail("invalid closure duplication");
      tb_seal_children(e, value);
      source = term_peek(e, value); tb_allocation(e, source, cls_fit(count));
      target = heap_alloc(e, cls_fit(count));
      *destination = term_clo(fid, target);
      frame = (TBDuplicateFrame *)io_mem(tb_host_calloc(1, sizeof(*frame)));
      frame->previous = stack; frame->source = source; frame->target = target; frame->count = count;
      stack = frame;
    } else {
      Term kept = term_keep(e, value);
      if (kept != value) *owner = kept;
      *destination = kept;
    }
    while (stack != NULL && stack->index == stack->count) {
      TBDuplicateFrame *previous = stack->previous; tb_host_free(stack); stack = previous;
    }
    if (stack == NULL) return result;
    tb_tick();
    owner = e.mem + stack->source + stack->index;
    destination = e.mem + stack->target + stack->index++;
  }
}
/* Destructive pointer reversal holds traversal state in nodes that are already
 * uniquely owned. It needs no C recursion or input-sized host stack. */
OUTLINE void term_drop(Env e, Term term) {
  u64 current = 0;
  Term first = 0;
  for (;;) {
    if (!term_triv(term) && term_rfc(term)) {
      Loc at = rfc_release(e, term_loc(term));
      term = at == 0 ? 0 : (term & ~(RFC_BIT | LOC_MASK)) | at;
    }
    if (!term_triv(term)) {
      u32 tag = (u32)term_tag(term), aux = (u32)term_aux(term), count = 0;
      Loc at = term_loc(term); Cls cls;
      if (tag == TAG_BUF || tag == TAG_ARR) {
        if (aux > 17) err_fail("invalid array layout");
        cls = tag == TAG_ARR ? aux : aux == 0 ? 0 : aux - 1;
      } else if (tag == TAG_CTR) { count = cid_arity(aux); cls = cls_fit(count); }
      else if (tag == TAG_CLO) {
        count = tb_closure_captures[aux];
        if (aux < 2 || count == 0) err_fail("invalid closure destruction");
        cls = cls_fit(count);
      } else if (tag == TAG_TSK) { count = fid_arity(aux); cls = cls_fit(count + 2); }
      else { err_fail("invalid native term tag"); }
      tb_allocation(e, at, cls);
      if (tag == TAG_BUF) heap_free(e, cls, at);
      else {
        first = e.mem[at]; e.mem[at] = current;
        current = at | ((u64)count << 48) | ((u64)(tag == TAG_ARR ? cls | 64 : cls) << 56);
      }
    }
    for (;;) {
      Loc at; u32 index, count, position; Cls cls; bool array;
      if (current == 0) return;
      tb_tick();
      at = current & LOC_MASK; index = (u8)(current >> 40); count = (u8)(current >> 48);
      cls = (Cls)(current >> 56); array = cls >= 64; position = index;
      if (array) {
        cls &= 63; count = 1u << cls;
        if (index == 2) position = (u32)e.mem[at + 1];
      }
      if (position < count) {
        Term child = position == 0 ? first : e.mem[at + position];
        bool owned = tb_cell_owned(e, at + position);
        if (array && position > 0) e.mem[at + 1] = position + 1;
        if (!array || index < 2) current += UINT64_C(1) << 40;
        if (owned && !term_triv(child)) { term = child; break; }
      } else {
        u64 previous = e.mem[at]; heap_free(e, cls, at); current = previous;
      }
    }
  }
}
INLINE void term_sink(Env e, Term term) { if (!term_triv(term)) term_drop(e, term); }
INLINE Loc ctr_take(Env e, Term term, u32 count, Term *fields) {
  Loc at;
  if (term_tag(term) == TAG_PAK) {
    if (count > 1) err_fail("packed constructor has too many fields");
    if (count == 1) fields[0] = term_loc(term);
    return 0;
  }
  if (term_tag(term) != TAG_CTR) err_fail("constructor expected");
  if (count != cid_arity((u32)term_aux(term))) err_fail("constructor arity mismatch");
  at = term_peek(e, term);
  tb_allocation(e, at, cls_fit(count));
  if (term_rfc(term)) {
    Loc cell = term_loc(term);
    if (!rfc_claim_unique(e, cell)) {
      for (u32 index = 0; index < count; ++index)
        fields[index] = tb_cell_owned(e, at + index) ? tb_duplicate(e, e.mem + at + index) : e.mem[at + index];
      term_drop(e, term);
      return 0;
    }
  }
  memcpy(fields, e.mem + at, count * sizeof(Term));
  tb_set_sealed(e, at, false);
  return at;
}
INLINE Term tb_construct(Env e, u32 cid, u32 count, const Term *fields, bool packed) {
  Loc at;
  if (cid >= BEND_CID_COUNT || count != cid_arity(cid)) err_fail("constructor arity mismatch");
  if (packed) {
    if (count > 1 || (count != 0 && fields[0] > LOC_MASK)) err_fail("invalid packed constructor");
    return term_pak(cid, count == 0 ? 0 : fields[0]);
  }
  at = heap_alloc(e, cls_fit(count));
  if (count != 0) memcpy(e.mem + at, fields, count * sizeof(Term));
  return term_ctr(cid, at);
}
INLINE void tb_fields(Env e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  if (term_aux(value) != cid || count != cid_arity(cid) || term_tag(value) != (packed ? TAG_PAK : TAG_CTR))
    err_fail("constructor representation mismatch");
  spare_free(e, cls_fit(count), ctr_take(e, value, count, out));
}
INLINE void tb_borrow_fields(Env e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  Loc at;
  if (term_aux(value) != cid || count != cid_arity(cid) || term_tag(value) != (packed ? TAG_PAK : TAG_CTR))
    err_fail("constructor representation mismatch");
  if (packed) {
    if (count > 1) err_fail("packed constructor has too many fields");
    if (count != 0) out[0] = term_loc(value);
    return;
  }
  at = term_peek(e, value); tb_allocation(e, at, cls_fit(count));
  for (u32 index = 0; index < count; ++index)
    out[index] = tb_cell_owned(e, at + index) ? tb_duplicate(e, e.mem + at + index) : e.mem[at + index];
}
INLINE Term io_hand(u64 handle) {
  if (handle >> 56) err_fail("host handle exceeds native 56-bit representation");
  return term_pak(handle >> 40, handle & LOC_MASK);
}
INLINE u64 io_hand_v(Term handle) {
  if (term_tag(handle) != TAG_PAK) err_fail("native handle expected");
  return (term_aux(handle) << 40) | term_loc(handle);
}
INLINE f32 f32_unbox(u64 bits) { u32 raw = (u32)bits; f32 value; memcpy(&value, &raw, 4); return value; }
INLINE u64 f32_rewrap(f32 value) { u32 bits; memcpy(&bits, &value, 4); return bits; }
INLINE Nat nat_chk(Env e, Nat value) { (void)e; if (value > NAT_IMM) err_fail("Nat exceeds largest immediate"); return value; }
INLINE Nat nat_mul(Env e, Nat left, Nat right) {
  if (right != 0 && left > NAT_IMM / right) err_fail("Nat multiplication overflow");
  return nat_chk(e, left * right);
}
#define U32_BIN(a,op,b) ((u64)((u32)(a) op (u32)(b)))
#if defined(__CUDA_ARCH__)
INLINE Loc tb_device_corpus_reserve_pair(const Env *e, Cls first_cls,
    Cls repeated_cls, u32 repeated_count);
INLINE Loc tb_device_corpus_ticket_take(const Env *e, Loc *ticket);
INLINE void tb_device_corpus_initialize_reserved(const Env *e, Loc at, Cls cls);
#endif
INLINE Term term_word(Env e, Term word) {
  Term owner = word;
  u32 value = 0;
  u32 index = 0;
  while (term_aux(word) == CID_WCON && index < 32) {
    Loc at = term_peek(e, word);
    tb_span(e, at, 2);
    value |= ((u32)e.mem[at] & 1) << index++;
    word = e.mem[at + 1];
  }
  term_sink(e, owner); return value;
}
INLINE Term tb_word(Env e, u32 value) {
#if CID_WNIL >= BEND_CID_COUNT || CID_WCON >= BEND_CID_COUNT
  err_fail("Word representation unavailable");
#endif
  Term word = term_pak(CID_WNIL, 0);
#if defined(__CUDA_ARCH__)
  Loc ticket = tb_device_corpus_reserve_pair(&e, 1, 1, 31);
  for (u32 index = 32; index != 0; --index) {
    Loc at = tb_device_corpus_ticket_take(&e, &ticket);
    tb_device_corpus_initialize_reserved(&e, at, 1);
    e.mem[at] = (value >> (index - 1)) & 1;
    e.mem[at + 1] = word;
    word = term_ctr(CID_WCON, at);
  }
  if (ticket != 0) err_fail("invalid heap reservation");
#else
  for (u32 index = 32; index != 0; --index) {
    Loc at = heap_alloc(e, 1);
    e.mem[at] = (value >> (index - 1)) & 1; e.mem[at + 1] = word;
    word = term_ctr(CID_WCON, at);
  }
#endif
  return word;
}
