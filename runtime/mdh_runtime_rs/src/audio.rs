use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use raylib::prelude::*;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

use super::{
    mdh_float_value, mdh_make_string_from_rust, mdh_string_to_rust, MdhValue, MDH_TAG_BOOL,
    MDH_TAG_FLOAT, MDH_TAG_INT, MDH_TAG_NIL, MDH_TAG_STRING,
};

const ERR_NO_DEVICE: &str = "Soond device isnae stairtit";
const ERR_BAD_HANDLE: &str = "Thon handle isnae guid";

const MIDI_SAMPLE_RATE: i32 = 44_100;
const MIDI_STREAM_FRAMES: usize = 1_024;
const DEFAULT_SOUNDFONT_PATH: &str = "assets/soundfonts/MuseScore_General.sf2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Stopped,
    Playing,
    Paused,
}

struct SoundEntry {
    sound: Sound<'static>,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    pitch: f32,
}

struct MusicEntry {
    music: Music<'static>,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    pitch: f32,
}

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

struct AudioState {
    audio: Option<Box<RaylibAudio>>,
    master_volume: f32,
    muted: bool,
    sounds: Vec<Option<SoundEntry>>,
    music: Vec<Option<MusicEntry>>,
    midi: Vec<Option<MidiEntry>>,
    default_soundfont: Option<Arc<SoundFont>>,
}

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

