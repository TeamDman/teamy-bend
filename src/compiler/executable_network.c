/* SPDX-License-Identifier: Apache-2.0
 * TCP/UDP effects and address parsing derived from Bend 2.0.5 comp.ts and
 * bend2/effs/{tcp,udp,socket,listener}_*.c, Copyright 2026 HigherOrderCO.
 * Windows adaptation and bounded ownership: TeamDman. See NOTICE. */
#ifdef _WIN32
typedef SOCKET TBNetRaw;
typedef int TBNetLength;
#define TB_NET_INVALID INVALID_SOCKET
#else
typedef int TBNetRaw;
typedef socklen_t TBNetLength;
#define TB_NET_INVALID (-1)
#endif
struct TBNetSocket {
  TBNetSocket *next;
  TBNetSocket *idle_next;
  intptr_t descriptor;
};
static u32 tb_net_error(void) {
#ifdef _WIN32
  return (u32)WSAGetLastError();
#else
  return (u32)errno;
#endif
}
static bool tb_net_again(u32 code) {
#ifdef _WIN32
  return code == WSAEWOULDBLOCK;
#else
  return code == EAGAIN || code == EWOULDBLOCK;
#endif
}
static bool tb_net_pending(u32 code) {
#ifdef _WIN32
  return code == WSAEWOULDBLOCK || code == WSAEINPROGRESS;
#else
  return code == EINPROGRESS;
#endif
}
static void tb_net_close_system(intptr_t descriptor) {
#ifdef _WIN32
  (void)closesocket((SOCKET)descriptor);
#else
  if (descriptor >= 0 && descriptor <= INT_MAX) (void)close((int)descriptor);
#endif
}
static int tb_net_nonblocking(intptr_t descriptor) {
#ifdef _WIN32
  u_long enabled = 1;
  return ioctlsocket((SOCKET)descriptor, FIONBIO, &enabled);
#else
  int flags = fcntl((int)descriptor, F_GETFL);
  return flags < 0 ? -1 : fcntl((int)descriptor, F_SETFL, flags | O_NONBLOCK);
#endif
}
static int io_sys_addr(const char *host, u32 port, struct sockaddr_in *address) {
  memset(address, 0, sizeof(*address));
  address->sin_family = AF_INET; address->sin_port = htons((uint16_t)port);
  for (const char *at = host; *at != 0; ++at) {
    if ((at == host || at[-1] == '.') && *at == '0' && at[1] >= '0' && at[1] <= '9') return -1;
  }
  if (port > 65535) return -1;
#ifdef _WIN32
  return InetPtonA(AF_INET, host, &address->sin_addr) == 1 ? 0 : -1;
#else
  return inet_pton(AF_INET, host, &address->sin_addr) == 1 ? 0 : -1;
#endif
}
static Term tb_net_fail(Env e, u32 code) {
#ifdef _WIN32
  if (code >= WSABASEERR) {
    wchar_t wide[512]; char message[2100];
    DWORD length = FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        NULL, code, 0, wide, (DWORD)(sizeof(wide) / sizeof(wide[0])), NULL);
    int count = 0;
    while (length != 0 && (wide[length - 1] == L'\r' || wide[length - 1] == L'\n'
        || wide[length - 1] == L' ' || wide[length - 1] == L'\t')) --length;
    if (length != 0) count = WideCharToMultiByte(CP_UTF8, 0, wide, (int)length,
        message, 2048, NULL, NULL);
    if (count == 0) (void)snprintf(message, sizeof(message), "OS error %u", code);
    else (void)snprintf(message + count, sizeof(message) - (size_t)count, " (os error %u)", code);
    return io_fail(e, code, message);
  }
