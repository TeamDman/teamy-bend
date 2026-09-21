/* SPDX-License-Identifier: Apache-2.0
 * Native ABI derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
 * Bounded CPU implementation: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * CPU ownership uses bounded, validated allocation classes and count cells.
 * This does not implement the upstream parallel GPU/task optimizer. */
#ifdef _MSC_VER
#define _CRT_SECURE_NO_WARNINGS
#endif
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <limits.h>
#include <errno.h>
#include <setjmp.h>
#include <math.h>
#include <time.h>
#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <process.h>
#include <io.h>
#include <fcntl.h>
#include <sys/stat.h>
#ifdef _MSC_VER
#pragma comment(lib, "ws2_32.lib")
#endif
#define TB_THREAD_LOCAL __declspec(thread)
#define INLINE static __inline
#else
#include <pthread.h>
#include <unistd.h>
#include <poll.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#define TB_THREAD_LOCAL _Thread_local
#define INLINE static inline
#endif
#define OUTLINE static
#ifdef _MSC_VER
#define TB_NORETURN __declspec(noreturn)
#define TB_NOINLINE __declspec(noinline)
#else
#define TB_NORETURN _Noreturn
#define TB_NOINLINE __attribute__((noinline))
#endif
#define THR
#define DEV
#define DEVL
#define CONSTV static const
#ifndef BEND_MAX_STEPS
#define BEND_MAX_STEPS UINT64_C(2000000)
#endif
#ifndef BEND_MAX_DEPTH
#define BEND_MAX_DEPTH 512u
#endif
#ifndef BEND_MAX_FRAMES
#define BEND_MAX_FRAMES 512u
#endif
#ifndef BEND_MAX_CONTINUATIONS
#define BEND_MAX_CONTINUATIONS 65536u
#endif
#ifndef BEND_MAX_ALLOC
#define BEND_MAX_ALLOC UINT64_C(67108864)
#endif
#ifndef BEND_MAX_HOST_ALLOC
#define BEND_MAX_HOST_ALLOC UINT64_C(67108864)
#endif
#ifndef BEND_MAX_HOST_BUFFER
#define BEND_MAX_HOST_BUFFER UINT64_C(8388608)
#endif
#ifndef BEND_MAX_ACTIONS
#define BEND_MAX_ACTIONS 131072u
#endif
#ifndef BEND_MAX_WORKERS
#define BEND_MAX_WORKERS 64u
#endif
#ifndef IO_HOTS
#define IO_HOTS 0
#endif
#ifndef CID_UNIT
#define CID_UNIT BEND_CID_COUNT
#endif
#ifndef CID_EMIT
#define CID_EMIT BEND_CID_COUNT
#endif
#ifndef CID_HALT
#define CID_HALT BEND_CID_COUNT
#endif
#ifndef CID_SCON
#define CID_SCON BEND_CID_COUNT
#endif
#ifndef CID_SNIL
#define CID_SNIL BEND_CID_COUNT
#endif
#ifndef CID_TUPLE
#define CID_TUPLE BEND_CID_COUNT
#endif
#ifndef CID_DONE
#define CID_DONE BEND_CID_COUNT
#endif
#ifndef CID_FAIL
#define CID_FAIL BEND_CID_COUNT
#endif
#ifndef CID_NONE
#define CID_NONE BEND_CID_COUNT
#endif
#ifndef CID_SOME
#define CID_SOME BEND_CID_COUNT
#endif
#ifndef CID_TRUE
#define CID_TRUE BEND_CID_COUNT
#endif
#ifndef CID_FALSE
#define CID_FALSE BEND_CID_COUNT
#endif
#ifndef CID_WCON
#define CID_WCON BEND_CID_COUNT
#endif
#ifndef CID_WNIL
#define CID_WNIL BEND_CID_COUNT
#endif
#ifndef CID_CHR
#define CID_CHR BEND_CID_COUNT
#endif
#ifndef CID_NIL
#define CID_NIL BEND_CID_COUNT
#endif
#ifndef CID_CON
#define CID_CON BEND_CID_COUNT
#endif
typedef uint64_t u64;
typedef uint32_t u32;
typedef uint8_t u8;
typedef float f32;
typedef u64 Term;
typedef u64 Loc;
typedef u32 Cls;
typedef u32 Fid;
typedef Term Nat;
typedef Term U32;
typedef Term Reply;
typedef Term *Corpus;
typedef struct { Corpus mem; u64 *alc; } Env;
#define LOC_MASK ((UINT64_C(1) << 40) - 1)
#define TAG_PAK 1u
#define TAG_CTR 2u
#define TAG_CLO 3u
#define TAG_BUF 4u
#define TAG_TSK 5u
#define TAG_ARR 6u
#define TERM_HOLE UINT64_MAX
#define RFC_BIT (UINT64_C(1) << 63)
#define NAT_IMM ((UINT64_C(1) << 48) - 1)
#define HEAP_OFF 1u
#define NCLS_ALL 32u
#ifndef FID_CLO_APPLY
#define FID_CLO_APPLY 0u
#endif
#ifndef FID_IO_EMIT
#define FID_IO_EMIT 1u
#endif

