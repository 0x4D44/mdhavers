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

const structures = require('lib/structures');
blether("");
blether("=======================================================");
blether("    DATA STRUCTURES DEMO");
blether("    'Aw yer data, aw in order!'");
blether("=======================================================");
blether("");
blether("=== Stack (Last In, First Out) ===");
blether("");
let stack = new Stack();
blether("Pushing items: 1, 2, 3, 4, 5");
stack.push(1);
stack.push(2);
stack.push(3);
stack.push(4);
stack.push(5);
blether(`Stack size: ${stack.size()}`);
blether(`Top item (peek): ${stack.peek()}`);
blether("");
blether("Popping items:");
while ((!stack.is_empty())) {
  blether(`  Popped: ${stack.pop()}`);
}
blether(`Stack empty: ${stack.is_empty()}`);
blether("");
blether("=== Queue (First In, First Out) ===");
blether("");
let queue = new Queue();
blether("Enqueueing: apple, banana, cherry, date");
queue.enqueue("apple");
queue.enqueue("banana");
queue.enqueue("cherry");
queue.enqueue("date");
blether(`Queue size: ${queue.size()}`);
blether(`Front item (peek): ${queue.peek()}`);
blether("");
blether("Dequeueing items:");
while ((!queue.is_empty())) {
  blether(`  Dequeued: ${queue.dequeue()}`);
}
blether("");
blether("=== Deque (Double-Ended Queue) ===");
blether("");
let deque = new Deque();
blether("Adding to front: C, B, A");
deque.push_front("C");
deque.push_front("B");
deque.push_front("A");
blether("Adding to back: D, E, F");
deque.push_back("D");
deque.push_back("E");
deque.push_back("F");
blether(`Deque contents: ${deque.tae_list()}`);
blether(`Front: ${deque.peek_front()}`);
blether(`Back: ${deque.peek_back()}`);
blether("");
blether(`Pop front: ${deque.pop_front()}`);
blether(`Pop back: ${deque.pop_back()}`);
blether(`Remaining: ${deque.tae_list()}`);
blether("");
blether("=== Set (Unique Elements) ===");
blether("");
let set1 = make_set([1, 2, 3, 4, 5]);
let set2 = make_set([4, 5, 6, 7, 8]);
blether(`Set 1: ${set1.tae_list()}`);
blether(`Set 2: ${set2.tae_list()}`);
blether("");
blether(`Set 1 has 3: ${set1.has(3)}`);
blether(`Set 1 has 9: ${set1.has(9)}`);
blether("");
let union_set = set1.union(set2);
let intersection_set = set1.intersection(set2);
let difference_set = set1.difference(set2);
blether(`Union: ${union_set.tae_list()}`);
blether(`Intersection: ${intersection_set.tae_list()}`);
blether(`Difference (1 - 2): ${difference_set.tae_list()}`);
blether("");
let unique = new Set();
unique.add("apple");
unique.add("banana");
unique.add("apple");
unique.add("cherry");
unique.add("banana");
blether(`Adding with duplicates: ${unique.tae_list()}`);
blether(`Size: ${unique.size()}`);
blether("");
blether("=== Priority Queue ===");
blether("");
let pq = new PriorityQueue();
blether("Adding tasks with priorities:");
pq.enqueue("Low priority task", 10);
pq.enqueue("High priority task", 1);
pq.enqueue("Medium priority task", 5);
pq.enqueue("Urgent task", 0);
pq.enqueue("Normal task", 5);
blether(`Queue size: ${pq.size()}`);
blether("");
blether("Processing tasks by priority:");
while ((!pq.is_empty())) {
  blether(`  Processing: ${pq.dequeue()}`);
}
blether("");
blether("=== Ring Buffer (Capacity 5) ===");
blether("");
let ring = new RingBuffer(5);
blether("Adding 1, 2, 3");
ring.push(1);
ring.push(2);
ring.push(3);
blether(`Buffer: ${ring.tae_list()}`);
blether(`Size: ${ring.size()}, Full: ${ring.is_full()}`);
blether("");
blether("Adding 4, 5, 6, 7 (will overflow)");
ring.push(4);
ring.push(5);
ring.push(6);
ring.push(7);
blether(`Buffer: ${ring.tae_list()}`);
blether(`Size: ${ring.size()}, Full: ${ring.is_full()}`);
blether("");
blether(`Pop oldest: ${ring.pop()}`);
blether(`Buffer after pop: ${ring.tae_list()}`);
blether("");
blether("=== LRU Cache (Capacity 3) ===");
blether("");
let cache = new LRUCache(3);
blether("Setting: a=1, b=2, c=3");
cache.set("a", 1);
cache.set("b", 2);
cache.set("c", 3);
blether(`Get a: ${cache.get("a")}`);
blether(`Get b: ${cache.get("b")}`);
blether("");
blether("Setting d=4 (will evict least recently used)");
cache.set("d", 4);
blether(`Get c (evicted?): ${cache.get("c")}`);
blether(`Get a (still there?): ${cache.get("a")}`);
blether(`Get d: ${cache.get("d")}`);
blether("");
blether("=== Counter ===");
blether("");
let words = ["apple", "banana", "apple", "cherry", "apple", "banana", "date", "apple", "cherry", "apple"];
let counter = new Counter(words);
blether(`Counting words: ${words}`);
blether("");
blether(`apple count: ${counter.get("apple")}`);
blether(`banana count: ${counter.get("banana")}`);
blether(`cherry count: ${counter.get("cherry")}`);
blether(`date count: ${counter.get("date")}`);
blether(`missing count: ${counter.get("missing")}`);
blether("");
blether(`Total: ${counter.total()}`);
blether("");
blether("Most common:");
for (const item of counter.most_common(3)) {
  blether(`  ${item["key"]}: ${item["count"]}`);
}
blether("");
blether("=== Practical Example: Task Scheduler ===");
blether("");
class TaskScheduler {
  constructor() {
    (this.todo = new Queue());
    (this.done = new Stack());
  }
  add_task(task) {
    this.todo.enqueue(task);
  }
  do_next() {
    let task = this.todo.dequeue();
    if ((task !== null)) {
      blether(`  Doing: ${task}`);
      this.done.push(task);
    }
    return task;
  }
  undo_last() {
    let task = this.done.pop();
    if ((task !== null)) {
      blether(`  Undoing: ${task}`);
      this.todo.enqueue(task);
    }
    return task;
  }
  status() {
    blether(`  Pending: ${this.todo.size()}, Done: ${this.done.size()}`);
  }
}
let scheduler = new TaskScheduler();
blether("Adding tasks...");
scheduler.add_task("Write code");
scheduler.add_task("Run tests");
scheduler.add_task("Fix bugs");
scheduler.add_task("Deploy");
scheduler.status();
blether("");
blether("Doing tasks...");
scheduler.do_next();
scheduler.do_next();
scheduler.status();
blether("");
blether("Oops, need to undo...");
scheduler.undo_last();
scheduler.status();
blether("");
blether("=======================================================");
blether("    Data structures demo complete!");
blether("    'Yer data's aw in guid order noo!'");
blether("=======================================================");
blether("");
