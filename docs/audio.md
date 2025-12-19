# Audio (Soond) API

Audio is enabled by default and independent of graphics. To enable audio
explicitly (for example, without graphics), build with:

```bash
cargo build --release --no-default-features --features cli,llvm,audio
```

Note: audio uses raylib. On Ubuntu/WSL you’ll need:
```bash
sudo apt install cmake libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
```
If you don’t have those, build without the `audio` feature.

If you see:
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
- **Updates**: streaming sources require `soond_haud_gang()` to be called regularly.

## Device / Global
```scots
soond_stairt()         # init audio device
soond_steek()          # close device + unload everything
soond_wheesht(aye)     # mute
soond_wheesht(nae)     # unmute
soond_luid(0.8)        # master volume 0..1
soond_hou_luid()       # get master volume
soond_haud_gang()      # tick streaming audio
```

## Sounds (SFX)
```scots
ken ding = soond_lade("assets/sfx/ding.wav")
soond_pit_luid(ding, 0.6)
soond_pit_pan(ding, -0.2)   # -1 left, 0 center, 1 right
soond_spiel(ding)
```

Available:
- `soond_lade`, `soond_spiel`, `soond_haud`, `soond_gae_on`, `soond_stap`, `soond_unlade`
- `soond_is_spielin`
- `soond_pit_luid`, `soond_pit_pan`, `soond_pit_tune`, `soond_pit_rin_roond`

## Music (MP3 / streaming)
```scots
ken tune = muisic_lade("music/theme.mp3")
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
ken song = midi_lade("music/air.mid", naething)
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
- Pan uses -1..1 in the API and is mapped to raylib’s 0..1 range.
- Streaming audio requires periodic updates; call `soond_haud_gang()` in your main loop.
- MIDI loop flag affects playback start; set it before `midi_spiel` for best results.
