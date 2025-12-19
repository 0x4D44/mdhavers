//! Audio module for mdhavers using raylib + RustySynth
//!
//! Provides Scots-only audio API for sounds, music, and MIDI.

// Test-only backend shims so we can exercise audio logic without a real
// audio device or external libraries when the audio feature is off.
#[cfg(all(test, not(feature = "audio")))]
mod raylib {
    pub mod prelude {
        use std::cell::Cell;
        use std::marker::PhantomData;
        use std::path::Path;

        #[derive(Debug)]
        pub struct RaylibAudio {
            master_volume: Cell<f32>,
        }

        #[derive(Debug, Clone, Copy)]
        pub struct RaylibAudioInitError;

        impl RaylibAudio {
            pub fn init_audio_device() -> Result<Self, RaylibAudioInitError> {
                Ok(Self {
                    master_volume: Cell::new(1.0),
                })
            }

            pub fn set_master_volume(&self, value: f32) {
                self.master_volume.set(value);
            }

            pub fn new_sound(&self, path: &str) -> Result<Sound<'static>, RaylibAudioInitError> {
                if Path::new(path).exists() {
                    Ok(Sound::new())
                } else {
                    Err(RaylibAudioInitError)
                }
            }

            pub fn new_music(&self, path: &str) -> Result<Music<'static>, RaylibAudioInitError> {
                if Path::new(path).exists() {
                    Ok(Music::new())
                } else {
                    Err(RaylibAudioInitError)
                }
            }

            pub fn new_audio_stream(
                &self,
                _sample_rate: u32,
                _sample_size: u32,
                _channels: u32,
            ) -> AudioStream<'static> {
                AudioStream::new()
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum SimpleState {
            Stopped,
            Playing,
            Paused,
        }

        pub struct Sound<'aud> {
            state: Cell<SimpleState>,
            play_checks: Cell<u8>,
            volume: Cell<f32>,
            pan: Cell<f32>,
            pitch: Cell<f32>,
            _marker: PhantomData<&'aud ()>,
        }

        impl<'aud> Sound<'aud> {
            fn new() -> Sound<'static> {
                Sound {
                    state: Cell::new(SimpleState::Stopped),
                    play_checks: Cell::new(0),
                    volume: Cell::new(1.0),
                    pan: Cell::new(0.0),
                    pitch: Cell::new(1.0),
                    _marker: PhantomData,
                }
            }

            pub fn play(&self) {
                self.state.set(SimpleState::Playing);
                self.play_checks.set(2);
            }

            pub fn pause(&self) {
                self.state.set(SimpleState::Paused);
            }

            pub fn resume(&self) {
                self.state.set(SimpleState::Playing);
            }

            pub fn stop(&self) {
                self.state.set(SimpleState::Stopped);
            }

            pub fn is_playing(&self) -> bool {
                if self.state.get() != SimpleState::Playing {
                    return false;
                }
                let remaining = self.play_checks.get();
                if remaining > 0 {
                    self.play_checks.set(remaining - 1);
                    true
                } else {
                    false
                }
            }

            pub fn set_volume(&self, value: f32) {
                self.volume.set(value);
            }

            pub fn set_pan(&self, value: f32) {
                self.pan.set(value);
            }

            pub fn set_pitch(&self, value: f32) {
                self.pitch.set(value);
            }
        }

        pub struct Music<'aud> {
            playing: Cell<bool>,
            time_played: Cell<f32>,
            length: f32,
            volume: Cell<f32>,
            pan: Cell<f32>,
            pitch: Cell<f32>,
            _marker: PhantomData<&'aud ()>,
        }

        impl<'aud> Music<'aud> {
            fn new() -> Music<'static> {
                Music {
                    playing: Cell::new(false),
                    time_played: Cell::new(0.0),
                    length: 1.0,
                    volume: Cell::new(1.0),
                    pan: Cell::new(0.0),
                    pitch: Cell::new(1.0),
                    _marker: PhantomData,
                }
            }

            pub fn play_stream(&self) {
                self.playing.set(true);
            }

            pub fn update_stream(&self) {
                if self.playing.get() {
                    let next = self.time_played.get() + 0.6;
                    self.time_played.set(next);
                }
            }

            pub fn stop_stream(&self) {
                self.playing.set(false);
                self.time_played.set(0.0);
            }

            pub fn pause_stream(&self) {
                self.playing.set(false);
            }

            pub fn resume_stream(&self) {
                self.playing.set(true);
            }

            pub fn is_stream_playing(&self) -> bool {
                self.playing.get()
            }

            pub fn set_volume(&self, value: f32) {
                self.volume.set(value);
            }

            pub fn set_pitch(&self, value: f32) {
                self.pitch.set(value);
            }

            pub fn get_time_length(&self) -> f32 {
                self.length
            }

            pub fn get_time_played(&self) -> f32 {
                self.time_played.get()
            }

            pub fn seek_stream(&self, position: f32) {
                self.time_played.set(position);
            }

            pub fn set_pan(&self, value: f32) {
                self.pan.set(value);
            }
        }

        pub trait AudioSample {}
        impl AudioSample for u8 {}
        impl AudioSample for i16 {}
        impl AudioSample for f32 {}

        pub struct AudioStream<'aud> {
            playing: Cell<bool>,
            processed: Cell<bool>,
            volume: Cell<f32>,
            pan: Cell<f32>,
            _marker: PhantomData<&'aud ()>,
        }

        impl<'aud> AudioStream<'aud> {
            fn new() -> AudioStream<'static> {
                AudioStream {
                    playing: Cell::new(false),
                    processed: Cell::new(true),
                    volume: Cell::new(1.0),
                    pan: Cell::new(0.0),
                    _marker: PhantomData,
                }
            }

            pub fn update<T: AudioSample>(&mut self, _data: &[T]) {}

            pub fn play(&self) {
                self.playing.set(true);
            }

            pub fn pause(&self) {
                self.playing.set(false);
            }

            pub fn resume(&self) {
                self.playing.set(true);
            }

            pub fn stop(&self) {
                self.playing.set(false);
            }

            pub fn is_playing(&self) -> bool {
                self.playing.get()
            }

            pub fn set_volume(&self, value: f32) {
                self.volume.set(value);
            }

            pub fn is_processed(&self) -> bool {
                self.processed.get()
            }

            pub fn set_pan(&self, value: f32) {
                self.pan.set(value);
            }

            pub fn set_processed_for_test(&self, value: bool) {
                self.processed.set(value);
            }
        }
    }
}

