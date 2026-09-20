// SPDX-License-Identifier: Apache-2.0
// Cooperative scheduling derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Bounded Node timer/readiness adapter and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
// Execution-only IO driver. Foreign JavaScript is trusted host code;
// its returned values are not proof certificates or validated Bend constructors.
const $tbRequests = new WeakSet();
const $tbChannelRows = new WeakMap();
const $tbManagedProviders = new WeakMap();
let $tbIo = null;
const $tbPendingLimit = 131072;
const $tbWaitWord = new Int32Array(new SharedArrayBuffer(4));

function $tbMakeRequest(index, args, continuation) {
  const foreign = $tbForeign[index];
  const request = {
    index, args, continuation, $: '$FFI',
    run: foreign && foreign.run,
    need: foreign && foreign.need,
    kont: continuation
  };
  $tbRequests.add(request);
  return request;
}

function $tbIsRequest(value) {
  return value !== null && (typeof value === 'object' || typeof value === 'function')
    && $tbRequests.has(value);
}

function $tbBytes(text) {
  // Coerce once, with TextEncoder's string conversion, before checking size.
  // Checking first prevents a small checked string-doubling program from
  // allocating an unbounded UTF-8 buffer merely to discover it is too large.
  if (typeof text !== 'string') text = text === undefined ? '' : `${text}`;
  if (text.length > 8388608 || Buffer.byteLength(text, 'utf8') > 8388608) {
    throw new Error('console output byte budget exhausted');
  }
  return new TextEncoder().encode(text);
}

function $tbOutput(fd, text) {
  $tbOutputBytes(fd, $tbBytes(text));
}

function $tbOutputBytes(fd, bytes) {
  if (bytes.length > 8388608) throw new Error('IO byte buffer budget exhausted');
  const fs = require('fs');
  let offset = 0;
  while (offset < bytes.length) {
    $tbTick();
    let written;
    try {
      written = fs.writeSync(fd, bytes, offset, bytes.length - offset);
    } catch (error) {
      if (error && (error.code === 'EINTR' || error.code === 'EAGAIN')) continue;
      throw new Error('failed to write a standard stream', { cause: error });
    }
    if (!Number.isInteger(written) || written <= 0 || written > bytes.length - offset) {
      throw new Error('short write on a standard stream');
    }
    offset += written;
  }
}

function $tbPrint(text) {
  $tbOutput(1, text + '\n');
  return { $: 'Unit' };
}

function $tbWrite(text) {
  $tbOutput(1, text);
  return { $: 'Unit' };
}

function $tbPrintErr(text) {
  $tbOutput(2, text + '\n');
  return { $: 'Unit' };
}

function $tbSpawn(action) {
  $tbPush(action, value => ({ $: 'Emit', value }), true);
  return { $: 'Unit' };
}
function $tbSleep() { return { $: 'Unit' }; }
function $tbSleepNeed() { return { time: true }; }
function $tbNow() { return BigInt(Math.floor(performance.now())); }

function $tbPending(io, extra) {
  if (io.runs.length - io.head + io.waits.length + io.channelWaiters + extra > $tbPendingLimit) {
    throw new Error('IO pending task budget exhausted');
  }
}

// Handles deliberately retain upstream's raw row/array representation. Foreign
// code may supply rows or keep aliases; accounting is reconciled on each access.
// Arbitrary mutations performed by host code remain outside Bend's budget.
function $tbChannel(row) {
  const io = $tbIo;
  if (io === null) throw new Error('channel operation requires an active IO scheduler');
  if (row === null || typeof row !== 'object'
      || !Array.isArray(row.ring) || !Array.isArray(row.wait)) {
    throw new Error('invalid channel row');
  }
  let entry = $tbChannelRows.get(row);
  if (entry?.owner !== io) {
    if (io.channels.size >= $tbPendingLimit) throw new Error('IO channel handle budget exhausted');
    entry = { owner: io, buffered: 0, waiting: 0 };
    $tbChannelRows.set(row, entry);
    io.channels.add(row);
  }
  const buffered = io.channelBuffered + row.ring.length - entry.buffered;
  const waiting = io.channelWaiters + row.wait.length - entry.waiting;
  if (buffered > $tbPendingLimit) throw new Error('IO channel buffer budget exhausted');
  if (waiting > $tbPendingLimit) throw new Error('IO channel waiter budget exhausted');
  $tbPending(io, waiting - io.channelWaiters);
  io.channelBuffered = buffered;
  io.channelWaiters = waiting;
  entry.buffered = row.ring.length;
  entry.waiting = row.wait.length;
  return row;
}