#ifdef _WIN32
typedef SRWLOCK TBMutex;
#define TB_MUTEX_INIT SRWLOCK_INIT
INLINE void tb_lock(TBMutex *mutex) { AcquireSRWLockExclusive(mutex); }
INLINE void tb_unlock(TBMutex *mutex) { ReleaseSRWLockExclusive(mutex); }
#else
typedef pthread_mutex_t TBMutex;
#define TB_MUTEX_INIT PTHREAD_MUTEX_INITIALIZER
INLINE void tb_lock(TBMutex *mutex) { (void)pthread_mutex_lock(mutex); }
INLINE void tb_unlock(TBMutex *mutex) { (void)pthread_mutex_unlock(mutex); }
#endif

typedef struct TBAllocation TBAllocation;
typedef struct TBHost TBHost;
typedef struct TBFile TBFile;
typedef struct TBNetSocket TBNetSocket;
#ifdef _MSC_VER
struct __declspec(align(16)) TBAllocation { TBAllocation *next; size_t size; };
#else
struct TBAllocation { TBAllocation *next; size_t size; max_align_t alignment; };
#endif
struct TBHost {
  TBMutex mutex;
  TBAllocation *allocations;
  TBFile *files;
  TBFile *idle_files;
  u32 file_count;
  TBNetSocket *sockets;
  TBNetSocket *idle_sockets;
  u32 socket_count;
  intptr_t wake_read;
  intptr_t wake_write;
  bool wake_failed;
  u64 bytes;
  u32 references;
  bool stopped;
  bool worker_failed;
#ifdef _WIN32
  bool winsock;
#endif
};
static TB_THREAD_LOCAL TBHost *tb_host_current;
static TB_THREAD_LOCAL jmp_buf *tb_failure_guard;
static Corpus tb_memory;
static u64 *tb_heap_meta;
static Loc tb_free_lists[NCLS_ALL];
static Loc tb_bump;
static Loc tb_capacity;
static u64 tb_live_words;
static u64 tb_live_blocks;
static u64 tb_steps;
static u32 tb_depth;
static u32 tb_frames;
static u32 tb_continuations;
static u32 tb_closure_captures[65536];
static void tb_files_release(TBHost *host);
static void tb_network_shutdown(TBHost *host);