#[cfg(all(test, not(feature = "audio")))]
mod rustysynth {
    use std::io::Read;
    use std::sync::Arc;

    pub struct SoundFont;

    impl SoundFont {
        pub fn new<R: Read>(_reader: &mut R) -> Result<Self, ()> {
            Ok(SoundFont)
        }
    }

    #[derive(Clone)]
    pub struct MidiFile {
        length: f64,
    }

    impl MidiFile {
        pub fn new<R: Read>(_reader: &mut R) -> Result<Self, ()> {
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

#[cfg(any(feature = "audio", test))]
use crate::value::{NativeFunction, Value};
#[cfg(any(feature = "audio", test))]
use raylib::prelude::*;
#[cfg(any(feature = "audio", test))]
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
#[cfg(any(feature = "audio", test))]
use std::cell::RefCell;
#[cfg(any(feature = "audio", test))]
use std::fs::File;
#[cfg(any(feature = "audio", test))]
use std::path::Path;
#[cfg(any(feature = "audio", test))]
use std::path::PathBuf;
#[cfg(any(feature = "audio", test))]
use std::rc::Rc;
#[cfg(any(feature = "audio", test))]
use std::sync::Arc;
#[cfg(not(any(feature = "audio", test)))]
use std::cell::RefCell;
#[cfg(not(any(feature = "audio", test)))]
use std::rc::Rc;

#[cfg(any(feature = "audio", test))]
const DEFAULT_SOUNDFONT_PATH: &str = "assets/soundfonts/MuseScore_General.sf2";
#[cfg(any(feature = "audio", test))]
const MIDI_SAMPLE_RATE: i32 = 44_100;
#[cfg(any(feature = "audio", test))]
const MIDI_STREAM_FRAMES: usize = 1_024;

#[cfg(not(any(feature = "audio", test)))]
const ERR_NO_AUDIO: &str = "Soond isnae available - build wi' --features audio";
#[cfg(any(feature = "audio", test))]
const ERR_NO_DEVICE: &str = "Soond device isnae stairtit";
#[cfg(any(feature = "audio", test))]
const ERR_BAD_HANDLE: &str = "Thon handle isnae guid";

#[cfg(any(feature = "audio", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Stopped,
    Playing,
    Paused,
}

#[cfg(any(feature = "audio", test))]
struct SoundEntry {
    sound: Sound<'static>,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    pitch: f32,
}

#[cfg(any(feature = "audio", test))]
struct MusicEntry {
    music: Music<'static>,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    pitch: f32,
}

#[cfg(any(feature = "audio", test))]
struct MidiEntry {
    midi: Arc<MidiFile>,
    sequencer: MidiFileSequencer,
    stream: AudioStream<'static>,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    sample_rate: i32,
}

#[cfg(any(feature = "audio", test))]
struct AudioState {
    audio: Option<Box<RaylibAudio>>,
    master_volume: f32,
    muted: bool,
    sounds: Vec<Option<SoundEntry>>,
    music: Vec<Option<MusicEntry>>,
    midi: Vec<Option<MidiEntry>>,
    default_soundfont: Option<Arc<SoundFont>>,
}

#[cfg(any(feature = "audio", test))]
impl AudioState {
    fn new() -> Self {
        Self {
            audio: None,
            master_volume: 1.0,
            muted: false,
            sounds: Vec::new(),
            music: Vec::new(),
            midi: Vec::new(),
            default_soundfont: None,
        }
    }

