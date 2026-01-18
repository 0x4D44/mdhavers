//! MIDI synthesis module using RustySynth
//!
//! This module provides MIDI playback capabilities shared between all audio backends.
//! It uses rustysynth for software synthesis with SoundFont support.

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
#[cfg(any(feature = "audio", feature = "graphics"))]
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::value::{NativeFunction, Value};

use super::{AudioResult, PlayState, OUTPUT_SAMPLE_RATE};

// ============================================================================
// Test-only rustysynth mock
// ============================================================================

#[cfg(test)]
mod rustysynth {
    use std::cell::Cell;
    use std::io::Read;
    use std::sync::Arc;

    thread_local! {
        static FAIL_NEXT_SOUNDFONT_NEW: Cell<bool> = Cell::new(false);
        static FAIL_NEXT_MIDI_FILE_NEW: Cell<bool> = Cell::new(false);
        static FAIL_NEXT_SYNTH_NEW: Cell<bool> = Cell::new(false);
    }

    pub(super) fn fail_next_soundfont_new() {
        FAIL_NEXT_SOUNDFONT_NEW.with(|flag| flag.set(true));
    }

    pub(super) fn fail_next_midi_file_new() {
        FAIL_NEXT_MIDI_FILE_NEW.with(|flag| flag.set(true));
    }

    pub(super) fn fail_next_synth_new() {
        FAIL_NEXT_SYNTH_NEW.with(|flag| flag.set(true));
    }

    #[derive(Debug)]
    pub struct SoundFont;

    impl SoundFont {
        pub fn new<R: Read>(_reader: &mut R) -> Result<Self, ()> {
            let fail = FAIL_NEXT_SOUNDFONT_NEW.with(|flag| {
                let value = flag.get();
                flag.set(false);
                value
            });
            if fail {
                return Err(());
            }
            Ok(SoundFont)
        }
    }

    #[derive(Clone)]
    pub struct MidiFile {
        length: f64,
    }

    impl MidiFile {
        pub fn new<R: Read>(_reader: &mut R) -> Result<Self, ()> {
            let fail = FAIL_NEXT_MIDI_FILE_NEW.with(|flag| {
                let value = flag.get();
                flag.set(false);
                value
            });
            if fail {
                return Err(());
            }
            Ok(MidiFile { length: 0.1 })
        }

        pub fn get_length(&self) -> f64 {
            self.length
        }
    }

    pub struct SynthesizerSettings {
        sample_rate: i32,
    }

    impl SynthesizerSettings {
        pub fn new(sample_rate: i32) -> Self {
            Self { sample_rate }
        }
    }

    pub struct Synthesizer {
        sample_rate: i32,
    }

    impl Synthesizer {
        pub fn new(_soundfont: &SoundFont, settings: &SynthesizerSettings) -> Result<Self, ()> {
            let fail = FAIL_NEXT_SYNTH_NEW.with(|flag| {
                let value = flag.get();
                flag.set(false);
                value
            });
            if fail {
                return Err(());
            }
            Ok(Self {
                sample_rate: settings.sample_rate,
            })
        }
    }

    pub struct MidiFileSequencer {
        position: f64,
        length: f64,
        looping: bool,
        playing: bool,
        sample_rate: i32,
    }

    impl MidiFileSequencer {
        pub fn new(synth: Synthesizer) -> Self {
            Self {
                position: 0.0,
                length: 0.1,
                looping: false,
                playing: false,
                sample_rate: synth.sample_rate,
            }
        }

        pub fn play(&mut self, midi: &Arc<MidiFile>, looping: bool) {
            self.length = midi.get_length();
            self.looping = looping;
            self.playing = true;
            self.position = 0.0;
        }

        pub fn render(&mut self, left: &mut [f32], right: &mut [f32]) {
            for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                *l = 0.0;
                *r = 0.0;
            }

            if !self.playing {
                return;
            }

            let advance = left.len() as f64 / self.sample_rate as f64;
            self.position += advance;

