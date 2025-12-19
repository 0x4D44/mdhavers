// mdhavers audio runtime (WebAudio + rustysynth wasm)
const __havers_audio = (() => {
  const AudioCtx = typeof AudioContext !== "undefined"
    ? AudioContext
    : (typeof webkitAudioContext !== "undefined" ? webkitAudioContext : null);
  const hasFetch = typeof fetch === "function";

  const state = {
    ctx: null,
    masterGain: null,
    muted: false,
    masterVolume: 1.0,
    pendingErrors: [],
    soundNext: 0,
    musicNext: 0,
    midiNext: 0,
    sounds: new Map(),
    music: new Map(),
    midi: new Map(),
  };

  const DEFAULT_SOUNDFONT = "assets/soundfonts/MuseScore_General.sf2";
  const DEFAULT_MIDI_WASM = "assets/wasm/mdh_rustysynth.wasm";
  const MIDI_SAMPLE_RATE = 44100;

  function clamp01(v) {
    if (v < 0) return 0;
    if (v > 1) return 1;
    return v;
  }

  function requireAudio() {
    if (!AudioCtx) {
      throw new Error("Soond isnae available in this environment");
    }
  }

  function ensureCtx() {
    requireAudio();
    if (!state.ctx) {
      state.ctx = new AudioCtx();
      state.masterGain = state.ctx.createGain();
      state.masterGain.gain.value = state.masterVolume;
      state.masterGain.connect(state.ctx.destination);
    }
    if (state.ctx.state === "suspended") {
      state.ctx.resume().catch(() => {});
    }
    return state.ctx;
  }

  function applyMaster() {
    if (!state.masterGain) return;
    state.masterGain.gain.value = state.muted ? 0 : state.masterVolume;
  }

  function resolveBase() {
    if (typeof globalThis !== "undefined" && globalThis.__havers_audio_base) {
      let base = String(globalThis.__havers_audio_base);
      if (base && !base.endsWith("/")) base += "/";
      return base;
    }
    return "";
  }

  function resolvePath(path) {
    if (!path) return "";
    const str = String(path);
    if (/^(https?:|data:|blob:)/.test(str)) return str;
    return resolveBase() + str;
  }

  function resolveSoundfontPath(path) {
    if (!path || path === "naething") {
      if (typeof globalThis !== "undefined" && globalThis.__havers_soundfont) {
        return resolvePath(globalThis.__havers_soundfont);
      }
      return resolvePath(DEFAULT_SOUNDFONT);
    }
    return resolvePath(path);
  }

  function resolveMidiWasmPath() {
    if (typeof globalThis !== "undefined" && globalThis.__havers_midi_wasm) {
      return resolvePath(globalThis.__havers_midi_wasm);
    }
    return resolvePath(DEFAULT_MIDI_WASM);
  }

  function makePanNode(ctx) {
    if (typeof ctx.createStereoPanner === "function") {
      return ctx.createStereoPanner();
    }
    const panner = ctx.createPanner();
    panner.panningModel = "equalpower";
    panner.setPosition(0, 0, 1);
    panner.pan = { value: 0 };
    return panner;
  }

  function queueError(err) {
    state.pendingErrors.push(err);
  }

  function flushErrors() {
    if (state.pendingErrors.length) {
      const err = state.pendingErrors.shift();
      throw err;
    }
  }

  async function loadArrayBuffer(url, errMsg) {
    if (!hasFetch) {
      throw new Error(errMsg);
    }
    const res = await fetch(url);
    if (!res.ok) {
      throw new Error(errMsg);
    }
    return await res.arrayBuffer();
  }

  async function loadAudioBuffer(url) {
    const ctx = ensureCtx();
    const data = await loadArrayBuffer(url, "Cannae lade the soond");
    return await ctx.decodeAudioData(data);
  }

  function makeBufferEntry() {
    const ctx = ensureCtx();
    const gain = ctx.createGain();
    const pan = makePanNode(ctx);
    gain.connect(pan);
    pan.connect(state.masterGain);
    return {
      ready: false,
      buffer: null,
      source: null,
      gain,
      pan,
      volume: 1.0,
      panValue: 0.0,
      pitch: 1.0,
      looped: false,
      state: "stopped",
      offset: 0.0,
      startTime: 0.0,
      pendingPlay: false,
    };
  }

  function playBufferEntry(entry) {
    const ctx = ensureCtx();
    if (!entry.ready) {
      entry.pendingPlay = true;
      entry.state = "playing";
      return;
    }
    if (entry.source) {
      try { entry.source.stop(); } catch (_) {}
    }
    const source = ctx.createBufferSource();
    source.buffer = entry.buffer;
    source.loop = entry.looped;
    source.playbackRate.value = entry.pitch;
    entry.gain.gain.value = entry.volume;
    if (entry.pan && typeof entry.pan.pan !== "undefined") {
      entry.pan.pan.value = entry.panValue;
    }
    source.connect(entry.gain);
    entry.source = source;
    entry.startTime = ctx.currentTime;
    entry.state = "playing";
    source.onended = () => {
      if (entry.state === "playing" && !entry.looped) {
        entry.state = "stopped";
        entry.offset = 0;
        entry.source = null;
      }
    };
    try {
      source.start(0, entry.offset || 0);
    } catch (_) {
      entry.state = "stopped";
      entry.source = null;
    }
  }

  function pauseBufferEntry(entry) {
    if (entry.state !== "playing") return;
    const ctx = ensureCtx();
    if (entry.source) {
      const elapsed = ctx.currentTime - entry.startTime;
      entry.offset = (entry.offset || 0) + Math.max(0, elapsed);
      try { entry.source.stop(); } catch (_) {}
      entry.source = null;
    }
    entry.state = "paused";
  }

  function resumeBufferEntry(entry) {
    if (entry.state !== "paused") return;
    playBufferEntry(entry);
  }

  function stopBufferEntry(entry) {
    if (entry.source) {
      try { entry.source.stop(); } catch (_) {}
      entry.source = null;
    }
    entry.state = "stopped";
    entry.offset = 0;
  }

  function loadSound(handle, path) {
    const entry = state.sounds.get(handle);
    if (!entry) return;
    loadAudioBuffer(resolvePath(path))
      .then((buf) => {
        entry.buffer = buf;
        entry.ready = true;
        if (entry.pendingPlay) {
          entry.pendingPlay = false;
          playBufferEntry(entry);
        }
      })
      .catch(() => {
        queueError(new Error("Cannae lade the soond"));
      });
  }

  function loadMidi(handle, midiPath, sfPath) {
    const entry = state.midi.get(handle);
    if (!entry) return;
    const midiUrl = resolvePath(midiPath);
    const sfUrl = resolveSoundfontPath(sfPath);
    Promise.all([
      loadArrayBuffer(midiUrl, "Cannae read the midi"),
      loadArrayBuffer(sfUrl, "Cannae read the soondfont"),
      loadRustySynth(),
    ])
      .then(([midiBuf, sfBuf, wasm]) => {
        return renderMidi(wasm, sfBuf, midiBuf);
      })
      .then((rendered) => {
        const ctx = ensureCtx();
        const buffer = ctx.createBuffer(2, rendered.frames, MIDI_SAMPLE_RATE);
        const left = buffer.getChannelData(0);
        const right = buffer.getChannelData(1);
        const data = rendered.data;
        for (let i = 0, j = 0; i < rendered.frames; i++) {
          left[i] = data[j++];
          right[i] = data[j++];
        }
        entry.buffer = buffer;
        entry.length = buffer.duration;
        entry.ready = true;
        if (entry.pendingPlay) {
          entry.pendingPlay = false;
          playBufferEntry(entry);
        }
      })
      .catch((err) => {
        queueError(new Error(err && err.message ? err.message : "Cannae read the midi"));
      });
  }

  function makeMusicEntry(path) {
    const ctx = ensureCtx();
    const audio = new Audio();
    audio.preload = "auto";
    audio.crossOrigin = "anonymous";
    audio.src = resolvePath(path);
    const source = ctx.createMediaElementSource(audio);
    const gain = ctx.createGain();
    const pan = makePanNode(ctx);
    source.connect(gain);
    gain.connect(pan);
    pan.connect(state.masterGain);
    const entry = {
      audio,
      source,
      gain,
      pan,
      volume: 1.0,
      panValue: 0.0,
      pitch: 1.0,
      looped: false,
      state: "stopped",
      ready: false,
    };
    audio.addEventListener("canplay", () => {
      entry.ready = true;
    });
    audio.addEventListener("ended", () => {
      if (!entry.looped) {
        entry.state = "stopped";
      }
    });
    return entry;
  }

  let rustysynthPromise = null;

  function loadRustySynth() {
    if (rustysynthPromise) return rustysynthPromise;
    rustysynthPromise = (async () => {
      if (typeof WebAssembly === "undefined") {
        throw new Error("MIDI needs WebAssembly");
      }
      const wasmUrl = resolveMidiWasmPath();
      const wasmBuf = await loadArrayBuffer(wasmUrl, "Cannae load the midi synth");
      const result = await WebAssembly.instantiate(wasmBuf, {});
      const exports = result.instance.exports;
      if (!exports || !exports.memory || !exports.alloc || !exports.render_midi) {
        throw new Error("MIDI synth isnae richt");
      }
      return exports;
    })();
    return rustysynthPromise;
  }

  function renderMidi(wasm, sfBuf, midiBuf) {
    const mem = new Uint8Array(wasm.memory.buffer);
    const sfPtr = wasm.alloc(sfBuf.byteLength);
    mem.set(new Uint8Array(sfBuf), sfPtr);
    const midiPtr = wasm.alloc(midiBuf.byteLength);
    mem.set(new Uint8Array(midiBuf), midiPtr);
    const ptr = wasm.render_midi(sfPtr, sfBuf.byteLength, midiPtr, midiBuf.byteLength, MIDI_SAMPLE_RATE);
    wasm.dealloc(sfPtr, sfBuf.byteLength);
    wasm.dealloc(midiPtr, midiBuf.byteLength);
    const len = wasm.render_midi_len();
    const frames = wasm.render_midi_frames();
    if (!ptr) {
      const errMsg = getRustyError(wasm);
      if (!errMsg && len === 0 && frames === 0) {
        return { data: new Float32Array(0), frames: 0 };
      }
      throw new Error(errMsg || "Cannae read the midi");
    }
    const view = new Float32Array(wasm.memory.buffer, ptr, len);
    const data = new Float32Array(len);
    data.set(view);
    wasm.render_midi_free(ptr, len);
    return { data, frames };
  }

  function getRustyError(wasm) {
    if (!wasm.last_error_ptr || !wasm.last_error_len) return "";
    const ptr = wasm.last_error_ptr();
    const len = wasm.last_error_len();
    if (!ptr || !len) return "";
    const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
    try {
      return new TextDecoder("utf-8").decode(bytes);
    } catch (_) {
      return "";
    }
  }

  function ensureSound(handle, name) {
    const entry = state.sounds.get(handle);
    if (!entry) throw new Error("Thon handle isnae guid");
    return entry;
  }

  function ensureMusic(handle) {
    const entry = state.music.get(handle);
    if (!entry) throw new Error("Thon handle isnae guid");
    return entry;
  }

  function ensureMidi(handle) {
    const entry = state.midi.get(handle);
    if (!entry) throw new Error("Thon handle isnae guid");
    return entry;
  }

  return {
    soond_stairt() {
      ensureCtx();
      applyMaster();
      return null;
    },
    soond_steek() {
      for (const entry of state.sounds.values()) {
        stopBufferEntry(entry);
      }
      for (const entry of state.music.values()) {
        entry.audio.pause();
        try { entry.audio.currentTime = 0; } catch (_) {}
      }
      for (const entry of state.midi.values()) {
        stopBufferEntry(entry);
      }
      state.sounds.clear();
      state.music.clear();
      state.midi.clear();
      if (state.ctx) {
        const ctx = state.ctx;
        state.ctx = null;
        state.masterGain = null;
        try { ctx.close(); } catch (_) {}
      } else {
        state.ctx = null;
        state.masterGain = null;
      }
      return null;
    },
    soond_wheesht(on) {
      if (typeof on !== "boolean") throw new Error("soond_wheesht needs aye or nae");
      ensureCtx();
      state.muted = on;
      applyMaster();
      return null;
    },
    soond_luid(v) {
      if (typeof v !== "number") throw new Error("soond_luid needs a nummer");
      ensureCtx();
      state.masterVolume = clamp01(v);
      applyMaster();
      return null;
    },
    soond_hou_luid() {
      return state.masterVolume;
    },
    soond_haud_gang() {
      flushErrors();
      return null;
    },
    soond_lade(path) {
      if (typeof path !== "string") throw new Error("soond_lade needs a string path");
      const handle = state.soundNext++;
      const entry = makeBufferEntry();
      state.sounds.set(handle, entry);
      loadSound(handle, path);
      return handle;
    },
    soond_ready(handle) {
      if (typeof handle !== "number") throw new Error("soond_ready needs a guid handle");
      const entry = ensureSound(handle);
      return !!entry.ready;
    },
    soond_spiel(handle) {
      if (typeof handle !== "number") throw new Error("soond_spiel needs a guid handle");
      const entry = ensureSound(handle);
      playBufferEntry(entry);
      return null;
    },
    soond_haud(handle) {
      if (typeof handle !== "number") throw new Error("soond_haud needs a guid handle");
      const entry = ensureSound(handle);
      pauseBufferEntry(entry);
      return null;
    },
    soond_gae_on(handle) {
      if (typeof handle !== "number") throw new Error("soond_gae_on needs a guid handle");
      const entry = ensureSound(handle);
      resumeBufferEntry(entry);
      return null;
    },
    soond_stap(handle) {
      if (typeof handle !== "number") throw new Error("soond_stap needs a guid handle");
      const entry = ensureSound(handle);
      stopBufferEntry(entry);
      return null;
    },
    soond_unlade(handle) {
      if (typeof handle !== "number") throw new Error("soond_unlade needs a guid handle");
      const entry = ensureSound(handle);
      stopBufferEntry(entry);
      state.sounds.delete(handle);
      return null;
    },
    soond_is_spielin(handle) {
      if (typeof handle !== "number") throw new Error("soond_is_spielin needs a guid handle");
      const entry = ensureSound(handle);
      return entry.state === "playing";
    },
    soond_pit_luid(handle, v) {
      if (typeof handle !== "number") throw new Error("soond_pit_luid needs a guid handle");
      if (typeof v !== "number") throw new Error("soond_pit_luid needs a nummer");
      const entry = ensureSound(handle);
      entry.volume = clamp01(v);
      if (entry.gain) entry.gain.gain.value = entry.volume;
      return null;
    },
    soond_pit_pan(handle, v) {
      if (typeof handle !== "number") throw new Error("soond_pit_pan needs a guid handle");
      if (typeof v !== "number") throw new Error("soond_pit_pan needs a nummer");
      const entry = ensureSound(handle);
      entry.panValue = v;
      if (entry.pan && typeof entry.pan.pan !== "undefined") {
        entry.pan.pan.value = v;
      }
      return null;
    },
    soond_pit_tune(handle, v) {
      if (typeof handle !== "number") throw new Error("soond_pit_tune needs a guid handle");
      if (typeof v !== "number") throw new Error("soond_pit_tune needs a nummer");
      const entry = ensureSound(handle);
      entry.pitch = v;
      if (entry.source) entry.source.playbackRate.value = v;
      return null;
    },
    soond_pit_rin_roond(handle, on) {
      if (typeof handle !== "number") throw new Error("soond_pit_rin_roond needs a guid handle");
      if (typeof on !== "boolean") throw new Error("soond_pit_rin_roond needs aye or nae");
      const entry = ensureSound(handle);
      entry.looped = on;
      if (entry.source) entry.source.loop = on;
      return null;
    },

    muisic_lade(path) {
      if (typeof path !== "string") throw new Error("muisic_lade needs a string path");
      const handle = state.musicNext++;
      const entry = makeMusicEntry(path);
      state.music.set(handle, entry);
      return handle;
    },
    muisic_spiel(handle) {
      if (typeof handle !== "number") throw new Error("muisic_spiel needs a guid handle");
      const entry = ensureMusic(handle);
      ensureCtx();
      entry.audio.play().catch(() => {});
      entry.state = "playing";
      return null;
    },
    muisic_haud(handle) {
      if (typeof handle !== "number") throw new Error("muisic_haud needs a guid handle");
      const entry = ensureMusic(handle);
      entry.audio.pause();
      entry.state = "paused";
      return null;
    },
    muisic_gae_on(handle) {
      if (typeof handle !== "number") throw new Error("muisic_gae_on needs a guid handle");
      const entry = ensureMusic(handle);
      entry.audio.play().catch(() => {});
      entry.state = "playing";
      return null;
    },
    muisic_stap(handle) {
      if (typeof handle !== "number") throw new Error("muisic_stap needs a guid handle");
      const entry = ensureMusic(handle);
      entry.audio.pause();
      try { entry.audio.currentTime = 0; } catch (_) {}
      entry.state = "stopped";
      return null;
    },
    muisic_unlade(handle) {
      if (typeof handle !== "number") throw new Error("muisic_unlade needs a guid handle");
      const entry = ensureMusic(handle);
      entry.audio.pause();
      state.music.delete(handle);
      return null;
    },
    muisic_is_spielin(handle) {
      if (typeof handle !== "number") throw new Error("muisic_is_spielin needs a guid handle");
      const entry = ensureMusic(handle);
      return entry.state === "playing" && !entry.audio.paused;
    },
    muisic_loup(handle, seconds) {
      if (typeof handle !== "number") throw new Error("muisic_loup needs a guid handle");
      if (typeof seconds !== "number") throw new Error("muisic_loup needs a nummer");
      const entry = ensureMusic(handle);
      try { entry.audio.currentTime = seconds; } catch (_) {}
      return null;
    },
    muisic_hou_lang(handle) {
      if (typeof handle !== "number") throw new Error("muisic_hou_lang needs a guid handle");
      const entry = ensureMusic(handle);
      const d = entry.audio.duration;
      return Number.isFinite(d) ? d : 0;
    },
    muisic_whaur(handle) {
      if (typeof handle !== "number") throw new Error("muisic_whaur needs a guid handle");
      const entry = ensureMusic(handle);
      return entry.audio.currentTime || 0;
    },
    muisic_pit_luid(handle, v) {
      if (typeof handle !== "number") throw new Error("muisic_pit_luid needs a guid handle");
      if (typeof v !== "number") throw new Error("muisic_pit_luid needs a nummer");
      const entry = ensureMusic(handle);
      entry.volume = clamp01(v);
      entry.gain.gain.value = entry.volume;
      return null;
    },
    muisic_pit_pan(handle, v) {
      if (typeof handle !== "number") throw new Error("muisic_pit_pan needs a guid handle");
      if (typeof v !== "number") throw new Error("muisic_pit_pan needs a nummer");
      const entry = ensureMusic(handle);
      entry.panValue = v;
      if (entry.pan && typeof entry.pan.pan !== "undefined") {
        entry.pan.pan.value = v;
      }
      return null;
    },
    muisic_pit_tune(handle, v) {
      if (typeof handle !== "number") throw new Error("muisic_pit_tune needs a guid handle");
      if (typeof v !== "number") throw new Error("muisic_pit_tune needs a nummer");
      const entry = ensureMusic(handle);
      entry.pitch = v;
      entry.audio.playbackRate = v;
      return null;
    },
    muisic_pit_rin_roond(handle, on) {
      if (typeof handle !== "number") throw new Error("muisic_pit_rin_roond needs a guid handle");
      if (typeof on !== "boolean") throw new Error("muisic_pit_rin_roond needs aye or nae");
      const entry = ensureMusic(handle);
      entry.looped = on;
      entry.audio.loop = on;
      return null;
    },

    midi_lade(path, soundfont) {
      if (typeof path !== "string") throw new Error("midi_lade needs a midi filepath");
      const handle = state.midiNext++;
      const entry = makeBufferEntry();
      entry.length = 0;
      state.midi.set(handle, entry);
      const sfPath = soundfont === null || typeof soundfont === "undefined" ? "" : soundfont;
      loadMidi(handle, path, sfPath);
      return handle;
    },
    midi_spiel(handle) {
      if (typeof handle !== "number") throw new Error("midi_spiel needs a guid handle");
      const entry = ensureMidi(handle);
      playBufferEntry(entry);
      return null;
    },
    midi_haud(handle) {
      if (typeof handle !== "number") throw new Error("midi_haud needs a guid handle");
      const entry = ensureMidi(handle);
      pauseBufferEntry(entry);
      return null;
    },
    midi_gae_on(handle) {
      if (typeof handle !== "number") throw new Error("midi_gae_on needs a guid handle");
      const entry = ensureMidi(handle);
      resumeBufferEntry(entry);
      return null;
    },
    midi_stap(handle) {
      if (typeof handle !== "number") throw new Error("midi_stap needs a guid handle");
      const entry = ensureMidi(handle);
      stopBufferEntry(entry);
      return null;
    },
    midi_unlade(handle) {
      if (typeof handle !== "number") throw new Error("midi_unlade needs a guid handle");
      const entry = ensureMidi(handle);
      stopBufferEntry(entry);
      state.midi.delete(handle);
      return null;
    },
    midi_is_spielin(handle) {
      if (typeof handle !== "number") throw new Error("midi_is_spielin needs a guid handle");
      const entry = ensureMidi(handle);
      return entry.state === "playing";
    },
    midi_loup(handle, seconds) {
      if (typeof handle !== "number") throw new Error("midi_loup needs a guid handle");
      if (typeof seconds !== "number") throw new Error("midi_loup needs a nummer");
      const entry = ensureMidi(handle);
      entry.offset = Math.max(0, seconds || 0);
      if (entry.state === "playing") {
        stopBufferEntry(entry);
        playBufferEntry(entry);
      }
      return null;
    },
    midi_hou_lang(handle) {
      if (typeof handle !== "number") throw new Error("midi_hou_lang needs a guid handle");
      const entry = ensureMidi(handle);
      return entry.length || 0;
    },
    midi_whaur(handle) {
      if (typeof handle !== "number") throw new Error("midi_whaur needs a guid handle");
      const entry = ensureMidi(handle);
      if (entry.state !== "playing") return entry.offset || 0;
      const ctx = ensureCtx();
      return (entry.offset || 0) + Math.max(0, ctx.currentTime - entry.startTime);
    },
    midi_pit_luid(handle, v) {
      if (typeof handle !== "number") throw new Error("midi_pit_luid needs a guid handle");
      if (typeof v !== "number") throw new Error("midi_pit_luid needs a nummer");
      const entry = ensureMidi(handle);
      entry.volume = clamp01(v);
      if (entry.gain) entry.gain.gain.value = entry.volume;
      return null;
    },
    midi_pit_pan(handle, v) {
      if (typeof handle !== "number") throw new Error("midi_pit_pan needs a guid handle");
      if (typeof v !== "number") throw new Error("midi_pit_pan needs a nummer");
      const entry = ensureMidi(handle);
      entry.panValue = v;
      if (entry.pan && typeof entry.pan.pan !== "undefined") {
        entry.pan.pan.value = v;
      }
      return null;
    },
    midi_pit_rin_roond(handle, on) {
      if (typeof handle !== "number") throw new Error("midi_pit_rin_roond needs a guid handle");
      if (typeof on !== "boolean") throw new Error("midi_pit_rin_roond needs aye or nae");
      const entry = ensureMidi(handle);
      entry.looped = on;
      if (entry.source) entry.source.loop = on;
      return null;
    },
  };
})();
if (typeof globalThis !== "undefined") {
  globalThis.__havers_audio = __havers_audio;
}
