//! Audio module for mdhavers using miniaudio + RustySynth
//!
//! Provides Scots-only audio API for sounds, music, and MIDI.

// Test-only backend shims so we can exercise audio logic without a real
// audio device or external libraries.
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

    #[cfg(test)]
	    mod tests {
	        use super::*;

	        #[test]
	        fn device_start_skips_poisoned_callback_lock_for_coverage() {
	            let mut config = DeviceConfig::new(DeviceType::Playback);
	            config.playback_mut().set_channels(2);
	            config.set_data_callback(|_device, _output, _input| {});

	            let device = Device::new(None, &config).expect("device");
	            device.start().expect("start");

	            let callback = config.callback.clone().expect("callback");
	            let callback_clone = Arc::clone(&callback);
	            let _ = std::thread::spawn(move || {
	                let _guard = callback_clone.lock().expect("lock callback");
	                panic!("poison callback");
	            })
	            .join();

	            device.start().expect("start");
	        }
	    }
}

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

#[cfg(any(feature = "audio", test))]
use crate::value::{NativeFunction, Value};
#[cfg(any(feature = "audio", test))]
use miniaudio::{Decoder, DecoderConfig, Device, DeviceConfig, DeviceType, Format, FramesMut};
#[cfg(any(feature = "audio", test))]
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
#[cfg(any(feature = "audio", test))]
use std::cell::RefCell;
#[cfg(not(any(feature = "audio", test)))]
use std::cell::RefCell;
#[cfg(any(feature = "audio", test))]
use std::f32::consts::FRAC_PI_2;
#[cfg(any(feature = "audio", test))]
use std::fs::File;
#[cfg(any(feature = "audio", test))]
use std::path::{Path, PathBuf};
#[cfg(any(feature = "audio", test))]
use std::rc::Rc;
#[cfg(not(any(feature = "audio", test)))]
use std::rc::Rc;
#[cfg(any(feature = "audio", test))]
use std::sync::{Arc, Mutex};

#[cfg(any(feature = "audio", test))]
const DEFAULT_SOUNDFONT_PATH: &str = "assets/soundfonts/MuseScore_General.sf2";
#[cfg(any(feature = "audio", test))]
const OUTPUT_SAMPLE_RATE: u32 = 44_100;
#[cfg(any(feature = "audio", test))]
const OUTPUT_CHANNELS: u32 = 2;
#[cfg(any(feature = "audio", test))]
const DECODE_CHUNK_FRAMES: usize = 1_024;

#[cfg(not(any(feature = "audio", test)))]
const ERR_NO_AUDIO: &str = "Soond isnae available - build wi' --features audio";
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
#[derive(Clone, Debug)]
struct SampleBuffer {
    samples: Arc<Vec<f32>>,
    frames: usize,
}

#[cfg(any(feature = "audio", test))]
struct BufferEntry {
    buffer: SampleBuffer,
    position: f64,
    state: PlayState,
    looped: bool,
    volume: f32,
    pan: f32,
    pitch: f32,
}

#[cfg(any(feature = "audio", test))]
type SoundEntry = BufferEntry;
#[cfg(any(feature = "audio", test))]
type MusicEntry = BufferEntry;

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
struct MixerState {
    master_volume: f32,
    muted: bool,
    sounds: Vec<Option<SoundEntry>>,
    music: Vec<Option<MusicEntry>>,
    midi: Vec<Option<MidiEntry>>,
    default_soundfont: Option<Arc<SoundFont>>,
}

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
struct AudioState {
    device: Option<Device>,
    shared: Arc<Mutex<MixerState>>,
}

#[cfg(any(feature = "audio", test))]
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
    value.clamp(0.0, 1.0)
}

#[cfg(any(feature = "audio", test))]
fn pan_gains(pan: f32) -> (f32, f32) {
    let clamped = pan.clamp(-1.0, 1.0);
    let t = (clamped + 1.0) * 0.5;
    let angle = t * FRAC_PI_2;
    (angle.cos(), angle.sin())
}

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
fn load_soundfont(path: &Path) -> Result<Arc<SoundFont>, String> {
    let mut file = File::open(path).map_err(|_| "Cannae open the soondfont file".to_string())?;
    let sf = SoundFont::new(&mut file).map_err(|_| "Cannae read the soondfont".to_string())?;
    Ok(Arc::new(sf))
}

#[cfg(any(feature = "audio", test))]
fn current_exe_for_soundfont_candidates() -> std::io::Result<PathBuf> {
    if cfg!(test) && std::env::var_os("MDHAVERS_TEST_FORCE_CURRENT_EXE_ERROR").is_some() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "forced current_exe failure",
        ))
    } else {
        std::env::current_exe()
    }
}

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
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

