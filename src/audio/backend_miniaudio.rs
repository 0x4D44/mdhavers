//! miniaudio audio backend
//!
//! This backend is used when the `audio` feature is enabled without `graphics`.
//! It provides full audio functionality using the miniaudio library.

use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::value::{NativeFunction, Value};

use super::{AudioError, AudioResult, PlayState, OUTPUT_CHANNELS, OUTPUT_SAMPLE_RATE};

// ============================================================================
// Test-only backend shims
// ============================================================================

#[cfg(test)]
mod miniaudio {
    use std::cell::Cell;
    use std::marker::PhantomData;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    type DataCallback = dyn FnMut(&Device, &mut FramesMut, &mut FramesMut) + Send;

    thread_local! {
        static FAIL_NEXT_DEVICE_NEW: Cell<bool> = Cell::new(false);
        static FAIL_NEXT_DEVICE_START: Cell<bool> = Cell::new(false);
    }

    pub(super) fn fail_next_device_new() {
        FAIL_NEXT_DEVICE_NEW.with(|flag| flag.set(true));
    }

    pub(super) fn fail_next_device_start() {
        FAIL_NEXT_DEVICE_START.with(|flag| flag.set(true));
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Format {
        F32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DeviceType {
        Playback,
    }

    pub struct PlaybackConfig {
        format: Format,
        channels: u32,
    }

    impl PlaybackConfig {
        pub fn set_format(&mut self, format: Format) {
            self.format = format;
        }

        pub fn set_channels(&mut self, channels: u32) {
            self.channels = channels;
        }
    }

    pub struct DeviceConfig {
        sample_rate: u32,
        playback: PlaybackConfig,
        callback: Option<Arc<Mutex<Box<DataCallback>>>>,
    }

    impl DeviceConfig {
        pub fn new(_device_type: DeviceType) -> Self {
            Self {
                sample_rate: 44_100,
                playback: PlaybackConfig {
                    format: Format::F32,
                    channels: 2,
                },
                callback: None,
            }
        }

        pub fn set_sample_rate(&mut self, sample_rate: u32) {
            self.sample_rate = sample_rate;
        }

        pub fn playback_mut(&mut self) -> &mut PlaybackConfig {
            &mut self.playback
        }

        pub fn set_data_callback<F>(&mut self, callback: F)
        where
            F: FnMut(&Device, &mut FramesMut, &mut FramesMut) + Send + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }
    }

    pub struct Device {
        callback: Option<Arc<Mutex<Box<DataCallback>>>>,
        channels: u32,
    }

    impl Device {
        pub fn new(_ctx: Option<()>, config: &DeviceConfig) -> Result<Self, ()> {
            let fail = FAIL_NEXT_DEVICE_NEW.with(|flag| {
                let value = flag.get();
                flag.set(false);
                value
            });
            if fail {
                return Err(());
            }
            Ok(Device {
                callback: config.callback.clone(),
                channels: config.playback.channels,
            })
        }

        pub fn start(&self) -> Result<(), ()> {
            let fail = FAIL_NEXT_DEVICE_START.with(|flag| {
                let value = flag.get();
                flag.set(false);
                value
            });
            if fail {
                return Err(());
            }
            let Some(callback) = &self.callback else {
                return Ok(());
            };

            let mut output_samples = vec![0.0f32; self.channels as usize * 2];
            let mut output = FramesMut::wrap(&mut output_samples, Format::F32, self.channels);

            let mut input_samples = Vec::new();
            let mut input = FramesMut::wrap(&mut input_samples, Format::F32, self.channels);

            if let Ok(mut cb) = callback.lock() {
                (cb.as_mut())(self, &mut output, &mut input);
            }
            Ok(())
        }

        pub fn stop(&self) -> Result<(), ()> {
            Ok(())
        }
    }

    pub struct DecoderConfig {
        _format: Format,
        _channels: u32,
        _sample_rate: u32,
    }

    impl DecoderConfig {
        pub fn new(format: Format, channels: u32, sample_rate: u32) -> Self {
            Self {
                _format: format,
                _channels: channels,
                _sample_rate: sample_rate,
            }
        }
    }

    pub struct Decoder {
        frames_left: u64,
        _marker: PhantomData<()>,
    }

    impl Decoder {
        pub fn from_file<P: AsRef<Path>>(
            _path: P,
            _config: Option<&DecoderConfig>,
        ) -> Result<Self, ()> {
            if !_path.as_ref().exists() {
                return Err(());
            }
            Ok(Self {
                frames_left: 1,
                _marker: PhantomData,
            })
        }

        pub fn read_pcm_frames(&mut self, frames: &mut FramesMut) -> u64 {
            if self.frames_left == 0 {
                return 0;
            }
            let count = self.frames_left.min(frames.frame_count() as u64);
            self.frames_left -= count;
            for sample in frames.as_samples_mut::<f32>().iter_mut() {
                *sample = 0.1;
            }
            count
        }
    }

    pub struct FramesMut<'a> {
        data: &'a mut [f32],
        channels: u32,
    }

