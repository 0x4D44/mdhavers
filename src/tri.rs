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
