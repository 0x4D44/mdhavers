use std::collections::{HashMap, HashSet};
use std::path::Path;

use wasmtime::{Caller, Engine, Linker, Memory, MemoryType, Module, Store};

type Handle = i64;

#[derive(Debug, Clone)]
enum HostValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Handle>),
    Dict(HashMap<ValueKey, Handle>),
    NativeObject(TriObject),
    NativeCtor(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ValueKey {
    Nil,
    Bool(bool),
    Int(i64),
    Float(u64),
    String(String),
    List(Handle),
    Dict(Handle),
    NativeObject(Handle),
    NativeCtor(Handle),
}

#[derive(Debug, Clone)]
struct TriObject {
    kind: String,
    fields: HashMap<String, Handle>,
}

impl TriObject {
    fn new(kind: &str) -> Self {
        TriObject {
            kind: kind.to_string(),
            fields: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct HostStore {
    values: Vec<HostValue>,
}

impl HostStore {
    fn new() -> Self {
        HostStore {
            values: vec![HostValue::Nil],
        }
    }

    fn alloc(&mut self, value: HostValue) -> Handle {
        self.values.push(value);
        (self.values.len() - 1) as Handle
    }

    fn get(&self, handle: Handle) -> Option<&HostValue> {
        let idx = handle as usize;
        self.values.get(idx)
    }

    fn get_mut(&mut self, handle: Handle) -> Option<&mut HostValue> {
        let idx = handle as usize;
        self.values.get_mut(idx)
    }

    fn key_for(&self, handle: Handle) -> Option<ValueKey> {
        match self.get(handle)? {
            HostValue::Nil => Some(ValueKey::Nil),
            HostValue::Bool(b) => Some(ValueKey::Bool(*b)),
            HostValue::Int(n) => Some(ValueKey::Int(*n)),
            HostValue::Float(f) => Some(ValueKey::Float(f.to_bits())),
            HostValue::String(s) => Some(ValueKey::String(s.clone())),
            HostValue::List(_) => Some(ValueKey::List(handle)),
            HostValue::Dict(_) => Some(ValueKey::Dict(handle)),
            HostValue::NativeObject(_) => Some(ValueKey::NativeObject(handle)),
            HostValue::NativeCtor(_) => Some(ValueKey::NativeCtor(handle)),
        }
    }

    fn is_truthy(&self, handle: Handle) -> bool {
        match self.get(handle) {
            Some(HostValue::Nil) => false,
            Some(HostValue::Bool(b)) => *b,
            Some(HostValue::Int(n)) => *n != 0,
            Some(HostValue::Float(f)) => *f != 0.0,
            Some(HostValue::String(s)) => !s.is_empty(),
            Some(HostValue::List(items)) => !items.is_empty(),
            Some(HostValue::Dict(map)) => !map.is_empty(),
            Some(HostValue::NativeObject(_)) => true,
            Some(HostValue::NativeCtor(_)) => true,
            None => false,
        }
    }

    fn to_string(&self, handle: Handle) -> String {
        match self.get(handle) {
            Some(HostValue::Nil) => "naething".to_string(),
            Some(HostValue::Bool(true)) => "aye".to_string(),
            Some(HostValue::Bool(false)) => "nae".to_string(),
            Some(HostValue::Int(n)) => n.to_string(),
            Some(HostValue::Float(f)) => f.to_string(),
            Some(HostValue::String(s)) => s.clone(),
            Some(HostValue::List(items)) => {
                let mut parts = Vec::new();
                for item in items {
                    parts.push(self.to_string(*item));
                }
                format!("[{}]", parts.join(", "))
            }
            Some(HostValue::Dict(map)) => {
                let mut parts = Vec::new();
                for (key, value) in map {
                    let key_str = match key {
                        ValueKey::String(s) => s.clone(),
                        _ => format!("{:?}", key),
                    };
                    parts.push(format!("\"{}\": {}", key_str, self.to_string(*value)));
                }
                format!("{{{}}}", parts.join(", "))
            }
            Some(HostValue::NativeObject(obj)) => format!("<native {}>", obj.kind),
            Some(HostValue::NativeCtor(kind)) => format!("<native dae tri.{}>", kind),
            None => "<invalid>".to_string(),
        }
    }
}

#[derive(Debug)]
struct HostState {
    store: HostStore,
    tri_modules: HashSet<Handle>,
}

impl HostState {
    fn new() -> Self {
        HostState {
            store: HostStore::new(),
            tri_modules: HashSet::new(),
        }
    }
}

fn read_memory_string(caller: &Caller<'_, HostState>, mem: &Memory, ptr: i32, len: i32) -> String {
    let data = mem.data(caller);
    let start = ptr.max(0) as usize;
    let end = start.saturating_add(len.max(0) as usize);
    if end <= data.len() {
        String::from_utf8_lossy(&data[start..end]).to_string()
    } else {
        "".to_string()
    }
}

fn alloc_bool(store: &mut HostStore, value: bool) -> Handle {
    store.alloc(HostValue::Bool(value))
}

fn alloc_int(store: &mut HostStore, value: i64) -> Handle {
    store.alloc(HostValue::Int(value))
}

fn alloc_float(store: &mut HostStore, value: f64) -> Handle {
    store.alloc(HostValue::Float(value))
}

fn alloc_string(store: &mut HostStore, value: String) -> Handle {
    store.alloc(HostValue::String(value))
}

fn alloc_tri_ctor(store: &mut HostStore, kind: &str) -> Handle {
    store.alloc(HostValue::NativeCtor(kind.to_string()))
}

fn alloc_tri_object(store: &mut HostStore, kind: &str, args: &[Handle]) -> Handle {
    let mut obj = TriObject::new(kind);
    let type_handle = alloc_string(store, kind.to_string());
    obj.fields.insert("type".to_string(), type_handle);
    if tri_has_transform(kind) {
        let pos = alloc_tri_vec3(store, "Vec3", 0.0, 0.0, 0.0);
        let rot = alloc_tri_vec3(store, "Euler", 0.0, 0.0, 0.0);
        let scl = alloc_tri_vec3(store, "Vec3", 1.0, 1.0, 1.0);
        obj.fields.insert("position".to_string(), pos);
        obj.fields.insert("rotation".to_string(), rot);
        obj.fields.insert("scale".to_string(), scl);
        obj.fields.insert(
            "children".to_string(),
            store.alloc(HostValue::List(Vec::new())),
        );
        obj.fields.insert("parent".to_string(), 0);
    }
    apply_constructor_args(store, kind, &mut obj.fields, args);
    store.alloc(HostValue::NativeObject(obj))
}

fn alloc_tri_vec3(store: &mut HostStore, kind: &str, x: f64, y: f64, z: f64) -> Handle {
    let mut obj = TriObject::new(kind);
    let type_handle = alloc_string(store, kind.to_string());
    obj.fields.insert("type".to_string(), type_handle);
    obj.fields.insert("x".to_string(), alloc_float(store, x));
    obj.fields.insert("y".to_string(), alloc_float(store, y));
    obj.fields.insert("z".to_string(), alloc_float(store, z));
    store.alloc(HostValue::NativeObject(obj))
}

fn tri_has_transform(kind: &str) -> bool {
    matches!(
        kind,
        "Sicht"
            | "Thing3D"
            | "Clump"
            | "Mesch"
            | "Kamera"
            | "PerspectivKamera"
            | "OrthograffikKamera"
            | "Licht"
            | "AmbiantLicht"
            | "DireksionalLicht"
            | "PyntLicht"
    )
}

fn ensure_children_handle(store: &mut HostStore, obj: Handle) -> Option<Handle> {
    let existing = match store.get(obj) {
        Some(HostValue::NativeObject(native)) => native.fields.get("children").copied(),
        _ => None,
    };
    if let Some(handle) = existing {
        if matches!(store.get(handle), Some(HostValue::List(_))) {
            return Some(handle);
        }
    }
    let new_handle = store.alloc(HostValue::List(Vec::new()));
    if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
        native.fields.insert("children".to_string(), new_handle);
    }
    Some(new_handle)
}

fn tri_object_method(store: &mut HostStore, obj: Handle, method: &str, args: &[Handle]) -> Handle {
    match method {
        "cloan" | "clone" => {
            let (kind, fields) = match store.get(obj) {
                Some(HostValue::NativeObject(native)) => {
                    (native.kind.clone(), native.fields.clone())
                }
                _ => return 0,
            };
            let clone = TriObject { kind, fields };
            store.alloc(HostValue::NativeObject(clone))
        }
        "adde" | "add" => {
            if let Some(children) = ensure_children_handle(store, obj) {
                if let Some(HostValue::List(items)) = store.get_mut(children) {
                    items.extend_from_slice(args);
                }
            }
            0
        }
        "remuiv" | "remove" => {
            if let Some(children) = ensure_children_handle(store, obj) {
                if let Some(HostValue::List(items)) = store.get_mut(children) {
                    let to_remove: Vec<Handle> = args.to_vec();
                    items.retain(|item| !to_remove.contains(item));
                }
            }
            0
        }
        "dyspos" | "dispose" => 0,
        "luik_at" | "lookAt" => {
            if let Some(target) = args.get(0) {
                if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                    native.fields.insert("lookAtTarget".to_string(), *target);
                }
            }
            0
        }
        "set_sise" | "setSize" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(width) = args.get(0) {
                    native.fields.insert("width".to_string(), *width);
                }
                if let Some(height) = args.get(1) {
                    native.fields.insert("height".to_string(), *height);
                }
            }
            0
        }
        "set_pixel_ratio" | "setPixelRatio" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(ratio) = args.get(0) {
                    native.fields.insert("pixelRatio".to_string(), *ratio);
                }
            }
            0
        }
        "render" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(scene) = args.get(0) {
                    native.fields.insert("scene".to_string(), *scene);
                }
                if let Some(camera) = args.get(1) {
                    native.fields.insert("camera".to_string(), *camera);
                }
            }
            0
        }
        "loop" => {
            if let Some(HostValue::NativeObject(native)) = store.get_mut(obj) {
                if let Some(callback) = args.get(0) {
                    native.fields.insert("loopFn".to_string(), *callback);
                }
            }
            0
        }
        _ => 0,
    }
}

