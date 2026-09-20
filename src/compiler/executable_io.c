/* SPDX-License-Identifier: Apache-2.0
 * Effect ABI and scheduler contracts derived from Bend 2.0.5 comp.ts,
 * Copyright 2026 HigherOrderCO. Bounded CPU port: TeamDman.
 * See NOTICE and licenses/Apache-2.0.txt. */
#define IO_READ 1u
#define IO_TIME 2u
#define IO_PARK TERM_HOLE
typedef struct IoWork IoWork;
typedef void (*IoCall)(IoWork *);
typedef Term (*IoPack)(Env, IoWork *);
struct IoWork {
  intptr_t hand;
  intptr_t made;
  u32 word;
  u64 size;
  char *data;
  char *text;
  u32 code;
  IoCall call;
  IoPack pack;
};
typedef Term (*Effect)(Env, Term *, IoWork *);
typedef struct { Effect run; u32 ask; } IoEff;
typedef struct IoAct IoAct;
typedef struct { IoAct *head; IoAct *last; } IoQue;
typedef struct TBIOState TBIOState;
struct IoAct {
  IoWork work;
  Term cont;
  Term item;
  u64 time;
  short evts;
  intptr_t descriptor;
  IoAct *next;
  TBIOState *owner;
  u32 parked;
};
struct TBIOState {
  TBHost *host;
  IoQue completed;
  u32 active;
};
static IoEff io_eff_rows[65536];
static IoQue io_runs;
static IoQue io_park;
static IoQue io_jobs;
static u32 io_live;
static u32 io_busy;
static TBIOState *tb_io;
#ifdef _WIN32
static int tb_stdout_mode = -1;
static int tb_stderr_mode = -1;
#endif

INLINE void io_eff(u32 cid, Effect run, u32 need) {
  if (cid >= BEND_CID_COUNT || run == NULL || (need & ~(IO_READ | IO_TIME)) != 0 || need == (IO_READ | IO_TIME))
    err_fail("invalid foreign effect registration");
  if (io_eff_rows[cid].run != NULL && io_eff_rows[cid].run != run) err_fail("duplicate foreign effect registration");
  io_eff_rows[cid].run = run;
  io_eff_rows[cid].ask = need;
}
INLINE void tb_require_effect(u32 cid) {
  if (cid >= BEND_CID_COUNT || io_eff_rows[cid].run == NULL) err_fail("unregistered foreign effect");
}
INLINE void tb_reject_request(Term term) {
  u64 tag = term_tag(term), cid = term_aux(term);
  if (tag == TAG_CTR && cid < BEND_CID_COUNT && io_eff_rows[cid].run != NULL)
    err_fail("foreign request inspected as ordinary data");
}
INLINE void io_push(IoQue *queue, IoAct *action) {
  action->next = NULL;
  if (queue->head == NULL) queue->head = action; else queue->last->next = action;
  queue->last = action;
}
INLINE IoAct *io_pop(IoQue *queue) {
  IoAct *action = queue->head;
  if (action == NULL) err_fail("empty scheduler queue");
  queue->head = action->next;
  if (queue->head == NULL) queue->last = NULL;
  action->next = NULL;
  return action;
}
OUTLINE u64 io_tick(void) {
#ifdef _WIN32
  LARGE_INTEGER counter, frequency;
  if (!QueryPerformanceCounter(&counter) || !QueryPerformanceFrequency(&frequency) || frequency.QuadPart <= 0)
    err_fail("monotonic clock failed");
  return (u64)(counter.QuadPart / frequency.QuadPart) * UINT64_C(1000000000)
    + (u64)((counter.QuadPart % frequency.QuadPart) * UINT64_C(1000000000) / frequency.QuadPart);
#else
  struct timespec now;
  if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) err_fail("monotonic clock failed");
  return (u64)now.tv_sec * UINT64_C(1000000000) + (u64)now.tv_nsec;
