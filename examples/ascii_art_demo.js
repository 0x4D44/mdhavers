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

const ascii_art = require('../stdlib/ascii_art');
const colors = require('lib/colors');
blether("");
blether(cyan("╔════════════════════════════════════════════════════════╗"));
blether(cyan("║          ASCII ART MODULE DEMO                         ║"));
blether(cyan("║         \"Drawin' pictures wi' letters!\"                ║"));
blether(cyan("╚════════════════════════════════════════════════════════╝"));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  SCOTTISH SYMBOLS"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("The Scottish Thistle:");
display_art(thistle());
blether("");
blether("The Saltire (St Andrew's Cross):");
display_art(saltire());
blether("");
blether("A Highland Coo:");
display_art(highland_coo());
blether("");
blether("Bagpipes:");
display_art(bagpipes());
blether("");
blether("A Wee Castle:");
display_art(castle());
blether("");
blether("The Wild Haggis (rare creature!):");
display_art(haggis_creature());
blether("");
blether("A Glass of Whisky:");
display_art(whisky_glass());
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  BOX DRAWING STYLES"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Single line box:");
display_art(draw_box(20, 5, "single"));
blether("");
blether("Double line box:");
display_art(draw_box(20, 5, "double"));
blether("");
blether("Rounded box:");
display_art(draw_box(20, 5, "rounded"));
blether("");
blether("Box with title:");
display_art(draw_box_with_title("SCOTLAND", 30, "double"));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  PROGRESS BARS"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Standard progress bars:");
blether(`  0%:   ${progress_bar(0, 100)}`);
blether(`  25%:  ${progress_bar(25, 100)}`);
blether(`  50%:  ${progress_bar(50, 100)}`);
blether(`  75%:  ${progress_bar(75, 100)}`);
blether(`  100%: ${progress_bar(100, 100)}`);
blether("");
blether("Scottish progress bars:");
blether(scots_progress_bar(10, 100));
blether(scots_progress_bar(40, 100));
blether(scots_progress_bar(60, 100));
blether(scots_progress_bar(85, 100));
blether(scots_progress_bar(100, 100));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  SPINNER ANIMATIONS"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Default spinner frames:");
let spin_line = "";
for (const i of __havers.range(0, 8)) {
  (spin_line = ((spin_line + spinner_frame(i)) + " "));
}
blether(`  ${spin_line}`);
blether("");
blether("Dot spinner frames:");
let dot_line = "";
for (const i of __havers.range(0, 10)) {
  (dot_line = ((dot_line + spinner_frame(i, "dots")) + " "));
}
blether(`  ${dot_line}`);
blether("");
blether("Scottish spinner frames:");
let scots_spin = "";
for (const i of __havers.range(0, 4)) {
  (scots_spin = ((scots_spin + spinner_frame(i, "scots")) + " "));
}
blether(`  ${scots_spin}`);
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  LINE STYLES"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Single line:");
blether(horizontal_line(40));
blether("Double line:");
blether(double_line(40));
blether("Dotted line:");
blether(dotted_line(40));
blether("Wavy line:");
blether(wavy_line(40));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  BIG BLOCK TEXT"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Saying BRAW:");
display_art(big_text("BRAW"));
blether("");
blether("Saying SCOTLAND:");
display_art(big_text("SCOTLAND"));
blether("");
blether("Saying HI:");
display_art(big_text("HI"));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  BANNERS"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Simple banner:");
display_art(banner("Welcome tae Scotland!"));
blether("");
blether("Scottish banner:");
display_art(scots_banner("Alba gu bràth!"));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  PATTERNS"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Tartan pattern (20x8):");
display_art(tartan_pattern(20, 8));
blether("");
blether("Scottish Flag:");
display_art(scottish_flag());
blether("");
blether("Checkerboard (16x6):");
display_art(checkerboard(16, 6));
blether("");
blether("Diagonal pattern (20x5):");
display_art(diagonal_pattern(20, 5));
blether("");
blether("Vertical gradient (20x8):");
display_art(vertical_gradient(20, 8));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  FRAMING ART"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Framed Highland Coo:");
let coo = highland_coo();
display_art(frame_art(coo, "double"));
blether("");
blether("Shadow box:");
display_art(shadow_box(15, 5));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  SPEECH & THOUGHT BUBBLES"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Speech bubble:");
display_art(speech_bubble("Och aye!"));
blether("");
blether("Thought bubble:");
display_art(thought_bubble("I fancy a wee dram..."));
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  COMBINING ART"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Thistle and Whisky Glass side by side:");
let combined = combine_art_horizontal(thistle(), whisky_glass(), 5);
display_art(combined);
blether("");
blether("Vertical combination:");
let top_art = banner("SCOTLAND");
let bottom_art = banner("THE BRAVE");
let vert = combine_art_vertical(top_art, bottom_art, 0);
display_art(vert);
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  TEXT UTILITIES"));
blether(yellow("═══════════════════════════════════════"));
blether("");
blether("Centered text (width 40):");
blether(`[${center_text("Hello Scotland!", 40)}]`);
blether("");
blether("Right-aligned text (width 40):");
blether(`[${right_align("Right side!", 40)}]`);
blether("");
blether("Repeated characters:");
blether(`  Stars: ${repeat_char("*", 20)}`);
blether(`  Hash:  ${repeat_char("#", 20)}`);
blether(`  Tilde: ${repeat_char("~", 20)}`);
blether("");
blether(yellow("═══════════════════════════════════════"));
blether(yellow("  GRAND FINALE"));
blether(yellow("═══════════════════════════════════════"));
blether("");
let title = big_text("BRAW");
let title_framed = frame_art(title, "double");
display_art(title_framed);
blether("");
blether(green("═══════════════════════════════════════"));
blether(green("  ASCII Art demo complete!"));
blether(green("  \"Every picture tells a story!\""));
blether(green("═══════════════════════════════════════"));
blether("");
