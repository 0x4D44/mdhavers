// WASM audio host helpers for mdhavers WAT backend.
// Requires audio_runtime.js to be loaded first.

function mdh_wasm_audio_imports(memory) {
  const decoder = new TextDecoder("utf-8");

  function toNumber(v) {
    if (typeof v === "bigint") return Number(v);
    return Number(v);
  }

  function toBool(v) {
    return toNumber(v) !== 0;
  }

  function toBigInt(v) {
    if (typeof v === "bigint") return v;
    if (typeof v === "boolean") return v ? 1n : 0n;
    if (typeof v === "number") return BigInt(Math.trunc(v));
    return 0n;
  }

  function readCString(ptr) {
    const mem = new Uint8Array(memory.buffer);
    let p = toNumber(ptr);
    let end = p;
    while (mem[end] !== 0) end++;
    return decoder.decode(mem.subarray(p, end));
  }

  function call(fn, args) {
    const result = fn.apply(null, args);
    return toBigInt(result);
  }

  return {
    soond_stairt: () => call(__havers_audio.soond_stairt, []),
    soond_steek: () => call(__havers_audio.soond_steek, []),
    soond_wheesht: (v) => call(__havers_audio.soond_wheesht, [toBool(v)]),
    soond_luid: (v) => call(__havers_audio.soond_luid, [toNumber(v)]),
    soond_hou_luid: () => call(__havers_audio.soond_hou_luid, []),
    soond_haud_gang: () => call(__havers_audio.soond_haud_gang, []),
    soond_lade: (ptr) => call(__havers_audio.soond_lade, [readCString(ptr)]),
    soond_spiel: (h) => call(__havers_audio.soond_spiel, [toNumber(h)]),
    soond_haud: (h) => call(__havers_audio.soond_haud, [toNumber(h)]),
    soond_gae_on: (h) => call(__havers_audio.soond_gae_on, [toNumber(h)]),
    soond_stap: (h) => call(__havers_audio.soond_stap, [toNumber(h)]),
    soond_unlade: (h) => call(__havers_audio.soond_unlade, [toNumber(h)]),
    soond_is_spielin: (h) => call(__havers_audio.soond_is_spielin, [toNumber(h)]),
    soond_pit_luid: (h, v) => call(__havers_audio.soond_pit_luid, [toNumber(h), toNumber(v)]),
    soond_pit_pan: (h, v) => call(__havers_audio.soond_pit_pan, [toNumber(h), toNumber(v)]),
    soond_pit_tune: (h, v) => call(__havers_audio.soond_pit_tune, [toNumber(h), toNumber(v)]),
    soond_pit_rin_roond: (h, v) => call(__havers_audio.soond_pit_rin_roond, [toNumber(h), toBool(v)]),
    soond_ready: (h) => call(__havers_audio.soond_ready, [toNumber(h)]),

    muisic_lade: (ptr) => call(__havers_audio.muisic_lade, [readCString(ptr)]),
    muisic_spiel: (h) => call(__havers_audio.muisic_spiel, [toNumber(h)]),
    muisic_haud: (h) => call(__havers_audio.muisic_haud, [toNumber(h)]),
    muisic_gae_on: (h) => call(__havers_audio.muisic_gae_on, [toNumber(h)]),
    muisic_stap: (h) => call(__havers_audio.muisic_stap, [toNumber(h)]),
    muisic_unlade: (h) => call(__havers_audio.muisic_unlade, [toNumber(h)]),
    muisic_is_spielin: (h) => call(__havers_audio.muisic_is_spielin, [toNumber(h)]),
    muisic_loup: (h, v) => call(__havers_audio.muisic_loup, [toNumber(h), toNumber(v)]),
    muisic_hou_lang: (h) => call(__havers_audio.muisic_hou_lang, [toNumber(h)]),
    muisic_whaur: (h) => call(__havers_audio.muisic_whaur, [toNumber(h)]),
    muisic_pit_luid: (h, v) => call(__havers_audio.muisic_pit_luid, [toNumber(h), toNumber(v)]),
    muisic_pit_pan: (h, v) => call(__havers_audio.muisic_pit_pan, [toNumber(h), toNumber(v)]),
    muisic_pit_tune: (h, v) => call(__havers_audio.muisic_pit_tune, [toNumber(h), toNumber(v)]),
    muisic_pit_rin_roond: (h, v) => call(__havers_audio.muisic_pit_rin_roond, [toNumber(h), toBool(v)]),

    midi_lade: (midi_ptr, sf_ptr) => {
      const midiPath = readCString(midi_ptr);
      const sfPath = toNumber(sf_ptr) === 0 ? null : readCString(sf_ptr);
      return call(__havers_audio.midi_lade, [midiPath, sfPath]);
    },
    midi_spiel: (h) => call(__havers_audio.midi_spiel, [toNumber(h)]),
    midi_haud: (h) => call(__havers_audio.midi_haud, [toNumber(h)]),
    midi_gae_on: (h) => call(__havers_audio.midi_gae_on, [toNumber(h)]),
    midi_stap: (h) => call(__havers_audio.midi_stap, [toNumber(h)]),
    midi_unlade: (h) => call(__havers_audio.midi_unlade, [toNumber(h)]),
    midi_is_spielin: (h) => call(__havers_audio.midi_is_spielin, [toNumber(h)]),
    midi_loup: (h, v) => call(__havers_audio.midi_loup, [toNumber(h), toNumber(v)]),
    midi_hou_lang: (h) => call(__havers_audio.midi_hou_lang, [toNumber(h)]),
    midi_whaur: (h) => call(__havers_audio.midi_whaur, [toNumber(h)]),
    midi_pit_luid: (h, v) => call(__havers_audio.midi_pit_luid, [toNumber(h), toNumber(v)]),
    midi_pit_pan: (h, v) => call(__havers_audio.midi_pit_pan, [toNumber(h), toNumber(v)]),
    midi_pit_rin_roond: (h, v) => call(__havers_audio.midi_pit_rin_roond, [toNumber(h), toBool(v)]),
  };
}

if (typeof globalThis !== "undefined") {
  globalThis.mdh_wasm_audio_imports = mdh_wasm_audio_imports;
}