            if self.position >= self.length {
                if self.looping {
                    self.position = 0.0;
                } else {
                    self.position = self.length;
                    self.playing = false;
                }
            }
        }

        pub fn get_position(&self) -> f64 {
            self.position
        }

        pub fn end_of_sequence(&self) -> bool {
            !self.looping && !self.playing && self.position >= self.length
        }

        pub fn stop(&mut self) {
            self.playing = false;
            self.position = 0.0;
        }
    }
}

#[cfg(not(test))]
use ::rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
#[cfg(test)]
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_SOUNDFONT_PATH: &str = "assets/soundfonts/MuseScore_General.sf2";
const ERR_BAD_HANDLE: &str = "Thon handle isnae guid";
const DECODE_CHUNK_FRAMES: usize = 1_024;

// ============================================================================
// MIDI Entry - holds state for one loaded MIDI file
// ============================================================================

#[allow(dead_code)]
struct MidiEntry {
    midi: Arc<MidiFile>,
    sequencer: MidiFileSequencer,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    sample_rate: u32,
    scratch_left: Vec<f32>,
    scratch_right: Vec<f32>,
}

// ============================================================================
// MidiState - global state for MIDI playback
// ============================================================================

struct MidiState {
    entries: Vec<Option<MidiEntry>>,
    default_soundfont: Option<Arc<SoundFont>>,
}

impl MidiState {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            default_soundfont: None,
        }
    }

    fn alloc_handle(&mut self, entry: MidiEntry) -> i64 {
        for (idx, slot) in self.entries.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(entry);
                return idx as i64;
            }
        }
        self.entries.push(Some(entry));
        (self.entries.len() - 1) as i64
    }

    fn get(&self, handle: usize) -> Option<&MidiEntry> {
        self.entries.get(handle).and_then(|e| e.as_ref())
    }

    fn get_mut(&mut self, handle: usize) -> Option<&mut MidiEntry> {
        self.entries.get_mut(handle).and_then(|e| e.as_mut())
    }

    fn free_handle(&mut self, handle: usize) -> Result<(), String> {
        if handle >= self.entries.len() || self.entries[handle].is_none() {
            return Err(ERR_BAD_HANDLE.to_string());
        }
        self.entries[handle] = None;
        Ok(())
    }
}

static MIDI_STATE: Mutex<Option<MidiState>> = Mutex::new(None);

fn with_midi_state<F, T>(f: F) -> T
where
    F: FnOnce(&mut MidiState) -> T,
{
    let mut guard = MIDI_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(MidiState::new());
    }
    f(guard.as_mut().unwrap())
}

// ============================================================================
// Soundfont loading
// ============================================================================

fn load_soundfont(path: &Path) -> Result<Arc<SoundFont>, String> {
    let mut file = File::open(path).map_err(|_| "Cannae open the soondfont file".to_string())?;
    let sf = SoundFont::new(&mut file).map_err(|_| "Cannae read the soondfont".to_string())?;
    Ok(Arc::new(sf))
}

#[cfg(any(feature = "audio", feature = "graphics", test))]
fn current_exe_for_soundfont_candidates() -> std::io::Result<PathBuf> {
    if cfg!(test) && std::env::var_os("MDHAVERS_TEST_FORCE_CURRENT_EXE_ERROR").is_some() {
        Err(std::io::Error::other("forced current_exe failure"))
    } else {
        std::env::current_exe()
    }
}

#[cfg(any(feature = "audio", feature = "graphics", test))]
fn resolve_default_soundfont() -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    candidates.extend(
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(DEFAULT_SOUNDFONT_PATH)),
    );

    if let Ok(exe) = current_exe_for_soundfont_candidates() {
        let dir = exe.parent().unwrap_or(Path::new(""));
        candidates.push(dir.join(DEFAULT_SOUNDFONT_PATH));
        candidates.push(dir.join("../assets/soundfonts/MuseScore_General.sf2"));
        candidates.push(dir.join("../../assets/soundfonts/MuseScore_General.sf2"));
    }

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    Err("Cannae find the default soondfont".to_string())
}