fn sound_to_static(sound: Sound<'_>) -> Sound<'static> {
    unsafe { std::mem::transmute(sound) }
}

fn music_to_static(music: Music<'_>) -> Music<'static> {
    unsafe { std::mem::transmute(music) }
}

fn stream_to_static(stream: AudioStream<'_>) -> AudioStream<'static> {
    unsafe { std::mem::transmute(stream) }
}

thread_local! {
    static AUDIO_STATE: RefCell<AudioState> = RefCell::new(AudioState::new());
}

fn with_state<F>(func: F) -> MdhValue
where
    F: FnOnce(&mut AudioState) -> MdhValue,
{
    AUDIO_STATE.with(|state| {
        let mut state = state.borrow_mut();
        func(&mut state)
    })
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

fn pan_to_raylib(value: f32) -> f32 {
    let mapped = (value + 1.0) * 0.5;
    clamp01(mapped)
}

fn load_soundfont(path: &Path) -> Result<Arc<SoundFont>, String> {
    let mut file = File::open(path).map_err(|_| "Cannae open the soondfont file".to_string())?;
    let sf = SoundFont::new(&mut file).map_err(|_| "Cannae read the soondfont".to_string())?;
    Ok(Arc::new(sf))
}

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

fn seek_midi(entry: &mut MidiEntry, seconds: f64, audio: &'static RaylibAudio) -> Result<(), String> {
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

fn expect_number(value: MdhValue, name: &str) -> Result<f64, String> {
    if value.tag == MDH_TAG_FLOAT {
        Ok(unsafe { mdh_float_value(value) })
    } else if value.tag == MDH_TAG_INT {
        Ok(value.data as f64)
    } else {
        Err(format!("{} needs a nummer", name))
    }
}

fn expect_bool(value: MdhValue, name: &str) -> Result<bool, String> {
    if value.tag == MDH_TAG_BOOL {
        Ok(value.data != 0)
    } else {
        Err(format!("{} needs aye or nae", name))
    }
}

fn expect_handle(value: MdhValue, name: &str) -> Result<usize, String> {
    if value.tag == MDH_TAG_INT && value.data >= 0 {
        Ok(value.data as usize)
    } else {
        Err(format!("{} needs a guid handle", name))
    }
}

fn expect_string(value: MdhValue, name: &str) -> Result<String, String> {
    if value.tag == MDH_TAG_STRING {
        Ok(unsafe { mdh_string_to_rust(value) })
    } else {
        Err(format!("{} needs a string path", name))
    }
}

fn hurl_msg(msg: &str) -> MdhValue {
    unsafe {
        __mdh_hurl(mdh_make_string_from_rust(msg));
        __mdh_make_nil()
    }
}

extern "C" {
    fn __mdh_make_nil() -> MdhValue;
    fn __mdh_make_bool(value: bool) -> MdhValue;
    fn __mdh_make_int(value: i64) -> MdhValue;
    fn __mdh_make_float(value: f64) -> MdhValue;
    fn __mdh_hurl(value: MdhValue);
}

#[no_mangle]
pub extern "C" fn __mdh_soond_stairt() -> MdhValue {
    with_state(|state| match state.ensure_audio() {
        Ok(_) => unsafe { __mdh_make_nil() },
        Err(msg) => hurl_msg(&msg),
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_steek() -> MdhValue {
    with_state(|state| {
        state.shutdown();
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_wheesht(value: MdhValue) -> MdhValue {
    with_state(|state| {
        let wheesht = match expect_bool(value, "soond_wheesht") {
            Ok(v) => v,
            Err(msg) => return hurl_msg(&msg),
        };
        let audio = match state.ensure_audio() {
            Ok(a) => a,
            Err(msg) => return hurl_msg(&msg),
        };
        state.muted = wheesht;
        if wheesht {
            audio.set_master_volume(0.0);
        } else {
            audio.set_master_volume(state.master_volume);
        }
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_luid(value: MdhValue) -> MdhValue {
    with_state(|state| {
        let mut volume = match expect_number(value, "soond_luid") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        volume = clamp01(volume);
        let audio = match state.ensure_audio() {
            Ok(a) => a,
            Err(msg) => return hurl_msg(&msg),
        };
        state.master_volume = volume;
        if !state.muted {
            audio.set_master_volume(volume);
        }
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_hou_luid() -> MdhValue {
    with_state(|state| unsafe { __mdh_make_float(state.master_volume as f64) })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_haud_gang() -> MdhValue {
    with_state(|state| {
        if state.audio.is_none() {
            return unsafe { __mdh_make_nil() };
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

        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_lade(path_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let path = match expect_string(path_val, "soond_lade") {
            Ok(p) => p,
            Err(msg) => return hurl_msg(&msg),
        };
        let audio = match state.ensure_audio() {
            Ok(a) => a,
            Err(msg) => return hurl_msg(&msg),
        };
        let sound = audio
            .new_sound(&path)
            .map_err(|_| "Cannae lade the soond".to_string());
        let sound = match sound {
            Ok(s) => sound_to_static(s),
            Err(msg) => return hurl_msg(&msg),
        };
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
        unsafe { __mdh_make_int(handle) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_spiel(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_spiel") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.sound.play();
        entry.state = PlayState::Playing;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_haud(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_haud") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.sound.pause();
        entry.state = PlayState::Paused;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_gae_on(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_gae_on") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.sound.resume();
        entry.state = PlayState::Playing;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_stap(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_stap") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.sound.stop();
        entry.state = PlayState::Stopped;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_unlade(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_unlade") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        if state.sounds.get(handle).and_then(|e| e.as_ref()).is_none() {
            return hurl_msg(ERR_BAD_HANDLE);
        }
        state.sounds[handle] = None;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_is_spielin(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_is_spielin") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_bool(entry.sound.is_playing()) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_luid(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_pit_luid") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let mut value = match expect_number(val, "soond_pit_luid") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        value = clamp01(value);
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.volume = value;
        entry.sound.set_volume(value);
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_pan(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_pit_pan") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pan = match expect_number(val, "soond_pit_pan") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.pan = pan;
        entry.sound.set_pan(pan_to_raylib(pan));
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_tune(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_pit_tune") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pitch = match expect_number(val, "soond_pit_tune") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.pitch = pitch;
        entry.sound.set_pitch(pitch);
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_rin_roond(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_pit_rin_roond") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let looped = match expect_bool(val, "soond_pit_rin_roond") {
            Ok(v) => v,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.looped = looped;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_ready(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "soond_ready") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.sounds.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        let _ = entry;
        unsafe { __mdh_make_bool(true) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_lade(path_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let path = match expect_string(path_val, "muisic_lade") {
            Ok(p) => p,
            Err(msg) => return hurl_msg(&msg),
        };
        let audio = match state.ensure_audio() {
            Ok(a) => a,
            Err(msg) => return hurl_msg(&msg),
        };
        let music = audio
            .new_music(&path)
            .map_err(|_| "Cannae lade the muisic".to_string());
        let music = match music {
            Ok(m) => music_to_static(m),
            Err(msg) => return hurl_msg(&msg),
        };
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
        unsafe { __mdh_make_int(handle) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_spiel(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_spiel") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        if entry.state == PlayState::Paused {
            entry.music.resume_stream();
        } else {
            entry.music.play_stream();
        }
        entry.state = PlayState::Playing;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_haud(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_haud") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.music.pause_stream();
        entry.state = PlayState::Paused;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_gae_on(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_gae_on") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.music.resume_stream();
        entry.state = PlayState::Playing;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_stap(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_stap") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.music.stop_stream();
        entry.state = PlayState::Stopped;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_unlade(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_unlade") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.music.stop_stream();
        state.music[handle] = None;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_is_spielin(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_is_spielin") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_bool(entry.music.is_stream_playing()) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_loup(handle_val: MdhValue, seconds_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_loup") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pos = match expect_number(seconds_val, "muisic_loup") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.music.seek_stream(pos);
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_hou_lang(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_hou_lang") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_float(entry.music.get_time_length() as f64) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_whaur(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_whaur") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_float(entry.music.get_time_played() as f64) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_luid(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_pit_luid") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let mut value = match expect_number(val, "muisic_pit_luid") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        value = clamp01(value);
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.volume = value;
        entry.music.set_volume(value);
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_pan(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_pit_pan") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pan = match expect_number(val, "muisic_pit_pan") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.pan = pan;
        entry.music.set_pan(pan_to_raylib(pan));
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_tune(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_pit_tune") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pitch = match expect_number(val, "muisic_pit_tune") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.pitch = pitch;
        entry.music.set_pitch(pitch);
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_rin_roond(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "muisic_pit_rin_roond") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let looped = match expect_bool(val, "muisic_pit_rin_roond") {
            Ok(v) => v,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.looped = looped;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_lade(midi_val: MdhValue, sf_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let midi_path = match expect_string(midi_val, "midi_lade") {
            Ok(p) => p,
            Err(msg) => return hurl_msg(&msg),
        };

        let sf = if sf_val.tag == MDH_TAG_NIL {
            if let Some(sf) = &state.default_soundfont {
                Arc::clone(sf)
            } else {
                let path = match resolve_default_soundfont() {
                    Ok(p) => p,
                    Err(msg) => return hurl_msg(&msg),
                };
                let sf = match load_soundfont(path.as_path()) {
                    Ok(sf) => sf,
                    Err(msg) => return hurl_msg(&msg),
                };
                state.default_soundfont = Some(Arc::clone(&sf));
                sf
            }
        } else if sf_val.tag == MDH_TAG_STRING {
            let path = unsafe { mdh_string_to_rust(sf_val) };
            match load_soundfont(Path::new(&path)) {
                Ok(sf) => sf,
                Err(msg) => return hurl_msg(&msg),
            }
        } else {
            return hurl_msg("midi_lade needs a soondfont path or naething");
        };

        let mut midi_file = match File::open(&midi_path) {
            Ok(f) => f,
            Err(_) => return hurl_msg("Cannae open the midi file"),
        };
        let midi = match MidiFile::new(&mut midi_file) {
            Ok(m) => m,
            Err(_) => return hurl_msg("Cannae read the midi"),
        };
        let midi = Arc::new(midi);

        let settings = SynthesizerSettings::new(MIDI_SAMPLE_RATE);
        let synth = match Synthesizer::new(&sf, &settings) {
            Ok(s) => s,
            Err(_) => return hurl_msg("Cannae set up the synth"),
        };
        let sequencer = MidiFileSequencer::new(synth);

        let audio = match state.ensure_audio() {
            Ok(a) => a,
            Err(msg) => return hurl_msg(&msg),
        };
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
        unsafe { __mdh_make_int(handle) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_spiel(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_spiel") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        if entry.state == PlayState::Stopped {
            entry.sequencer.play(&entry.midi, entry.looped);
        }
        if entry.state == PlayState::Paused {
            entry.stream.resume();
        } else {
            entry.stream.play();
        }
        entry.state = PlayState::Playing;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_haud(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_haud") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.stream.pause();
        entry.state = PlayState::Paused;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_gae_on(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_gae_on") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.stream.resume();
        entry.state = PlayState::Playing;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_stap(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_stap") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.stream.stop();
        entry.sequencer.stop();
        entry.state = PlayState::Stopped;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_unlade(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_unlade") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.stream.stop();
        state.midi[handle] = None;
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_is_spielin(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_is_spielin") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_bool(entry.stream.is_playing()) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_loup(handle_val: MdhValue, seconds_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_loup") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pos = match expect_number(seconds_val, "midi_loup") {
            Ok(v) => v,
            Err(msg) => return hurl_msg(&msg),
        };
        let audio = match state.audio_ref() {
            Ok(a) => a,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        if let Err(msg) = seek_midi(entry, pos, audio) {
            return hurl_msg(&msg);
        }
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_hou_lang(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_hou_lang") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_float(entry.midi.get_length() as f64) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_whaur(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_whaur") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        unsafe { __mdh_make_float(entry.sequencer.get_position() as f64) }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_luid(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_pit_luid") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let mut value = match expect_number(val, "midi_pit_luid") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        value = clamp01(value);
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.volume = value;
        entry.stream.set_volume(value);
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_pan(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_pit_pan") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let pan = match expect_number(val, "midi_pit_pan") {
            Ok(v) => v as f32,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.pan = pan;
        entry.stream.set_pan(pan_to_raylib(pan));
        unsafe { __mdh_make_nil() }
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_rin_roond(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = match expect_handle(handle_val, "midi_pit_rin_roond") {
            Ok(h) => h,
            Err(msg) => return hurl_msg(&msg),
        };
        let looped = match expect_bool(val, "midi_pit_rin_roond") {
            Ok(v) => v,
            Err(msg) => return hurl_msg(&msg),
        };
        let entry = match state.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return hurl_msg(ERR_BAD_HANDLE),
        };
        entry.looped = looped;
        unsafe { __mdh_make_nil() }
    })
}
