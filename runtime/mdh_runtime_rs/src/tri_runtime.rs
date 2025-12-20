use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use crate::{
    mdh_float_value, mdh_make_string_from_rust, mdh_value_to_string, MdhList, MdhValue,
    MDH_TAG_BOOL, MDH_TAG_CLOSURE, MDH_TAG_DICT, MDH_TAG_FLOAT, MDH_TAG_FUNCTION, MDH_TAG_INT,
    MDH_TAG_LIST, MDH_TAG_NATIVE, MDH_TAG_NIL, MDH_TAG_STRING,
};
use crate::tri_engine::{with_engine, LoopCallback, MeshData, RenderItem};
use glam::{EulerRot, Mat4, Quat, Vec3};

#[repr(C)]
pub(crate) struct MdhNativeObject {
    kind: i32,
    type_name: *const c_char,
    ctor_kind: *const c_char,
    fields: MdhValue,
}

const MDH_NATIVE_TRI_MODULE: i32 = 1;
const MDH_NATIVE_TRI_OBJECT: i32 = 2;
const MDH_NATIVE_TRI_CTOR: i32 = 3;

struct TriObject {
    kind: String,
    fields: HashMap<String, MdhValue>,
    renderer_handle: Option<usize>,
}

#[derive(Clone)]
struct TriObjectSnapshot {
    kind: String,
    fields: HashMap<String, MdhValue>,
}

struct LightInfo {
    ambient: Vec3,
    directional_dir: Option<Vec3>,
    directional_color: Vec3,
    point_pos: Option<Vec3>,
    point_color: Vec3,
    point_distance: f32,
    point_decay: f32,
}

struct TriState {
    objects: HashMap<usize, TriObject>,
}

impl TriState {
    fn new() -> Self {
        TriState {
            objects: HashMap::new(),
        }
    }
}

static TRI_STATE: OnceLock<Mutex<TriState>> = OnceLock::new();
static TRI_MODULE: OnceLock<MdhValue> = OnceLock::new();

fn tri_state() -> &'static Mutex<TriState> {
    TRI_STATE.get_or_init(|| Mutex::new(TriState::new()))
}

fn leak_cstring(value: &str) -> *const c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("tri").expect("CString literal"))
        .into_raw()
}

unsafe fn mdh_make_native(ptr: *mut MdhNativeObject) -> MdhValue {
    MdhValue {
        tag: MDH_TAG_NATIVE,
        data: ptr as i64,
    }
}

type MdhFn0 = unsafe extern "C" fn() -> MdhValue;
type MdhFn1 = unsafe extern "C" fn(MdhValue) -> MdhValue;
type MdhFn2 = unsafe extern "C" fn(MdhValue, MdhValue) -> MdhValue;
type MdhFn3 = unsafe extern "C" fn(MdhValue, MdhValue, MdhValue) -> MdhValue;
type MdhFn4 = unsafe extern "C" fn(MdhValue, MdhValue, MdhValue, MdhValue) -> MdhValue;
type MdhFn5 =
    unsafe extern "C" fn(MdhValue, MdhValue, MdhValue, MdhValue, MdhValue) -> MdhValue;
type MdhFn6 =
    unsafe extern "C" fn(MdhValue, MdhValue, MdhValue, MdhValue, MdhValue, MdhValue) -> MdhValue;

unsafe fn mdh_call_value(func_val: MdhValue, args: &[MdhValue]) -> MdhValue {
    let mut call_args = [__mdh_make_nil(); 6];
    let mut total_args = args.len();
    let mut fn_val = func_val;

    if func_val.tag == MDH_TAG_CLOSURE {
        let base = func_val.data as *const u8;
        if base.is_null() {
            __mdh_hurl(mdh_make_string_from_rust("Invalid closure"));
            return __mdh_make_nil();
        }
        let header = base as *const i64;
        let len = *header.add(1);
        if len <= 0 {
            __mdh_hurl(mdh_make_string_from_rust("Invalid closure"));
            return __mdh_make_nil();
        }
        let elems = base.add(16) as *const MdhValue;
        fn_val = *elems;
        let captures = len - 1;
        if captures > 3 {
            __mdh_hurl(mdh_make_string_from_rust(
                "Closure captures > 3 not supported in render loop",
            ));
            return __mdh_make_nil();
        }
        if (captures as usize) + args.len() > 6 {
            __mdh_hurl(mdh_make_string_from_rust(
                "Too many arguments for render loop callback",
            ));
            return __mdh_make_nil();
        }
        for i in 0..captures {
            call_args[i as usize] = *elems.add(1 + i as usize);
        }
        for (i, arg) in args.iter().enumerate() {
            call_args[captures as usize + i] = *arg;
        }
        total_args = captures as usize + args.len();
    } else if func_val.tag == MDH_TAG_FUNCTION {
        if args.len() > 6 {
            __mdh_hurl(mdh_make_string_from_rust(
                "Too many arguments for render loop callback",
            ));
            return __mdh_make_nil();
        }
        for (i, arg) in args.iter().enumerate() {
            call_args[i] = *arg;
        }
    } else {
        __mdh_hurl(mdh_make_string_from_rust(
            "Renderar.loop expects a function",
        ));
        return __mdh_make_nil();
    }

    let fn_ptr = fn_val.data as usize;
    match total_args {
        0 => std::mem::transmute::<usize, MdhFn0>(fn_ptr)(),
        1 => std::mem::transmute::<usize, MdhFn1>(fn_ptr)(call_args[0]),
        2 => std::mem::transmute::<usize, MdhFn2>(fn_ptr)(call_args[0], call_args[1]),
        3 => std::mem::transmute::<usize, MdhFn3>(fn_ptr)(
            call_args[0],
            call_args[1],
            call_args[2],
        ),
        4 => std::mem::transmute::<usize, MdhFn4>(fn_ptr)(
            call_args[0],
            call_args[1],
            call_args[2],
            call_args[3],
        ),
        5 => std::mem::transmute::<usize, MdhFn5>(fn_ptr)(
            call_args[0],
            call_args[1],
            call_args[2],
            call_args[3],
            call_args[4],
        ),
        6 => std::mem::transmute::<usize, MdhFn6>(fn_ptr)(
            call_args[0],
            call_args[1],
            call_args[2],
            call_args[3],
            call_args[4],
            call_args[5],
        ),
        _ => __mdh_make_nil(),
    }
}

