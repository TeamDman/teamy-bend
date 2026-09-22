/* SPDX-License-Identifier: Apache-2.0
 * Generated executable controls derived from Bend 2.0.5 comp.ts,
 * Copyright 2026 HigherOrderCO. Port and checked parsing: TeamDman.
 * See NOTICE and licenses/Apache-2.0.txt. */
#ifndef TB_NO_MAIN
#ifdef __APPLE__
#include <mach-o/dyld.h>
#endif

static int tb_cli_error(const char *message, const char *argument) {
  (void)fprintf(stderr, "bend: %s%s\n", message, argument == NULL ? "" : argument);
  return 1;
}

static bool tb_cli_threads(const char *text, u32 *threads) {
  uint64_t count = 0;
  if (text == NULL || *text == 0) return false;
  /* Values above 128 still parse and clamp like upstream, without overflowing
   * a host long. Retain validation of every digit after reaching the cap. */
  if (*text == '+') ++text;
  if (*text == 0) return false;
  for (; *text != 0; ++text) {
    if (*text < '0' || *text > '9') return false;
    if (count < TB_CPU_LIMIT) count = count * 10 + (uint64_t)(*text - '0');
  }
  if (count == 0) return false;
  *threads = count > TB_CPU_LIMIT ? TB_CPU_LIMIT : (u32)count;
  return true;
}

static bool tb_cli_span(const char *text, u64 *span) {
  char *end = NULL;
  if (text == NULL) return false;
  errno = 0;
  long double number = strtold(text, &end);
  u64 multiplier = strcmp(end, "MB") == 0 ? UINT64_C(1048576)
    : strcmp(end, "GB") == 0 ? UINT64_C(1073741824) : 0;
  if (errno == ERANGE || end == text || multiplier == 0 || !isfinite(number) || number <= 0)
    return false;
  long double bytes = number * (long double)multiplier;
  if (!isfinite(bytes) || bytes >= 18446744073709551616.0L
      || bytes < 16384.0L) return false;
  u64 value = ((u64)bytes) & ~UINT64_C(16383);
  if (value > SIZE_MAX || value / sizeof(Term) > LOC_MASK
      || value / sizeof(Term) <= HEAP_OFF) return false;
  *span = value;
  return true;
}

#ifdef TB_GPU_ENABLED
/* Resolve the running binary, never argv[0] or the compilation directory.
 * The Windows loader path is Unicode; the adapter accepts UTF-8 cache paths. */
static char *tb_cli_cache_path(void) {
  enum { CAPACITY = TB_CUDA_CACHE_PATH_LIMIT };
  char *path = (char *)malloc(CAPACITY);
  if (path == NULL) return NULL;
#ifdef _WIN32
  wchar_t *wide = (wchar_t *)malloc(32768u * sizeof(wchar_t));
  if (wide == NULL) { free(path); return NULL; }
  DWORD length = GetModuleFileNameW(NULL, wide, 32768u);
  int bytes = length == 0 || length >= 32768u ? 0
    : WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, wide, (int)length,
        path, CAPACITY - 5, NULL, NULL);
  free(wide);
  if (bytes == 0) { free(path); return NULL; }
  size_t used = (size_t)bytes;
#elif defined(__APPLE__)
  uint32_t capacity = CAPACITY - 4;
  if (_NSGetExecutablePath(path, &capacity) != 0) { free(path); return NULL; }
  char *resolved = realpath(path, NULL);
  if (resolved == NULL) { free(path); return NULL; }
  size_t used = strlen(resolved);
  if (used >= CAPACITY - 4) { free(resolved); free(path); return NULL; }
  memcpy(path, resolved, used); free(resolved);
#else
  ssize_t length = readlink("/proc/self/exe", path, CAPACITY - 5);
  if (length <= 0 || length >= CAPACITY - 5) { free(path); return NULL; }
  size_t used = (size_t)length;
#endif
  memcpy(path + used, ".gpu", 5);
  return path;
}

static int tb_cli_gpu_prepare(bool prebuild) {
  char *path = tb_cli_cache_path();
  bool configured = tb_cuda_set_cache_path(path);
  free(path);
  bool success = configured && tb_gpu_prepare(prebuild);
  if (!success) {
    (void)tb_cli_error(tb_cuda_error(), NULL);
    tb_gpu_shutdown();
    return 1;
  }
  if (prebuild) tb_gpu_shutdown();
  return 0;
}