OUTLINE TB_NORETURN void err_fail(const char *message) {
  if (tb_host_current != NULL) {
    tb_lock(&tb_host_current->mutex);
    tb_host_current->worker_failed = true;
    tb_unlock(&tb_host_current->mutex);
  }
  (void)fprintf(stderr, "teamy-bend executable C: %s\n", message);
  if (tb_failure_guard != NULL) longjmp(*tb_failure_guard, 1);
  exit(1);
}
INLINE void tb_tick(void) {
  if (tb_steps++ >= BEND_MAX_STEPS) err_fail("evaluation budget exhausted");
}
OUTLINE void *tb_host_malloc(size_t size) {
  TBHost *host = tb_host_current;
  TBAllocation *allocation;
  u64 charge;
  if (host == NULL || size > BEND_MAX_HOST_BUFFER || size > SIZE_MAX - sizeof(*allocation)) {
    errno = ENOMEM; return NULL;
  }
  charge = (u64)size + sizeof(*allocation);
  tb_lock(&host->mutex);
  if (host->bytes > BEND_MAX_HOST_ALLOC || charge > BEND_MAX_HOST_ALLOC - host->bytes) {
    tb_unlock(&host->mutex); errno = ENOMEM; return NULL;
  }
  allocation = (TBAllocation *)malloc(sizeof(*allocation) + size);
  if (allocation != NULL) {
    allocation->size = size;
    allocation->next = host->allocations;
    host->allocations = allocation;
    host->bytes += charge;
  }
  tb_unlock(&host->mutex);
  return allocation == NULL ? NULL : allocation + 1;
}
OUTLINE void *tb_host_calloc(size_t count, size_t size) {
  void *memory;
  if (size != 0 && count > SIZE_MAX / size) { errno = ENOMEM; return NULL; }
  memory = tb_host_malloc(count * size);
  if (memory != NULL) memset(memory, 0, count * size);
  return memory;
}
OUTLINE void tb_host_free(void *memory) {
  TBHost *host = tb_host_current;
  TBAllocation **at;
  TBAllocation *allocation;
  if (memory == NULL) return;
  if (host == NULL) err_fail("foreign free outside an invocation");
  tb_lock(&host->mutex);
  for (at = &host->allocations; *at != NULL && (void *)(*at + 1) != memory; at = &(*at)->next) {}
  allocation = *at;
  if (allocation != NULL) { *at = allocation->next; host->bytes -= allocation->size + sizeof(*allocation); }
  tb_unlock(&host->mutex);
  if (allocation == NULL) err_fail("foreign free of an untracked allocation");
  free(allocation);
}
OUTLINE void *tb_host_realloc(void *memory, size_t size) {
  TBHost *host = tb_host_current;
  TBAllocation *allocation;
  size_t old_size = 0;
  void *replacement;
  if (memory == NULL) return tb_host_malloc(size);
  if (size == 0) { tb_host_free(memory); return NULL; }
  if (host == NULL) err_fail("foreign realloc outside an invocation");
  tb_lock(&host->mutex);
  for (allocation = host->allocations; allocation != NULL; allocation = allocation->next) {
    if ((void *)(allocation + 1) == memory) { old_size = allocation->size; break; }
  }
  tb_unlock(&host->mutex);
  if (allocation == NULL) err_fail("foreign realloc of an untracked allocation");
  replacement = tb_host_malloc(size);
  if (replacement == NULL) return NULL;
  memcpy(replacement, memory, old_size < size ? old_size : size);
  tb_host_free(memory);
  return replacement;
}
OUTLINE void tb_host_release(TBHost *host) {
  TBAllocation *allocation;
  bool final;
  tb_lock(&host->mutex);
  final = --host->references == 0;
  tb_unlock(&host->mutex);
  if (!final) return;
  tb_files_release(host);
  tb_network_shutdown(host);
  while ((allocation = host->allocations) != NULL) {
    host->allocations = allocation->next;
    free(allocation);
  }
#ifdef _WIN32
  if (host->winsock) (void)WSACleanup();
#endif
#ifndef _WIN32
  (void)pthread_mutex_destroy(&host->mutex);
#endif
  free(host);
}
INLINE void *io_mem(void *memory) {
  if (memory == NULL) err_fail("host allocation budget exhausted");
  return memory;
}
/* Generated functions keep every temporary Term in these bounded heap frames.
 * Small non-inlined wrappers allocate before entering a body and release after
 * every normal return. The host owner releases frames abandoned by longjmp.
 * Count conversion/printer/definition calls as well as closure applications. */
