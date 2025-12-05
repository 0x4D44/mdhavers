// mdhavers runtime - pure havers, but working havers!
const __havers = {
  len: (x) => {
    if (typeof x === 'string' || Array.isArray(x)) return x.length;
    if (x && typeof x === 'object') return Object.keys(x).length;
    throw new Error('Och! Cannae get length o\' that!');
  },
  whit_kind: (x) => {
    if (x === null || x === undefined) return 'naething';
    if (Array.isArray(x)) return 'list';
    if (typeof x === 'object') return 'dict';
    return typeof x;
  },
  tae_string: (x) => String(x),
  tae_int: (x) => {
    const n = parseInt(x, 10);
    if (isNaN(n)) throw new Error(`Cannae turn '${x}' intae an integer`);
    return n;
  },
  tae_float: (x) => {
    const n = parseFloat(x);
    if (isNaN(n)) throw new Error(`Cannae turn '${x}' intae a float`);
    return n;
  },
  shove: (arr, val) => { arr.push(val); },
  yank: (arr) => {
    if (arr.length === 0) throw new Error('Cannae yank fae an empty list!');
    return arr.pop();
  },
  keys: (obj) => Object.keys(obj),
  values: (obj) => Object.values(obj),
  range: (start, end) => {
    const result = [];
    for (let i = start; i < end; i++) result.push(i);
    return result;
  },
  abs: Math.abs,
  min: Math.min,
  max: Math.max,
  floor: Math.floor,
  ceil: Math.ceil,
  round: Math.round,
  sqrt: Math.sqrt,
  split: (str, delim) => str.split(delim),
  join: (arr, delim) => arr.join(delim),
  contains: (container, item) => {
    if (typeof container === 'string') return container.includes(item);
    if (Array.isArray(container)) return container.includes(item);
    if (typeof container === 'object') return item in container;
    return false;
  },
  reverse: (x) => {
    if (typeof x === 'string') return x.split('').reverse().join('');
    if (Array.isArray(x)) return [...x].reverse();
    throw new Error('reverse() expects a list or string');
  },
  sort: (arr) => [...arr].sort((a, b) => {
    if (typeof a === 'number' && typeof b === 'number') return a - b;
    return String(a).localeCompare(String(b));
  }),
  blether: console.log,
  speir: (prompt) => {
    const fs = require('fs');
    process.stdout.write(String(prompt));
    const buf = Buffer.alloc(1024);
    const n = fs.readSync(0, buf);
    return buf.toString('utf8', 0, n).trim();
  },
};

const { len, whit_kind, tae_string, tae_int, tae_float, shove, yank, keys, values, range, abs, min, max, floor, ceil, round, sqrt, split, join, contains, reverse, sort, blether, speir } = __havers;

blether("Hullo, World!");
blether("Welcome tae mdhavers - pure havers, but working havers!");
