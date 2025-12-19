use std::io::Cursor;
use std::sync::Arc;

use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

const CHUNK_FRAMES: usize = 1024;

static mut LAST_LEN: usize = 0;
static mut LAST_FRAMES: usize = 0;
static mut LAST_ERR_PTR: *mut u8 = std::ptr::null_mut();
static mut LAST_ERR_LEN: usize = 0;

fn clear_error() {
    unsafe {
        if !LAST_ERR_PTR.is_null() && LAST_ERR_LEN > 0 {
            let _ = Vec::from_raw_parts(LAST_ERR_PTR, LAST_ERR_LEN, LAST_ERR_LEN);
        }
        LAST_ERR_PTR = std::ptr::null_mut();
        LAST_ERR_LEN = 0;
    }
}

fn set_error(msg: &str) {
    clear_error();
    let mut bytes = msg.as_bytes().to_vec();
    let len = bytes.len();
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    unsafe {
        LAST_ERR_PTR = ptr;
        LAST_ERR_LEN = len;
    }
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(ptr, size, size);
    }
}

#[no_mangle]
pub extern "C" fn render_midi(
    sf_ptr: *const u8,
    sf_len: usize,
    midi_ptr: *const u8,
    midi_len: usize,
    sample_rate: u32,
) -> *mut f32 {
    clear_error();
    if sf_ptr.is_null() || sf_len == 0 {
        set_error("Cannae read the soondfont");
        return std::ptr::null_mut();
    }
    if midi_ptr.is_null() || midi_len == 0 {
        set_error("Cannae read the midi");
        return std::ptr::null_mut();
    }

    let sf_bytes = unsafe { std::slice::from_raw_parts(sf_ptr, sf_len) };
    let midi_bytes = unsafe { std::slice::from_raw_parts(midi_ptr, midi_len) };

    let mut sf_cursor = Cursor::new(sf_bytes);
    let sf = match SoundFont::new(&mut sf_cursor) {
        Ok(sf) => Arc::new(sf),
        Err(_) => {
            set_error("Cannae read the soondfont");
            return std::ptr::null_mut();
        }
    };

    let mut midi_cursor = Cursor::new(midi_bytes);
    let midi = match MidiFile::new(&mut midi_cursor) {
        Ok(m) => Arc::new(m),
        Err(_) => {
            set_error("Cannae read the midi");
            return std::ptr::null_mut();
        }
    };

    let sr = sample_rate.max(8000).min(192000) as i32;
    let settings = SynthesizerSettings::new(sr);
    let synth = match Synthesizer::new(&sf, &settings) {
        Ok(s) => s,
        Err(_) => {
            set_error("Cannae set up the synth");
            return std::ptr::null_mut();
        }
    };

    let mut sequencer = MidiFileSequencer::new(synth);
    sequencer.play(&midi, false);

    let length = midi.get_length();
    let total_frames = if length <= 0.0 {
        0
    } else {
        (length * sr as f64).ceil() as usize
    };

    unsafe {
        LAST_FRAMES = total_frames;
    }

    if total_frames == 0 {
        unsafe {
            LAST_LEN = 0;
        }
        return std::ptr::null_mut();
    }

    let mut output: Vec<f32> = Vec::with_capacity(total_frames * 2);
    let mut left = vec![0.0_f32; CHUNK_FRAMES];
    let mut right = vec![0.0_f32; CHUNK_FRAMES];

    let mut remaining = total_frames;
    while remaining > 0 {
        let chunk = remaining.min(CHUNK_FRAMES);
        sequencer.render(&mut left[..chunk], &mut right[..chunk]);
        for i in 0..chunk {
            output.push(left[i]);
            output.push(right[i]);
        }
        remaining -= chunk;
    }

    unsafe {
        LAST_LEN = output.len();
    }

    let ptr = output.as_mut_ptr();
    std::mem::forget(output);
    ptr
}

#[no_mangle]
pub extern "C" fn render_midi_len() -> usize {
    unsafe { LAST_LEN }
}

#[no_mangle]
pub extern "C" fn render_midi_frames() -> usize {
    unsafe { LAST_FRAMES }
}

#[no_mangle]
pub extern "C" fn render_midi_free(ptr: *mut f32, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}

#[no_mangle]
pub extern "C" fn last_error_ptr() -> *const u8 {
    unsafe { LAST_ERR_PTR }
}

#[no_mangle]
pub extern "C" fn last_error_len() -> usize {
    unsafe { LAST_ERR_LEN }
}
