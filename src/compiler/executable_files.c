/* SPDX-License-Identifier: Apache-2.0
 * File effects derived from Bend 2.0.5 bend2/effs/file_*.c and get_env.c,
 * Copyright 2026 HigherOrderCO. Windows adapters and bounded ownership:
 * TeamDman. See NOTICE and licenses/Apache-2.0.txt. */

/* IoWork continues to carry raw CRT/POSIX descriptors. Only File.open creates
 * an owned row; trusted foreign descriptors are usable but not auto-adopted.
 * A worker lease prevents shutdown from closing/reusing its descriptor. */
struct TBFile {
  TBFile *next;
  TBFile *idle_next;
  int descriptor;
  u32 busy;
  bool live;
  bool closing;
};

#ifdef _WIN32
static void __cdecl tb_file_invalid_parameter(const wchar_t *expression,
    const wchar_t *function, const wchar_t *file, unsigned int line,
    uintptr_t reserved) {
  (void)expression; (void)function; (void)file; (void)line; (void)reserved;
}
/* A thread-local handler makes invalid foreign descriptors return errno
 * without changing the embedding application's process-wide CRT policy. */
static int tb_file_close_system(int descriptor) {
  _invalid_parameter_handler previous =
      _set_thread_local_invalid_parameter_handler(tb_file_invalid_parameter);
  int result = _close(descriptor), code = errno;
  (void)_set_thread_local_invalid_parameter_handler(previous);
  errno = code; return result;
}
static intptr_t tb_file_read_system(intptr_t descriptor, char *data, u32 count) {
  _invalid_parameter_handler previous;
  int result, code;
  if (descriptor < 0 || descriptor > INT_MAX) { errno = EBADF; return -1; }
  previous = _set_thread_local_invalid_parameter_handler(tb_file_invalid_parameter);
  result = _read((int)descriptor, data, count); code = errno;
  (void)_set_thread_local_invalid_parameter_handler(previous);
  errno = code; return result;
}
static intptr_t tb_file_write_system(intptr_t descriptor, const char *data, u32 count) {
  _invalid_parameter_handler previous;
  int result, code;
  if (descriptor < 0 || descriptor > INT_MAX) { errno = EBADF; return -1; }
  previous = _set_thread_local_invalid_parameter_handler(tb_file_invalid_parameter);
  result = _write((int)descriptor, data, count); code = errno;
  (void)_set_thread_local_invalid_parameter_handler(previous);
  errno = code; return result;
}
static wchar_t *tb_file_wide(const char *text, u32 *code) {
  int count = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, text, -1, NULL, 0);
  wchar_t *wide;
  if (count == 0) { *code = EILSEQ; return NULL; }
  if ((u64)count * sizeof(wchar_t) > BEND_MAX_HOST_BUFFER)
    err_fail("wide text byte budget exhausted");
  wide = (wchar_t *)io_mem(tb_host_malloc((size_t)count * sizeof(wchar_t)));
  if (MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, text, -1, wide, count) == 0) {
    tb_host_free(wide); *code = EILSEQ; return NULL;
  }
  return wide;
}
#else
static int tb_file_close_system(int descriptor) { return close(descriptor); }
static intptr_t tb_file_read_system(intptr_t descriptor, char *data, u32 count) {
  if (descriptor < 0 || descriptor > INT_MAX) { errno = EBADF; return -1; }
  return read((int)descriptor, data, count);
}
static intptr_t tb_file_write_system(intptr_t descriptor, const char *data, u32 count) {
  if (descriptor < 0 || descriptor > INT_MAX) { errno = EBADF; return -1; }
  return write((int)descriptor, data, count);
}
#endif

