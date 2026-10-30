# Sound (PCM output)

The `sound_*` built-ins play raw sound by **queueing PCM**: you generate
samples yourself and push them into a ring buffer that the audio device drains.
There is no file loading, mixing or timing here; that is your program's job.
(Bogle Castle's mixer, `games/bogle/audio/mixer.braw`, is plain mdhavers on
top of `sound_queue`.)

These four functions are separate from the older `soond_*`, `muisic_*` and
`midi_*` file-playback functions described in [Audio and MIDI](./builtins/audio.md).

## A minimal complete program

This opens a mono 22050 Hz stream, queues a quarter of a second of silence
(5512 samples of 16-bit PCM is 11024 bytes), reports how much is waiting and
closes. The interpreter always uses a silent null device for these calls, so
it runs anywhere.

```mdhavers
sound_open(22050, 1)
ken pcm = bytes(11024)
sound_queue(pcm)
blether sound_queued()
sound_close()
```

## Format

| Property | Value |
|---|---|
| Sample rate | `22050` or `44100` Hz, nothing else. |
| Channels | `1` (mono) or `2` (stereo). |
| Sample | Signed 16-bit, little-endian. |
| Frame | One sample per channel, `channels * 2` bytes. Stereo is interleaved left, right, left, right. |
| Ring capacity | 1,048,576 bytes (1 MiB) of queued, unplayed PCM. |

Write each sample with `bytes_set` from the [bytes built-ins](./builtins.md),
low byte first: for a sample `s` at frame offset `o`, set byte `o` to the low
8 bits and byte `o + 1` to the high 8 bits (two's complement for negative
values). The `bytes_write_u16be` and `bytes_write_u32be` helpers are
big-endian, so they do not fit this format. Then pass the buffer to
`sound_queue`.

## Backend support

| Backend | Behaviour |
|---|---|
| Interpreter (every build) | A deterministic **null** device implemented in `src/graphics.rs` (`register_sound_functions`). Nothing is ever played. Queued PCM stays queued: `sound_queued()` never counts down. |
| Native on macOS | A real output through the system audio queue (`runtime/mdh_sound.c`, AudioToolbox, three buffers of 5 ms each). `sound_queued()` falls as the device consumes data. |
| Native on other platforms | The same code, but the device is always the null device (`mdh_sound.c`: `null_device = true` outside macOS). Nothing is played. |
| JavaScript and WebAssembly | Not implemented for `sound_*` (no helper in `src/compiler.rs` or `src/wasm_compiler.rs`; inference from the absence). The WebAudio support described in [Audio and MIDI](./builtins/audio.md) is for the `soond_*` family. |

Native calls must come from the main thread: `Nae sound aff the main thread!`.

### Silencing the device: `MDH_SOUND=none`

For a native program, setting `MDH_SOUND=none` (or `MDH_SCREEN=none`, which
also disables sound) before `sound_open` makes it use the null device. Calls,
checks and error messages are the same; the audio ring only fills, because
nothing drains it. Use this for tests and servers with no audio hardware.

A missing or busy audio device makes `sound_open` hurl, so a game should
catch it and carry on silently, as Bogle does:

```mdhavers
ken opened = aye
hae_a_bash {
    sound_open(22050, 1)
} gin_it_gangs_wrang err {
    blether "nae sound the day, playin on quiet"
    opened = nae
}
gin opened {
    sound_close()
}
```

## Functions

### `sound_open(rate, channels)`

Opens the one sound stream.

| Argument | Type | Meaning |
|---|---|---|
| `rate` | integer | `22050` or `44100`. |
| `channels` | integer | `1` or `2`. |

Returns `naething`. Clears the ring.

| Error | When |
|---|---|
| `sound_open wants rate 22050 or 44100 and channels 1 or 2` | Any other value, or a non-integer. |
| `sound_open: sound is already open` | Called again before `sound_close`. |
| `sound_open: couldnae open the audio device` | Native macOS only: the system refused the device. |

### `sound_queue(pcm)`

Copies `pcm` into the ring. The bytes are copied before the call returns, so
you may reuse the buffer straight away.

| Argument | Type | Meaning |
|---|---|---|
| `pcm` | bytes | Whole frames of signed 16-bit little-endian PCM. Length must be a multiple of `channels * 2`. An empty buffer is accepted. |

Returns `naething`.

| Error | When |
|---|---|
| `sound_queue: nae sound is open` | `sound_open` has not been called (or `sound_close` has). |
| `sound_queue wants bytes PCM` | `pcm` is not `bytes`. |
| `sound_queue wants complete signed16 PCM frames` | The length is not a whole number of frames. |
| `sound_queue: PCM ring is full` | The queued data would go past 1 MiB. Nothing is queued. |

### `sound_queued()`

Returns how much PCM is still waiting, as an **integer count of 16-bit
samples across all channels** (bytes queued divided by two). In stereo, one
frame is two samples, so divide by `channels` to get frames. On a real device
the count falls as the device plays; on a null device it does not.
Takes no arguments. Does not need an open stream: with none open it returns
`0`. In native code it also counts data the device has taken but not yet
finished playing.

### `sound_close()`

Stops the stream and discards anything still queued. Safe to call when
nothing is open. Returns `naething`.

## Keeping the device fed

Because the ring only holds PCM, the usual pattern is: each frame, top the
queue up to a target depth, then let the device drain it. Bogle's
`games/bogle/shell/frame.braw` keeps 60 ms ahead (1323 samples at 22050 Hz
mono):

```mdhavers
dae top_up(target) {
    ken needed = max(0, target - sound_queued())
    gin needed > 0 {
        # needed is in samples; mono 16-bit means 2 bytes each
        sound_queue(bytes(needed * 2))
    }
}

sound_open(22050, 1)
top_up(1323)
blether sound_queued()
sound_close()
```

In the interpreter the null device never drains, so a loop that tops up each
frame stops queueing after the first call; that is the intended
deterministic behaviour for tests.
