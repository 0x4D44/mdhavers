//! raylib audio backend
//!
//! This backend is used when the `graphics` feature is enabled.
//! It uses raylib's built-in audio system (which includes miniaudio internally).
//! This avoids symbol conflicts when graphics and audio are used together.

use std::cell::RefCell;
use std::rc::Rc;

use super::{AudioError, AudioProperties, AudioResult, PlayState};

/// Audio engine using raylib
///
/// Note: raylib audio is global and managed alongside the graphics window.
/// This struct coordinates with the graphics module for proper initialization.
pub struct AudioEngine {
    initialized: bool,
}

impl AudioEngine {
    /// Create a new audio engine
    ///
    /// Note: raylib audio requires the graphics window to be initialized first.
    pub fn new() -> AudioResult<Self> {
        // Stub - will be implemented in Stage A3
        Ok(Self { initialized: false })
    }

    /// Start the audio device
    pub fn start(&mut self) -> AudioResult<()> {
        self.initialized = true;
        Ok(())
    }

    /// Stop the audio device
    pub fn stop(&mut self) -> AudioResult<()> {
        self.initialized = false;
        Ok(())
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&mut self, _volume: f32) {
        // Stub - will call raylib::ffi::SetMasterVolume
    }

    /// Get master volume
    pub fn master_volume(&self) -> f32 {
        1.0
    }

    /// Mute/unmute all audio
    pub fn set_muted(&mut self, _muted: bool) {
        // Stub
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        false
    }
}

/// Sound handle (short audio clips)
///
/// Wraps raylib::ffi::Sound for short sound effects.
pub struct Sound {
    // Will hold raylib::ffi::Sound
    _marker: std::marker::PhantomData<()>,
}

impl Sound {
    /// Load a sound from file
    pub fn load(_path: &str) -> AudioResult<Self> {
        // Stub - will use raylib::ffi::LoadSound
        Err(AudioError::new("Not implemented yet"))
    }

    /// Play the sound
    pub fn play(&mut self) -> AudioResult<()> {
        Ok(())
    }

    /// Pause playback
    pub fn pause(&mut self) {
        // raylib doesn't have pause for Sound, only for Music
    }

    /// Resume playback
    pub fn resume(&mut self) {
        // raylib doesn't have resume for Sound
    }

    /// Stop playback
    pub fn stop(&mut self) {
        // Stub - will call raylib::ffi::StopSound
    }

    /// Get current play state
    pub fn state(&self) -> PlayState {
        PlayState::Stopped
    }

    /// Check if sound is playing
    pub fn is_playing(&self) -> bool {
        // Stub - will call raylib::ffi::IsSoundPlaying
        false
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, _volume: f32) {
        // Stub - will call raylib::ffi::SetSoundVolume
    }

    /// Set pan (-1.0 left to 1.0 right)
    pub fn set_pan(&mut self, _pan: f32) {
        // Stub - will call raylib::ffi::SetSoundPan
    }

    /// Set pitch (1.0 = normal)
    pub fn set_pitch(&mut self, _pitch: f32) {
        // Stub - will call raylib::ffi::SetSoundPitch
    }

    /// Set looping
    pub fn set_looping(&mut self, _looping: bool) {
        // Note: raylib Sound doesn't have native looping, we'd need to handle it
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        // Stub - will call raylib::ffi::UnloadSound
    }
}

/// Music handle (streaming audio)
///
/// Wraps raylib::ffi::Music for streaming playback of long audio files.
pub struct Music {
    // Will hold raylib::ffi::Music
    _marker: std::marker::PhantomData<()>,
}

impl Music {
    /// Load music from file (streaming)
    pub fn load(_path: &str) -> AudioResult<Self> {
        // Stub - will use raylib::ffi::LoadMusicStream
        Err(AudioError::new("Not implemented yet"))
    }

    /// Play the music
    pub fn play(&mut self) -> AudioResult<()> {
        Ok(())
    }

    /// Pause playback
    pub fn pause(&mut self) {
        // Stub - will call raylib::ffi::PauseMusicStream
    }

    /// Resume playback
    pub fn resume(&mut self) {
        // Stub - will call raylib::ffi::ResumeMusicStream
    }

    /// Stop playback
    pub fn stop(&mut self) {
        // Stub - will call raylib::ffi::StopMusicStream
    }

    /// Get current play state
    pub fn state(&self) -> PlayState {
        PlayState::Stopped
    }

    /// Check if music is playing
    pub fn is_playing(&self) -> bool {
        // Stub - will call raylib::ffi::IsMusicStreamPlaying
        false
    }

