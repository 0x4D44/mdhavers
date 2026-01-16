use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{HaversError, HaversResult};
use crate::value::{NativeFunction, NativeObject, Value};

pub fn tri_module_value() -> Value {
    Value::NativeObject(Rc::new(TriModule::new()))
}

pub fn is_tri_module(path: &str) -> bool {
    path == "tri" || path == "tri.braw"
}

#[derive(Debug)]
struct TriModule {
    constants: HashMap<&'static str, Value>,
}

impl TriModule {
    fn new() -> Self {
        let mut constants = HashMap::new();
        constants.insert("DEG_TO_RAD", Value::Float(std::f64::consts::PI / 180.0));
        constants.insert("RAD_TO_DEG", Value::Float(180.0 / std::f64::consts::PI));
        TriModule { constants }
    }

    fn constructor_kind(method: &str) -> Option<&'static str> {
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
}

impl NativeObject for TriModule {
    fn type_name(&self) -> &str {
        "tri.module"
    }

    fn get(&self, prop: &str) -> HaversResult<Value> {
        if let Some(val) = self.constants.get(prop) {
            return Ok(val.clone());
        }
        if let Some(kind) = Self::constructor_kind(prop) {
            return Ok(make_constructor(kind));
        }
        Err(HaversError::UndefinedVariable {
            name: prop.to_string(),
            line: 0,
        })
    }

    fn set(&self, prop: &str, _value: Value) -> HaversResult<Value> {
        Err(HaversError::TypeError {
            message: format!("Cannae set '{}' on tri module", prop),
            line: 0,
        })
    }

