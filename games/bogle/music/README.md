# Bogle Castle music

Two original compositions, written here as readable, deterministic mdhavers
scores. No tune or phrase was taken from an existing song or the synth's albums.
The `.mid` files are source assets and should be committed with the generators.
The runtime streams the baked `.bmu` files without a MIDI synth or new dependency.

| Asset | Composition | Form and instruments | Loop body | Baked bytes |
| --- | --- | --- | --- | --- |
| `title` | The Laird's Missing Slipper | D mixolydian, stately 4/4 march at 75 quarters/minute; pipes, harp, fiddle, low cello ground, light snare | 24 bars, 76.8 s | 1,693,456 |
| `gatehouse` | The Porter's Other Key | D dorian, sneaking 6/8 jig at 80 dotted quarters/minute; fiddle/whistle exchanges, cello pulses, harp, low toms and two short pipe replies | 48 bars, 72 s | 1,587,616 |

The title's middle strain passes to the fiddle and its return varies the melody
and cadence. The jig has six eight-bar conversations, including a quieter third
section. Melody stays between D4 and D5; sustained high whistle notes, cymbal
loops and constant snare rolls are absent. The low toms suggest a bodhran rather
than a rock kit. Music should sit underneath the game's effects; the playback
mixer owns that balance.

MIDI programs are **zero-based**: bagpipe 109, fiddle 110, harp 46, cello 42,
whistle 78. Drums use MIDI channel 10 (index 9): title acoustic snare 38;
gatehouse low floor tom 41, high floor tom 43 and soft side-stick 37.
These correspond to the synth's dedicated reed, bowed, plucked and wind voices.
Channels are centred because the baked asset is mono. No external soundfont is
needed. Render with the same ferrosintesis binary to reproduce the same audio.

## Loop and file format

Each MIDI contains **two identical complete passes**, with no intervening reset,
pause or fade. Render both in one call, including the synth's release/reverb.
The baker discards the first pass and retains precisely the second pass:

- `title`: input frames from `rate * 384 / 5` for another `rate * 384 / 5` frames.
- `gatehouse`: input frames from `rate * 72` for another `rate * 72` frames.

Thus the loop begins with the preceding pass's reverb and release tails already
present. The extra render tail after pass two is discarded. No silence is
inserted at the join. Both outputs have loop start **L = 0**, at 22050 Hz;
title has **N = 1693440**, gatehouse **N = 1587600** samples. This preserves the
whole musical period, including its final rests and their sounding tails.

`bake.braw` reports the decoded PCM step at `N-1 -> L`, plus the maximum adjacent
step among the last and first 512 samples and that join. Check these measurements
after rendering, and audition repeated playback before accepting the final assets;
two passes preserve tails but do not guarantee matching oscillator phases.

`BMU1` consists of 16 header bytes: ASCII `BMU1`, then big-endian u32 sample
rate, sample count N and loop start L. Exactly N mono G.711 mu-law bytes follow
(bias 132, clip 32635, complemented bits). The baker reads RIFF chunk lengths,
accepts signed 16-bit PCM at any positive rate/channel count, averages channels,
resamples with integer linear interpolation, and normalises the selected body's
peak to 23170 (about -3 dBFS before mu-law quantisation). It has no filtering
dependency; linear interpolation is intentionally simple. WAVs remain scratch
build output.

## Regenerate

Run from the mdhavers worktree root. Use the existing mdhavers CLI and the
lead-supplied ferrosintesis binary; do not build or edit the synth's main clone.
The bake runs as compiled JavaScript for the two long sample loops. Node is an
offline authoring tool only; the game does not use it.

```sh
MDH="$PWD/.scratch/mdhavers"
FERRO="/private/tmp/claude-501/-Users-md-language-mdhavers/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2/scratchpad/ferro/ferrosintesis"
mkdir -p .scratch/music
set -o pipefail

nice -n 19 deltic timeout 60 "$MDH" games/bogle/music/title.braw </dev/null 2>&1 | head -c 1048576 > .scratch/music/title-generator.log
nice -n 19 deltic timeout 60 "$MDH" games/bogle/music/gatehouse.braw </dev/null 2>&1 | head -c 1048576 > .scratch/music/gatehouse-generator.log

nice -n 19 deltic timeout 60 "$FERRO" games/bogle/music/title.mid -o .scratch/music/title.wav --rate 44100 --wet 0.24 --delay 0 --tail 6 -q </dev/null 2>&1 | head -c 1048576 > .scratch/music/title-render.log
nice -n 19 deltic timeout 60 "$FERRO" games/bogle/music/gatehouse.mid -o .scratch/music/gatehouse.wav --rate 44100 --wet 0.24 --delay 0 --tail 6 -q </dev/null 2>&1 | head -c 1048576 > .scratch/music/gatehouse-render.log

nice -n 19 deltic timeout 60 "$MDH" compile games/bogle/music/bake.braw -o .scratch/music/bake.js </dev/null 2>&1 | head -c 1048576 > .scratch/music/bake-compile.log
nice -n 19 deltic timeout 60 node .scratch/music/bake.js </dev/null 2>&1 | head -c 1048576 > .scratch/music/bake.log
cat .scratch/music/bake.log
```

Log caps apply to stdout/stderr, not the WAV or BMU output files. A process-wide
`ulimit -f` of 1 MB would prevent either of those assets being written.

The output files are `games/bogle/data/music_title.bmu` and
`games/bogle/data/music_gatehouse.bmu`. If a render times out, inspect its log and
WAV before retrying; no completed audio render is implied by an existing file.

## Focused checks

`games/bogle/tests/music_score.braw` pins variable-length MIDI integers, the
entire byte encoding of a small score, and stable off-before-on ordering.
`games/bogle/tests/music_bake.braw` builds its own small WAVs under `.scratch/music`
and checks chunk padding, unequal channels, signed PCM, rational cuts, 11025 Hz
interpolation, 44100 Hz downsampling, normalisation, standard mu-law vectors,
decoded sample values, silence, headers and invalid inputs. Each has a matching
`.expected` file and is picked up by the existing Bogle fixture harness.

```sh
nice -n 19 deltic timeout 60 "$MDH" games/bogle/tests/music_score.braw </dev/null > .scratch/music/score-test.log
diff -u games/bogle/tests/music_score.expected .scratch/music/score-test.log
nice -n 19 deltic timeout 60 "$MDH" games/bogle/tests/music_bake.braw </dev/null > .scratch/music/bake-test.log
diff -u games/bogle/tests/music_bake.expected .scratch/music/bake-test.log
```

For reuse, fetching `games/bogle/tools/bake_music` only defines functions.
`bake_music(input, output, start_frame, frame_count, loop_start_output)` returns
`rate`, `frames`, `loop_start` and the selected body's original PCM `peak`.
`music_wav_info(path)` returns `rate`, `channels` and `frames`;
`music_seconds_to_frames(rate, numerator, denominator)` rejects a duration that
does not land on an exact source frame.
