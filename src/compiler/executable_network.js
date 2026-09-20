// SPDX-License-Identifier: Apache-2.0
// Network effects derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Bounded buffers, resource tracking and host adaptation: TeamDman.
// See NOTICE and licenses/Apache-2.0.txt.
const $tbNetworkOwners = new Map();
let $tbNetworkHandles = 0;
let $tbNetworkBytes = 0;

function io_addr(host, port) {
  const parts = host.split('.');
  if (!Number.isInteger(port) || port < 0 || port > 65535 || parts.length !== 4
      || !parts.every(part => /^(0|[1-9]\d{0,2})$/.test(part) && Number(part) < 256)) {
    return null;
  }
  const address = new Uint8Array(16);
  address.set([...(io_sys().mac ? [16, 2] : [2, 0]), port >> 8, port & 255,
    ...parts.map(Number)]);
  return address;
}

function $tbOwnSocket(sys, fd) {
  $tbDescriptor(fd);
  if ($tbNetworkHandles >= $tbPendingLimit) {
    sys.close(fd);
    throw new Error('IO socket handle budget exhausted');
  }
  let owned = $tbNetworkOwners.get(sys);
  if (owned === undefined) { owned = new Map(); $tbNetworkOwners.set(sys, owned); }
  const key = String(fd);
  if (owned.has(key)) throw new Error('host returned an already owned socket');
  owned.set(key, fd);
  $tbNetworkHandles++;
  return fd;
}

function $tbForgetSocket(sys, fd) {
  if (typeof fd !== 'number' && typeof fd !== 'bigint') return;
  const owned = $tbNetworkOwners.get(sys);
  if (owned?.delete(String(fd))) $tbNetworkHandles--;
  if (owned?.size === 0) $tbNetworkOwners.delete(sys);
}

function $tbDropSocket(sys, fd) {
  try { sys.close(fd); }
  finally { $tbForgetSocket(sys, fd); }
}

function $tbCloseNetwork() {
  for (const [sys, owned] of $tbNetworkOwners) {
    for (const fd of owned.values()) {
      try { sys.close(fd); } catch (_) { /* Preserve the invocation's result. */ }
    }
  }
  $tbNetworkOwners.clear();
  $tbNetworkHandles = 0;
  $tbNetworkBytes = 0;
}

function $tbNetworkLength(value) {
  const length = Number(value);
  if (!Number.isSafeInteger(length) || length < 0 || length > 8388608) {
    throw new Error('IO network byte buffer budget exhausted');
  }
  return length;
}

function $tbNetworkLease(length) {
  if ($tbNetworkBytes + length > 67108864) {
    throw new Error('IO retained network buffer budget exhausted');
  }
  $tbNetworkBytes += length;
  let live = true;
  return () => { if (live) { live = false; $tbNetworkBytes -= length; } };
}

function $tbNetworkReadNeed() { return { read: true }; }
function $tbNetworkNonblocking(sys, fd) {
  return sys.fcntl(fd, 4, sys.fcntl(fd, 3, 0) | (sys.mac ? 4 : 0x800));
}

function $tbNetworkBind(port, stream) {
  const sys = io_sys();
  const fd = sys.socket(2, stream ? 1 : 2, 0);
  if (fd < 0) return io_fail(sys.errno());
  $tbOwnSocket(sys, fd);
  if (stream) {
    sys.setsockopt(fd, sys.mac ? 0xffff : 1, sys.mac ? 4 : 2,
      sys.ptr(new Int32Array([1])), 4);
  }
  const address = io_addr('0.0.0.0', Number(port));
  if (address === null) { $tbDropSocket(sys, fd); return io_fail(22); }
  if (sys.bind(fd, sys.ptr(address), 16) < 0 || (stream && sys.listen(fd, 16) < 0)
      || $tbNetworkNonblocking(sys, fd) < 0) {
    const code = sys.errno();
    $tbDropSocket(sys, fd);
    return io_fail(code);
  }
  return io_done(fd);
}
function $tbTcpListen(port) { return $tbNetworkBind(port, true); }
function $tbUdpBind(port) { return $tbNetworkBind(port, false); }

function $tbTcpAccept(listener, k) {
  const sys = io_sys();
  const go = () => {
    const fd = sys.accept(listener, null, null);
    if (fd < 0) {
      const code = sys.errno();
      if (code === (sys.mac ? 35 : 11)) { io_park_on(listener, false, k, go); return; }
      return io_tup(listener, io_fail(code));
    }
    $tbOwnSocket(sys, fd);
    if ($tbNetworkNonblocking(sys, fd) < 0) {
      const code = sys.errno();
      $tbDropSocket(sys, fd);
      return io_tup(listener, io_fail(code));
    }
    return io_tup(listener, io_done(fd));
  };
  return go();
}