fn apply_constructor_args(
    _store: &mut HostStore,
    kind: &str,
    fields: &mut HashMap<String, Handle>,
    args: &[Handle],
) {
    fn arg_or_default<F>(args: &[Handle], index: usize, default: F) -> Handle
    where
        F: FnOnce() -> Handle,
    {
        args.get(index).copied().unwrap_or_else(default)
    }

    match kind {
        "Mesch" => {
            fields.insert("geometry".to_string(), arg_or_default(args, 0, || 0));
            fields.insert("material".to_string(), arg_or_default(args, 1, || 0));
        }
        "PerspectivKamera" => {
            fields.insert(
                "fov".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, 50)),
            );
            fields.insert(
                "aspect".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "near".to_string(),
                arg_or_default(args, 2, || alloc_float(_store, 0.1)),
            );
            fields.insert(
                "far".to_string(),
                arg_or_default(args, 3, || alloc_int(_store, 2000)),
            );
        }
        "OrthograffikKamera" => {
            fields.insert(
                "left".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, -1)),
            );
            fields.insert(
                "right".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "top".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 1)),
            );
            fields.insert(
                "bottom".to_string(),
                arg_or_default(args, 3, || alloc_int(_store, -1)),
            );
            fields.insert(
                "near".to_string(),
                arg_or_default(args, 4, || alloc_float(_store, 0.1)),
            );
            fields.insert(
                "far".to_string(),
                arg_or_default(args, 5, || alloc_int(_store, 2000)),
            );
        }
        "BoxGeometrie" => {
            fields.insert(
                "width".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, 1)),
            );
            fields.insert(
                "height".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "depth".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 1)),
            );
        }
        "SpherGeometrie" => {
            fields.insert(
                "radius".to_string(),
                arg_or_default(args, 0, || alloc_int(_store, 1)),
            );
            fields.insert(
                "widthSegments".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 8)),
            );
            fields.insert(
                "heightSegments".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 6)),
            );
        }
        "Maiterial" | "MeshBasicMaiterial" | "MeshStandardMaiterial" | "Renderar" => {
            fields.insert("opts".to_string(), arg_or_default(args, 0, || 0));
        }
        "Licht" | "AmbiantLicht" | "DireksionalLicht" => {
            fields.insert("color".to_string(), arg_or_default(args, 0, || 0));
            fields.insert(
                "intensity".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
        }
        "PyntLicht" => {
            fields.insert("color".to_string(), arg_or_default(args, 0, || 0));
            fields.insert(
                "intensity".to_string(),
                arg_or_default(args, 1, || alloc_int(_store, 1)),
            );
            fields.insert(
                "distance".to_string(),
                arg_or_default(args, 2, || alloc_int(_store, 0)),
            );
            fields.insert(
                "decay".to_string(),
                arg_or_default(args, 3, || alloc_int(_store, 2)),
            );
        }
        "Colour" => {
            fields.insert("value".to_string(), arg_or_default(args, 0, || 0));
        }
        _ => {}
    }
}

