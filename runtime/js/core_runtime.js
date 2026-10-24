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
      return "{" + Object.keys(x).map((k) => '"' + k + '": ' + show(x[k])).join(", ") + "}";
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
    if (o !== null && typeof o === "object" && Object.getPrototypeOf(o) === Object.prototype) {
      if (!Object.prototype.hasOwnProperty.call(o, i)) {
        throw fail(`Awa' an bile yer heid! '${show(i)}' hasnae been defined yet at line ${line}`);
      }
      return o[i];
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
    o[i] = v;
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

  const builtins = {
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
    show, truthy, eq, fail, typeName, add, sub, mul, div, mod, neg, lt, le, gt, ge,
    idx, setIdx, slice, rangeOf, inRange, iter, builtins, nums, numArgs, kin, hurl, caught, and, or,
    foreign, jsArgs,
  };
})();