function $tbTcpConnect(host, port, k) {
  const sys = io_sys();
  const address = io_addr(host, Number(port));
  if (address === null) return io_fail(22);
  const fd = sys.socket(2, 1, 0);
  if (fd < 0) return io_fail(sys.errno());
  $tbOwnSocket(sys, fd);
  const end = code => {
    if (code !== 0) { $tbDropSocket(sys, fd); return io_fail(code); }
    return io_done(fd);
  };
  const error = () => {
    const value = new Int32Array([0]);
    const length = new Uint32Array([4]);
    return sys.getsockopt(fd, sys.mac ? 0xffff : 1, sys.mac ? 0x1007 : 4,
      sys.ptr(value), sys.ptr(length)) < 0 ? sys.errno() : value[0];
  };
  const ok = $tbNetworkNonblocking(sys, fd) >= 0 && sys.connect(fd, sys.ptr(address), 16) >= 0;
  const code = ok ? 0 : sys.errno();
  if (code !== (sys.mac ? 36 : 115)) return end(code);
  io_park_on(fd, true, k, () => end(error()));
  return undefined;
}

function $tbTcpSend(socket, data, k) {
  const sys = io_sys();
  const bytes = $tbFileBytes(data);
  const release = $tbNetworkLease(bytes.length);
  const done = result => { release(); return io_tup(socket, result); };
  const go = offset => {
    while (offset < bytes.length) {
      $tbTick();
      const part = bytes.subarray(offset);
      const count = Number(sys.send(socket, sys.ptr(part), part.length, 0));
      if (count < 0) {
        const code = sys.errno();
        if (code === (sys.mac ? 35 : 11)) {
          io_park_on(socket, true, k, () => go(offset));
          return undefined;
        }
        return done(io_fail(code));
      }
      if (!Number.isSafeInteger(count) || count <= 0 || count > part.length) {
        throw new Error('invalid host socket send count');
      }
      offset += count;
    }
    return done(io_done({ $: 'Unit' }));
  };
  return go(0);
}

function $tbTcpRecv(socket, max, k) {
  const sys = io_sys();
  const length = $tbNetworkLength(max);
  const release = $tbNetworkLease(Math.max(length, 1));
  const bytes = new Uint8Array(Math.max(length, 1));
  const done = result => { release(); return io_tup(socket, result); };
  const go = () => {
    const count = Number(sys.recv(socket, sys.ptr(bytes), length, 0));
    if (count < 0) {
      const code = sys.errno();
      if (code === (sys.mac ? 35 : 11)) { io_park_on(socket, false, k, go); return; }
      return done(io_fail(code));
    }
    if (!Number.isSafeInteger(count) || count > length) throw new Error('invalid host socket receive count');
    return done(io_done(io_text(bytes, count)));
  };
  return go();
}

function $tbUdpSendTo(socket, host, port, data, k) {
  const sys = io_sys();
  const address = io_addr(host, Number(port));
  if (address === null) return io_tup(socket, io_fail(22));
  const bytes = $tbFileBytes(data);
  const release = $tbNetworkLease(bytes.length + address.length);
  const done = result => { release(); return io_tup(socket, result); };
  const go = () => {
    const count = Number(sys.sendto(socket, sys.ptr(bytes), bytes.length, 0, sys.ptr(address), 16));
    if (count < 0) {
      const code = sys.errno();
      if (code === (sys.mac ? 35 : 11)) { io_park_on(socket, true, k, go); return; }
      return done(io_fail(code));
    }
    if (!Number.isSafeInteger(count) || count !== bytes.length) throw new Error('invalid host datagram send count');
    return done(io_done({ $: 'Unit' }));
  };
  return go();
}

function $tbUdpReceive(socket, max, k, immediate) {
  const sys = io_sys();
  const length = $tbNetworkLength(max);
  const release = $tbNetworkLease(Math.max(length, 1) + 20);
  const bytes = new Uint8Array(Math.max(length, 1));
  const peer = new Uint8Array(16);
  const size = new Uint32Array([16]);
  const done = result => { release(); return io_tup(socket, result); };
  const go = () => {
    const count = Number(sys.recvfrom(socket, sys.ptr(bytes), length, 0, sys.ptr(peer), sys.ptr(size)));
    if (count < 0) {
      const code = sys.errno();
      if (code === (sys.mac ? 35 : 11)) {
        if (immediate) return done(io_done({ $: 'None' }));
        io_park_on(socket, false, k, go);
        return undefined;
      }
      return done(io_fail(code));
    }
    if (!Number.isSafeInteger(count) || count > length) throw new Error('invalid host datagram receive count');
    const host = `${peer[4]}.${peer[5]}.${peer[6]}.${peer[7]}`;
    const port = (peer[2] << 8) | peer[3];
    const value = io_tup(host, port, io_text(bytes, count));
    return done(io_done(immediate ? { $: 'Some', value } : value));
  };
  return go();
}
function $tbUdpRecvFrom(socket, max, k) { return $tbUdpReceive(socket, max, k, false); }
function $tbUdpPoll(socket, max) { return $tbUdpReceive(socket, max, undefined, true); }
function $tbSocketClose(socket) { $tbDropSocket(io_sys(), socket); return { $: 'Unit' }; }
