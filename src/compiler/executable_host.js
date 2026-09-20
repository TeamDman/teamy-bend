// SPDX-License-Identifier: MPL-2.0
// Node's file syscalls remain available without loading a socket addon.
let $tbDefaultHost;
let $tbNativeNetwork;

function $tbLoadNetwork() {
  if ($tbNativeNetwork !== undefined) return $tbNativeNetwork;
  const fs = require('node:fs');
  const path = require('node:path');
  const configured = process.env.TEAMY_BEND_SYS_MODULE;
  const candidates = configured ? [path.resolve(configured)] : [
    'teamy-bend-sys.node', 'teamy_bend_sys.dll',
    'libteamy_bend_sys.so', 'libteamy_bend_sys.dylib'
  ].map(name => path.join(__dirname, name));
  const selected = candidates.find(candidate => fs.existsSync(candidate));
  if (selected === undefined) {
    throw new Error('networking requires the teamy-bend Node-API provider beside this program or TEAMY_BEND_SYS_MODULE');
  }
  let provider;
  if (/\.(?:c?js)$/.test(selected)) provider = require(selected);
  else {
    const module = { exports: {} };
    process.dlopen(module, selected);
    provider = module.exports;
  }
  if (typeof provider?.create_sys === 'function') provider = provider.create_sys();
  if (provider === null || typeof provider !== 'object'
      || typeof provider.socket !== 'function' || typeof provider.poll_descriptors !== 'function') {
    throw new Error('invalid teamy-bend Node-API socket provider');
  }
  $tbNativeNetwork = provider;
  return provider;
}

function $tbHostSys() {
  if ($tbDefaultHost !== undefined) return $tbDefaultHost;
  const files = $tbFileSys();
  let last = files;
  const host = {
    get mac() { return $tbLoadNetwork().mac === true; },
    ptr: bytes => bytes,
    errno: () => last.errno(),
    read(...args) { last = files; return files.read(...args); },
    strerror(code) {
      // Preserve the existing file adapter's spelling. Winsock failures retain
      // their native numbers; the provider supplies their operating-system text.
      return code >= 10000 ? $tbLoadNetwork().strerror(code) : files.strerror(code);
    }
  };
  for (const name of ['socket', 'bind', 'listen', 'accept', 'connect', 'fcntl',
    'setsockopt', 'getsockopt', 'getsockname', 'send', 'recv', 'sendto', 'recvfrom',
    'close', 'poll', 'poll_descriptors']) {
    host[name] = (...args) => {
      const provider = $tbLoadNetwork();
      if (typeof provider[name] !== 'function') throw new Error(`socket provider is missing ${name}`);
      last = provider;
      return provider[name](...args);
    };
  }
  $tbDefaultHost = host;
  return host;
}