/* All row mutations, including closing before reuse, hold the owner's lock. */
static void tb_file_retire(TBHost *host, TBFile *file) {
  if (file->descriptor >= 0) (void)tb_file_close_system(file->descriptor);
  file->descriptor = -1; file->live = false; file->closing = false;
  file->idle_next = host->idle_files; host->idle_files = file;
}
static TBFile *tb_file_reserve(IoAct *action) {
  TBHost *host = action->owner->host;
  TBFile *file;
  tb_lock(&host->mutex);
  file = host->idle_files;
  if (file != NULL) host->idle_files = file->idle_next;
  tb_unlock(&host->mutex);
  if (file == NULL) {
    /* Only the VM creates rows, and tracked allocation takes the same lock. */
    if (host->file_count >= BEND_MAX_ACTIONS) err_fail("file handle budget exhausted");
    file = (TBFile *)io_mem(tb_host_calloc(1, sizeof(TBFile)));
    tb_lock(&host->mutex);
    file->next = host->files; host->files = file; ++host->file_count;
    tb_unlock(&host->mutex);
  }
  tb_lock(&host->mutex);
  file->descriptor = -1; file->busy = 1; file->live = true; file->closing = false;
  action->file = file;
  tb_unlock(&host->mutex);
  return file;
}
static bool tb_file_lease(IoAct *action) {
  TBHost *host = action->owner->host;
  bool allowed = true;
  tb_lock(&host->mutex);
  for (TBFile *file = host->files; file != NULL; file = file->next) {
    if (file->live && file->descriptor >= 0 && file->descriptor == action->work.hand) {
      if (file->closing || file->busy == UINT32_MAX) allowed = false;
      else { ++file->busy; action->file = file; }
      break;
    }
  }
  tb_unlock(&host->mutex);
  return allowed;
}
static void tb_file_job_finished(IoAct *action) {
  TBHost *host = action->owner->host;
  tb_lock(&host->mutex);
  if (action->file != NULL) {
    TBFile *file = action->file;
    --file->busy; action->file = NULL;
    if (file->busy == 0 && (host->stopped || file->closing || file->descriptor < 0))
      tb_file_retire(host, file);
  }
  tb_unlock(&host->mutex);
}
static void tb_files_shutdown(TBHost *host) {
  tb_lock(&host->mutex);
  for (TBFile *file = host->files; file != NULL; file = file->next) {
    if (file->live && file->busy == 0) tb_file_retire(host, file);
  }
  tb_unlock(&host->mutex);
}
static void tb_files_release(TBHost *host) {
  /* The final reference implies there are no workers; metadata is still live. */
  for (TBFile *file = host->files; file != NULL; file = file->next) {
    if (file->live && file->descriptor >= 0) (void)tb_file_close_system(file->descriptor);
  }
}

static void tb_file_open_call(IoWork *work) {
  IoAct *action = (IoAct *)work;
#ifdef _WIN32
  wchar_t *path = tb_file_wide(work->data, &work->code);
  if (path == NULL) return;
  work->made = _wopen(path, (int)work->word | _O_BINARY, _S_IREAD | _S_IWRITE);
  work->code = work->made < 0 ? (u32)errno : 0;
  tb_host_free(path);
#else
  work->made = open(work->data, (int)work->word, 0644);
  work->code = work->made < 0 ? (u32)errno : 0;
#endif
  tb_lock(&action->owner->host->mutex);
  action->file->descriptor = (int)work->made;
  tb_unlock(&action->owner->host->mutex);
}
static Term tb_file_open_pack(Env e, IoWork *work) {
  tb_host_free(work->data); work->data = NULL;
  return work->code != 0 ? io_fail(e, work->code, NULL) : io_done(e, io_hand((u64)work->made));
}
static Term tb_file_open_run(Env e, Term *fields, IoWork *work) {
  u64 path_length, mode_length;
  char *mode;
  int flags;
  work->data = io_cstr(e, fields[0], &path_length);
  mode = io_cstr(e, fields[1], &mode_length);
  flags = io_nul(mode, mode_length) ? -1 : strcmp(mode, "r") == 0 ? O_RDONLY
      : strcmp(mode, "w") == 0 ? O_WRONLY | O_CREAT | O_TRUNC
      : strcmp(mode, "a") == 0 ? O_WRONLY | O_CREAT | O_APPEND : -1;
  tb_host_free(mode);
  work->code = io_nul(work->data, path_length) ? EILSEQ : flags < 0 ? EINVAL : 0;
#ifdef _WIN32
  if (work->code == 0 && MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
      work->data, -1, NULL, 0) == 0) work->code = EILSEQ;
