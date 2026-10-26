use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use miniaudio::{Decoder, DecoderConfig, Device, DeviceConfig, DeviceType, Format, FramesMut};
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

use super::{
    mdh_error_string, mdh_float_value, mdh_string_to_rust, MdhValue, MDH_TAG_BOOL,
    MDH_TAG_FLOAT, MDH_TAG_INT, MDH_TAG_NIL, MDH_TAG_STRING,
};

const ERR_NO_DEVICE: &str = "Soond device isnae stairtit";
const ERR_BAD_HANDLE: &str = "Thon handle isnae guid";

const OUTPUT_SAMPLE_RATE: u32 = 44_100;
const OUTPUT_CHANNELS: u32 = 2;
const DECODE_CHUNK_FRAMES: usize = 1_024;
const DEFAULT_SOUNDFONT_PATH: &str = "assets/soundfonts/MuseScore_General.sf2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone)]
struct SampleBuffer {
    samples: Arc<Vec<f32>>, // interleaved stereo
    frames: usize,
}

struct BufferEntry {
    buffer: SampleBuffer,
    position: f64,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    pitch: f32,
}

type SoundEntry = BufferEntry;
type MusicEntry = BufferEntry;

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

struct MixerState {
    master_volume: f32,
    muted: bool,
    sounds: Vec<Option<SoundEntry>>,
    music: Vec<Option<MusicEntry>>,
    midi: Vec<Option<MidiEntry>>,
    default_soundfont: Option<Arc<SoundFont>>,
}

impl MixerState {
    fn new() -> Self {
        Self {
            master_volume: 1.0,
            muted: false,
            sounds: Vec::new(),
            music: Vec::new(),
            midi: Vec::new(),
            default_soundfont: None,
        }
    }

    fn reset(&mut self) {
        self.master_volume = 1.0;
        self.muted = false;
        self.sounds.clear();
        self.music.clear();
        self.midi.clear();
        self.default_soundfont = None;
    }
}

struct AudioState {
    device: Option<Device>,
    shared: Arc<Mutex<MixerState>>,
}

impl AudioState {
    fn new() -> Self {
        Self {
            device: None,
            shared: Arc::new(Mutex::new(MixerState::new())),
        }
    }

    fn ensure_audio(&mut self) -> Result<(), String> {
        if self.device.is_some() {
            return Ok(());
        }

        let shared = Arc::clone(&self.shared);
        let mut config = DeviceConfig::new(DeviceType::Playback);
        config.set_sample_rate(OUTPUT_SAMPLE_RATE);
        config.playback_mut().set_format(Format::F32);
        config.playback_mut().set_channels(OUTPUT_CHANNELS);
        config.set_data_callback(move |_device, output, _input| {
            mix_output(&shared, output);
        });

        let device = Device::new(None, &config)
            .map_err(|_| "Cannae stairt the soond device".to_string())?;
        device
            .start()
            .map_err(|_| "Cannae stairt the soond device".to_string())?;
        self.device = Some(device);
        Ok(())
    }

    fn mixer(&self) -> Result<std::sync::MutexGuard<'_, MixerState>, String> {
        let guard = match self.shared.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        Ok(guard)
    }

    fn shutdown(&mut self) {
        if let Some(device) = self.device.take() {
            let _ = device.stop();
        }
        let mut mixer = match self.shared.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        mixer.reset();
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
}

thread_local! {
    static AUDIO_STATE: RefCell<AudioState> = RefCell::new(AudioState::new());
}