    fn ensure_audio(&mut self) -> Result<&'static RaylibAudio, String> {
        if self.audio.is_none() {
            let audio = RaylibAudio::init_audio_device()
                .map_err(|_| "Cannae stairt the soond device".to_string())?;
            self.audio = Some(Box::new(audio));
            self.master_volume = 1.0;
            self.muted = false;
        }
        let audio_ref = self.audio.as_ref().unwrap().as_ref();
        let audio_static: &'static RaylibAudio = unsafe { std::mem::transmute(audio_ref) };
        Ok(audio_static)
    }

    fn audio_ref(&self) -> Result<&'static RaylibAudio, String> {
        let audio_ref = self.audio.as_ref().ok_or_else(|| ERR_NO_DEVICE.to_string())?;
        let audio_static: &'static RaylibAudio = unsafe { std::mem::transmute(audio_ref.as_ref()) };
        Ok(audio_static)
    }

    fn alloc_handle<T>(slots: &mut Vec<Option<T>>, value: T) -> i64 {
        for (idx, slot) in slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(value);
                return idx as i64;
            }
        }
        slots.push(Some(value));
        (slots.len() - 1) as i64
    }

    fn shutdown(&mut self) {
        self.sounds.clear();
        self.music.clear();
        self.midi.clear();
        self.default_soundfont = None;
        self.audio.take();
    }
}

#[cfg(any(feature = "audio", test))]
#[cfg(feature = "audio")]
fn sound_to_static(sound: Sound<'_>) -> Sound<'static> {
    unsafe { std::mem::transmute(sound) }
}

#[cfg(any(feature = "audio", test))]
#[cfg(not(feature = "audio"))]
fn sound_to_static(sound: Sound<'static>) -> Sound<'static> {
    sound
}

#[cfg(any(feature = "audio", test))]
#[cfg(feature = "audio")]
fn music_to_static(music: Music<'_>) -> Music<'static> {
    unsafe { std::mem::transmute(music) }
}

#[cfg(any(feature = "audio", test))]
#[cfg(not(feature = "audio"))]
fn music_to_static(music: Music<'static>) -> Music<'static> {
    music
}

#[cfg(any(feature = "audio", test))]
#[cfg(feature = "audio")]
fn stream_to_static(stream: AudioStream<'_>) -> AudioStream<'static> {
    unsafe { std::mem::transmute(stream) }
}

#[cfg(any(feature = "audio", test))]
#[cfg(not(feature = "audio"))]
fn stream_to_static(stream: AudioStream<'static>) -> AudioStream<'static> {
    stream
}

#[cfg(any(feature = "audio", test))]
thread_local! {
    static AUDIO_STATE: RefCell<AudioState> = RefCell::new(AudioState::new());
}

#[cfg(any(feature = "audio", test))]
fn as_number(value: &Value, name: &str) -> Result<f64, String> {
    match value {
        Value::Float(f) => Ok(*f),
        Value::Integer(i) => Ok(*i as f64),
        _ => Err(format!("{} needs a nummer", name)),
    }
}

#[cfg(any(feature = "audio", test))]
fn as_bool(value: &Value, name: &str) -> Result<bool, String> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(format!("{} needs aye or nae", name)),
    }
}

#[cfg(any(feature = "audio", test))]
fn as_handle(value: &Value, name: &str) -> Result<usize, String> {
    match value {
        Value::Integer(i) if *i >= 0 => Ok(*i as usize),
        _ => Err(format!("{} needs a guid handle", name)),
    }
}

#[cfg(any(feature = "audio", test))]
fn clamp01(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

#[cfg(any(feature = "audio", test))]
fn pan_to_raylib(value: f32) -> f32 {
    let mapped = (value + 1.0) * 0.5;
    clamp01(mapped)
}

#[cfg(any(feature = "audio", test))]
fn load_soundfont(path: &Path) -> Result<Arc<SoundFont>, String> {
    let mut file = File::open(path).map_err(|_| "Cannae open the soondfont file".to_string())?;
    let sf = SoundFont::new(&mut file).map_err(|_| "Cannae read the soondfont".to_string())?;
    Ok(Arc::new(sf))
}

#[cfg(any(feature = "audio", test))]
fn resolve_default_soundfont() -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(DEFAULT_SOUNDFONT_PATH));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(DEFAULT_SOUNDFONT_PATH));
            candidates.push(dir.join("../assets/soundfonts/MuseScore_General.sf2"));
            candidates.push(dir.join("../../assets/soundfonts/MuseScore_General.sf2"));
        }
    }

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    Err("Cannae find the default soondfont".to_string())
}