function $tbChanNew(room) {
  return $tbChannel({ room: Number(room), ring: [], wait: [], shut: false });
}

function $tbChanWake(row, value) {
  const waiter = row.wait.shift();
  $tbChannel(row);
  $tbPush(waiter.cont, value, false);
  return waiter.item;
}

function $tbChanTake(row) {
  const value = row.ring.shift();
  $tbChannel(row);
  if (row.wait.length > 0) {
    row.ring.push($tbChanWake(row, true));
    $tbChannel(row);
  }
  return value;
}

function $tbChanSend(handle, value, continuation) {
  const row = $tbChannel(handle);
  if (row.shut) return false;
  // Upstream uses null as its receiver sentinel, including the observable
  // collision with a blocked sender whose live type/proof payload is null.
  if (row.wait.length > 0 && row.wait[0].item === null) {
    $tbChanWake(row, { $: 'Some', value });
    return true;
  }
  if (row.ring.length < row.room) {
    if ($tbIo.channelBuffered >= $tbPendingLimit) throw new Error('IO channel buffer budget exhausted');
    row.ring.push(value);
    $tbChannel(row);
    return true;
  }
  $tbPending($tbIo, 1);
  row.wait.push({ cont: continuation, item: value });
  $tbChannel(row);
  return undefined;
}

function $tbChanRecv(handle, continuation) {
  const row = $tbChannel(handle);
  if (row.ring.length > 0) {
    return { $: 'Some', value: $tbChanTake(row) };
  }
  if (row.wait.length > 0 && row.wait[0].item !== null) {
    return { $: 'Some', value: $tbChanWake(row, true) };
  }
  if (row.shut) return { $: 'None' };
  $tbPending($tbIo, 1);
  row.wait.push({ cont: continuation, item: null });
  $tbChannel(row);
  return undefined;
}

function $tbChanShut(row) {
  row.shut = true;
  // No queued continuation runs during close. Drain once, retaining the
  // original public wait array identity and the exact FIFO wake-up order.
  const waiters = row.wait.splice(0);
  $tbChannel(row);
  for (const waiter of waiters) {
    $tbPush(waiter.cont, waiter.item === null ? { $: 'None' } : false, false);
  }
}

function $tbChanClose(handle) {
  const row = $tbChannel(handle);
  if (!row.shut) $tbChanShut(row);
  return { $: 'Unit' };
}

function $tbPush(fun, arg, fresh) {
  $tbTick();
  const io = $tbIo;
  if (io === null || typeof fun !== 'function') {
    throw new Error('IO task scheduling requires an active scheduler and a function');
  }
  $tbPending(io, 1);
  if (fresh && io.live >= $tbPendingLimit) throw new Error('IO live task budget exhausted');
  io.runs.push({ fun, arg });
  if (fresh) io.live++;
}

function $tbWake(wait) {
  const value = $tbSuspendable(wait.more());
  return value === undefined ? undefined : $tbForceCall(wait.k, [value]);
}

function $tbDescriptor(fd) {
  if ((typeof fd === 'number' && Number.isSafeInteger(fd))
      || (typeof fd === 'bigint' && fd >= 0n && fd <= 0xffffffffffffffffn)) return fd;
  throw new Error('IO readiness scheduling requires a lossless descriptor');
}

