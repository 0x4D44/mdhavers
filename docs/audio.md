# Audio (Soond) API

Audio is enabled by default and independent of graphics. To enable audio
explicitly (for example, without graphics), build with:

```bash
cargo build --release --no-default-features --features cli,llvm,audio
```

Note: graphics uses raylib (X11 deps on Ubuntu/WSL). Audio (interpreter + native) uses
miniaudio and does not require X11. X11 deps are only needed for graphics.

Backend support: interpreter, LLVM/native, JavaScript, and WAT/WASM.

For JavaScript/WASM, audio uses WebAudio and a rustysynth WASM helper. You must
host these assets alongside your compiled output (or set overrides):
- `assets/wasm/mdh_rustysynth.wasm`
- `assets/soundfonts/MuseScore_General.sf2`

See `assets/wasm/README.md` (rebuild the WASM helper) and `assets/soundfonts/README.md` (download the default SoundFont).

Optional overrides (set before running audio code):
```js
globalThis.__havers_audio_base = "/static/";
globalThis.__havers_soundfont = "/static/sf2/custom.sf2";
globalThis.__havers_midi_wasm = "/static/wasm/mdh_rustysynth.wasm";
```

For WAT/WASM in the browser, wire audio imports via the helper runtime:
```js
import "../runtime/js/audio_runtime.js";
import "../runtime/js/wasm_audio_host.js";

const imports = {
  env: {
    memory,
    // print_i32/print_f64/print_str, etc.
    ...mdh_wasm_audio_imports(memory),
  },
};
```

If you see (raylib builds):
```
Unable to find libclang
```
install clang + libclang and retry:
```bash
sudo apt install clang libclang-dev llvm-dev
export LIBCLANG_PATH=$(llvm-config --libdir)
```

## Core Concepts
- **Sounds (soond)**: short SFX loaded into memory.
- **Music (muisic)**: streaming audio (MP3 / long WAV).
- **MIDI**: synthesized via bundled SoundFont.
- **Updates**: some backends (JS/WASM) require `soond_haud_gang()` to be called regularly.
  Native audio is driven by the device callback, but it is always safe to call.

## Device / Global
```scots
soond_stairt()         # init audio device
soond_steek()          # close device + unload everything
soond_wheesht(aye)     # mute
soond_wheesht(nae)     # unmute
soond_luid(0.8)        # master volume 0..1
soond_hou_luid()       # get master volume
soond_haud_gang()      # tick streaming audio on backends that need it
soond_ready(handle)    # check SFX load status (web backends)
```

## Sounds (SFX)
```scots
ken ding = soond_lade("assets/audio/ding.wav")
soond_pit_luid(ding, 0.6)
soond_pit_pan(ding, -0.2)   # -1 left, 0 center, 1 right
soond_spiel(ding)
```

Available:
- `soond_lade`, `soond_spiel`, `soond_haud`, `soond_gae_on`, `soond_stap`, `soond_unlade`
- `soond_is_spielin`
- `soond_pit_luid`, `soond_pit_pan`, `soond_pit_tune`, `soond_pit_rin_roond`
- `soond_ready`

## Music (MP3 / streaming)
```scots
ken tune = muisic_lade("assets/audio/theme.mp3")
muisic_spiel(tune)

whiles aye {
    soond_haud_gang()
}
```

Available:
- `muisic_lade`, `muisic_spiel`, `muisic_haud`, `muisic_gae_on`, `muisic_stap`, `muisic_unlade`
- `muisic_is_spielin`
- `muisic_loup`, `muisic_hou_lang`, `muisic_whaur`
- `muisic_pit_luid`, `muisic_pit_pan`, `muisic_pit_tune`, `muisic_pit_rin_roond`

## MIDI
The default SoundFont is bundled at:
`assets/soundfonts/MuseScore_General.sf2`

```scots
ken song = midi_lade("assets/audio/wee_tune.mid", naething)
midi_spiel(song)

whiles aye {
    soond_haud_gang()
}
```

Available:
- `midi_lade`, `midi_spiel`, `midi_haud`, `midi_gae_on`, `midi_stap`, `midi_unlade`
- `midi_is_spielin`
- `midi_loup`, `midi_hou_lang`, `midi_whaur`
- `midi_pit_luid`, `midi_pit_pan`, `midi_pit_rin_roond`

## Notes
- Pan uses -1..1 in the API; backends map this to their mixer panning.
- Streaming audio requires periodic updates; call `soond_haud_gang()` in your main loop.
- MIDI loop flag affects playback start; set it before `midi_spiel` for best results.