    impl<'a> FramesMut<'a> {
        pub fn wrap(data: &'a mut [f32], _format: Format, channels: u32) -> Self {
            Self { data, channels }
        }

        pub fn frame_count(&self) -> usize {
            if self.channels == 0 {
                return 0;
            }
            self.data.len() / self.channels as usize
        }

        pub fn as_samples_mut<T>(&mut self) -> &mut [T] {
            unsafe {
                std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.data.len())
            }
        }
    }
}

#[cfg(not(test))]
use miniaudio::{Decoder, DecoderConfig, Device, DeviceConfig, DeviceType, Format, FramesMut};

#[cfg(test)]
use self::miniaudio::{
    Decoder, DecoderConfig, Device, DeviceConfig, DeviceType, Format, FramesMut,
};

// ============================================================================
// Constants
// ============================================================================

const DECODE_CHUNK_FRAMES: usize = 1_024;
const ERR_BAD_HANDLE: &str = "Thon handle isnae guid";

// ============================================================================
// Core Types
// ============================================================================

#[derive(Clone, Debug)]
struct SampleBuffer {
    samples: Arc<Vec<f32>>,
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

struct MixerState {
    master_volume: f32,
    muted: bool,
    sounds: Vec<Option<SoundEntry>>,
    music: Vec<Option<MusicEntry>>,
}

impl MixerState {
    fn new() -> Self {
        Self {
            master_volume: 1.0,
            muted: false,
            sounds: Vec::new(),
            music: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.master_volume = 1.0;
        self.muted = false;
        self.sounds.clear();
        self.music.clear();
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

        let device =
            Device::new(None, &config).map_err(|_| "Cannae stairt the soond device".to_string())?;
        device
            .start()
            .map_err(|_| "Cannae stairt the soond device".to_string())?;
        self.device = Some(device);
        Ok(())
    }

    fn mixer(&self) -> std::sync::MutexGuard<'_, MixerState> {
        match self.shared.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
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

// ============================================================================
// Helper Functions
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

    let master = if state.muted {
        0.0
    } else {
        state.master_volume
    };
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
                position %= total_frames as f64;
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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn with_state<F>(func: F) -> Result<Value, String>
where
    F: FnOnce(&mut AudioState) -> Result<Value, String>,
{
    AUDIO_STATE.with(|state| func(&mut state.borrow_mut()))
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
// Public API Types (for re-export)
// ============================================================================

/// Audio engine using miniaudio
pub struct AudioEngine;

impl AudioEngine {
    pub fn new() -> AudioResult<Self> {
        with_state(|state| {
            state.ensure_audio()?;
            Ok(Value::Nil)
        })
        .map_err(AudioError::new)?;
        Ok(AudioEngine)
    }

    pub fn shutdown() {
        with_state(|state| {
            state.shutdown();
            Ok(Value::Nil)
        })
        .ok();
    }
}

/// Sound handle (short audio clips)
pub struct Sound;

impl Sound {
    pub fn load(path: &str) -> AudioResult<i64> {
        with_state(|state| {
            state.ensure_audio()?;
            let buffer = decode_audio(path, "Cannae lade the soond")?;
            let entry = SoundEntry {
                buffer,
                position: 0.0,
                state: PlayState::Stopped,
                looped: false,
                volume: 1.0,
                pan: 0.0,
                pitch: 1.0,
            };
            let mut mixer = state.mixer();
            let handle = AudioState::alloc_handle(&mut mixer.sounds, entry);
            Ok(Value::Integer(handle))
        })
        .map(|v| match v {
            Value::Integer(h) => h,
            _ => -1,
        })
        .map_err(AudioError::new)
    }
}

/// Music handle (streaming audio)
pub struct Music;

impl Music {
    pub fn load(path: &str) -> AudioResult<i64> {
        with_state(|state| {
            state.ensure_audio()?;
            let buffer = decode_audio(path, "Cannae lade the muisic")?;
            let entry = MusicEntry {
                buffer,
                position: 0.0,
                state: PlayState::Stopped,
                looped: false,
                volume: 1.0,
                pan: 0.0,
                pitch: 1.0,
            };
            let mut mixer = state.mixer();
            let handle = AudioState::alloc_handle(&mut mixer.music, entry);
            Ok(Value::Integer(handle))
        })
        .map(|v| match v {
            Value::Integer(h) => h,
            _ => -1,
        })
        .map_err(AudioError::new)
    }
}

// ============================================================================
// Builtin Function Registration
// ============================================================================

/// Register sound and music builtin functions for the interpreter
pub fn register_builtin_functions(globals: &Rc<RefCell<crate::value::Environment>>) {
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
            state.ensure_audio()?;
            let mut mixer = state.mixer();
            mixer.muted = wheesht;
            Ok(Value::Nil)
        })
    });