function $tbPark(fd, out, k, more) {
  const io = $tbIo;
  if (io === null) throw new Error('IO readiness scheduling requires an active scheduler');
  fd = $tbDescriptor(fd);
  if (typeof k !== 'function' || typeof more !== 'function') {
    throw new Error('IO readiness scheduling requires continuation functions');
  }
  $tbPending(io, 1);
  io.waits.push({ fd, out, k, more });
}

function $tbPoll(descriptors, milliseconds) {
  const sys = io_sys();
  if (typeof sys?.poll_descriptors === 'function') {
    // This extension keeps a Windows SOCKET intact instead of truncating it
    // through POSIX's signed-i32 pollfd field. Masks retain the upstream ABI.
    const ready = $tbSuspendable(sys.poll_descriptors(descriptors, milliseconds));
    if ((!Array.isArray(ready) && !(ArrayBuffer.isView(ready) && !(ready instanceof DataView)))
        || ready.length !== descriptors.length) {
      throw new Error('IO readiness provider returned an invalid result count');
    }
    for (const mask of ready) {
      $tbTick();
      if (!Number.isInteger(mask) || mask < 0 || mask > 65535) {
        throw new Error('IO readiness provider returned an invalid event mask');
      }
    }
    return ready;
  }
  if (sys?.poll_descriptors !== undefined
      || typeof sys?.poll !== 'function'
      || (descriptors.length > 0 && typeof sys?.ptr !== 'function')) {
    throw new Error('IO readiness scheduling requires a synchronous poll provider');
  }
  const buffer = new Int32Array(descriptors.length * 2);
  for (let i = 0; i < descriptors.length; i++) {
    $tbTick();
    const { fd, events } = descriptors[i];
    if (fd < -2147483648 || fd > 2147483647) {
      throw new Error('IO readiness provider needs poll_descriptors for a full-width handle');
    }
    buffer[2 * i] = Number(fd);
    buffer[2 * i + 1] = events;
  }
  const pointer = descriptors.length > 0 ? sys.ptr(buffer) : null;
  const count = $tbSuspendable(sys.poll(pointer, descriptors.length, milliseconds));
  if (!Number.isInteger(count) || count < 0 || count > descriptors.length) {
    throw new Error('IO readiness provider returned an invalid poll count');
  }
  const ready = [];
  let observed = 0;
  for (let i = 0; i < descriptors.length; i++) {
    $tbTick();
    const mask = buffer[2 * i + 1] >>> 16;
    ready.push(mask);
    if (mask !== 0) observed++;
  }
  if (observed !== count) throw new Error('IO readiness provider returned inconsistent poll events');
  return ready;
}

function $tbWait(io) {
  let soon = Infinity;
  const descriptors = [];
  for (const wait of io.waits) {
    $tbTick();
    if (wait.at !== undefined) soon = Math.min(soon, wait.at);
    if (wait.fd !== undefined) descriptors.push({ fd: wait.fd, events: wait.out ? 4 : 1 });
  }
  const delay = Math.min(Math.max(0, Math.ceil(soon - performance.now())), 1000);
  $tbTick();
  // Poll even for overdue timers. Both adapters are synchronous; ordinary Node
  // event-loop callbacks are not pumped. Recheck the shared budget each second.
  const supplied = globalThis.BEND_SYS;
  const poll = descriptors.length > 0 || (supplied != null
    && (supplied.poll_descriptors !== undefined || supplied.poll !== undefined));
  // A supplied poller also owns zero-descriptor waits, just like upstream.
  // Default timer-only programs never need to load the native network addon.
  const ready = poll ? $tbPoll(descriptors, delay) : [];
  if (!poll) Atomics.wait($tbWaitWord, 0, 0, delay);
  const now = performance.now();
  const waiting = io.waits;
  io.waits = [];
  let index = 0;
  // Preserve the common registration order, including duplicate descriptors.
  // Any returned event (including ERR/HUP/NVAL) retries the original operation.
  for (const wait of waiting) {
    $tbTick();
    const mask = wait.fd === undefined ? 0 : ready[index++];
    if (mask !== 0 || wait.at <= now) $tbPush($tbWake, wait, false);
    else io.waits.push(wait);
  }
}