#endif
}
INLINE u64 io_sys_end(IoWork *work, intptr_t count) {
  work->code = count < 0 ? (u32)errno : 0;
  return count < 0 ? 0 : (u64)count;
}
INLINE void io_out(FILE *stream, const char *data, u64 length) {
  if (length > SIZE_MAX || fwrite(data, 1, (size_t)length, stream) != length) err_fail("short standard stream write");
}
INLINE void io_sync(void) { if (fflush(stdout) != 0) err_fail("standard stream flush failed"); }
INLINE u64 io_utf8(char *buffer, u64 character) {
  u64 count = character < 0x80 ? 1 : character < 0x800 ? 2 : character < 0x10000 ? 3 : 4;
  for (u64 index = count; index > 1; --index) { buffer[index - 1] = (char)(0x80 | (character & 63)); character >>= 6; }
  buffer[0] = (char)(count == 1 ? character : (0xf00 >> count) | character);
  return count;
}
OUTLINE char *io_cstr(Env e, Term string, u64 *length) {
  u64 capacity = 64, used = 0;
  char *buffer = (char *)io_mem(tb_host_malloc((size_t)capacity));
  while (term_aux(string) == CID_SCON) {
    Term fields[2]; tb_tick();
    (void)ctr_take(e, string, 2, fields);
    if (used + 5 > capacity) {
      if (capacity > BEND_MAX_HOST_BUFFER / 2) err_fail("text byte budget exhausted");
      capacity *= 2;
      buffer = (char *)io_mem(tb_host_realloc(buffer, (size_t)capacity));
    }
    used += io_utf8(buffer + used, fields[0]);
    string = fields[1];
  }
  if (term_aux(string) != CID_SNIL || term_tag(string) != TAG_PAK) err_fail("malformed native string");
  buffer[used] = 0; *length = used;
  return buffer;
}
#define io_nul(text,length) (strlen(text) != (length))
INLINE Term io_seal(Env e, Term term, int hot) { return hot != 0 ? rfc_seal(e, term) : term; }
INLINE Term io_node(Env e, u64 cid, Term first, Term second, int hot) {
  if (cid >= BEND_CID_COUNT) err_fail("native constructor unavailable");
  Loc at = heap_alloc(e, 1);
  e.mem[at] = io_seal(e, first, hot); e.mem[at + 1] = io_seal(e, second, hot);
  return term_ctr(cid, at);
}
INLINE Term io_box(Env e, u64 cid, Term value, int hot) {
  if (cid >= BEND_CID_COUNT) err_fail("native constructor unavailable");
  Loc at = heap_alloc(e, 0); e.mem[at] = io_seal(e, value, hot); return term_ctr(cid, at);
}
#define io_tup(e,a,b) io_node((e),CID_TUPLE,(a),(b),IO_HOTS & 2)
#define io_done(e,value) io_box((e),CID_DONE,(value),IO_HOTS & 4)
OUTLINE Term io_str(Env e, const char *text, u64 length) {
  Term string = term_pak(CID_SNIL, 0);
  if (length > BEND_MAX_HOST_BUFFER) err_fail("text byte budget exhausted");
  while (length > 0) {
    u64 back = 0, first, count, character;
    tb_tick();
    while (back < 3 && back + 1 < length && ((u8)text[length - 1 - back] & 0xc0) == 0x80) ++back;
    first = (u8)text[length - 1 - back];
    count = first < 0xc0 ? 0 : first < 0xe0 ? 2 : first < 0xf0 ? 3 : 4;
    character = (u8)text[length - 1];
    if (count == back + 1) {
      character = first & (0x7f >> count);
      for (u64 index = 1; index < count; ++index) character = (character << 6) | ((u8)text[length - count + index] & 63);
    } else count = 1;
    string = io_node(e, CID_SCON, character, string, IO_HOTS & 1);
    length -= count;
  }
  return string;
}
INLINE Term io_fail(Env e, u32 code, const char *message) {
  const char *text = message != NULL ? message : strerror((int)code);
  return io_box(e, CID_FAIL, io_tup(e, code, io_str(e, text, strlen(text))), IO_HOTS & 8);
}
INLINE void io_errs(Env e, Term string) {
  u64 length; char *text = io_cstr(e, string, &length);
  io_sync(); io_out(stderr, text, length); io_out(stderr, "\n", 1); tb_host_free(text);
}
OUTLINE void io_spawn(Term computation) {
  IoAct *action;
  if (io_live >= BEND_MAX_ACTIONS) err_fail("activation budget exhausted");
  action = (IoAct *)io_mem(tb_host_calloc(1, sizeof(IoAct)));
  action->cont = computation; action->item = term_clo(FID_IO_EMIT, 0); action->owner = tb_io;
  io_push(&io_runs, action); ++io_live;
}
INLINE Term io_wait_on(IoWork *work, intptr_t descriptor, short events, IoPack more) {
  IoAct *action = (IoAct *)work;
  if (action->parked != 0 || more == NULL || descriptor < 0) err_fail("invalid readiness continuation");
  action->descriptor = descriptor; work->word = (u32)descriptor; work->pack = more;
  action->time = 0; action->evts = events; action->parked = 1;
  io_push(&io_park, action);
  return IO_PARK;
}
static void tb_start_jobs(void);
INLINE Term io_work(IoWork *work, IoCall call, IoPack pack) {
  IoAct *action = (IoAct *)work;
  if (action->parked != 0 || call == NULL || pack == NULL) err_fail("invalid worker continuation");
  work->call = call; work->pack = pack; action->parked = 2;
  io_push(&io_jobs, action); ++io_busy;
  tb_start_jobs();
  return IO_PARK;
}
#ifdef _WIN32
static unsigned __stdcall tb_worker(void *opaque)
#else
static void *tb_worker(void *opaque)
#endif
{
  IoAct *action = (IoAct *)opaque;
  TBIOState *owner = action->owner;
  TBHost *host = owner->host;
  jmp_buf guard;
  tb_host_current = host; tb_failure_guard = &guard;
  if (setjmp(guard) == 0) action->work.call(&action->work);
  tb_failure_guard = NULL;
  tb_lock(&host->mutex);
  --owner->active;
  if (!host->stopped) io_push(&owner->completed, action);
  tb_unlock(&host->mutex);
  tb_host_current = NULL;
  tb_host_release(host);
#ifdef _WIN32
  return 0;
#else
  return NULL;
#endif
}
static void tb_start_jobs(void) {
  while (io_jobs.head != NULL) {
    IoAct *action;
    bool available;
    tb_lock(&tb_io->host->mutex);
    available = tb_io->active < BEND_MAX_WORKERS;
    if (available) { ++tb_io->active; ++tb_io->host->references; }
    tb_unlock(&tb_io->host->mutex);
    if (!available) return;
    action = io_pop(&io_jobs);
#ifdef _WIN32
    uintptr_t thread = _beginthreadex(NULL, 0, tb_worker, action, 0, NULL);
    if (thread != 0) { CloseHandle((HANDLE)thread); continue; }
#else
    pthread_t thread;
    if (pthread_create(&thread, NULL, tb_worker, action) == 0) { (void)pthread_detach(thread); continue; }
#endif
    tb_lock(&tb_io->host->mutex); --tb_io->active; --tb_io->host->references; tb_unlock(&tb_io->host->mutex);
    err_fail("worker creation failed");
  }
}
OUTLINE void io_take(Env e) {
  IoQue completed;
  bool failed;
  tb_lock(&tb_io->host->mutex);
  completed = tb_io->completed;
  memset(&tb_io->completed, 0, sizeof(tb_io->completed));
  failed = tb_io->host->worker_failed;
  tb_unlock(&tb_io->host->mutex);
  if (failed) err_fail("native worker failed");
  while (completed.head != NULL) {
    IoAct *action = io_pop(&completed);
    Term value;
    action->parked = 0; --io_busy;
    value = action->work.pack(e, &action->work);
    if (value == IO_PARK) { if (action->parked == 0) err_fail("worker pack parked without a wait"); }
    else { action->item = value; io_push(&io_runs, action); }
  }
  tb_start_jobs();
}
OUTLINE Term io_exec(Env e, IoWork *work) {
  IoAct *action = (IoAct *)work;
  Term fields[256]; u32 cid = (u32)term_aux(action->cont), count = cid_arity(cid);
  if (term_tag(action->cont) != TAG_CTR) err_fail("foreign request requires a node");
  if (count == 0 || count > 255) err_fail("foreign request has invalid arity");
  tb_require_effect(cid);
  (void)ctr_take(e, action->cont, count, fields);
  action->cont = fields[count - 1];
  return io_eff_rows[cid].run(e, fields, work);
}
OUTLINE void io_wait(Env e) {
  u32 descriptors = 0, position = 0;
  u64 soon = 0, now = io_tick();
  int timeout = io_busy != 0 ? 10 : 1000;
  IoQue waiting;
#ifdef _WIN32
  WSAPOLLFD *polls;
#else
  struct pollfd *polls;
#endif
  for (IoAct *action = io_park.head; action != NULL; action = action->next) {
    if (action->time != 0) { if (soon == 0 || action->time < soon) soon = action->time; }
    else ++descriptors;
  }
  if (soon != 0) {
    u64 gap = soon <= now ? 0 : (soon - now) / UINT64_C(1000000) + 1;
    if (gap < (u64)timeout) timeout = (int)gap;
  }
  polls = io_mem(tb_host_calloc(descriptors == 0 ? 1 : descriptors, sizeof(*polls)));
  for (IoAct *action = io_park.head; action != NULL; action = action->next) if (action->time == 0) {
#ifdef _WIN32
    polls[position].fd = (SOCKET)action->descriptor;
#else
    if (action->descriptor > INT_MAX) err_fail("descriptor exceeds POSIX int range");
    polls[position].fd = (int)action->descriptor;
#endif
    polls[position].events = action->evts; ++position;
  }
  io_sync();
#ifdef _WIN32
  if (descriptors == 0) Sleep((DWORD)timeout);
  else if (WSAPoll(polls, descriptors, timeout) == SOCKET_ERROR) err_fail("descriptor poll failed");
#else
  while (poll(polls, descriptors, timeout) < 0) if (errno != EINTR) err_fail("descriptor poll failed");
#endif
  /* A worker pack may register a new wait. Keep it outside this poll snapshot. */
  waiting = io_park; memset(&io_park, 0, sizeof(io_park));
  if (io_busy != 0) io_take(e);
  now = io_tick(); position = 0;
  while (waiting.head != NULL) {
    IoAct *action = io_pop(&waiting);
    bool due = action->time != 0 ? action->time <= now : polls[position++].revents != 0;
    if (!due) { io_push(&io_park, action); continue; }
    Term value; action->parked = 0;
    value = action->work.pack(e, &action->work);
    if (value == IO_PARK) { if (action->parked == 0) err_fail("continuation parked without a wait"); }
    else { action->item = value; io_push(&io_runs, action); }
  }
  tb_host_free(polls);
}
OUTLINE int io_step(Env e, IoAct *action) {
  for (;;) {
    Term request = tb_apply(e, action->cont, action->item);
    u32 cid = (u32)term_aux(request), need;
    Loc at;
    tb_tick();
    if (term_tag(request) != TAG_PAK && term_tag(request) != TAG_CTR) err_fail("IO continuation returned ordinary data");
    if (cid == CID_EMIT) { tb_host_free(action); --io_live; return -1; }
    if (cid == CID_HALT) {
      if (term_tag(request) != TAG_CTR) err_fail("malformed Halt request");
      at = term_peek(e, request); tb_span(e, at, 2);
      io_errs(e, e.mem[at + 1]); return (int)(u32)e.mem[at];
    }
    tb_require_effect(cid); need = io_eff_rows[cid].ask; action->cont = request;
    if (need != 0) {
      Term fields[256]; u32 count = cid_arity(cid);
      if (count == 0 || count > 255) err_fail("invalid parked request");
      (void)ctr_take(e, request, count, fields);
      (void)io_wait_on(&action->work, (intptr_t)(need & IO_READ ? io_hand_v(fields[0]) : fields[0]), POLLIN, io_exec);
      if (need & IO_TIME) {
        u64 delay = (u32)fields[0] * UINT64_C(1000000), now = io_tick();
        if (delay > UINT64_MAX - now) err_fail("timer deadline overflow");
        action->time = now + delay;
        if (action->time == 0) action->time = 1;
      }
      return -1;
    }
    Term value = io_exec(e, &action->work);
    if (value == IO_PARK) { if (action->parked == 0) err_fail("effect parked without a wait"); return -1; }
    action->item = value;
  }
}
static int io_loop(Env e, Term main) {
  io_spawn(main);
  for (u64 turns = 0;; ++turns) {
    tb_tick();
    if (io_runs.head == NULL) {
      if (io_live == 0) { io_sync(); return 0; }
      if (io_park.head == NULL && io_busy == 0) { io_sync(); (void)fprintf(stderr, "bend: deadlock: every computation waits on a channel\n"); return 1; }
      io_wait(e); continue;
    }
    if ((turns & 63) == 0 && io_busy != 0) io_take(e);
    int code = io_step(e, io_pop(&io_runs));
    if (code >= 0) { io_sync(); return code; }
  }
}

