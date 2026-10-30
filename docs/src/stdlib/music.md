# music

Music notation as plain data, with no sound output. Build notes, rests and melodies as dictionaries, print them as ASCII staffs and rhythm bars, convert notes to frequencies and MIDI numbers, transpose, and use a few bundled Scottish tunes, rhythm patterns, bagpipe notation, a visual metronome and a song builder.

Import: `fetch "stdlib/music"`

Pure mdhavers; it does not call any audio built-in, so it runs on any backend. Loading the module prints `Music module loaded! Scotland the Brave! ...`. It also defines the global helpers `replace_all(s, old, new)` and `has_key(dict, key)`. For real playback use the audio built-ins, not this module.

Data shapes: a note is `{"pitch", "duration", "octave", "type": "note"}`; a rest is `{"pitch": "R", "duration", "type": "rest"}`; a melody is `{"notes", "tempo" (120), "time_sig" ([4, 4]), "title" ("Untitled")}`; a song is `{"title", "verses", "chorus", "melody"}`. Durations are in beats. No function in this module hurls its own errors.

## Constants

- Durations: `WHOLE` 4.0, `HALF` 2.0, `QUARTER` 1.0, `EIGHTH` 0.5, `SIXTEENTH` 0.25, `DOTTED_HALF` 3.0, `DOTTED_QUARTER` 1.5, `DOTTED_EIGHTH` 0.75.
- `NOTES`: the twelve chromatic names `C` to `B` with sharps.
- `SCOTS_SCALE`: `["D", "E", "G", "A", "B"]`.
- `BAGPIPE_SCALE`: the nine Highland pipe notes.
- `GRACE_NOTES`: dictionary of bagpipe embellishments (`grip`, `doubling`, `throw`, `birl`, `taorluath`, `crunluath`) to their note patterns.

## Functions

### has_key(dict, key)
`aye` if `key` is one of the dictionary's keys.

### note(pitch, duration = QUARTER, octave = 4)
Returns a note dictionary. `pitch` is a name such as `"C"` or `"F#"`.

### rest(duration = QUARTER)
Returns a rest dictionary.

### note_tae_string(n)
Formats a note as `A4(2)` and a rest as `R(1)`.

### melody(notes = [])
Returns a new melody dictionary.

### add_note(mel, n)
Appends a note or rest to the melody (mutating it) and returns the melody.

### set_title(mel, title)
Sets the title and returns the melody.

### set_tempo(mel, bpm)
Sets the tempo in beats per minute and returns the melody.

### melody_duration(mel)
Total length in beats, as a float.

### note_tae_staff(n)
A five-character staff symbol for a note (`"  -  "` for a rest, `" [?] "` for an unknown pitch). Only the letters `A` to `G` are known.

### draw_staff(mel)
Prints the title, the tempo, a blank line, one `│ A4(1) │` line per note, and a closing `╰─────────────────╯` line. Returns `naething`.

### visualize_rhythm(mel)
Prints block bars for the note lengths (longer notes get longer bars, rests use a light shade), starting a new line every 4 beats. Returns `naething`.

### auld_lang_syne()
Returns a 15-note simplified melody at tempo 100.

### scotland_the_brave()
Returns an 8-note opening phrase at tempo 120.

### amazing_grace()
Returns a 9-note melody at tempo 80.

### flower_of_scotland()
Returns an 8-note melody at tempo 88.

### strathspey_rhythm()
Returns a list of six durations (dotted eighth, sixteenth, quarter, repeated).

### reel_rhythm()
Returns a list of eight eighth-note durations.

### jig_rhythm()
Returns a list of six eighth-note durations.

### march_rhythm()
Returns a list of five durations: two quarters, two eighths, a quarter.

### bagpipe_tune(title)
Returns a bagpipe tune dictionary with `title`, empty `notes`, `tune_type` `"march"` and `time_sig` `[4, 4]`.

### pipe_note(tune, pitch, grace = naething)
Appends a `BagpipeNote` to the tune, with an optional grace-note name from `GRACE_NOTES`, and returns the tune. An unknown grace name is silently ignored.

### display_bagpipe(tune)
Prints a boxed header and the notes, eight to a line. Returns `naething`.

### ms_per_beat(bpm)
Milliseconds per beat: `60000.0 / bpm`.

### metronome_tick(beat_num, beats_per_bar = 4)
`"TICK"` on the first beat of a bar (beats counted from 1), otherwise `"tock"`.

### visual_metronome(beats, bpm = 120)
Prints one line per beat with a dot marker. It only prints; it does not wait.

### draw_wave(length = 40, amplitude = 3)
Prints an ASCII sine wave `length` columns wide.

### draw_beat_pattern(pattern, char = "█")
Prints one bar per duration in the list, scaled so the longest is 8 characters, followed by the duration.

### random_scots_tune(length = 8)
Returns a melody of `length` random notes from `SCOTS_SCALE`, with random durations and octave 4 or 5.

### song(title)
Returns a new song dictionary.

### add_verse(s, text)
Appends a verse and returns the song.

### set_chorus(s, text)
Sets the chorus and returns the song.

### display_song(s)
Prints the song; the chorus, if set, is printed after every verse. Returns `naething`.

### wee_scots_song()
Returns a ready-made song with two verses and a chorus.

### note_tae_frequency(n, a4_freq = 440.0)
Frequency in Hz of a note dictionary, from equal temperament. An unknown pitch is treated as `C`.

### midi_tae_note(midi_num)
Returns a quarter-note dictionary for a MIDI number (60 gives C4).

### transpose_melody(mel, semitones)
Returns a new melody, titled `<title> (transposed)`, with every note shifted by `semitones` (rests are kept). The original is not changed.

### replace_all(s, old, new)
Replaces every character of `s` equal to the single character `old` with `new`. It works one character at a time, so `old` must be one character.

## Classes

### BagpipeNote
`BagpipeNote(pitch, is_grace = nae)`. Fields: `pitch`, `is_grace`, `embellishment` (`naething`).

- `add_grace(grace_type)`: sets the embellishment if the name is in `GRACE_NOTES`; returns the note.
- `tae_string()`: the pitch, prefixed with `[grace]` when an embellishment is set.

## Example

```mdhavers
fetch "stdlib/music"

ken m = scotland_the_brave()
blether melody_duration(m)
blether note_tae_string(note("A", HALF, 4))
blether note_tae_frequency(note("A"))
blether note_tae_string(midi_tae_note(60))
visualize_rhythm(m)
blether note_tae_string(transpose_melody(m, 2)["notes"][0])
```