function $tbRunTasks(main) {
  if ($tbIo !== null) throw new Error('IO scheduler is already active');
  const io = { runs: [], head: 0, live: 0, waits: [], channels: new Set(), channelBuffered: 0, channelWaiters: 0 };
  $tbIo = io;
  try {
    $tbPush(main, value => ({ $: 'Emit', value }), true);
    while (true) {
      $tbTick();
      if (io.head === io.runs.length) {
        io.runs.length = 0;
        io.head = 0;
        if (io.live === 0) return 0;
        if (io.waits.length === 0) throw new Error('IO deadlock: no runnable task, timer or descriptor');
        $tbWait(io);
        continue;
      }
      const task = io.runs[io.head];
      io.runs[io.head++] = undefined;
      if (io.head >= 4096 && io.head * 2 >= io.runs.length) {
        io.runs = io.runs.slice(io.head);
        io.head = 0;
      }
      let operation = $tbForceCall(task.fun, [task.arg]);
      while (true) {
        $tbTick();
        operation = $tbSuspendable(operation);
        if (operation === undefined) break;
        if ($tbIsRequest(operation)) {
          if (typeof operation.run !== 'function') {
            throw new Error('JavaScript foreign implementation is missing');
          }
          // The hook receives no arguments, with the request as its receiver.
          const need = operation.need?.() ?? {};
          // Upstream ignores write-only hooks; writes explicitly call
          // io_park_on after their first nonblocking syscall would block.
          const fd = need.read ? operation.args[0] : null;
          if (need.time || fd !== null) {
            const request = operation;
            const more = () => request.run(...request.args, request.continuation);
            if (fd !== null) {
              $tbPark(fd, false, request.continuation, more);
            } else {
              const at = performance.now() + Number(request.args[0]);
              if (!Number.isFinite(at)) throw new Error('IO timer deadline must be finite');
              $tbPending(io, 1);
              io.waits.push({ at, k: request.continuation, more });
            }
            break;
          }
          const value = $tbSuspendable(operation.run(...operation.args, operation.continuation));
          if (value === undefined) break;
          operation = $tbForceCall(operation.continuation, [value]);
        } else if (operation && operation.$ === 'Emit') {
          if (io.live <= 0) throw new Error('IO completion has no live task');
          io.live--;
          break;
        } else if (operation && operation.$ === 'Halt') {
          $tbOutput(2, operation.message + '\n');
          return operation.code >>> 0;
        } else {
          throw new Error('IO action returned neither an operation nor a foreign request');
        }
      }
    }
  } finally {
    for (const row of io.channels) {
      // Drop private accounting without changing raw rows retained by foreign
      // code. Upstream leaves their buffers, waiters and closed flag observable.
      $tbChannelRows.delete(row);
    }
    io.channels.clear();
    io.channelBuffered = 0;
    io.channelWaiters = 0;
    io.runs.length = 0;
    io.waits.length = 0;
    io.live = 0;
    $tbIo = null;
    // Only the network companion owns/cleans its handles. Foreign providers
    // remain authoritative; cleanup must never replace the original failure.
    try { if (typeof $tbCloseNetwork === 'function') $tbCloseNetwork(); }
    catch (_) { /* Closing an owned descriptor is best effort on every exit. */ }
  }
}

