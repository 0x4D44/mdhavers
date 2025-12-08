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
  heid: (x) => {
    if (typeof x === 'string' || Array.isArray(x)) {
      if (x.length === 0) throw new Error('Cannae get heid o\' an empty list!');
      return x[0];
    }
    throw new Error('heid() expects a list or string');
  },
  tail: (x) => {
    if (typeof x === 'string') return x.slice(1);
    if (Array.isArray(x)) return x.slice(1);
    throw new Error('tail() expects a list or string');
  },
  bum: (x) => {
    if (typeof x === 'string' || Array.isArray(x)) {
      if (x.length === 0) throw new Error('Cannae get bum o\' an empty list!');
      return x[x.length - 1];
    }
    throw new Error('bum() expects a list or string');
  },
  scran: (x, start, end) => {
    if (typeof x === 'string' || Array.isArray(x)) return x.slice(start, end);
    throw new Error('scran() expects a list or string');
  },
  slap: (a, b) => {
    if (typeof a === 'string' && typeof b === 'string') return a + b;
    if (Array.isArray(a) && Array.isArray(b)) return [...a, ...b];
    throw new Error('slap() expects two lists or two strings');
  },
  sumaw: (arr) => {
    if (!Array.isArray(arr)) throw new Error('sumaw() expects a list');
    return arr.reduce((a, b) => a + b, 0);
  },
  coont: (x, item) => {
    if (typeof x === 'string') return x.split(item).length - 1;
    if (Array.isArray(x)) return x.filter(e => e === item).length;
    throw new Error('coont() expects a list or string');
  },
  wheesht: (str) => String(str).trim(),
  upper: (str) => String(str).toUpperCase(),
  lower: (str) => String(str).toLowerCase(),
  shuffle: (arr) => {
    if (!Array.isArray(arr)) throw new Error('shuffle() expects a list');
    const result = [...arr];
    for (let i = result.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [result[i], result[j]] = [result[j], result[i]];
    }
    return result;
  },
  slice: (x, start, end, step) => {
    const len = x.length;
    const isStr = typeof x === 'string';
    const arr = isStr ? x.split('') : x;
    if (step === 0) throw new Error('Slice step cannae be zero, ya dafty!');
    // Handle defaults based on step direction
    const s = start !== null ? (start < 0 ? Math.max(len + start, 0) : Math.min(start, len)) : (step > 0 ? 0 : len - 1);
    const e = end !== null ? (end < 0 ? Math.max(len + end, step > 0 ? 0 : -1) : Math.min(end, len)) : (step > 0 ? len : -len - 1);
    const result = [];
    if (step > 0) {
      for (let i = s; i < e; i += step) result.push(arr[i]);
    } else {
      for (let i = s; i > e; i += step) result.push(arr[i]);
    }
    return isStr ? result.join('') : result;
  },
  // Timing functions
  noo: () => Date.now(),
  tick: () => {
    if (typeof process !== 'undefined' && process.hrtime) {
      const [s, ns] = process.hrtime();
      return s * 1e9 + ns;
    }
    return Date.now() * 1e6; // Fallback for browser
  },
  bide: (ms) => {
    const end = Date.now() + ms;
    while (Date.now() < end) {} // Busy wait (sync)
  },
  // Higher-order functions
  gaun: (arr, fn) => arr.map(fn),
  sieve: (arr, fn) => arr.filter(fn),
  tumble: (arr, init, fn) => arr.reduce(fn, init),
  aw: (arr, fn) => arr.every(fn),
  ony: (arr, fn) => arr.some(fn),
  hunt: (arr, fn) => arr.find(fn),
};

const { len, whit_kind, tae_string, tae_int, tae_float, shove, yank, keys, values, range, abs, min, max, floor, ceil, round, sqrt, split, join, contains, reverse, sort, blether, speir, heid, tail, bum, scran, slap, sumaw, coont, wheesht, upper, lower, shuffle, noo, tick, bide, gaun, sieve, tumble, aw, ony, hunt } = __havers;

