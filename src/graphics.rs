//! Graphics module for mdhavers using raylib
//!
//! Provides immediate-mode graphics with Scots-themed API names.
//! All graphics functions are prefixed with "draw_" for drawing
//! and "screen_" for window/screen operations.

#[cfg(feature = "graphics")]
use raylib::prelude::*;

#[cfg(feature = "graphics")]
use std::cell::RefCell;

#[cfg(feature = "graphics")]
use std::rc::Rc;

#[cfg(feature = "graphics")]
use crate::value::{NativeFunction, Value};

#[cfg(feature = "graphics")]
thread_local! {
    static RAYLIB_HANDLE: RefCell<Option<RaylibHandle>> = const { RefCell::new(None) };
    static RAYLIB_THREAD: RefCell<Option<RaylibThread>> = const { RefCell::new(None) };
}

/// Register all graphics functions in the interpreter globals
#[cfg(feature = "graphics")]
pub fn register_graphics_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    // Window/Screen functions
    register_screen_functions(globals);

    // Drawing functions
    register_draw_functions(globals);

    // Input functions
    register_input_functions(globals);

    // Color helpers
    register_color_functions(globals);

    // Audio functions are registered separately in crate::audio
}

#[cfg(feature = "graphics")]
fn register_screen_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    // screen_open - Open a graphics window
    globals.borrow_mut().define(
        "screen_open".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_open", 3, |args| {
            let width = args[0].as_integer().ok_or("width must be an integer")? as i32;
            let height = args[1].as_integer().ok_or("height must be an integer")? as i32;
            let title = match &args[2] {
                Value::String(s) => s.clone(),
                _ => return Err("title must be a string".to_string()),
            };

            RAYLIB_HANDLE.with(|h| {
                if h.borrow().is_some() {
                    return Err("Window already open".to_string());
                }

                let (rl, thread) = raylib::init().size(width, height).title(&title).build();

                *h.borrow_mut() = Some(rl);
                RAYLIB_THREAD.with(|t| *t.borrow_mut() = Some(thread));
                Ok(Value::Nil)
            })
        }))),
    );

    // screen_close - Close the graphics window
    globals.borrow_mut().define(
        "screen_close".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_close", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                *h.borrow_mut() = None;
            });
            RAYLIB_THREAD.with(|t| {
                *t.borrow_mut() = None;
            });
            Ok(Value::Nil)
        }))),
    );

    // screen_should_close - Check if window should close (X button, ESC)
    globals.borrow_mut().define(
        "screen_should_close".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new(
            "screen_should_close",
            0,
            |_args| {
                RAYLIB_HANDLE.with(|h| {
                    let borrowed = h.borrow();
                    if let Some(rl) = borrowed.as_ref() {
                        Ok(Value::Bool(rl.window_should_close()))
                    } else {
                        Err("Window not open".to_string())
                    }
                })
            },
        ))),
    );

    // screen_begin - Begin drawing frame
    globals.borrow_mut().define(
        "screen_begin".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_begin", 0, |_args| {
            // Drawing is handled in screen_clear and screen_end
            Ok(Value::Nil)
        }))),
    );

    // screen_end - End drawing frame (swap buffers)
    globals.borrow_mut().define(
        "screen_end".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_end", 0, |_args| {
            // Raylib handles this automatically in the draw closure
            Ok(Value::Nil)
        }))),
    );

    // screen_clear - Clear the screen with a color
    globals.borrow_mut().define(
        "screen_clear".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_clear", 1, |args| {
            let color = value_to_color(&args[0])?;
            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    RAYLIB_THREAD.with(|t| {
                        let t_borrowed = t.borrow();
                        if let Some(thread) = t_borrowed.as_ref() {
                            let mut d = rl.begin_drawing(thread);
                            d.clear_background(color);
                        }
                    });
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // screen_width - Get screen width
    globals.borrow_mut().define(
        "screen_width".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_width", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Integer(rl.get_screen_width() as i64))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // screen_height - Get screen height
    globals.borrow_mut().define(
        "screen_height".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_height", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Integer(rl.get_screen_height() as i64))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // screen_fps - Set target FPS
    globals.borrow_mut().define(
        "screen_fps".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_fps", 1, |args| {
            let fps = args[0].as_integer().ok_or("fps must be an integer")? as u32;
            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    rl.set_target_fps(fps);
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );
}

#[cfg(feature = "graphics")]
fn register_draw_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    use std::rc::Rc;

    // draw_rect - Draw a filled rectangle
    globals.borrow_mut().define(
        "draw_rect".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("draw_rect", 5, |args| {
            let x = args[0].as_integer().ok_or("x must be an integer")? as i32;
            let y = args[1].as_integer().ok_or("y must be an integer")? as i32;
            let width = args[2].as_integer().ok_or("width must be an integer")? as i32;
            let height = args[3].as_integer().ok_or("height must be an integer")? as i32;
            let color = value_to_color(&args[4])?;

            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    RAYLIB_THREAD.with(|t| {
                        let t_borrowed = t.borrow();
                        if let Some(thread) = t_borrowed.as_ref() {
                            let mut d = rl.begin_drawing(thread);
                            d.draw_rectangle(x, y, width, height, color);
                        }
                    });
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // draw_circle - Draw a filled circle
    globals.borrow_mut().define(
        "draw_circle".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("draw_circle", 4, |args| {
            let x = args[0].as_integer().ok_or("x must be an integer")? as i32;
            let y = args[1].as_integer().ok_or("y must be an integer")? as i32;
            let radius = args[2].as_integer().ok_or("radius must be an integer")? as f32;
            let color = value_to_color(&args[3])?;

            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    RAYLIB_THREAD.with(|t| {
                        let t_borrowed = t.borrow();
                        if let Some(thread) = t_borrowed.as_ref() {
                            let mut d = rl.begin_drawing(thread);
                            d.draw_circle(x, y, radius, color);
                        }
                    });
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // draw_line - Draw a line
    globals.borrow_mut().define(
        "draw_line".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("draw_line", 5, |args| {
            let x1 = args[0].as_integer().ok_or("x1 must be an integer")? as i32;
            let y1 = args[1].as_integer().ok_or("y1 must be an integer")? as i32;
            let x2 = args[2].as_integer().ok_or("x2 must be an integer")? as i32;
            let y2 = args[3].as_integer().ok_or("y2 must be an integer")? as i32;
            let color = value_to_color(&args[4])?;

            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    RAYLIB_THREAD.with(|t| {
                        let t_borrowed = t.borrow();
                        if let Some(thread) = t_borrowed.as_ref() {
                            let mut d = rl.begin_drawing(thread);
                            d.draw_line(x1, y1, x2, y2, color);
                        }
                    });
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // draw_text - Draw text at position
    globals.borrow_mut().define(
        "draw_text".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("draw_text", 5, |args| {
            let text = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("text must be a string".to_string()),
            };
            let x = args[1].as_integer().ok_or("x must be an integer")? as i32;
            let y = args[2].as_integer().ok_or("y must be an integer")? as i32;
            let size = args[3].as_integer().ok_or("size must be an integer")? as i32;
            let color = value_to_color(&args[4])?;

            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    RAYLIB_THREAD.with(|t| {
                        let t_borrowed = t.borrow();
                        if let Some(thread) = t_borrowed.as_ref() {
                            let mut d = rl.begin_drawing(thread);
                            d.draw_text(&text, x, y, size, color);
                        }
                    });
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // draw_pixel - Draw a single pixel
    globals.borrow_mut().define(
        "draw_pixel".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("draw_pixel", 3, |args| {
            let x = args[0].as_integer().ok_or("x must be an integer")? as i32;
            let y = args[1].as_integer().ok_or("y must be an integer")? as i32;
            let color = value_to_color(&args[2])?;

            RAYLIB_HANDLE.with(|h| {
                let mut borrowed = h.borrow_mut();
                if let Some(rl) = borrowed.as_mut() {
                    RAYLIB_THREAD.with(|t| {
                        let t_borrowed = t.borrow();
                        if let Some(thread) = t_borrowed.as_ref() {
                            let mut d = rl.begin_drawing(thread);
                            d.draw_pixel(x, y, color);
                        }
                    });
                    Ok(Value::Nil)
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );
}

#[cfg(feature = "graphics")]
fn register_input_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    use std::rc::Rc;

    // key_pressed - Check if a key was pressed this frame
    globals.borrow_mut().define(
        "key_pressed".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("key_pressed", 1, |args| {
            let key_name = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("key name must be a string".to_string()),
            };

            let key = string_to_key(&key_name)?;

            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Bool(rl.is_key_pressed(key)))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // key_down - Check if a key is currently held down
    globals.borrow_mut().define(
        "key_down".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("key_down", 1, |args| {
            let key_name = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("key name must be a string".to_string()),
            };

            let key = string_to_key(&key_name)?;

            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Bool(rl.is_key_down(key)))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // mouse_x - Get mouse X position
    globals.borrow_mut().define(
        "mouse_x".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("mouse_x", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Integer(rl.get_mouse_x() as i64))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // mouse_y - Get mouse Y position
    globals.borrow_mut().define(
        "mouse_y".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("mouse_y", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Integer(rl.get_mouse_y() as i64))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // mouse_pressed - Check if mouse button was pressed
    globals.borrow_mut().define(
        "mouse_pressed".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("mouse_pressed", 1, |args| {
            let button = args[0].as_integer().ok_or("button must be an integer")? as i32;
            let mouse_button = match button {
                0 => MouseButton::MOUSE_BUTTON_LEFT,
                1 => MouseButton::MOUSE_BUTTON_RIGHT,
                2 => MouseButton::MOUSE_BUTTON_MIDDLE,
                _ => return Err("Invalid mouse button (use 0=left, 1=right, 2=middle)".to_string()),
            };

            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Bool(rl.is_mouse_button_pressed(mouse_button)))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );
}

#[cfg(feature = "graphics")]
fn register_color_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    use std::rc::Rc;

    // rgb - Create a color from RGB values
    globals.borrow_mut().define(
        "rgb".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("rgb", 3, |args| {
            let r = args[0].as_integer().ok_or("r must be an integer")?;
            let g = args[1].as_integer().ok_or("g must be an integer")?;
            let b = args[2].as_integer().ok_or("b must be an integer")?;
            Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(
                vec![
                    Value::Integer(r.clamp(0, 255)),
                    Value::Integer(g.clamp(0, 255)),
                    Value::Integer(b.clamp(0, 255)),
                    Value::Integer(255), // Alpha
                ],
            ))))
        }))),
    );

    // rgba - Create a color from RGBA values
    globals.borrow_mut().define(
        "rgba".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("rgba", 4, |args| {
            let r = args[0].as_integer().ok_or("r must be an integer")?;
            let g = args[1].as_integer().ok_or("g must be an integer")?;
            let b = args[2].as_integer().ok_or("b must be an integer")?;
            let a = args[3].as_integer().ok_or("a must be an integer")?;
            Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(
                vec![
                    Value::Integer(r.clamp(0, 255)),
                    Value::Integer(g.clamp(0, 255)),
                    Value::Integer(b.clamp(0, 255)),
                    Value::Integer(a.clamp(0, 255)),
                ],
            ))))
        }))),
    );
}

/// Convert a mdhavers Value to a raylib Color
#[cfg(feature = "graphics")]
fn value_to_color(value: &Value) -> Result<Color, String> {
    match value {
        // Named color strings
        Value::String(name) => string_to_color(name),
        // [r, g, b] or [r, g, b, a] list
        Value::List(list) => {
            let list = list.borrow();
            if list.len() < 3 {
                return Err("Color list must have at least 3 elements (r, g, b)".to_string());
            }
            let r = list[0].as_integer().ok_or("r must be an integer")? as u8;
            let g = list[1].as_integer().ok_or("g must be an integer")? as u8;
            let b = list[2].as_integer().ok_or("b must be an integer")? as u8;
            let a = if list.len() >= 4 {
                list[3].as_integer().ok_or("a must be an integer")? as u8
            } else {
                255
            };
            Ok(Color::new(r, g, b, a))
        }
        _ => Err("Color must be a string name or [r, g, b] list".to_string()),
    }
}

/// Convert a color name string to a raylib Color
#[cfg(feature = "graphics")]
fn string_to_color(name: &str) -> Result<Color, String> {
    match name.to_lowercase().as_str() {
        "reid" | "red" => Ok(Color::RED),
        "green" | "gress" => Ok(Color::GREEN),
        "blue" | "blae" => Ok(Color::BLUE),
        "white" | "whit" => Ok(Color::WHITE),
        "black" | "bleck" => Ok(Color::BLACK),
        "yellow" | "yella" => Ok(Color::YELLOW),
        "orange" => Ok(Color::ORANGE),
        "pink" => Ok(Color::PINK),
        "purple" | "purpie" => Ok(Color::PURPLE),
        "gray" | "grey" => Ok(Color::GRAY),
        "darkgray" | "derkgrey" => Ok(Color::DARKGRAY),
        "lightgray" | "lichtgrey" => Ok(Color::LIGHTGRAY),
        "brown" | "broon" => Ok(Color::BROWN),
        "gold" | "gowd" => Ok(Color::GOLD),
        "skyblue" | "skyblae" => Ok(Color::SKYBLUE),
        _ => Err(format!("Unknown color: {}", name)),
    }
}

/// Convert a key name string to a raylib KeyboardKey
#[cfg(feature = "graphics")]
fn string_to_key(name: &str) -> Result<KeyboardKey, String> {
    match name.to_lowercase().as_str() {
        "up" => Ok(KeyboardKey::KEY_UP),
        "down" => Ok(KeyboardKey::KEY_DOWN),
        "left" => Ok(KeyboardKey::KEY_LEFT),
        "right" => Ok(KeyboardKey::KEY_RIGHT),
        "space" => Ok(KeyboardKey::KEY_SPACE),
        "enter" => Ok(KeyboardKey::KEY_ENTER),
        "escape" | "esc" => Ok(KeyboardKey::KEY_ESCAPE),
        "a" => Ok(KeyboardKey::KEY_A),
        "b" => Ok(KeyboardKey::KEY_B),
        "c" => Ok(KeyboardKey::KEY_C),
        "d" => Ok(KeyboardKey::KEY_D),
        "e" => Ok(KeyboardKey::KEY_E),
        "f" => Ok(KeyboardKey::KEY_F),
        "g" => Ok(KeyboardKey::KEY_G),
        "h" => Ok(KeyboardKey::KEY_H),
        "i" => Ok(KeyboardKey::KEY_I),
        "j" => Ok(KeyboardKey::KEY_J),
        "k" => Ok(KeyboardKey::KEY_K),
        "l" => Ok(KeyboardKey::KEY_L),
        "m" => Ok(KeyboardKey::KEY_M),
        "n" => Ok(KeyboardKey::KEY_N),
        "o" => Ok(KeyboardKey::KEY_O),
        "p" => Ok(KeyboardKey::KEY_P),
        "q" => Ok(KeyboardKey::KEY_Q),
        "r" => Ok(KeyboardKey::KEY_R),
        "s" => Ok(KeyboardKey::KEY_S),
        "t" => Ok(KeyboardKey::KEY_T),
        "u" => Ok(KeyboardKey::KEY_U),
        "v" => Ok(KeyboardKey::KEY_V),
        "w" => Ok(KeyboardKey::KEY_W),
        "x" => Ok(KeyboardKey::KEY_X),
        "y" => Ok(KeyboardKey::KEY_Y),
        "z" => Ok(KeyboardKey::KEY_Z),
        "0" => Ok(KeyboardKey::KEY_ZERO),
        "1" => Ok(KeyboardKey::KEY_ONE),
        "2" => Ok(KeyboardKey::KEY_TWO),
        "3" => Ok(KeyboardKey::KEY_THREE),
        "4" => Ok(KeyboardKey::KEY_FOUR),
        "5" => Ok(KeyboardKey::KEY_FIVE),
        "6" => Ok(KeyboardKey::KEY_SIX),
        "7" => Ok(KeyboardKey::KEY_SEVEN),
        "8" => Ok(KeyboardKey::KEY_EIGHT),
        "9" => Ok(KeyboardKey::KEY_NINE),
        _ => Err(format!("Unknown key: {}", name)),
    }
}

/// Stub for when graphics feature is not enabled
#[cfg(not(feature = "graphics"))]
pub fn register_graphics_functions(
    _globals: &std::rc::Rc<std::cell::RefCell<crate::value::Environment>>,
) {
    // Graphics not available - do nothing
}
