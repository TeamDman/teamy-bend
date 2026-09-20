// SPDX-License-Identifier: Apache-2.0
// Cooperative scheduling derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Bounded Node timer adapter and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
// Execution-only IO driver. Foreign JavaScript is trusted host code;
// its returned values are not proof certificates or validated Bend constructors.
const $tbRequests = new WeakSet();
const $tbChannelRows = new WeakMap();
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

function $tbWait(io) {
  let soon = Infinity;
  for (const wait of io.waits) { $tbTick(); soon = Math.min(soon, wait.at); }
  const delay = Math.ceil(soon - performance.now());
  $tbTick();
  // Poll even for overdue timers, matching upstream's scheduling point.
  // Timer-only Node waits are synchronous: promises and ordinary event-loop
  // callbacks are not pumped. Long sleeps recheck the shared budget each second.
  Atomics.wait($tbWaitWord, 0, 0, Math.min(Math.max(0, delay), 1000));
  const now = performance.now();
  const waiting = io.waits;
  io.waits = [];
  // If several deadlines are overdue, upstream resumes registration order.
  for (const wait of waiting) {
    $tbTick();
    if (wait.at <= now) $tbPush($tbWake, wait, false);
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
        if (io.waits.length === 0) throw new Error('IO deadlock: no runnable task or timer');
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
          if (need.read || need.write) throw new Error('IO readiness scheduling is not supported');
          if (need.time) {
            const at = performance.now() + Number(operation.args[0]);
            if (!Number.isFinite(at)) throw new Error('IO timer deadline must be finite');
            $tbPending(io, 1);
            const request = operation;
            io.waits.push({ at, k: request.continuation,
              more: () => request.run(...request.args, request.continuation) });
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
function io_sys() { throw new Error('native system FFI is not supported by this JavaScript runtime'); }
function io_fail(code) {
  return { $: 'Fail', error: io_tup(code >>> 0, String(io_sys().strerror(code))) };
}
function io_push(fun, arg, fresh) { $tbPush(fun, arg, fresh); }
function io_park_on() { throw new Error('IO readiness scheduling is not supported'); }
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
