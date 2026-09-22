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
#ifndef BEND_MAX_TASKS
#define BEND_MAX_TASKS 65536u
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
static TB_THREAD_LOCAL char tb_failure_message[256];
static TB_THREAD_LOCAL bool tb_worker_failure_capture;
static TB_THREAD_LOCAL bool tb_vm_held;
static TBMutex tb_vm_mutex = TB_MUTEX_INIT;
static bool tb_vm_cancelled;
static bool tb_cpu_active;
static Corpus tb_memory;
static u64 *tb_heap_meta;
static Loc tb_free_lists[NCLS_ALL];
static Loc tb_bump;
static Loc tb_capacity;
static u64 tb_live_words;
static u64 tb_live_blocks;
static u64 tb_steps;
static TB_THREAD_LOCAL u32 tb_depth;
static TB_THREAD_LOCAL u32 tb_frames;
static u32 tb_continuations;
static u32 tb_tasks;
static u64 tb_task_joins;
static u32 tb_task_peak;
static u64 tb_segment_calls;
static u64 tb_segment_result_words;
static u64 tb_segment_multiword_results;
typedef struct TBTaskRecord TBTaskRecord;
static TBTaskRecord *tb_task_table[1024];
static u32 tb_closure_captures[65536];
static bool tb_parallel_functions[65536];
INLINE u32 fid_arity(Fid fid);
static void tb_files_release(TBHost *host);
static void tb_network_shutdown(TBHost *host);
static void tb_task_context_reset(void);
static void tb_cpu_initialize(void);
static void tb_cpu_shutdown(void);
#ifdef TB_GPU_ENABLED
static void tb_gpu_shutdown(void);
#endif
static TB_NORETURN void tb_cpu_cancel_error(void);

