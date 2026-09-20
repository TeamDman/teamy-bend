// SPDX-License-Identifier: Apache-2.0
// File effects derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Bounded Node adapter and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
const $tbOpenFiles = new Set();
let $tbNodeFileSys;

// Upstream uses Bun's libc FFI. Node's synchronous descriptor API preserves the
// same one-read/current-offset behavior. A supplied BEND_SYS remains authoritative
// for foreign modules and libc messages. The default text table is deliberately
// limited to common file errors; other errors use Node's libuv message spelling.
function $tbFileSys() {
  if ($tbNodeFileSys !== undefined) return $tbNodeFileSys;
  const errors = require('node:util').getSystemErrorMap();
  const constants = require('node:os').constants.errno;
  const messages = {
    EPERM: 'Operation not permitted', ENOENT: 'No such file or directory',
    EINTR: 'Interrupted system call', EIO: 'Input/output error',
    EBADF: 'Bad file descriptor', EAGAIN: 'Resource temporarily unavailable',
    ENOMEM: 'Cannot allocate memory', EACCES: 'Permission denied',
    EEXIST: 'File exists', ENOTDIR: 'Not a directory', EISDIR: 'Is a directory',
    EINVAL: 'Invalid argument', ENFILE: 'Too many open files in system',
    EMFILE: 'Too many open files', EFBIG: 'File too large',
    ENOSPC: 'No space left on device', ESPIPE: 'Illegal seek',
    EROFS: 'Read-only file system', EPIPE: 'Broken pipe',
    ENAMETOOLONG: 'File name too long', ENOTEMPTY: 'Directory not empty',
    ELOOP: 'Too many levels of symbolic links',
    EILSEQ: 'Invalid or incomplete multibyte or wide character'
  };
  let lastError = 0;
  $tbNodeFileSys = {
    ptr: bytes => bytes,
    errno: () => lastError,
    read(fd, bytes, length) {
      if (!Number.isInteger(fd) || fd < 0 || fd > 2147483647) {
        lastError = constants.EBADF;
        return -1;
      }
      try { return require('node:fs').readSync(fd, bytes, 0, length, null); }
      catch (error) { lastError = Math.abs(error.errno ?? 5); return -1; }
    },
    strerror(code) {
      const entry = errors.get(-Number(code));
      const name = entry?.[0] ?? Object.keys(constants).find(name => constants[name] === code);
      // Upstream's JS path validator explicitly uses these POSIX codes, even
      // when the Node host itself uses Windows CRT errno numbers.
      if (code === (process.platform === 'darwin' ? 92 : 84)) {
        return process.platform === 'darwin' ? 'Illegal byte sequence' : messages.EILSEQ;
      }
      return messages[name] ?? entry?.[1] ?? `Unknown error ${code}`;
    }
  };
  return $tbNodeFileSys;
}

function $tbFileBytes(text) {
  if (typeof text !== 'string') text = text === undefined ? '' : `${text}`;
  if (text.length > 8388608 || Buffer.byteLength(text, 'utf8') > 8388608) {
    throw new Error('IO byte buffer budget exhausted');
  }
  return new TextEncoder().encode(text);
}

function $tbGetEnv(name) {
  $tbFileBytes(name);
  const value = Object.hasOwn(process.env, name) ? process.env[name] : undefined;
  if (value === undefined) return io_fail(2);
  $tbFileBytes(value);
  return io_done(value);
}

function $tbFileOpen(path, mode) {
  const name = $tbFileBytes(path);
  if (name.includes(0)) return io_fail(process.platform === 'darwin' ? 92 : 84);
  if (!['r', 'w', 'a'].includes(mode)) return io_fail(22);
  if ($tbOpenFiles.size >= $tbPendingLimit) throw new Error('IO file handle budget exhausted');
  let file;
  try {
    file = require('node:fs').openSync(name.length > 0 ? Buffer.from(name) : '', mode, 0o644);
  } catch (error) { return io_fail(-error.errno); }
  $tbOpenFiles.add(file);
  return io_done(file);
}

function $tbFileReadBuffer(file, max, pack) {
  const length = Math.min(max, 2147483647);
  if (!Number.isSafeInteger(length) || length < 0 || length > 8388608) {
    throw new Error('IO byte buffer budget exhausted');
  }
  const bytes = new Uint8Array(Math.max(length, 1));
  const sys = io_sys();
  const count = Number(sys.read(file, sys.ptr(bytes), length));
  if (count < 0) return io_tup(file, io_fail(sys.errno()));
  if (!Number.isSafeInteger(count) || count > length) throw new Error('invalid host file read count');
  return io_tup(file, io_done(pack(bytes, count)));
}

function $tbFileRead(file, max) {
  return $tbFileReadBuffer(file, max,
    (bytes, count) => new TextDecoder().decode(bytes.subarray(0, count)));
}

function $tbFileReadBytes(file, max) {
  return $tbFileReadBuffer(file, max, (bytes, count) => {
    let result = { $: 'Nil' };
    for (let index = count; index > 0; index--) {
      $tbTick();
      result = { $: 'Con', head: bytes[index - 1], tail: result };
    }
    return result;
  });
}

function $tbFileWrite(file, data) {
  const bytes = $tbFileBytes(data);
  let offset = 0;
  while (offset < bytes.length) {
    $tbTick();
    let count;
    try {
      count = require('node:fs').writeSync(file, bytes, offset, bytes.length - offset, null);
    } catch (error) { return io_tup(file, io_fail(Math.abs(error.errno ?? 5))); }
    if (!Number.isSafeInteger(count) || count <= 0 || count > bytes.length - offset) {
      throw new Error('short write on a file');
    }
    offset += count;
  }
  return io_tup(file, io_done({ $: 'Unit' }));
}

function $tbFileClose(file) {
  try { require('node:fs').closeSync(file); } catch (_) { /* Upstream discards close failures. */ }
  $tbOpenFiles.delete(file);
  return { $: 'Unit' };
}