/// Run `func` on the audio state. An error it returns is raised only once the state's
/// borrow, the mixer lock and every other Rust value of the call are dropped: raising
/// inside would longjmp past them and leave the state borrowed for good
/// (MDH-BUG-FLU-00183).
fn with_state<F>(func: F) -> MdhValue
where
    F: FnOnce(&mut AudioState) -> Result<MdhValue, String>,
{
    let result = AUDIO_STATE.with(|state| {
        let mut state = state.borrow_mut();
        func(&mut state)
    });
    match result {
        Ok(value) => value,
        Err(msg) => {
            let error = unsafe { mdh_error_string(&msg) };
            drop(msg);
            unsafe {
                __mdh_hurl(error);
                __mdh_make_nil()
            }
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn pan_gains(pan: f32) -> (f32, f32) {
    let clamped = pan.clamp(-1.0, 1.0);
    let t = (clamped + 1.0) * 0.5;
    let angle = t * FRAC_PI_2;
    (angle.cos(), angle.sin())
}

fn decode_audio(path: &str, err_msg: &str) -> Result<SampleBuffer, String> {
    let config = DecoderConfig::new(Format::F32, OUTPUT_CHANNELS, OUTPUT_SAMPLE_RATE);
    let mut decoder = Decoder::from_file(path, Some(&config)).map_err(|_| err_msg.to_string())?;

    let mut samples: Vec<f32> = Vec::new();
    let mut temp = vec![0.0_f32; DECODE_CHUNK_FRAMES * OUTPUT_CHANNELS as usize];

    loop {
        let mut frames = FramesMut::wrap(&mut temp, Format::F32, OUTPUT_CHANNELS);
        let read = decoder.read_pcm_frames(&mut frames) as usize;
        if read == 0 {
            break;
        }
        samples.extend_from_slice(&temp[..read * OUTPUT_CHANNELS as usize]);
    }

    let frames = samples.len() / OUTPUT_CHANNELS as usize;
    Ok(SampleBuffer {
        samples: Arc::new(samples),
        frames,
    })
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

fn mix_output(shared: &Arc<Mutex<MixerState>>, output: &mut FramesMut) {
    let frames = output.frame_count();
    let out_samples = output.as_samples_mut::<f32>();
    for sample in out_samples.iter_mut() {
        *sample = 0.0;
    }

    let mut mixer = match shared.try_lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    mix_state(&mut mixer, out_samples, frames);
}

fn mix_state(state: &mut MixerState, output: &mut [f32], frames: usize) {
    let channels = OUTPUT_CHANNELS as usize;

    for slot in state.sounds.iter_mut() {
        if let Some(entry) = slot.as_mut() {
            mix_buffer_entry(entry, output, frames, channels);
        }
    }

    for slot in state.music.iter_mut() {
        if let Some(entry) = slot.as_mut() {
            mix_buffer_entry(entry, output, frames, channels);
        }
    }

    for slot in state.midi.iter_mut() {
        if let Some(entry) = slot.as_mut() {
            mix_midi_entry(entry, output, frames, channels);
        }
    }

    let master = if state.muted { 0.0 } else { state.master_volume };
    if master != 1.0 {
        for sample in output.iter_mut() {
            *sample *= master;
        }
    }

    for sample in output.iter_mut() {
        *sample = sample.clamp(-1.0, 1.0);
    }
}

fn mix_buffer_entry(entry: &mut BufferEntry, output: &mut [f32], frames: usize, channels: usize) {
    if entry.state != PlayState::Playing {
        return;
    }

    if entry.buffer.frames == 0 {
        entry.state = PlayState::Stopped;
        return;
    }

    let pitch = if entry.pitch <= 0.0 { 1.0 } else { entry.pitch };
    let (left_gain, right_gain) = pan_gains(entry.pan);
    let volume = entry.volume;
    let total_frames = entry.buffer.frames;
    let samples = &entry.buffer.samples;
    let mut position = entry.position;

    for frame in 0..frames {
        if position >= total_frames as f64 {
            if entry.looped {
                position = position % total_frames as f64;
            } else {
                entry.state = PlayState::Stopped;
                break;
            }
        }

        let idx = position.floor() as usize;
        let frac = (position - idx as f64) as f32;
        let next_idx = if idx + 1 < total_frames {
            idx + 1
        } else if entry.looped {
            0
        } else {
            idx
        };

        let base = idx * channels;
        let next_base = next_idx * channels;

        let left = lerp(samples[base], samples[next_base], frac);
        let right = lerp(samples[base + 1], samples[next_base + 1], frac);

        let out_base = frame * channels;
        output[out_base] += left * volume * left_gain;
        output[out_base + 1] += right * volume * right_gain;

        position += pitch as f64;
    }

    entry.position = position;
}

fn mix_midi_entry(entry: &mut MidiEntry, output: &mut [f32], frames: usize, channels: usize) {
    if entry.state != PlayState::Playing {
        return;
    }

    if entry.scratch_left.len() < frames {
        entry.scratch_left.resize(frames, 0.0);
        entry.scratch_right.resize(frames, 0.0);
    }

    let left_buf = &mut entry.scratch_left[..frames];
    let right_buf = &mut entry.scratch_right[..frames];
    entry.sequencer.render(left_buf, right_buf);

    let (left_gain, right_gain) = pan_gains(entry.pan);
    let volume = entry.volume;

    for i in 0..frames {
        let out_base = i * channels;
        output[out_base] += left_buf[i] * volume * left_gain;
        output[out_base + 1] += right_buf[i] * volume * right_gain;
    }

    if entry.sequencer.end_of_sequence() && !entry.looped {
        entry.state = PlayState::Stopped;
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn seek_midi(entry: &mut MidiEntry, seconds: f64) -> Result<(), String> {
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
        entry
            .sequencer
            .render(&mut left[..chunk], &mut right[..chunk]);
        remaining -= chunk;
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

extern "C" {
    fn __mdh_make_nil() -> MdhValue;
    fn __mdh_make_bool(value: bool) -> MdhValue;
    fn __mdh_make_int(value: i64) -> MdhValue;
    fn __mdh_make_float(value: f64) -> MdhValue;
    fn __mdh_hurl(value: MdhValue);
}

#[no_mangle]
pub extern "C" fn __mdh_soond_stairt() -> MdhValue {
    with_state(|state| {
        state.ensure_audio()?;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_steek() -> MdhValue {
    with_state(|state| {
        state.shutdown();
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_wheesht(value: MdhValue) -> MdhValue {
    with_state(|state| {
        let wheesht = expect_bool(value, "soond_wheesht")?;
        state.ensure_audio()?;
        let mut mixer = state.mixer()?;
        mixer.muted = wheesht;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_luid(value: MdhValue) -> MdhValue {
    with_state(|state| {
        let mut volume = expect_number(value, "soond_luid")? as f32;
        volume = clamp01(volume);
        state.ensure_audio()?;
        let mut mixer = state.mixer()?;
        mixer.master_volume = volume;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_hou_luid() -> MdhValue {
    with_state(|state| {
        let mixer = state.mixer()?;
        Ok(unsafe { __mdh_make_float(mixer.master_volume as f64) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_haud_gang() -> MdhValue {
    with_state(|state| {
        let _ = state;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_lade(path_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let path = expect_string(path_val, "soond_lade")?;
        state.ensure_audio()?;
        let buffer = decode_audio(&path, "Cannae lade the soond")?;
        let entry = SoundEntry {
            buffer,
            position: 0.0,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };
        let mut mixer = state.mixer()?;
        let handle = AudioState::alloc_handle(&mut mixer.sounds, entry);
        Ok(unsafe { __mdh_make_int(handle) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_spiel(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_spiel")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.position = 0.0;
        entry.state = PlayState::Playing;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_haud(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_haud")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Paused;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_gae_on(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_gae_on")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Playing;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_stap(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_stap")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Stopped;
        entry.position = 0.0;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_unlade(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_unlade")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Stopped;
        mixer.sounds[handle] = None;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_is_spielin(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_is_spielin")?;
        let mixer = state.mixer()?;
        let entry = match mixer.sounds.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        Ok(unsafe { __mdh_make_bool(entry.state == PlayState::Playing) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_luid(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_pit_luid")?;
        let mut value = expect_number(val, "soond_pit_luid")? as f32;
        value = clamp01(value);
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.volume = value;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_pan(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_pit_pan")?;
        let pan = expect_number(val, "soond_pit_pan")? as f32;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.pan = pan;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_tune(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_pit_tune")?;
        let pitch = expect_number(val, "soond_pit_tune")? as f32;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.pitch = pitch;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_rin_roond(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_pit_rin_roond")?;
        let looped = expect_bool(val, "soond_pit_rin_roond")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.sounds.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.looped = looped;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_soond_ready(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "soond_ready")?;
        let mixer = state.mixer()?;
        let entry = match mixer.sounds.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        let _ = entry;
        Ok(unsafe { __mdh_make_bool(true) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_lade(path_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let path = expect_string(path_val, "muisic_lade")?;
        state.ensure_audio()?;
        let buffer = decode_audio(&path, "Cannae lade the muisic")?;
        let entry = MusicEntry {
            buffer,
            position: 0.0,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };
        let mut mixer = state.mixer()?;
        let handle = AudioState::alloc_handle(&mut mixer.music, entry);
        Ok(unsafe { __mdh_make_int(handle) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_spiel(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_spiel")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        if entry.state == PlayState::Stopped {
            entry.position = 0.0;
        }
        entry.state = PlayState::Playing;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_haud(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_haud")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Paused;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_gae_on(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_gae_on")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Playing;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_stap(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_stap")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Stopped;
        entry.position = 0.0;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_unlade(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_unlade")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Stopped;
        mixer.music[handle] = None;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_is_spielin(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_is_spielin")?;
        let mixer = state.mixer()?;
        let entry = match mixer.music.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        Ok(unsafe { __mdh_make_bool(entry.state == PlayState::Playing) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_loup(handle_val: MdhValue, seconds_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_loup")?;
        let pos = expect_number(seconds_val, "muisic_loup")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        let target = (pos * OUTPUT_SAMPLE_RATE as f64).max(0.0);
        entry.position = target.min(entry.buffer.frames as f64);
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_hou_lang(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_hou_lang")?;
        let mixer = state.mixer()?;
        let entry = match mixer.music.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        let length = entry.buffer.frames as f64 / OUTPUT_SAMPLE_RATE as f64;
        Ok(unsafe { __mdh_make_float(length) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_whaur(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_whaur")?;
        let mixer = state.mixer()?;
        let entry = match mixer.music.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        let pos = entry.position / OUTPUT_SAMPLE_RATE as f64;
        Ok(unsafe { __mdh_make_float(pos) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_luid(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_pit_luid")?;
        let mut value = expect_number(val, "muisic_pit_luid")? as f32;
        value = clamp01(value);
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.volume = value;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_pan(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_pit_pan")?;
        let pan = expect_number(val, "muisic_pit_pan")? as f32;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.pan = pan;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_tune(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_pit_tune")?;
        let pitch = expect_number(val, "muisic_pit_tune")? as f32;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.pitch = pitch;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_rin_roond(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "muisic_pit_rin_roond")?;
        let looped = expect_bool(val, "muisic_pit_rin_roond")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.music.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.looped = looped;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_lade(midi_val: MdhValue, sf_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let midi_path = expect_string(midi_val, "midi_lade")?;

        let sf = if sf_val.tag == MDH_TAG_NIL {
            if let Ok(mut mixer) = state.mixer() {
                if let Some(sf) = &mixer.default_soundfont {
                    Arc::clone(sf)
                } else {
                    let path = resolve_default_soundfont()?;
                    let sf = load_soundfont(path.as_path())?;
                    mixer.default_soundfont = Some(Arc::clone(&sf));
                    sf
                }
            } else {
                return Err(ERR_NO_DEVICE.to_string());
            }
        } else if sf_val.tag == MDH_TAG_STRING {
            let path = unsafe { mdh_string_to_rust(sf_val) };
            load_soundfont(Path::new(&path))?
        } else {
            return Err("midi_lade needs a soondfont path or naething".to_string());
        };

        let mut midi_file = match File::open(&midi_path) {
            Ok(f) => f,
            Err(_) => return Err("Cannae open the midi file".to_string()),
        };
        let midi = match MidiFile::new(&mut midi_file) {
            Ok(m) => m,
            Err(_) => return Err("Cannae read the midi".to_string()),
        };
        let midi = Arc::new(midi);

        let settings = SynthesizerSettings::new(OUTPUT_SAMPLE_RATE as i32);
        let synth = match Synthesizer::new(&sf, &settings) {
            Ok(s) => s,
            Err(_) => return Err("Cannae set up the synth".to_string()),
        };
        let sequencer = MidiFileSequencer::new(synth);

        state.ensure_audio()?;

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

        let mut mixer = state.mixer()?;
        let handle = AudioState::alloc_handle(&mut mixer.midi, entry);
        Ok(unsafe { __mdh_make_int(handle) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_spiel(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_spiel")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        if entry.state == PlayState::Stopped {
            entry.sequencer.play(&entry.midi, entry.looped);
        }
        entry.state = PlayState::Playing;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_haud(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_haud")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Paused;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_gae_on(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_gae_on")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.state = PlayState::Playing;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_stap(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_stap")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.sequencer.stop();
        entry.state = PlayState::Stopped;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_unlade(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_unlade")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.sequencer.stop();
        mixer.midi[handle] = None;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_is_spielin(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_is_spielin")?;
        let mixer = state.mixer()?;
        let entry = match mixer.midi.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        Ok(unsafe { __mdh_make_bool(entry.state == PlayState::Playing) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_loup(handle_val: MdhValue, seconds_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_loup")?;
        let pos = expect_number(seconds_val, "midi_loup")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        seek_midi(entry, pos)?;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_hou_lang(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_hou_lang")?;
        let mixer = state.mixer()?;
        let entry = match mixer.midi.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        Ok(unsafe { __mdh_make_float(entry.midi.get_length() as f64) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_whaur(handle_val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_whaur")?;
        let mixer = state.mixer()?;
        let entry = match mixer.midi.get(handle).and_then(|e| e.as_ref()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        Ok(unsafe { __mdh_make_float(entry.sequencer.get_position() as f64) })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_luid(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_pit_luid")?;
        let mut value = expect_number(val, "midi_pit_luid")? as f32;
        value = clamp01(value);
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.volume = value;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_pan(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_pit_pan")?;
        let pan = expect_number(val, "midi_pit_pan")? as f32;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.pan = pan;
        Ok(unsafe { __mdh_make_nil() })
    })
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_rin_roond(handle_val: MdhValue, val: MdhValue) -> MdhValue {
    with_state(|state| {
        let handle = expect_handle(handle_val, "midi_pit_rin_roond")?;
        let looped = expect_bool(val, "midi_pit_rin_roond")?;
        let mut mixer = state.mixer()?;
        let entry = match mixer.midi.get_mut(handle).and_then(|e| e.as_mut()) {
            Some(entry) => entry,
            None => return Err(ERR_BAD_HANDLE.to_string()),
        };
        entry.looped = looped;
        Ok(unsafe { __mdh_make_nil() })
    })
}
