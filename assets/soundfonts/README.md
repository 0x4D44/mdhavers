# SoundFonts

This folder contains bundled SoundFonts used by mdhavers MIDI playback.

## MuseScore General
- File: `MuseScore_General.sf2`
- Source: MuseScore General SoundFont (downloaded from OSUOSL mirror)
- License: MIT (see `MuseScore_General_License.md`)
- Notes: Use this as the default SoundFont when `midi_lade` is passed `naething`.

## Download

If `MuseScore_General.sf2` is missing, you can fetch it from the OSUOSL mirror:

```bash
curl -L -o assets/soundfonts/MuseScore_General.sf2 \
  https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General.sf2
```

Notes:
- There is also a compressed `.sf3` variant on the same mirror, but mdhavers currently defaults to the `.sf2` path.
- Please keep `MuseScore_General_License.md` and `MuseScore_General_Readme.md` alongside the SoundFont data.

## Files
- `MuseScore_General.sf2` — the SoundFont data
- `MuseScore_General_License.md` — license and attribution from upstream
- `MuseScore_General_Readme.md` — upstream README
