// mdhavers core runtime - display, truthiness, equality and checked arithmetic,
// kept in step wi' the interpreter's Value semantics (src/value.rs).
const __havers_core = (() => {
  const isKin = (x) => typeof x === "function" && x.__mdh_kin === true;

  // Rust's f64 Display never uses exponent notation; JS String() does for
  // very big or very wee numbers, so expand the exponent form by hand.
  const showNumber = (n) => {
    if (Number.isNaN(n)) return "NaN";
    if (n === Infinity) return "inf";
    if (n === -Infinity) return "-inf";
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
    if (x === null || x === undefined || x === false || x === 0 || x === "") return false;
    if (Array.isArray(x)) return x.length > 0;
    if (x instanceof Set) return x.size > 0;
    if (isDict(x)) return Object.keys(x).length > 0;
    return true;
  };

  // Mirrors `impl PartialEq for Value`: lists an' dicts compare structurally, creels
  // by their members, the rest by identity. `path` holds the container pairs being
  // compared right now; meeting one again counts as equal, so cycles finish.
  const eq = (a, b, path = []) => {
    if (a === undefined) a = null;
    if (b === undefined) b = null;
    if (a instanceof Set && b instanceof Set) {
      if (a.size !== b.size) return false;
      for (const item of a) if (!b.has(item)) return false;
      return true;
    }
    const lists = Array.isArray(a) && Array.isArray(b);
    const dicts = isDict(a) && isDict(b);
    if (!lists && !dicts) return a === b;
    if (a === b) return true;
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

  const typeName = (x) => {
    if (x === null || x === undefined) return "naething";
    if (typeof x === "boolean") return "bool";
    if (typeof x === "number") return Number.isInteger(x) ? "integer" : "float";
    if (typeof x === "string") return "string";
    if (Array.isArray(x)) return "list";
    if (x instanceof Set) return "creel";
    if (typeof x === "function") return isKin(x) ? (x.__mdh_struct ? "struct" : "class") : "function";
    const ctor = Object.getPrototypeOf(x) === null ? null : x.constructor;
    if (ctor && ctor !== Object && !ctor.__mdh_struct) return "instance";
    return "dict";
  };

  const add = (a, b, line) => {
    if (typeof a === "number" && typeof b === "number") return a + b;
    if (typeof a === "string" || typeof b === "string") return show(a) + show(b);
    if (Array.isArray(a) && Array.isArray(b)) return [...a, ...b];
    throw fail(`That's pure mince! Type error at line ${line}: Cannae add ${typeName(a)} an' ${typeName(b)}`);
  };

  const divByZero = (line) => fail(`Ye numpty! Tryin' tae divide by zero at line ${line}`);
  const div = (a, b, line) => {
    if (b === 0) throw divByZero(line);
    return a / b;
  };
  // Integer division fer two integer literals (Rust truncates toward zero).
  const idiv = (a, b, line) => {
    if (b === 0) throw divByZero(line);
    return Math.trunc(a / b);
  };
  const mod = (a, b, line) => {
    if (b === 0) throw divByZero(line);
    return a % b;
  };

  const oob = (i, size, line) =>
    fail(`Hoachin'! Index ${i} is oot o' bounds (size is ${size}) at line ${line}`);
  const idx = (o, i, line) => {
    if (Array.isArray(o) && Number.isInteger(i)) {
      const j = i < 0 ? o.length + i : i;
      if (j < 0 || j >= o.length) throw oob(i, o.length, line);
      return o[j];
    }
    if (typeof o === "string" && Number.isInteger(i)) {
      const chars = Array.from(o);
      const j = i < 0 ? chars.length + i : i;
      if (j < 0 || j >= chars.length) throw oob(i, chars.length, line);
      return chars[j];
    }
    if (o !== null && typeof o === "object" && Object.getPrototypeOf(o) === Object.prototype) {
      if (!Object.prototype.hasOwnProperty.call(o, i)) {
        throw fail(`Awa' an bile yer heid! '${show(i)}' hasnae been defined yet at line ${line}`);
      }
      return o[i];
    }
    return o[i];
  };

  // `fer` iterates ower a snapshot, like the interpreter.
  const iter = (x, line) => {
    if (Array.isArray(x)) return x.slice();
    if (typeof x === "string") return Array.from(x);
    throw fail(`That's pure mince! Type error at line ${line}: Cannae iterate ower a ${typeName(x)}`);
  };

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

  return { show, truthy, eq, fail, typeName, add, div, idiv, mod, idx, iter, kin, hurl, caught, and, or };
})();
