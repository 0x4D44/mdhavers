//! Unified Audio Abstraction for mdhavers
//!
//! This module provides a backend-agnostic audio API that works with either:
//! - **miniaudio** backend: Used when `audio` feature is enabled (without `graphics`)
//! - **raylib** backend: Used when `graphics` feature is enabled (raylib includes audio)
//!
//! The backend selection is done at compile-time to avoid symbol conflicts on Windows
//! where both miniaudio and raylib would otherwise link conflicting symbols.

// Backend modules - only one will be compiled based on features
#[cfg(all(feature = "audio", not(feature = "graphics")))]
mod backend_miniaudio;
#[cfg(all(feature = "audio", not(feature = "graphics")))]
use backend_miniaudio as backend;

#[cfg(feature = "graphics")]
mod backend_raylib;
#[cfg(feature = "graphics")]
use backend_raylib as backend;

// MIDI support (shared between backends, requires rustysynth)
#[cfg(any(feature = "audio", feature = "graphics"))]
mod midi;

use std::cell::RefCell;
use std::rc::Rc;

// Re-export the backend types through unified names
#[cfg(any(feature = "audio", feature = "graphics"))]
pub use backend::{AudioEngine, Music, Sound};

#[cfg(any(feature = "audio", feature = "graphics"))]
pub use midi::MidiPlayer;

/// Error type for audio operations
#[derive(Debug, Clone)]
pub struct AudioError {
    pub message: String,
}

impl AudioError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AudioError {}

impl From<String> for AudioError {
    fn from(s: String) -> Self {
        AudioError::new(s)
    }
}

impl From<&str> for AudioError {
    fn from(s: &str) -> Self {
        AudioError::new(s)
    }
}

/// Result type for audio operations
pub type AudioResult<T> = Result<T, AudioError>;

/// Play state for audio sources
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

/// Audio source properties that can be modified during playback
#[derive(Clone, Debug)]
pub struct AudioProperties {
    pub volume: f32,
    pub pan: f32,
    pub pitch: f32,
    pub looping: bool,
}

impl Default for AudioProperties {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
            looping: false,
        }
    }
}

// Constants - will be used in Stage A2/A3 implementation
#[cfg(any(feature = "audio", feature = "graphics"))]
#[allow(dead_code)]
pub(crate) const OUTPUT_SAMPLE_RATE: u32 = 44_100;
#[cfg(any(feature = "audio", feature = "graphics"))]
#[allow(dead_code)]
pub(crate) const OUTPUT_CHANNELS: u32 = 2;

#[cfg(not(any(feature = "audio", feature = "graphics")))]
const ERR_NO_AUDIO: &str = "Soond isnae available - build wi' --features audio or --features graphics";

// ============================================================================
// Public API: register_audio_functions
// ============================================================================

/// Register all audio-related builtin functions in the interpreter environment.
///
/// This is the main entry point for interpreter integration.
#[cfg(any(feature = "audio", feature = "graphics"))]
pub fn register_audio_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    // Import the actual implementation module
    use backend::register_builtin_functions;
    register_builtin_functions(globals);

    // Register MIDI functions
    midi::register_midi_functions(globals);
}

/// Stub implementation when no audio backend is available
#[cfg(not(any(feature = "audio", feature = "graphics")))]
pub fn register_audio_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    use crate::value::{NativeFunction, Value};

    fn define_stub(globals: &Rc<RefCell<crate::value::Environment>>, name: &str, arity: usize) {
        globals.borrow_mut().define(
            name.to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, |_args| {
                Err(ERR_NO_AUDIO.to_string())
            }))),
        );
    }

    let stubs = [
        ("soond_stairt", 0),
        ("soond_steek", 0),
        ("soond_wheesht", 1),
        ("soond_luid", 1),
        ("soond_hou_luid", 0),
        ("soond_haud_gang", 0),
        ("soond_lade", 1),
        ("soond_spiel", 1),
        ("soond_haud", 1),
        ("soond_gae_on", 1),
        ("soond_stap", 1),
        ("soond_unlade", 1),
        ("soond_is_spielin", 1),
        ("soond_pit_luid", 2),
        ("soond_pit_pan", 2),
        ("soond_pit_tune", 2),
        ("soond_pit_rin_roond", 2),
        ("soond_ready", 1),
        ("muisic_lade", 1),
        ("muisic_spiel", 1),
        ("muisic_haud", 1),
        ("muisic_gae_on", 1),
        ("muisic_stap", 1),
        ("muisic_unlade", 1),
        ("muisic_is_spielin", 1),
        ("muisic_loup", 2),
        ("muisic_hou_lang", 1),
        ("muisic_whaur", 1),
        ("muisic_pit_luid", 2),
        ("muisic_pit_pan", 2),
        ("muisic_pit_tune", 2),
        ("muisic_pit_rin_roond", 2),
        ("midi_lade", 2),
        ("midi_spiel", 1),
        ("midi_haud", 1),
        ("midi_gae_on", 1),
        ("midi_stap", 1),
        ("midi_unlade", 1),
        ("midi_is_spielin", 1),
        ("midi_loup", 2),
        ("midi_hou_lang", 1),
        ("midi_whaur", 1),
        ("midi_pit_luid", 2),
        ("midi_pit_pan", 2),
        ("midi_pit_rin_roond", 2),
    ];

    for (name, arity) in stubs {
        define_stub(globals, name, arity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_error_display() {
        let err = AudioError::new("test error");
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn audio_error_from_string() {
        let err: AudioError = "test".to_string().into();
        assert_eq!(err.message, "test");
    }

    #[test]
    fn audio_error_from_str() {
        let err: AudioError = "test".into();
        assert_eq!(err.message, "test");
    }

    #[test]
    fn play_state_equality() {
        assert_eq!(PlayState::Stopped, PlayState::Stopped);
        assert_ne!(PlayState::Playing, PlayState::Paused);
    }

    #[test]
    fn audio_properties_default() {
        let props = AudioProperties::default();
        assert_eq!(props.volume, 1.0);
        assert_eq!(props.pan, 0.0);
        assert_eq!(props.pitch, 1.0);
        assert!(!props.looping);
    }
}