#[cfg(any(feature = "audio", test))]
fn prime_midi_stream(entry: &mut MidiEntry) {
    if !entry.stream.is_processed() {
        return;
    }
    let frames = MIDI_STREAM_FRAMES;
    let mut left = vec![0.0_f32; frames];
    let mut right = vec![0.0_f32; frames];
    entry.sequencer.render(&mut left, &mut right);
    let mut interleaved: Vec<f32> = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        interleaved.push(left[i]);
        interleaved.push(right[i]);
    }
    entry.stream.update(&interleaved);
}

#[cfg(any(feature = "audio", test))]
fn seek_midi(entry: &mut MidiEntry, seconds: f64, audio: &'static RaylibAudio) -> Result<(), String> {
    let length = entry.midi.get_length();
    let target = if seconds < 0.0 { 0.0 } else if seconds > length { length } else { seconds };

    entry.sequencer.play(&entry.midi, entry.looped);

    let total_frames = (target * entry.sample_rate as f64) as usize;
    let mut left = vec![0.0_f32; MIDI_STREAM_FRAMES];
    let mut right = vec![0.0_f32; MIDI_STREAM_FRAMES];
    let mut remaining = total_frames;
    while remaining > 0 {
        let chunk = if remaining > MIDI_STREAM_FRAMES {
            MIDI_STREAM_FRAMES
        } else {
            remaining
        };
        entry.sequencer.render(&mut left[..chunk], &mut right[..chunk]);
        remaining -= chunk;
    }

    entry.stream.stop();
    entry.stream = stream_to_static(audio.new_audio_stream(MIDI_SAMPLE_RATE as u32, 32, 2));
    entry.stream.set_volume(entry.volume);
    entry.stream.set_pan(pan_to_raylib(entry.pan));

    match entry.state {
        PlayState::Playing => entry.stream.play(),
        PlayState::Paused => {
            entry.stream.play();
            entry.stream.pause();
        }
        PlayState::Stopped => {}
    }
    Ok(())
}