fn tri_constructor_kind(method: &str) -> Option<&'static str> {
    match method {
        "Sicht" => Some("Sicht"),
        "Thing3D" => Some("Thing3D"),
        "Clump" => Some("Clump"),
        "Mesch" => Some("Mesch"),
        "Kamera" => Some("Kamera"),
        "PerspectivKamera" => Some("PerspectivKamera"),
        "OrthograffikKamera" => Some("OrthograffikKamera"),
        "Geometrie" => Some("Geometrie"),
        "BoxGeometrie" => Some("BoxGeometrie"),
        "SpherGeometrie" => Some("SpherGeometrie"),
        "Maiterial" => Some("Maiterial"),
        "MeshBasicMaiterial" => Some("MeshBasicMaiterial"),
        "MeshStandardMaiterial" => Some("MeshStandardMaiterial"),
        "Licht" => Some("Licht"),
        "AmbiantLicht" => Some("AmbiantLicht"),
        "DireksionalLicht" => Some("DireksionalLicht"),
        "PyntLicht" => Some("PyntLicht"),
        "Textur" => Some("Textur"),
        "Renderar" => Some("Renderar"),
        "Colour" => Some("Colour"),
        _ => None,
    }
}

fn tri_handle_call(
    state: &mut HostState,
    obj: Handle,
    name: &str,
    args: &[Handle],
) -> Option<Handle> {
    if state.tri_modules.contains(&obj) {
        if let Some(kind) = tri_constructor_kind(name) {
            return Some(alloc_tri_object(&mut state.store, kind, args));
        }
        return Some(tri_object_method(&mut state.store, obj, name, args));
    }

    let ctor_kind = match state.store.get(obj) {
        Some(HostValue::NativeCtor(kind)) => Some(kind.clone()),
        _ => None,
    };
    if let Some(kind) = ctor_kind {
        if name == "call" || name == kind {
            return Some(alloc_tri_object(&mut state.store, &kind, args));
        }
        return Some(0);
    }

    if matches!(state.store.get(obj), Some(HostValue::NativeObject(_))) {
        return Some(tri_object_method(&mut state.store, obj, name, args));
    }

    None
}