#endif
  return io_fail(e, code, NULL);
}
static intptr_t tb_net_created(TBNetRaw raw) {
  TBHost *host = tb_host_current;
  TBNetSocket *row;
  if (raw == TB_NET_INVALID) return -1;
  if ((u64)raw > INTPTR_MAX || (u64)raw >> 56) {
#ifdef _WIN32
    (void)closesocket(raw);
#else
    (void)close(raw);
#endif
    err_fail("socket exceeds native handle representation");
  }
  row = host->idle_sockets;
  if (row != NULL) host->idle_sockets = row->idle_next;
  else {
    if (host->socket_count >= BEND_MAX_ACTIONS) {
      tb_net_close_system((intptr_t)raw); err_fail("socket handle budget exhausted");
    }
    row = (TBNetSocket *)tb_host_calloc(1, sizeof(TBNetSocket));
    if (row == NULL) { tb_net_close_system((intptr_t)raw); err_fail("socket ownership allocation failed"); }
    row->next = host->sockets; host->sockets = row; ++host->socket_count;
  }
  row->descriptor = (intptr_t)raw;
  return row->descriptor;
}
static void tb_net_close(intptr_t descriptor) {
  TBHost *host = tb_host_current;
  for (TBNetSocket *row = host->sockets; row != NULL; row = row->next) {
    if (row->descriptor == descriptor && descriptor >= 0) {
      row->descriptor = -1; row->idle_next = host->idle_sockets; host->idle_sockets = row;
      break;
    }
  }
  if (descriptor >= 0) tb_net_close_system(descriptor);
}
static void tb_network_shutdown(TBHost *host) {
  /* Socket effects run on the VM. Worker notification alone shares this lock. */
  tb_lock(&host->mutex);
  if (host->wake_read >= 0) tb_net_close_system(host->wake_read);
  if (host->wake_write >= 0) tb_net_close_system(host->wake_write);
  host->wake_read = -1; host->wake_write = -1;
  for (TBNetSocket *row = host->sockets; row != NULL; row = row->next) {
    if (row->descriptor >= 0) { tb_net_close_system(row->descriptor); row->descriptor = -1; }
  }
  tb_unlock(&host->mutex);
}
static intptr_t tb_net_handle(Term value) {
  u64 descriptor = io_hand_v(value);
  if (descriptor > INTPTR_MAX) err_fail("socket exceeds host pointer range");
#ifndef _WIN32
  if (descriptor > INT_MAX) err_fail("socket exceeds POSIX descriptor range");
#endif
  return (intptr_t)descriptor;
}
static u32 tb_net_receive_count(Term value) {
  u32 count = (u32)value;
  if (count > INT32_MAX) count = INT32_MAX;
  if (count > BEND_MAX_HOST_BUFFER) err_fail("network receive byte budget exhausted");
  return count;
}
static char *tb_net_receive_buffer(u32 count) {
  return (char *)io_mem(tb_host_malloc(count == 0 ? 1 : count));
}
static int tb_net_send_flags(intptr_t descriptor) {
  (void)descriptor;
#ifdef MSG_NOSIGNAL
  return MSG_NOSIGNAL;
#elif defined(SO_NOSIGPIPE)
  int one = 1;
  if (setsockopt((TBNetRaw)descriptor, SOL_SOCKET, SO_NOSIGPIPE, &one, sizeof(one)) < 0) return -1;
  return 0;
#else
  return 0;
#endif
}
static intptr_t tb_net_send_system(intptr_t descriptor, const char *data, u32 size,
    const struct sockaddr_in *address) {
  int flags = tb_net_send_flags(descriptor);
  if (flags < 0) return -1;
  if (address != NULL) return sendto((TBNetRaw)descriptor, data, (int)size, flags,
      (const struct sockaddr *)address, (TBNetLength)sizeof(*address));
  return send((TBNetRaw)descriptor, data, (int)size, flags);
}
static intptr_t tb_net_recv_system(intptr_t descriptor, char *data, u32 size,
    struct sockaddr_in *address) {
#ifdef _WIN32
  WSABUF buffer; DWORD received = 0, flags = 0;
  int status; TBNetLength length = sizeof(*address);
  buffer.buf = data; buffer.len = size;
  if (address != NULL) status = WSARecvFrom((SOCKET)descriptor, &buffer, 1, &received,
      &flags, (struct sockaddr *)address, &length, NULL, NULL);
  else status = WSARecv((SOCKET)descriptor, &buffer, 1, &received, &flags, NULL, NULL);
  if (status == SOCKET_ERROR && WSAGetLastError() != WSAEMSGSIZE) return -1;
  return received > size ? size : received;
#else
  TBNetLength length = sizeof(*address);
  if (address != NULL) return recvfrom((int)descriptor, data, size, 0, (struct sockaddr *)address, &length);
  return recv((int)descriptor, data, size, 0);
#endif
}
static Term tb_net_packet(Env e, const struct sockaddr_in *address, const char *data, u64 length) {
  char host[16];
#ifdef _WIN32
  struct in_addr copy = address->sin_addr;
  if (InetNtopA(AF_INET, &copy, host, sizeof(host)) == NULL) err_fail("IPv4 formatting failed");
#else
  if (inet_ntop(AF_INET, &address->sin_addr, host, sizeof(host)) == NULL) err_fail("IPv4 formatting failed");
#endif
  return io_tup(e, io_str(e, host, strlen(host)), io_tup(e, ntohs(address->sin_port), io_str(e, data, length)));
}

