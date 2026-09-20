// SPDX-License-Identifier: MPL-2.0
// Run with: node native/node-sys/tests/provider.cjs <built shared library>
const assert = require('node:assert/strict');
const path = require('node:path');
const addon = { exports: {} };
process.dlopen(addon, path.resolve(process.argv[2]));
const sys = addon.exports.create_sys();
const sockets = new Set();
function socket(kind) {
  const fd = sys.socket(2, kind, 0);
  assert(fd >= 0, sys.strerror(sys.errno()));
  sockets.add(fd);
  assert.equal(sys.fcntl(fd, 4, 0x800), 0);
  return fd;
}
function address(port) {
  return new Uint8Array([2, 0, port >> 8, port & 255, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
}
function local(fd) {
  const bytes = new Uint8Array(16);
  assert.equal(sys.getsockname(fd, bytes, new Uint32Array([16])), 0);
  return bytes;
}
function port(bytes) { return bytes[2] * 256 + bytes[3]; }
try {
  assert.equal(sys.mac, false);
  const receiver = socket(2);
  const sender = socket(2);
  assert.equal(sys.bind(receiver, address(0), 16), 0);
  assert.equal(sys.bind(sender, address(0), 16), 0);
  const receiverAddress = local(receiver);
  const senderAddress = local(sender);
  const backing = new Uint8Array([9, 65, 66, 67, 9]);
  assert.equal(sys.sendto(sender, backing.subarray(1, 4), 3, 0, receiverAddress, 16), 3);
  const registrations = Array.from({length: 70}, () => ({fd: BigInt(receiver), events: 1}));
  assert.deepEqual(sys.poll_descriptors(registrations, 1000), Array(70).fill(1));
  const legacyBacking = new Int32Array([99, 99, Number(receiver), 1, 99, 99]);
  assert.equal(sys.poll(legacyBacking.subarray(2, 4), 1, -1), 1);
  assert.equal(legacyBacking[3], 1 | (1 << 16));
  assert.deepEqual([...legacyBacking.subarray(0, 2)], [99, 99]);
  const output = new Uint8Array([9, 9, 9, 9]);
  const peer = new Uint8Array(16);
  const peerLengthBacking = new Uint32Array([99, 16, 99]);
  assert.equal(sys.recvfrom(receiver, output.subarray(1, 3), 2, 0, peer, peerLengthBacking.subarray(1, 2)), 2);
  assert.deepEqual([...output], [9, 65, 66, 9]);
  assert.equal(port(peer), port(senderAddress));
  assert.deepEqual([...peerLengthBacking], [99, 16, 99]);
  assert.equal(sys.recvfrom(receiver, output, 4, 0, peer, new Uint32Array([16])), -1);
  assert.equal(sys.errno(), 11);
  assert.equal(sys.sendto(sender, new Uint8Array([7]), 1, 0, receiverAddress, 16), 1);
  assert.equal(sys.recvfrom(receiver, new Uint8Array(0), 0, 0, peer, new Uint32Array([16])), 0);
  assert.equal(port(peer), port(senderAddress));
  assert.deepEqual(sys.poll_descriptors([{fd: receiver, events: 1}], 0), [0]);
  assert.equal(sys.sendto(sender, new Uint8Array(0), 0, 0, receiverAddress, 16), 0);
  assert.equal(sys.recvfrom(receiver, new Uint8Array(0), 0, 0, peer, new Uint32Array([16])), 0);
  assert.equal(sys.send(sender, output, 5, 0), -1);
  assert.equal(sys.errno(), 22);
  assert.equal(sys.send(sender, output, 0xffffffff, 0), -1);
  assert.equal(sys.errno(), 22);
  for (const bad of [0.5, NaN, Infinity, -1, 2 ** 32, 2 ** 32 + 1]) {
    assert.equal(sys.sendto(sender, output, bad, 0, receiverAddress, 16), -1);
    assert.equal(sys.errno(), 22);
    assert.deepEqual(sys.poll_descriptors([{fd: receiver, events: 1}], 0), [0]);
    assert.equal(sys.poll(null, bad, 0), -1);
    assert.equal(sys.errno(), 22);
  }
  for (const bad of [0.5, NaN, Infinity, 2 ** 32 - 1]) {
    assert.equal(sys.poll(null, 0, bad), -1);
    assert.throws(() => sys.poll_descriptors([], bad));
    assert.throws(() => sys.poll_descriptors([{fd: receiver, events: bad}], 0));
  }
  const oversized = new Array(131073);
  Object.defineProperty(oversized, 0, {get() { throw new Error('must not read oversized registrations'); }});
  assert.throws(() => sys.poll_descriptors(oversized, 0), /bounds exceeded/);
  assert.throws(() => sys.recv(receiver, new Uint8Array(new SharedArrayBuffer(8)), 8, 0), /shared/);
  assert.equal(sys.close(1.5), -1);
  assert.equal(sys.close(Number.MAX_SAFE_INTEGER + 1), -1);
  assert.equal(sys.close(1n << 64n), -1);

  const listener = socket(1);
  const client = socket(1);
  assert.equal(sys.setsockopt(listener, 1, 2, new Int32Array([1]), 4), 0);
  assert.equal(sys.bind(listener, address(0), 16), 0);
  assert.equal(sys.listen(listener, 16), 0);
  assert.equal(sys.accept(listener, null, null), -1);
  assert.equal(sys.errno(), 11);
  const connected = sys.connect(client, local(listener), 16);
  assert(connected === 0 || connected === -1 && sys.errno() === 115);
  assert(sys.poll_descriptors([{fd: client, events: 4}, {fd: listener, events: 1}], 1000).some(Boolean));
  const accepted = sys.accept(listener, null, null);
  assert(accepted >= 0);
  sockets.add(accepted);
  assert.equal(sys.fcntl(accepted, 4, 0x800), 0);
  const error = new Int32Array(1);
  assert.equal(sys.getsockopt(client, 1, 4, error, new Uint32Array([4])), 0);
  assert.equal(error[0], 0);
  assert.equal(sys.send(client, backing.subarray(1, 4), 3, 0), 3);
  assert(sys.poll_descriptors([{fd: accepted, events: 1}], 1000)[0] & 1);
  const data = new Uint8Array(3);
  assert.equal(sys.recv(accepted, data, 3, 0), 3);
  assert.deepEqual([...data], [65, 66, 67]);
  assert.equal(sys.close(client), 0);
  sockets.delete(client);
  assert.deepEqual(sys.poll_descriptors([{fd: client, events: 1}], 0), [32]);
  if (process.platform === 'win32') {
    assert.deepEqual(sys.poll_descriptors([{fd: (1n << 64n) - 1n, events: 1}], 0), [32]);
  }
  assert(sys.poll_descriptors([{fd: accepted, events: 1}], 1000)[0]);
  assert.equal(sys.recv(accepted, data, 3, 0), 0);
  const started = performance.now();
  assert.equal(sys.poll(null, 0, 5), 0);
  assert(performance.now() - started >= 3);
  console.log('provider: real TCP/UDP, 70 ordered registrations, truncation, raw BigInt handles, view offsets and bounds passed');
} finally {
  for (const fd of sockets) sys.close(fd);
}