    /// Seek to position in seconds
    pub fn seek(&mut self, _seconds: f64) -> AudioResult<()> {
        // Stub - will call raylib::ffi::SeekMusicStream
        Ok(())
    }

    /// Get current position in seconds
    pub fn position(&self) -> f64 {
        // Stub - will call raylib::ffi::GetMusicTimePlayed
        0.0
    }

    /// Get total duration in seconds
    pub fn duration(&self) -> f64 {
        // Stub - will call raylib::ffi::GetMusicTimeLength
        0.0
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, _volume: f32) {
        // Stub - will call raylib::ffi::SetMusicVolume
    }

    /// Set pan (-1.0 left to 1.0 right)
    pub fn set_pan(&mut self, _pan: f32) {
        // Stub - will call raylib::ffi::SetMusicPan
    }

    /// Set pitch (1.0 = normal)
    pub fn set_pitch(&mut self, _pitch: f32) {
        // Stub - will call raylib::ffi::SetMusicPitch
    }

    /// Set looping
    pub fn set_looping(&mut self, _looping: bool) {
        // Note: raylib music looping is handled via Music.looping field
    }

    /// Update music stream (must be called in game loop)
    pub fn update(&mut self) {
        // Stub - will call raylib::ffi::UpdateMusicStream
    }
}

impl Drop for Music {
    fn drop(&mut self) {
        // Stub - will call raylib::ffi::UnloadMusicStream
    }
}

/// Register sound and music builtin functions for the interpreter
pub fn register_builtin_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    use crate::value::{NativeFunction, Value};

    // Helper to define native functions
    fn define_native<F>(
        globals: &Rc<RefCell<crate::value::Environment>>,
        name: &str,
        arity: usize,
        func: F,
    ) where
        F: Fn(Vec<Value>) -> Result<Value, String> + 'static,
    {
        globals.borrow_mut().define(
            name.to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, func))),
        );
    }

    // Audio engine functions (stubs for now - raylib audio is initialized with graphics)
    define_native(globals, "soond_stairt", 0, |_args| {
        // raylib audio is initialized with InitAudioDevice() or automatically with InitWindow
        Ok(Value::Nil)
    });

    define_native(globals, "soond_steek", 0, |_args| {
        // raylib audio is closed with CloseAudioDevice() or automatically with CloseWindow
        Ok(Value::Nil)
    });

    define_native(globals, "soond_wheesht", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_luid", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_hou_luid", 0, |_args| {
        Ok(Value::Float(1.0))
    });

    define_native(globals, "soond_haud_gang", 0, |_args| {
        Ok(Value::Nil)
    });

    // Sound functions (stubs)
    define_native(globals, "soond_lade", 1, |_args| {
        Ok(Value::Integer(-1))
    });

    define_native(globals, "soond_spiel", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_haud", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_gae_on", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_stap", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_unlade", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_is_spielin", 1, |_args| {
        Ok(Value::Bool(false))
    });

    define_native(globals, "soond_pit_luid", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_pit_pan", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_pit_tune", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_pit_rin_roond", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "soond_ready", 1, |_args| {
        Ok(Value::Bool(false))
    });

    // Music functions (stubs)
    define_native(globals, "muisic_lade", 1, |_args| {
        Ok(Value::Integer(-1))
    });

    define_native(globals, "muisic_spiel", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_haud", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_gae_on", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_stap", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_unlade", 1, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_is_spielin", 1, |_args| {
        Ok(Value::Bool(false))
    });

    define_native(globals, "muisic_loup", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_hou_lang", 1, |_args| {
        Ok(Value::Float(0.0))
    });

    define_native(globals, "muisic_whaur", 1, |_args| {
        Ok(Value::Float(0.0))
    });

    define_native(globals, "muisic_pit_luid", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_pit_pan", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_pit_tune", 2, |_args| {
        Ok(Value::Nil)
    });

    define_native(globals, "muisic_pit_rin_roond", 2, |_args| {
        Ok(Value::Nil)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_engine_new() {
        let engine = AudioEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn audio_engine_start_stop() {
        let mut engine = AudioEngine::new().unwrap();
        assert!(engine.start().is_ok());
        assert!(engine.stop().is_ok());
    }

    #[test]
    fn sound_load_returns_error() {
        let result = Sound::load("test.wav");
        assert!(result.is_err());
    }

    #[test]
    fn music_load_returns_error() {
        let result = Music::load("test.mp3");
        assert!(result.is_err());
    }
}