    // soond_luid
    define_native(globals, "soond_luid", 1, |args| {
        with_state(|state| {
            let mut value = as_number(&args[0], "soond_luid")? as f32;
            value = clamp01(value);
            state.ensure_audio()?;
            let mut mixer = state.mixer();
            mixer.master_volume = value;
            Ok(Value::Nil)
        })
    });

    // soond_hou_luid
    define_native(globals, "soond_hou_luid", 0, |_args| {
        with_state(|state| {
            let mixer = state.mixer();
            Ok(Value::Float(mixer.master_volume as f64))
        })
    });

    // soond_haud_gang
    define_native(globals, "soond_haud_gang", 0, |_args| {
        with_state(|_state| Ok(Value::Nil))
    });

    // soond_ready
    define_native(globals, "soond_ready", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_ready")?;
            let mixer = state.mixer();
            let entry = mixer
                .sounds
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let _ = entry;
            Ok(Value::Bool(true))
        })
    });

    // soond_lade
    define_native(globals, "soond_lade", 1, |args| {
        with_state(|state| {
            let path = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("soond_lade needs a string path".to_string()),
            };
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
            let mut mixer = state.mixer();
            let handle = AudioState::alloc_handle(&mut mixer.sounds, entry);
            Ok(Value::Integer(handle))
        })
    });

    // soond_spiel
    define_native(globals, "soond_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_spiel")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // soond_haud
    define_native(globals, "soond_haud", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_haud")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Paused;
            Ok(Value::Nil)
        })
    });

    // soond_gae_on
    define_native(globals, "soond_gae_on", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_gae_on")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // soond_stap
    define_native(globals, "soond_stap", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_stap")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Stopped;
            entry.position = 0.0;
            Ok(Value::Nil)
        })
    });

    // soond_unlade
    define_native(globals, "soond_unlade", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_unlade")?;
            let mut mixer = state.mixer();
            if handle >= mixer.sounds.len() || mixer.sounds[handle].is_none() {
                return Err(ERR_BAD_HANDLE.to_string());
            }
            mixer.sounds[handle] = None;
            Ok(Value::Nil)
        })
    });

    // soond_is_spielin
    define_native(globals, "soond_is_spielin", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_is_spielin")?;
            let mixer = state.mixer();
            let entry = mixer
                .sounds
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.state == PlayState::Playing))
        })
    });

    // soond_pit_luid
    define_native(globals, "soond_pit_luid", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_luid")?;
            let mut value = as_number(&args[1], "soond_pit_luid")? as f32;
            value = clamp01(value);
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value;
            Ok(Value::Nil)
        })
    });

    // soond_pit_pan
    define_native(globals, "soond_pit_pan", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_pan")?;
            let pan = as_number(&args[1], "soond_pit_pan")? as f32;
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan;
            Ok(Value::Nil)
        })
    });

    // soond_pit_tune
    define_native(globals, "soond_pit_tune", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_tune")?;
            let pitch = as_number(&args[1], "soond_pit_tune")? as f32;
            let mut mixer = state.mixer();
            let entry = mixer
                .sounds
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pitch = pitch;
            Ok(Value::Nil)
        })
    });

    // soond_pit_rin_roond
    define_native(globals, "soond_pit_rin_roond", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_pit_rin_roond")?;
            let looped = as_bool(&args[1], "soond_pit_rin_roond")?;
            let mut mixer = state.mixer();
            let entry = mixer
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
            let mut mixer = state.mixer();
            let handle = AudioState::alloc_handle(&mut mixer.music, entry);
            Ok(Value::Integer(handle))
        })
    });

    // muisic_spiel
    define_native(globals, "muisic_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_spiel")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // muisic_haud
    define_native(globals, "muisic_haud", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_haud")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Paused;
            Ok(Value::Nil)
        })
    });

    // muisic_gae_on
    define_native(globals, "muisic_gae_on", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_gae_on")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // muisic_stap
    define_native(globals, "muisic_stap", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_stap")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Stopped;
            entry.position = 0.0;
            Ok(Value::Nil)
        })
    });

    // muisic_unlade
    define_native(globals, "muisic_unlade", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_unlade")?;
            let mut mixer = state.mixer();
            if handle >= mixer.music.len() || mixer.music[handle].is_none() {
                return Err(ERR_BAD_HANDLE.to_string());
            }
            mixer.music[handle] = None;
            Ok(Value::Nil)
        })
    });

    // muisic_is_spielin
    define_native(globals, "muisic_is_spielin", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_is_spielin")?;
            let mixer = state.mixer();
            let entry = mixer
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.state == PlayState::Playing))
        })
    });

    // muisic_loup (seek)
    define_native(globals, "muisic_loup", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_loup")?;
            let seconds = as_number(&args[1], "muisic_loup")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let frame = (seconds * OUTPUT_SAMPLE_RATE as f64).max(0.0);
            entry.position = frame.min(entry.buffer.frames as f64);
            Ok(Value::Nil)
        })
    });

    // muisic_hou_lang (duration)
    define_native(globals, "muisic_hou_lang", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_hou_lang")?;
            let mixer = state.mixer();
            let entry = mixer
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let duration = entry.buffer.frames as f64 / OUTPUT_SAMPLE_RATE as f64;
            Ok(Value::Float(duration))
        })
    });

    // muisic_whaur (position)
    define_native(globals, "muisic_whaur", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_whaur")?;
            let mixer = state.mixer();
            let entry = mixer
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let position = entry.position / OUTPUT_SAMPLE_RATE as f64;
            Ok(Value::Float(position))
        })
    });

    // muisic_pit_luid
    define_native(globals, "muisic_pit_luid", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_luid")?;
            let mut value = as_number(&args[1], "muisic_pit_luid")? as f32;
            value = clamp01(value);
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value;
            Ok(Value::Nil)
        })
    });

    // muisic_pit_pan
    define_native(globals, "muisic_pit_pan", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_pan")?;
            let pan = as_number(&args[1], "muisic_pit_pan")? as f32;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan;
            Ok(Value::Nil)
        })
    });

    // muisic_pit_tune
    define_native(globals, "muisic_pit_tune", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_tune")?;
            let pitch = as_number(&args[1], "muisic_pit_tune")? as f32;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pitch = pitch;
            Ok(Value::Nil)
        })
    });

    // muisic_pit_rin_roond
    define_native(globals, "muisic_pit_rin_roond", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_rin_roond")?;
            let looped = as_bool(&args[1], "muisic_pit_rin_roond")?;
            let mut mixer = state.mixer();
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.looped = looped;
            Ok(Value::Nil)
        })
    });
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn audio_state_new() {
        let state = AudioState::new();
        assert!(state.device.is_none());
    }

    #[test]
    fn audio_state_ensure_audio() {
        let mut state = AudioState::new();
        assert!(state.ensure_audio().is_ok());
        assert!(state.device.is_some());
    }

    #[test]
    fn audio_state_ensure_audio_idempotent() {
        let mut state = AudioState::new();
        assert!(state.ensure_audio().is_ok());
        assert!(state.ensure_audio().is_ok());
    }

    #[test]
    fn audio_state_shutdown() {
        let mut state = AudioState::new();
        state.ensure_audio().unwrap();
        state.shutdown();
        assert!(state.device.is_none());
    }

    #[test]
    fn audio_state_device_new_failure() {
        miniaudio::fail_next_device_new();
        let mut state = AudioState::new();
        assert!(state.ensure_audio().is_err());
    }

    #[test]
    fn audio_state_device_start_failure() {
        miniaudio::fail_next_device_start();
        let mut state = AudioState::new();
        assert!(state.ensure_audio().is_err());
    }

    #[test]
    fn mixer_state_new() {
        let mixer = MixerState::new();
        assert_eq!(mixer.master_volume, 1.0);
        assert!(!mixer.muted);
        assert!(mixer.sounds.is_empty());
        assert!(mixer.music.is_empty());
    }

    #[test]
    fn mixer_state_reset() {
        let mut mixer = MixerState::new();
        mixer.master_volume = 0.5;
        mixer.muted = true;
        mixer.reset();
        assert_eq!(mixer.master_volume, 1.0);
        assert!(!mixer.muted);
    }

    #[test]
    fn alloc_handle_reuses_slots() {
        let mut slots: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
        let handle = AudioState::alloc_handle(&mut slots, 2);
        assert_eq!(handle, 1);
        assert_eq!(slots[1], Some(2));
    }

    #[test]
    fn alloc_handle_appends_when_full() {
        let mut slots: Vec<Option<i32>> = vec![Some(1), Some(2)];
        let handle = AudioState::alloc_handle(&mut slots, 3);
        assert_eq!(handle, 2);
        assert_eq!(slots.len(), 3);
    }

    #[test]
    fn as_number_float() {
        let val = Value::Float(1.5);
        assert_eq!(as_number(&val, "test").unwrap(), 1.5);
    }

    #[test]
    fn as_number_integer() {
        let val = Value::Integer(42);
        assert_eq!(as_number(&val, "test").unwrap(), 42.0);
    }

    #[test]
    fn as_number_invalid() {
        let val = Value::String("not a number".to_string());
        assert!(as_number(&val, "test").is_err());
    }

    #[test]
    fn as_bool_true() {
        let val = Value::Bool(true);
        assert!(as_bool(&val, "test").unwrap());
    }

    #[test]
    fn as_bool_invalid() {
        let val = Value::Integer(1);
        assert!(as_bool(&val, "test").is_err());
    }

    #[test]
    fn as_handle_valid() {
        let val = Value::Integer(5);
        assert_eq!(as_handle(&val, "test").unwrap(), 5);
    }

    #[test]
    fn as_handle_negative() {
        let val = Value::Integer(-1);
        assert!(as_handle(&val, "test").is_err());
    }

    #[test]
    fn clamp01_in_range() {
        assert_eq!(clamp01(0.5), 0.5);
    }

    #[test]
    fn clamp01_below() {
        assert_eq!(clamp01(-0.5), 0.0);
    }

    #[test]
    fn clamp01_above() {
        assert_eq!(clamp01(1.5), 1.0);
    }

    #[test]
    fn pan_gains_center() {
        let (l, r) = pan_gains(0.0);
        assert!((l - r).abs() < 0.01);
    }

    #[test]
    fn pan_gains_left() {
        let (l, r) = pan_gains(-1.0);
        assert!(l > r);
    }

    #[test]
    fn pan_gains_right() {
        let (l, r) = pan_gains(1.0);
        assert!(r > l);
    }

    #[test]
    fn lerp_basic() {
        assert_eq!(lerp(0.0, 1.0, 0.5), 0.5);
        assert_eq!(lerp(0.0, 1.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn decode_audio_nonexistent_file() {
        let result = decode_audio("nonexistent.wav", "error");
        assert!(result.is_err());
    }

    #[test]
    fn decode_audio_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.wav");
        std::fs::write(&path, b"RIFF....").unwrap();
        let result = decode_audio(path.to_str().unwrap(), "error");
        assert!(result.is_ok());
    }

    #[test]
    fn mix_buffer_entry_stopped() {
        let buffer = SampleBuffer {
            samples: Arc::new(vec![0.5, 0.5]),
            frames: 1,
        };
        let mut entry = BufferEntry {
            buffer,
            position: 0.0,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };
        let mut output = vec![0.0; 4];
        mix_buffer_entry(&mut entry, &mut output, 2, 2);
        assert_eq!(output, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn mix_buffer_entry_empty_buffer() {
        let buffer = SampleBuffer {
            samples: Arc::new(vec![]),
            frames: 0,
        };
        let mut entry = BufferEntry {
            buffer,
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };
        let mut output = vec![0.0; 4];
        mix_buffer_entry(&mut entry, &mut output, 2, 2);
        assert_eq!(entry.state, PlayState::Stopped);
    }
}