blether("=== Higher-Order Functions in mdhavers ===");
blether("");
blether("1. Lambda expressions:");
let add = (a, b) => (a + b);
let square = (x) => (x * x);
let is_even = (n) => ((n % 2) === 0);
blether(("   add(3, 4) = " + tae_string(add(3, 4))));
blether(("   square(5) = " + tae_string(square(5))));
blether(("   is_even(6) = " + tae_string(is_even(6))));
blether("");
blether("2. gaun() - map a function over a list:");
let nums = [1, 2, 3, 4, 5];
blether(("   Original: " + tae_string(nums)));
let doubled = gaun(nums, (x) => (x * 2));
blether(("   Doubled:  " + tae_string(doubled)));
let squared = gaun(nums, (x) => (x * x));
blether(("   Squared:  " + tae_string(squared)));
let words = ["hullo", "scotland", "braw"];
let shouted = gaun(words, (w) => upper(w));
blether(("   Shouted:  " + tae_string(shouted)));
blether("");
blether("3. sieve() - filter a list:");
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
blether(("   Original: " + tae_string(numbers)));
let evens = sieve(numbers, (x) => ((x % 2) === 0));
blether(("   Evens:    " + tae_string(evens)));
let odds = sieve(numbers, (x) => ((x % 2) !== 0));
blether(("   Odds:     " + tae_string(odds)));
let big = sieve(numbers, (x) => (x > 5));
blether(("   > 5:      " + tae_string(big)));
blether("");
blether("4. tumble() - reduce/fold a list:");
let values = [1, 2, 3, 4, 5];
blether(("   values = " + tae_string(values)));
let sum = tumble(values, 0, (acc, x) => (acc + x));
blether(("   Sum:      " + tae_string(sum)));
let product = tumble(values, 1, (acc, x) => (acc * x));
blether(("   Product:  " + tae_string(product)));
function bigger(a, b) {
  if ((a > b)) {
    return a;
  } else {
    return b;
  }
}
let maximum = tumble(values, values[0], (acc, x) => bigger(acc, x));
blether(("   Maximum:  " + tae_string(maximum)));
let names = ["Hamish", "Morag", "Angus"];
let greeting = tumble(names, "", (acc, name) => ((acc + name) + " "));
blether(("   Names:    " + wheesht(greeting)));
blether("");
blether("5. ilk() - do something fer each item:");
let items = ["haggis", "neeps", "tatties"];
blether("   Supper menu:");
function print_item(item) {
  blether(("   - " + item));
}
ilk(items, print_item);
blether("");
blether("6. Combining higher-order functions:");
let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
blether(("   Original:      " + tae_string(data)));
let evens_only = sieve(data, (x) => ((x % 2) === 0));
blether(("   Evens:         " + tae_string(evens_only)));
let evens_squared = gaun(evens_only, (x) => (x * x));
blether(("   Squared:       " + tae_string(evens_squared)));
let total = tumble(evens_squared, 0, (acc, x) => (acc + x));
blether(("   Sum of squares: " + tae_string(total)));
blether("");
blether("7. Practical example - processing student scores:");
let scores = [72, 85, 91, 68, 77, 94, 82, 59, 88, 73];
blether(("   All scores:  " + tae_string(scores)));
let passed = sieve(scores, (s) => (s >= 70));
blether(("   Passed (>=70): " + tae_string(passed)));
let average = (tumble(scores, 0, (acc, s) => (acc + s)) / len(scores));
blether(("   Average:     " + tae_string(average)));
let top_scores = sieve(scores, (s) => (s >= 90));
blether(("   Top scores:  " + tae_string(top_scores)));
function grade(s) {
  if ((s >= 90)) {
    return "A";
  } else   if ((s >= 80)) {
    return "B";
  } else   if ((s >= 70)) {
    return "C";
  } else   if ((s >= 60)) {
    return "D";
  } else {
    return "F";
  }



}
let grades = gaun(scores, (s) => grade(s));
blether(("   Grades:      " + tae_string(grades)));
blether("");
blether("Braw! Functional programming in Scots!");