#endif
  work->made = -1;
  if (work->code != 0) return tb_file_open_pack(e, work);
  work->word = (u32)flags;
  (void)tb_file_reserve((IoAct *)work);
  return io_work(work, tb_file_open_call, tb_file_open_pack);
}
static void tb_file_read_call(IoWork *work) {
  intptr_t count = tb_file_read_system(work->hand, work->data, work->word);
  work->code = count < 0 ? (u32)errno : 0;
  work->size = count < 0 ? 0 : (u64)count;
}
static Term tb_file_read_pack(Env e, IoWork *work) {
  Term result = work->code != 0 ? io_fail(e, work->code, NULL)
      : io_done(e, io_str(e, work->data, work->size));
  tb_host_free(work->data); work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), result);
}
static Term tb_file_read_bytes_pack(Env e, IoWork *work) {
  Term result;
  if (work->code != 0) result = io_fail(e, work->code, NULL);
  else {
    Term list = term_pak(CID_NIL, 0);
    for (u64 at = work->size; at > 0; --at) {
      tb_tick(); list = io_node(e, CID_CON, (u8)work->data[at - 1], list, IO_HOTS & 16);
    }
    result = io_done(e, list);
  }
  tb_host_free(work->data); work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), result);
}
static Term tb_file_read_start(Env e, Term *fields, IoWork *work, IoPack pack) {
  u64 handle = io_hand_v(fields[0]);
  u32 count = (u32)fields[1];
  if (handle > INTPTR_MAX) err_fail("file descriptor exceeds host pointer range");
  work->hand = (intptr_t)handle;
  work->word = count > INT32_MAX ? INT32_MAX : count;
  if (work->word > BEND_MAX_HOST_BUFFER) err_fail("file read byte budget exhausted");
  /* Neither io_str nor byte packing needs a trailing NUL. Preserve the full
   * allowed read count rather than losing a byte to an unused terminator. */
  work->data = (char *)io_mem(tb_host_malloc(work->word == 0 ? 1 : work->word));
  work->size = 0; work->code = 0;
  if (!tb_file_lease((IoAct *)work)) { work->code = EBADF; return pack(e, work); }
  return io_work(work, tb_file_read_call, pack);
}
static Term tb_file_read_run(Env e, Term *fields, IoWork *work) {
  return tb_file_read_start(e, fields, work, tb_file_read_pack);
}
static Term tb_file_read_bytes_run(Env e, Term *fields, IoWork *work) {
  return tb_file_read_start(e, fields, work, tb_file_read_bytes_pack);
}
static void tb_file_write_call(IoWork *work) {
  u64 written = 0;
  work->code = 0;
  while (written < work->size) {
    u64 left = work->size - written;
    u32 count = left > INT32_MAX ? INT32_MAX : (u32)left;
    intptr_t result = tb_file_write_system(work->hand, work->data + written, count);
    if (result < 0) { work->code = (u32)errno; return; }
    if (result == 0 || (u64)result > count) err_fail("file write made invalid progress");
    written += (u64)result;
  }
}
static Term tb_file_write_pack(Env e, IoWork *work) {
  tb_host_free(work->data); work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), work->code != 0
      ? io_fail(e, work->code, NULL) : io_done(e, term_pak(CID_UNIT, 0)));
}
static Term tb_file_write_run(Env e, Term *fields, IoWork *work) {
  u64 handle = io_hand_v(fields[0]);
  if (handle > INTPTR_MAX) err_fail("file descriptor exceeds host pointer range");
  work->hand = (intptr_t)handle;
  work->data = io_cstr(e, fields[1], &work->size); work->code = 0;
  /* Upstream empty writes still use a worker, but never issue a syscall. */
  if (work->size != 0 && !tb_file_lease((IoAct *)work)) {
    work->code = EBADF; return tb_file_write_pack(e, work);
  }
  return io_work(work, tb_file_write_call, tb_file_write_pack);
}
static Term tb_file_close_run(Env e, Term *fields, IoWork *work) {
  TBHost *host = tb_host_current;
  u64 handle = io_hand_v(fields[0]);
  bool owned = false;
  (void)e; (void)work;
  tb_lock(&host->mutex);
  for (TBFile *file = host->files; file != NULL; file = file->next) {
    if (file->live && file->descriptor >= 0 && (u64)file->descriptor == handle) {
      owned = true; file->closing = true;
      if (file->busy == 0) tb_file_retire(host, file);
      break;
    }
  }
  tb_unlock(&host->mutex);
  if (!owned && handle <= INT_MAX) (void)tb_file_close_system((int)handle);
  return term_pak(CID_UNIT, 0);
}