OUTLINE Term *tb_frame_push(size_t slots) {
  if (tb_frames >= BEND_MAX_FRAMES) err_fail("generated frame depth budget exhausted");
  ++tb_frames;
  if (slots == 0) return NULL;
  if (slots > SIZE_MAX / sizeof(Term)) err_fail("generated frame allocation overflow");
  return (Term *)io_mem(tb_host_calloc(slots, sizeof(Term)));
}
OUTLINE void tb_frame_pop(Term *values) {
  if (tb_frames == 0) err_fail("unbalanced generated frame");
  tb_host_free(values);
  --tb_frames;
}
INLINE Cls cls_fit(u32 count) {
  Cls cls = 0;
  while ((UINT64_C(1) << cls) < count) ++cls;
  return cls;
}
#define TB_META_OWNED (UINT64_C(1) << 63)
#define TB_META_FREE (UINT64_C(1) << 62)
INLINE u64 tb_meta(Loc at, Cls cls) { return at | ((u64)(cls + 1) << 40); }
INLINE Cls tb_meta_class(u64 metadata) { return (Cls)((metadata >> 40) & 63) - 1; }
INLINE Loc heap_alloc(Env e, Cls cls) {
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
    tb_bump += size;
  }
  memset(e.mem + at, 0, (size_t)size * sizeof(Term));
  for (u64 index = 0; index < size; ++index) tb_heap_meta[at + index] = tb_meta(at, cls) | TB_META_OWNED;
  tb_live_words += size; ++tb_live_blocks;
  return at;
}
INLINE void tb_span(Env e, Loc at, u64 count) {
  u64 metadata, size; Loc owner; Cls cls;
  if (e.mem != tb_memory || tb_heap_meta == NULL || at < HEAP_OFF || at >= tb_bump)
    err_fail("invalid native heap span");
  metadata = tb_heap_meta[at]; owner = metadata & LOC_MASK; cls = tb_meta_class(metadata);
  if (metadata == 0 || (metadata & TB_META_FREE) != 0 || owner < HEAP_OFF || owner > at || cls >= NCLS_ALL)
    err_fail("invalid native heap span");
  size = UINT64_C(1) << cls;
  if (at - owner >= size || count > size - (at - owner)) err_fail("invalid native heap span");
}
INLINE void tb_allocation(Env e, Loc at, Cls cls) {
  tb_span(e, at, 1);
  if (cls >= NCLS_ALL || (tb_heap_meta[at] & ~(TB_META_OWNED)) != tb_meta(at, cls))
    err_fail("native allocation class mismatch");
}
INLINE bool tb_cell_owned(Env e, Loc at) { tb_span(e, at, 1); return (tb_heap_meta[at] & TB_META_OWNED) != 0; }
INLINE void tb_mark_raw(Env e, Loc at, u64 count) {
  tb_span(e, at, count);
  for (u64 index = 0; index < count; ++index) tb_heap_meta[at + index] &= ~TB_META_OWNED;
}
INLINE void tb_mark_owned(Env e, Loc at, u64 count) {
  tb_span(e, at, count);
  for (u64 index = 0; index < count; ++index) tb_heap_meta[at + index] |= TB_META_OWNED;
}
INLINE void heap_free(Env e, Cls cls, Loc at) {
  tb_allocation(e, at, cls);
  memset(tb_heap_meta + at, 0, ((size_t)1 << cls) * sizeof(u64));
  tb_heap_meta[at] = tb_meta(at, cls) | TB_META_FREE;
  e.mem[at] = tb_free_lists[cls]; tb_free_lists[cls] = at;
  tb_live_words -= UINT64_C(1) << cls; --tb_live_blocks;
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
INLINE u64 rfc_view(Env e, Loc cell) {
  u64 value;
  tb_allocation(e, cell, 0); value = e.mem[cell];
  if ((value & RFC_CNT) == 0 || (value & RFC_CNT) == RFC_CNT) err_fail("invalid native reference count");
  tb_span(e, value >> 24, 1);
  return value;
}
INLINE void rfc_bump(Env e, Loc cell, u32 amount) {
  u64 value = rfc_view(e, cell);
  if (amount >= RFC_CNT - (u32)(value & RFC_CNT)) err_fail("native reference count overflow");
  e.mem[cell] = value + amount;
}
OUTLINE Term rfc_wrap(Env e, Term term, u32 count) {
  Loc cell;
  if (term_rfc(term) || (term_tag(term) != TAG_CTR && term_tag(term) != TAG_ARR && term_tag(term) != TAG_BUF))
    err_fail("native value cannot use a reference-count cell");
  if (count == 0 || count >= RFC_CNT) err_fail("native reference count overflow");
  tb_span(e, term_loc(term), 1);
  cell = heap_alloc(e, 0); e.mem[cell] = (term_loc(term) << 24) | count;
  tb_mark_raw(e, cell, 1);
  return (term & ~LOC_MASK) | RFC_BIT | cell;
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
/* Upstream closures remain affine count-cell values. The correctness-first
 * emitter duplicates a closure structurally, updating shared captures in its
 * original shell. An explicit heap stack also supports deep capture chains. */
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
      source = term_peek(e, value); tb_allocation(e, source, cls_fit(count));
      target = heap_alloc(e, cls_fit(count));
      *destination = term_clo(fid, target);
      frame = (TBDuplicateFrame *)io_mem(tb_host_calloc(1, sizeof(*frame)));
      frame->previous = stack; frame->source = source; frame->target = target; frame->count = count;
      stack = frame;
    } else { *owner = term_keep(e, value); *destination = *owner; }
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
      Loc cell = term_loc(term); u64 value = rfc_view(e, cell);
      if ((value & RFC_CNT) != 1) { e.mem[cell] = value - 1; term = 0; }
      else { term = (term & ~(RFC_BIT | LOC_MASK)) | (value >> 24); heap_free(e, 0, cell); }
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
      } else if (tag == TAG_TSK && aux == FID_CLO_APPLY) { count = 2; cls = 2; }
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
    Loc cell = term_loc(term); u64 value = rfc_view(e, cell);
    if ((value & RFC_CNT) != 1) {
      for (u32 index = 0; index < count; ++index)
        fields[index] = tb_cell_owned(e, at + index) ? tb_duplicate(e, e.mem + at + index) : e.mem[at + index];
      e.mem[cell] = value - 1;
      return 0;
    }
    heap_free(e, 0, cell);
  }
  memcpy(fields, e.mem + at, count * sizeof(Term));
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
  for (u32 index = 32; index != 0; --index) {
    Loc at = heap_alloc(e, 1);
    e.mem[at] = (value >> (index - 1)) & 1; e.mem[at + 1] = word;
    word = term_ctr(CID_WCON, at);
  }
  return word;
}