#[cfg(any(feature = "audio", test))]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(any(feature = "audio", test))]
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
            state.ensure_audio()?;
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
            mixer.master_volume = value;
            Ok(Value::Nil)
        })
    });

    // soond_hou_luid
    define_native(globals, "soond_hou_luid", 0, |_args| {
        with_state(|state| {
            let mixer = state.mixer()?;
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
            let mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
            let handle = AudioState::alloc_handle(&mut mixer.sounds, entry);
            Ok(Value::Integer(handle))
        })
    });

    // soond_spiel
    define_native(globals, "soond_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "soond_spiel")?;
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
            let handle = AudioState::alloc_handle(&mut mixer.music, entry);
            Ok(Value::Integer(handle))
        })
    });

    // muisic_spiel
    define_native(globals, "muisic_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_spiel")?;
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mixer = state.mixer()?;
            let entry = mixer
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.state == PlayState::Playing))
        })
    });

    // muisic_loup
    define_native(globals, "muisic_loup", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_loup")?;
            let pos = as_number(&args[1], "muisic_loup")? as f64;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .music
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let target = (pos * OUTPUT_SAMPLE_RATE as f64).max(0.0);
            entry.position = target.min(entry.buffer.frames as f64);
            Ok(Value::Nil)
        })
    });

    // muisic_hou_lang
    define_native(globals, "muisic_hou_lang", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_hou_lang")?;
            let mixer = state.mixer()?;
            let entry = mixer
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let length = entry.buffer.frames as f64 / OUTPUT_SAMPLE_RATE as f64;
            Ok(Value::Float(length))
        })
    });

    // muisic_whaur
    define_native(globals, "muisic_whaur", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_whaur")?;
            let mixer = state.mixer()?;
            let entry = mixer
                .music
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            let pos = entry.position / OUTPUT_SAMPLE_RATE as f64;
            Ok(Value::Float(pos))
        })
    });

    // muisic_pit_luid
    define_native(globals, "muisic_pit_luid", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "muisic_pit_luid")?;
            let mut value = as_number(&args[1], "muisic_pit_luid")? as f32;
            value = clamp01(value);
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
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
            let mut mixer = state.mixer()?;
            let entry = mixer
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

            state.ensure_audio()?;

            let sf = match &args[1] {
                Value::Nil => {
                    let existing = {
                        let mixer = state.mixer()?;
                        mixer.default_soundfont.clone()
                    };
                    if let Some(sf) = existing {
                        sf
                    } else {
                        let path = resolve_default_soundfont()?;
                        let sf = load_soundfont(path.as_path())?;
                        let mut mixer = state.mixer()?;
                        mixer
                            .default_soundfont
                            .get_or_insert_with(|| Arc::clone(&sf));
                        sf
                    }
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
            let synth = Synthesizer::new(&sf, &settings)
                .map_err(|_| "Cannae set up the synth".to_string())?;
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

            let mut mixer = state.mixer()?;
            let handle = AudioState::alloc_handle(&mut mixer.midi, entry);
            Ok(Value::Integer(handle))
        })
    });

    // midi_spiel
    define_native(globals, "midi_spiel", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_spiel")?;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            if entry.state == PlayState::Stopped {
                entry.sequencer.play(&entry.midi, entry.looped);
            }
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // midi_haud
    define_native(globals, "midi_haud", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_haud")?;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Paused;
            Ok(Value::Nil)
        })
    });

    // midi_gae_on
    define_native(globals, "midi_gae_on", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_gae_on")?;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.state = PlayState::Playing;
            Ok(Value::Nil)
        })
    });

    // midi_stap
    define_native(globals, "midi_stap", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_stap")?;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.sequencer.stop();
            entry.state = PlayState::Stopped;
            Ok(Value::Nil)
        })
    });

    // midi_unlade
    define_native(globals, "midi_unlade", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_unlade")?;
            let mut mixer = state.mixer()?;
            if handle >= mixer.midi.len() || mixer.midi[handle].is_none() {
                return Err(ERR_BAD_HANDLE.to_string());
            }
            mixer.midi[handle] = None;
            Ok(Value::Nil)
        })
    });

    // midi_is_spielin
    define_native(globals, "midi_is_spielin", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_is_spielin")?;
            let mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get(handle)
                .and_then(|e| e.as_ref())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            Ok(Value::Bool(entry.state == PlayState::Playing))
        })
    });

    // midi_loup
    define_native(globals, "midi_loup", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_loup")?;
            let pos = as_number(&args[1], "midi_loup")?;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            seek_midi(entry, pos)?;
            Ok(Value::Nil)
        })
    });

    // midi_hou_lang
    define_native(globals, "midi_hou_lang", 1, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_hou_lang")?;
            let mixer = state.mixer()?;
            let entry = mixer
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
            let mixer = state.mixer()?;
            let entry = mixer
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
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.volume = value;
            Ok(Value::Nil)
        })
    });

    // midi_pit_pan
    define_native(globals, "midi_pit_pan", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_pit_pan")?;
            let pan = as_number(&args[1], "midi_pit_pan")? as f32;
            let mut mixer = state.mixer()?;
            let entry = mixer
                .midi
                .get_mut(handle)
                .and_then(|e| e.as_mut())
                .ok_or_else(|| ERR_BAD_HANDLE.to_string())?;
            entry.pan = pan;
            Ok(Value::Nil)
        })
    });

    // midi_pit_rin_roond
    define_native(globals, "midi_pit_rin_roond", 2, |args| {
        with_state(|state| {
            let handle = as_handle(&args[0], "midi_pit_rin_roond")?;
            let looped = as_bool(&args[1], "midi_pit_rin_roond")?;
            let mut mixer = state.mixer()?;
            let entry = mixer
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
    use crate::value::{Environment, NativeFunction};
    use std::fs;
    use std::io::Cursor;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
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

    fn sample_buffer(frames: usize, left: f32, right: f32) -> SampleBuffer {
        let mut samples = Vec::with_capacity(frames * 2);
        for _ in 0..frames {
            samples.push(left);
            samples.push(right);
        }
        SampleBuffer {
            samples: Arc::new(samples),
            frames,
        }
    }

    fn get_native(env: &Rc<RefCell<Environment>>, name: &str) -> Rc<NativeFunction> {
        let value = env.borrow().get(name).unwrap();
        match value {
            Value::NativeFunction(func) => func,
            _ => panic!("expected native function {}", name),
        }
    }

    fn restore_env_var(key: &str, value: Option<std::ffi::OsString>) {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn test_miniaudio_device_start_invokes_callback() {
        let mut config = miniaudio::DeviceConfig::new(miniaudio::DeviceType::Playback);
        config.playback_mut().set_channels(2);

        let called = Arc::new(AtomicUsize::new(0));
        let called_for_cb = called.clone();
        config.set_data_callback(move |_device, output, _input| {
            called_for_cb.fetch_add(1, Ordering::Relaxed);
            for sample in output.as_samples_mut::<f32>() {
                *sample = 0.0;
            }
        });

        let device = miniaudio::Device::new(None, &config).unwrap();
        assert!(device.start().is_ok());
        assert_eq!(called.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_miniaudio_device_start_without_callback_is_ok() {
        let mut config = miniaudio::DeviceConfig::new(miniaudio::DeviceType::Playback);
        config.playback_mut().set_channels(2);

        let device = miniaudio::Device::new(None, &config).unwrap();
        assert!(device.start().is_ok());
    }

    #[test]
    #[should_panic(expected = "expected native function")]
    fn test_get_native_panics_on_non_native_value() {
        let env = Rc::new(RefCell::new(Environment::new()));
        env.borrow_mut().define("x".to_string(), Value::Integer(1));
        let _ = get_native(&env, "x");
    }

    #[test]
    fn test_clamp_and_pan_helpers() {
        assert!((clamp01(-0.1) - 0.0).abs() < 1e-6);
        assert!((clamp01(0.5) - 0.5).abs() < 1e-6);
        assert!((clamp01(1.5) - 1.0).abs() < 1e-6);

        let (l, r) = pan_gains(-1.0);
        assert!((l - 1.0).abs() < 1e-6);
        assert!((r - 0.0).abs() < 1e-6);

        let (l, r) = pan_gains(0.0);
        assert!((l - r).abs() < 1e-6);

        let (l, r) = pan_gains(1.0);
        assert!((l - 0.0).abs() < 1e-6);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_value_parsing_helpers() {
        assert!((as_number(&Value::Integer(3), "num").unwrap() - 3.0).abs() < 1e-6);
        assert!((as_number(&Value::Float(2.5), "num").unwrap() - 2.5).abs() < 1e-6);
        assert!(as_number(&Value::Bool(true), "num").is_err());

        assert!(as_bool(&Value::Bool(true), "bool").unwrap());
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
    fn test_alloc_handle_reuse_buffer_entry_instantiation() {
        let entry = BufferEntry {
            buffer: sample_buffer(1, 0.0, 0.0),
            position: 0.0,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };

        let mut slots: Vec<Option<BufferEntry>> = vec![None];
        let handle = AudioState::alloc_handle(&mut slots, entry);
        assert_eq!(handle, 0);
        assert!(slots[0].is_some());
    }

    #[test]
    fn test_alloc_handle_reuse_midi_entry_instantiation() {
        let mut sf_data = Cursor::new(Vec::new());
        let soundfont = SoundFont::new(&mut sf_data).unwrap();
        let settings = SynthesizerSettings::new(OUTPUT_SAMPLE_RATE as i32);
        let synth = Synthesizer::new(&soundfont, &settings).unwrap();
        let sequencer = MidiFileSequencer::new(synth);

        let mut midi_data = Cursor::new(Vec::new());
        let midi = Arc::new(MidiFile::new(&mut midi_data).unwrap());

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

        let mut slots: Vec<Option<MidiEntry>> = vec![None];
        let handle = AudioState::alloc_handle(&mut slots, entry);
        assert_eq!(handle, 0);
        assert!(slots[0].is_some());
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
    fn test_resolve_default_soundfont_current_exe_error_branch() {
        let dir = tempdir().unwrap();
        let sf_dir = dir.path().join("assets/soundfonts");
        fs::create_dir_all(&sf_dir).unwrap();
        let sf_path = sf_dir.join("MuseScore_General.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        let _lock = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let key = "MDHAVERS_TEST_FORCE_CURRENT_EXE_ERROR";
        let pre_test_value = std::env::var_os(key);
        std::env::set_var(key, "preexisting");
        let original_value = std::env::var_os(key);
        std::env::set_var(key, "1");

	        let found = resolve_default_soundfont().unwrap();
	        assert_eq!(found, sf_path);

	        restore_env_var(key, original_value);
	        restore_env_var(key, pre_test_value);

	        std::env::set_current_dir(original_dir).unwrap();
	    }

    #[test]
    fn test_resolve_default_soundfont_current_exe_error_branch_restores_existing_env_var() {
        let dir = tempdir().unwrap();
        let sf_dir = dir.path().join("assets/soundfonts");
        fs::create_dir_all(&sf_dir).unwrap();
        let sf_path = sf_dir.join("MuseScore_General.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        let _lock = CWD_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let key = "MDHAVERS_TEST_FORCE_CURRENT_EXE_ERROR";
        let pre_test_value = std::env::var_os(key);
        std::env::set_var(key, "actual");
        let actual_original = std::env::var_os(key);
        std::env::remove_var(key);
        let original_value = std::env::var_os(key);
        std::env::set_var(key, "1");

	        let found = resolve_default_soundfont().unwrap();
	        assert_eq!(found, sf_path);

	        restore_env_var(key, original_value);
	        restore_env_var(key, actual_original);
	        restore_env_var(key, pre_test_value);

	        std::env::set_current_dir(original_dir).unwrap();
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
    fn test_load_soundfont_maps_soundfont_new_error() {
        let dir = tempdir().unwrap();
        let sf_path = dir.path().join("test.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        rustysynth::fail_next_soundfont_new();
        let err = load_soundfont(sf_path.as_path()).err().unwrap();
        assert_eq!(err, "Cannae read the soondfont");
    }

    #[test]
    fn test_decode_audio_missing_file() {
        let err = decode_audio("nope.wav", "Cannae lade the soond").unwrap_err();
        assert_eq!(err, "Cannae lade the soond");
    }

    #[test]
    fn test_audio_state_ensure_and_shutdown() {
        let mut state = AudioState::new();
        state.ensure_audio().unwrap();
        {
            let mut mixer = state.mixer().unwrap();
            mixer.master_volume = 0.2;
            mixer.sounds.push(Some(SoundEntry {
                buffer: sample_buffer(1, 0.5, 0.5),
                position: 0.0,
                state: PlayState::Playing,
                looped: false,
                volume: 1.0,
                pan: 0.0,
                pitch: 1.0,
            }));
        }
        state.shutdown();
        assert!(state.device.is_none());
        let mixer = state.mixer().unwrap();
        assert!(mixer.sounds.is_empty());
        assert!(mixer.music.is_empty());
        assert!(mixer.midi.is_empty());
        assert_eq!(mixer.master_volume, 1.0);
    }

    #[test]
    fn test_audio_state_ensure_audio_maps_device_new_error() {
        miniaudio::fail_next_device_new();
        let mut state = AudioState::new();
        let err = state.ensure_audio().unwrap_err();
        assert_eq!(err, "Cannae stairt the soond device");
    }

    #[test]
    fn test_audio_state_ensure_audio_maps_device_start_error() {
        miniaudio::fail_next_device_start();
        let mut state = AudioState::new();
        let err = state.ensure_audio().unwrap_err();
        assert_eq!(err, "Cannae stairt the soond device");
    }

    #[test]
    fn test_mix_buffer_entry_stops_and_advances() {
        let buffer = sample_buffer(4, 1.0, 0.0);
        let mut entry = BufferEntry {
            buffer,
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: -1.0,
            pitch: 1.0,
        };
        let mut output = vec![0.0_f32; 12];
        mix_buffer_entry(&mut entry, &mut output, 6, 2);
        for frame in 0..4 {
            assert!((output[frame * 2] - 1.0).abs() < 1e-6);
            assert!((output[frame * 2 + 1] - 0.0).abs() < 1e-6);
        }
        assert!(output[8].abs() < 1e-6);
        assert!(output[10].abs() < 1e-6);
        assert_eq!(entry.state, PlayState::Stopped);
        assert!(entry.position >= 4.0);
    }

    #[test]
    fn test_mix_buffer_entry_loops() {
        let buffer = sample_buffer(4, 1.0, 0.0);
        let mut entry = BufferEntry {
            buffer,
            position: 0.0,
            state: PlayState::Playing,
            looped: true,
            volume: 1.0,
            pan: -1.0,
            pitch: 1.0,
        };
        let mut output = vec![0.0_f32; 12];
        mix_buffer_entry(&mut entry, &mut output, 6, 2);
        for frame in 0..6 {
            assert!((output[frame * 2] - 1.0).abs() < 1e-6);
            assert!(output[frame * 2 + 1].abs() < 1e-6);
        }
        assert_eq!(entry.state, PlayState::Playing);
        assert!((entry.position - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_mix_state_master_volume_and_mute() {
        let mut state = MixerState::new();
        state.master_volume = 0.5;
        state.sounds.push(Some(SoundEntry {
            buffer: sample_buffer(1, 1.0, 1.0),
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: -1.0,
            pitch: 1.0,
        }));
        let mut output = vec![0.0_f32; 2];
        mix_state(&mut state, &mut output, 1);
        assert!((output[0] - 0.5).abs() < 1e-6);
        assert!((output[1]).abs() < 1e-6);

        let mut state = MixerState::new();
        state.muted = true;
        state.sounds.push(Some(SoundEntry {
            buffer: sample_buffer(1, 1.0, 1.0),
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: -1.0,
            pitch: 1.0,
        }));
        let mut output = vec![0.0_f32; 2];
        mix_state(&mut state, &mut output, 1);
        assert!((output[0]).abs() < 1e-6);
        assert!((output[1]).abs() < 1e-6);
    }

    #[test]
    fn test_mix_state_master_volume_one_leaves_samples_unscaled() {
        let mut state = MixerState::new();
        state.master_volume = 1.0;
        state.sounds.push(Some(SoundEntry {
            buffer: sample_buffer(1, 1.0, 1.0),
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: -1.0,
            pitch: 1.0,
        }));
        let mut output = vec![0.0_f32; 2];
        mix_state(&mut state, &mut output, 1);
        assert!((output[0] - 1.0).abs() < 1e-6);
        assert!((output[1]).abs() < 1e-6);
    }

    #[test]
    fn test_mix_state_skips_none_slots_for_coverage() {
        let mut state = MixerState::new();
        state.sounds = vec![None];
        state.music = vec![None];
        state.midi = vec![None];

        let mut output = vec![0.0_f32; 2];
        mix_state(&mut state, &mut output, 1);
        assert_eq!(output, vec![0.0, 0.0]);
    }

    #[test]
    fn test_mix_midi_entry_advances_and_stops() {
        let dir = tempdir().unwrap();
        let midi_path = dir.path().join("test.mid");
        let sf_path = dir.path().join("test.sf2");
        fs::write(&midi_path, b"midi").unwrap();
        fs::write(&sf_path, b"sf").unwrap();

        let sf = load_soundfont(sf_path.as_path()).unwrap();
        let mut midi_file = File::open(&midi_path).unwrap();
        let midi = MidiFile::new(&mut midi_file).unwrap();
        let midi = Arc::new(midi);

        let settings = SynthesizerSettings::new(10);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let mut sequencer = MidiFileSequencer::new(synth);
        sequencer.play(&midi, false);

        let mut entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: 10,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };

        let mut output = vec![0.0_f32; 4];
        mix_midi_entry(&mut entry, &mut output, 2, 2);
        assert_eq!(entry.state, PlayState::Stopped);
    }

    #[test]
    fn test_seek_midi_clamps() {
        let dir = tempdir().unwrap();
        let midi_path = dir.path().join("test.mid");
        let sf_path = dir.path().join("test.sf2");
        fs::write(&midi_path, b"midi").unwrap();
        fs::write(&sf_path, b"sf").unwrap();

        let sf = load_soundfont(sf_path.as_path()).unwrap();
        let mut midi_file = File::open(&midi_path).unwrap();
        let midi = MidiFile::new(&mut midi_file).unwrap();
        let midi = Arc::new(midi);

        let settings = SynthesizerSettings::new(10);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let sequencer = MidiFileSequencer::new(synth);

        let mut entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: 10,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };

        seek_midi(&mut entry, 5.0).unwrap();
        assert!((entry.sequencer.get_position() - entry.midi.get_length()).abs() < 1e-6);
    }

    #[test]
    fn test_frames_mut_helpers() {
        let mut data = vec![0.0_f32; 4];
        let frames = FramesMut::wrap(&mut data, Format::F32, 2);
        assert_eq!(frames.frame_count(), 2);

        let frames_zero = FramesMut::wrap(&mut data, Format::F32, 0);
        assert_eq!(frames_zero.frame_count(), 0);

        let mut frames = FramesMut::wrap(&mut data, Format::F32, 2);
        let samples = frames.as_samples_mut::<f32>();
        samples[0] = 0.25;
        assert!((data[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_decode_audio_success_reads_frames() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ok.wav");
        fs::write(&path, b"wav").unwrap();
        let buffer = decode_audio(path.to_str().unwrap(), "nope").unwrap();
        assert_eq!(buffer.frames, 1);
        assert_eq!(buffer.samples.len(), OUTPUT_CHANNELS as usize);
    }

    #[test]
    fn test_mix_output_try_lock_failure() {
        let state = Arc::new(Mutex::new(MixerState::new()));
        let mut data = vec![0.0_f32; 4];
        let mut frames = FramesMut::wrap(&mut data, Format::F32, OUTPUT_CHANNELS);
        let _guard = state.lock().unwrap();
        mix_output(&state, &mut frames);
    }

    #[test]
    fn test_midi_sequencer_render_branches() {
        let sf = SoundFont::new(&mut Cursor::new(Vec::new())).unwrap();
        let settings = SynthesizerSettings::new(10);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let mut seq = MidiFileSequencer::new(synth);
        let mut left = [1.0_f32; 2];
        let mut right = [1.0_f32; 2];

        seq.render(&mut left, &mut right);
        assert_eq!(left, [0.0, 0.0]);
        assert_eq!(right, [0.0, 0.0]);

        let midi = Arc::new(MidiFile::new(&mut Cursor::new(Vec::new())).unwrap());
        seq.play(&midi, true);
        seq.render(&mut left, &mut right);
        assert!(seq.get_position() < 1e-6);

        seq.play(&midi, false);
        seq.render(&mut left, &mut right);
        assert!(seq.end_of_sequence());
    }

    #[test]
    fn test_audio_state_mixer_poisoned() {
        let mut state = AudioState::new();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = state.shared.lock().unwrap();
            panic!("poison");
        }));
        let guard = state.mixer().unwrap();
        drop(guard);
        state.shutdown();
    }

    #[test]
    fn test_mix_output_success_path() {
        let mut state = MixerState::new();
        state.master_volume = 0.5;
        state.sounds.push(Some(SoundEntry {
            buffer: sample_buffer(1, 1.0, 0.0),
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: -1.0,
            pitch: 1.0,
        }));
        state.music.push(Some(MusicEntry {
            buffer: sample_buffer(1, 0.25, 0.25),
            position: 0.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        }));

        let sf = SoundFont::new(&mut Cursor::new(Vec::new())).unwrap();
        let settings = SynthesizerSettings::new(10);
        let synth = Synthesizer::new(&sf, &settings).unwrap();
        let midi = Arc::new(MidiFile::new(&mut Cursor::new(Vec::new())).unwrap());
        let mut sequencer = MidiFileSequencer::new(synth);
        sequencer.play(&midi, false);
        state.midi.push(Some(MidiEntry {
            midi,
            sequencer,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: 10,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        }));

        let shared = Arc::new(Mutex::new(state));
        let mut data = vec![0.0_f32; OUTPUT_CHANNELS as usize];
        let mut frames = FramesMut::wrap(&mut data, Format::F32, OUTPUT_CHANNELS);
        mix_output(&shared, &mut frames);
        assert!(data.iter().any(|v| v.abs() > 0.0));
    }

    #[test]
    fn test_mix_buffer_entry_early_returns() {
        let buffer = sample_buffer(1, 1.0, 0.5);
        let mut entry = BufferEntry {
            buffer,
            position: 0.0,
            state: PlayState::Paused,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };
        let mut output = vec![0.0_f32; 2];
        mix_buffer_entry(&mut entry, &mut output, 1, 2);
        assert_eq!(entry.state, PlayState::Paused);

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
        mix_buffer_entry(&mut entry, &mut output, 1, 2);
        assert_eq!(entry.state, PlayState::Stopped);

        let buffer = sample_buffer(1, 1.0, 0.0);
        let mut entry = BufferEntry {
            buffer,
            position: 10.0,
            state: PlayState::Playing,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
        };
        mix_buffer_entry(&mut entry, &mut output, 1, 2);
        assert_eq!(entry.state, PlayState::Stopped);
    }

    #[test]
    fn test_music_builtins_roundtrip() {
        let dir = tempdir().unwrap();
        let music_path = dir.path().join("song.wav");
        fs::write(&music_path, b"wav").unwrap();

        let env = Rc::new(RefCell::new(Environment::new()));
        register_audio_functions(&env);

        let muisic_lade = get_native(&env, "muisic_lade");
        let err = (muisic_lade.func)(vec![Value::Integer(1)]).unwrap_err();
        assert_eq!(err, "muisic_lade needs a string path");

        let handle = as_handle(
            &(muisic_lade.func)(vec![Value::String(music_path.to_string_lossy().to_string())])
                .unwrap(),
            "handle",
        )
        .unwrap() as i64;

        let muisic_spiel = get_native(&env, "muisic_spiel");
        let muisic_haud = get_native(&env, "muisic_haud");
        let muisic_gae_on = get_native(&env, "muisic_gae_on");
        let muisic_stap = get_native(&env, "muisic_stap");
        let muisic_is_spielin = get_native(&env, "muisic_is_spielin");
        let muisic_loup = get_native(&env, "muisic_loup");
        let muisic_hou_lang = get_native(&env, "muisic_hou_lang");
        let muisic_whaur = get_native(&env, "muisic_whaur");
        let muisic_pit_luid = get_native(&env, "muisic_pit_luid");
        let muisic_pit_pan = get_native(&env, "muisic_pit_pan");
        let muisic_pit_tune = get_native(&env, "muisic_pit_tune");
        let muisic_pit_rin_roond = get_native(&env, "muisic_pit_rin_roond");
        let muisic_unlade = get_native(&env, "muisic_unlade");

        (muisic_spiel.func)(vec![Value::Integer(handle)]).unwrap();
        assert_eq!(
            (muisic_is_spielin.func)(vec![Value::Integer(handle)]).unwrap(),
            Value::Bool(true)
        );

        (muisic_haud.func)(vec![Value::Integer(handle)]).unwrap();
        assert_eq!(
            (muisic_is_spielin.func)(vec![Value::Integer(handle)]).unwrap(),
            Value::Bool(false)
        );

        (muisic_gae_on.func)(vec![Value::Integer(handle)]).unwrap();
        (muisic_pit_luid.func)(vec![Value::Integer(handle), Value::Float(2.0)]).unwrap();
        (muisic_pit_pan.func)(vec![Value::Integer(handle), Value::Float(0.25)]).unwrap();
        (muisic_pit_tune.func)(vec![Value::Integer(handle), Value::Float(1.5)]).unwrap();
        (muisic_pit_rin_roond.func)(vec![Value::Integer(handle), Value::Bool(true)]).unwrap();

        (muisic_loup.func)(vec![Value::Integer(handle), Value::Float(0.05)]).unwrap();
        assert!(matches!(
            (muisic_hou_lang.func)(vec![Value::Integer(handle)]).unwrap(),
            Value::Float(_)
        ));
        assert!(matches!(
            (muisic_whaur.func)(vec![Value::Integer(handle)]).unwrap(),
            Value::Float(_)
        ));

        (muisic_stap.func)(vec![Value::Integer(handle)]).unwrap();
        let err = (muisic_unlade.func)(vec![Value::Integer(999)]).unwrap_err();
        assert_eq!(err, ERR_BAD_HANDLE);
        (muisic_unlade.func)(vec![Value::Integer(handle)]).unwrap();
    }

    #[test]
    fn test_midi_builtins_default_soundfont() {
        let dir = tempdir().unwrap();
        let sf_dir = dir.path().join("assets/soundfonts");
        fs::create_dir_all(&sf_dir).unwrap();
        let sf_path = sf_dir.join("MuseScore_General.sf2");
        fs::write(&sf_path, b"sf").unwrap();
        let midi_path = dir.path().join("song.mid");
        fs::write(&midi_path, b"midi").unwrap();

        let env = Rc::new(RefCell::new(Environment::new()));
        register_audio_functions(&env);

        let midi_lade = get_native(&env, "midi_lade");
        let err = (midi_lade.func)(vec![Value::Integer(1), Value::Nil]).unwrap_err();
        assert_eq!(err, "midi_lade needs a midi filepath");

        let (handle1, handle2) = with_cwd(dir.path(), || {
            let handle1 = as_handle(
                &(midi_lade.func)(vec![
                    Value::String(midi_path.to_string_lossy().to_string()),
                    Value::Nil,
                ])
                .unwrap(),
                "handle",
            )
            .unwrap() as i64;
            let handle2 = as_handle(
                &(midi_lade.func)(vec![
                    Value::String(midi_path.to_string_lossy().to_string()),
                    Value::Nil,
                ])
                .unwrap(),
                "handle",
            )
            .unwrap() as i64;
            (handle1, handle2)
        });

        let midi_spiel = get_native(&env, "midi_spiel");
        let midi_haud = get_native(&env, "midi_haud");
        let midi_gae_on = get_native(&env, "midi_gae_on");
        let midi_stap = get_native(&env, "midi_stap");
        let midi_is_spielin = get_native(&env, "midi_is_spielin");
        let midi_loup = get_native(&env, "midi_loup");
        let midi_hou_lang = get_native(&env, "midi_hou_lang");
        let midi_whaur = get_native(&env, "midi_whaur");
        let midi_pit_luid = get_native(&env, "midi_pit_luid");
        let midi_pit_pan = get_native(&env, "midi_pit_pan");
        let midi_pit_rin_roond = get_native(&env, "midi_pit_rin_roond");
        let midi_unlade = get_native(&env, "midi_unlade");

        (midi_spiel.func)(vec![Value::Integer(handle1)]).unwrap();
        assert_eq!(
            (midi_is_spielin.func)(vec![Value::Integer(handle1)]).unwrap(),
            Value::Bool(true)
        );
        (midi_haud.func)(vec![Value::Integer(handle1)]).unwrap();
        assert_eq!(
            (midi_is_spielin.func)(vec![Value::Integer(handle1)]).unwrap(),
            Value::Bool(false)
        );
        (midi_gae_on.func)(vec![Value::Integer(handle1)]).unwrap();
        (midi_pit_luid.func)(vec![Value::Integer(handle1), Value::Float(0.8)]).unwrap();
        (midi_pit_pan.func)(vec![Value::Integer(handle1), Value::Float(-0.5)]).unwrap();
        (midi_pit_rin_roond.func)(vec![Value::Integer(handle1), Value::Bool(true)]).unwrap();
        (midi_loup.func)(vec![Value::Integer(handle1), Value::Float(0.05)]).unwrap();
        assert!(matches!(
            (midi_hou_lang.func)(vec![Value::Integer(handle1)]).unwrap(),
            Value::Float(_)
        ));
        assert!(matches!(
            (midi_whaur.func)(vec![Value::Integer(handle1)]).unwrap(),
            Value::Float(_)
        ));
        (midi_stap.func)(vec![Value::Integer(handle1)]).unwrap();

        let err = (midi_unlade.func)(vec![Value::Integer(999)]).unwrap_err();
        assert_eq!(err, ERR_BAD_HANDLE);
        (midi_unlade.func)(vec![Value::Integer(handle1)]).unwrap();
        (midi_unlade.func)(vec![Value::Integer(handle2)]).unwrap();
    }

    #[test]
    fn test_midi_lade_maps_midifile_new_error() {
        let dir = tempdir().unwrap();
        let midi_path = dir.path().join("song.mid");
        fs::write(&midi_path, b"midi").unwrap();
        let sf_path = dir.path().join("MuseScore_General.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        let env = Rc::new(RefCell::new(Environment::new()));
        register_audio_functions(&env);
        let midi_lade = get_native(&env, "midi_lade");

        rustysynth::fail_next_midi_file_new();
        let err = (midi_lade.func)(vec![
            Value::String(midi_path.to_string_lossy().to_string()),
            Value::String(sf_path.to_string_lossy().to_string()),
        ])
        .unwrap_err();
        assert_eq!(err, "Cannae read the midi");
    }

    #[test]
    fn test_midi_lade_maps_synth_new_error() {
        let dir = tempdir().unwrap();
        let midi_path = dir.path().join("song.mid");
        fs::write(&midi_path, b"midi").unwrap();
        let sf_path = dir.path().join("MuseScore_General.sf2");
        fs::write(&sf_path, b"sf").unwrap();

        let env = Rc::new(RefCell::new(Environment::new()));
        register_audio_functions(&env);
        let midi_lade = get_native(&env, "midi_lade");

        rustysynth::fail_next_synth_new();
        let err = (midi_lade.func)(vec![
            Value::String(midi_path.to_string_lossy().to_string()),
            Value::String(sf_path.to_string_lossy().to_string()),
        ])
        .unwrap_err();
        assert_eq!(err, "Cannae set up the synth");
    }

    #[test]
    fn test_mix_midi_entry_branches() {
        let mut sf_data = std::io::Cursor::new(Vec::new());
        let soundfont = SoundFont::new(&mut sf_data).unwrap();
        let mut midi_data = std::io::Cursor::new(Vec::new());
        let midi = MidiFile::new(&mut midi_data).unwrap();
        let midi = Arc::new(midi);
        let sequencer = MidiFileSequencer::new(
            Synthesizer::new(&soundfont, &SynthesizerSettings::new(1)).unwrap(),
        );
        let mut entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: 1,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };

        let mut output = vec![0.0_f32; 2];
        mix_midi_entry(&mut entry, &mut output, 1, 2);
        assert_eq!(entry.state, PlayState::Stopped);

        entry.state = PlayState::Playing;
        entry.sequencer.play(&entry.midi, false);
        mix_midi_entry(&mut entry, &mut output, 1, 2);
        assert_eq!(entry.state, PlayState::Stopped);

        // Cover the no-resize path and the "looped" case that doesn't stop playback.
        entry.state = PlayState::Playing;
        entry.looped = true;
        entry.sequencer.play(&entry.midi, false);
        let mut output = vec![0.0_f32; 4];
        mix_midi_entry(&mut entry, &mut output, 2, 2);
        assert_eq!(entry.state, PlayState::Playing);
        mix_midi_entry(&mut entry, &mut output, 1, 2);
        assert_eq!(entry.state, PlayState::Playing);
    }

    #[test]
    fn test_seek_midi_clamps_negative_and_overflow() {
        let mut sf_data = std::io::Cursor::new(Vec::new());
        let soundfont = SoundFont::new(&mut sf_data).unwrap();
        let settings = SynthesizerSettings::new(1);
        let synth = Synthesizer::new(&soundfont, &settings).unwrap();
        let sequencer = MidiFileSequencer::new(synth);
        let mut midi_data = std::io::Cursor::new(Vec::new());
        let midi = MidiFile::new(&mut midi_data).unwrap();
        let midi = Arc::new(midi);
        let mut entry = MidiEntry {
            midi,
            sequencer,
            state: PlayState::Stopped,
            looped: false,
            volume: 1.0,
            pan: 0.0,
            sample_rate: 1,
            scratch_left: Vec::new(),
            scratch_right: Vec::new(),
        };
        seek_midi(&mut entry, -1.0).unwrap();
        seek_midi(&mut entry, 10.0).unwrap();
    }

    #[test]
    fn test_soond_unlade_invalid_handle_returns_error() {
        let env = Rc::new(RefCell::new(crate::value::Environment::new()));
        register_audio_functions(&env);
        let native = get_native(&env, "soond_unlade");
        let err = (native.func)(vec![Value::Integer(999)]).unwrap_err();
        assert_eq!(err, ERR_BAD_HANDLE);
    }

    #[test]
    fn test_audio_natives_return_bad_handle_for_invalid_handles() {
        let env = Rc::new(RefCell::new(crate::value::Environment::new()));
        register_audio_functions(&env);

        let cases: &[(&str, Vec<Value>)] = &[
            ("soond_ready", vec![Value::Integer(999)]),
            ("soond_spiel", vec![Value::Integer(999)]),
            ("soond_haud", vec![Value::Integer(999)]),
            ("soond_gae_on", vec![Value::Integer(999)]),
            ("soond_stap", vec![Value::Integer(999)]),
            ("soond_is_spielin", vec![Value::Integer(999)]),
            ("soond_pit_luid", vec![Value::Integer(999), Value::Float(0.5)]),
            ("soond_pit_pan", vec![Value::Integer(999), Value::Float(0.0)]),
            ("soond_pit_tune", vec![Value::Integer(999), Value::Float(1.0)]),
            (
                "soond_pit_rin_roond",
                vec![Value::Integer(999), Value::Bool(true)],
            ),
            ("muisic_spiel", vec![Value::Integer(999)]),
            ("muisic_haud", vec![Value::Integer(999)]),
            ("muisic_gae_on", vec![Value::Integer(999)]),
            ("muisic_stap", vec![Value::Integer(999)]),
            ("muisic_is_spielin", vec![Value::Integer(999)]),
            ("muisic_loup", vec![Value::Integer(999), Value::Float(0.0)]),
            ("muisic_hou_lang", vec![Value::Integer(999)]),
            ("muisic_whaur", vec![Value::Integer(999)]),
            ("muisic_pit_luid", vec![Value::Integer(999), Value::Float(0.5)]),
            ("muisic_pit_pan", vec![Value::Integer(999), Value::Float(0.0)]),
            ("muisic_pit_tune", vec![Value::Integer(999), Value::Float(1.0)]),
            (
                "muisic_pit_rin_roond",
                vec![Value::Integer(999), Value::Bool(false)],
            ),
            ("midi_haud", vec![Value::Integer(999)]),
            ("midi_gae_on", vec![Value::Integer(999)]),
            ("midi_stap", vec![Value::Integer(999)]),
            ("midi_is_spielin", vec![Value::Integer(999)]),
            ("midi_loup", vec![Value::Integer(999), Value::Float(0.0)]),
            ("midi_hou_lang", vec![Value::Integer(999)]),
            ("midi_whaur", vec![Value::Integer(999)]),
            ("midi_pit_luid", vec![Value::Integer(999), Value::Float(0.5)]),
            ("midi_pit_pan", vec![Value::Integer(999), Value::Float(0.0)]),
            (
                "midi_pit_rin_roond",
                vec![Value::Integer(999), Value::Bool(true)],
            ),
        ];

        for (name, args) in cases {
            let native = get_native(&env, name);
            let err = (native.func)(args.clone()).unwrap_err();
            assert_eq!(err, ERR_BAD_HANDLE, "native {name} returned unexpected error");
        }
    }
}