typedef struct ChanRow {
  u32 gen, next, room, size, head, live, shut;
  Term *ring;
  IoQue wait;
} ChanRow;
static ChanRow *chan_rows;
static u32 chan_len;
static u32 chan_idle = UINT32_MAX;
#define chan_some(e,value) io_box((e),CID_SOME,(value),IO_HOTS & 32)
#define chan_bool(value) term_pak((value) ? CID_TRUE : CID_FALSE,0)
OUTLINE Term chan_open(u32 room) {
  u32 index = chan_idle;
  ChanRow *row;
  if (room > BEND_MAX_ACTIONS) err_fail("channel capacity budget exhausted");
  if (index != UINT32_MAX) chan_idle = chan_rows[index].next;
  else {
    if (chan_len >= BEND_MAX_ACTIONS) err_fail("channel count budget exhausted");
    if ((chan_len & (chan_len - 1)) == 0) {
      u32 capacity = chan_len == 0 ? 1 : chan_len * 2;
      chan_rows = (ChanRow *)io_mem(tb_host_realloc(chan_rows, capacity * sizeof(ChanRow)));
    }
    index = chan_len++; memset(&chan_rows[index], 0, sizeof(ChanRow));
  }
  row = &chan_rows[index];
  if (row->gen == UINT32_MAX) err_fail("channel generation exhausted");
  ++row->gen; row->room = room; row->size = 0; row->head = 0; row->live = 1; row->shut = 0;
  row->ring = room == 0 ? NULL : (Term *)io_mem(tb_host_calloc(room, sizeof(Term)));
  memset(&row->wait, 0, sizeof(row->wait));
  return io_hand(((u64)row->gen << 24) | index);
}
INLINE ChanRow *chan_at(Term channel) {
  u64 raw = io_hand_v(channel); u32 index = (u32)raw & 0xffffff;
  ChanRow *row = index < chan_len ? &chan_rows[index] : NULL;
  return row != NULL && row->live && row->gen == (u32)(raw >> 24) ? row : NULL;
}
INLINE Term chan_park(ChanRow *row, IoWork *work, Term item) {
  IoAct *action = (IoAct *)work;
  if (action->parked != 0) err_fail("activation already parked");
  action->item = item; action->parked = 3; io_push(&row->wait, action); return IO_PARK;
}
INLINE Term chan_wake(ChanRow *row, Term value) {
  IoAct *action = io_pop(&row->wait); Term old = action->item;
  action->item = value; action->parked = 0; io_push(&io_runs, action); return old;
}
INLINE void chan_free(ChanRow *row) {
  tb_host_free(row->ring); row->ring = NULL; row->live = 0; row->next = chan_idle;
  chan_idle = (u32)(row - chan_rows);
}
INLINE Term chan_take(ChanRow *row) {
  Term value = row->ring[row->head]; row->head = (row->head + 1) % row->room; --row->size;
  if (row->wait.head != NULL) {
    Term item = chan_wake(row, chan_bool(true)); row->ring[(row->head + row->size) % row->room] = item; ++row->size;
  }
  return value;
}
INLINE void chan_shut(Env e, ChanRow *row) {
  row->shut = 1;
  while (row->wait.head != NULL) {
    Term value = row->wait.head->item == TERM_HOLE ? term_pak(CID_NONE, 0) : chan_bool(false);
    term_sink(e, chan_wake(row, value));
  }
  if (row->size == 0) chan_free(row);
}