/* A connected loopback pair wakes file-worker completions without a busy poll.
 * Installation and notification share the completion-queue mutex, so completion
 * immediately before installation cannot be lost. Notifications never block. */
static void tb_network_notify_locked(TBHost *host) {
  if (host->wake_write < 0 || host->stopped) return;
  for (;;) {
    intptr_t sent = tb_net_send_system(host->wake_write, "w", 1, NULL);
    u32 code;
    if (sent == 1) return;
    code = sent < 0 ? tb_net_error() : 0;
#ifdef _WIN32
    if (code == WSAEINTR) continue;
#else
    if (code == EINTR) continue;
#endif
    /* A full notifier queue already contains a wake. Other errors must reach
     * the VM even when no notification can make its poll snapshot ready. */
    if (!tb_net_again(code)) host->wake_failed = true;
    return;
  }
}
static void tb_network_wake_prepare(TBIOState *owner) {
  TBHost *host = owner->host;
  TBNetRaw reader = TB_NET_INVALID, writer = TB_NET_INVALID;
  struct sockaddr_in local, from, to;
  TBNetLength length;
  if (host->wake_read >= 0) return;
  (void)io_sys_addr("127.0.0.1", 0, &local);
  reader = socket(AF_INET, SOCK_DGRAM, 0);
  writer = socket(AF_INET, SOCK_DGRAM, 0);
  if (reader == TB_NET_INVALID || writer == TB_NET_INVALID) goto failed;
  if ((u64)reader > INTPTR_MAX || (u64)writer > INTPTR_MAX) goto failed;
  if (bind(reader, (struct sockaddr *)&local, sizeof(local)) != 0
      || bind(writer, (struct sockaddr *)&local, sizeof(local)) != 0) goto failed;
  length = sizeof(from);
  if (getsockname(reader, (struct sockaddr *)&from, &length) != 0) goto failed;
  length = sizeof(to);
  if (getsockname(writer, (struct sockaddr *)&to, &length) != 0) goto failed;
  if (connect(reader, (struct sockaddr *)&to, sizeof(to)) != 0
      || connect(writer, (struct sockaddr *)&from, sizeof(from)) != 0
      || tb_net_nonblocking((intptr_t)reader) != 0 || tb_net_nonblocking((intptr_t)writer) != 0) goto failed;
  tb_lock(&host->mutex);
  host->wake_read = (intptr_t)reader; host->wake_write = (intptr_t)writer;
  if (owner->completed.head != NULL) tb_network_notify_locked(host);
  tb_unlock(&host->mutex);
  return;
failed:
#ifdef _WIN32
  if (reader != TB_NET_INVALID) (void)closesocket(reader);
  if (writer != TB_NET_INVALID) (void)closesocket(writer);
#else
  if (reader != TB_NET_INVALID) (void)close(reader);
  if (writer != TB_NET_INVALID) (void)close(writer);
#endif
  err_fail("worker notification socket setup failed");
}
static u32 tb_network_wake_drain(TBHost *host) {
  char byte;
  u32 code;
  while (tb_net_recv_system(host->wake_read, &byte, 1, NULL) >= 0) {}
  code = tb_net_error();
  return tb_net_again(code) ? 0 : code;
}

