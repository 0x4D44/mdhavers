// File bundles use the interpreter's environments: declarations replace local
// bindings, assignments follow shared cells, and aliases copy current values.
function __mdh_scope(parent, parameters = []) {
  const bindings = Object.create(null);
  const find = (name) => Object.prototype.hasOwnProperty.call(bindings, name) ? bindings[name] : parent?.find(name);
  const define = (name, value) => { bindings[name] = { value }; return true; };
  const missing = (name) => __havers.fail(`Dinnae ken whit '${name}' is!`);
  const scope = {
    bindings, find, define,
    get: new Proxy(Object.create(null), {
      get: (_, name) => { const cell = find(name); if (!cell) throw missing(name); return cell.value; },
      set: (_, name, value) => { const cell = find(name); if (!cell) throw missing(name); cell.value = value; return true; },
    }),
    def: new Proxy(Object.create(null), { set: (_, name, value) => define(name, value) }),
  };
  for (const [name, value] of parameters) define(name, value);
  return scope;
}

const __mdh_builtins = __mdh_scope(null);
const __mdh_globals = __mdh_scope(__mdh_builtins);
const __mdh_factories = [];
const __mdh_paths = [];
const __mdh_directories = [];
const __mdh_imports = [];
let __mdh_current_dir;
const __mdh_cache = new Map();
const __mdh_loading = [];

function __mdh_fetch(id, scope, alias) {
  if (__mdh_loading.includes(id)) {
    const chain = [...__mdh_loading, id].map((key) => __mdh_paths[key]).join(' -> ');
    throw __havers.fail("The import's gone in a fankle! Circular import detected: " + chain);
  }
  let exports = __mdh_cache.get(id);
  if (!exports) {
    __mdh_loading.push(id);
    const callerDirectory = __mdh_current_dir;
    __mdh_current_dir = __mdh_directories[id];
    try {
      const module = __mdh_scope(__mdh_globals);
      __mdh_factories[id](module);
      // Copy the table, retaining the cells: later declarations in the module
      // replace its binding without replacing a previously exported cell.
      exports = Object.assign(Object.create(null), module.bindings);
      __mdh_cache.set(id, exports);
    } finally {
      __mdh_current_dir = callerDirectory;
      __mdh_loading.pop();
    }
  }
  if (alias === null) {
    Object.assign(scope.bindings, exports);
  } else {
    const snapshot = Object.create(null);
    for (const [name, cell] of Object.entries(exports)) snapshot[__havers.dkey(name)] = cell.value;
    scope.define(alias, snapshot);
  }
}
