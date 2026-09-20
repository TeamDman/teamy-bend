// SPDX-License-Identifier: Apache-2.0
// Native representations derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
// Input definitions retain their own licenses.
'use strict';
const $tb = (() => {
  const raw = Symbol('Bend raw closure');
  const jumping = Symbol('Bend tail call');
  let steps = 2000000;
  let depth = 0;
  const maxArray = 131072;
  const maxBytes = 8388608;
  function tick() {
    if (--steps < 0) throw new Error('executable JavaScript step budget exhausted');
  }
  function jump(fn, args) { return { [jumping]: true, fn, args }; }
  function run(value) {
    while (value !== null && typeof value === 'object' && value[jumping] === true) {
      tick();
      value = value.fn(...value.args);
    }
    return value;
  }
  function invoke(fn, args) {
    if (++depth > 512) { --depth; throw new Error('executable JavaScript call depth budget exhausted'); }
    try {
      return run(fn(...args));
    } finally { --depth; }
  }
  function closure(body) {
    const fn = value => invoke(body, [value]);
    Object.defineProperty(fn, raw, { value: body });
    return fn;
  }
  function apply(fn, args, tail) {
    for (let i = 0; i < args.length; i++) {
      tick();
      if (typeof fn !== 'function') throw new Error('application of a non-function');
      const body = fn[raw] ?? fn;
      if (tail && i + 1 === args.length) return jump(body, [args[i]]);
      fn = invoke(body, [args[i]]);
    }
    return fn;
  }
  function forceCall(fn, args) {
    if (args.length === 0) {
      if (typeof fn !== 'function') throw new Error('entry is not a function');
      return invoke(fn, []);
    }
    return run(apply(fn, args, true));
  }
  function curry(arity, fn, args = []) {
    if (args.length === arity) return fn(...args);
    return closure(value => {
      // Array spread canonicalizes NaN payloads in V8; slice/push retains them.
      const next = args.slice();
      next.push(value);
      return curry(arity, fn, next);
    });
  }
  function nat(value) {
    if (value > 0xFFFFFFFFFFFFn) throw new Error('a Nat past the largest immediate 2^48-1');
    return value;
  }
  function bits(value) { return new Uint32Array(new Float32Array([value]).buffer)[0]; }
  function fromBits(value) { return new Float32Array(new Uint32Array([value]).buffer)[0]; }
  function wordToU32(word) {
    let result = 0;
    for (let i = 0; i < 32; i++) {
      tick();
      if (word?.$ !== 'WCon') throw new Error('expected a 32-bit Word');
      if (word.head) result = (result | (1 << i)) >>> 0;
      word = word.tail;
    }
    if (word?.$ !== 'WNil') throw new Error('expected exactly 32 Word bits');
    return result;
  }
  function u32ToWord(value) {
    let result = {$: 'WNil'};
    for (let i = 31; i >= 0; i--) {
      tick();
      result = {$: 'WCon', head: ((value >>> i) & 1) !== 0, tail: result};
    }
    return result;
  }
  function charNew(code) {
    if (code > 0x10FFFF || code >= 0xD800 && code <= 0xDFFF) {
      throw new Error(String(code) + ' is not a Unicode scalar value');
    }
    return String.fromCodePoint(code);
  }
  function boundedArray(value) {
    if (value.length > maxArray) throw new Error('executable JavaScript array element budget exhausted');
    return value;
  }
  function appendText(left, right) {
    if (typeof left === 'string' && typeof right === 'string') {
      if (left.length + right.length > maxBytes) throw new Error('executable JavaScript string byte budget exhausted');
      const end = left.charCodeAt(left.length - 1);
      const start = right.charCodeAt(0);
      const joinedSurrogate = end >= 0xD800 && end <= 0xDBFF && start >= 0xDC00 && start <= 0xDFFF;
      const bytes = Buffer.byteLength(left, 'utf8') + Buffer.byteLength(right, 'utf8') - (joinedSurrogate ? 2 : 0);
      if (bytes > maxBytes) throw new Error('executable JavaScript string byte budget exhausted');
    }
    const text = left + right;
    if (typeof text === 'string' && Buffer.byteLength(text, 'utf8') > maxBytes) {
      throw new Error('executable JavaScript string byte budget exhausted');
    }
    return text;
  }
  function construct(owner, tag, fields, values) {
    tick();
    switch (owner) {
      case 'Nat': return tag === 'Zero' ? 0n : nat(values[0] + 1n);
      case 'Bool': return tag === 'True';
      case 'U32': return wordToU32(values[0]);
      case 'F32': return fromBits(wordToU32(values[0]));
      case 'Char': return charNew(values[0]);
      case 'String': return tag === 'SNil' ? '' : appendText(values[0], values[1]);
      case 'Array': {
        if (tag === 'ALeaf') return [values[0]];
        if (values[0].length + values[1].length > maxArray) {
          throw new Error('executable JavaScript array element budget exhausted');
        }
        return boundedArray(values[0].concat(values[1]));
      }
      default: {
        const result = {$: tag};
        for (let i = 0; i < fields.length; i++) {
          Object.defineProperty(result, fields[i], { value: values[i], enumerable: true, writable: true, configurable: true });
        }
        return result;
      }
    }
  }
  function matches(owner, tag, value) {
    switch (owner) {
      case 'Nat': return tag === 'Zero' ? value === 0n : value !== 0n;
      case 'Bool': return tag === 'False' ? !value : !!value;
      case 'String': return tag === 'SNil' ? value === '' : value !== '';
      case 'Array': return tag === 'ALeaf' ? value.length === 1 : value.length !== 1;
      default: return value?.$ === tag;
    }
  }
  function field(owner, tag, value, index, name) {
    switch (owner) {
      case 'Nat': return value - 1n;
      case 'U32': return u32ToWord(value);
      case 'F32': return u32ToWord(bits(value));
      case 'Char': return value.codePointAt(0);
      case 'String': {
        const width = value.codePointAt(0) > 0xFFFF ? 2 : 1;
        return index === 0 ? value.slice(0, width) : value.slice(width);
      }
      case 'Array': return tag === 'ALeaf' ? value[0]
        : index === 0 ? value.slice(0, value.length >> 1) : value.slice(value.length >> 1);
      default: return value[name];
    }
  }
  function cmp(a,b) { return {$: a < b ? 'LT' : a === b ? 'EQ' : 'GT'}; }
  function tuple(fst,snd) { return {$:'Tuple',fst,snd}; }
  function native(name, args) {
    tick();
    const [a,b,c] = args;
    switch (name) {
      case 'U32.add': return (a + b) >>> 0;
      case 'U32.sub': return (a - b) >>> 0;
      case 'U32.mul': return Math.imul(a,b) >>> 0;
      case 'U32.div': return b === 0 ? 0 : (a / b) >>> 0;
      case 'U32.mod': return b === 0 ? a : a % b;
      case 'U32.and': return (a & b) >>> 0;
      case 'U32.or': return (a | b) >>> 0;
      case 'U32.xor': return (a ^ b) >>> 0;
      case 'U32.not': return ~a >>> 0;
      case 'U32.inc': return (a + 1) >>> 0;
      case 'U32.shl': return (a << 1) >>> 0;
      case 'U32.shr': return a >>> 1;
      case 'U32.shln': return b >= 32n ? 0 : (a << Number(b)) >>> 0;
      case 'U32.shrn': return b >= 32n ? 0 : a >>> Number(b);
      case 'U32.is_eq': case 'F32.is_eq': return a === b;
      case 'U32.is_ne': case 'F32.is_ne': return a !== b;
      case 'U32.is_lt': case 'F32.is_lt': case 'Nat.is_lt': return a < b;
      case 'U32.is_le': case 'F32.is_le': return a <= b;
      case 'U32.is_gt': case 'F32.is_gt': return a > b;
      case 'U32.is_ge': case 'F32.is_ge': return a >= b;
      case 'U32.is_zero': return a === 0;
      case 'U32.cmp': case 'Nat.cmp': return cmp(a,b);
      case 'U32.to_nat': return BigInt(a);
      case 'U32.from_nat': return Number(a & 0xFFFFFFFFn);
      case 'U32.to_f32': return Math.fround(a);
      case 'F32.to_u32': return a >= 1 && a < 4294967296 ? Math.floor(a) : 0;
      case 'F32.add': return Math.fround(a+b);
      case 'F32.sub': return Math.fround(a-b);
      case 'F32.mul': return Math.fround(a*b);
      case 'F32.div': return Math.fround(a/b);
      case 'F32.mod': return Math.fround(a%b);
      case 'F32.neg': return -a;
      case 'F32.abs': return Math.fround(Math.abs(a));
      case 'F32.bits': return bits(a);
      case 'Nat.add': return nat(a+b);
      case 'Nat.sub': return a < b ? 0n : a-b;
      case 'Nat.mul': return nat(a*b);
      case 'Nat.double': return nat(a << 1n);
      case 'Nat.divmod': return tuple(b === 0n ? 0n : a/b, b === 0n ? a : a%b);
      case 'Bool.or': return a || b;
      case 'Bool.xor': return a !== b;
      case 'String.append': return appendText(a,b);
      case 'Array.new': {
        if (a > 31n) throw new Error('an array past the deepest block class 31');
        const size = 2 ** Number(a);
        if (size > maxArray) throw new Error('executable JavaScript array element budget exhausted');
        return Array(size).fill(b);
      }
      case 'Array.set': a[b % a.length] = c; return a;
      case 'Array.get': return tuple(a,a[b % a.length]);
      case 'Array.swap': { const index = b % a.length; const old = a[index]; a[index] = c; return tuple(a,old); }
      case 'Array.size': return tuple(a,a.length);
      case 'Array.clone': return tuple(a,a.slice());
      default: throw new Error('unsupported native operation ' + name);
    }
  }
  function floatText(value) {
    if (Number.isNaN(value)) return 'nan';
    if (!Number.isFinite(value)) return value < 0 ? '-inf' : 'inf';
    if (Object.is(value,-0)) return '-0';
    let text = 'x';
    for (let precision = 1; precision <= 9 && Math.fround(Number(text)) !== value; precision++) {
      text = String(Number(value.toExponential(precision - 1)));
    }
    return text;
  }
  function escaped(character, quote) {
    const code = character.codePointAt(0);
    const escape = ({10:'n',9:'t',13:'r',0:'0',92:'\\'})[code] ?? (character === quote ? quote : null);
    return escape !== null ? '\\' + escape : code < 32 || code === 127 ? '\\u{' + code.toString(16) + '}' : character;
  }
  function show(types, root, value) {
    let nodes = 0;
    let bytes = 0;
    const parts = [];
    function write(text) {
      if (text.length > maxBytes || (bytes += Buffer.byteLength(text, 'utf8')) > maxBytes) {
        throw new Error('pure output byte budget exhausted');
      }
      parts.push(text);
    }
    function quoted(text, quote) {
      if (text.length > maxBytes) throw new Error('pure output byte budget exhausted');
      write(quote);
      let chunk = '';
      for (const character of text) {
        chunk += escaped(character, quote);
        if (chunk.length >= 4096) { tick(); write(chunk); chunk = ''; }
      }
      write(chunk);
      write(quote);
    }
    function go(index, value, depth, chain) {
      tick();
      if (++nodes > 16384 || depth > 96) throw new Error('pure output budget exhausted');
      if ($tbIsRequest(value)) throw value;
      const type = types[index];
      switch (type.kind) {
        case 'U32': write(String(value)); return;
        case 'Nat': write(String(value) + 'n'); return;
        case 'F32': write(floatText(value).replace(/^-?\d+(?=e|$)/, '$&.0')); return;
        case 'Char': quoted(value, "'"); return;
        case 'String': quoted(value, '"'); return;
        case 'proof': write('{==}'); return;
        case 'Array': {
          write('[');
          for (let i = 0; i < value.length; i++) {
            if (i > 0) write(', ');
            go(type.element,value[i],depth+1,0);
          }
          write(']');
          return;
        }
        case 'data': {
          const tag = typeof value === 'boolean' ? value ? 'True' : 'False' : value?.$;
          const variant = type.variants.find(variant => variant.tag === tag);
          if (!variant) throw new Error('pure output constructor does not match its type');
          const open = tag === 'Con' || tag === 'Nil' ? '[' : tag === 'Tuple' ? '(' : '{';
          const close = open === '[' ? ']' : open === '(' ? ')' : '}';
          write(open === '{' ? tag + open : chain === open ? '' : open);
          for (let i = 0; i < variant.fields.length; i++) {
            const [name, field] = variant.fields[i];
            if (open === '[' ? i === 0 && chain === open : i > 0) write(', ');
            go(field,value[name],depth+1,i === 1 && open !== '{' ? open : 0);
          }
          if (open === '{' || chain !== open) write(close);
          return;
        }
        default: throw new Error('unsupported pure printer type');
      }
    }
    go(root,value,0,0);
    return parts.join('');
  }
  return {tick,jump,run,closure,apply,forceCall,curry,construct,matches,field,native,show,fromBits};
})();
function $tbTick() { $tb.tick(); }
function $tbForceCall(fn,args) { return $tb.forceCall(fn,args); }
