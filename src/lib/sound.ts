// Small UI sounds, synthesized on the fly. No audio files ship with the app.

export type SoundLevel = "off" | "soft" | "normal";
export type Cue = "start" | "stop" | "error" | "add" | "remove" | "tick";

let ctx: AudioContext | null = null;
let level: SoundLevel = "soft";
let last = 0;

const GAIN: Record<SoundLevel, number> = { off: 0, soft: 0.05, normal: 0.12 };

export function setSoundLevel(next: SoundLevel) {
  level = next;
}

function audio(): AudioContext | null {
  try {
    ctx ??= new AudioContext();
    if (ctx.state === "suspended") void ctx.resume();
    return ctx;
  } catch {
    return null;
  }
}

interface Note {
  freq: number;
  at: number;
  len: number;
  type?: OscillatorType;
  /** frequency to glide to over the note */
  to?: number;
  gain?: number;
}

function play(notes: Note[]) {
  const volume = GAIN[level];
  if (volume === 0) return;
  const c = audio();
  if (!c) return;

  const now = c.currentTime;
  for (const n of notes) {
    const osc = c.createOscillator();
    const amp = c.createGain();
    const t0 = now + n.at;
    osc.type = n.type ?? "sine";
    osc.frequency.setValueAtTime(n.freq, t0);
    if (n.to) osc.frequency.exponentialRampToValueAtTime(n.to, t0 + n.len);

    // Quick attack, exponential decay: reads as a soft tap rather than a beep.
    const peak = volume * (n.gain ?? 1);
    amp.gain.setValueAtTime(0.0001, t0);
    amp.gain.exponentialRampToValueAtTime(peak, t0 + 0.008);
    amp.gain.exponentialRampToValueAtTime(0.0001, t0 + n.len);

    osc.connect(amp).connect(c.destination);
    osc.start(t0);
    osc.stop(t0 + n.len + 0.02);
  }
}

const cues: Record<Cue, Note[]> = {
  // rising fifth
  start: [
    { freq: 659.25, at: 0, len: 0.16 },
    { freq: 987.77, at: 0.07, len: 0.26, gain: 0.9 },
  ],
  // falling fifth
  stop: [
    { freq: 783.99, at: 0, len: 0.14 },
    { freq: 523.25, at: 0.07, len: 0.24, gain: 0.9 },
  ],
  error: [{ freq: 220, to: 150, at: 0, len: 0.22, type: "triangle", gain: 1.3 }],
  add: [{ freq: 880, at: 0, len: 0.1, gain: 0.8 }],
  remove: [{ freq: 330, to: 260, at: 0, len: 0.1, type: "triangle", gain: 0.9 }],
  tick: [{ freq: 1500, at: 0, len: 0.035, gain: 0.55 }],
};

export function cue(name: Cue) {
  // Several cues can fire from one action; keep them from stacking.
  const t = performance.now();
  if (t - last < 40 && name === "tick") return;
  last = t;
  play(cues[name]);
}