typedef Term (*BendClosureFn)(Env, const Term *, Term);
typedef struct TBCallFrame TBCallFrame;
typedef Term (*BendResumeFn)(const Env *, TBCallFrame *);
struct TBCallFrame {
  size_t pc, destination;
  bool waiting;
  Term *values;
  const Term *captures;
  Term argument;
  /* Dispatcher-owned metadata; generated bodies update only pc/destination/waiting. */
  TBCallFrame *parent;
  u32 fid;
  size_t slots;
};
static BendClosureFn tb_closure_functions[65536];
static BendResumeFn tb_resume_functions[65536];
static size_t tb_resume_slots[65536];
OUTLINE void tb_register_closure(u32 fid, BendClosureFn function, u32 count) {
  if (fid < 2 || fid >= 65536 || function == NULL || count > 255) err_fail("invalid closure registration");
  if (tb_closure_functions[fid] != NULL
      && (tb_closure_functions[fid] != function || tb_closure_captures[fid] != count))
    err_fail("duplicate closure registration");
  tb_closure_functions[fid] = function;
  tb_closure_captures[fid] = count;
}
OUTLINE void tb_register_generated(u32 fid, BendClosureFn function, BendResumeFn resume, u32 count, size_t slots) {
  if (fid < 2 || fid >= 65536 || resume == NULL || slots > SIZE_MAX / sizeof(Term))
    err_fail("invalid generated closure registration");
  if (tb_resume_functions[fid] != NULL
      && (tb_resume_functions[fid] != resume || tb_resume_slots[fid] != slots))
    err_fail("duplicate generated closure registration");
  tb_register_closure(fid, function, count);
  tb_resume_functions[fid] = resume;
  tb_resume_slots[fid] = slots;
}
INLINE Term tb_closure(Env e, u32 fid, u32 count, const Term *captures) {
  Loc at = 0;
  if (fid < 2 || fid >= 65536 || tb_closure_functions[fid] == NULL || count != tb_closure_captures[fid])
    err_fail("unregistered closure");
  if (count != 0) { at = heap_alloc(e, cls_fit(count)); memcpy(e.mem + at, captures, count * sizeof(Term)); }
  return term_clo(fid, at);
}
INLINE Loc task_node(Env e, Fid fid, Term continuation, u32 index, u32 remaining) {
  Loc at;
  if (continuation != TERM_HOLE || index != 0 || remaining != 0) err_fail("foreign task continuation is unsupported");
  if (fid != FID_CLO_APPLY) err_fail("foreign task id is unsupported");
  /* Upstream task_node stores arity arguments followed by continuation and
   * packed index/remaining words. Only ready root applications run here. */
  at = heap_alloc(e, 2);
  e.mem[at] = TERM_HOLE; e.mem[at + 1] = TERM_HOLE;
  e.mem[at + 2] = continuation; e.mem[at + 3] = 0;
  tb_mark_raw(e, at + 2, 2);
  return at;
}
INLINE void tb_task_take(Env e, Term task, Term *closure, Term *argument) {
  Loc at;
  if (term_tag(task) != TAG_TSK || term_aux(task) != FID_CLO_APPLY) err_fail("foreign task is unsupported");
  if (term_rfc(task)) err_fail("reference-counted foreign task is unsupported");
  at = term_loc(task); tb_allocation(e, at, 2);
  if (e.mem[at + 2] != TERM_HOLE || e.mem[at + 3] != 0)
    err_fail("foreign task continuation is unsupported");
  if (e.mem[at] == TERM_HOLE || e.mem[at + 1] == TERM_HOLE)
    err_fail("foreign task argument is missing");
  *closure = e.mem[at]; *argument = e.mem[at + 1]; heap_free(e, 2, at);
}
INLINE Term tb_tail_apply(Env e, Term closure, Term argument) {
  Loc at = task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0);
  e.mem[at] = closure; e.mem[at + 1] = argument;
  return term_tsk(FID_CLO_APPLY, at);
}
/* A suspended generated call owns its persistent buffers. Scratch slots are
 * aliases and raw layout words as well as Terms; generated ownership lowering
 * releases their live values, so freeing a frame must not sink every slot. */
