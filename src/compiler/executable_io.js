// SPDX-License-Identifier: MPL-2.0
// Execution-only synchronous IO driver. Foreign JavaScript is trusted host code;
// its returned values are not proof certificates or validated Bend constructors.
const $tbRequests = new WeakSet();

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
function io_push() { throw new Error('IO task scheduling is not supported'); }
function io_park_on() { throw new Error('IO readiness scheduling is not supported'); }

function $tbSynchronous(value) {
  if (value === undefined) throw new Error('asynchronous IO suspension is not supported');
  if (value !== null && (typeof value === 'object' || typeof value === 'function')
      && typeof value.then === 'function') {
    throw new Error('asynchronous foreign results are not supported');
  }
  return value;
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
    let operation = $tbForceCall(main, [value => ({ $: 'Emit', value })]);
    while (true) {
      $tbTick();
      operation = $tbSynchronous(operation);
      if ($tbIsRequest(operation)) {
        if (typeof operation.run !== 'function') {
          throw new Error('JavaScript foreign implementation is missing');
        }
        if (typeof operation.need === 'function') {
          throw new Error('foreign _need scheduling is not supported');
        }
        // Preserve the upstream calling convention, including the trailing
        // continuation argument. Synchronous callbacks are ordinary closures.
        const value = $tbSynchronous(operation.run(...operation.args, operation.continuation));
        operation = $tbForceCall(operation.continuation, [value]);
      } else if (operation && operation.$ === 'Emit') {
        return 0;
      } else if (operation && operation.$ === 'Halt') {
        $tbOutput(2, operation.message + '\n');
        return operation.code >>> 0;
      } else {
        throw new Error('IO action returned neither an operation nor a foreign request');
      }
    }
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