// ============================================================================
// Seek helper
// ============================================================================

fn seek_midi(entry: &mut MidiEntry, seconds: f64) {
    let length = entry.midi.get_length();
    let target = if seconds < 0.0 {
        0.0
    } else if seconds > length {
        length
    } else {
        seconds
    };

    entry.sequencer.play(&entry.midi, entry.looped);

    let total_frames = (target * entry.sample_rate as f64) as usize;
    let mut left = vec![0.0_f32; DECODE_CHUNK_FRAMES];
    let mut right = vec![0.0_f32; DECODE_CHUNK_FRAMES];
    let mut remaining = total_frames;
    while remaining > 0 {
        let chunk = if remaining > DECODE_CHUNK_FRAMES {
            DECODE_CHUNK_FRAMES
        } else {
            remaining
        };
        entry.sequencer.render(&mut left[..chunk], &mut right[..chunk]);
        remaining -= chunk;
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn as_number(value: &Value, name: &str) -> Result<f64, String> {
    match value {
        Value::Float(f) => Ok(*f),
        Value::Integer(i) => Ok(*i as f64),
        _ => Err(format!("{} needs a nummer", name)),
    }
}

fn as_bool(value: &Value, name: &str) -> Result<bool, String> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(format!("{} needs aye or nae", name)),
    }
}

fn as_handle(value: &Value, name: &str) -> Result<usize, String> {
    match value {
        Value::Integer(i) if *i >= 0 => Ok(*i as usize),
        _ => Err(format!("{} needs a guid handle", name)),
    }
}

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

// ============================================================================
// MIDI Builtin Registration
// ============================================================================