unsafe fn native_ctor_kind(ptr: *mut MdhNativeObject) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let ctor_ptr = (*ptr).ctor_kind;
    if ctor_ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ctor_ptr).to_string_lossy().into_owned()
}

unsafe fn new_native_object(kind: i32, type_name: &str, ctor_kind: Option<&str>) -> *mut MdhNativeObject {
    let obj = Box::new(MdhNativeObject {
        kind,
        type_name: leak_cstring(type_name),
        ctor_kind: ctor_kind.map(leak_cstring).unwrap_or(std::ptr::null()),
        fields: __mdh_empty_dict(),
    });
    Box::into_raw(obj)
}

unsafe fn register_object(ptr: *mut MdhNativeObject, obj: TriObject) {
    let key = ptr as usize;
    if let Ok(mut state) = tri_state().lock() {
        state.objects.insert(key, obj);
    }
}

unsafe fn tri_snapshot_from_value(value: MdhValue) -> Option<TriObjectSnapshot> {
    if value.tag != MDH_TAG_NATIVE || value.data == 0 {
        return None;
    }
    let key = value.data as usize;
    let state = tri_state().lock().ok()?;
    let obj = state.objects.get(&key)?;
    Some(TriObjectSnapshot {
        kind: obj.kind.clone(),
        fields: obj.fields.clone(),
    })
}

unsafe fn with_object<F, R>(ptr: *mut MdhNativeObject, f: F) -> Option<R>
where
    F: FnOnce(usize, &mut TriObject) -> R,
{
    let key = ptr as usize;
    let mut state = tri_state().lock().ok()?;
    let obj = state.objects.get_mut(&key)?;
    Some(f(key, obj))
}