    fn call(&self, method: &str, args: Vec<Value>) -> HaversResult<Value> {
        if let Some(kind) = Self::constructor_kind(method) {
            let obj = TriObject::with_args(kind, &args);
            return Ok(Value::NativeObject(Rc::new(obj)));
        }
        Err(HaversError::UndefinedVariable {
            name: method.to_string(),
            line: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct TriObject {
    kind: &'static str,
    fields: RefCell<HashMap<String, Value>>,
}

impl TriObject {
    fn new(kind: &'static str) -> Self {
        let mut fields = HashMap::new();
        fields.insert("type".to_string(), Value::String(kind.to_string()));
        if tri_has_transform(kind) {
            fields.insert("position".to_string(), make_vec3("Vec3", 0.0, 0.0, 0.0));
            fields.insert("rotation".to_string(), make_vec3("Euler", 0.0, 0.0, 0.0));
            fields.insert("scale".to_string(), make_vec3("Vec3", 1.0, 1.0, 1.0));
            fields.insert(
                "children".to_string(),
                Value::List(Rc::new(RefCell::new(Vec::new()))),
            );
            fields.insert("parent".to_string(), Value::Nil);
        }
        TriObject {
            kind,
            fields: RefCell::new(fields),
        }
    }

    fn with_args(kind: &'static str, args: &[Value]) -> Self {
        let obj = TriObject::new(kind);
        {
            let mut fields = obj.fields.borrow_mut();
            apply_constructor_args(kind, &mut fields, args);
        }
        obj
    }
}

impl NativeObject for TriObject {
    fn type_name(&self) -> &str {
        self.kind
    }

    fn get(&self, prop: &str) -> HaversResult<Value> {
        self.fields
            .borrow()
            .get(prop)
            .cloned()
            .ok_or_else(|| HaversError::UndefinedVariable {
                name: prop.to_string(),
                line: 0,
            })
    }

    fn set(&self, prop: &str, value: Value) -> HaversResult<Value> {
        self.fields
            .borrow_mut()
            .insert(prop.to_string(), value.clone());
        Ok(value)
    }

    fn call(&self, method: &str, args: Vec<Value>) -> HaversResult<Value> {
        match method {
            "cloan" | "clone" => {
                let fields = self.fields.borrow().clone();
                Ok(Value::NativeObject(Rc::new(TriObject {
                    kind: self.kind,
                    fields: RefCell::new(fields),
                })))
            }
            "adde" | "add" => {
                self.add_children(&args);
                Ok(Value::Nil)
            }
            "remuiv" | "remove" => {
                self.remove_children(&args);
                Ok(Value::Nil)
            }
            "dyspos" | "dispose" => Ok(Value::Nil),
            "luik_at" | "lookAt" => {
                if let Some(target) = args.first() {
                    self.fields
                        .borrow_mut()
                        .insert("lookAtTarget".to_string(), target.clone());
                }
                Ok(Value::Nil)
            }
            "set_sise" | "setSize" => {
                if let Some(width) = args.first() {
                    self.fields
                        .borrow_mut()
                        .insert("width".to_string(), width.clone());
                }
                if let Some(height) = args.get(1) {
                    self.fields
                        .borrow_mut()
                        .insert("height".to_string(), height.clone());
                }
                Ok(Value::Nil)
            }
            "set_pixel_ratio" | "setPixelRatio" => {
                if let Some(ratio) = args.first() {
                    self.fields
                        .borrow_mut()
                        .insert("pixelRatio".to_string(), ratio.clone());
                }
                Ok(Value::Nil)
            }
            "render" => {
                if let Some(scene) = args.first() {
                    self.fields
                        .borrow_mut()
                        .insert("scene".to_string(), scene.clone());
                }
                if let Some(camera) = args.get(1) {
                    self.fields
                        .borrow_mut()
                        .insert("camera".to_string(), camera.clone());
                }
                Ok(Value::Nil)
            }
            "loop" => {
                if let Some(callback) = args.first() {
                    self.fields
                        .borrow_mut()
                        .insert("loopFn".to_string(), callback.clone());
                }
                Ok(Value::Nil)
            }
            _ => Ok(Value::Nil),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TriObject {
    fn add_children(&self, args: &[Value]) {
        let list = {
            let mut fields = self.fields.borrow_mut();
            if let Some(Value::List(children)) = fields.get("children") {
                children.clone()
            } else {
                let children = Rc::new(RefCell::new(Vec::new()));
                fields.insert("children".to_string(), Value::List(children.clone()));
                children
            }
        };
        let mut list_mut = list.borrow_mut();
        for arg in args {
            list_mut.push(arg.clone());
        }
    }

    fn remove_children(&self, args: &[Value]) {
        let children = {
            let fields = self.fields.borrow();
            match fields.get("children") {
                Some(Value::List(children)) => Some(children.clone()),
                _ => None,
            }
        };
        if let Some(children) = children {
            let mut list = children.borrow_mut();
            list.retain(|item| !args.iter().any(|arg| arg == item));
        }
    }
}

fn make_vec3(kind: &'static str, x: f64, y: f64, z: f64) -> Value {
    let obj = TriObject::new(kind);
    {
        let mut fields = obj.fields.borrow_mut();
        fields.insert("x".to_string(), Value::Float(x));
        fields.insert("y".to_string(), Value::Float(y));
        fields.insert("z".to_string(), Value::Float(z));
    }
    Value::NativeObject(Rc::new(obj))
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

fn apply_constructor_args(kind: &str, fields: &mut HashMap<String, Value>, args: &[Value]) {
    match kind {
        "Mesch" => {
            set_arg(fields, args, 0, "geometry", Value::Nil);
            set_arg(fields, args, 1, "material", Value::Nil);
        }
        "PerspectivKamera" => {
            set_arg(fields, args, 0, "fov", Value::Integer(50));
            set_arg(fields, args, 1, "aspect", Value::Integer(1));
            set_arg(fields, args, 2, "near", Value::Float(0.1));
            set_arg(fields, args, 3, "far", Value::Integer(2000));
        }
        "OrthograffikKamera" => {
            set_arg(fields, args, 0, "left", Value::Integer(-1));
            set_arg(fields, args, 1, "right", Value::Integer(1));
            set_arg(fields, args, 2, "top", Value::Integer(1));
            set_arg(fields, args, 3, "bottom", Value::Integer(-1));
            set_arg(fields, args, 4, "near", Value::Float(0.1));
            set_arg(fields, args, 5, "far", Value::Integer(2000));
        }
        "BoxGeometrie" => {
            set_arg(fields, args, 0, "width", Value::Integer(1));
            set_arg(fields, args, 1, "height", Value::Integer(1));
            set_arg(fields, args, 2, "depth", Value::Integer(1));
        }
        "SpherGeometrie" => {
            set_arg(fields, args, 0, "radius", Value::Integer(1));
            set_arg(fields, args, 1, "widthSegments", Value::Integer(8));
            set_arg(fields, args, 2, "heightSegments", Value::Integer(6));
        }
        "Maiterial" | "MeshBasicMaiterial" | "MeshStandardMaiterial" | "Renderar" => {
            set_arg(fields, args, 0, "opts", Value::Nil);
        }
        "Licht" | "AmbiantLicht" | "DireksionalLicht" => {
            set_arg(fields, args, 0, "color", Value::Nil);
            set_arg(fields, args, 1, "intensity", Value::Integer(1));
        }
        "PyntLicht" => {
            set_arg(fields, args, 0, "color", Value::Nil);
            set_arg(fields, args, 1, "intensity", Value::Integer(1));
            set_arg(fields, args, 2, "distance", Value::Integer(0));
            set_arg(fields, args, 3, "decay", Value::Integer(2));
        }
        "Colour" => {
            set_arg(fields, args, 0, "value", Value::Nil);
        }
        _ => {}
    }
}

fn set_arg(
    fields: &mut HashMap<String, Value>,
    args: &[Value],
    index: usize,
    name: &str,
    default: Value,
) {
    let value = args.get(index).cloned().unwrap_or(default);
    fields.insert(name.to_string(), value);
}

fn make_constructor(kind: &'static str) -> Value {
    let name = format!("tri.{}", kind);
    let func = NativeFunction::new(&name, usize::MAX, move |args| {
        Ok(Value::NativeObject(Rc::new(TriObject::with_args(
            kind, &args,
        ))))
    });
    Value::NativeFunction(Rc::new(func))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec3_components(value: &Value) -> Option<(f64, f64, f64)> {
        let obj = match value {
            Value::NativeObject(obj) => obj,
            _ => return None,
        };
        let x = obj.get("x").ok()?.as_float()?;
        let y = obj.get("y").ok()?.as_float()?;
        let z = obj.get("z").ok()?.as_float()?;
        Some((x, y, z))
    }
    #[test]
    fn test_is_tri_module() {
        assert!(is_tri_module("tri"));
        assert!(is_tri_module("tri.braw"));
        assert!(!is_tri_module("tri.txt"));
        assert!(!is_tri_module("math"));
    }

    #[test]
    fn test_tri_module_to_string_uses_native_object_default_impl() {
        let module = TriModule::new();
        let module_obj: &dyn NativeObject = &module;
        assert_eq!(module_obj.to_string(), "<native tri.module>");
    }

    #[test]
    fn test_tri_module_constants_and_constructors() {
        let module = TriModule::new();
        let deg = module.get("DEG_TO_RAD").unwrap();
        let deg = deg.as_float().expect("expected float");
        assert!((deg - std::f64::consts::PI / 180.0).abs() < 1e-6);

        let result = module.call("Mesch", vec![]).unwrap();
        assert_eq!(format!("{}", result), "<native Mesch>");
    }

    #[test]
    fn test_tri_module_constructors_cover_all_kinds_for_unit_coverage() {
        let module = TriModule::new();

        let kinds = [
            "Sicht",
            "Thing3D",
            "Clump",
            "Mesch",
            "Kamera",
            "PerspectivKamera",
            "OrthograffikKamera",
            "Geometrie",
            "BoxGeometrie",
            "SpherGeometrie",
            "Maiterial",
            "MeshBasicMaiterial",
            "MeshStandardMaiterial",
            "Licht",
            "AmbiantLicht",
            "DireksionalLicht",
            "PyntLicht",
            "Textur",
            "Renderar",
            "Colour",
        ];

        fn args_for_kind(kind: &str) -> Vec<Value> {
            match kind {
                "Mesch" => vec![Value::Integer(1), Value::Integer(2)],
                "PerspectivKamera" => vec![
                    Value::Integer(75),
                    Value::Float(1.6),
                    Value::Float(0.2),
                    Value::Integer(100),
                ],
                "OrthograffikKamera" => vec![
                    Value::Integer(-2),
                    Value::Integer(2),
                    Value::Integer(2),
                    Value::Integer(-2),
                    Value::Float(0.01),
                    Value::Integer(999),
                ],
                "BoxGeometrie" => vec![Value::Integer(2), Value::Integer(3), Value::Integer(4)],
                "SpherGeometrie" => vec![Value::Integer(2), Value::Integer(9), Value::Integer(7)],
                "Maiterial" | "MeshBasicMaiterial" | "MeshStandardMaiterial" | "Renderar" => {
                    vec![Value::String("opts".to_string())]
                }
                "Licht" | "AmbiantLicht" | "DireksionalLicht" => {
                    vec![Value::Integer(1), Value::Float(0.5)]
                }
                "PyntLicht" => vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                    Value::Integer(4),
                ],
                "Colour" => vec![Value::String("red".to_string())],
                _ => Vec::new(),
            }
        }

        for kind in kinds {
            let ctor_val = module.get(kind).unwrap();
            let ctor = ctor_val
                .as_native_function()
                .expect("expected native constructor");

            // Empty args covers default-path constructor arg plumbing.
            let empty = (ctor.func)(Vec::new()).unwrap();
            assert_eq!(format!("{}", empty), format!("<native {}>", kind));

            // Non-empty args covers explicit-arg path for kinds that accept options.
            let with_args = (ctor.func)(args_for_kind(kind)).unwrap();
            assert_eq!(format!("{}", with_args), format!("<native {}>", kind));

            // Also cover the module.call(...) constructor path.
            let called = module.call(kind, Vec::new()).unwrap();
            assert_eq!(format!("{}", called), format!("<native {}>", kind));
        }
    }

    #[test]
    fn test_tri_module_get_set_and_call_errors() {
        let module = TriModule::new();
        let err = module.get("Nope").unwrap_err();
        let is_undefined = |e: &HaversError| matches!(e, HaversError::UndefinedVariable { .. });
        assert!(is_undefined(&err));
        assert!(!is_undefined(&HaversError::TypeError {
            message: String::new(),
            line: 0,
        }));

        let err = module.set("x", Value::Nil).unwrap_err();
        let is_type_error = |e: &HaversError| matches!(e, HaversError::TypeError { .. });
        assert!(is_type_error(&err));
        assert!(!is_type_error(&HaversError::UndefinedVariable {
            name: String::new(),
            line: 0,
        }));

        let err = module.call("Nope", vec![]).unwrap_err();
        assert!(is_undefined(&err));
    }

    #[test]
    fn test_tri_module_call_constructor() {
        let module = TriModule::new();
        let value = module.call("BoxGeometrie", vec![]).unwrap();
        assert_eq!(format!("{}", value), "<native BoxGeometrie>");
    }

    #[test]
    fn test_tri_object_transform_fields() {
        let obj = TriObject::new("Thing3D");
        assert!(obj.get("position").is_ok());
        assert!(obj.get("rotation").is_ok());
        assert!(obj.get("scale").is_ok());
        assert!(obj.get("children").is_ok());
        assert_eq!(obj.get("parent").unwrap(), Value::Nil);
    }

    #[test]
    fn test_tri_object_non_transform_fields() {
        let obj = TriObject::new("Geometrie");
        assert!(obj.get("position").is_err());
        assert!(obj.get("children").is_err());
    }

    #[test]
    fn test_apply_constructor_args_defaults() {
        let obj = TriObject::with_args("BoxGeometrie", &[]);
        assert_eq!(obj.get("width").unwrap(), Value::Integer(1));
        assert_eq!(obj.get("height").unwrap(), Value::Integer(1));
        assert_eq!(obj.get("depth").unwrap(), Value::Integer(1));

        let obj = TriObject::with_args(
            "PerspectivKamera",
            &[
                Value::Integer(75),
                Value::Float(1.6),
                Value::Float(0.2),
                Value::Integer(100),
            ],
        );
        assert_eq!(obj.get("fov").unwrap(), Value::Integer(75));
        assert_eq!(obj.get("aspect").unwrap(), Value::Float(1.6));
        assert_eq!(obj.get("near").unwrap(), Value::Float(0.2));
        assert_eq!(obj.get("far").unwrap(), Value::Integer(100));

        let obj = TriObject::with_args("Colour", &[]);
        assert_eq!(obj.get("value").unwrap(), Value::Nil);
    }

    #[test]
    fn test_tri_object_methods_and_children() {
        let obj = TriObject::new("Thing3D");
        obj.call("adde", vec![Value::Integer(1), Value::Integer(2)])
            .unwrap();
        let children = obj.get("children").unwrap();
        let list = children.as_list().expect("expected list");
        assert_eq!(list.borrow().len(), 2);

        obj.call("remuiv", vec![Value::Integer(1)]).unwrap();
        let children = obj.get("children").unwrap();
        let list = children.as_list().expect("expected list");
        assert_eq!(list.borrow().len(), 1);

        obj.call("luik_at", vec![Value::String("target".to_string())])
            .unwrap();
        obj.call("luik_at", Vec::new()).unwrap();
        assert_eq!(
            obj.get("lookAtTarget").unwrap(),
            Value::String("target".to_string())
        );

        obj.call("set_sise", vec![Value::Integer(640), Value::Integer(480)])
            .unwrap();
        obj.call("set_sise", Vec::new()).unwrap();
        assert_eq!(obj.get("width").unwrap(), Value::Integer(640));
        assert_eq!(obj.get("height").unwrap(), Value::Integer(480));

        obj.call("set_pixel_ratio", vec![Value::Float(2.0)])
            .unwrap();
        obj.call("set_pixel_ratio", Vec::new()).unwrap();
        assert_eq!(obj.get("pixelRatio").unwrap(), Value::Float(2.0));

        obj.call(
            "render",
            vec![
                Value::String("scene".to_string()),
                Value::String("camera".to_string()),
            ],
        )
        .unwrap();
        obj.call("render", Vec::new()).unwrap();
        assert_eq!(
            obj.get("scene").unwrap(),
            Value::String("scene".to_string())
        );
        assert_eq!(
            obj.get("camera").unwrap(),
            Value::String("camera".to_string())
        );

        obj.call("loop", vec![Value::String("cb".to_string())])
            .unwrap();
        obj.call("loop", Vec::new()).unwrap();
        assert_eq!(obj.get("loopFn").unwrap(), Value::String("cb".to_string()));

        assert_eq!(obj.call("dyspos", Vec::new()).unwrap(), Value::Nil);

        let cloned = obj.call("cloan", vec![]).unwrap();
        assert_eq!(format!("{}", cloned), "<native Thing3D>");
    }

    #[test]
    fn test_tri_module_type_name_and_as_any() {
        let module = TriModule::new();
        assert_eq!(module.type_name(), "tri.module");
        assert!(module.as_any().is::<TriModule>());
    }

    #[test]
    fn test_tri_object_set_unknown_call_and_as_any() {
        let obj = TriObject::new("Thing3D");
        obj.set("custom", Value::Integer(5)).unwrap();
        assert_eq!(obj.get("custom").unwrap(), Value::Integer(5));
        assert_eq!(obj.call("nope", vec![]).unwrap(), Value::Nil);
        assert!(obj.as_any().is::<TriObject>());
    }

    #[test]
    fn test_tri_object_remove_children_without_list() {
        let obj = TriObject::new("Geometrie");
        obj.call("remuiv", vec![Value::Integer(1)]).unwrap();
        assert!(obj.get("children").is_err());
    }

    #[test]
    fn test_apply_constructor_args_more_kinds() {
        let obj = TriObject::with_args("OrthograffikKamera", &[]);
        assert_eq!(obj.get("left").unwrap(), Value::Integer(-1));
        assert_eq!(obj.get("right").unwrap(), Value::Integer(1));
        assert_eq!(obj.get("top").unwrap(), Value::Integer(1));
        assert_eq!(obj.get("bottom").unwrap(), Value::Integer(-1));
        assert_eq!(obj.get("near").unwrap(), Value::Float(0.1));
        assert_eq!(obj.get("far").unwrap(), Value::Integer(2000));

        let obj = TriObject::with_args("SpherGeometrie", &[]);
        assert_eq!(obj.get("radius").unwrap(), Value::Integer(1));
        assert_eq!(obj.get("widthSegments").unwrap(), Value::Integer(8));
        assert_eq!(obj.get("heightSegments").unwrap(), Value::Integer(6));

        let obj = TriObject::with_args("Maiterial", &[]);
        assert_eq!(obj.get("opts").unwrap(), Value::Nil);
        let obj = TriObject::with_args("MeshBasicMaiterial", &[]);
        assert_eq!(obj.get("opts").unwrap(), Value::Nil);
        let obj = TriObject::with_args("Renderar", &[]);
        assert_eq!(obj.get("opts").unwrap(), Value::Nil);

        let obj = TriObject::with_args("Licht", &[]);
        assert_eq!(obj.get("color").unwrap(), Value::Nil);
        assert_eq!(obj.get("intensity").unwrap(), Value::Integer(1));

        let obj = TriObject::with_args("PyntLicht", &[]);
        assert_eq!(obj.get("color").unwrap(), Value::Nil);
        assert_eq!(obj.get("intensity").unwrap(), Value::Integer(1));
        assert_eq!(obj.get("distance").unwrap(), Value::Integer(0));
        assert_eq!(obj.get("decay").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_tri_object_children_on_non_transform() {
        let obj = TriObject::new("Geometrie");
        obj.call("adde", vec![Value::Integer(1)]).unwrap();
        let children = obj.get("children").unwrap();
        let list = children.as_list().expect("expected list");
        assert_eq!(list.borrow().len(), 1);
        obj.call("remuiv", vec![Value::Integer(2)]).unwrap();
    }

    #[test]
    fn test_make_vec3_and_transform_check() {
        let vec = make_vec3("Vec3", 1.0, 2.0, 3.0);
        assert_eq!(vec3_components(&vec), Some((1.0, 2.0, 3.0)));
        assert_eq!(vec3_components(&Value::Nil), None);

        assert!(tri_has_transform("Sicht"));
        assert!(!tri_has_transform("Geometrie"));
    }

    #[test]
    fn test_add_children_creates_list() {
        let obj = TriObject::new("Maiterial");
        obj.add_children(&[Value::Integer(1)]);
        let children = obj.get("children").unwrap();
        let list = children.as_list().expect("expected list");
        assert_eq!(list.borrow().len(), 1);
    }
}
