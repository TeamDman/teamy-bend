/* SPDX-License-Identifier: MPL-2.0
 * Exclusive CPU/CUDA task handoff. A ready task is detached from its host
 * destination before entering here. Corpus words, exact ownership metadata,
 * and allocator state cross together; host frames stay parked on the CPU.
 * The CUDA module, stream and buffers persist between marked calls. */
#ifndef BEND_MAX_GPU_SCRATCH
#define BEND_MAX_GPU_SCRATCH (BEND_MAX_ALLOC / 2)
#endif
#ifndef BEND_GPU_BLOCKS
#define BEND_GPU_BLOCKS 8u
#endif
#ifndef BEND_GPU_THREADS
#define BEND_GPU_THREADS 128u
#endif
#ifndef BEND_GPU_QUANTUM
#define BEND_GPU_QUANTUM 64u
#endif
#ifndef BEND_GPU_PRIMITIVE_QUANTUM
#define BEND_GPU_PRIMITIVE_QUANTUM 1024u
#endif
#ifndef TB_GPU_COMPLETE
#define TB_GPU_COMPLETE(control, state, info) ((void)0)
#endif
#ifndef TB_GPU_OBSERVE
#define TB_GPU_OBSERVE(state, control) ((void)0)
#endif

enum { TB_GPU_CORPUS, TB_GPU_METADATA, TB_GPU_STATE, TB_GPU_SCRATCH, TB_GPU_CONTROL };
static int tb_gpu_status; /* 0 untried, 1 initialized, -1 unavailable. */
static const char *tb_gpu_policy_override;
static const char *tb_gpu_policy(void) {
  return tb_gpu_policy_override != NULL ? tb_gpu_policy_override : getenv("BEND_GPU");
}

/* This read-only gate is safe at CPU call boundaries: CUDA status changes
 * only after the coordinator drains workers. Never initialize CUDA here.
 * Preserve ordinary ready-call reuse when a mark cannot request offload. */
static bool tb_gpu_cpu_only(void) {
  const char *policy = tb_gpu_policy();
  if (policy != NULL && (strcmp(policy, "off") == 0 || strcmp(policy, "0") == 0)) return true;
  return tb_gpu_status < 0 && (policy == NULL || strcmp(policy, "auto") == 0);
}

static void tb_gpu_shutdown(void) {
  tb_cuda_shutdown();
  tb_gpu_status = 0;
}
static TB_NORETURN void tb_gpu_fail(const char *message) {
  char saved[2048];
  (void)snprintf(saved, sizeof(saved), "%s", message);
  tb_gpu_shutdown();
  err_fail(saved);
}
static void tb_gpu_require(bool success) {
  if (!success) tb_gpu_fail(tb_cuda_error());
}
/* This preparation also runs before the host VM exists during --gpu-build.
 * Use ordinary owned temporary storage and return errors without longjmp. */
static bool tb_gpu_prepare(bool prebuild) {
  static const char *const required[] = {
    "tb_device_initialize", "tb_device_tables_initialize", "tb_device_tasks_begin",
    "tb_device_tasks_commit", "tb_device_tasks_step"
  };
  /* Include the slice size in the compiled source and therefore the persistent
   * cache identity. Host compiler definitions do not otherwise reach NVRTC. */
  u64 primitive_quantum = BEND_GPU_PRIMITIVE_QUANTUM;
  if (primitive_quantum == 0 || primitive_quantum > 4096)
    return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "invalid CUDA primitive quantum");
  char prefix[80];
  int prefix_length = snprintf(prefix, sizeof(prefix),
    "#define BEND_GPU_PRIMITIVE_QUANTUM %u\n", (u32)primitive_quantum);
  if (prefix_length < 0 || (size_t)prefix_length >= sizeof(prefix))
    return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA source prefix is too large");
  size_t length = (size_t)prefix_length, at = length;
  for (size_t i = 0; tb_gpu_source_parts[i] != NULL; ++i) {
    size_t count = strlen(tb_gpu_source_parts[i]);
    if (length == SIZE_MAX || count > SIZE_MAX - length - 1)
      return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA generated source is too large");
    length += count;
  }
  char *source = (char *)malloc(length + 1);
  if (source == NULL) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA source allocation failed");
  memcpy(source, prefix, at);
  for (size_t i = 0; tb_gpu_source_parts[i] != NULL; ++i) {
    size_t count = strlen(tb_gpu_source_parts[i]);
    memcpy(source + at, tb_gpu_source_parts[i], count); at += count;
  }
  source[at] = 0;
  bool ready = tb_cuda_initialize_kernels(source, prebuild, required,
    sizeof(required) / sizeof(required[0]));
  free(source);
  if (tb_cuda_error_class() == TB_CUDA_ERROR_UNAVAILABLE) { tb_gpu_status = -1; return true; }
  if (ready) tb_gpu_status = 1;
  return ready;
}
/* Only explicit disable or unavailability before execution permits fallback.
 * A compilation, allocation, transfer or execution failure is never replayed. */