#[cfg(any(feature = "audio", test))]
fn with_state<F>(func: F) -> Result<Value, String>
where
    F: FnOnce(&mut AudioState) -> Result<Value, String>,
{
    AUDIO_STATE.with(|state| func(&mut state.borrow_mut()))
}

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
pub fn register_audio_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    // soond_stairt
    define_native(globals, "soond_stairt", 0, |_args| {
        with_state(|state| {
            state.ensure_audio()?;
            Ok(Value::Nil)
        })
    });

    // soond_steek
    define_native(globals, "soond_steek", 0, |_args| {
        with_state(|state| {
            state.shutdown();
            Ok(Value::Nil)
        })
    });

    // soond_wheesht
    define_native(globals, "soond_wheesht", 1, |args| {
        with_state(|state| {
            let wheesht = as_bool(&args[0], "soond_wheesht")?;
            let audio = state.ensure_audio()?;
            state.muted = wheesht;
            if wheesht {
                audio.set_master_volume(0.0);
            } else {
                audio.set_master_volume(state.master_volume);
            }
            Ok(Value::Nil)
        })
    });

    // soond_luid
    define_native(globals, "soond_luid", 1, |args| {
        with_state(|state| {
            let mut value = as_number(&args[0], "soond_luid")? as f32;
            value = clamp01(value);
            let audio = state.ensure_audio()?;
            state.master_volume = value;
            if !state.muted {
                audio.set_master_volume(value);
            }
            Ok(Value::Nil)
        })
    });

    // soond_hou_luid
    define_native(globals, "soond_hou_luid", 0, |_args| {
        with_state(|state| Ok(Value::Float(state.master_volume as f64)))
    });

    // soond_haud_gang
    define_native(globals, "soond_haud_gang", 0, |_args| {
        with_state(|state| {
            if state.audio.is_none() {
                return Ok(Value::Nil);
            }

            for slot in state.sounds.iter_mut() {
                let entry = match slot {
                    Some(entry) => entry,
                    None => continue,
                };
                if entry.looped && entry.state == PlayState::Playing && !entry.sound.is_playing() {
                    entry.sound.play();
                }
            }

            for slot in state.music.iter_mut() {
                let entry = match slot {
                    Some(entry) => entry,
                    None => continue,
                };

                if entry.state != PlayState::Playing {
                    continue;
                }

                entry.music.update_stream();
                let length = entry.music.get_time_length();
                let played = entry.music.get_time_played();

                if entry.looped && length > 0.0 && played >= length - 0.01 {
                    entry.music.seek_stream(0.0);
                    entry.music.play_stream();
                } else if !entry.looped && length > 0.0 && played >= length - 0.01 {
                    entry.music.stop_stream();
                    entry.state = PlayState::Stopped;
                }
            }

            for slot in state.midi.iter_mut() {
                let entry = match slot {
                    Some(entry) => entry,
                    None => continue,
                };

                if entry.state != PlayState::Playing {
                    continue;
                }

                if !entry.stream.is_playing() {
                    entry.stream.play();
                }

                if entry.stream.is_processed() {
                    prime_midi_stream(entry);
                }

                if entry.sequencer.end_of_sequence() && !entry.looped {
                    entry.stream.stop();
                    entry.state = PlayState::Stopped;
                }
            }

            Ok(Value::Nil)
        })
    });

    // soond_lade
    define_native(globals, "soond_lade", 1, |args| {
        with_state(|state| {
            let path = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("soond_lade needs a string path".to_string()),
            };
            let audio = state.ensure_audio()?;
            let sound = audio
                .new_sound(&path)
                .map_err(|_| "Cannae lade the soond".to_string())?;
            let sound = sound_to_static(sound);
            sound.set_volume(1.0);
            sound.set_pan(pan_to_raylib(0.0));
            sound.set_pitch(1.0);
            let entry = SoundEntry {
                sound,
                state: PlayState::Stopped,
                looped: false,
                volume: 1.0,
                pan: 0.0,
                pitch: 1.0,
            };
            let handle = AudioState::alloc_handle(&mut state.sounds, entry);
            Ok(Value::Integer(handle))
        })
    });

    // soond_spiel
    define_native(globals, "soond_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_spiel")?;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.sound.play();
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // soond_haud
    define_native(globals, "soond_haud", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_haud")?;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.sound.pause();
            entry.state = PlayState::Paused;
            Ok(Value::Nil)
        })
    });

    // soond_gae_on
    define_native(globals, "soond_gae_on", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_gae_on")?;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.sound.resume();
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // soond_stap
    define_native(globals, "soond_stap", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_stap")?;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.sound.stop();
            entry.state = PlayState::Stopped;
            Ok(Value::Nil)
        })
    });

    // soond_unlade
    define_native(globals, "soond_unlade", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_unlade")?;
            if handle >= state.sounds.len() || state.sounds[handle].is_none() {
                return Err(ERR_BAD_HANDLE.to_string());
            }
            state.sounds[handle] = None;
            Ok(Value::Nil)
        })
    });

    // soond_is_spielin
    define_native(globals, "soond_is_spielin", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_is_spielin")?;
            let entry = state
                .sounds
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.sound.is_playing()))
        })
    });

    // soond_pit_luid
    define_native(globals, "soond_pit_luid", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_luid")?;
            let mut value = as_number(&args[1], "soond_pit_luid")? as f32;
            value = clamp01(value);
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value;
            entry.sound.set_volume(value);
            Ok(Value::Nil)
        })
    });

    // soond_pit_pan
    define_native(globals, "soond_pit_pan", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_pan")?;
            let pan = as_number(&args[1], "soond_pit_pan")? as f32;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan;
            entry.sound.set_pan(pan_to_raylib(pan));
            Ok(Value::Nil)
        })
    });

    // soond_pit_tune
    define_native(globals, "soond_pit_tune", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_tune")?;
            let pitch = as_number(&args[1], "soond_pit_tune")? as f32;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pitch = pitch;
            entry.sound.set_pitch(pitch);
            Ok(Value::Nil)
        })
    });

    // soond_pit_rin_roond
    define_native(globals, "soond_pit_rin_roond", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_rin_roond")?;
            let looped = as_bool(&args[1], "soond_pit_rin_roond")?;
            let entry = state
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.looped = looped;
            Ok(Value::Nil)
        })
    });

    // muisic_lade
    define_native(globals, "muisic_lade", 1, |args| {
        with_state(|state| {
            let path = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("muisic_lade needs a string path".to_string()),
            };
            let audio = state.ensure_audio()?;
            let music = audio
                .new_music(&path)
                .map_err(|_| "Cannae lade the muisic".to_string())?;
            let music = music_to_static(music);
            music.set_volume(1.0);
            music.set_pan(pan_to_raylib(0.0));
            music.set_pitch(1.0);
            let entry = MusicEntry {
                music,
                state: PlayState::Stopped,
                looped: false,
                volume: 1.0,
                pan: 0.0,
                pitch: 1.0,
            };
            let handle = AudioState::alloc_handle(&mut state.music, entry);
            Ok(Value::Integer(handle))
        })
    });

    // muisic_spiel
    define_native(globals, "muisic_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_spiel")?;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            if entry.state == PlayState::Paused {
                entry.music.resume_stream();
            } else {
                entry.music.play_stream();
            }
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // muisic_haud
    define_native(globals, "muisic_haud", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_haud")?;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.music.pause_stream();
            entry.state = PlayState::Paused;
            Ok(Value::Nil)
        })
    });

    // muisic_gae_on
    define_native(globals, "muisic_gae_on", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_gae_on")?;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.music.resume_stream();
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // muisic_stap
    define_native(globals, "muisic_stap", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_stap")?;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.music.stop_stream();
            entry.state = PlayState::Stopped;
            Ok(Value::Nil)
        })
    });

    // muisic_unlade
    define_native(globals, "muisic_unlade", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_unlade")?;
            if handle >= state.music.len() || state.music[handle].is_none() {
                return Err(ERR_BAD_HANDLE.to_string());
            }
            state.music[handle] = None;
            Ok(Value::Nil)
        })
    });

    // muisic_is_spielin
    define_native(globals, "muisic_is_spielin", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_is_spielin")?;
            let entry = state
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.music.is_stream_playing()))
        })
    });

    // muisic_loup
    define_native(globals, "muisic_loup", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_loup")?;
            let pos = as_number(&args[1], "muisic_loup")? as f32;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.music.seek_stream(pos);
            Ok(Value::Nil)
        })
    });

    // muisic_hou_lang
    define_native(globals, "muisic_hou_lang", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_hou_lang")?;
            let entry = state
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Float(entry.music.get_time_length() as f64))
        })
    });

    // muisic_whaur
    define_native(globals, "muisic_whaur", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_whaur")?;
            let entry = state
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Float(entry.music.get_time_played() as f64))
        })
    });

    // muisic_pit_luid
    define_native(globals, "muisic_pit_luid", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_luid")?;
            let mut value = as_number(&args[1], "muisic_pit_luid")? as f32;
            value = clamp01(value);
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value;
            entry.music.set_volume(value);
            Ok(Value::Nil)
        })
    });

    // muisic_pit_pan
    define_native(globals, "muisic_pit_pan", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_pan")?;
            let pan = as_number(&args[1], "muisic_pit_pan")? as f32;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan;
            entry.music.set_pan(pan_to_raylib(pan));
            Ok(Value::Nil)
        })
    });

    // muisic_pit_tune
    define_native(globals, "muisic_pit_tune", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_tune")?;
            let pitch = as_number(&args[1], "muisic_pit_tune")? as f32;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pitch = pitch;
            entry.music.set_pitch(pitch);
            Ok(Value::Nil)
        })
    });

    // muisic_pit_rin_roond
    define_native(globals, "muisic_pit_rin_roond", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_rin_roond")?;
            let looped = as_bool(&args[1], "muisic_pit_rin_roond")?;
            let entry = state
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.looped = looped;
            Ok(Value::Nil)
        })
    });

    // midi_lade (path, soundfont or naething)
    define_native(globals, "midi_lade", 2, |args| {
        with_state(|state| {
            let midi_path = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("midi_lade needs a midi filepath".to_string()),
            };

            let sf = match &args[1] {
                Value::Nil => {
                    if let Some(sf) = &state.default_soundfont {
                        Arc::clone(sf)
                    } else {
                        let path = resolve_default_soundfont()?;
                        let sf = load_soundfont(path.as_path())?;
                        state.default_soundfont = Some(Arc::clone(&sf));
                        sf
                    }
                }
                Value::String(path) => load_soundfont(Path::new(path))?,
                _ => return Err("midi_lade needs a soondfont path or naething".to_string()),
            };

            let mut midi_file = File::open(&midi_path)
                .map_err(|_| "Cannae open the midi file".to_string())?;
            let midi = MidiFile::new(&mut midi_file)
                .map_err(|_| "Cannae read the midi".to_string())?;
            let midi = Arc::new(midi);

            let settings = SynthesizerSettings::new(MIDI_SAMPLE_RATE);
            let synth = Synthesizer::new(&sf, &settings)
                .map_err(|_| "Cannae set up the synth".to_string())?;
            let sequencer = MidiFileSequencer::new(synth);

            let audio = state.ensure_audio()?;
            let stream = stream_to_static(audio.new_audio_stream(MIDI_SAMPLE_RATE as u32, 32, 2));
            stream.set_volume(1.0);
            stream.set_pan(pan_to_raylib(0.0));

            let entry = MidiEntry {
                midi,
                sequencer,
                stream,
                state: PlayState::Stopped,
                looped: false,
                volume: 1.0,
                pan: 0.0,
                sample_rate: MIDI_SAMPLE_RATE,
            };

            let handle = AudioState::alloc_handle(&mut state.midi, entry);
            Ok(Value::Integer(handle))
        })
    });

    // midi_spiel
    define_native(globals, "midi_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_spiel")?;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            if entry.state == PlayState::Stopped {
                entry.sequencer.play(&entry.midi, entry.looped);
            }
            if entry.state == PlayState::Paused {
                entry.stream.resume();
            } else {
                entry.stream.play();
            }
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // midi_haud
    define_native(globals, "midi_haud", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_haud")?;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.stream.pause();
            entry.state = PlayState::Paused;
            Ok(Value::Nil)
        })
    });

    // midi_gae_on
    define_native(globals, "midi_gae_on", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_gae_on")?;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.stream.resume();
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // midi_stap
    define_native(globals, "midi_stap", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_stap")?;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.stream.stop();
            entry.sequencer.stop();
            entry.state = PlayState::Stopped;
            Ok(Value::Nil)
        })
    });

    // midi_unlade
    define_native(globals, "midi_unlade", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_unlade")?;
            if handle >= state.midi.len() || state.midi[handle].is_none() {
                return Err(ERR_BAD_HANDLE.to_string());
            }
            state.midi[handle] = None;
            Ok(Value::Nil)
        })
    });

    // midi_is_spielin
    define_native(globals, "midi_is_spielin", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_is_spielin")?;
            let entry = state
                .midi
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.stream.is_playing()))
        })
    });

    // midi_loup
    define_native(globals, "midi_loup", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_loup")?;
            let pos = as_number(&args[1], "midi_loup")?;
            let audio = state.audio_ref()?;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            seek_midi(entry, pos, audio)?;
            Ok(Value::Nil)
        })
    });

    // midi_hou_lang
    define_native(globals, "midi_hou_lang", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_hou_lang")?;
            let entry = state
                .midi
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Float(entry.midi.get_length()))
        })
    });

    // midi_whaur
    define_native(globals, "midi_whaur", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_whaur")?;
            let entry = state
                .midi
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Float(entry.sequencer.get_position()))
        })
    });

    // midi_pit_luid
    define_native(globals, "midi_pit_luid", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_pit_luid")?;
            let mut value = as_number(&args[1], "midi_pit_luid")? as f32;
            value = clamp01(value);
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value;
            entry.stream.set_volume(value);
            Ok(Value::Nil)
        })
    });

    // midi_pit_pan
    define_native(globals, "midi_pit_pan", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_pit_pan")?;
            let pan = as_number(&args[1], "midi_pit_pan")? as f32;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan;
            entry.stream.set_pan(pan_to_raylib(pan));
            Ok(Value::Nil)
        })
    });

    // midi_pit_rin_roond
    define_native(globals, "midi_pit_rin_roond", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_pit_rin_roond")?;
            let looped = as_bool(&args[1], "midi_pit_rin_roond")?;
            let entry = state
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.looped = looped;
            Ok(Value::Nil)
        })
    });
}