OUTLINE TBCallFrame *tb_call_frame(Env e, Term closure, Term argument) {
  u32 fid = (u32)term_aux(closure), count = tb_closure_captures[fid];
  size_t slots = tb_resume_slots[fid];
  TBCallFrame *frame;
  Term *captures = NULL;
  if (tb_continuations == UINT32_MAX || tb_continuations >= BEND_MAX_CONTINUATIONS)
    err_fail("generated continuation budget exhausted");
  frame = (TBCallFrame *)io_mem(tb_host_calloc(1, sizeof(*frame)));
  frame->values = slots == 0 ? NULL : (Term *)io_mem(tb_host_calloc(slots, sizeof(Term)));
  if (count != 0) {
    Loc at = term_peek(e, closure); tb_allocation(e, at, cls_fit(count));
    captures = (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
    memcpy(captures, e.mem + at, count * sizeof(Term));
    heap_free(e, cls_fit(count), at);
  }
  frame->captures = captures; frame->argument = argument;
  frame->fid = fid; frame->slots = slots;
  ++tb_continuations;
  return frame;
}
OUTLINE void tb_call_frame_free(TBCallFrame *frame) {
  if (tb_continuations == 0) err_fail("unbalanced generated continuation");
  tb_host_free((void *)frame->captures);
  tb_host_free(frame->values);
  tb_host_free(frame);
  --tb_continuations;
}
OUTLINE Term tb_apply(Env e, Term closure, Term argument) {
  Term result;
  TBCallFrame *current = NULL, *pending = NULL;
  if (++tb_depth > BEND_MAX_DEPTH) err_fail("call depth budget exhausted");
  for (;;) {
    tb_tick();
    if (current != NULL) {
      if (current->waiting || current->fid >= 65536 || tb_resume_functions[current->fid] == NULL)
        err_fail("invalid generated continuation state");
      result = tb_resume_functions[current->fid](&e, current);
      if (current->waiting) {
        if (current->pc == 0 || current->destination >= current->slots)
          err_fail("invalid generated continuation destination");
        /* Taking the task transfers the child arguments. The caller remains
         * owned by this invocation until that child's final value arrives. */
        tb_task_take(e, result, &closure, &argument);
        current->parent = pending; pending = current; current = NULL;
        continue;
      }
      tb_call_frame_free(current); current = NULL;
    } else {
      u32 fid, count;
      Term *captures = NULL;
      if (term_tag(closure) != TAG_CLO) err_fail("application of a non-function");
      if (term_rfc(closure)) err_fail("reference-counted closure application is unsupported");
      fid = (u32)term_aux(closure);
      if (fid == FID_IO_EMIT) { term_sink(e, argument); result = term_pak(CID_EMIT, 0); }
      else {
        if (tb_closure_functions[fid] == NULL) err_fail("unregistered closure application");
        if (tb_resume_functions[fid] != NULL) {
          current = tb_call_frame(e, closure, argument);
          continue;
        }
        count = tb_closure_captures[fid];
        if (count != 0) {
          Loc at = term_peek(e, closure); tb_allocation(e, at, cls_fit(count));
          captures = (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
          memcpy(captures, e.mem + at, count * sizeof(Term));
          heap_free(e, cls_fit(count), at);
        }
        /* Legacy callbacks may enter tb_apply again. Their nested invocation
         * has its own pending chain and remains bounded by native call depth. */
        result = tb_closure_functions[fid](e, captures, argument);
        tb_host_free(captures);
      }
    }
    if (term_tag(result) == TAG_TSK) {
      tb_task_take(e, result, &closure, &argument);
      continue;
    }
    if (pending == NULL) break;
    current = pending; pending = current->parent; current->parent = NULL;
    if (!current->waiting || current->pc == 0 || current->destination >= current->slots)
      err_fail("invalid generated continuation destination");
    current->values[current->destination] = result;
    current->waiting = false;
  }
  --tb_depth;
  return result;
}
INLINE Term corpus_eval(Corpus memory, Term task) {
  Env e = {memory, NULL}; Term closure, argument;
  tb_task_take(e, task, &closure, &argument);
  return tb_apply(e, closure, argument);
}

INLINE Term term_blk(bool array, Cls cls, Loc loc) { return term_make(array ? TAG_ARR : TAG_BUF, cls, loc); }
INLINE Cls blk_cls(Term block) { return (Cls)term_aux(block); }
INLINE Cls buf_wcls(Cls cls) { return cls == 0 ? 0 : cls - 1; }
INLINE Cls blk_span(Term block) { return term_tag(block) == TAG_ARR ? blk_cls(block) : buf_wcls(blk_cls(block)); }
INLINE void blk_free(Env e, Term block) {
  if (term_rfc(block) || (term_tag(block) != TAG_ARR && term_tag(block) != TAG_BUF) || blk_cls(block) > 17)
    err_fail("unique array storage expected");
  heap_free(e, blk_span(block), term_loc(block));
}
INLINE Term blk_read(Corpus memory, bool array, Loc at, u32 index) {
  Env e = {memory, NULL};
  tb_span(e, at, (array ? (u64)index : (u64)index / 2) + 1);
  return array ? memory[at + index] : (u32)(memory[at + index / 2] >> ((index & 1) * 32));
}
INLINE void blk_write(Corpus memory, bool array, Loc at, u32 index, Term value) {
  Env e = {memory, NULL}; u32 shift = (index & 1) * 32;
  tb_span(e, at, (array ? (u64)index : (u64)index / 2) + 1);
  if (array) memory[at + index] = value;
  else memory[at + index / 2] = (memory[at + index / 2] & ~(UINT64_C(0xffffffff) << shift)) | ((u64)(u32)value << shift);
}
INLINE u32 blk_at(Term block, U32 index, u32 lgs) {
  Cls cls = blk_cls(block);
  if (cls < lgs || cls > 17) err_fail("invalid array layout");
  return ((u32)index & ((1u << (cls - lgs)) - 1)) << lgs;
}
INLINE Term blk_keep(Env e, Loc at) {
  return tb_cell_owned(e, at) ? tb_duplicate(e, e.mem + at) : e.mem[at];
}
INLINE Term tb_blk_new_masked(Env e, bool array, Nat depth, u32 lgs, u32 count, Term *values, const Term *box_mask) {
  Cls cls; Loc at; u32 stride;
  if (depth > 17 || lgs > 17 || depth + lgs > 17 || count > (1u << lgs)) err_fail("array budget exhausted");
  cls = (Cls)depth + lgs; stride = 1u << lgs;
  at = heap_alloc(e, array ? cls : buf_wcls(cls));
  for (u32 index = 0; index < (1u << cls); ++index) {
    u32 column = index % stride;
    bool owned = array && column < count && (box_mask == NULL || box_mask[column] != 0);
    Term value = 0;
    if (column < count) {
      value = owned && (index >> lgs) + 1 < (1u << (u32)depth)
          ? tb_duplicate(e, values + column) : values[column];
    }
    blk_write(e.mem, array, at, index, value);
    if (array && !owned) tb_mark_raw(e, at + index, 1);
  }
  return term_blk(array, cls, at);
}
INLINE Term blk_new(Env e, bool array, Nat depth, u32 lgs, u32 count, Term *values) {
  return tb_blk_new_masked(e, array, depth, lgs, count, values, NULL);
}
INLINE Term blk_copy(Env e, Term block) {
  Cls cls = blk_span(block); Loc at; Loc from = term_peek(e, block);
  bool array = term_tag(block) == TAG_ARR;
  if ((!array && term_tag(block) != TAG_BUF) || blk_cls(block) > 17) err_fail("array budget exhausted");
  tb_allocation(e, from, cls);
  at = heap_alloc(e, cls);
  for (u64 index = 0; index < (UINT64_C(1) << cls); ++index) {
    bool owned = array && tb_cell_owned(e, from + index);
    e.mem[at + index] = owned ? blk_keep(e, from + index) : e.mem[from + index];
    if (!owned) tb_mark_raw(e, at + index, 1);
  }
  return term_blk(term_tag(block) == TAG_ARR, blk_cls(block), at);
}
INLINE Term blk_unique(Env e, Term block) {
  Loc at = term_peek(e, block);
  if ((term_tag(block) != TAG_ARR && term_tag(block) != TAG_BUF) || blk_cls(block) > 17)
    err_fail("array expected");
  tb_allocation(e, at, blk_span(block));
  if (term_rfc(block)) {
    Loc cell = term_loc(block); u64 value = rfc_view(e, cell);
    if ((value & RFC_CNT) == 1) { heap_free(e, 0, cell); return (block & ~(RFC_BIT | LOC_MASK)) | at; }
    Term copy = blk_copy(e, block); term_drop(e, block); return copy;
  }
  return block;
}
INLINE Term blk_half(Env e, Term block, u32 high) {
  bool array = term_tag(block) == TAG_ARR; Cls cls = blk_cls(block); Loc at; Loc from = term_peek(e, block);
  if (term_rfc(block) || (!array && term_tag(block) != TAG_BUF) || cls == 0 || cls > 17 || high > 1)
    err_fail("invalid array half");
  tb_allocation(e, from, blk_span(block));
  --cls; at = heap_alloc(e, array ? cls : buf_wcls(cls));
  for (u32 index = 0; index < (1u << cls); ++index) {
    u32 source = (high << cls) + index;
    blk_write(e.mem, array, at, index, blk_read(e.mem, array, from, source));
    if (array && !tb_cell_owned(e, from + source)) tb_mark_raw(e, at + index, 1);
  }
  if (high != 0) blk_free(e, block);
  return term_blk(array, cls, at);
}
INLINE Term blk_node(Env e, Term left, Term right) {
  bool array = term_tag(left) == TAG_ARR; Cls cls = blk_cls(left); Loc at, l, r;
  if (term_rfc(left) || term_rfc(right) || (!array && term_tag(left) != TAG_BUF)
      || cls != blk_cls(right) || term_tag(left) != term_tag(right) || cls >= 17) err_fail("invalid array concatenation");
  l = term_peek(e, left); r = term_peek(e, right);
  tb_allocation(e, l, blk_span(left)); tb_allocation(e, r, blk_span(right));
  if (l == r) err_fail("array concatenation aliases unique storage");
  at = heap_alloc(e, array ? cls + 1 : buf_wcls(cls + 1));
  for (u32 index = 0; index < (1u << cls); ++index) {
    blk_write(e.mem, array, at, index, blk_read(e.mem, array, l, index));
    blk_write(e.mem, array, at, (1u << cls) + index, blk_read(e.mem, array, r, index));
    if (array && !tb_cell_owned(e, l + index)) tb_mark_raw(e, at + index, 1);
    if (array && !tb_cell_owned(e, r + index)) tb_mark_raw(e, at + (UINT64_C(1) << cls) + index, 1);
  }
  blk_free(e, left); blk_free(e, right);
  return term_blk(array, cls + 1, at);
}

/* Implemented by executable_io.c after this core. */
static void tb_io_initialize(void);
static void tb_io_shutdown(void);
static int io_loop(Env e, Term main);
OUTLINE int tb_run(Term (*entry)(Env), int is_io, void (*show)(Env, Term), void (*initialize)(Env)) {
  TBHost *host = (TBHost *)calloc(1, sizeof(TBHost));
  jmp_buf guard;
  volatile int result = 1;
  Env e;
  if (host == NULL) { (void)fprintf(stderr, "host initialization failed\n"); return 1; }
  host->wake_read = -1; host->wake_write = -1;
#ifdef _WIN32
  InitializeSRWLock(&host->mutex);
#else
  if (pthread_mutex_init(&host->mutex, NULL) != 0) { free(host); return 1; }
#endif
  host->references = 1;
  tb_host_current = host; tb_failure_guard = &guard;
  tb_capacity = BEND_MAX_ALLOC / sizeof(Term);
  tb_memory = NULL; tb_heap_meta = NULL;
  e.mem = tb_memory; e.alc = NULL;
  tb_bump = HEAP_OFF; tb_steps = 0; tb_depth = 0; tb_frames = 0; tb_continuations = 0;
  tb_live_words = 0; tb_live_blocks = 0;
  memset(tb_free_lists, 0, sizeof(tb_free_lists));
  memset(tb_closure_functions, 0, sizeof(tb_closure_functions));
  memset(tb_closure_captures, 0, sizeof(tb_closure_captures));
  memset(tb_resume_functions, 0, sizeof(tb_resume_functions));
  memset(tb_resume_slots, 0, sizeof(tb_resume_slots));
  if (setjmp(guard) == 0) {
    Term main;
    if (tb_capacity <= HEAP_OFF || tb_capacity > LOC_MASK || tb_capacity > SIZE_MAX / sizeof(Term))
      err_fail("VM initialization failed");
    tb_memory = (Corpus)calloc((size_t)tb_capacity, sizeof(Term));
    tb_heap_meta = (u64 *)calloc((size_t)tb_capacity, sizeof(u64));
    e.mem = tb_memory;
    if (tb_memory == NULL || tb_heap_meta == NULL) err_fail("VM initialization failed");
    tb_io_initialize();
    if (initialize != NULL) initialize(e);
    if (entry == NULL) { (void)fputs("All terms check.\n", stdout); result = 0; }
    else {
      main = entry(e);
      if (is_io) result = io_loop(e, main);
      else { if (show != NULL) show(e, main); term_sink(e, main); result = 0; }
    }
    if (tb_continuations != 0) { result = 1; err_fail("generated continuation ownership leaked"); }
  }
  tb_io_shutdown();
  free(tb_memory); tb_memory = NULL;
  free(tb_heap_meta); tb_heap_meta = NULL;
  tb_failure_guard = NULL; tb_host_current = NULL;
  tb_host_release(host);
  return result;
}