fn tri_constructor_kind(name: &str) -> Option<&'static str> {
    match name {
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

unsafe fn tri_make_vec3(kind: &str, x: f64, y: f64, z: f64) -> MdhValue {
    let mut fields = HashMap::new();
    fields.insert("type".to_string(), mdh_make_string_from_rust(kind));
    fields.insert("x".to_string(), __mdh_make_float(x));
    fields.insert("y".to_string(), __mdh_make_float(y));
    fields.insert("z".to_string(), __mdh_make_float(z));
    let tri = TriObject {
        kind: kind.to_string(),
        fields,
        renderer_handle: None,
    };
    let native_ptr = new_native_object(MDH_NATIVE_TRI_OBJECT, kind, None);
    register_object(native_ptr, tri);
    mdh_make_native(native_ptr)
}

unsafe fn apply_constructor_defaults(kind: &str, fields: &mut HashMap<String, MdhValue>, args: &[MdhValue]) {
    let arg_or_default = |index: usize, default: MdhValue| -> MdhValue {
        args.get(index).copied().unwrap_or(default)
    };

    match kind {
        "Mesch" => {
            fields.insert("geometry".to_string(), arg_or_default(0, __mdh_make_nil()));
            fields.insert("material".to_string(), arg_or_default(1, __mdh_make_nil()));
        }
        "PerspectivKamera" => {
            fields.insert("fov".to_string(), arg_or_default(0, __mdh_make_int(50)));
            fields.insert("aspect".to_string(), arg_or_default(1, __mdh_make_int(1)));
            fields.insert("near".to_string(), arg_or_default(2, __mdh_make_float(0.1)));
            fields.insert("far".to_string(), arg_or_default(3, __mdh_make_int(2000)));
        }
        "OrthograffikKamera" => {
            fields.insert("left".to_string(), arg_or_default(0, __mdh_make_int(-1)));
            fields.insert("right".to_string(), arg_or_default(1, __mdh_make_int(1)));
            fields.insert("top".to_string(), arg_or_default(2, __mdh_make_int(1)));
            fields.insert("bottom".to_string(), arg_or_default(3, __mdh_make_int(-1)));
            fields.insert("near".to_string(), arg_or_default(4, __mdh_make_float(0.1)));
            fields.insert("far".to_string(), arg_or_default(5, __mdh_make_int(2000)));
        }
        "BoxGeometrie" => {
            fields.insert("width".to_string(), arg_or_default(0, __mdh_make_int(1)));
            fields.insert("height".to_string(), arg_or_default(1, __mdh_make_int(1)));
            fields.insert("depth".to_string(), arg_or_default(2, __mdh_make_int(1)));
        }
        "SpherGeometrie" => {
            fields.insert("radius".to_string(), arg_or_default(0, __mdh_make_int(1)));
            fields.insert("widthSegments".to_string(), arg_or_default(1, __mdh_make_int(8)));
            fields.insert("heightSegments".to_string(), arg_or_default(2, __mdh_make_int(6)));
        }
        "Maiterial" | "MeshBasicMaiterial" | "MeshStandardMaiterial" | "Renderar" => {
            fields.insert("opts".to_string(), arg_or_default(0, __mdh_make_nil()));
        }
        "Licht" | "AmbiantLicht" | "DireksionalLicht" => {
            fields.insert("color".to_string(), arg_or_default(0, __mdh_make_nil()));
            fields.insert("intensity".to_string(), arg_or_default(1, __mdh_make_int(1)));
        }
        "PyntLicht" => {
            fields.insert("color".to_string(), arg_or_default(0, __mdh_make_nil()));
            fields.insert("intensity".to_string(), arg_or_default(1, __mdh_make_int(1)));
            fields.insert("distance".to_string(), arg_or_default(2, __mdh_make_int(0)));
            fields.insert("decay".to_string(), arg_or_default(3, __mdh_make_int(2)));
        }
        "Colour" => {
            fields.insert("value".to_string(), arg_or_default(0, __mdh_make_nil()));
        }
        _ => {}
    }
}

unsafe fn tri_make_object(kind: &str, args: &[MdhValue]) -> MdhValue {
    let mut fields = HashMap::new();
    fields.insert("type".to_string(), mdh_make_string_from_rust(kind));

    if tri_has_transform(kind) {
        fields.insert("position".to_string(), tri_make_vec3("Vec3", 0.0, 0.0, 0.0));
        fields.insert("rotation".to_string(), tri_make_vec3("Euler", 0.0, 0.0, 0.0));
        fields.insert("scale".to_string(), tri_make_vec3("Vec3", 1.0, 1.0, 1.0));
        fields.insert("children".to_string(), __mdh_make_list(0));
        fields.insert("parent".to_string(), __mdh_make_nil());
    }

    apply_constructor_defaults(kind, &mut fields, args);

    let (opt_width, opt_height, opt_ratio) = if kind == "Renderar" {
        tri_renderar_opts(&mut fields)
    } else {
        (None, None, None)
    };

    let renderer_handle = if kind == "Renderar" {
        with_engine(|engine| engine.create_renderer().ok())
    } else {
        None
    };
    if let Some(handle) = renderer_handle {
        if let (Some(width), Some(height)) = (opt_width, opt_height) {
            with_engine(|engine| engine.set_size(handle, width, height));
        }
        if let Some(ratio) = opt_ratio {
            with_engine(|engine| engine.set_pixel_ratio(handle, ratio));
        }
    }

    let tri = TriObject {
        kind: kind.to_string(),
        fields,
        renderer_handle,
    };
    let native_ptr = new_native_object(MDH_NATIVE_TRI_OBJECT, kind, None);
    register_object(native_ptr, tri);
    mdh_make_native(native_ptr)
}

unsafe fn tri_make_ctor(kind: &str) -> MdhValue {
    let native_ptr = new_native_object(MDH_NATIVE_TRI_CTOR, "native function", Some(kind));
    mdh_make_native(native_ptr)
}

unsafe fn tri_method_add(obj: &mut TriObject, args: &[MdhValue]) {
    let children = obj.fields.get("children").copied();
    let list_val = match children {
        Some(val) if val.tag == MDH_TAG_LIST => val,
        _ => {
            let list = __mdh_make_list(0);
            obj.fields.insert("children".to_string(), list);
            list
        }
    };
    for arg in args {
        __mdh_list_push(list_val, *arg);
    }
}

unsafe fn tri_method_remove(obj: &mut TriObject, args: &[MdhValue]) {
    let list_val = match obj.fields.get("children").copied() {
        Some(val) if val.tag == MDH_TAG_LIST => val,
        _ => return,
    };
    let list_ptr = list_val.data as *mut MdhList;
    if list_ptr.is_null() {
        return;
    }
    let len = (*list_ptr).length.max(0) as usize;
    let items = (*list_ptr).items;
    if items.is_null() || len == 0 {
        return;
    }
    let mut write = 0usize;
    for i in 0..len {
        let item = *items.add(i);
        let mut remove = false;
        for arg in args {
            if __mdh_eq(item, *arg) {
                remove = true;
                break;
            }
        }
        if !remove {
            *items.add(write) = item;
            write += 1;
        }
    }
    (*list_ptr).length = write as i64;
}

unsafe fn tri_method_clone(obj: &TriObject) -> MdhValue {
    let mut fields = HashMap::new();
    for (k, v) in &obj.fields {
        fields.insert(k.clone(), *v);
    }
    let clone = TriObject {
        kind: obj.kind.clone(),
        fields,
        renderer_handle: obj.renderer_handle,
    };
    let native_ptr = new_native_object(MDH_NATIVE_TRI_OBJECT, &obj.kind, None);
    register_object(native_ptr, clone);
    mdh_make_native(native_ptr)
}

unsafe fn tri_method_set_field(obj: &mut TriObject, key: &str, value: MdhValue) {
    obj.fields.insert(key.to_string(), value);
}

unsafe fn mdh_number(value: MdhValue) -> Option<f64> {
    match value.tag {
        MDH_TAG_INT => Some(value.data as f64),
        MDH_TAG_FLOAT => Some(mdh_float_value(value)),
        _ => None,
    }
}

unsafe fn mdh_bool(value: MdhValue) -> Option<bool> {
    match value.tag {
        MDH_TAG_BOOL => Some(value.data != 0),
        _ => None,
    }
}

unsafe fn tri_renderar_opts(
    fields: &mut HashMap<String, MdhValue>,
) -> (Option<u32>, Option<u32>, Option<f32>) {
    let opts_val = match fields.get("opts").copied() {
        Some(val) if val.tag == MDH_TAG_DICT => val,
        _ => return (None, None, None),
    };
    let width_val =
        __mdh_dict_get_default(opts_val, mdh_make_string_from_rust("width"), __mdh_make_nil());
    let height_val =
        __mdh_dict_get_default(opts_val, mdh_make_string_from_rust("height"), __mdh_make_nil());
    let ratio_val = __mdh_dict_get_default(
        opts_val,
        mdh_make_string_from_rust("pixelRatio"),
        __mdh_make_nil(),
    );
    let wire_val =
        __mdh_dict_get_default(opts_val, mdh_make_string_from_rust("wireframe"), __mdh_make_nil());

    if width_val.tag != MDH_TAG_NIL {
        fields.insert("width".to_string(), width_val);
    }
    if height_val.tag != MDH_TAG_NIL {
        fields.insert("height".to_string(), height_val);
    }
    if ratio_val.tag != MDH_TAG_NIL {
        fields.insert("pixelRatio".to_string(), ratio_val);
    }
    if wire_val.tag != MDH_TAG_NIL {
        fields.insert("wireframe".to_string(), wire_val);
    }

    let width = mdh_number(width_val).map(|v| v.max(1.0) as u32);
    let height = mdh_number(height_val).map(|v| v.max(1.0) as u32);
    let ratio = mdh_number(ratio_val).map(|v| v as f32);
    (width, height, ratio)
}

unsafe fn tri_vec3_from_value(value: MdhValue, default: Vec3) -> Vec3 {
    let snapshot = match tri_snapshot_from_value(value) {
        Some(snapshot) => snapshot,
        None => return default,
    };
    let x = snapshot
        .fields
        .get("x")
        .and_then(|v| mdh_number(*v))
        .unwrap_or(default.x as f64) as f32;
    let y = snapshot
        .fields
        .get("y")
        .and_then(|v| mdh_number(*v))
        .unwrap_or(default.y as f64) as f32;
    let z = snapshot
        .fields
        .get("z")
        .and_then(|v| mdh_number(*v))
        .unwrap_or(default.z as f64) as f32;
    Vec3::new(x, y, z)
}

unsafe fn tri_model_matrix(snapshot: &TriObjectSnapshot) -> Mat4 {
    let position = snapshot
        .fields
        .get("position")
        .map(|v| tri_vec3_from_value(*v, Vec3::ZERO))
        .unwrap_or(Vec3::ZERO);
    let rotation = snapshot
        .fields
        .get("rotation")
        .map(|v| tri_vec3_from_value(*v, Vec3::ZERO))
        .unwrap_or(Vec3::ZERO);
    let scale = snapshot
        .fields
        .get("scale")
        .map(|v| tri_vec3_from_value(*v, Vec3::ONE))
        .unwrap_or(Vec3::ONE);
    let quat = Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
    Mat4::from_scale_rotation_translation(scale, quat, position)
}

unsafe fn tri_mesh_from_geometry(value: MdhValue) -> Option<(MeshData, Option<usize>)> {
    let snapshot = tri_snapshot_from_value(value)?;
    let mesh_key = if value.tag == MDH_TAG_NATIVE && value.data != 0 {
        Some(value.data as usize)
    } else {
        None
    };
    match snapshot.kind.as_str() {
        "BoxGeometrie" => {
            let width = snapshot
                .fields
                .get("width")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            let height = snapshot
                .fields
                .get("height")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            let depth = snapshot
                .fields
                .get("depth")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            let x = width / 2.0;
            let y = height / 2.0;
            let z = depth / 2.0;
            let vertices = vec![
                // +X
                [x, -y, -z],
                [x, y, -z],
                [x, y, z],
                [x, -y, z],
                // -X
                [-x, -y, z],
                [-x, y, z],
                [-x, y, -z],
                [-x, -y, -z],
                // +Y
                [-x, y, -z],
                [-x, y, z],
                [x, y, z],
                [x, y, -z],
                // -Y
                [-x, -y, z],
                [-x, -y, -z],
                [x, -y, -z],
                [x, -y, z],
                // +Z
                [-x, -y, z],
                [x, -y, z],
                [x, y, z],
                [-x, y, z],
                // -Z
                [x, -y, -z],
                [-x, -y, -z],
                [-x, y, -z],
                [x, y, -z],
            ];
            let normals = vec![
                // +X
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                // -X
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                // +Y
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                // -Y
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                // +Z
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                // -Z
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
            ];
            let mut indices = Vec::new();
            for face in 0..6 {
                let base = face * 4;
                indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);
            }
            Some((
                MeshData {
                    vertices,
                    normals,
                    indices,
                },
                mesh_key,
            ))
        }
        "SpherGeometrie" => {
            let radius = snapshot
                .fields
                .get("radius")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            let width_segments = snapshot
                .fields
                .get("widthSegments")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(8.0) as u32;
            let height_segments = snapshot
                .fields
                .get("heightSegments")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(6.0) as u32;
            let width_segments = width_segments.max(3);
            let height_segments = height_segments.max(2);
            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            let mut indices = Vec::new();

            for y in 0..=height_segments {
                let v = y as f32 / height_segments as f32;
                let theta = v * std::f32::consts::PI;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();
                for x in 0..=width_segments {
                    let u = x as f32 / width_segments as f32;
                    let phi = u * std::f32::consts::PI * 2.0;
                    let sin_phi = phi.sin();
                    let cos_phi = phi.cos();
                    let px = radius * cos_phi * sin_theta;
                    let py = radius * cos_theta;
                    let pz = radius * sin_phi * sin_theta;
                    vertices.push([px, py, pz]);
                    let len = (px * px + py * py + pz * pz).sqrt().max(1e-6);
                    normals.push([px / len, py / len, pz / len]);
                }
            }

            let row = width_segments + 1;
            for y in 0..height_segments {
                for x in 0..width_segments {
                    let a = y * row + x;
                    let b = a + row;
                    let c = b + 1;
                    let d = a + 1;
                    indices.push(a);
                    indices.push(b);
                    indices.push(d);
                    indices.push(b);
                    indices.push(c);
                    indices.push(d);
                }
            }

            Some((
                MeshData {
                    vertices,
                    normals,
                    indices,
                },
                mesh_key,
            ))
        }
        _ => None,
    }
}

