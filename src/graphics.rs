//! Graphics module for mdhavers using raylib
//!
//! Provides immediate-mode graphics with Scots-themed API names.
//! All graphics functions are prefixed with "draw_" for drawing
//! and "screen_" for window/screen operations.

#[cfg(feature = "graphics")]
use raylib::prelude::*;

#[cfg(feature = "graphics")]
use std::cell::RefCell;

use std::rc::Rc;

use crate::value::{NativeFunction, Value};

#[cfg(feature = "graphics")]
thread_local! {
    static RAYLIB_HANDLE: RefCell<Option<RaylibHandle>> = const { RefCell::new(None) };
    static RAYLIB_THREAD: RefCell<Option<RaylibThread>> = const { RefCell::new(None) };
    static FRAMEBUFFER_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static FRAMEBUFFER_RGBA: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Register all graphics functions in the interpreter globals
#[cfg(feature = "graphics")]
pub fn register_graphics_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    register_sound_functions(globals);
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
                // Drop the GPU texture while its graphics context is still open.
                FRAMEBUFFER_TEXTURE.with(|texture| {
                    texture.borrow_mut().take();
                });
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

    // screen_shaw - Present indexed RGB or BGRA framebuffer bytes through one
    // reusable, point-filtered texture. The existing window dimensions remain
    // the logical framebuffer dimensions.
    globals.borrow_mut().define(
        "screen_shaw".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("screen_shaw", 2, |args| {
            let pixels = match &args[0] {
                Value::Bytes(bytes) => bytes.borrow(),
                _ => return Err("screen_shaw() pixels must be bytes".to_string()),
            };
            let palette = match &args[1] {
                Value::Nil => None,
                Value::Bytes(bytes) => Some(bytes.borrow()),
                _ => return Err("screen_shaw() palette must be bytes or naething".to_string()),
            };

            if let Some(palette) = palette.as_ref() {
                if palette.len() != 256 * 3 {
                    return Err("screen_shaw() palette must contain 768 RGB bytes".to_string());
                }
            }

            let (window_width, window_height) = RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok((rl.get_screen_width(), rl.get_screen_height()))
                } else {
                    Err("Window not open".to_string())
                }
            })?;
            if window_width <= 0 || window_height <= 0 {
                return Err("screen_shaw() window size must be positive".to_string());
            }
            let bytes_per_pixel = if palette.is_some() { 1 } else { 4 };
            let width = window_width;
            let height = window_height;
            let pixel_count = (width as usize)
                .checked_mul(height as usize)
                .ok_or_else(|| "screen_shaw() framebuffer is too large".to_string())?;
            let expected_len = pixel_count
                .checked_mul(bytes_per_pixel)
                .ok_or_else(|| "screen_shaw() framebuffer is too large".to_string())?;
            if pixels.len() != expected_len {
                return Err(format!(
                    "screen_shaw() needs exactly {} pixel bytes, got {}",
                    expected_len,
                    pixels.len()
                ));
            }
            let byte_count = pixel_count
                .checked_mul(4)
                .ok_or_else(|| "screen_shaw() framebuffer is too large".to_string())?;

            FRAMEBUFFER_RGBA.with(|rgba_cell| {
                let mut rgba = rgba_cell.borrow_mut();
                rgba.resize(byte_count, 0);
                if let Some(palette) = palette.as_ref() {
                    for (index, &color_index) in pixels.iter().enumerate() {
                        let palette_index = color_index as usize * 3;
                        let rgba_index = index * 4;
                        rgba[rgba_index] = palette[palette_index];
                        rgba[rgba_index + 1] = palette[palette_index + 1];
                        rgba[rgba_index + 2] = palette[palette_index + 2];
                        rgba[rgba_index + 3] = 255;
                    }
                } else {
                    for (source, destination) in
                        pixels.chunks_exact(4).zip(rgba.chunks_exact_mut(4))
                    {
                        // The bridge's naething-palette format is BGRA. Native
                        // treats alpha as padding (AlphaNoneSkipFirst), so use
                        // opaque alpha here to keep the displayed result equal.
                        destination[0] = source[2];
                        destination[1] = source[1];
                        destination[2] = source[0];
                        destination[3] = 255;
                    }
                }

                RAYLIB_HANDLE.with(|h| {
                    let mut borrowed = h.borrow_mut();
                    let Some(rl) = borrowed.as_mut() else {
                        return Err("Window not open".to_string());
                    };
                    RAYLIB_THREAD.with(|t| {
                        let thread_borrowed = t.borrow();
                        let Some(thread) = thread_borrowed.as_ref() else {
                            return Err("Graphics thread not available".to_string());
                        };
                        FRAMEBUFFER_TEXTURE.with(|texture_cell| {
                            let mut texture_slot = texture_cell.borrow_mut();
                            let texture_mismatched = texture_slot
                                .as_ref()
                                .map(|texture| {
                                    texture.width() != width || texture.height() != height
                                })
                                .unwrap_or(true);
                            if texture_mismatched {
                                // Replacing a texture happens before the next one is
                                // created and while the raylib context is still live.
                                texture_slot.take();
                                let image = Image::gen_image_color(width, height, Color::BLANK);
                                let texture = rl
                                    .load_texture_from_image(thread, &image)
                                    .map_err(|error| error.to_string())?;
                                texture.set_texture_filter(
                                    thread,
                                    TextureFilter::TEXTURE_FILTER_POINT,
                                );
                                *texture_slot = Some(texture);
                            }

                            let texture = texture_slot.as_mut().expect("texture was created");
                            texture
                                .update_texture(rgba.as_slice())
                                .map_err(|error| error.to_string())?;
                            let mut drawing = rl.begin_drawing(thread);
                            drawing.clear_background(Color::BLACK);
                            drawing.draw_texture_pro(
                                texture,
                                Rectangle::new(0.0, 0.0, width as f32, height as f32),
                                Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32),
                                Vector2::new(0.0, 0.0),
                                0.0,
                                Color::WHITE,
                            );
                            Ok(Value::Nil)
                        })
                    })
                })
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

    // mouse_dx / mouse_dy - Get raw motion since raylib's previous frame.
    globals.borrow_mut().define(
        "mouse_dx".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("mouse_dx", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Integer(rl.get_mouse_delta().x as i64))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );
    globals.borrow_mut().define(
        "mouse_dy".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("mouse_dy", 0, |_args| {
            RAYLIB_HANDLE.with(|h| {
                let borrowed = h.borrow();
                if let Some(rl) = borrowed.as_ref() {
                    Ok(Value::Integer(rl.get_mouse_delta().y as i64))
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

    // mouse_down - Check whether a mouse button is currently held.
    globals.borrow_mut().define(
        "mouse_down".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new("mouse_down", 1, |args| {
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
                    Ok(Value::Bool(rl.is_mouse_button_down(mouse_button)))
                } else {
                    Err("Window not open".to_string())
                }
            })
        }))),
    );

    // screen_grab_mouse - Hide and lock the pointer for relative mouse input.
    globals.borrow_mut().define(
        "screen_grab_mouse".to_string(),
        Value::NativeFunction(Rc::new(NativeFunction::new(
            "screen_grab_mouse",
            1,
            |args| {
                let grabbed = match &args[0] {
                    Value::Bool(grabbed) => *grabbed,
                    _ => return Err("screen_grab_mouse() expects aye or nae".to_string()),
                };
                RAYLIB_HANDLE.with(|h| {
                    let mut borrowed = h.borrow_mut();
                    if let Some(rl) = borrowed.as_mut() {
                        if grabbed {
                            rl.disable_cursor();
                        } else {
                            rl.enable_cursor();
                        }
                        Ok(Value::Nil)
                    } else {
                        Err("Window not open".to_string())
                    }
                })
            },
        ))),
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
    globals: &std::rc::Rc<std::cell::RefCell<crate::value::Environment>>,
) {
    register_sound_functions(globals);
    // Keep the framebuffer bridge vocabulary available in CLI/minimal builds;
    // calls fail with the same Scots hurl as the native runtime placeholders.
    const BRIDGE_FUNCTIONS: &[(&str, usize)] = &[
        ("screen_open", 3),
        ("screen_shaw", 2),
        ("screen_should_close", 0),
        ("screen_close", 0),
        ("screen_width", 0),
        ("screen_height", 0),
        ("key_down", 1),
        ("key_pressed", 1),
        ("mouse_x", 0),
        ("mouse_y", 0),
        ("mouse_dx", 0),
        ("mouse_dy", 0),
        ("mouse_down", 1),
        ("screen_grab_mouse", 1),
    ];
    for &(name, arity) in BRIDGE_FUNCTIONS {
        globals.borrow_mut().define(
            name.to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, |_args| {
                Err(
                    "Nae windae! Screen an' keyboard functions cannae be used in native code"
                        .to_string(),
                )
            }))),
        );
    }
}