static int tb_cli_gpu_start(void) {
  const char *policy = tb_gpu_policy();
  bool forced = false;
  if (policy == NULL || strcmp(policy, "auto") == 0) { }
  else if (strcmp(policy, "off") == 0 || strcmp(policy, "0") == 0) return 0;
  else if (strcmp(policy, "on") == 0 || strcmp(policy, "forced") == 0 || strcmp(policy, "1") == 0)
    forced = true;
  else return tb_cli_error("BEND_GPU must be auto, off, or on", NULL);
  if (tb_cli_gpu_prepare(false) != 0) return 1;
  if (forced && tb_gpu_status < 0) {
    tb_gpu_shutdown();
    return tb_cli_error("--gpu on, but this binary found no GPU device", NULL);
  }
  tb_cli_gpu_active = tb_gpu_status > 0;
#if !TB_CUDA_GPU_ALLOC_EXPLICIT
  if (tb_cli_gpu_active && tb_cli_gpu_span != 0) {
    u64 overhead = (u64)BEND_MAX_GPU_SCRATCH;
    if (overhead > UINT64_MAX - sizeof(TBDeviceState) - sizeof(struct TBDeviceControl)) {
      tb_gpu_shutdown(); return tb_cli_error("GPU allocation limit overflows", NULL);
    }
    overhead += sizeof(TBDeviceState) + sizeof(struct TBDeviceControl);
    if (tb_cli_gpu_span > (UINT64_MAX - overhead) / 3
        || !tb_cuda_set_allocation_limit(tb_cli_gpu_span * 3 + overhead)) {
      (void)tb_cli_error("GPU span exceeds the allocation limit", NULL);
      tb_gpu_shutdown(); return 1;
    }
  }
#endif
  if (tb_cli_gpu_active) {
    u64 corpus = tb_cli_gpu_span != 0 ? tb_cli_gpu_span : (u64)BEND_MAX_ALLOC;
    u64 scratch = (u64)BEND_MAX_GPU_SCRATCH / sizeof(u64) * sizeof(u64);
    corpus = corpus / sizeof(Term) * sizeof(Term);
    if (corpus > SIZE_MAX || corpus / sizeof(Term) <= HEAP_OFF
        || corpus / sizeof(Term) > LOC_MASK || scratch > SIZE_MAX || scratch < 4 * sizeof(u64)) {
      tb_gpu_shutdown(); return tb_cli_error("invalid CUDA corpus or scratch reservation", NULL);
    }
    /* Reserve before any Bend/foreign initialization can produce effects.
     * Handoffs reuse these buffers; prebuild never performs this reservation. */
    if (!tb_cuda_reserve(TB_GPU_CORPUS, (size_t)corpus)
        || !tb_cuda_reserve(TB_GPU_METADATA, (size_t)corpus)
        || !tb_cuda_reserve(TB_GPU_STATE, sizeof(TBDeviceState))
        || !tb_cuda_reserve(TB_GPU_SCRATCH, (size_t)scratch)
        || !tb_cuda_reserve(TB_GPU_CONTROL, sizeof(struct TBDeviceControl))) {
      (void)tb_cli_error(tb_cuda_error(), NULL);
      tb_gpu_shutdown(); return 1;
    }
  }
  return 0;
}
#endif

static int tb_cli_dispatch(int argc, char **argv, int (*program)(void)) {
  tb_cpu_requested_workers = 0;
  tb_cli_gpu_span = 0; tb_cli_gpu_active = false;
#ifdef TB_GPU_ENABLED
  tb_gpu_policy_override = NULL;
#endif
  for (int i = 1; i < argc; ++i) {
    const char *arg = argv[i];
    if (strcmp(arg, "--help") == 0) {
      (void)printf("usage: %s [options]\n"
        "  --threads N       worker threads, 1 to 128 (default: CPU count)\n"
        "  --gpu on|off|4GB  run ! calls on the GPU; a size reserves corpus memory\n"
        "  --gpu-build       write the GPU program and exit without running main\n"
        "  --help            show this text\n", argc > 0 ? argv[0] : "bend-program");
      return 0;
    }
    if (strcmp(arg, "--gpu-build") == 0) {
#ifdef TB_GPU_ENABLED
      return tb_cli_gpu_prepare(true);
#else
      return 0;
#endif
    }
    const char *value = i + 1 < argc ? argv[i + 1] : NULL;
    if (strcmp(arg, "--threads") == 0) {
      if (!tb_cli_threads(value, &tb_cpu_requested_workers))
        return tb_cli_error("expected a thread count of 1 or more after --threads", NULL);
      ++i;
    } else if (strcmp(arg, "--gpu") == 0) {
      const char *policy = "on";
      if (value != NULL && strcmp(value, "off") == 0) policy = "off";
      else if (value != NULL && strcmp(value, "on") == 0) tb_cli_gpu_span = 0;
      else if (!tb_cli_span(value, &tb_cli_gpu_span))
        return tb_cli_error("expected on, off or a size like 4GB after --gpu", NULL);
#ifdef TB_GPU_ENABLED
      tb_gpu_policy_override = policy;
#else
      (void)policy;
#endif
      ++i;
    } else return tb_cli_error("unknown option ", arg);
  }
#ifdef TB_GPU_ENABLED
  if (tb_cli_gpu_start() != 0) return 1;
#endif
  int result = program();
#ifdef TB_GPU_ENABLED
  /* tb_run normally shuts down itself; also cover its earliest host-allocation
   * failure before its guarded cleanup becomes active. Shutdown is idempotent. */
  tb_gpu_shutdown();
#endif
  tb_cli_gpu_active = false;
  return result;
}

static int tb_cli_main(int argc, char **argv, int (*program)(void)) {
#ifdef _WIN32
  /* CLI/prebuild diagnostics precede tb_io_initialize. Give them the same LF
   * byte contract as Bend output and restore the caller's descriptor modes. */
  int output_mode = _setmode(_fileno(stdout), _O_BINARY);
  int error_mode = _setmode(_fileno(stderr), _O_BINARY);
#endif
  int result = tb_cli_dispatch(argc, argv, program);
#ifdef _WIN32
  (void)fflush(stdout); (void)fflush(stderr);
  if (output_mode != -1) (void)_setmode(_fileno(stdout), output_mode);
  if (error_mode != -1) (void)_setmode(_fileno(stderr), error_mode);
#endif
  return result;
}
#endif