OUTLINE TB_NORETURN void err_fail(const char *message) {
  /* No longjmp may strand the VM lock. The lock protects only short core
   * operations; generated bodies and callbacks never execute while held. */
  if (tb_vm_held) { tb_vm_held = false; tb_unlock(&tb_vm_mutex); }
  if (tb_worker_failure_capture) {
    (void)snprintf(tb_failure_message, sizeof(tb_failure_message), "%s", message);
    if (tb_failure_guard != NULL) longjmp(*tb_failure_guard, 1);
    abort();
  }
#ifdef TB_GPU_ENABLED
  /* The coordinator owns CUDA. Release its stream and buffers before a
   * guarded error abandons any host transfer storage. Workers never enter it. */
  tb_gpu_shutdown();
#endif
  if (tb_host_current != NULL) {
    tb_lock(&tb_host_current->mutex);
    tb_host_current->worker_failed = true;
    tb_unlock(&tb_host_current->mutex);
  }
  (void)fprintf(stderr, "teamy-bend executable C: %s\n", message);
  if (tb_failure_guard != NULL) longjmp(*tb_failure_guard, 1);
  exit(1);
}
INLINE void tb_vm_acquire(void) {
  if (tb_vm_held) err_fail("recursive VM lock acquisition");
  tb_lock(&tb_vm_mutex); tb_vm_held = true;
}
INLINE void tb_vm_release(void) { tb_vm_held = false; tb_unlock(&tb_vm_mutex); }
INLINE void tb_vm_cancel(void) {
  tb_vm_acquire(); tb_vm_cancelled = true; tb_vm_release();
}
INLINE void tb_counter_add(u64 *counter, u64 amount, const char *message) {
  tb_vm_acquire();
  if (*counter > UINT64_MAX - amount) err_fail(message);
  *counter += amount;
  tb_vm_release();
}
INLINE void tb_tick(void) {
  tb_vm_acquire();
  if (tb_vm_cancelled) { tb_vm_release(); tb_cpu_cancel_error(); }
  if (tb_steps >= BEND_MAX_STEPS || tb_steps == UINT64_MAX) err_fail("evaluation budget exhausted");
  ++tb_steps;
  tb_vm_release();
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
/* TB_SHARED_VALUES */
typedef Term (*BendClosureFn)(Env, const Term *, Term);
typedef struct TBCallFrame TBCallFrame;
typedef Term (*BendResumeFn)(const Env *, TBCallFrame *);
enum { TB_OUTCOME_WORDS = 0, TB_OUTCOME_TASK = 1, TB_OUTCOME_CALL = 2 };
typedef struct {
  Term task;
  const Term *words;
  const Term *owned;
  u32 count;
  u32 pending;
} TBOutcome;
typedef TBOutcome (*BendSegmentFn)(const Env *, TBCallFrame *);
INLINE TBOutcome tb_segment_task(Term task) {
  TBOutcome outcome = {task, NULL, NULL, 0, TB_OUTCOME_TASK}; return outcome;
}
INLINE TBOutcome tb_segment_words(const Term *words, const Term *owned, u32 count) {
  TBOutcome outcome = {0, words, owned, count, TB_OUTCOME_WORDS}; return outcome;
}
/* A direct call borrows its argument span until the dispatcher copies it.
 * Only this outcome stores a plain function ID in the task field. */
INLINE TBOutcome tb_segment_call(Fid fid, u32 count, const Term *words, const Term *owned) {
  TBOutcome outcome = {fid, words, owned, count, TB_OUTCOME_CALL}; return outcome;
}
/* Scalar representation changes compose without retaining identity frames.
 * Bit zero enables an adapter, bit one selects an owned result, and bit two
 * preserves any required native W32 truncation across the tail chain. */
enum {
  TB_RESULT_NONE = 0, TB_RESULT_RAW64 = 1, TB_RESULT_BOX64 = 3,
  TB_RESULT_RAW32 = 5, TB_RESULT_BOX32 = 7
};
struct TBCallFrame {
  size_t pc, destination;
  u32 expected, tail_result;
  bool waiting;
  Term *values;
  const Term *captures;
  Term argument;
  /* Dispatcher-owned metadata; generated bodies update only the fields above. */
  TBCallFrame *parent;
  u32 fid, saved_result;
  size_t slots;
};
INLINE u32 tb_result_compose(u32 outer, u32 inner) {
  if ((outer != 0 && ((outer & 1u) == 0 || outer > 7))
      || (inner != 0 && ((inner & 1u) == 0 || inner > 7)))
    err_fail("invalid scalar tail result adapter");
  return outer == 0 ? inner : (outer & 3u) | ((outer | inner) & 4u);
}
INLINE Term tb_tail_result(TBCallFrame *frame, Term task, u32 mode) {
  if (frame == NULL || frame->waiting || mode == TB_RESULT_NONE)
    err_fail("invalid scalar tail result adapter");
  frame->tail_result = tb_result_compose(frame->tail_result, mode);
  return task;
}
static BendClosureFn tb_closure_functions[65536];
static BendResumeFn tb_resume_functions[65536];
static size_t tb_resume_slots[65536];
static BendSegmentFn tb_segment_functions[65536];
static u32 tb_segment_arities[65536], tb_segment_widths[65536];
INLINE u32 fid_arity(Fid fid) {
  if (fid == FID_CLO_APPLY) return 2;
  if (fid == FID_IO_EMIT) return 1;
  if (fid >= 65536) err_fail("foreign task id is unsupported");
  if (tb_segment_functions[fid] != NULL) return tb_segment_arities[fid];
  if (tb_closure_functions[fid] == NULL) err_fail("foreign task id is unsupported");
  if (tb_closure_captures[fid] >= 255) err_fail("foreign task arity is unsupported");
  return tb_closure_captures[fid] + 1;
}
INLINE u32 fid_result_width(Fid fid) {
  if (fid == FID_CLO_APPLY || fid == FID_IO_EMIT) return 1;
  if (fid >= 65536 || (tb_segment_functions[fid] == NULL && tb_closure_functions[fid] == NULL))
    err_fail("foreign task id is unsupported");
  return tb_segment_functions[fid] == NULL ? 1 : tb_segment_widths[fid];
}
INLINE void tb_registry_guard(void) {
  if (tb_cpu_active || tb_worker_failure_capture) err_fail("registration during CPU execution");
}
INLINE void tb_register_closure_locked(u32 fid, BendClosureFn function, u32 count) {
  tb_registry_guard();
  if (fid < 2 || fid >= 65536 || function == NULL || count > 255) err_fail("invalid closure registration");
  if (tb_segment_functions[fid] != NULL) err_fail("conflicting task registration");
  if (tb_closure_functions[fid] != NULL
      && (tb_closure_functions[fid] != function || tb_closure_captures[fid] != count))
    err_fail("duplicate closure registration");
  tb_closure_functions[fid] = function;
  tb_closure_captures[fid] = count;
  tb_parallel_functions[fid] = false;
}
OUTLINE void tb_register_closure(u32 fid, BendClosureFn function, u32 count) {
  tb_vm_acquire(); tb_register_closure_locked(fid, function, count); tb_vm_release();
}
OUTLINE void tb_register_generated(u32 fid, BendClosureFn function, BendResumeFn resume, u32 count, size_t slots) {
  tb_vm_acquire(); tb_registry_guard();
  if (fid < 2 || fid >= 65536 || resume == NULL || slots > SIZE_MAX / sizeof(Term))
    err_fail("invalid generated closure registration");
  if (tb_resume_functions[fid] != NULL
      && (tb_resume_functions[fid] != resume || tb_resume_slots[fid] != slots))
    err_fail("duplicate generated closure registration");
  tb_register_closure_locked(fid, function, count);
  tb_resume_functions[fid] = resume;
  tb_resume_slots[fid] = slots;
  tb_vm_release();
}
OUTLINE void tb_register_segment(u32 fid, BendSegmentFn function, u32 arity, u32 result_width, size_t slots) {
  tb_vm_acquire(); tb_registry_guard();
  if (fid < 2 || fid >= 65536 || function == NULL || arity > 255 || result_width == 0
      || result_width > 255 || slots > SIZE_MAX / sizeof(Term))
    err_fail("invalid segment registration");
  if (tb_closure_functions[fid] != NULL) err_fail("conflicting task registration");
  if (tb_segment_functions[fid] != NULL
      && (tb_segment_functions[fid] != function || tb_segment_arities[fid] != arity
          || tb_segment_widths[fid] != result_width || tb_resume_slots[fid] != slots))
    err_fail("duplicate segment registration");
  tb_segment_functions[fid] = function; tb_segment_arities[fid] = arity;
  tb_segment_widths[fid] = result_width; tb_resume_slots[fid] = slots;
  tb_parallel_functions[fid] = false;
  tb_vm_release();
}
OUTLINE void tb_register_parallel(u32 fid) {
  tb_vm_acquire(); tb_registry_guard();
  if (fid < 2 || fid >= 65536 || (tb_resume_functions[fid] == NULL && tb_segment_functions[fid] == NULL))
    err_fail("invalid parallel registration");
  tb_parallel_functions[fid] = true;
  tb_vm_release();
}
INLINE Term tb_closure(Env e, u32 fid, u32 count, const Term *captures) {
  Loc at = 0;
  if (fid < 2 || fid >= 65536 || tb_closure_functions[fid] == NULL || count != tb_closure_captures[fid])
    err_fail("unregistered closure");
  if (count != 0) { at = heap_alloc(e, cls_fit(count)); memcpy(e.mem + at, captures, count * sizeof(Term)); }
  return term_clo(fid, at);
}
INLINE Loc task_node(Env e, Fid fid, Term continuation, u32 index, u32 remaining) {
  u32 arity = fid_arity(fid);
  Loc at;
  if ((continuation == TERM_HOLE && index != 0)
      || (continuation != TERM_HOLE && (term_tag(continuation) != TAG_TSK || term_rfc(continuation))))
    err_fail("foreign task continuation is unsupported");
  if (remaining > arity) err_fail("invalid task dependency count");
  at = heap_alloc(e, cls_fit(arity + 2));
  /* Ready payloads are initialized too, so malformed foreign construction
   * cannot accidentally turn recycled words into owned arguments. */
  for (u32 i = 0; i < arity; ++i) e.mem[at + i] = TERM_HOLE;
  e.mem[at + arity] = continuation;
  e.mem[at + arity + 1] = ((u64)index << 32) | remaining;
  tb_mark_raw(e, at + arity, 2);
  return at;
}
INLINE Loc task_tail(Term task) {
  Env e = {tb_memory, NULL};
  if (term_tag(task) != TAG_TSK) err_fail("foreign task is unsupported");
  if (term_rfc(task)) err_fail("reference-counted foreign task is unsupported");
  u32 arity = fid_arity((Fid)term_aux(task));
  Loc at = term_loc(task);
  tb_allocation(e, at, cls_fit(arity + 2));
  return at + arity;
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
OUTLINE TBCallFrame *tb_call_frame_owned(u32 fid, Term *captures, Term argument) {
  size_t slots = tb_resume_slots[fid];
  TBCallFrame *frame;
  tb_vm_acquire();
  if (tb_continuations == UINT32_MAX || tb_continuations >= BEND_MAX_CONTINUATIONS)
    err_fail("generated continuation budget exhausted");
  ++tb_continuations;
  tb_vm_release();
  frame = (TBCallFrame *)io_mem(tb_host_calloc(1, sizeof(*frame)));
  frame->values = slots == 0 ? NULL : (Term *)io_mem(tb_host_calloc(slots, sizeof(Term)));
  frame->captures = captures; frame->argument = argument;
  frame->fid = fid; frame->slots = slots; frame->expected = 1;
  return frame;
}
OUTLINE TBCallFrame *tb_call_frame(Env e, Term closure, Term argument) {
  u32 fid = (u32)term_aux(closure), count = tb_closure_captures[fid];
  Term *captures = NULL;
  if (count != 0) {
    Loc at = term_peek(e, closure); tb_allocation(e, at, cls_fit(count));
    captures = (Term *)io_mem(tb_host_malloc(count * sizeof(Term)));
    memcpy(captures, e.mem + at, count * sizeof(Term));
    heap_free(e, cls_fit(count), at);
  }
  return tb_call_frame_owned(fid, captures, argument);
}
OUTLINE void tb_call_frame_free(TBCallFrame *frame) {
  tb_vm_acquire();
  if (tb_continuations == 0) err_fail("unbalanced generated continuation");
  --tb_continuations;
  tb_vm_release();
  tb_host_free((void *)frame->captures);
  tb_host_free(frame->values);
  tb_host_free(frame);
}
OUTLINE Term tb_apply(Env e, Term closure, Term argument);
OUTLINE Term corpus_eval(Corpus memory, Term task);
OUTLINE u32 corpus_eval_words(Corpus memory, Term task, Term *words, Term *owned, u32 capacity);

/* TB_SHARED_ARRAYS */
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
  tb_failure_message[0] = '\0'; tb_worker_failure_capture = false;
  tb_vm_held = false; tb_vm_cancelled = false; tb_cpu_active = false;
  tb_capacity = BEND_MAX_ALLOC / sizeof(Term);
  tb_memory = NULL; tb_heap_meta = NULL;
  e.mem = tb_memory; e.alc = NULL;
  tb_bump = HEAP_OFF; tb_steps = 0; tb_depth = 0; tb_frames = 0; tb_continuations = 0; tb_tasks = 0;
  tb_task_joins = 0; tb_task_peak = 0; tb_task_context_reset();
  tb_segment_calls = 0; tb_segment_result_words = 0; tb_segment_multiword_results = 0;
  tb_live_words = 0; tb_live_blocks = 0;
  memset(tb_free_lists, 0, sizeof(tb_free_lists));
  memset(tb_closure_functions, 0, sizeof(tb_closure_functions));
  memset(tb_closure_captures, 0, sizeof(tb_closure_captures));
  memset(tb_parallel_functions, 0, sizeof(tb_parallel_functions));
  memset(tb_resume_functions, 0, sizeof(tb_resume_functions));
  memset(tb_resume_slots, 0, sizeof(tb_resume_slots));
  memset(tb_segment_functions, 0, sizeof(tb_segment_functions));
  memset(tb_segment_arities, 0, sizeof(tb_segment_arities));
  memset(tb_segment_widths, 0, sizeof(tb_segment_widths));
  memset(tb_task_table, 0, sizeof(tb_task_table));
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
    tb_cpu_initialize();
    if (entry == NULL) { (void)fputs("All terms check.\n", stdout); result = 0; }
    else {
      main = entry(e);
      if (is_io) result = io_loop(e, main);
      else { if (show != NULL) show(e, main); term_sink(e, main); result = 0; }
    }
    if (tb_continuations != 0) { result = 1; err_fail("generated continuation ownership leaked"); }
    if (tb_tasks != 0) { result = 1; err_fail("task ownership leaked"); }
  }
  /* The coordinator joins every CPU worker, including partially initialized
   * pools and guarded failures, before any corpus or host storage is released. */
  tb_cpu_shutdown();
#ifdef TB_GPU_ENABLED
  tb_gpu_shutdown();
#endif
  tb_io_shutdown();
  free(tb_memory); tb_memory = NULL;
  free(tb_heap_meta); tb_heap_meta = NULL;
  tb_failure_guard = NULL; tb_host_current = NULL;
  tb_task_context_reset();
  tb_host_release(host);
  return result;
}
