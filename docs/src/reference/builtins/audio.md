# Audio built-ins (soond, muisic, midi)

Three families of built-ins play audio:

- `soond_*` - the device, the master volume, and short sound effects.
- `muisic_*` - longer music tracks (MP3 or long WAV).
- `midi_*` - MIDI files played through a SoundFont.

All three take and return plain integer handles. A handle from `soond_lade(path)` only works with the other `soond_*` functions, one from `muisic_lade(path)` only with `muisic_*`, and one from `midi_lade(path, soundfont)` only with `midi_*`.

Sound and music need a build with the `audio` or `graphics` feature. See [Which backend runs](#which-backend-runs) before you rely on any of this: the backend picked by the default build does not play sound yet.

## Quick look

These examples need an audio device and the asset files, so they are shown as plain text.

```text
soond_stairt()
ken ding = soond_lade("assets/audio/ding.wav")
soond_pit_luid(ding, 0.6)
soond_pit_pan(ding, -0.2)
soond_spiel(ding)

ken tune = muisic_lade("assets/audio/theme.mp3")
muisic_pit_rin_roond(tune, aye)
muisic_spiel(tune)

ken song = midi_lade("assets/audio/wee_tune.mid", naething)
midi_spiel(song)

whiles aye {
    soond_haud_gang()
}
```

## Argument rules

These checks come from the miniaudio backend and the MIDI module. The error texts are quoted from the code.

| Argument | Rule | Error when wrong |
|---|---|---|
| handle | a non-negative integer | `<name> needs a guid handle` |
| handle that is not loaded (never loaded, or already unloaded) | | `Thon handle isnae guid` |
| number (volume, pan, pitch, seconds) | integer or float | `<name> needs a nummer` |
| flag (mute, loop) | `aye` or `nae` | `<name> needs aye or nae` |
| `soond_lade` path | a string | `soond_lade needs a string path` |
| `muisic_lade` path | a string | `muisic_lade needs a string path` |
| `midi_lade` path | a string | `midi_lade needs a midi filepath` |
| `midi_lade` soundfont | a string or `naething` | `midi_lade needs a soondfont path or naething` |

`<name>` is the name of the function you called, for example `soond_pit_luid needs a nummer`.

## Device and master controls

| Function | Returns | What it does |
|---|---|---|
| `soond_stairt()` | `naething` | Opens and starts the audio device. Fails with `Cannae stairt the soond device` if the device will not open. The functions that load or change global state also start the device on demand, so calling it first is optional. |
| `soond_steek()` | `naething` | Stops the device and unloads every `soond_*` and `muisic_*` handle. It also sets the master volume back to 1.0 and unmutes. |
| `soond_wheesht(flag)` | `naething` | `soond_wheesht(aye)` mutes all output, `soond_wheesht(nae)` unmutes. Starts the device if needed. |
| `soond_luid(volume)` | `naething` | Sets the master volume. The value is clamped to 0.0 to 1.0. Starts the device if needed. |
| `soond_hou_luid()` | float | The current master volume. Does not start the device. |
| `soond_haud_gang()` | `naething` | Keeps streaming audio going on backends that need a regular tick (JavaScript and WASM). On the miniaudio backend the device callback does the work and this call does nothing, so it is always safe to call in a main loop. |
| `soond_ready(handle)` | bool | Whether a sound has finished loading. On miniaudio it returns `aye` for any loaded handle, because loading is complete when `soond_lade(path)` returns. |

## Sound effects (`soond_*`)

| Function | Returns | What it does |
|---|---|---|
| `soond_lade(path)` | integer handle | Decodes the file in full and returns a new handle. Fails with `Cannae lade the soond` if the file cannot be opened or decoded. |
| `soond_spiel(handle)` | `naething` | Starts playing, or restarts a paused sound from where it stopped. It does not rewind: a sound that played to its end without looping stays at its end, so call `soond_stap(handle)` before playing it again. |
| `soond_haud(handle)` | `naething` | Pauses. |
| `soond_gae_on(handle)` | `naething` | Resumes. On miniaudio it sets the state to playing. |
| `soond_stap(handle)` | `naething` | Stops and rewinds to the start. |
| `soond_unlade(handle)` | `naething` | Frees the handle. Using it again fails with `Thon handle isnae guid`. Freed slots are reused by the next load. |
| `soond_is_spielin(handle)` | bool | `aye` while the sound is playing. A sound that reaches its end without looping goes back to stopped. |
| `soond_pit_luid(handle, volume)` | `naething` | Per-sound volume, clamped to 0.0 to 1.0. Default 1.0. |
| `soond_pit_pan(handle, pan)` | `naething` | -1.0 is full left, 0.0 centre, 1.0 full right. Values outside that range are treated as the nearest end when mixing. Default 0.0. |
| `soond_pit_tune(handle, pitch)` | `naething` | Playback speed, 1.0 normal. A value of 0 or below plays at 1.0. Changing pitch changes speed as well. |
| `soond_pit_rin_roond(handle, flag)` | `naething` | `aye` makes the sound loop. Default `nae`. |

## Music (`muisic_*`)

`muisic_*` mirrors `soond_*`. On the miniaudio backend a music track is decoded fully into memory just like a sound, so the two families behave alike apart from the extra seek and position calls.

| Function | Returns | What it does |
|---|---|---|
| `muisic_lade(path)` | integer handle | Loads the file. Fails with `Cannae lade the muisic` if it cannot be opened or decoded. |
| `muisic_spiel(handle)` | `naething` | Plays. |
| `muisic_haud(handle)` | `naething` | Pauses. |
| `muisic_gae_on(handle)` | `naething` | Resumes. |
| `muisic_stap(handle)` | `naething` | Stops and rewinds. |
| `muisic_unlade(handle)` | `naething` | Frees the handle. |
| `muisic_is_spielin(handle)` | bool | `aye` while playing. |
| `muisic_loup(handle, seconds)` | `naething` | Seeks. Negative values go to 0, values past the end go to the end. |
| `muisic_hou_lang(handle)` | float | Length in seconds. |
| `muisic_whaur(handle)` | float | Current position in seconds. |
| `muisic_pit_luid(handle, volume)` | `naething` | Volume, clamped to 0.0 to 1.0. |
| `muisic_pit_pan(handle, pan)` | `naething` | Pan from -1.0 to 1.0. |
| `muisic_pit_tune(handle, pitch)` | `naething` | Playback speed. |
| `muisic_pit_rin_roond(handle, flag)` | `naething` | Loop on or off. |

## MIDI (`midi_*`)

MIDI files are turned into sound with a SoundFont (an `.sf2` file). The default SoundFont is `assets/soundfonts/MuseScore_General.sf2`. See `assets/soundfonts/README.md` for how to fetch it.

`midi_lade(path, soundfont)` takes two arguments. Pass `naething` as the second to use the default SoundFont, or pass the path of your own `.sf2`. The default is searched for in the current directory, next to the executable, and one and two directories above the executable (under `assets/soundfonts/`). It is loaded once and reused.

| Function | Returns | What it does |
|---|---|---|
| `midi_lade(path, soundfont)` | integer handle | Loads the MIDI file and builds a synthesizer. Errors: `Cannae find the default soondfont`, `Cannae open the soondfont file`, `Cannae read the soondfont`, `Cannae open the midi file`, `Cannae read the midi`, `Cannae set up the synth`. |
| `midi_spiel(handle)` | `naething` | Starts from the beginning if stopped, then plays. |
| `midi_haud(handle)` | `naething` | Pauses (only has an effect while playing). |
| `midi_gae_on(handle)` | `naething` | Resumes (only has an effect while paused). |
| `midi_stap(handle)` | `naething` | Stops. |
| `midi_unlade(handle)` | `naething` | Frees the handle. |
| `midi_is_spielin(handle)` | bool | `aye` while playing. |
| `midi_loup(handle, seconds)` | `naething` | Seeks, clamped to 0 and the file length. Seeking renders the music up to that point, so a big seek takes time. |
| `midi_hou_lang(handle)` | float | Length in seconds. |
| `midi_whaur(handle)` | float | Current position in seconds. |
| `midi_pit_luid(handle, volume)` | `naething` | Volume, clamped to 0.0 to 1.0. |
| `midi_pit_pan(handle, pan)` | `naething` | Pan, clamped to -1.0 to 1.0. |
| `midi_pit_rin_roond(handle, flag)` | `naething` | Loop on or off. Set it before `midi_spiel(handle)`: the loop flag is given to the sequencer when playback starts. |

There is no `midi_pit_tune`.

## Which backend runs

The interpreter picks one audio backend when it is built.

| Build features | Backend | Behaviour |
|---|---|---|
| `audio` without `graphics` | miniaudio | Real playback, as described above. |
| `graphics` (the default `full` build includes it) | raylib | Placeholder. See below. |
| neither | stub | Every audio function fails. |

The raylib backend, in the code as it stands, does not play anything for `soond_*` and `muisic_*`:

- `soond_lade(path)` and `muisic_lade(path)` check that the path is a string and the file exists (otherwise `Cannae lade the soond` or `Cannae lade the muisic`) and return the handle `-1`.
- `soond_spiel(handle)` and `muisic_spiel(handle)` accept only that `-1` handle. A non-negative handle fails with `Thon handle isnae guid`, anything that is not an integer with `<name> needs a guid handle`.
- `soond_wheesht(flag)` checks for `aye` or `nae`. The other `soond_*` and `muisic_*` calls take their arguments without checking them and do nothing.
- `soond_hou_luid()` returns 1.0, `soond_ready(handle)` returns `nae`, the `*_is_spielin` calls return `nae`, and `muisic_hou_lang(handle)` and `muisic_whaur(handle)` return 0.0.

The `midi_*` functions are the same in both audio backends (`src/audio/midi.rs`). In the interpreter they keep playback state, length, volume, pan and loop flag for each handle, but nothing renders the synthesizer into the device output: the only render call is the one `midi_loup(handle, seconds)` uses to seek. So in the interpreter `midi_*` playback is silent, and `midi_whaur(handle)` moves only when you seek. The native (LLVM) runtime does mix MIDI; see below.

With neither feature built, every function in this page fails with:

```text
Soond isnae available - build wi' --features audio or --features graphics
```

The interpreter binary you run decides this. A binary built without audio, for example, fails on the first `soond_*` call.

## Other backends

| Backend | Support |
|---|---|
| JavaScript | Implemented with WebAudio in `runtime/js/audio_runtime.js`, which `src/compiler.rs` embeds in the output. Untested for these docs. Argument errors use the same wording, for example `soond_luid needs a nummer`. Without WebAudio, calls fail with `Soond isnae available in this environment`. |
| WAT/WASM | The compiler emits `env` imports for the audio names (`src/wasm_compiler.rs`); the host must supply them. In a browser, load `runtime/js/audio_runtime.js` and `runtime/js/wasm_audio_host.js`. Untested for these docs. |
| LLVM/native | Calls go to `__mdh_soond_*`, `__mdh_muisic_*` and `__mdh_midi_*` in `runtime/mdh_runtime_rs/src/audio.rs`, a miniaudio runtime that also mixes MIDI. It is linked in only when the compiler itself was built with the `audio` feature (`build.rs`); otherwise every call fails with `Soond isnae available - build wi' --features audio` (`runtime/mdh_runtime_rs/src/audio_stub.rs`). Untested for these docs. (`runtime/mdh_sound.c` is the separate `sound_*` PCM bridge.) |

For JavaScript and WASM, MIDI and sound need two assets served next to your output:

- `assets/wasm/mdh_rustysynth.wasm`
- `assets/soundfonts/MuseScore_General.sf2`

Override the locations before running audio code:

```text
globalThis.__havers_audio_base = "/static/";
globalThis.__havers_soundfont = "/static/sf2/custom.sf2";
globalThis.__havers_midi_wasm = "/static/wasm/mdh_rustysynth.wasm";
```

On the web, `soond_lade(path)` starts loading in the background. Check `soond_ready(handle)` before relying on it, and call `soond_haud_gang()` regularly in your main loop. MIDI in JavaScript fails with `MIDI needs WebAssembly` where WebAssembly is missing. `assets/wasm/README.md` explains how to rebuild the WASM helper.

For WASM in a browser, wire the imports like this:

```text
import "../runtime/js/audio_runtime.js";
import "../runtime/js/wasm_audio_host.js";

const imports = {
  env: {
    memory,
    ...mdh_wasm_audio_imports(memory),
  },
};
```

## Building with audio

```text
cargo build --release --no-default-features --features cli,llvm,audio
```

Audio on the miniaudio backend does not need X11. Only graphics (raylib) does. If a raylib build reports `Unable to find libclang`, install clang and libclang and set `LIBCLANG_PATH` (on Ubuntu: `sudo apt install clang libclang-dev llvm-dev`, then `export LIBCLANG_PATH=$(llvm-config --libdir)`).

## Related

For the screen-bound `sound_open`, `sound_queue`, `sound_queued` and `sound_close` calls (raw PCM), see [screen and sound](./../screen.md).