fn add_binop(
    linker: &mut Linker<HostState>,
    name: &'static str,
    op: fn(f64, f64) -> f64,
) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            move |mut caller: Caller<'_, HostState>, a: i64, b: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                numeric_binop(store, a, b, op)
            },
        )
        .map_err(|e| e.to_string())
}

fn add_cmp(
    linker: &mut Linker<HostState>,
    name: &'static str,
    op: fn(&HostStore, i64, i64) -> bool,
) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            move |mut caller: Caller<'_, HostState>, a: i64, b: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_bool(store, op(store, a, b))
            },
        )
        .map_err(|e| e.to_string())
}

fn add_audio_stub0(linker: &mut Linker<HostState>, name: &str) -> Result<(), String> {
    linker
        .func_wrap("env", name, |_caller: Caller<'_, HostState>| -> i64 { 0 })
        .map_err(|e| e.to_string())
}

fn add_audio_stub1(linker: &mut Linker<HostState>, name: &str) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            |_caller: Caller<'_, HostState>, _a0: i64| -> i64 { 0 },
        )
        .map_err(|e| e.to_string())
}

fn add_audio_stub2(linker: &mut Linker<HostState>, name: &str) -> Result<(), String> {
    linker
        .func_wrap(
            "env",
            name,
            |_caller: Caller<'_, HostState>, _a0: i64, _a1: i64| -> i64 { 0 },
        )
        .map_err(|e| e.to_string())
}