static Term tb_get_env_run(Env e, Term *fields, IoWork *work) {
  u64 length;
  char *name = io_cstr(e, fields[0], &length);
  Term result;
  (void)work;
  if (io_nul(name, length) || strchr(name, '=') != NULL) {
    tb_host_free(name); return io_fail(e, ENOENT, NULL);
  }
#ifdef _WIN32
  u32 code = 0;
  wchar_t *wide_name = tb_file_wide(name, &code);
  tb_host_free(name);
  if (wide_name == NULL) return io_fail(e, code, NULL);
  /* Read the Unicode process environment, independent of the current ACP.
   * The small retry bound handles a concurrently resized environment value. */
  for (u32 attempt = 0; attempt < 8; ++attempt) {
    DWORD count, used;
    wchar_t *wide_value;
    char *value;
    int bytes;
    SetLastError(ERROR_SUCCESS);
    count = GetEnvironmentVariableW(wide_name, NULL, 0);
    if (count == 0) {
      code = GetLastError() == ERROR_SUCCESS ? 0 : ENOENT;
      tb_host_free(wide_name);
      return code != 0 ? io_fail(e, code, NULL) : io_done(e, term_pak(CID_SNIL, 0));
    }
    if ((u64)count * sizeof(wchar_t) > BEND_MAX_HOST_BUFFER)
      err_fail("environment byte budget exhausted");
    wide_value = (wchar_t *)io_mem(tb_host_malloc((size_t)count * sizeof(wchar_t)));
    SetLastError(ERROR_SUCCESS);
    used = GetEnvironmentVariableW(wide_name, wide_value, count);
    if (used >= count) { tb_host_free(wide_value); continue; }
    if (used == 0) {
      code = GetLastError() == ERROR_SUCCESS ? 0 : ENOENT;
      tb_host_free(wide_value); tb_host_free(wide_name);
      return code != 0 ? io_fail(e, code, NULL) : io_done(e, term_pak(CID_SNIL, 0));
    }
    bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, wide_value, (int)used, NULL, 0, NULL, NULL);
    if (bytes == 0) { tb_host_free(wide_value); tb_host_free(wide_name); return io_fail(e, EILSEQ, NULL); }
    value = (char *)io_mem(tb_host_malloc((size_t)bytes));
    if (WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, wide_value, (int)used, value, bytes, NULL, NULL) == 0)
      err_fail("environment UTF-8 conversion failed");
    result = io_done(e, io_str(e, value, (u64)bytes));
    tb_host_free(value); tb_host_free(wide_value); tb_host_free(wide_name); return result;
  }
  err_fail("environment changed during bounded lookup");
#else
  const char *value = getenv(name);
  result = value == NULL ? io_fail(e, ENOENT, NULL) : io_done(e, io_str(e, value, strlen(value)));
  tb_host_free(name); return result;
#endif
}
static void tb_register_files(void) {
#ifdef CID_FILE_OPEN
  io_eff(CID_FILE_OPEN, tb_file_open_run, 0);
#endif
#ifdef CID_FILE_READ
  io_eff(CID_FILE_READ, tb_file_read_run, 0);
#endif
#ifdef CID_FILE_READ_BYTES
  io_eff(CID_FILE_READ_BYTES, tb_file_read_bytes_run, 0);
#endif
#ifdef CID_FILE_WRITE
  io_eff(CID_FILE_WRITE, tb_file_write_run, 0);
#endif
#ifdef CID_FILE_CLOSE
  io_eff(CID_FILE_CLOSE, tb_file_close_run, 0);
#endif
}
