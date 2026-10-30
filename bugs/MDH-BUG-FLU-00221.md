# MDH-BUG-FLU-00221 — Default full build plays no sound: raylib audio backend is all stubs

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** audio
- **Raised:** 2026-10-30T07:01:28Z
- **Discovery source:** Agent
- **Owner:** -
- **Owner role:** -
- **Owner run:** -
- **Owner host:** -
- **Owner branch:** -
- **Owner base:** -
- **Owner fingerprint:** -
- **Owner since:** -
- **Owner until:** -
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-30T07:01:28Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

**Symptom:** in the default `full` build (graphics + audio), every `soond_*` / `muisic_*` call succeeds but plays nothing. The backend selected when the `graphics` feature is on, src/audio/backend_raylib.rs, is stubs throughout: `load` returns a handle, `play`/`seek` return Ok(()), `position`/`duration` return 0.0, and `set_volume`/`set_pan`/`set_pitch`/`set_looping` do nothing (comments "Stub - will call raylib::ffi::...", lines ~24-230).

**Expected:** CLAUDE.md says "`graphics` feature -> raylib's built-in audio (no symbol conflicts)" and "Both features -> raylib audio takes precedence", so a default build should play sound through raylib. Only an `audio`-without-`graphics` build (miniaudio backend) makes sound today.

**Repro:** build with default features, run a program that does `soond_lade("x.wav")` then `soond_spiel(...)`: no audio, no error. (Function names per docs/src/reference/builtins/audio.md.) Verified by reading the backend; not yet observed on a speaker.

Found by the docs worker (2026-10-30).

Evidence fingerprint: `manual:v1:default-full-build-plays-no-sound-raylib-audio--1b45c72e095bbf48`


## Fix

<unfixed — raised only>

## Notes