static bool tb_gpu_initialize(void) {
  const char *policy = tb_gpu_policy();
  bool forced = false;
  if (policy == NULL || strcmp(policy, "auto") == 0) { }
  else if (strcmp(policy, "off") == 0 || strcmp(policy, "0") == 0) return false;
  else if (strcmp(policy, "on") == 0 || strcmp(policy, "forced") == 0 || strcmp(policy, "1") == 0)
    forced = true;
  else tb_gpu_fail("BEND_GPU must be auto, off, or on");
  if (tb_gpu_status == 0) {
    if (!tb_gpu_prepare(false)) tb_gpu_fail(tb_cuda_error());
  }
  if (tb_gpu_status < 0 && forced) tb_gpu_fail(tb_cuda_error());
  return tb_gpu_status > 0;
}

static TBOutcome tb_gpu_execute(Env e, Term root) {
  TBDeviceState state;
  struct TBDeviceControl control;
  const u64 initial_bump = tb_bump, initial_steps = tb_steps;
  const u64 scratch_words = BEND_MAX_GPU_SCRATCH / sizeof(u64);
  const u64 helper_lanes = (u64)BEND_GPU_BLOCKS * (u64)BEND_GPU_THREADS;
  u32 quantum = BEND_GPU_QUANTUM;
  u64 state_address, corpus_address, metadata_address, scratch_address, control_address;
  if (tb_worker_failure_capture || tb_cpu_active || tb_cpu_pending != 0 || e.mem != tb_memory)
    tb_gpu_fail("CUDA requires an idle CPU task pool");
  Loc tail = task_tail(root);
  if (e.mem[tail] != TERM_HOLE || e.mem[tail + 1] != 0)
    tb_gpu_fail("CUDA requires a detached ready task");
  if (!tb_gpu_initialize()) return tb_segment_task(root);
  if (scratch_words < 4 || scratch_words > SIZE_MAX / sizeof(u64)
      || quantum == 0 || quantum > 1024 || BEND_GPU_BLOCKS == 0 || BEND_GPU_THREADS == 0
      || BEND_GPU_BLOCKS > (u32)tb_cuda_info()->max_grid
      || BEND_GPU_THREADS > (u32)tb_cuda_info()->max_block
      || helper_lanes > UINT32_MAX || helper_lanes > SIZE_MAX / sizeof(u32)
      || (u64)BEND_MAX_FRAMES > UINT32_MAX)
    tb_gpu_fail("invalid CUDA scheduling budget");
  if (tb_tasks > BEND_MAX_TASKS || tb_continuations >= BEND_MAX_CONTINUATIONS)
    tb_gpu_fail("CUDA task or continuation budget exhausted");

  memset(&state, 0, sizeof(state)); memset(&control, 0, sizeof(control));
  state.capacity = tb_capacity; state.bump = tb_bump;
  state.live_words = tb_live_words; state.live_blocks = tb_live_blocks;
  state.steps = tb_steps; state.step_limit = BEND_MAX_STEPS;
  memcpy(state.free_lists, tb_free_lists, sizeof(tb_free_lists));
  state.scratch_capacity = scratch_words; state.scratch_bump = 2;
  /* The device root replaces one parked host record. Other host records and
   * frames remain charged while the device graph is active. */
  control.task_limit = BEND_MAX_TASKS - tb_tasks + 1;
  control.run_limit = control.task_limit;
  control.frame_limit = BEND_MAX_CONTINUATIONS - tb_continuations;
  /* Synchronous helper depth is per device lane, like the CPU thread-local
   * helper limit. Persistent continuations have separate shared accounting. */
  control.helper_lanes = (u32)helper_lanes; control.helper_limit = BEND_MAX_FRAMES;
  size_t corpus_bytes = (size_t)tb_capacity * sizeof(Term);
  size_t used_bytes = (size_t)tb_bump * sizeof(Term);
  tb_gpu_require(tb_cuda_reserve(TB_GPU_CORPUS, corpus_bytes));
  tb_gpu_require(tb_cuda_reserve(TB_GPU_METADATA, corpus_bytes));
  tb_gpu_require(tb_cuda_reserve(TB_GPU_STATE, sizeof(state)));
  tb_gpu_require(tb_cuda_reserve(TB_GPU_SCRATCH, (size_t)scratch_words * sizeof(u64)));
  tb_gpu_require(tb_cuda_reserve(TB_GPU_CONTROL, sizeof(control)));
  tb_gpu_require(tb_cuda_upload(TB_GPU_CORPUS, 0, e.mem, used_bytes));
  tb_gpu_require(tb_cuda_upload(TB_GPU_METADATA, 0, tb_heap_meta, used_bytes));
  tb_gpu_require(tb_cuda_upload(TB_GPU_STATE, 0, &state, sizeof(state)));
  tb_gpu_require(tb_cuda_upload(TB_GPU_CONTROL, 0, &control, sizeof(control)));
  state_address = tb_cuda_address(TB_GPU_STATE); corpus_address = tb_cuda_address(TB_GPU_CORPUS);
  metadata_address = tb_cuda_address(TB_GPU_METADATA); scratch_address = tb_cuda_address(TB_GPU_SCRATCH);
  control_address = tb_cuda_address(TB_GPU_CONTROL);
  void *initialize[] = {&state_address, &corpus_address, &metadata_address, &scratch_address};
  void *begin[] = {&control_address, &root};
  void *advance[] = {&control_address, &quantum};
  tb_gpu_require(tb_cuda_launch("tb_device_initialize", 1, 1, initialize));
  tb_gpu_require(tb_cuda_launch("tb_device_tables_initialize", 1, 1, NULL));
  tb_gpu_require(tb_cuda_launch("tb_device_tasks_begin", 1, 1, begin));
  tb_gpu_require(tb_cuda_launch("tb_device_tasks_commit", 1, 1, advance));
  /* Each launch executes bounded segment/primitive work. Primitive slices and
   * their completed-to-ready transfers are progress, not new language steps.
   * Retain frames and queues while requiring an actual advance each round. */
  u64 previous_steps = initial_steps, previous_progress = 0, previous_requeues = 0;
  u64 previous_starts = 0, previous_yields = 0;
  for (;;) {
    tb_gpu_require(tb_cuda_launch("tb_device_tasks_step", BEND_GPU_BLOCKS, BEND_GPU_THREADS, advance));
    tb_gpu_require(tb_cuda_launch("tb_device_tasks_commit", 1, 1, advance));
    tb_gpu_require(tb_cuda_synchronize());
    tb_gpu_require(tb_cuda_download(TB_GPU_STATE, 0, &state, sizeof(state)));
    tb_gpu_require(tb_cuda_download(TB_GPU_CONTROL, 0, &control, sizeof(control)));
    if (state.error != 0) {
      state.error_text[sizeof(state.error_text) - 1] = 0;
      (void)tb_cuda_message(TB_CUDA_ERROR_RUNTIME,
        state.error_text[0] == 0 ? "CUDA task execution failed" : state.error_text);
      tb_gpu_fail(tb_cuda_error());
    }
    if (state.steps < previous_steps || state.steps > BEND_MAX_STEPS
        || state.step_limit != BEND_MAX_STEPS
        || control.primitive_progress < previous_progress
        || control.primitive_requeues < previous_requeues
        || control.primitive_starts < previous_starts || control.primitive_yields < previous_yields
        || control.primitive_requeues > control.primitive_yields
        || control.primitive_yields > control.primitive_progress
        || control.primitive_live > control.primitive_starts)
      tb_gpu_fail("invalid CUDA progress boundary");
    if (control.done != 0) break;
    if (state.steps == previous_steps && control.primitive_progress == previous_progress
        && control.primitive_requeues == previous_requeues)
      tb_gpu_fail("CUDA task graph made no progress");
    previous_steps = state.steps; previous_progress = control.primitive_progress;
    previous_requeues = control.primitive_requeues; previous_starts = control.primitive_starts;
    previous_yields = control.primitive_yields;
    if (control.ready_count == 0 && control.completed_count == 0 && control.adopt_head == 0)
      tb_gpu_fail("incomplete CUDA task graph");
  }
  if (state.capacity != tb_capacity || state.bump < initial_bump || state.bump > tb_capacity
      || state.live_words > state.bump || state.live_blocks > state.live_words
      || state.steps < initial_steps || state.steps > BEND_MAX_STEPS
      || state.step_limit != BEND_MAX_STEPS || state.lock != 0 || state.active_lanes != 0
      || state.scratch_capacity != scratch_words || state.scratch_bump < 2
      || state.scratch_bump > scratch_words || state.scratch_live != 0
      || control.done != 1 || control.live_runs != 0 || control.live_records != 0
      || control.live_frames != 0 || control.ready_count != 0 || control.completed_count != 0
      || control.helper_depths != 0 || control.helper_live != 0 || control.primitive_live != 0
      || control.helper_lanes != helper_lanes || control.helper_limit != BEND_MAX_FRAMES
      || control.helper_peak > control.helper_limit
      || control.adopt_head != 0 || control.lock != 0 || control.active_lanes != 0
      || control.root_count != fid_result_width((Fid)term_aux(root)))
    tb_gpu_fail("invalid CUDA completion boundary");
  for (u32 i = 0; i < NCLS_ALL; ++i)
    if (state.free_lists[i] != 0 && (state.free_lists[i] < HEAP_OFF || state.free_lists[i] >= state.bump))
      tb_gpu_fail("invalid CUDA allocator state");
  for (u32 i = 0; i < control.root_count; ++i) {
    if (control.root_owned[i] > 1) tb_gpu_fail("invalid CUDA result ownership mask");
    if (control.root_owned[i] != 0
        && (control.root_words[i] == TERM_HOLE || term_tag(control.root_words[i]) == TAG_TSK))
      tb_gpu_fail("incomplete CUDA task result");
  }
  /* Every allocated or freed address lies below bump. The unallocated suffix
   * has no owners or allocator links; future allocation initializes its cells. */
  used_bytes = (size_t)state.bump * sizeof(Term);
  tb_heap_commit(state.bump);
  tb_gpu_require(tb_cuda_download(TB_GPU_CORPUS, 0, e.mem, used_bytes));
  tb_gpu_require(tb_cuda_download(TB_GPU_METADATA, 0, tb_heap_meta, used_bytes));
  tb_bump = state.bump; tb_live_words = state.live_words; tb_live_blocks = state.live_blocks;
  tb_steps = state.steps; memcpy(tb_free_lists, state.free_lists, sizeof(tb_free_lists));
  TBOutcome result = tb_task_packet(control.root_words, control.root_owned, control.root_count);
  TB_GPU_OBSERVE(&state, &control);
  TB_GPU_COMPLETE(&control, &state, tb_cuda_info());
  return result;
}