static Term tb_print_run(Env e, Term *fields, IoWork *work) {
  u64 length; char *text = io_cstr(e, fields[0], &length); (void)work;
  io_out(stdout, text, length); io_out(stdout, "\n", 1); tb_host_free(text); return term_pak(CID_UNIT, 0);
}
static Term tb_write_run(Env e, Term *fields, IoWork *work) {
  u64 length; char *text = io_cstr(e, fields[0], &length); (void)work;
  io_out(stdout, text, length); tb_host_free(text); return term_pak(CID_UNIT, 0);
}
static Term tb_print_err_run(Env e, Term *fields, IoWork *work) {
  (void)work; io_errs(e, fields[0]); return term_pak(CID_UNIT, 0);
}
static Term tb_spawn_run(Env e, Term *fields, IoWork *work) { (void)e; (void)work; io_spawn(fields[0]); return term_pak(CID_UNIT, 0); }
static Term tb_sleep_run(Env e, Term *fields, IoWork *work) { (void)e; (void)fields; (void)work; return term_pak(CID_UNIT, 0); }
static Term tb_now_run(Env e, Term *fields, IoWork *work) { (void)e; (void)fields; (void)work; return io_tick() / 1000000; }
static Term tb_get_env_run(Env e, Term *fields, IoWork *work) {
  u64 length; char *name = io_cstr(e, fields[0], &length); const char *value; Term result; (void)work;
#ifdef _MSC_VER
#pragma warning(push)
#pragma warning(disable: 4996) /* Match the upstream native narrow getenv ABI. */
#endif
  value = io_nul(name, length) ? NULL : getenv(name);
#ifdef _MSC_VER
#pragma warning(pop)
#endif
  result = value == NULL ? io_fail(e, ENOENT, NULL) : io_done(e, io_str(e, value, strlen(value)));
  tb_host_free(name); return result;
}
static Term tb_chan_new_run(Env e, Term *fields, IoWork *work) { (void)e; (void)work; return chan_open((u32)fields[0]); }
static Term tb_chan_send_run(Env e, Term *fields, IoWork *work) {
  ChanRow *row = chan_at(fields[0]);
  if (row == NULL || row->shut) { term_drop(e, fields[1]); return chan_bool(false); }
  if (row->wait.head != NULL && row->wait.head->item == TERM_HOLE) { (void)chan_wake(row, chan_some(e, fields[1])); return chan_bool(true); }
  if (row->size < row->room) { row->ring[(row->head + row->size) % row->room] = fields[1]; ++row->size; return chan_bool(true); }
  return chan_park(row, work, fields[1]);
}
static Term tb_chan_recv_run(Env e, Term *fields, IoWork *work) {
  ChanRow *row = chan_at(fields[0]);
  if (row == NULL) return term_pak(CID_NONE, 0);
  if (row->size != 0) { Term value = chan_take(row); if (row->shut && row->size == 0) chan_free(row); return chan_some(e, value); }
  if (row->wait.head != NULL && row->wait.head->item != TERM_HOLE) return chan_some(e, chan_wake(row, chan_bool(true)));
  if (row->shut) { chan_free(row); return term_pak(CID_NONE, 0); }
  return chan_park(row, work, TERM_HOLE);
}
static Term tb_chan_close_run(Env e, Term *fields, IoWork *work) { ChanRow *row = chan_at(fields[0]); (void)work; if (row != NULL) chan_shut(e, row); return term_pak(CID_UNIT, 0); }
OUTLINE void tb_register_builtins(void) {
#ifdef CID_IO_PRINT
  io_eff(CID_IO_PRINT, tb_print_run, 0);
#endif
#ifdef CID_IO_WRITE
  io_eff(CID_IO_WRITE, tb_write_run, 0);
#endif
#ifdef CID_IO_PRINT_ERR
  io_eff(CID_IO_PRINT_ERR, tb_print_err_run, 0);
#endif
#ifdef CID_IO_SPAWN
  io_eff(CID_IO_SPAWN, tb_spawn_run, 0);
#endif
#ifdef CID_IO_SLEEP
  io_eff(CID_IO_SLEEP, tb_sleep_run, IO_TIME);
#endif
#ifdef CID_IO_NOW
  io_eff(CID_IO_NOW, tb_now_run, 0);
#endif
#ifdef CID_IO_GET_ENV
  io_eff(CID_IO_GET_ENV, tb_get_env_run, 0);
#endif
#ifdef CID_CHAN_NEW
  io_eff(CID_CHAN_NEW, tb_chan_new_run, 0);
#endif
#ifdef CID_CHAN_SEND
  io_eff(CID_CHAN_SEND, tb_chan_send_run, 0);
#endif
#ifdef CID_CHAN_RECV
  io_eff(CID_CHAN_RECV, tb_chan_recv_run, 0);
#endif
#ifdef CID_CHAN_CLOSE
  io_eff(CID_CHAN_CLOSE, tb_chan_close_run, 0);
#endif
}
static void tb_io_initialize(void) {
  memset(&io_runs, 0, sizeof(io_runs)); memset(&io_park, 0, sizeof(io_park)); memset(&io_jobs, 0, sizeof(io_jobs));
  io_live = 0; io_busy = 0; chan_rows = NULL; chan_len = 0; chan_idle = UINT32_MAX;
  tb_io = (TBIOState *)io_mem(tb_host_calloc(1, sizeof(TBIOState))); tb_io->host = tb_host_current;
#ifdef _WIN32
  WSADATA winsock;
  if (WSAStartup(MAKEWORD(2, 2), &winsock) != 0) err_fail("Winsock initialization failed");
  tb_host_current->winsock = true;
  tb_stdout_mode = _setmode(_fileno(stdout), _O_BINARY);
  tb_stderr_mode = _setmode(_fileno(stderr), _O_BINARY);
  if (tb_stdout_mode == -1 || tb_stderr_mode == -1) err_fail("standard stream binary mode failed");
#endif
  /* Registrations run inside the invocation's guard through initialize(). */
  memset(io_eff_rows, 0, sizeof(io_eff_rows));
}
static void tb_io_shutdown(void) {
  if (tb_host_current != NULL) {
    tb_lock(&tb_host_current->mutex); tb_host_current->stopped = true; tb_unlock(&tb_host_current->mutex);
  }
  tb_io = NULL;
#ifdef _WIN32
  (void)fflush(stdout); (void)fflush(stderr);
  if (tb_stdout_mode != -1) { (void)_setmode(_fileno(stdout), tb_stdout_mode); tb_stdout_mode = -1; }
  if (tb_stderr_mode != -1) { (void)_setmode(_fileno(stderr), tb_stderr_mode); tb_stderr_mode = -1; }
#endif
}