#[cfg(not(any(feature = "audio", test)))]
pub fn register_audio_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
    use crate::value::{NativeFunction, Value};

    fn define_stub(
        globals: &Rc<RefCell<crate::value::Environment>>,
        name: &str,
        arity: usize,
    ) {
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
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn with_cwd<T>(path: &Path, func: impl FnOnce() -> T) -> T {
        let _lock = CWD_LOCK.lock().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        let result = func();
        std::env::set_current_dir(original).unwrap();
        result
    }

    #[test]
    fn test_clamp_and_pan_helpers() {
        assert_eq!(clamp01(-0.1), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(1.5), 1.0);

        assert_eq!(pan_to_raylib(-1.0), 0.0);
        assert_eq!(pan_to_raylib(0.0), 0.5);
        assert_eq!(pan_to_raylib(1.0), 1.0);
        assert_eq!(pan_to_raylib(2.0), 1.0);
    }

    #[test]
    fn test_value_parsing_helpers() {
        assert_eq!(as_number(&Value::Integer(3), "num").unwrap(), 3.0);
        assert_eq!(as_number(&Value::Float(2.5), "num").unwrap(), 2.5);
        assert!(as_number(&Value::Bool(true), "num").is_err());

        assert_eq!(as_bool(&Value::Bool(true), "bool").unwrap(), true);
        assert!(as_bool(&Value::Integer(1), "bool").is_err());

        assert_eq!(as_handle(&Value::Integer(2), "handle").unwrap(), 2);
        assert!(as_handle(&Value::Integer(-1), "handle").is_err());
        assert!(as_handle(&Value::String("x".to_string()), "handle").is_err());
    }

    #[test]
    fn test_alloc_handle_reuse() {
        let mut slots = vec![Some(1), None, Some(2)];
        let h1 = AudioState::alloc_handle(&mut slots, 99);
        assert_eq!(h1, 1);
        assert_eq!(slots[1], Some(99));

        let h2 = AudioState::alloc_handle(&mut slots, 77);
        assert_eq!(h2, 3);
        assert_eq!(slots[3], Some(77));
    }

    #[test]
    fn test_resolve_default_soundfont_success() {
        let dir = tempdir().unwrap();
        let sf_dir = dir.path().join("assets/soundfonts");
        fs::create_dir_all(&sf_dir).unwrap();
        let sf_path = sf_dir.join("MuseScore_General.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        let found = with_cwd(dir.path(), || resolve_default_soundfont().unwrap());
        assert_eq!(found, sf_path);
    }

    #[test]
    fn test_resolve_default_soundfont_missing() {
        let dir = tempdir().unwrap();
        let err = with_cwd(dir.path(), || resolve_default_soundfont().unwrap_err());
        assert_eq!(err, "Cannae find the default soondfont");
    }

    #[test]
    fn test_load_soundfont_ok_and_missing() {
        let dir = tempdir().unwrap();
        let sf_path = dir.path().join("test.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        assert!(load_soundfont(sf_path.as_path()).is_ok());
        assert!(load_soundfont(dir.path().join("nope.sf2").as_path()).is_err());
    }

    #[test]
    fn test_audio_ref_and_ensure_audio() {
        AUDIO_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.shutdown();
            assert_eq!(state.audio_ref().unwrap_err(), ERR_NO_DEVICE);
            state.ensure_audio().unwrap();
            state.ensure_audio().unwrap();
            assert!(state.audio_ref().is_ok());
        });
    }

    #[test]
    fn test_prime_midi_stream_skips_when_unprocessed() {
        let dir = tempdir().unwrap();
        let midi_path = dir.path().join("test.mid");
        let sf_path = dir.path().join("test.sf2");
        fs::write(&midi_path, b"midi").unwrap();
        fs::write(&sf_path, b"sf").unwrap();

        let sf = load_soundfont(sf_path.as_path()).unwrap();
        let mut midi_file = File::open(&midi_path).unwrap();
        let midi = MidiFile::new(&mut midi_file).unwrap();
        let midi = Arc::new(midi);

        let settings = SynthesizerSettings::new(MIDI_SAMPLE_RATE);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let mut sequencer = MidiFileSequencer::new(synth);
        sequencer.play(&midi, false);

        let audio = RaylibAudio::init_audio_device().unwrap();
        let stream = stream_to_static(audio.new_audio_stream(MIDI_SAMPLE_RATE as u32, 32, 2));

        let mut entry = MidiEntry {
            midi,
            sequencer,
            stream,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: MIDI_SAMPLE_RATE,
        };

        entry.stream.set_processed_for_test(false);
        prime_midi_stream(&mut entry);
        assert_eq!(entry.sequencer.get_position(), 0.0);

        entry.stream.set_processed_for_test(true);
        prime_midi_stream(&mut entry);
        assert!(entry.sequencer.get_position() > 0.0);
    }
}