/// The interpreter uses deterministic null sound in every feature configuration.
/// State belongs to this interpreter, and queued input is copied before returning.
fn register_sound_functions(globals: &Rc<std::cell::RefCell<crate::value::Environment>>) {
    #[derive(Default)]
    struct Sound {
        channels: usize,
        pcm: Vec<u8>,
    }
    let state = Rc::new(std::cell::RefCell::new(Sound::default()));
    for (name, arity) in [
        ("sound_open", 2),
        ("sound_queue", 1),
        ("sound_queued", 0),
        ("sound_close", 0),
    ] {
        let state = state.clone();
        globals.borrow_mut().define(
            name.to_owned(),
            Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, move |args| {
                let mut sound = state.borrow_mut();
                match name {
                    "sound_open" => {
                        let channels = match (&args[0], &args[1]) {
                            (Value::Integer(22050 | 44100), Value::Integer(channels @ (1 | 2))) => {
                                *channels as usize
                            }
                            _ => {
                                return Err(
                                    "sound_open wants rate 22050 or 44100 and channels 1 or 2"
                                        .to_owned(),
                                )
                            }
                        };
                        if sound.channels != 0 {
                            return Err("sound_open: sound is already open".to_owned());
                        }
                        sound.channels = channels;
                        sound.pcm.clear();
                        Ok(Value::Nil)
                    }
                    "sound_queue" => {
                        if sound.channels == 0 {
                            return Err("sound_queue: nae sound is open".to_owned());
                        }
                        let pcm = match &args[0] {
                            Value::Bytes(pcm) => pcm.borrow(),
                            _ => return Err("sound_queue wants bytes PCM".to_owned()),
                        };
                        if pcm.len() % (sound.channels * 2) != 0 {
                            return Err("sound_queue wants complete signed16 PCM frames".to_owned());
                        }
                        if pcm.len() > 1048576 - sound.pcm.len() {
                            return Err("sound_queue: PCM ring is full".to_owned());
                        }
                        sound.pcm.extend_from_slice(&pcm);
                        Ok(Value::Nil)
                    }
                    "sound_queued" => Ok(Value::Integer((sound.pcm.len() / 2) as i64)),
                    "sound_close" => {
                        sound.channels = 0;
                        sound.pcm.clear();
                        Ok(Value::Nil)
                    }
                    _ => unreachable!(),
                }
            }))),
        );
    }
}
