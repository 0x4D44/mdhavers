// mdhavers core runtime - display, truthiness, equality and checked arithmetic,
// kept in step wi' the interpreter's Value semantics (src/value.rs).
//
// Number model (DEC F5): an mdhavers integer is a JS BigInt held in the i64
// range, an' a float is a JS number. `typeof` tells them apart, so every
// arithmetic helper below dispatches on the two `typeof`s wi' nae allocation
// beyond the BigInt result itsel'. Integer results are range-checked an' raise
// the interpreter's overflow error; mixed int/float work converts the integer
// wi' Number(), which rounds like Rust's `i64 as f64`.
const __havers_core = (() => {
  const isKin = (x) => typeof x === "function" && x.__mdh_kin === true;

  const I64_MIN = -(2n ** 63n);
  const I64_MAX = 2n ** 63n - 1n;

  // Rust's f64 Display never uses exponent notation; JS String() does for
  // very big or very wee numbers, so expand the exponent form by hand.
  const showNumber = (n) => {
    if (Number.isNaN(n)) return "NaN";
    if (n === Infinity) return "inf";
    if (n === -Infinity) return "-inf";
    if (n === 0) return Object.is(n, -0) ? "-0" : "0";
    const s = String(n);
    const m = /^(-?)(\d)(?:\.(\d+))?e([+-]\d+)$/.exec(s);
    if (!m) return s;
    const sign = m[1];
    const digits = m[2] + (m[3] || "");
    const exp = parseInt(m[4], 10);
    if (exp < 0) return sign + "0." + "0".repeat(-exp - 1) + digits;
    return sign + digits + "0".repeat(exp - (digits.length - 1));
  };

  // ---------- dict keys ----------
  //
  // A dict is a plain JS object, but its keys are typed like the interpreter's
  // `ValueKey`: 1, 1.0, "1" an' aye are fower different keys. A string key is
  // its ain property name, so `d.name`, logging fields an' three.js options see
  // plain names. Every ither key is encoded as "\0" + a tag + its text, an' so
  // is a string that JS would reorder (an array index like "1") or that is
  // special ("__proto__", or one startin' wi' "\0"). Nae encoded name is an
  // array index, so a dict keeps its insertion order. A list, dict, creel or
  // function key is keyed by identity through a registry, as in the interpreter.
  const refIds = new WeakMap(); // object key -> its id
  const refKeys = new Map(); // id -> object key
  const plainKey = (s) => !(/^(0|[1-9]\d*)$/.test(s) || s.charCodeAt(0) === 0 || s === "__proto__");
  const dkey = (k) => {
    switch (typeof k) {
      case "string":
        return plainKey(k) ? k : "\0s" + k;
      case "bigint":
        return "\0i" + String(k);
      case "number":
        return "\0f" + (Object.is(k, -0) ? "-0" : String(k));
      case "boolean":
        return k ? "\0b1" : "\0b0";
      case "undefined":
        return "\0n";
      default: {
        if (k === null) return "\0n";
        let id = refIds.get(k);
        if (id === undefined) {
          id = refKeys.size;
          refIds.set(k, id);
          refKeys.set(id, k);
        }
        return "\0r" + id;
      }
    }
  };
  const undkey = (p) => {
    if (p.charCodeAt(0) !== 0) return p;
    const body = p.slice(2);
    switch (p[1]) {
      case "s": return body;
      case "i": return BigInt(body);
      case "f": return Number(body);
      case "b": return body === "1";
      case "n": return null;
      default: return refKeys.get(Number(body));
    }
  };
  const dictKeys = (d) => Object.keys(d).map(undkey);

  // Mirrors `impl Display for Value`.
  const show = (x) => {
    if (x === null || x === undefined) return "naething";
    if (x === true) return "aye";
    if (x === false) return "nae";
    if (typeof x === "bigint") return String(x);
    if (typeof x === "number") return showNumber(x);
    if (typeof x === "string") return x;
    if (Array.isArray(x)) return "[" + x.map(show).join(", ") + "]";
    if (x instanceof Set) {
      const items = [...x].map(show).sort();
      return "creel{" + items.map((s) => '"' + s + '"').join(", ") + "}";
    }
    if (typeof x === "function") {
      if (isKin(x)) return x.__mdh_struct ? "<thing " + x.name + ">" : "<kin " + x.name + ">";
      return "<dae " + (x.name || "lambda") + ">";
    }
    if (typeof x === "object") {
      const ctor = Object.getPrototypeOf(x) === null ? null : x.constructor;
      if (ctor && ctor !== Object && !ctor.__mdh_struct) return "<" + ctor.name + " instance>";
      return "{" + Object.keys(x).map((k) => '"' + show(undkey(k)) + '": ' + show(x[k])).join(", ") + "}";
    }
    return String(x);
  };

  // A plain dict (no' a class instance or a struct).
  const isDict = (x) => {
    if (x === null || typeof x !== "object" || Array.isArray(x) || x instanceof Set) return false;
    const proto = Object.getPrototypeOf(x);
    return proto === Object.prototype || proto === null;
  };

  // Mirrors `Value::is_truthy`: empty containers are falsy; NaN stays truthy, as in the interpreter.
  const truthy = (x) => {
    if (x === null || x === undefined || x === false || x === 0n || x === 0 || x === "") return false;
    if (Array.isArray(x)) return x.length > 0;
    if (x instanceof Set) return x.size > 0;
    if (isDict(x)) return Object.keys(x).length > 0;
    return true;
  };

  // Integer == float compares the integer cast tae f64, as `impl PartialEq for Value` does.
  const scalarEq = (a, b) => {
    if (a === b) return true;
    const ta = typeof a;
    const tb = typeof b;
    if (ta === "bigint" && tb === "number") return Number(a) === b;
    if (ta === "number" && tb === "bigint") return a === Number(b);
    return false;
  };

  // Mirrors `impl PartialEq for Value`: lists an' dicts compare structurally, creels
  // by their members, the rest by identity (numbers by value). `path` holds the
  // container pairs being compared right now; meeting one again counts as equal,
  // so cycles finish.
  const eq = (a, b, path) => {
    if (a === undefined) a = null;
    if (b === undefined) b = null;
    if (a instanceof Set && b instanceof Set) {
      if (a.size !== b.size) return false;
      for (const item of a) if (!b.has(item)) return false;
      return true;
    }
    const lists = Array.isArray(a) && Array.isArray(b);
    const dicts = isDict(a) && isDict(b);
    if (!lists && !dicts) return scalarEq(a, b);
    if (a === b) return true;
    if (path === undefined) path = [];
    if (path.some(([x, y]) => x === a && y === b)) return true;
    path.push([a, b]);
    let same = true;
    if (lists) {
      if (a.length !== b.length) same = false;
      for (let i = 0; same && i < a.length; i++) same = eq(a[i], b[i], path);
    } else {
      const keys = Object.keys(a);
      if (keys.length !== Object.keys(b).length) same = false;
      for (let i = 0; same && i < keys.length; i++) {
        const k = keys[i];
        same = Object.prototype.hasOwnProperty.call(b, k) && eq(a[k], b[k], path);
      }
    }
    path.pop();
    return same;
  };

  const fail = (message) => new Error(message);
  // A builtin's error, worded as the interpreter wraps a native function's message.
  const native = (message) => fail(`Jings! Something went awfy wrang: ${message}`);
  const typeError = (message, line) => fail(`That's pure mince! Type error at line ${line}: ${message}`);

  const typeName = (x) => {
    if (x === null || x === undefined) return "naething";
    if (typeof x === "boolean") return "bool";
    if (typeof x === "bigint") return "integer";
    if (typeof x === "number") return "float";
    if (typeof x === "string") return "string";
    if (Array.isArray(x)) return "list";
    if (x instanceof Set) return "creel";
    if (typeof x === "function") return isKin(x) ? (x.__mdh_struct ? "struct" : "class") : "function";
    const ctor = Object.getPrototypeOf(x) === null ? null : x.constructor;
    if (ctor && ctor !== Object && !ctor.__mdh_struct) return "instance";
    return "dict";
  };

  // ---------- checked i64 arithmetic ----------

  const overflow = (line) => fail(`Wheesht! Yer number's too muckle at line ${line} - it's overflowed!`);
  // An integer result must fit in an i64, like Rust's checked_* ops.
  const chk = (r, line) => {
    if (r > I64_MAX || r < I64_MIN) throw overflow(line);
    return r;
  };

  // `f as i64`: truncates, saturates at the i64 limits, NaN is 0.
  const f64ToI64 = (f) => {
    if (Number.isNaN(f)) return 0n;
    if (f >= 9223372036854775807) return I64_MAX;
    if (f <= -9223372036854775808) return I64_MIN;
    return BigInt(Math.trunc(f));
  };
  // `Value::as_integer`: integers as they are, floats truncated, else null.
  const asInt = (x) => (typeof x === "bigint" ? x : typeof x === "number" ? f64ToI64(x) : null);

  const add = (a, b, line) => {
    const ta = typeof a;
    const tb = typeof b;
    if (ta === "bigint") {
      if (tb === "bigint") return chk(a + b, line);
      if (tb === "number") return Number(a) + b;
    } else if (ta === "number") {
      if (tb === "number") return a + b;
      if (tb === "bigint") return a + Number(b);
    }
    if (ta === "string" || tb === "string") return show(a) + show(b);
    if (Array.isArray(a) && Array.isArray(b)) return [...a, ...b];
    throw typeError(`Cannae add ${typeName(a)} an' ${typeName(b)}`, line);
  };

  const sub = (a, b, line) => {
    const ta = typeof a;
    const tb = typeof b;
    if (ta === "bigint") {
      if (tb === "bigint") return chk(a - b, line);
      if (tb === "number") return Number(a) - b;
    } else if (ta === "number") {
      if (tb === "number") return a - b;
      if (tb === "bigint") return a - Number(b);
    }
    throw typeError(`Cannae subtract ${typeName(b)} fae ${typeName(a)}`, line);
  };

  const repeat = (s, n, line) => {
    if (n < 0n) throw typeError("Repeat count must be non-negative", line);
    return s.repeat(Number(n));
  };

  const mul = (a, b, line) => {
    const ta = typeof a;
    const tb = typeof b;
    if (ta === "bigint") {
      if (tb === "bigint") return chk(a * b, line);
      if (tb === "number") return Number(a) * b;
      if (tb === "string") return repeat(b, a, line);
    } else if (ta === "number") {
      if (tb === "number") return a * b;
      if (tb === "bigint") return a * Number(b);
    } else if (ta === "string" && tb === "bigint") {
      return repeat(a, b, line);
    }
    throw typeError(`Cannae multiply ${typeName(a)} by ${typeName(b)}`, line);
  };

  const divByZero = (line) => fail(`Ye numpty! Tryin' tae divide by zero at line ${line}`);
  // Integer division truncates toward zero (BigInt `/` does, like Rust);
  // i64::MIN / -1 overflows.
  const div = (a, b, line) => {
    if (b === 0n || b === 0) throw divByZero(line);
    const ta = typeof a;
    const tb = typeof b;
    if (ta === "bigint") {
      if (tb === "bigint") return chk(a / b, line);
      if (tb === "number") return Number(a) / b;
    } else if (ta === "number") {
      if (tb === "number") return a / b;
      if (tb === "bigint") return a / Number(b);
    }
    throw typeError(`Cannae divide ${typeName(a)} by ${typeName(b)}`, line);
  };

  // The remainder takes the dividend's sign (BigInt `%` an' Rust agree). Only an
  // integer zero divisor is an error; a float `% 0.0` is NaN. Mixed int/float is
  // a type error, as in the interpreter.
  const mod = (a, b, line) => {
    if (b === 0n) throw divByZero(line);
    const ta = typeof a;
    const tb = typeof b;
    if (ta === "bigint" && tb === "bigint") {
      if (b === -1n && a === I64_MIN) throw overflow(line);
      return a % b;
    }
    if (ta === "number" && tb === "number") return a % b;
    throw typeError(`Cannae get remainder o' ${typeName(a)} by ${typeName(b)}`, line);
  };

  const neg = (a, line) => {
    if (typeof a === "bigint") return chk(-a, line);
    if (typeof a === "number") return -a;
    throw typeError(`Cannae negate a ${typeName(a)}`, line);
  };

  // Ordering: integers, floats an' strings compare among themselves; an integer
  // against a float compares as f64. Onything else is a type error.
  const cmpError = (a, b, line) => typeError(`Cannae compare ${typeName(a)} wi' ${typeName(b)}`, line);
  const ordered = (a, b, line) => {
    const ta = typeof a;
    const tb = typeof b;
    if (ta === tb && (ta === "bigint" || ta === "number" || ta === "string")) return 0;
    if ((ta === "bigint" && tb === "number") || (ta === "number" && tb === "bigint")) return 1;
    throw cmpError(a, b, line);
  };
  const lt = (a, b, line) => (ordered(a, b, line) === 0 ? a < b : Number(a) < Number(b));
  const le = (a, b, line) => (ordered(a, b, line) === 0 ? a <= b : Number(a) <= Number(b));
  const gt = (a, b, line) => (ordered(a, b, line) === 0 ? a > b : Number(a) > Number(b));
  const ge = (a, b, line) => (ordered(a, b, line) === 0 ? a >= b : Number(a) >= Number(b));

  // ---------- indexing an' slicing ----------

  const oob = (i, size, line) =>
    fail(`Hoachin'! Index ${i} is oot o' bounds (size is ${size}) at line ${line}`);
  const idx = (o, i, line) => {
    if (Array.isArray(o) || typeof o === "string") {
      if (typeof i !== "bigint") throw typeError(`Cannae index a ${typeName(o)} wi' a ${typeName(i)}`, line);
      const items = Array.isArray(o) ? o : Array.from(o);
      const n = Number(i);
      const j = n < 0 ? items.length + n : n;
      if (j < 0 || j >= items.length) throw oob(i, items.length, line);
      return items[j];
    }
    if (isDict(o)) {
      const k = dkey(i);
      if (!Object.prototype.hasOwnProperty.call(o, k)) {
        throw fail(`Awa' an bile yer heid! '${show(i)}' hasnae been defined yet at line ${line}`);
      }
      return o[k];
    }
    return o[i];
  };

  // `object[index] = value`; lists need an in-range integer index.
  const setIdx = (o, i, v, line) => {
    if (Array.isArray(o)) {
      if (typeof i !== "bigint") throw typeError(`Cannae set index on a list wi' a ${typeName(i)}`, line);
      const n = Number(i);
      const j = n < 0 ? o.length + n : n;
      if (j < 0 || j >= o.length) throw oob(i, o.length, line);
      o[j] = v;
      return v;
    }
    if (typeof o === "string") throw typeError(`Cannae set index on a string wi' a ${typeName(i)}`, line);
    o[isDict(o) ? dkey(i) : i] = v;
    return v;
  };

  // `x[start:end:step]`; a missing part is null. Mirrors `eval_slice`.
  const slice = (x, start, end, step, line) => {
    if (start !== null && typeof start !== "bigint") throw typeError("Slice start must be an integer", line);
    if (end !== null && typeof end !== "bigint") throw typeError("Slice end must be an integer", line);
    if (step !== null && typeof step !== "bigint") throw typeError("Slice step must be an integer", line);
    if (step === 0n) throw typeError("Slice step cannae be zero, ya dafty!", line);
    const isStr = typeof x === "string";
    if (!isStr && !Array.isArray(x)) throw typeError(`Cannae slice a ${typeName(x)}, ya numpty!`, line);
    const items = isStr ? Array.from(x) : x;
    const len = items.length;
    const st = step === null ? 1 : Number(step);
    let s;
    let e;
    if (st > 0) {
      s = start === null ? 0 : Number(start);
      e = end === null ? len : Number(end);
    } else {
      s = start === null ? -1 : Number(start);
      e = end === null ? -(len + 1) : Number(end);
    }
    s = s < 0 ? Math.max(len + s, 0) : Math.min(s, len);
    e = e < 0 ? Math.max(len + e, -1) : Math.min(e, len);
    const out = [];
    if (st > 0) {
      for (let i = s; i < e; i += st) out.push(items[i]);
    } else {
      for (let i = s; i > e; i += st) if (i < len) out.push(items[i]);
    }
    return isStr ? out.join("") : out;
  };

  const rangeList = (s, e, inclusive) => {
    const out = [];
    if (inclusive) {
      for (let i = s; i <= e; i++) out.push(i);
    } else {
      for (let i = s; i < e; i++) out.push(i);
    }
    return out;
  };
  // `start..end` / `start..=end`.
  const rangeOf = (start, end, inclusive, line) => {
    const s = asInt(start);
    const e = asInt(end);
    if (s === null || e === null) throw typeError("Range bounds must be integers", line);
    return rangeList(s, e, inclusive);
  };
  // A `whan a..b` arm matches only an integer inside the bounds.
  const inRange = (v, start, end, inclusive) => {
    if (typeof v !== "bigint") return false;
    const s = asInt(start);
    const e = asInt(end);
    if (s === null || e === null) return false;
    return v >= s && (inclusive ? v <= e : v < e);
  };

  // `fer` iterates ower a snapshot, like the interpreter.
  const iter = (x, line) => {
    if (Array.isArray(x)) return x.slice();
    if (typeof x === "string") return Array.from(x);
    throw typeError(`Cannae iterate ower a ${typeName(x)}`, line);
  };

  // ---------- builtins that take or gie integers ----------

  const nativeOverflow = (what) => native(`Wheesht! Yer number's too muckle in ${what} - it's overflowed!`);

  // A number as f64 (`n as f64` fer an integer), or the builtin's error.
  const f64Of = (x, message) => {
    if (typeof x === "number") return x;
    if (typeof x === "bigint") return Number(x);
    throw native(message);
  };
  // A one-number float function: sin, cos, log, ...
  const floatFn = (name, f, message) => (x) => f(f64Of(x, message || `${name}() needs a number`));
  // Rust's f64::max/min ignore a NaN operand.
  const fmax = (a, b) => (Number.isNaN(a) ? b : Number.isNaN(b) ? a : Math.max(a, b));
  const fmin = (a, b) => (Number.isNaN(a) ? b : Number.isNaN(b) ? a : Math.min(a, b));
  // Rust's `f64::powi` (compiler-rt __powidf2): square-an'-multiply, so the
  // rounding matches the interpreter's `pooer(float, int)`.
  const powi = (a, b) => {
    const recip = b < 0;
    let n = recip ? -b : b;
    let r = 1;
    for (;;) {
      if (n & 1) r *= a;
      n = Math.trunc(n / 2);
      if (n === 0) break;
      a *= a;
    }
    return recip ? 1 / r : r;
  };
  // `float_pow_int` in the interpreter: powi fer an i32 exponent, else powf.
  const floatPowInt = (base, exp) =>
    exp >= -2147483648n && exp <= 2147483647n ? powi(base, Number(exp)) : Math.pow(base, Number(exp));
  const gcdBig = (a, b) => {
    a = a < 0n ? -a : a;
    b = b < 0n ? -b : b;
    while (b !== 0n) [a, b] = [b, a % b];
    return a;
  };
  const numList = (arr, name) => {
    if (!Array.isArray(arr)) throw native(`${name}() needs a list`);
    return arr;
  };
  // Uniform in [0, 1), as the interpreter's havers_random_f64.
  const random = () => Math.random();
  const randomIntInclusive = (min, max) => {
    const span = max - min + 1n;
    // 53 random bits at a time, enough fer ony i64 span.
    let r = 0n;
    for (let bits = 0n; (1n << bits) < span; bits += 53n) {
      r = (r << 53n) | BigInt(Math.floor(Math.random() * 2 ** 53));
    }
    return min + (r % span);
  };

  // Blockin' sleep. Node waits on a SharedArrayBuffer (nae busy loop); a browser's
  // main thread cannae Atomics.wait, so it spins like `bide` does.
  const sleepSync = (ms) => {
    if (ms <= 0) return;
    try {
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
    } catch (_) {
      const end = Date.now() + ms;
      while (Date.now() < end) {}
    }
  };

  // get_key: a keydown buffer in the browser, a raw-mode stdin read in node.
  const keyBuffer = [];
  let keyListening = false;
  const keyName = (k) => {
    switch (k) {
      case "ArrowLeft": return "Left";
      case "ArrowRight": return "Right";
      case "ArrowUp": return "Up";
      case "ArrowDown": return "Down";
      case "Enter": return "\n";
      case "Escape": return "\x1b";
      case "Backspace": return "\x08";
      default: return Array.from(k).length === 1 ? k : "";
    }
  };
  const nodeKey = () => {
    if (typeof process === "undefined" || !process.stdin || !process.stdin.isTTY) return "";
    if (typeof require !== "function") return "";
    let s = "";
    try {
      const buf = Buffer.alloc(8);
      process.stdin.setRawMode(true);
      try {
        s = buf.toString("utf8", 0, require("fs").readSync(0, buf, 0, buf.length, null));
      } finally {
        process.stdin.setRawMode(false);
      }
    } catch (_) {
      return "";
    }
    if (s === "\x03") process.exit(130);
    if (s === "\x1b[A") return "Up";
    if (s === "\x1b[B") return "Down";
    if (s === "\x1b[C") return "Right";
    if (s === "\x1b[D") return "Left";
    if (s === "\r" || s === "\n") return "\n";
    if (s === "\x7f" || s === "\b") return "\x08";
    if (s === "\x1b") return "\x1b";
    return Array.from(s).length === 1 ? s : "";
  };
  const readKey = () => {
    if (typeof document !== "undefined" && typeof document.addEventListener === "function") {
      if (!keyListening) {
        keyListening = true;
        document.addEventListener("keydown", (e) => {
          keyBuffer.push(keyName(e.key));
          if (keyBuffer.length > 64) keyBuffer.shift();
        });
      }
      return keyBuffer.length > 0 ? keyBuffer.shift() : "";
    }
    return nodeKey();
  };

  // ---------- JSON, as `value_to_json` in the interpreter ----------

  const MAX_JSON_DEPTH = 512;
  // Rust's `char::is_control` is the Cc category: U+0000-001F an' U+007F-009F.
  const jsonEscape = (s) => {
    let out = '"';
    for (const c of s) {
      if (c === '"') out += '\\"';
      else if (c === "\\") out += "\\\\";
      else if (c === "\n") out += "\\n";
      else if (c === "\t") out += "\\t";
      else if (c === "\r") out += "\\r";
      else if (/[\x00-\x1f\x7f-\x9f]/.test(c)) out += "\\u" + c.charCodeAt(0).toString(16).padStart(4, "0");
      else out += c;
    }
    return out + '"';
  };
  const jsonEnter = (x, open) => {
    if (open.includes(x)) {
      throw native("Cannae write JSON fer a list or dict that contains itsel - it wid gang roond forever!");
    }
    if (open.length >= MAX_JSON_DEPTH) {
      throw native(`JSON's nested ower deep (mair than ${MAX_JSON_DEPTH} levels) - ah'm no' diggin' that far!`);
    }
    open.push(x);
  };
  const toJson = (x, open) => {
    if (x === null || x === undefined) return "null";
    if (x === true) return "true";
    if (x === false) return "false";
    if (typeof x === "bigint") return String(x);
    if (typeof x === "number") return Number.isFinite(x) ? showNumber(x) : "null";
    if (typeof x === "string") return jsonEscape(x);
    if (Array.isArray(x)) {
      jsonEnter(x, open);
      const items = x.map((v) => toJson(v, open));
      open.pop();
      return "[" + items.join(", ") + "]";
    }
    if (isDict(x)) {
      jsonEnter(x, open);
      const pairs = Object.keys(x).map((p) => {
        const k = undkey(p);
        return jsonEscape(typeof k === "string" ? k : show(k)) + ": " + toJson(x[p], open);
      });
      open.pop();
      return "{" + pairs.join(", ") + "}";
    }
    return '"' + show(x).replace(/"/g, '\\"') + '"';
  };

  // The source line o' the latest higher-order call, set by `hof` (the compiler wraps such calls).
  let hofLineNo = 0;
  const hof = (line, f, args) => {
    hofLineNo = line;
    return f(...args);
  };
  // The list argument, or the interpreter's TypeError. Reads the line at entry, before callbacks run.
  const hofList = (name, l) => {
    const line = hofLineNo;
    if (!Array.isArray(l)) throw typeError(`${name}() expects a list as first argument`, line);
    hofLine = line;
    return l;
  };
  const hofCall = (f) => {
    if (typeof f === "function") return f;
    return () => {
      throw fail(`Whit's aw this aboot? '${show(f)}' isnae a function at line ${hofLine}`);
    };
  };
  let hofLine = 0;

  // ---------- pure string, list, bit, JSON an' assertion builtins (MDH-BUG-FLU-00114) ----------
  //
  // Strings are walked by code point, as Rust's `chars()` does, no' by UTF-16 unit. Case an'
  // character-class tests are ASCII-only, as the interpreter's are.
  const MAX_BUILT_SIZE = 1073741824n;
  const U64_MAX = 2n ** 64n - 1n;
  const cps = (s) => Array.from(s);
  const utf8Len = (s) => new TextEncoder().encode(s).length;
  // Mirrors `checked_alloc_size`: `unit * count` bytes must stay within the build limit.
  const allocCheck = (unit, count, what) => {
    const size = BigInt(unit) * count;
    if (size > U64_MAX) {
      throw native(`${what} wid be ower muckle (bigger than the ${MAX_BUILT_SIZE} limit), ya greedy gannet!`);
    }
    if (size > MAX_BUILT_SIZE) {
      throw native(`${what} wid be ower muckle (${size} is mair than the ${MAX_BUILT_SIZE} limit), ya greedy gannet!`);
    }
  };
  // Rust's `char::is_whitespace` (Unicode White_Space); JS's `\s` differs at U+0085 an' U+FEFF.
  const isRustWs = (c) => {
    const n = c.codePointAt(0);
    return (n >= 9 && n <= 13) || n === 32 || n === 0x85 || n === 0xa0 || n === 0x1680 ||
      (n >= 0x2000 && n <= 0x200a) || n === 0x2028 || n === 0x2029 || n === 0x202f || n === 0x205f || n === 0x3000;
  };
  const isMdhWs = (c) => c === " " || c === "\t" || c === "\n" || c === "\r";
  const trimStart = (s, pred) => {
    const a = cps(s);
    let i = 0;
    while (i < a.length && pred(a[i])) i++;
    return a.slice(i).join("");
  };
  const trimEnd = (s, pred) => {
    const a = cps(s);
    let i = a.length;
    while (i > 0 && pred(a[i - 1])) i--;
    return a.slice(0, i).join("");
  };
  // Code points before UTF-16 offset `i`.
  const cpIndex = (s, i) => cps(s.slice(0, i)).length;
  const isLowerA = (c) => c >= "a" && c <= "z";
  const isUpperA = (c) => c >= "A" && c <= "Z";
  const allAscii = (s, pred) => s.length > 0 && Array.from(s).every(pred);
  const swapAscii = (c) => (isUpperA(c) ? c.toLowerCase() : isLowerA(c) ? c.toUpperCase() : c);
  const needStr = (v, msg) => {
    if (typeof v !== "string") throw native(msg);
    return v;
  };
  const needList = (v, msg) => {
    if (!Array.isArray(v)) throw native(msg);
    return v;
  };
  // `usize` view o' a clamped-at-zero i64 count, capped at `max` (all that can matter).
  const clampCount = (n, max) => Number(n < 0n ? 0n : n > BigInt(max) ? BigInt(max) : n);
  const chunkList = (l, size) => {
    const out = [];
    for (let i = 0; i < l.length; i += size) out.push(l.slice(i, i + size));
    return out;
  };
  const interleaveLists = (a, b) => {
    const out = [];
    const n = Math.max(a.length, b.length);
    for (let i = 0; i < n; i++) {
      if (i < a.length) out.push(a[i]);
      if (i < b.length) out.push(b[i]);
    }
    return out;
  };
  const padArgs = (name, args) => {
    if (args.length < 2 || args.length > 3) throw native(`${name}() expects 2-3 arguments`);
    const bad = `${name}() needs (string, width, pad_char)`;
    if (typeof args[0] !== "string" || typeof args[1] !== "bigint") throw native(bad);
    let fill = " ";
    if (args.length === 3) {
      if (typeof args[2] !== "string") throw native(bad);
      fill = cps(args[2])[0] ?? " ";
    }
    return [args[0], args[1], fill];
  };
  const padCount = (name, s, width, fill) => {
    const len = BigInt(cps(s).length);
    if (width <= len) return 0n;
    const pad = width - len;
    allocCheck(utf8Len(fill), pad, `${name}()`);
    return pad;
  };
  const replaceAllCp = (s, from, to) => {
    if (from === "") return to + cps(s).map((c) => c + to).join("");
    return s.split(from).join(to);
  };
  // Rust `i64::from_str_radix`: an optional sign, then digits o' the radix; null if invalid or oot o' range.
  const parseRadix = (str, radix) => {
    const m = /^([+-]?)([0-9a-zA-Z]+)$/.exec(str);
    if (m === null) return null;
    let n = 0n;
    const r = BigInt(radix);
    for (const c of m[2]) {
      const d = parseInt(c, 36);
      if (d >= radix) return null;
      n = n * r + BigInt(d);
    }
    if (m[1] === "-") n = -n;
    return n < I64_MIN || n > I64_MAX ? null : n;
  };
  const stripPrefix = (s, p) => {
    while (s.startsWith(p)) s = s.slice(p.length);
    return s;
  };
  const asUnsigned = (n) => BigInt.asUintN(64, n);
  const needInts = (name, a, b) => {
    if (typeof a !== "bigint" || typeof b !== "bigint") throw native(`${name}() needs two integers`);
  };
  const shiftArgs = (name, a, b) => {
    needInts(name, a, b);
    if (b < 0n || b > 63n) throw native("Shift amount must be 0-63, ya numpty!");
    return Number(b);
  };
  // The interpreter's `uniq` compares the values' Debug text: so an integer an' a float differ, an'
  // lists, dicts an' instances match by content. Build an equivalent key (a container met again
  // while it is open is elided, as Debug does).
  const debugKey = (x, open) => {
    if (x === null || x === undefined) return "N";
    if (typeof x === "bigint") return "I" + x;
    if (typeof x === "number") return "F" + (Object.is(x, -0) ? "-0" : String(x));
    if (typeof x === "string") return "S" + JSON.stringify(x);
    if (typeof x === "boolean") return x ? "Bt" : "Bf";
    if (typeof x === "function") return dkey(x);
    if (open.includes(x)) return "...";
    open.push(x);
    let out;
    if (Array.isArray(x)) {
      out = "L[" + x.map((v) => debugKey(v, open)).join(",") + "]";
    } else if (x instanceof Set) {
      out = "C[" + [...x].map((v) => debugKey(v, open)).sort().join(",") + "]";
    } else if (isDict(x)) {
      out = "D{" + Object.keys(x).map((p) => debugKey(undkey(p), open) + ":" + debugKey(x[p], open)).join(",") + "}";
    } else {
      const ctor = Object.getPrototypeOf(x) === null ? null : x.constructor;
      out = "O" + (ctor ? ctor.name : "") + "{" + Object.keys(x).map((k) => k + ":" + debugKey(x[k], open)).join(",") + "}";
    }
    open.pop();
    return out;
  };

  // ---------- JSON parse an' pretty-print (src/interpreter.rs parse_json_value) ----------
  const parseJson = (src) => {
    const s = trimStart(trimEnd(src, isRustWs), isRustWs);
    if (s === "") throw native("Empty JSON string");
    const ch = cps(s);
    let pos = 0;
    const skipWs = () => {
      while (pos < ch.length && isRustWs(ch[pos])) pos++;
    };
    const tooDeep = () => native(`JSON's nested ower deep (mair than ${MAX_JSON_DEPTH} levels) - ah'm no' diggin' that far!`);
    const parseString = () => {
      pos++;
      let out = "";
      while (pos < ch.length) {
        const c = ch[pos];
        if (c === '"') {
          pos++;
          return out;
        }
        if (c === "\\") {
          pos++;
          if (pos >= ch.length) throw native("Unterminated string escape");
          const e = ch[pos];
          if (e === "n") out += "\n";
          else if (e === "t") out += "\t";
          else if (e === "r") out += "\r";
          else if (e === "u") {
            if (pos + 4 >= ch.length) throw native("Invalid unicode escape");
            const hex = ch.slice(pos + 1, pos + 5).join("");
            // Rust's from_str_radix takes a leading '+'; a surrogate is silently dropped.
            if (/^\+?[0-9a-fA-F]+$/.test(hex)) {
              const code = parseInt(hex, 16);
              if (code < 0xd800 || code > 0xdfff) out += String.fromCodePoint(code);
            }
            pos += 4;
          } else out += e;
        } else out += c;
        pos++;
      }
      throw native("Unterminated JSON string");
    };
    const parseNumber = () => {
      const start = pos;
      let dot = false;
      let exp = false;
      if (ch[pos] === "-") pos++;
      while (pos < ch.length) {
        const c = ch[pos];
        if (c >= "0" && c <= "9") pos++;
        else if (c === "." && !dot && !exp) {
          dot = true;
          pos++;
        } else if ((c === "e" || c === "E") && !exp) {
          exp = true;
          pos++;
          if (pos < ch.length && (ch[pos] === "+" || ch[pos] === "-")) pos++;
        } else break;
      }
      const text = ch.slice(start, pos).join("");
      if (dot || exp) {
        if (!/^-?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(text)) throw native(`Invalid number: ${text}`);
        return Number(text);
      }
      if (/^-?\d+$/.test(text)) {
        const n = BigInt(text);
        return n < I64_MIN || n > I64_MAX ? Number(text) : n;
      }
      throw native(`Invalid integer: ${text}`);
    };
    const literal = (word, value) => {
      if (ch.slice(pos, pos + word.length).join("") === word) {
        pos += word.length;
        return value;
      }
      throw native(`Invalid JSON value '${word}'`);
    };
    const parseValue = (depth) => {
      skipWs();
      if (pos >= ch.length) throw native("Unexpected end of JSON");
      const c = ch[pos];
      if ((c === "{" || c === "[") && depth >= MAX_JSON_DEPTH) throw tooDeep();
      if (c === "{") {
        pos++;
        skipWs();
        const obj = {};
        if (pos < ch.length && ch[pos] === "}") {
          pos++;
          return obj;
        }
        for (;;) {
          skipWs();
          if (pos >= ch.length || ch[pos] !== '"') throw native("Expected string key in JSON object");
          const key = parseString();
          skipWs();
          if (pos >= ch.length || ch[pos] !== ":") throw native("Expected ':' in JSON object");
          pos++;
          obj[dkey(key)] = parseValue(depth + 1);
          skipWs();
          if (pos >= ch.length) throw native("Unterminated JSON object");
          if (ch[pos] === "}") {
            pos++;
            break;
          }
          if (ch[pos] !== ",") throw native(`Expected '}' or ',' in JSON object, got '${ch[pos]}'`);
          pos++;
        }
        return obj;
      }
      if (c === "[") {
        pos++;
        skipWs();
        const list = [];
        if (pos < ch.length && ch[pos] === "]") {
          pos++;
          return list;
        }
        for (;;) {
          list.push(parseValue(depth + 1));
          skipWs();
          if (pos >= ch.length) throw native("Unterminated JSON array");
          if (ch[pos] === "]") {
            pos++;
            break;
          }
          if (ch[pos] !== ",") throw native(`Expected ']' or ',' in JSON array, got '${ch[pos]}'`);
          pos++;
        }
        return list;
      }
      if (c === '"') return parseString();
      if (c === "t") return literal("true", true);
      if (c === "f") return literal("false", false);
      if (c === "n") return literal("null", null);
      if (c === "-" || (c >= "0" && c <= "9")) return parseNumber();
      throw native(`Unexpected character '${c}' in JSON`);
    };
    return parseValue(0);
  };
  const toJsonPretty = (x, indent, open) => {
    if (x === null || x === undefined) return "null";
    if (x === true) return "true";
    if (x === false) return "false";
    if (typeof x === "bigint") return String(x);
    if (typeof x === "number") return Number.isFinite(x) ? showNumber(x) : "null";
    if (typeof x === "string") return jsonEscape(x);
    const ws = "  ".repeat(indent);
    const inner = "  ".repeat(indent + 1);
    if (Array.isArray(x)) {
      if (x.length === 0) return "[]";
      jsonEnter(x, open);
      const items = x.map((v) => inner + toJsonPretty(v, indent + 1, open));
      open.pop();
      return "[\n" + items.join(",\n") + "\n" + ws + "]";
    }
    if (isDict(x)) {
      const keys = Object.keys(x);
      if (keys.length === 0) return "{}";
      jsonEnter(x, open);
      const pairs = keys.map((p) => {
        const k = undkey(p);
        return inner + jsonEscape(typeof k === "string" ? k : show(k)) + ": " + toJsonPretty(x[p], indent + 1, open);
      });
      open.pop();
      return "{\n" + pairs.join(",\n") + "\n" + ws + "}";
    }
    return '"' + show(x).replace(/"/g, '\\"') + '"';
  };

  const builtins = {
    // Dicts: keys come back wi' their ain types (see `dkey`).
    keys: (d) => {
      if (!isDict(d)) throw native("keys() expects a dict");
      return dictKeys(d);
    },
    values: (d) => {
      if (!isDict(d)) throw native("values() expects a dict");
      return Object.values(d);
    },
    items: (d) => {
      if (!isDict(d)) throw native("items() needs a dictionary");
      return Object.keys(d).map((p) => [undkey(p), d[p]]);
    },
    // The second dict's values win; a key keeps its first place.
    dict_merge: (a, b) => {
      if (!isDict(a) || !isDict(b)) throw native("dict_merge() needs two dictionaries");
      return Object.assign({}, a, b);
    },
    json_stringify: (x) => toJson(x, []),

    // ---------- Strings by code point, like Rust's `chars()` (MDH-BUG-FLU-00116) ----------
    // upper/lower mirror `to_ascii_uppercase`: only a-z/A-Z change, so ß and é stay put.
    upper: (s) => {
      if (typeof s !== "string") throw native("upper() expects a string");
      return s.replace(/[a-z]+/g, (m) => m.toUpperCase());
    },
    lower: (s) => {
      if (typeof s !== "string") throw native("lower() expects a string");
      return s.replace(/[A-Z]+/g, (m) => m.toLowerCase());
    },
    heid: (x) => {
      if (Array.isArray(x)) {
        if (x.length === 0) throw native("Cannae get heid o' empty list!");
        return x[0];
      }
      if (typeof x === "string") {
        if (x === "") throw native("Cannae get heid o' empty string!");
        return cps(x)[0];
      }
      throw native("heid() expects a list or string");
    },
    tail: (x) => {
      if (Array.isArray(x)) return x.slice(1);
      if (typeof x === "string") return cps(x).slice(1).join("");
      throw native("tail() expects a list or string");
    },
    bum: (x) => {
      if (Array.isArray(x)) {
        if (x.length === 0) throw native("Cannae get bum o' empty list!");
        return x[x.length - 1];
      }
      if (typeof x === "string") {
        if (x === "") throw native("Cannae get bum o' empty string!");
        const a = cps(x);
        return a[a.length - 1];
      }
      throw native("bum() expects a list or string");
    },
    reverse: (x) => {
      if (Array.isArray(x)) return [...x].reverse();
      if (typeof x === "string") return cps(x).reverse().join("");
      throw native("reverse() expects a list or string");
    },
    // An empty separator gies back the whole string as one item, like the interpreter.
    split: (s, d) => {
      if (typeof s !== "string" || typeof d !== "string") throw native("split() expects two strings");
      return d === "" ? [s] : s.split(d);
    },

    // ---------- Strings (MDH-BUG-FLU-00114) ----------
    chars: (s) => cps(needStr(s, "chars() needs a string")),
    char_at: (s, i) => {
      needStr(s, "char_at() needs a string");
      let idx = asInt(i);
      if (idx === null) throw native("char_at() needs an integer index");
      const a = cps(s);
      const len = BigInt(a.length);
      if (idx < 0n) idx += len;
      if (idx < 0n || idx >= len) throw native(`Index ${idx} oot o' bounds fer string o' length ${len}`);
      return a[Number(idx)];
    },
    ord: (s) => {
      needStr(s, "ord() needs a string");
      if (s === "") throw native("Cannae get ord o' empty string!");
      return BigInt(s.codePointAt(0));
    },
    chr: (n) => {
      if (typeof n !== "bigint") throw native("chr() needs an integer");
      if (n < 0n || n > 0x10ffffn) throw native(`chr() needs a valid Unicode codepoint (0 to 1114111), got ${n}`);
      if (n >= 0xd800n && n <= 0xdfffn) throw native(`Invalid Unicode codepoint: ${n}`);
      return String.fromCodePoint(Number(n));
    },
    trim: (s) => trimEnd(trimStart(needStr(s, "trim() needs a string"), isMdhWs), isMdhWs),
    trim_start: (s) => trimStart(needStr(s, "trim_start() needs a string"), isMdhWs),
    trim_end: (s) => trimEnd(needStr(s, "trim_end() needs a string"), isMdhWs),
    strip_left: (s, chars) => {
      if (typeof s !== "string" || typeof chars !== "string") throw native("strip_left() needs two strings");
      const set = cps(chars);
      return trimStart(s, (c) => set.includes(c));
    },
    strip_right: (s, chars) => {
      if (typeof s !== "string" || typeof chars !== "string") throw native("strip_right() needs two strings");
      const set = cps(chars);
      return trimEnd(s, (c) => set.includes(c));
    },
    words: (s) => needStr(s, "words() needs a string").split(/[ \t\n\r]/).filter((w) => w !== ""),
    // Rust's `lines()`: split on "\n", drop a "\r" before it, an' no empty last line.
    lines: (s) => {
      const parts = needStr(s, "lines() needs a string").split("\n");
      const last = parts.pop();
      const out = parts.map((p) => (p.endsWith("\r") ? p.slice(0, -1) : p));
      if (last !== "") out.push(last);
      return out;
    },
    title: (s) => {
      needStr(s, "title() needs a string");
      let out = "";
      let fresh = true;
      for (const c of s) {
        if (isMdhWs(c)) {
          fresh = true;
          out += c;
        } else if (c < "\x80") {
          out += fresh ? c.toUpperCase() : c.toLowerCase();
          fresh = false;
        } else {
          out += c;
          fresh = false;
        }
      }
      return out;
    },
    capitalize: (s) => {
      needStr(s, "capitalize() needs a string");
      return s === "" ? "" : (s[0] < "\x80" ? s[0].toUpperCase() : s[0]) + s.slice(1);
    },
    swapcase: (s) => Array.from(needStr(s, "swapcase() needs a string"), swapAscii).join(""),
    center: (...args) => {
      if (args.length < 2 || args.length > 3) throw native("center() expects 2-3 arguments");
      const s = needStr(args[0], "center() needs a string");
      if (typeof args[1] !== "bigint") throw native("center() needs a width");
      let fill = " ";
      if (args.length === 3) fill = cps(needStr(args[2], "center() needs a fill character"))[0] ?? " ";
      const pad = padCount("center", s, args[1], fill);
      if (pad === 0n) return s;
      const left = pad / 2n;
      return fill.repeat(Number(left)) + s + fill.repeat(Number(pad - left));
    },
    pad_left: (...args) => {
      const [s, width, fill] = padArgs("pad_left", args);
      return fill.repeat(Number(padCount("pad_left", s, width, fill))) + s;
    },
    pad_right: (...args) => {
      const [s, width, fill] = padArgs("pad_right", args);
      return s + fill.repeat(Number(padCount("pad_right", s, width, fill)));
    },
    repeat: (s, n) => {
      if (typeof s !== "string" || typeof n !== "bigint") throw native("repeat() needs a string and an integer");
      if (n < 0n) throw native("Cannae repeat a negative number o' times!");
      allocCheck(utf8Len(s), n, "repeat()");
      return s === "" ? "" : s.repeat(Number(n));
    },
    replace: (s, from, to) => {
      if (typeof s !== "string" || typeof from !== "string" || typeof to !== "string") throw native("replace() needs three strings");
      return replaceAllCp(s, from, to);
    },
    replace_first: (s, from, to) => {
      if (typeof s !== "string" || typeof from !== "string" || typeof to !== "string") throw native("replace_first() needs three strings");
      if (from === "") return to + s;
      const i = s.indexOf(from);
      return i < 0 ? s : s.slice(0, i) + to + s.slice(i + from.length);
    },
    starts_with: (s, p) => {
      if (typeof s !== "string" || typeof p !== "string") throw native("starts_with() needs two strings");
      return s.startsWith(p);
    },
    starts_wi: (s, p) => {
      if (typeof s !== "string" || typeof p !== "string") throw native("starts_wi() needs two strings");
      return s.startsWith(p);
    },
    ends_with: (s, p) => {
      if (typeof s !== "string" || typeof p !== "string") throw native("ends_with() needs two strings");
      return s.endsWith(p);
    },
    ends_wi: (s, p) => {
      if (typeof s !== "string" || typeof p !== "string") throw native("ends_wi() needs two strings");
      return s.endsWith(p);
    },
    index_of: (x, needle) => {
      if (typeof x === "string" && typeof needle === "string") {
        const i = x.indexOf(needle);
        return i < 0 ? -1n : BigInt(cpIndex(x, i));
      }
      if (Array.isArray(x)) return BigInt(x.findIndex((v) => eq(v, needle)));
      throw native("index_of() needs a string/list and a value");
    },
    last_index_of: (s, needle) => {
      if (typeof s !== "string" || typeof needle !== "string") throw native("last_index_of() needs two strings");
      const i = s.lastIndexOf(needle);
      return i < 0 ? -1n : BigInt(cpIndex(s, i));
    },
    indices_o: (x, needle) => {
      if (Array.isArray(x)) {
        const out = [];
        x.forEach((v, i) => {
          if (eq(v, needle)) out.push(BigInt(i));
        });
        return out;
      }
      if (typeof x === "string") {
        if (typeof needle !== "string") throw native("indices_o() on string needs a string needle");
        if (needle === "") throw native("Cannae search fer an empty string, ya numpty!");
        const out = [];
        // Rust's `match_indices` skips overlaps: resume after each match.
        for (let i = x.indexOf(needle); i >= 0; i = x.indexOf(needle, i + needle.length)) out.push(BigInt(cpIndex(x, i)));
        return out;
      }
      throw native("indices_o() needs a list or string");
    },
    substring: (s, a, b) => {
      needStr(s, "substring() needs a string");
      let start = asInt(a);
      let end = asInt(b);
      if (start === null || end === null) throw native("substring() needs integer indices");
      const chars = cps(s);
      const len = BigInt(chars.length);
      if (start < 0n) start += len;
      if (end < 0n) end += len;
      start = start < 0n ? 0n : start > len ? len : start;
      end = end < 0n ? 0n : end > len ? len : end;
      if (end < start) end = start;
      return chars.slice(Number(start), Number(end)).join("");
    },
    substr_between: (s, start, end) => {
      if (typeof s !== "string" || typeof start !== "string" || typeof end !== "string") throw native("substr_between() needs three strings");
      const i = s.indexOf(start);
      if (i >= 0) {
        const after = i + start.length;
        const j = s.indexOf(end, after);
        if (j >= 0) return s.slice(after, j);
      }
      return null;
    },
    is_alpha: (s) => allAscii(needStr(s, "is_alpha() needs a string"), (c) => isLowerA(c) || isUpperA(c)),
    is_digit: (s) => allAscii(needStr(s, "is_digit() needs a string"), (c) => c >= "0" && c <= "9"),
    is_space: (s) => allAscii(needStr(s, "is_space() needs a string"), isMdhWs),
    is_upper: (s) => {
      needStr(s, "is_upper() needs a string");
      return /[A-Z]/.test(s) && !/[a-z]/.test(s);
    },
    is_lower: (s) => {
      needStr(s, "is_lower() needs a string");
      return /[a-z]/.test(s) && !/[A-Z]/.test(s);
    },
    is_blank: (s) => /^[ \t\n\f\r]*$/.test(needStr(s, "is_blank() needs a string")),
    is_empty: (x) => {
      if (typeof x === "string" || Array.isArray(x)) return x.length === 0;
      if (isDict(x)) return Object.keys(x).length === 0;
      throw native("is_empty() needs a string, list, or dict");
    },
    // Splits a list by a named predicate: [matching, rest].
    split_by: (l, pred) => {
      if (!Array.isArray(l) || typeof pred !== "string") throw native("split_by() needs a list and a predicate string");
      const tests = {
        even: (x) => typeof x === "bigint" && x % 2n === 0n,
        odd: (x) => typeof x === "bigint" && x % 2n !== 0n,
        positive: (x) => (typeof x === "bigint" && x > 0n) || (typeof x === "number" && x > 0),
        negative: (x) => (typeof x === "bigint" && x < 0n) || (typeof x === "number" && x < 0),
        truthy: (x) => truthy(x),
        nil: (x) => x === null || x === undefined,
        string: (x) => typeof x === "string",
        number: (x) => typeof x === "bigint" || typeof x === "number",
      };
      if (!Object.prototype.hasOwnProperty.call(tests, pred)) {
        throw native(`Unknown predicate '${pred}'. Try: even, odd, positive, negative, truthy, nil, string, number`);
      }
      const yes = [];
      const no = [];
      for (const x of l) (tests[pred](x) ? yes : no).push(x);
      return [yes, no];
    },

    // ---------- Lists (MDH-BUG-FLU-00114) ----------
    zip: (a, b) => {
      if (!Array.isArray(a) || !Array.isArray(b)) throw native("zip() needs two lists");
      return a.slice(0, Math.min(a.length, b.length)).map((x, i) => [x, b[i]]);
    },
    enumerate: (l) => needList(l, "enumerate() needs a list").map((v, i) => [BigInt(i), v]),
    flatten: (l) => {
      const out = [];
      for (const x of needList(l, "flatten() needs a list")) {
        if (Array.isArray(x)) out.push(...x);
        else out.push(x);
      }
      return out;
    },
    uniq: (l) => {
      const seen = new Set();
      const out = [];
      for (const x of needList(l, "uniq() needs a list")) {
        const k = debugKey(x, []);
        if (!seen.has(k)) {
          seen.add(k);
          out.push(x);
        }
      }
      return out;
    },
    unique: (l) => {
      const out = [];
      for (const x of needList(l, "unique() needs a list")) if (!out.some((y) => eq(x, y))) out.push(x);
      return out;
    },
    chunks: (l, n) => {
      needList(l, "chunks() needs a list");
      const size = asInt(n);
      if (size === null) throw native("chunks() needs an integer size");
      if (size <= 0n) throw native("chunks() size must be positive");
      return chunkList(l, clampCount(size, Math.max(l.length, 1)));
    },
    grup: (l, n) => {
      if (!Array.isArray(l) || typeof n !== "bigint") throw native("grup() needs a list and an integer");
      if (n <= 0n) throw native("grup() needs a positive chunk size");
      return chunkList(l, clampCount(n, Math.max(l.length, 1)));
    },
    pair_up: (l) => chunkList(needList(l, "pair_up() needs a list"), 2),
    interleave: (a, b) => {
      if (!Array.isArray(a) || !Array.isArray(b)) throw native("interleave() needs two lists");
      return interleaveLists(a, b);
    },
    fankle: (a, b) => {
      if (!Array.isArray(a) || !Array.isArray(b)) throw native("fankle() needs two lists");
      return interleaveLists(a, b);
    },
    ceilidh: (a, b) => {
      if (!Array.isArray(a) || !Array.isArray(b)) throw native("ceilidh needs two lists");
      return interleaveLists(a, b);
    },
    tak: (l, n) => {
      const count = Array.isArray(l) ? asInt(n) : null;
      if (count === null) throw native("tak() needs a list and an integer");
      return l.slice(0, clampCount(count, l.length));
    },
    drap: (l, n) => {
      const count = Array.isArray(l) ? asInt(n) : null;
      if (count === null) throw native("drap() needs a list and an integer");
      return l.slice(clampCount(count, l.length));
    },
    // Groups runs o' consecutive equal elements (compared with the run's first).
    grup_runs: (l) => {
      const out = [];
      let cur = [];
      for (const x of needList(l, "grup_runs() needs a list")) {
        if (cur.length === 0 || eq(cur[0], x)) cur.push(x);
        else {
          out.push(cur);
          cur = [x];
        }
      }
      if (cur.length > 0) out.push(cur);
      return out;
    },

    // ---------- Conversion an' bits (MDH-BUG-FLU-00114) ----------
    tae_bool: (x) => truthy(x),
    tae_binary: (n) => {
      if (typeof n !== "bigint") throw native("tae_binary() needs an integer");
      return asUnsigned(n).toString(2);
    },
    tae_hex: (n) => {
      if (typeof n !== "bigint") throw native("tae_hex() needs an integer");
      return asUnsigned(n).toString(16);
    },
    tae_octal: (n) => {
      if (typeof n !== "bigint") throw native("tae_octal() needs an integer");
      return asUnsigned(n).toString(8);
    },
    fae_binary: (s) => {
      if (typeof s !== "string") throw native("fae_binary() needs a string");
      const n = parseRadix(stripPrefix(s, "0b"), 2);
      if (n === null) throw native(`Cannae parse '${s}' as binary`);
      return n;
    },
    fae_hex: (s) => {
      if (typeof s !== "string") throw native("fae_hex() needs a string");
      const n = parseRadix(stripPrefix(s, "0x"), 16);
      if (n === null) throw native(`Cannae parse '${s}' as hex`);
      return n;
    },
    bit_an: (a, b) => {
      needInts("bit_an", a, b);
      return a & b;
    },
    bit_or: (a, b) => {
      needInts("bit_or", a, b);
      return a | b;
    },
    bit_xor: (a, b) => {
      needInts("bit_xor", a, b);
      return a ^ b;
    },
    bit_nae: (n) => {
      if (typeof n !== "bigint") throw native("bit_nae() needs an integer");
      return ~n;
    },
    bit_shove_left: (a, b) => BigInt.asIntN(64, a << BigInt(shiftArgs("bit_shove_left", a, b))),
    bit_shove_right: (a, b) => a >> BigInt(shiftArgs("bit_shove_right", a, b)),
    bit_coont: (n) => {
      if (typeof n !== "bigint") throw native("bit_coont() needs an integer");
      return BigInt(asUnsigned(n).toString(2).replace(/0/g, "").length);
    },

    // ---------- JSON an' assertions (MDH-BUG-FLU-00114) ----------
    json_parse: (s) => {
      if (typeof s !== "string") throw native("json_parse() expects a string, ya numpty!");
      return parseJson(s);
    },
    json_pretty: (x) => toJsonPretty(x, 0, []),
    json_stringify_pretty: (x) => toJsonPretty(x, 0, []),
    assert: (cond, msg) => {
      if (!truthy(cond)) throw native(`Assertion failed: ${show(msg)}`);
      return true;
    },
    assert_equal: (a, b) => {
      if (!eq(a, b)) throw native(`Assertion failed: expected ${show(a)} but got ${show(b)}`);
      return true;
    },
    assert_nae_equal: (a, b) => {
      if (eq(a, b)) throw native(`Assertion failed: ${show(a)} should not equal ${show(b)}`);
      return true;
    },

    // Dict helpers (MDH-BUG-FLU-00113); keys go through `dkey`, as in the interpreter's `DictValue`.
    dict_get: (d, k, dflt) => {
      if (!isDict(d)) throw native("dict_get() needs a dictionary");
      const p = dkey(k);
      return Object.prototype.hasOwnProperty.call(d, p) ? d[p] : dflt;
    },
    dict_has: (d, k) => {
      if (!isDict(d)) throw native("dict_has() needs a dictionary");
      return Object.prototype.hasOwnProperty.call(d, dkey(k));
    },
    // Returns a new dict; the original is left alone.
    dict_remove: (d, k) => {
      if (!isDict(d)) throw native("dict_remove() needs a dictionary");
      const out = Object.assign({}, d);
      delete out[dkey(k)];
      return out;
    },
    dict_invert: (d) => {
      if (!isDict(d)) throw native("dict_invert() needs a dictionary");
      const out = {};
      for (const p of Object.keys(d)) out[dkey(d[p])] = undkey(p);
      return out;
    },
    fae_pairs: (l) => {
      if (!Array.isArray(l)) throw native("fae_pairs() needs a list o' pairs");
      const out = {};
      for (const pair of l) if (Array.isArray(pair) && pair.length >= 2) out[dkey(pair[0])] = pair[1];
      return out;
    },

    // Higher-order builtins. The function gets only the item (an' the accumulator), and a
    // predicate's result is judged by `truthy`. A non-list or non-function raises the
    // interpreter's error; the line comes fae the call site via `hof`.
    gaun: (l, f) => hofList("gaun", l).map((x) => hofCall(f)(x)),
    sieve: (l, f) => hofList("sieve", l).filter((x) => truthy(hofCall(f)(x))),
    tumble: (l, init, f) => hofList("tumble", l).reduce((acc, x) => hofCall(f)(acc, x), init),
    ilk: (l, f) => {
      for (const x of hofList("ilk", l)) hofCall(f)(x);
      return null;
    },
    hunt: (l, f) => {
      for (const x of hofList("hunt", l)) if (truthy(hofCall(f)(x))) return x;
      return null;
    },
    ony: (l, f) => {
      for (const x of hofList("ony", l)) if (truthy(hofCall(f)(x))) return true;
      return false;
    },
    aw: (l, f) => {
      for (const x of hofList("aw", l)) if (!truthy(hofCall(f)(x))) return false;
      return true;
    },
    grup_up: (l, f) => {
      const out = {};
      for (const x of hofList("grup_up", l)) {
        const p = dkey(hofCall(f)(x));
        if (Object.prototype.hasOwnProperty.call(out, p)) out[p].push(x);
        else out[p] = [x];
      }
      return out;
    },
    pairt_by: (l, f) => {
      const yes = [];
      const no = [];
      for (const x of hofList("pairt_by", l)) (truthy(hofCall(f)(x)) ? yes : no).push(x);
      return [yes, no];
    },

    // Creels (sets): a JS Set keys BigInt/string/bool by value an' lists, dicts an' creels by
    // identity, as the interpreter's `ValueKey` does. Display an' list order are sorted by `show`.
    creel: (x) => {
      if (Array.isArray(x)) return new Set(x.map((v) => (v === undefined ? null : v)));
      if (x instanceof Set) return x;
      throw native("creel() needs a list tae make a set fae");
    },
    empty_creel: () => new Set(),
    toss_in: (s, x) => {
      if (!(s instanceof Set)) throw native("toss_in() needs a creel (set)");
      s.add(x === undefined ? null : x);
      return s;
    },
    heave_oot: (s, x) => {
      if (!(s instanceof Set)) throw native("heave_oot() needs a creel (set)");
      s.delete(x === undefined ? null : x);
      return s;
    },
    is_in_creel: (s, x) => {
      if (!(s instanceof Set)) throw native("is_in_creel() needs a creel (set)");
      return s.has(x === undefined ? null : x);
    },
    creels_thegither: (a, b) => {
      if (!(a instanceof Set && b instanceof Set)) throw native("creels_thegither() needs two creels");
      return new Set([...a, ...b]);
    },
    creels_baith: (a, b) => {
      if (!(a instanceof Set && b instanceof Set)) throw native("creels_baith() needs two creels");
      return new Set([...a].filter((v) => b.has(v)));
    },
    creels_differ: (a, b) => {
      if (!(a instanceof Set && b instanceof Set)) throw native("creels_differ() needs two creels");
      return new Set([...a].filter((v) => !b.has(v)));
    },
    creel_tae_list: (s) => {
      if (!(s instanceof Set)) throw native("creel_tae_list() needs a creel");
      return [...s].map((v) => [show(v), v]).sort((p, q) => (p[0] < q[0] ? -1 : p[0] > q[0] ? 1 : 0)).map((p) => p[1]);
    },
    is_subset: (a, b) => {
      if (!(a instanceof Set && b instanceof Set)) throw native("is_subset() needs two creels");
      return [...a].every((v) => b.has(v));
    },
    is_superset: (a, b) => {
      if (!(a instanceof Set && b instanceof Set)) throw native("is_superset() needs two creels");
      return [...b].every((v) => a.has(v));
    },
    is_disjoint: (a, b) => {
      if (!(a instanceof Set && b instanceof Set)) throw native("is_disjoint() needs two creels");
      return [...a].every((v) => !b.has(v));
    },
    len: (x) => {
      if (typeof x === "string") {
        let n = 0;
        for (const _c of x) n++;
        return BigInt(n);
      }
      if (Array.isArray(x)) return BigInt(x.length);
      if (x instanceof Set) return BigInt(x.size);
      if (x !== null && typeof x === "object") return BigInt(Object.keys(x).length);
      throw native("len() expects a string, list, dict, creel, or bytes");
    },
    tae_int: (x) => {
      if (typeof x === "bigint") return x;
      if (typeof x === "number") return f64ToI64(x);
      if (typeof x === "string") {
        if (/^[+-]?\d+$/.test(x)) {
          const n = BigInt(x);
          if (n >= I64_MIN && n <= I64_MAX) return n;
        }
        throw native(`Cannae turn '${x}' intae an integer`);
      }
      if (typeof x === "boolean") return x ? 1n : 0n;
      throw native(`Cannae turn ${typeName(x)} intae an integer`);
    },
    tae_float: (x) => {
      if (typeof x === "bigint") return Number(x);
      if (typeof x === "number") return x;
      if (typeof x === "string") {
        // Rust's f64 parse: nae whitespace, nae hex, "inf"/"infinity"/"nan" in ony case.
        const m = /^([+-]?)(?:(inf|infinity)|(nan)|(\d+\.?\d*(?:[eE][+-]?\d+)?|\.\d+(?:[eE][+-]?\d+)?))$/i.exec(x);
        if (m) {
          if (m[2]) return m[1] === "-" ? -Infinity : Infinity;
          if (m[3]) return NaN;
          return Number(x);
        }
        throw native(`Cannae turn '${x}' intae a float`);
      }
      throw native(`Cannae turn ${typeName(x)} intae a float`);
    },
    range: (start, end) => {
      const s = asInt(start);
      const e = asInt(end);
      if (s === null || e === null) throw native("range() expects integers");
      return rangeList(s, e, false);
    },
    abs: (x) => {
      if (typeof x === "bigint") {
        if (x === I64_MIN) throw nativeOverflow("abs()");
        return x < 0n ? -x : x;
      }
      if (typeof x === "number") return Math.abs(x);
      throw native("abs() expects a number");
    },
    // Rust's f64::min/max ignore a NaN operand.
    min: (a, b) => {
      if (typeof a === "bigint" && typeof b === "bigint") return a < b ? a : b;
      if (typeof a === "number" && typeof b === "number") {
        if (Number.isNaN(a)) return b;
        if (Number.isNaN(b)) return a;
        return Math.min(a, b);
      }
      throw native("min() expects two numbers of the same type");
    },
    max: (a, b) => {
      if (typeof a === "bigint" && typeof b === "bigint") return a > b ? a : b;
      if (typeof a === "number" && typeof b === "number") {
        if (Number.isNaN(a)) return b;
        if (Number.isNaN(b)) return a;
        return Math.max(a, b);
      }
      throw native("max() expects two numbers of the same type");
    },
    floor: (x) => {
      if (typeof x === "number") return f64ToI64(Math.floor(x));
      if (typeof x === "bigint") return x;
      throw native("floor() expects a number");
    },
    ceil: (x) => {
      if (typeof x === "number") return f64ToI64(Math.ceil(x));
      if (typeof x === "bigint") return x;
      throw native("ceil() expects a number");
    },
    // Rust rounds half away fae zero; Math.round rounds half up.
    round: (x) => {
      if (typeof x === "number") return f64ToI64(x < 0 ? -Math.round(-x) : Math.round(x));
      if (typeof x === "bigint") return x;
      throw native("round() expects a number");
    },
    sqrt: (x) => {
      if (typeof x === "number" || typeof x === "bigint") return Math.sqrt(Number(x));
      throw native("sqrt() expects a number");
    },
    scran: (x, start, end) => {
      const s0 = asInt(start);
      const e0 = asInt(end);
      if (s0 === null || e0 === null) throw native("scran() needs integer indices");
      const isStr = typeof x === "string";
      if (!isStr && !Array.isArray(x)) throw native("scran() expects a list or string");
      const items = isStr ? Array.from(x) : x;
      const len = items.length;
      let s = Number(s0);
      let e = Number(e0);
      if (s < 0) s = len + s;
      if (e < 0) e = len + e;
      s = Math.min(Math.max(s, 0), len);
      e = Math.min(Math.max(e, 0), len);
      if (e < s) e = s;
      const out = items.slice(s, e);
      return isStr ? out.join("") : out;
    },
    // Exact integer total unless a float is present.
    sumaw: (arr) => {
      if (!Array.isArray(arr)) throw native("sumaw() expects a list");
      let intSum = 0n;
      let sum = 0;
      let isFloat = false;
      for (const item of arr) {
        if (typeof item === "bigint") {
          intSum += item;
          sum += Number(item);
        } else if (typeof item === "number") {
          sum += item;
          isFloat = true;
        } else {
          throw native("sumaw() expects a list of numbers");
        }
      }
      if (isFloat) return sum;
      if (intSum > I64_MAX || intSum < I64_MIN) throw nativeOverflow("sumaw()");
      return intSum;
    },
    coont: (x, item) => {
      if (Array.isArray(x)) {
        let n = 0n;
        for (const e of x) if (eq(e, item)) n++;
        return n;
      }
      if (typeof x === "string") {
        if (typeof item !== "string") throw native("coont() on string needs a string tae count");
        // Rust's `matches("")` hits before each code point an' at the end.
        if (item === "") return BigInt(cps(x).length + 1);
        return BigInt(x.split(item).length - 1);
      }
      throw native("coont() expects a list or string");
    },

    // ---------- maths, matchin' the interpreter's natives ----------
    pooer: (base, exp) => {
      const tb = typeof base;
      const te = typeof exp;
      if (tb === "bigint" && te === "bigint") {
        if (exp < 0n) return floatPowInt(Number(base), exp);
        if (base === 0n || base === 1n) return exp === 0n ? 1n : base;
        if (base === -1n) return exp % 2n === 0n ? 1n : -1n;
        // |base| >= 2 overflows an i64 lang afore exp reaches 64.
        if (exp > 64n) throw nativeOverflow("pooer()");
        const r = base ** exp;
        if (r > I64_MAX || r < I64_MIN) throw nativeOverflow("pooer()");
        return r;
      }
      if (tb === "number" && te === "bigint") return floatPowInt(base, exp);
      if (tb === "number" && te === "number") return Math.pow(base, exp);
      if (tb === "bigint" && te === "number") return Math.pow(Number(base), exp);
      throw native("pooer() needs twa numbers");
    },
    pow: (base, exp) => Math.pow(f64Of(base, "pow() needs numbers"), f64Of(exp, "pow() needs numbers")),
    sin: floatFn("sin", Math.sin),
    cos: floatFn("cos", Math.cos),
    tan: floatFn("tan", Math.tan),
    asin: floatFn("asin", Math.asin),
    acos: floatFn("acos", Math.acos),
    atan: floatFn("atan", Math.atan),
    atan2: (y, x) => Math.atan2(f64Of(y, "atan2() needs numbers"), f64Of(x, "atan2() needs numbers")),
    hypot: (x, y) => Math.hypot(f64Of(x, "hypot() needs numbers"), f64Of(y, "hypot() needs numbers")),
    log: floatFn("log", Math.log),
    log10: floatFn("log10", Math.log10),
    log2: floatFn("log2", Math.log2),
    exp: floatFn("exp", Math.exp),
    // Rust's to_degrees / to_radians multiply by the same constants.
    degrees: floatFn("degrees", (f) => f * (180 / Math.PI)),
    radians: floatFn("radians", (f) => f * (Math.PI / 180)),
    PI: Math.PI,
    E: Math.E,
    TAU: 2 * Math.PI,
    pi: () => Math.PI,
    e: () => Math.E,
    tau: () => 2 * Math.PI,
    sign: (x) => {
      const f = f64Of(x, "sign() needs a number");
      return f > 0 ? 1n : f < 0 ? -1n : 0n;
    },
    trunc: (x) => {
      if (typeof x === "bigint") return x;
      if (typeof x === "number") return f64ToI64(Math.trunc(x));
      throw native("trunc() needs a number");
    },
    clamp: (n, lo, hi) => {
      if (typeof n === "bigint" && typeof lo === "bigint" && typeof hi === "bigint") {
        const m = n > lo ? n : lo;
        return m < hi ? m : hi;
      }
      if (typeof n === "number" && typeof lo === "number" && typeof hi === "number") return fmin(fmax(n, lo), hi);
      throw native("clamp() needs three numbers o' the same type");
    },
    lerp: (a, b, t) => {
      const fa = f64Of(a, "lerp() needs numbers");
      const fb = f64Of(b, "lerp() needs numbers");
      const ft = f64Of(t, "lerp() needs numbers");
      return fa + (fb - fa) * ft;
    },
    gcd: (a, b) => {
      if (typeof a !== "bigint" || typeof b !== "bigint") throw native("gcd() needs two integers");
      const g = gcdBig(a, b);
      if (g > I64_MAX) throw nativeOverflow("gcd()");
      return g;
    },
    lcm: (a, b) => {
      if (typeof a !== "bigint" || typeof b !== "bigint") throw native("lcm() needs two integers");
      if (a === 0n || b === 0n) return 0n;
      const l = ((a < 0n ? -a : a) / gcdBig(a, b)) * (b < 0n ? -b : b);
      if (l > I64_MAX) throw nativeOverflow("lcm()");
      return l;
    },
    factorial: (n) => {
      if (typeof n !== "bigint") throw native("factorial() needs an integer");
      if (n < 0n) throw native("Cannae calculate factorial o' negative number!");
      if (n > 20n) throw native("Factorial too big! Max is 20");
      let r = 1n;
      for (let i = 2n; i <= n; i++) r *= i;
      return r;
    },
    is_even: (n) => {
      if (typeof n !== "bigint") throw native("is_even() needs an integer");
      return n % 2n === 0n;
    },
    is_odd: (n) => {
      if (typeof n !== "bigint") throw native("is_odd() needs an integer");
      return n % 2n !== 0n;
    },
    is_prime: (n) => {
      if (typeof n !== "bigint") throw native("is_prime() needs an integer");
      if (n < 2n) return false;
      if (n === 2n) return true;
      if (n % 2n === 0n) return false;
      const root = f64ToI64(Math.sqrt(Number(n)));
      for (let i = 3n; i <= root; i += 2n) if (n % i === 0n) return false;
      return true;
    },
    average: (arr) => {
      const items = numList(arr, "average");
      if (items.length === 0) throw native("Cannae calculate average o' empty list!");
      let sum = 0;
      for (const item of items) sum += f64Of(item, "average() needs a list o' numbers");
      return sum / items.length;
    },
    median: (arr) => {
      const items = numList(arr, "median");
      if (items.length === 0) throw native("Cannae calculate median o' empty list!");
      const ns = items.map((item) => {
        const f = f64Of(item, "median() needs a list o' numbers");
        if (typeof item === "number" && Number.isNaN(f)) throw native("median() cannae handle NaN");
        return f;
      });
      ns.sort((a, b) => a - b);
      const mid = Math.floor(ns.length / 2);
      return ns.length % 2 === 0 ? (ns[mid - 1] + ns[mid]) / 2 : ns[mid];
    },
    // Exact integer product unless a float is present; a zero wins even past an overflow.
    product: (arr) => {
      const items = numList(arr, "product");
      if (items.length === 0) return 1n;
      let prod = 1;
      let intProd = 1n;
      let overflowed = false;
      let isFloat = false;
      let zero = false;
      for (const item of items) {
        if (typeof item === "bigint") {
          prod *= Number(item);
          if (item === 0n) zero = true;
          if (!overflowed) {
            intProd *= item;
            if (intProd > I64_MAX || intProd < I64_MIN) overflowed = true;
          }
        } else if (typeof item === "number") {
          prod *= item;
          isFloat = true;
        } else {
          throw native("product() needs a list o' numbers");
        }
      }
      if (isFloat) return prod;
      if (zero) return 0n;
      if (overflowed) throw nativeOverflow("product()");
      return intProd;
    },
    minaw: (arr) => {
      const items = numList(arr, "minaw");
      if (items.length === 0) throw native("Cannae find minimum o' empty list!");
      const bad = () => native("minaw() needs a list o' comparable numbers");
      let best = items[0];
      if (typeof best !== "bigint" && typeof best !== "number") throw bad();
      for (let i = 1; i < items.length; i++) {
        if (typeof items[i] !== typeof best) throw bad();
        if (items[i] < best) best = items[i];
      }
      return best;
    },
    maxaw: (arr) => {
      const items = numList(arr, "maxaw");
      if (items.length === 0) throw native("Cannae find maximum o' empty list!");
      const bad = () => native("maxaw() needs a list o' comparable numbers");
      let best = items[0];
      if (typeof best !== "bigint" && typeof best !== "number") throw bad();
      for (let i = 1; i < items.length; i++) {
        if (typeof items[i] !== typeof best) throw bad();
        if (items[i] > best) best = items[i];
      }
      return best;
    },
    range_o: (arr) => {
      const items = numList(arr, "range_o");
      if (items.length === 0) throw native("Cannae get range o' empty list!");
      let lo = Number.MAX_VALUE;
      let hi = -Number.MAX_VALUE;
      for (const item of items) {
        const f = f64Of(item, "range_o() needs a list o' numbers");
        if (f < lo) lo = f;
        if (f > hi) hi = f;
      }
      return hi - lo;
    },
    random,
    random_int: (min, max) => {
      const lo = asInt(min);
      const hi = asInt(max);
      if (lo === null || hi === null) throw native("random_int() needs integer bounds");
      if (lo > hi) throw native("random_int() min must be <= max");
      return randomIntInclusive(lo, hi);
    },
    random_choice: (arr) => {
      if (!Array.isArray(arr)) throw native("random_choice() needs a list");
      if (arr.length === 0) return null;
      return arr[Math.floor(Math.random() * arr.length)];
    },

    noo: () => BigInt(Date.now()),
    tick: () => {
      if (typeof process !== "undefined" && process.hrtime && process.hrtime.bigint) {
        return process.hrtime.bigint();
      }
      return BigInt(Date.now()) * 1000000n; // Fallback for browser
    },
    bide: (ms) => {
      const end = Date.now() + Number(ms);
      while (Date.now() < end) {} // Busy wait (sync)
    },
    // snooze(ms): the interpreter's blockin' sleep (an integer, or a float it truncates).
    snooze: (ms) => {
      const n = asInt(ms);
      if (n === null) throw native("snooze() needs an integer (milliseconds)");
      if (n < 0n) throw native("Cannae snooze fer negative time, ya daftie!");
      sleepSync(Number(n));
      return null;
    },
    // get_key(): the next key as a string ("" when nane is waitin'), named like the
    // interpreter's: a character, "\n", "\x1b", "\x08", or Left/Right/Up/Down.
    get_key: () => readKey(),
  };

  // JS libraries (the audio runtime, three.js) take plain numbers.
  const nums = (args) => args.map((a) => (typeof a === "bigint" ? Number(a) : a));
  const numArgs = (f) => (...args) => f(...nums(args));

  // ---------- the JS interop boundary (tri / three.js, `fetch`ed JS modules) ----------
  //
  // A foreign JS object is handed tae mdhavers code wrapped in a Proxy. Every
  // property write an' every method call through it turns mdhavers integers
  // (BigInts) intae plain numbers, so `sun.position.x = 2` an'
  // `kamera.luik_at(0, 0, 0)` reach three.js as numbers. Objects that come back
  // are wrapped the same way. Numbers that come back are JS numbers, so they read
  // as floats. Ordinary mdhavers values never pass through here, so plain
  // mdhavers code pays naething fer it.
  const proxyOf = new WeakMap(); // foreign object -> its proxy
  const targetOf = new WeakMap(); // proxy -> foreign object
  const callbacks = new WeakMap(); // mdhavers function -> its JS-facing wrapper
  const classy = new WeakMap(); // function -> is it an ES `class`?
  // Scots names fer three.js methods.
  const ALIASES = {
    __proto__: null,
    adde: "add",
    remuiv: "remove",
    cloan: "clone",
    dyspos: "dispose",
    luik_at: "lookAt",
    set_sise: "setSize",
    set_pixel_ratio: "setPixelRatio",
  };
  // An mdhavers class, struct or instance o' one: never wrapped.
  const isMdh = (v) => {
    if (typeof v === "function") return v.__mdh_kin === true;
    const proto = Object.getPrototypeOf(v);
    return proto !== null && typeof proto.constructor === "function" && proto.constructor.__mdh_kin === true;
  };
  const isClass = (f) => {
    let c = classy.get(f);
    if (c === undefined) {
      c = /^class\b/.test(Function.prototype.toString.call(f));
      classy.set(f, c);
    }
    return c;
  };

  // mdhavers value -> JS value. Lists an' dicts holdin' integers are copied;
  // onything wi'oot an integer in it is passed as itsel'.
  const toJs = (v, seen) => {
    switch (typeof v) {
      case "bigint":
        return Number(v);
      case "function": {
        const t = targetOf.get(v);
        if (t !== undefined) return t;
        if (isMdh(v)) return v;
        let w = callbacks.get(v);
        if (w === undefined) {
          const fn = v;
          w = function (...args) {
            return toJs(fn.apply(this, args.map(fromJs)));
          };
          callbacks.set(v, w);
        }
        return w;
      }
      case "object": {
        if (v === null) return v;
        const t = targetOf.get(v);
        if (t !== undefined) return t;
        const list = Array.isArray(v);
        if (!list && !isDict(v)) return v;
        if (seen === undefined) seen = new Set();
        if (seen.has(v)) return v;
        seen.add(v);
        let out = null;
        for (const k of list ? v.keys() : Object.keys(v)) {
          const c = toJs(v[k], seen);
          if (c !== v[k]) {
            if (out === null) out = list ? v.slice() : Object.assign(Object.create(Object.getPrototypeOf(v)), v);
            out[k] = c;
          }
        }
        seen.delete(v);
        return out === null ? v : out;
      }
      default:
        return v;
    }
  };
  const jsArgs = (args) => args.map((a) => toJs(a));

  // Ca' a foreign function wi' converted arguments; an ES class is constructed,
  // as mdhavers ca's a class like a function.
  const callForeign = (f, self, args) => {
    const a = jsArgs(args);
    return fromJs(isClass(f) ? Reflect.construct(f, a) : Reflect.apply(f, self, a));
  };

  const foreignHandler = {
    get(t, k) {
      let v = Reflect.get(t, k);
      if (typeof k !== "string" || k === "constructor") return v;
      if (v === undefined) {
        const real = ALIASES[k];
        if (real !== undefined && typeof t[real] === "function") {
          v = t[real];
        } else if (k === "loop" && typeof t.setAnimationLoop === "function") {
          // tri's `renderar.loop(|dt| ...)`: dt in seconds.
          return (fn) => fromJs(t.setAnimationLoop((ms) => fn(ms / 1000)));
        }
      }
      if (v === null || (typeof v !== "object" && typeof v !== "function")) return v;
      // A Proxy must hand back a frozen property's ain value.
      const d = Object.getOwnPropertyDescriptor(t, k);
      if (d !== undefined && !d.configurable && d.writable === false) return v;
      if (typeof v === "function") return isMdh(v) ? v : boundForeign(v, t);
      return fromJs(v);
    },
    set(t, k, v) {
      return Reflect.set(t, k, toJs(v));
    },
    apply(t, self, args) {
      const s = self !== null && self !== undefined ? targetOf.get(self) : undefined;
      return callForeign(t, s === undefined ? self : s, args);
    },
    construct(t, args) {
      return fromJs(Reflect.construct(t, jsArgs(args)));
    },
  };

  // A foreign method read aff `self`: ca'd wi' `self` as `this` (so it can be
  // passed aroond like a bound method), an' its ain properties (statics like
  // `Buffer.alloc`) still reachable through the proxy.
  const boundForeign = (f, self) => {
    const p = new Proxy(f, {
      __proto__: foreignHandler,
      apply: (fn, _this, args) => callForeign(fn, self, args),
    });
    targetOf.set(p, f);
    return p;
  };

  // Wrap a foreign JS object (or function) fer mdhavers code.
  const foreign = (obj) => {
    if (obj === null || (typeof obj !== "object" && typeof obj !== "function")) return obj;
    if (targetOf.has(obj) || isMdh(obj)) return obj;
    let p = proxyOf.get(obj);
    if (p === undefined) {
      p = new Proxy(obj, foreignHandler);
      proxyOf.set(obj, p);
      targetOf.set(p, obj);
    }
    return p;
  };
  // JS value -> mdhavers value: numbers stay JS numbers (floats); objects are wrapped.
  const fromJs = (v) => foreign(v);

  // A `kin` (or `thing`) is a JS class wrapped so it can be ca'd wi'oot `new`,
  // just as the interpreter lets ye ca' a class like a function.
  const kin = (cls, isStruct) => {
    cls.__mdh_kin = true;
    if (isStruct) cls.__mdh_struct = true;
    return new Proxy(cls, { apply: (target, _self, args) => new target(...args) });
  };

  const hurl = (message, line) => fail(`Hurled at line ${line}: ${show(message)}`);

  // Whit a `gin_it_gangs_wrang` binding sees: the error's message string.
  const caught = (e) => (e instanceof Error ? e.message : show(e));

  const and = (l, r) => (truthy(l) ? r() : l);
  const or = (l, r) => (truthy(l) ? l : r());

  return {
    show, truthy, eq, fail, typeName, isDict, dkey, hof, add, sub, mul, div, mod, neg, lt, le, gt, ge,
    idx, setIdx, slice, rangeOf, inRange, iter, builtins, nums, numArgs, kin, hurl, caught, and, or,
    foreign, jsArgs,
  };
})();