unsafe fn tri_parse_color(value: MdhValue) -> Option<[f32; 4]> {
    match value.tag {
        MDH_TAG_STRING => {
            let s = mdh_value_to_string(value);
            parse_hex_color(&s)
        }
        MDH_TAG_INT => {
            let v = value.data as u32;
            Some(color_from_hex(v))
        }
        MDH_TAG_FLOAT => {
            let f = mdh_float_value(value) as f32;
            if f >= 0.0 && f <= 1.0 {
                Some([f, f, f, 1.0])
            } else {
                Some(color_from_hex(f as u32))
            }
        }
        MDH_TAG_NATIVE => {
            if let Some(snapshot) = tri_snapshot_from_value(value) {
                if snapshot.kind == "Colour" {
                    if let Some(val) = snapshot.fields.get("value") {
                        return tri_parse_color(*val);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_hex_color(input: &str) -> Option<[f32; 4]> {
    let s = input.trim();
    let hex = if let Some(rest) = s.strip_prefix('#') {
        rest
    } else if let Some(rest) = s.strip_prefix("0x") {
        rest
    } else {
        return None;
    };
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(if hex.len() == 8 {
        let r = ((value >> 24) & 0xff) as u8;
        let g = ((value >> 16) & 0xff) as u8;
        let b = ((value >> 8) & 0xff) as u8;
        let a = (value & 0xff) as u8;
        [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ]
    } else {
        color_from_hex(value)
    })
}

fn color_from_hex(value: u32) -> [f32; 4] {
    let r = ((value >> 16) & 0xff) as u8;
    let g = ((value >> 8) & 0xff) as u8;
    let b = (value & 0xff) as u8;
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

fn clamp01(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

unsafe fn tri_material_info(material_val: MdhValue) -> ([f32; 4], bool, f32, f32) {
    let default = [0.7, 0.7, 0.75, 1.0];
    let snapshot = match tri_snapshot_from_value(material_val) {
        Some(snapshot) => snapshot,
        None => return (default, false, 0.0, 0.5),
    };
    let unlit = snapshot.kind == "MeshBasicMaiterial";
    let opts_val = match snapshot.fields.get("opts") {
        Some(val) => *val,
        None => return (default, unlit, 0.0, 0.5),
    };
    if opts_val.tag != MDH_TAG_DICT {
        return (default, unlit, 0.0, 0.5);
    }
    let color_val =
        __mdh_dict_get_default(opts_val, mdh_make_string_from_rust("color"), __mdh_make_nil());
    let metalness_val = __mdh_dict_get_default(
        opts_val,
        mdh_make_string_from_rust("metalness"),
        __mdh_make_nil(),
    );
    let roughness_val = __mdh_dict_get_default(
        opts_val,
        mdh_make_string_from_rust("roughness"),
        __mdh_make_nil(),
    );

    let metalness = if unlit {
        0.0
    } else {
        clamp01(
            mdh_number(metalness_val)
                .unwrap_or(0.0)
                .min(1.0)
                .max(0.0) as f32,
        )
    };
    let roughness = if unlit {
        0.0
    } else {
        clamp01(
            mdh_number(roughness_val)
                .unwrap_or(0.5)
                .min(1.0)
                .max(0.0) as f32,
        )
    };

    (
        tri_parse_color(color_val).unwrap_or(default),
        unlit,
        metalness,
        roughness,
    )
}

unsafe fn tri_light_info(scene_val: Option<MdhValue>) -> LightInfo {
    let mut info = LightInfo {
        ambient: Vec3::ZERO,
        directional_dir: None,
        directional_color: Vec3::ZERO,
        point_pos: None,
        point_color: Vec3::ZERO,
        point_distance: 0.0,
        point_decay: 0.0,
    };
    if let Some(scene) = scene_val {
        tri_collect_lights(scene, Mat4::IDENTITY, &mut info);
    }
    info
}

unsafe fn tri_collect_lights(value: MdhValue, parent: Mat4, info: &mut LightInfo) {
    let snapshot = match tri_snapshot_from_value(value) {
        Some(snapshot) => snapshot,
        None => return,
    };
    let mut world = parent;
    if tri_has_transform(&snapshot.kind) {
        world = parent * tri_model_matrix(&snapshot);
    }

    match snapshot.kind.as_str() {
        "AmbiantLicht" => {
            let color = snapshot
                .fields
                .get("color")
                .copied()
                .and_then(|val| tri_parse_color(val))
                .unwrap_or([1.0, 1.0, 1.0, 1.0]);
            let intensity = snapshot
                .fields
                .get("intensity")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            info.ambient += Vec3::new(color[0], color[1], color[2]) * intensity;
        }
        "DireksionalLicht" => {
            if info.directional_dir.is_none() {
                let color = snapshot
                    .fields
                    .get("color")
                    .copied()
                    .and_then(|val| tri_parse_color(val))
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                let intensity = snapshot
                    .fields
                    .get("intensity")
                    .and_then(|v| mdh_number(*v))
                    .unwrap_or(1.0) as f32;
                let position = world.transform_point3(Vec3::ZERO);
                let target = snapshot
                    .fields
                    .get("lookAtTarget")
                    .and_then(|v| tri_snapshot_from_value(*v))
                    .and_then(|snap| {
                        let x = snap.fields.get("x").and_then(|v| mdh_number(*v))? as f32;
                        let y = snap.fields.get("y").and_then(|v| mdh_number(*v))? as f32;
                        let z = snap.fields.get("z").and_then(|v| mdh_number(*v))? as f32;
                        Some(Vec3::new(x, y, z))
                    });
                let dir = if let Some(target) = target {
                    (target - position).normalize_or_zero()
                } else if position.length_squared() > 1e-6 {
                    (-position).normalize()
                } else {
                    Vec3::new(0.0, -1.0, 0.0)
                };
                info.directional_dir = Some(dir);
                info.directional_color = Vec3::new(color[0], color[1], color[2]) * intensity;
            }
        }
        "PyntLicht" => {
            if info.point_pos.is_none() {
                let color = snapshot
                    .fields
                    .get("color")
                    .copied()
                    .and_then(|val| tri_parse_color(val))
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                let intensity = snapshot
                    .fields
                    .get("intensity")
                    .and_then(|v| mdh_number(*v))
                    .unwrap_or(1.0) as f32;
                let distance = snapshot
                    .fields
                    .get("distance")
                    .and_then(|v| mdh_number(*v))
                    .unwrap_or(0.0) as f32;
                let decay = snapshot
                    .fields
                    .get("decay")
                    .and_then(|v| mdh_number(*v))
                    .unwrap_or(2.0) as f32;
                let pos = world.transform_point3(Vec3::ZERO);
                info.point_pos = Some(pos);
                info.point_color = Vec3::new(color[0], color[1], color[2]) * intensity;
                info.point_distance = distance.max(0.0);
                info.point_decay = decay.max(0.0);
            }
        }
        _ => {}
    }

    if let Some(children_val) = snapshot.fields.get("children") {
        if children_val.tag == MDH_TAG_LIST {
            let len = __mdh_list_len(*children_val).max(0) as i64;
            for i in 0..len {
                let child = __mdh_list_get(*children_val, i);
                tri_collect_lights(child, world, info);
            }
        }
    }
}

unsafe fn tri_camera_view_proj(camera_val: Option<MdhValue>, fallback_aspect: f32) -> Mat4 {
    let snapshot = match camera_val.and_then(|val| tri_snapshot_from_value(val)) {
        Some(snapshot) => snapshot,
        None => return Mat4::IDENTITY,
    };
    let position = snapshot
        .fields
        .get("position")
        .map(|v| tri_vec3_from_value(*v, Vec3::ZERO))
        .unwrap_or(Vec3::ZERO);
    let rotation = snapshot
        .fields
        .get("rotation")
        .map(|v| tri_vec3_from_value(*v, Vec3::ZERO))
        .unwrap_or(Vec3::ZERO);
    let look_target = snapshot
        .fields
        .get("lookAtTarget")
        .and_then(|v| tri_snapshot_from_value(*v))
        .and_then(|snap| {
            let x = snap.fields.get("x").and_then(|v| mdh_number(*v))? as f32;
            let y = snap.fields.get("y").and_then(|v| mdh_number(*v))? as f32;
            let z = snap.fields.get("z").and_then(|v| mdh_number(*v))? as f32;
            Some(Vec3::new(x, y, z))
        });

    let view = if let Some(target) = look_target {
        Mat4::look_at_rh(position, target, Vec3::Y)
    } else {
        let quat = Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
        Mat4::from_scale_rotation_translation(Vec3::ONE, quat, position).inverse()
    };

    let aspect = snapshot
        .fields
        .get("aspect")
        .and_then(|v| mdh_number(*v))
        .map(|v| v as f32)
        .filter(|v| *v > 0.0)
        .unwrap_or(fallback_aspect.max(0.01));

    let proj = match snapshot.kind.as_str() {
        "OrthograffikKamera" => {
            let left = snapshot
                .fields
                .get("left")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(-1.0) as f32;
            let right = snapshot
                .fields
                .get("right")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            let top = snapshot
                .fields
                .get("top")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(1.0) as f32;
            let bottom = snapshot
                .fields
                .get("bottom")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(-1.0) as f32;
            let near = snapshot
                .fields
                .get("near")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(0.1) as f32;
            let far = snapshot
                .fields
                .get("far")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(2000.0) as f32;
            Mat4::orthographic_rh(left, right, bottom, top, near, far)
        }
        _ => {
            let fov_deg = snapshot
                .fields
                .get("fov")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(50.0) as f32;
            let near = snapshot
                .fields
                .get("near")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(0.1) as f32;
            let far = snapshot
                .fields
                .get("far")
                .and_then(|v| mdh_number(*v))
                .unwrap_or(2000.0) as f32;
            Mat4::perspective_rh(fov_deg.to_radians(), aspect, near, far)
        }
    };

    proj * view
}

unsafe fn tri_collect_meshes(
    value: MdhValue,
    parent: Mat4,
    lights: &LightInfo,
    items: &mut Vec<RenderItem>,
) {
    let snapshot = match tri_snapshot_from_value(value) {
        Some(snapshot) => snapshot,
        None => return,
    };
    let mut world = parent;
    if tri_has_transform(&snapshot.kind) {
        world = parent * tri_model_matrix(&snapshot);
    }

    if snapshot.kind == "Mesch" {
        let geom_val = snapshot
            .fields
            .get("geometry")
            .copied()
        .unwrap_or_else(|| __mdh_make_nil());
        if let Some((mesh, mesh_key)) = tri_mesh_from_geometry(geom_val) {
            let object_key = if value.tag == MDH_TAG_NATIVE && value.data != 0 {
                Some(value.data as usize)
            } else {
                None
            };
            let mat_val = snapshot
                .fields
                .get("material")
                .copied()
                .unwrap_or_else(|| __mdh_make_nil());
            let (color, unlit, metalness, roughness) = tri_material_info(mat_val);
            let ambient = if unlit {
                Vec3::ONE
            } else if lights.ambient.length_squared() < 1e-6 {
                Vec3::splat(0.2)
            } else {
                lights.ambient
            };
            let (light_dir, light_color) = if unlit {
                (Vec3::ZERO, Vec3::ZERO)
            } else if let Some(dir) = lights.directional_dir {
                (dir, lights.directional_color)
            } else {
                (Vec3::ZERO, Vec3::ZERO)
            };
            let (point_pos, point_color, point_params) = if unlit {
                (Vec3::ZERO, Vec3::ZERO, [0.0, 0.0, 0.0, 0.0])
            } else if let Some(pos) = lights.point_pos {
                let params = [lights.point_distance, lights.point_decay, 0.0, 0.0];
                (pos, lights.point_color, params)
            } else {
                (Vec3::ZERO, Vec3::ZERO, [0.0, 0.0, 0.0, 0.0])
            };
            let mat_params = if unlit {
                [0.0, 0.0, 0.0, 0.0]
            } else {
                [metalness, roughness, 0.0, 0.0]
            };
            items.push(RenderItem {
                mesh,
                mesh_key,
                object_key,
                model: world,
                color,
                ambient: [ambient.x, ambient.y, ambient.z, 1.0],
                light_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
                light_color: [light_color.x, light_color.y, light_color.z, 1.0],
                point_pos: [point_pos.x, point_pos.y, point_pos.z, 1.0],
                point_color: [point_color.x, point_color.y, point_color.z, 1.0],
                point_params,
                mat_params,
            });
        }
    }

    if let Some(children_val) = snapshot.fields.get("children") {
        if children_val.tag == MDH_TAG_LIST {
            let len = __mdh_list_len(*children_val).max(0) as i64;
            for i in 0..len {
                let child = __mdh_list_get(*children_val, i);
                tri_collect_meshes(child, world, lights, items);
            }
        }
    }
}

unsafe fn tri_render_items(
    scene_val: Option<MdhValue>,
    camera_val: Option<MdhValue>,
    aspect: f32,
) -> (Mat4, Vec<RenderItem>) {
    let mut items = Vec::new();
    let lights = tri_light_info(scene_val);
    if let Some(scene) = scene_val {
        tri_collect_meshes(scene, Mat4::IDENTITY, &lights, &mut items);
    }
    let view_proj = tri_camera_view_proj(camera_val, aspect);
    (view_proj, items)
}

unsafe fn tri_renderar_aspect(renderar: &TriObject) -> f32 {
    let width = renderar
        .fields
        .get("width")
        .and_then(|v| mdh_number(*v))
        .unwrap_or(1.0);
    let height = renderar
        .fields
        .get("height")
        .and_then(|v| mdh_number(*v))
        .unwrap_or(1.0);
    if height.abs() < f64::EPSILON {
        1.0
    } else {
        (width / height) as f32
    }
}

unsafe fn tri_dispose(ptr: *mut MdhNativeObject) -> MdhValue {
    if ptr.is_null() {
        return __mdh_make_nil();
    }
    let key = ptr as usize;
    let (kind, renderer_handle) = {
        let mut state = match tri_state().lock() {
            Ok(state) => state,
            Err(_) => return __mdh_make_nil(),
        };
        if let Some(obj) = state.objects.remove(&key) {
            (obj.kind, obj.renderer_handle)
        } else {
            return __mdh_make_nil();
        }
    };

    match kind.as_str() {
        "Renderar" => {
            if let Some(handle) = renderer_handle {
                with_engine(|engine| engine.remove_renderer(handle));
            }
        }
        "Geometrie" | "BoxGeometrie" | "SpherGeometrie" => {
            with_engine(|engine| engine.remove_mesh(key));
        }
        "Mesch" => {
            with_engine(|engine| engine.remove_uniform(key));
        }
        _ => {}
    }

    __mdh_make_nil()
}

unsafe fn tri_object_call(obj: &mut TriObject, method: &str, args: &[MdhValue]) -> MdhValue {
    match method {
        "cloan" | "clone" => tri_method_clone(obj),
        "adde" | "add" => {
            tri_method_add(obj, args);
            __mdh_make_nil()
        }
        "remuiv" | "remove" => {
            tri_method_remove(obj, args);
            __mdh_make_nil()
        }
        "dyspos" | "dispose" => __mdh_make_nil(),
        "luik_at" | "lookAt" => {
            if let Some(target) = args.first() {
                tri_method_set_field(obj, "lookAtTarget", *target);
            }
            __mdh_make_nil()
        }
        "set_sise" | "setSize" => {
            if let Some(width) = args.first() {
                tri_method_set_field(obj, "width", *width);
            }
            if let Some(height) = args.get(1) {
                tri_method_set_field(obj, "height", *height);
            }
            if let Some(handle) = obj.renderer_handle {
                if let (Some(w), Some(h)) = (args.first(), args.get(1)) {
                    if let (Some(wv), Some(hv)) = (mdh_number(*w), mdh_number(*h)) {
                        with_engine(|engine| engine.set_size(handle, wv as u32, hv as u32));
                    }
                }
            }
            __mdh_make_nil()
        }
        "set_pixel_ratio" | "setPixelRatio" => {
            if let Some(ratio) = args.first() {
                tri_method_set_field(obj, "pixelRatio", *ratio);
            }
            if let Some(handle) = obj.renderer_handle {
                if let Some(ratio) = args.first().and_then(|v| mdh_number(*v)) {
                    with_engine(|engine| engine.set_pixel_ratio(handle, ratio as f32));
                }
            }
            __mdh_make_nil()
        }
        "render" => {
            if let Some(scene) = args.first() {
                tri_method_set_field(obj, "scene", *scene);
            }
            if let Some(camera) = args.get(1) {
                tri_method_set_field(obj, "camera", *camera);
            }
            if let Some(handle) = obj.renderer_handle {
                let aspect = tri_renderar_aspect(obj);
                let scene_val = args
                    .first()
                    .copied()
                    .or_else(|| obj.fields.get("scene").copied());
                let camera_val = args
                    .get(1)
                    .copied()
                    .or_else(|| obj.fields.get("camera").copied());
                let wireframe = obj
                    .fields
                    .get("wireframe")
                    .and_then(|v| mdh_bool(*v))
                    .unwrap_or(false);
                let (view_proj, items) = tri_render_items(scene_val, camera_val, aspect);
                let _ =
                    with_engine(|engine| engine.render_scene(handle, view_proj, items, wireframe));
            }
            __mdh_make_nil()
        }
        "tick" | "poll" => {
            if let Some(handle) = obj.renderer_handle {
                let scene_val = obj.fields.get("scene").copied();
                let camera_val = obj.fields.get("camera").copied();
                let aspect = tri_renderar_aspect(obj);
                let wireframe = obj
                    .fields
                    .get("wireframe")
                    .and_then(|v| mdh_bool(*v))
                    .unwrap_or(false);
                let (view_proj, items) = tri_render_items(scene_val, camera_val, aspect);
                let _ =
                    with_engine(|engine| engine.render_scene(handle, view_proj, items, wireframe));
            }
            __mdh_make_nil()
        }
        "loop" => {
            if let Some(callback) = args.first() {
                tri_method_set_field(obj, "loopFn", *callback);
            }
            if let Some(handle) = obj.renderer_handle {
                let callback = args.first().copied();
                let loop_cb: Option<LoopCallback> = callback.map(|cb| {
                    let cb: LoopCallback = Box::new(move |dt| unsafe {
                        let dt_val = __mdh_make_float(dt);
                        let _ = mdh_call_value(cb, &[dt_val]);
                    });
                    cb
                });
                let _ = with_engine(|engine| engine.run_loop(handle, loop_cb));
            }
            __mdh_make_nil()
        }
        _ => __mdh_make_nil(),
    }
}

unsafe fn tri_key_not_found(name: &str) -> MdhValue {
    __mdh_key_not_found(mdh_make_string_from_rust(name));
    __mdh_make_nil()
}

unsafe fn tri_module_get(prop: &str) -> MdhValue {
    match prop {
        "DEG_TO_RAD" => __mdh_make_float(std::f64::consts::PI / 180.0),
        "RAD_TO_DEG" => __mdh_make_float(180.0 / std::f64::consts::PI),
        _ => {
            if let Some(kind) = tri_constructor_kind(prop) {
                tri_make_ctor(kind)
            } else {
                tri_key_not_found(prop)
            }
        }
    }
}

unsafe fn tri_module_call(method: &str, args: &[MdhValue]) -> MdhValue {
    if let Some(kind) = tri_constructor_kind(method) {
        return tri_make_object(kind, args);
    }
    tri_key_not_found(method)
}

#[no_mangle]
pub unsafe extern "C" fn __mdh_tri_rs_module() -> MdhValue {
    *TRI_MODULE.get_or_init(|| {
        let ptr = new_native_object(MDH_NATIVE_TRI_MODULE, "tri.module", None);
        mdh_make_native(ptr)
    })
}

#[no_mangle]
pub unsafe extern "C" fn __mdh_tri_rs_get(obj: *mut MdhNativeObject, key: MdhValue) -> MdhValue {
    if obj.is_null() {
        return __mdh_make_nil();
    }
    let prop = mdh_value_to_string(key);
    match (*obj).kind {
        MDH_NATIVE_TRI_MODULE => tri_module_get(&prop),
        MDH_NATIVE_TRI_OBJECT => {
            let value = with_object(obj, |_key, tri| tri.fields.get(&prop).copied());
            match value {
                Some(Some(v)) => v,
                _ => tri_key_not_found(&prop),
            }
        }
        MDH_NATIVE_TRI_CTOR => tri_key_not_found(&prop),
        _ => __mdh_make_nil(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn __mdh_tri_rs_set(
    obj: *mut MdhNativeObject,
    key: MdhValue,
    value: MdhValue,
) -> MdhValue {
    if obj.is_null() {
        return __mdh_make_nil();
    }
    let prop = mdh_value_to_string(key);
    match (*obj).kind {
        MDH_NATIVE_TRI_OBJECT => {
            let _ = with_object(obj, |_key, tri| tri.fields.insert(prop, value));
            value
        }
        MDH_NATIVE_TRI_MODULE | MDH_NATIVE_TRI_CTOR => {
            __mdh_hurl(mdh_make_string_from_rust("Cannae set field on tri module/ctor"));
            __mdh_make_nil()
        }
        _ => __mdh_make_nil(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn __mdh_tri_rs_call(
    obj: *mut MdhNativeObject,
    method: MdhValue,
    argc: i32,
    argv: *const MdhValue,
) -> MdhValue {
    if obj.is_null() {
        return __mdh_make_nil();
    }
    let name = mdh_value_to_string(method);
    let args = if argc <= 0 || argv.is_null() {
        Vec::new()
    } else {
        std::slice::from_raw_parts(argv, argc as usize).to_vec()
    };

    match (*obj).kind {
        MDH_NATIVE_TRI_MODULE => tri_module_call(&name, &args),
        MDH_NATIVE_TRI_CTOR => {
            let ctor_kind = native_ctor_kind(obj);
            if name.is_empty() || name == "call" || name == ctor_kind {
                tri_make_object(&ctor_kind, &args)
            } else {
                tri_key_not_found(&name)
            }
        }
        MDH_NATIVE_TRI_OBJECT => {
            if name == "dyspos" || name == "dispose" {
                return tri_dispose(obj);
            }
            let mut result = None;
            let _ = with_object(obj, |_key, tri| {
                result = Some(tri_object_call(tri, &name, &args));
            });
            result.unwrap_or_else(|| __mdh_make_nil())
        }
        _ => __mdh_make_nil(),
    }
}

extern "C" {
    fn __mdh_make_nil() -> MdhValue;
    fn __mdh_make_int(value: i64) -> MdhValue;
    fn __mdh_make_float(value: f64) -> MdhValue;
    fn __mdh_make_list(capacity: i32) -> MdhValue;
    fn __mdh_list_get(list: MdhValue, index: i64) -> MdhValue;
    fn __mdh_list_push(list: MdhValue, value: MdhValue);
    fn __mdh_list_len(list: MdhValue) -> i64;
    fn __mdh_empty_dict() -> MdhValue;
    fn __mdh_dict_get_default(dict: MdhValue, key: MdhValue, default_val: MdhValue) -> MdhValue;
    fn __mdh_hurl(value: MdhValue);
    fn __mdh_key_not_found(key: MdhValue);
    fn __mdh_eq(a: MdhValue, b: MdhValue) -> bool;
}