// Companion modules use these upstream runtime helper names. The driver calls
// its private-prefixed helpers, so companion-local declarations cannot replace
// bundled console behavior. These are runtime utilities, not foreign contracts.
function io_out(fd, bytes) { $tbOutputBytes(fd, bytes); }
function io_bytes(text) { return $tbBytes(text); }
function io_text(bytes, length) {
  const selected = bytes.subarray(0, length);
  if (selected.length > 8388608) throw new Error('IO byte buffer budget exhausted');
  return new TextDecoder().decode(selected);
}
function io_errs(message) { $tbOutput(2, message + '\n'); }
function io_done(value) { return { $: 'Done', value }; }
function io_tup(...values) {
  return values.reduceRight((snd, fst) => ({ $: 'Tuple', fst, snd }));
}
function $tbManagedSys(provider) {
  if (provider === null || (typeof provider !== 'object' && typeof provider !== 'function')) return provider;
  let view = $tbManagedProviders.get(provider);
  if (view !== undefined) return view;
  const methods = new Map();
  // Use a separate proxy target: a frozen provider may have nonconfigurable
  // methods, whose identity a proxy directly over that object could not change.
  view = new Proxy({}, {
    get(_target, key) {
      const method = Reflect.get(provider, key, provider);
      if (typeof method !== 'function') return method;
      const cached = methods.get(key);
      if (cached?.method === method) return cached.call;
      const call = (...args) => {
        if (key !== 'close') return Reflect.apply(method, provider, args);
        try { return Reflect.apply(method, provider, args); }
        finally {
          // The original provider and raw descriptor remain authoritative.
          // Forget only its tracked ownership, including on a close failure.
          if (typeof $tbForgetSocket === 'function') $tbForgetSocket(view, args[0]);
        }
      };
      methods.set(key, { method, call });
      return call;
    },
    has(_target, key) { return Reflect.has(provider, key); },
    set(_target, key, value) { return Reflect.set(provider, key, value, provider); },
    deleteProperty(_target, key) { return Reflect.deleteProperty(provider, key); },
    ownKeys() { return Reflect.ownKeys(provider); },
    getOwnPropertyDescriptor(_target, key) {
      const descriptor = Reflect.getOwnPropertyDescriptor(provider, key);
      return descriptor === undefined ? undefined : { ...descriptor, configurable: true };
    }
  });
  $tbManagedProviders.set(provider, view);
  $tbManagedProviders.set(view, view);
  return view;
}
function io_sys() { return $tbManagedSys(globalThis.BEND_SYS ?? $tbHostSys()); }
function io_fail(code) {
  return { $: 'Fail', error: io_tup(code >>> 0, String(io_sys().strerror(code))) };
}
function io_push(fun, arg, fresh) { $tbPush(fun, arg, fresh); }
function io_park_on(fd, out, k, more) { $tbPark(fd, out, k, more); }
function chan_wake(row, value) { return $tbChanWake($tbChannel(row), value); }
function chan_take(row) { return $tbChanTake($tbChannel(row)); }
function chan_shut(row) { $tbChanShut($tbChannel(row)); }

function $tbSuspendable(value) {
  if (value !== null && (typeof value === 'object' || typeof value === 'function')
      && typeof value.then === 'function') {
    throw new Error('asynchronous foreign results are not supported');
  }
  return value;
}

function $tbSynchronous(value) {
  if (value === undefined) throw new Error('asynchronous IO suspension is not supported');
  return $tbSuspendable(value);
}

function $tbRunMain(mainThunk, isIo, purePrinter) {
  try {
    if (mainThunk === null) {
      $tbOutput(1, 'All terms check.\n');
      return 0;
    }
    const main = $tbSynchronous($tbForceCall(mainThunk, []));
    if (!isIo) {
      if ($tbIsRequest(main)) throw new Error('foreign request outside the IO driver');
      $tbOutput(1, purePrinter(main) + '\n');
      return 0;
    }
    return $tbRunTasks(main);
  } catch (error) {
    const message = $tbIsRequest(error)
      ? 'foreign request inspected outside the IO driver'
      : String(error && error.message !== undefined ? error.message : error);
    // Reporting must still work after the evaluation budget has been exhausted.
    try { require('fs').writeSync(2, 'teamy-bend generated program: ' + message + '\n'); }
    catch (_) { /* A failed diagnostic stream cannot turn failure into success. */ }
    return 1;
  }
}