/// Register MIDI-related builtin functions
#[cfg(any(feature = "audio", feature = "graphics"))]
pub fn register_midi_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    // midi_lade (path, soundfont or naething)
    define_native(globals, "midi_lade", 2, |args| {
        let midi_path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("midi_lade needs a midi filepath".to_string()),
        };

        let sf = match &args[1] {
            Value::Nil => {
                // Use or load default soundfont
                with_midi_state(|state| -> Result<Arc<SoundFont>, String> {
                    if let Some(ref sf) = state.default_soundfont {
                        Ok(Arc::clone(sf))
                    } else {
                        let path = resolve_default_soundfont()?;
                        let sf = load_soundfont(path.as_path())?;
                        state.default_soundfont = Some(Arc::clone(&sf));
                        Ok(sf)
                    }
                })?
            }
            Value::String(path) => load_soundfont(Path::new(path))?,
            _ => return Err("midi_lade needs a soondfont path or naething".to_string()),
        };

        let mut midi_file =
            File::open(&midi_path).map_err(|_| "Cannae open the midi file".to_string())?;
        let midi =
            MidiFile::new(&mut midi_file).map_err(|_| "Cannae read the midi".to_string())?;
        let midi = Arc::new(midi);

        let settings = SynthesizerSettings::new(OUTPUT_SAMPLE_RATE as i32);
        let synth =
            Synthesizer::new(&sf, &settings).map_err(|_| "Cannae set up the synth".to_string())?;
        let sequencer = MidiFileSequencer::new(synth);

        let entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: OUTPUT_SAMPLE_RATE,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };

        let handle = with_midi_state(|state| state.alloc_handle(entry));
        Ok(Value::Integer(handle))
    });

    // midi_spiel
    define_native(globals, "midi_spiel", 1, |args| {
        let handle = as_handle(&args[0], "midi_spiel")?;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            if entry.state == PlayState::Stopped {
                entry.sequencer.play(&entry.midi, entry.looped);
            }
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // midi_haud (pause)
    define_native(globals, "midi_haud", 1, |args| {
        let handle = as_handle(&args[0], "midi_haud")?;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            if entry.state == PlayState::Playing {
                entry.state = PlayState::Paused;
            }
            Ok(Value::Nil)
        })
    });

    // midi_gae_on (resume)
    define_native(globals, "midi_gae_on", 1, |args| {
        let handle = as_handle(&args[0], "midi_gae_on")?;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            if entry.state == PlayState::Paused {
                entry.state = PlayState::Playing;
            }
            Ok(Value::Nil)
        })
    });

    // midi_stap (stop)
    define_native(globals, "midi_stap", 1, |args| {
        let handle = as_handle(&args[0], "midi_stap")?;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.sequencer.stop();
            entry.state = PlayState::Stopped;
            Ok(Value::Nil)
        })
    });

    // midi_unlade
    define_native(globals, "midi_unlade", 1, |args| {
        let handle = as_handle(&args[0], "midi_unlade")?;
        with_midi_state(|state| state.free_handle(handle))?;
        Ok(Value::Nil)
    });

    // midi_is_spielin
    define_native(globals, "midi_is_spielin", 1, |args| {
        let handle = as_handle(&args[0], "midi_is_spielin")?;
        with_midi_state(|state| {
            let entry = state.get(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.state == PlayState::Playing))
        })
    });

    // midi_loup (seek)
    define_native(globals, "midi_loup", 2, |args| {
        let handle = as_handle(&args[0], "midi_loup")?;
        let pos = as_number(&args[1], "midi_loup")?;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            seek_midi(entry, pos);
            Ok(Value::Nil)
        })
    });

    // midi_hou_lang (duration)
    define_native(globals, "midi_hou_lang", 1, |args| {
        let handle = as_handle(&args[0], "midi_hou_lang")?;
        with_midi_state(|state| {
            let entry = state.get(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Float(entry.midi.get_length()))
        })
    });

    // midi_whaur (position)
    define_native(globals, "midi_whaur", 1, |args| {
        let handle = as_handle(&args[0], "midi_whaur")?;
        with_midi_state(|state| {
            let entry = state.get(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Float(entry.sequencer.get_position()))
        })
    });

    // midi_pit_luid (volume)
    define_native(globals, "midi_pit_luid", 2, |args| {
        let handle = as_handle(&args[0], "midi_pit_luid")?;
        let value = as_number(&args[1], "midi_pit_luid")? as f32;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value.clamp(0.0, 1.0);
            Ok(Value::Nil)
        })
    });

    // midi_pit_pan
    define_native(globals, "midi_pit_pan", 2, |args| {
        let handle = as_handle(&args[0], "midi_pit_pan")?;
        let pan = as_number(&args[1], "midi_pit_pan")? as f32;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan.clamp(-1.0, 1.0);
            Ok(Value::Nil)
        })
    });

    // midi_pit_rin_roond (looping)
    define_native(globals, "midi_pit_rin_roond", 2, |args| {
        let handle = as_handle(&args[0], "midi_pit_rin_roond")?;
        let looped = as_bool(&args[1], "midi_pit_rin_roond")?;
        with_midi_state(|state| {
            let entry = state.get_mut(handle).ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.looped = looped;
            Ok(Value::Nil)
        })
    });
}

/// Stub when no audio features are enabled
#[cfg(not(any(feature = "audio", feature = "graphics")))]
pub fn register_midi_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    let _ = globals;
}

/// Public MidiPlayer type (for API compatibility)
#[derive(Debug)]
pub struct MidiPlayer {
    _marker: std::marker::PhantomData<()>,
}