fn numeric_binop(store: &mut HostStore, a: Handle, b: Handle, op: fn(f64, f64) -> f64) -> Handle {
    let left = store.get(a);
    let right = store.get(b);
    match (left, right) {
        (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => {
            let result = op(*x as f64, *y as f64);
            if result.fract() == 0.0 {
                alloc_int(store, result as i64)
            } else {
                alloc_float(store, result)
            }
        }
        (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => alloc_float(store, op(*x, *y)),
        (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => {
            alloc_float(store, op(*x as f64, *y))
        }
        (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => {
            alloc_float(store, op(*x, *y as f64))
        }
        _ => 0,
    }
}

fn eq_values(store: &HostStore, a: Handle, b: Handle) -> bool {
    match (store.get(a), store.get(b)) {
        (Some(HostValue::Nil), Some(HostValue::Nil)) => true,
        (Some(HostValue::Bool(x)), Some(HostValue::Bool(y))) => x == y,
        (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => x == y,
        (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => x == y,
        (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => (*x as f64) == *y,
        (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => *x == (*y as f64),
        (Some(HostValue::String(x)), Some(HostValue::String(y))) => x == y,
        _ => a == b,
    }
}

pub fn run_wasm_file(path: &Path) -> Result<(), String> {
    let engine = Engine::default();

    let module = match path.extension().and_then(|s| s.to_str()) {
        Some("wat") => {
            let bytes = wat::parse_file(path).map_err(|e| e.to_string())?;
            Module::new(&engine, bytes).map_err(|e| e.to_string())?
        }
        _ => Module::from_file(&engine, path).map_err(|e| e.to_string())?,
    };

    let mut store = Store::new(&engine, HostState::new());
    let memory = Memory::new(&mut store, MemoryType::new(1, None)).map_err(|e| e.to_string())?;

    let mut linker = Linker::new(&engine);
    linker
        .define("env", "memory", memory.clone())
        .map_err(|e| e.to_string())?;

    // Value constructors
    linker
        .func_wrap(
            "env",
            "__mdh_make_nil",
            |mut caller: Caller<'_, HostState>| -> i64 {
                let _ = caller.data_mut();
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_make_bool",
            |mut caller: Caller<'_, HostState>, value: i32| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_bool(store, value != 0)
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_make_int",
            |mut caller: Caller<'_, HostState>, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_int(store, value)
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_make_float",
            |mut caller: Caller<'_, HostState>, value: f64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_float(store, value)
            },
        )
        .map_err(|e| e.to_string())?;

    let mem_for_string = memory.clone();
    linker
        .func_wrap(
            "env",
            "__mdh_make_string",
            move |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i64 {
                let store = &mut caller.data_mut().store;
                let s = read_memory_string(&caller, &mem_for_string, ptr, len);
                alloc_string(store, s)
            },
        )
        .map_err(|e| e.to_string())?;

    // Truthiness
    linker
        .func_wrap(
            "env",
            "__mdh_truthy",
            |mut caller: Caller<'_, HostState>, value: i64| -> i32 {
                let store = &mut caller.data_mut().store;
                if store.is_truthy(value) {
                    1
                } else {
                    0
                }
            },
        )
        .map_err(|e| e.to_string())?;

    // Arithmetic and comparisons
    linker
        .func_wrap(
            "env",
            "__mdh_add",
            |mut caller: Caller<'_, HostState>, a: i64, b: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                match (store.get(a), store.get(b)) {
                    (Some(HostValue::String(sa)), _) => {
                        let out = format!("{}{}", sa, store.to_string(b));
                        alloc_string(store, out)
                    }
                    (_, Some(HostValue::String(sb))) => {
                        let out = format!("{}{}", store.to_string(a), sb);
                        alloc_string(store, out)
                    }
                    _ => numeric_binop(store, a, b, |x, y| x + y),
                }
            },
        )
        .map_err(|e| e.to_string())?;

    add_binop(&mut linker, "__mdh_sub", |x, y| x - y)?;
    add_binop(&mut linker, "__mdh_mul", |x, y| x * y)?;
    add_binop(&mut linker, "__mdh_div", |x, y| x / y)?;
    add_binop(&mut linker, "__mdh_mod", |x, y| x % y)?;

    add_cmp(&mut linker, "__mdh_eq", |store, a, b| {
        eq_values(store, a, b)
    })?;
    add_cmp(&mut linker, "__mdh_ne", |store, a, b| {
        !eq_values(store, a, b)
    })?;
    add_cmp(&mut linker, "__mdh_lt", |store, a, b| {
        match (store.get(a), store.get(b)) {
            (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => x < y,
            (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => x < y,
            (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => (*x as f64) < *y,
            (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => *x < (*y as f64),
            (Some(HostValue::String(x)), Some(HostValue::String(y))) => x < y,
            _ => false,
        }
    })?;
    add_cmp(&mut linker, "__mdh_le", |store, a, b| {
        match (store.get(a), store.get(b)) {
            (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => x <= y,
            (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => x <= y,
            (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => (*x as f64) <= *y,
            (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => *x <= (*y as f64),
            (Some(HostValue::String(x)), Some(HostValue::String(y))) => x <= y,
            _ => false,
        }
    })?;
    add_cmp(&mut linker, "__mdh_gt", |store, a, b| {
        match (store.get(a), store.get(b)) {
            (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => x > y,
            (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => x > y,
            (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => (*x as f64) > *y,
            (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => *x > (*y as f64),
            (Some(HostValue::String(x)), Some(HostValue::String(y))) => x > y,
            _ => false,
        }
    })?;
    add_cmp(&mut linker, "__mdh_ge", |store, a, b| {
        match (store.get(a), store.get(b)) {
            (Some(HostValue::Int(x)), Some(HostValue::Int(y))) => x >= y,
            (Some(HostValue::Float(x)), Some(HostValue::Float(y))) => x >= y,
            (Some(HostValue::Int(x)), Some(HostValue::Float(y))) => (*x as f64) >= *y,
            (Some(HostValue::Float(x)), Some(HostValue::Int(y))) => *x >= (*y as f64),
            (Some(HostValue::String(x)), Some(HostValue::String(y))) => x >= y,
            _ => false,
        }
    })?;

    linker
        .func_wrap(
            "env",
            "__mdh_neg",
            |mut caller: Caller<'_, HostState>, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                match store.get(value) {
                    Some(HostValue::Int(n)) => alloc_int(store, -n),
                    Some(HostValue::Float(f)) => alloc_float(store, -f),
                    _ => 0,
                }
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_not",
            |mut caller: Caller<'_, HostState>, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                alloc_bool(store, !store.is_truthy(value))
            },
        )
        .map_err(|e| e.to_string())?;

    // Printing
    linker
        .func_wrap(
            "env",
            "__mdh_blether",
            |mut caller: Caller<'_, HostState>, value: i64| {
                let store = &mut caller.data_mut().store;
                println!("{}", store.to_string(value));
            },
        )
        .map_err(|e| e.to_string())?;

    // List helpers
    linker
        .func_wrap(
            "env",
            "__mdh_make_list",
            |mut caller: Caller<'_, HostState>, _cap: i32| -> i64 {
                let store = &mut caller.data_mut().store;
                store.alloc(HostValue::List(Vec::new()))
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_list_push",
            |mut caller: Caller<'_, HostState>, list: i64, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                if let Some(HostValue::List(items)) = store.get_mut(list) {
                    items.push(value);
                }
                list
            },
        )
        .map_err(|e| e.to_string())?;

    // Dict helpers
    linker
        .func_wrap(
            "env",
            "__mdh_make_dict",
            |mut caller: Caller<'_, HostState>| -> i64 {
                let store = &mut caller.data_mut().store;
                store.alloc(HostValue::Dict(HashMap::new()))
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_tri_module",
            |mut caller: Caller<'_, HostState>| -> i64 {
                let state = caller.data_mut();
                let module_handle = state.store.alloc(HostValue::Dict(HashMap::new()));
                if let Some(HostValue::Dict(map)) = state.store.get_mut(module_handle) {
                    let deg = alloc_float(&mut state.store, std::f64::consts::PI / 180.0);
                    let rad = alloc_float(&mut state.store, 180.0 / std::f64::consts::PI);
                    map.insert(ValueKey::String("DEG_TO_RAD".to_string()), deg);
                    map.insert(ValueKey::String("RAD_TO_DEG".to_string()), rad);
                }
                state.tri_modules.insert(module_handle);
                module_handle
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_dict_set",
            |mut caller: Caller<'_, HostState>, dict: i64, key: i64, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                let key_value = store.key_for(key);
                if let Some(HostValue::Dict(map)) = store.get_mut(dict) {
                    if let Some(k) = key_value {
                        map.insert(k, value);
                    }
                }
                dict
            },
        )
        .map_err(|e| e.to_string())?;

    // Property access
    linker
        .func_wrap(
            "env",
            "__mdh_prop_get",
            |mut caller: Caller<'_, HostState>, obj: i64, prop: i64| -> i64 {
                let state = caller.data_mut();
                if state.tri_modules.contains(&obj) {
                    if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
                        if let Some(kind) = tri_constructor_kind(&key) {
                            return alloc_tri_ctor(&mut state.store, kind);
                        }
                    }
                }
                match state.store.get(obj) {
                    Some(HostValue::Dict(map)) => {
                        if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
                            return map.get(&ValueKey::String(key)).copied().unwrap_or(0);
                        }
                        0
                    }
                    Some(HostValue::NativeObject(native)) => {
                        if let Some(ValueKey::String(key)) = state.store.key_for(prop) {
                            return native.fields.get(&key).copied().unwrap_or(0);
                        }
                        0
                    }
                    _ => 0,
                }
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_prop_set",
            |mut caller: Caller<'_, HostState>, obj: i64, prop: i64, value: i64| -> i64 {
                let store = &mut caller.data_mut().store;
                let prop_key = store.key_for(prop);
                match store.get_mut(obj) {
                    Some(HostValue::Dict(map)) => {
                        if let Some(ValueKey::String(key)) = prop_key {
                            map.insert(ValueKey::String(key), value);
                        }
                    }
                    Some(HostValue::NativeObject(native)) => {
                        if let Some(ValueKey::String(key)) = prop_key {
                            native.fields.insert(key, value);
                        }
                    }
                    _ => {}
                }
                value
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call0",
            move |mut caller: Caller<'_, HostState>, obj: i64, method: i64| -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                if let Some(result) = tri_handle_call(state, obj, &name, &[]) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call1",
            move |mut caller: Caller<'_, HostState>, obj: i64, method: i64, a0: i64| -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call2",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call3",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call4",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call5",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call6",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64,
                  a5: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4, a5];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call7",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64,
                  a5: i64,
                  a6: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4, a5, a6];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .func_wrap(
            "env",
            "__mdh_method_call8",
            move |mut caller: Caller<'_, HostState>,
                  obj: i64,
                  method: i64,
                  a0: i64,
                  a1: i64,
                  a2: i64,
                  a3: i64,
                  a4: i64,
                  a5: i64,
                  a6: i64,
                  a7: i64|
                  -> i64 {
                let state = caller.data_mut();
                let name = match state.store.get(method) {
                    Some(HostValue::String(name)) => name.clone(),
                    _ => return 0,
                };
                let args = [a0, a1, a2, a3, a4, a5, a6, a7];
                if let Some(result) = tri_handle_call(state, obj, &name, &args) {
                    return result;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;

    for name in [
        "soond_stairt",
        "soond_steek",
        "soond_hou_luid",
        "soond_haud_gang",
    ] {
        add_audio_stub0(&mut linker, name)?;
    }

    for name in [
        "soond_wheesht",
        "soond_luid",
        "soond_lade",
        "soond_spiel",
        "soond_haud",
        "soond_gae_on",
        "soond_stap",
        "soond_unlade",
        "soond_is_spielin",
        "soond_ready",
        "muisic_lade",
        "muisic_spiel",
        "muisic_haud",
        "muisic_gae_on",
        "muisic_stap",
        "muisic_unlade",
        "muisic_is_spielin",
        "muisic_hou_lang",
        "muisic_whaur",
        "midi_spiel",
        "midi_haud",
        "midi_gae_on",
        "midi_stap",
        "midi_unlade",
        "midi_is_spielin",
        "midi_hou_lang",
        "midi_whaur",
    ] {
        add_audio_stub1(&mut linker, name)?;
    }

    for name in [
        "soond_pit_luid",
        "soond_pit_pan",
        "soond_pit_tune",
        "soond_pit_rin_roond",
        "muisic_loup",
        "muisic_pit_luid",
        "muisic_pit_pan",
        "muisic_pit_tune",
        "muisic_pit_rin_roond",
        "midi_lade",
        "midi_loup",
        "midi_pit_luid",
        "midi_pit_pan",
        "midi_pit_tune",
    ] {
        add_audio_stub2(&mut linker, name)?;
    }

    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| e.to_string())?;

    if let Ok(func) = instance.get_typed_func::<(), i64>(&mut store, "main") {
        let _ = func.call(&mut store, ()).map_err(|e| e.to_string())?;
        return Ok(());
    }

    if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
        func.call(&mut store, ()).map_err(|e| e.to_string())?;
        return Ok(());
    }

    Err("WASM module lacks exported 'main' or '_start'".to_string())
}