static Term tb_tcp_listen_run(Env e, Term *fields, IoWork *work) {
  intptr_t descriptor = tb_net_created(socket(AF_INET, SOCK_STREAM, 0));
  struct sockaddr_in address; int one = 1; u32 code;
  (void)work;
  if (descriptor < 0) return tb_net_fail(e, tb_net_error());
  (void)setsockopt((TBNetRaw)descriptor, SOL_SOCKET, SO_REUSEADDR, (const char *)&one, sizeof(one));
  if (io_sys_addr("0.0.0.0", (u32)fields[0], &address) != 0) code = EINVAL;
  else if (bind((TBNetRaw)descriptor, (struct sockaddr *)&address, sizeof(address)) != 0
      || listen((TBNetRaw)descriptor, 16) != 0 || tb_net_nonblocking(descriptor) != 0) code = tb_net_error();
  else return io_done(e, io_hand((u64)descriptor));
  tb_net_close(descriptor); return tb_net_fail(e, code);
}
static Term tb_tcp_accept_more(Env e, IoWork *work) {
  intptr_t accepted = tb_net_created(accept((TBNetRaw)work->hand, NULL, NULL));
  u32 code = accepted < 0 ? tb_net_error() : 0;
  if (accepted >= 0 && tb_net_nonblocking(accepted) != 0) {
    code = tb_net_error(); tb_net_close(accepted); accepted = -1;
  }
  if (tb_net_again(code)) return io_wait_on(work, work->hand, POLLIN, tb_tcp_accept_more);
  return io_tup(e, io_hand((u64)work->hand), code != 0 ? tb_net_fail(e, code) : io_done(e, io_hand((u64)accepted)));
}
static Term tb_tcp_accept_run(Env e, Term *fields, IoWork *work) {
  work->hand = tb_net_handle(fields[0]); return tb_tcp_accept_more(e, work);
}
static Term tb_tcp_connect_more(Env e, IoWork *work) {
  int error = (int)work->code;
  TBNetLength size = sizeof(error);
  if (tb_net_pending(work->code) && getsockopt((TBNetRaw)work->made, SOL_SOCKET,
      SO_ERROR, (char *)&error, &size) != 0) error = (int)tb_net_error();
  tb_host_free(work->data); work->data = NULL;
  if (error != 0) { tb_net_close(work->made); return tb_net_fail(e, (u32)error); }
  return io_done(e, io_hand((u64)work->made));
}
static Term tb_tcp_connect_run(Env e, Term *fields, IoWork *work) {
  struct sockaddr_in address;
  work->data = io_cstr(e, fields[0], &work->size); work->made = -1;
  work->code = EINVAL;
  if (!io_nul(work->data, work->size) && io_sys_addr(work->data, (u32)fields[1], &address) == 0) {
    work->made = tb_net_created(socket(AF_INET, SOCK_STREAM, 0));
    if (work->made < 0 || tb_net_nonblocking(work->made) != 0) {
      u32 code = tb_net_error();
      tb_net_close(work->made); tb_host_free(work->data); work->data = NULL;
      return tb_net_fail(e, code);
    }
    work->code = connect((TBNetRaw)work->made, (struct sockaddr *)&address, sizeof(address)) != 0
      ? tb_net_error() : 0;
  }
  return tb_net_pending(work->code) ? io_wait_on(work, work->made, POLLOUT, tb_tcp_connect_more)
      : tb_tcp_connect_more(e, work);
}
static Term tb_tcp_send_more(Env e, IoWork *work) {
  while ((u64)work->made < work->size) {
    intptr_t count;
    tb_tick();
    count = tb_net_send_system(work->hand, work->data + work->made, (u32)(work->size - (u64)work->made), NULL);
    work->code = count < 0 ? tb_net_error() : 0;
    if (tb_net_again(work->code)) return io_wait_on(work, work->hand, POLLOUT, tb_tcp_send_more);
    if (work->code != 0) break;
    if (count == 0 || (u64)count > work->size - (u64)work->made) err_fail("TCP send made invalid progress");
    work->made += count;
  }
  tb_host_free(work->data); work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), work->code != 0 ? tb_net_fail(e, work->code) : io_done(e, term_pak(CID_UNIT, 0)));
}
static Term tb_tcp_send_run(Env e, Term *fields, IoWork *work) {
  work->hand = tb_net_handle(fields[0]); work->data = io_cstr(e, fields[1], &work->size);
  work->made = 0; work->code = 0; return tb_tcp_send_more(e, work);
}
static Term tb_tcp_recv_more(Env e, IoWork *work) {
  intptr_t count = tb_net_recv_system(work->hand, work->data, (u32)work->made, NULL);
  u32 code = count < 0 ? tb_net_error() : 0;
  Term result;
  if (tb_net_again(code)) return io_wait_on(work, work->hand, POLLIN, tb_tcp_recv_more);
  result = code != 0 ? tb_net_fail(e, code) : io_done(e, io_str(e, work->data, (u64)count));
  tb_host_free(work->data); work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), result);
}
static Term tb_tcp_recv_run(Env e, Term *fields, IoWork *work) {
  work->hand = tb_net_handle(fields[0]); work->made = tb_net_receive_count(fields[1]);
  work->data = tb_net_receive_buffer((u32)work->made); return tb_tcp_recv_more(e, work);
}
static Term tb_udp_bind_run(Env e, Term *fields, IoWork *work) {
  intptr_t descriptor = tb_net_created(socket(AF_INET, SOCK_DGRAM, 0));
  struct sockaddr_in address; u32 code;
  (void)work;
  if (descriptor < 0) return tb_net_fail(e, tb_net_error());
  if (io_sys_addr("0.0.0.0", (u32)fields[0], &address) != 0) code = EINVAL;
  else if (bind((TBNetRaw)descriptor, (struct sockaddr *)&address, sizeof(address)) != 0
      || tb_net_nonblocking(descriptor) != 0) code = tb_net_error();
  else return io_done(e, io_hand((u64)descriptor));
  tb_net_close(descriptor); return tb_net_fail(e, code);
}
static Term tb_udp_send_to_more(Env e, IoWork *work) {
  struct sockaddr_in address;
  u32 code = EINVAL;
  if (io_sys_addr(work->text, (u32)work->made, &address) == 0) {
    intptr_t count = tb_net_send_system(work->hand, work->data, (u32)work->size, &address);
    code = count < 0 ? tb_net_error() : 0;
  }
  if (tb_net_again(code)) return io_wait_on(work, work->hand, POLLOUT, tb_udp_send_to_more);
  tb_host_free(work->text); tb_host_free(work->data); work->text = NULL; work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), code != 0 ? tb_net_fail(e, code) : io_done(e, term_pak(CID_UNIT, 0)));
}
static Term tb_udp_send_to_run(Env e, Term *fields, IoWork *work) {
  u64 length;
  work->hand = tb_net_handle(fields[0]); work->text = io_cstr(e, fields[1], &length);
  work->made = (intptr_t)(u32)fields[2]; work->data = io_cstr(e, fields[3], &work->size);
  if (io_nul(work->text, length)) {
    tb_host_free(work->text); tb_host_free(work->data); work->text = NULL; work->data = NULL;
    return io_tup(e, io_hand((u64)work->hand), tb_net_fail(e, EINVAL));
  }
  return tb_udp_send_to_more(e, work);
}
static Term tb_udp_recv_from_more(Env e, IoWork *work) {
  struct sockaddr_in address = {0};
  intptr_t count = tb_net_recv_system(work->hand, work->data, (u32)work->made, &address);
  u32 code = count < 0 ? tb_net_error() : 0;
  Term result;
  if (tb_net_again(code)) return io_wait_on(work, work->hand, POLLIN, tb_udp_recv_from_more);
  result = code != 0 ? tb_net_fail(e, code) : io_done(e, tb_net_packet(e, &address, work->data, (u64)count));
  tb_host_free(work->data); work->data = NULL;
  return io_tup(e, io_hand((u64)work->hand), result);
}
static Term tb_udp_recv_from_run(Env e, Term *fields, IoWork *work) {
  work->hand = tb_net_handle(fields[0]); work->made = tb_net_receive_count(fields[1]);
  work->data = tb_net_receive_buffer((u32)work->made); return tb_udp_recv_from_more(e, work);
}
static Term tb_udp_poll_run(Env e, Term *fields, IoWork *work) {
  intptr_t descriptor = tb_net_handle(fields[0]);
  u32 capacity = tb_net_receive_count(fields[1]);
  char *data = tb_net_receive_buffer(capacity);
  struct sockaddr_in address = {0};
  intptr_t count = tb_net_recv_system(descriptor, data, capacity, &address);
  u32 code = count < 0 ? tb_net_error() : 0;
  Term result;
  (void)work;
  if (tb_net_again(code)) result = io_done(e, term_pak(CID_NONE, 0));
  else if (code != 0) result = tb_net_fail(e, code);
  else result = io_done(e, io_box(e, CID_SOME, tb_net_packet(e, &address, data, (u64)count), IO_HOTS & 32));
  tb_host_free(data); return io_tup(e, fields[0], result);
}
static Term tb_socket_close_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work; tb_net_close(tb_net_handle(fields[0])); return term_pak(CID_UNIT, 0);
}
static void tb_register_network(void) {
#ifdef CID_TCP_LISTEN
  io_eff(CID_TCP_LISTEN, tb_tcp_listen_run, 0);
#endif
#ifdef CID_TCP_ACCEPT
  io_eff(CID_TCP_ACCEPT, tb_tcp_accept_run, IO_READ);
#endif
#ifdef CID_TCP_CONNECT
  io_eff(CID_TCP_CONNECT, tb_tcp_connect_run, 0);
#endif
#ifdef CID_TCP_SEND
  io_eff(CID_TCP_SEND, tb_tcp_send_run, 0);
#endif
#ifdef CID_TCP_RECV
  io_eff(CID_TCP_RECV, tb_tcp_recv_run, IO_READ);
#endif
#ifdef CID_UDP_BIND
  io_eff(CID_UDP_BIND, tb_udp_bind_run, 0);
#endif
#ifdef CID_UDP_SEND_TO
  io_eff(CID_UDP_SEND_TO, tb_udp_send_to_run, 0);
#endif
#ifdef CID_UDP_RECV_FROM
  io_eff(CID_UDP_RECV_FROM, tb_udp_recv_from_run, IO_READ);
#endif
#ifdef CID_UDP_POLL
  io_eff(CID_UDP_POLL, tb_udp_poll_run, 0);
#endif
#ifdef CID_SOCKET_CLOSE
  io_eff(CID_SOCKET_CLOSE, tb_socket_close_run, 0);
#endif
#ifdef CID_LISTENER_CLOSE
  io_eff(CID_LISTENER_CLOSE, tb_socket_close_run, 0);
#endif
}