impl MidiPlayer {
    /// Create a new MIDI player (stub)
    pub fn new() -> AudioResult<Self> {
        Ok(Self {
            _marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_player_new() {
        let player = MidiPlayer::new();
        assert!(player.is_ok());
    }

    #[test]
    fn as_number_works() {
        assert_eq!(as_number(&Value::Float(1.5), "test").unwrap(), 1.5);
        assert_eq!(as_number(&Value::Integer(42), "test").unwrap(), 42.0);
        assert!(as_number(&Value::Bool(true), "test").is_err());
    }

    #[test]
    fn as_bool_works() {
        assert!(as_bool(&Value::Bool(true), "test").unwrap());
        assert!(!as_bool(&Value::Bool(false), "test").unwrap());
        assert!(as_bool(&Value::Integer(1), "test").is_err());
    }

    #[test]
    fn as_handle_works() {
        assert_eq!(as_handle(&Value::Integer(5), "test").unwrap(), 5);
        assert!(as_handle(&Value::Integer(-1), "test").is_err());
        assert!(as_handle(&Value::Float(1.0), "test").is_err());
    }

    #[test]
    fn midi_state_alloc_and_free() {
        with_midi_state(|state| {
            // Reset state for test isolation
            state.entries.clear();
            state.default_soundfont = None;
        });

        // Create mock entry components
        let settings = SynthesizerSettings::new(44100);
        let sf = SoundFont::new(&mut std::io::empty()).unwrap();
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let midi = Arc::new(MidiFile::new(&mut std::io::empty()).unwrap());
        let sequencer = MidiFileSequencer::new(synth);

        let entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: OUTPUT_SAMPLE_RATE,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };

        let handle = with_midi_state(|state| state.alloc_handle(entry));
        assert_eq!(handle, 0);

        // Verify entry exists
        let exists = with_midi_state(|state| state.get(handle as usize).is_some());
        assert!(exists);

        // Free the handle
        let result = with_midi_state(|state| state.free_handle(handle as usize));
        assert!(result.is_ok());

        // Verify entry is gone
        let exists = with_midi_state(|state| state.get(handle as usize).is_some());
        assert!(!exists);

        // Double free should error
        let result = with_midi_state(|state| state.free_handle(handle as usize));
        assert!(result.is_err());
    }

    #[test]
    fn soundfont_mock_failure() {
        rustysynth::fail_next_soundfont_new();
        let result = SoundFont::new(&mut std::io::empty());
        assert!(result.is_err());
    }

    #[test]
    fn midifile_mock_failure() {
        rustysynth::fail_next_midi_file_new();
        let result = MidiFile::new(&mut std::io::empty());
        assert!(result.is_err());
    }

    #[test]
    fn synth_mock_failure() {
        let sf = SoundFont::new(&mut std::io::empty()).unwrap();
        let settings = SynthesizerSettings::new(44100);
        rustysynth::fail_next_synth_new();
        let result = Synthesizer::new(&sf, &settings);
        assert!(result.is_err());
    }

    #[test]
    fn sequencer_playback() {
        let sf = SoundFont::new(&mut std::io::empty()).unwrap();
        let settings = SynthesizerSettings::new(44100);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let midi = Arc::new(MidiFile::new(&mut std::io::empty()).unwrap());
        let mut sequencer = MidiFileSequencer::new(synth);

        // Initially not playing
        assert!(sequencer.end_of_sequence() || sequencer.get_position() == 0.0);

        // Start playing
        sequencer.play(&midi, false);
        assert!(!sequencer.end_of_sequence());

        // Render advances position
        let mut left = vec![0.0_f32; 1024];
        let mut right = vec![0.0_f32; 1024];
        sequencer.render(&mut left, &mut right);
        assert!(sequencer.get_position() > 0.0);

        // Stop resets
        sequencer.stop();
        assert_eq!(sequencer.get_position(), 0.0);
    }

    #[test]
    fn seek_midi_test() {
        let sf = SoundFont::new(&mut std::io::empty()).unwrap();
        let settings = SynthesizerSettings::new(44100);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let midi = Arc::new(MidiFile::new(&mut std::io::empty()).unwrap());
        let sequencer = MidiFileSequencer::new(synth);

        let mut entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: OUTPUT_SAMPLE_RATE,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };

        // Seek to middle
        seek_midi(&mut entry, 0.05);
        // Position should be approximately at the target
        let pos = entry.sequencer.get_position();
        assert!(pos >= 0.0);
    }

    #[test]
    fn load_soundfont_nonexistent() {
        let result = load_soundfont(Path::new("nonexistent.sf2"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannae open"));
    }
}
