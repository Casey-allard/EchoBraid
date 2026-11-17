// The EchoBraid reference oracle.
//
// A line-for-line mirror of the LexLean model in app/src, used for one thing:
// to compute the acceptance vectors the holo carries (`node scripts/reference.mjs
// --write`). It decides nothing on the live page — every decision there is the
// verified guest's. `prismpm verify` replays each vector in the compiled guest,
// so a divergence between this file and the model fails the build: the oracle
// cannot silently disagree with what was proved.
//
// The frontier rhythms the engine discovers are read back from
// app/src/Braid/Frontier.lex.tex, so the engine and this oracle share one source.
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const MODEL = join(root, "app/src/App.lex.tex");
const FRONTIER = join(root, "app/src/Braid/Frontier.lex.tex");

// Braid.Text
const decimal = (n) => String(n);
const fields = (values) => values.join("\t");

// Braid.Resonance
const clampDrift = (v) => (v <= 100 ? v : 100);
const sub = (a, b) => (a > b ? a - b : 0);
export const driftStep = (drift, length) =>
  100 < length ? clampDrift(drift + 10) : length < 20 ? sub(clampDrift(drift), 10) : clampDrift(drift);
export const restStep = (drift) => sub(clampDrift(drift), 30);
export const coherenceRank = (d) => (d < 30 ? 0 : d < 70 ? 1 : 2);
export const offersPause = (d) => 80 < d;
const coherenceName = (d) => ["stable", "drifting", "divergent"][coherenceRank(d)];
const driftReply = (d) => fields([decimal(d), coherenceName(d), offersPause(d) ? "pause" : "flow"]);

// Braid.Breath
export const RELAX = { inhale: 4, hold: 7, exhale: 8, rest: 0 };
export const cycleLength = (r) => r.inhale + r.hold + r.exhale + r.rest;
export const phaseAt = (r, p) =>
  p < r.inhale ? 0 : p < r.inhale + r.hold ? 1 : p < r.inhale + r.hold + r.exhale ? 2 : 3;
export const phaseLength = (r, p) => [r.inhale, r.hold, r.exhale, r.rest][phaseAt(r, p)];
export const countAt = (r, p) =>
  p < r.inhale ? sub(r.inhale, p)
    : p < r.inhale + r.hold ? sub(r.inhale + r.hold, p)
      : p < r.inhale + r.hold + r.exhale ? sub(r.inhale + r.hold + r.exhale, p)
        : sub(cycleLength(r), p);
export const positionOf = (r, tick) => (cycleLength(r) === 0 ? 0 : tick % cycleLength(r));
const phaseName = (p) => ["inhale", "hold", "exhale", "rest"][p] ?? "rest";
export const rhythmSafe = (r) =>
  2 <= r.inhale && r.inhale <= 8 && r.hold <= 8 && r.inhale <= r.exhale && r.exhale <= 10 && r.rest <= 6 && cycleLength(r) <= 30;

export function frontier(text = readFileSync(FRONTIER, "utf8")) {
  const rhythms = [...text.matchAll(/record Braid\.Breath::Rhythm \{inhale := (\d+), hold := (\d+), exhale := (\d+), rest := (\d+)\}/g)]
    .map((m) => ({ inhale: +m[1], hold: +m[2], exhale: +m[3], rest: +m[4] }));
  const names = [...(text.match(/name: frontierName,[\s\S]*?body: ([^\n]*)/)?.[1] ?? "").matchAll(/then "([^"]*)"/g)].map((m) => m[1]);
  return { rhythms, names };
}

// Braid.Consent
export const nextFlag = (current, grant, revoke, action) =>
  action === 7 ? 0 : action === grant ? 1 : action === revoke ? 0 : current === 0 ? 0 : 1;

// Braid.Memory: Rule 30 on a 16-cell ring.
export const levelLength = (l) => [0, 3, 5, 7, 11, 13, 17, 23][l] ?? 0;
const cell = (s, i) => Math.floor(s / 2 ** (i % 16)) % 2;
const rule30 = (l, c, r) => (c + r === 0 ? l : l === 0 ? 1 : 0);
export function step(state) {
  let acc = 0;
  for (let i = 0; i < 16; i++) acc += rule30(cell(state, i + 15), cell(state, i), cell(state, i + 1)) * 2 ** i;
  return acc;
}
export const seedState = (seed) => (seed % 255) * 257 + 1;
const tileOf = (s) => Math.floor(s / 64) % 4;
export function tileAt(seed, index) {
  let s = seedState(seed);
  for (let k = 0; k <= index; k++) s = step(s);
  return tileOf(s);
}
export function sequenceText(seed, level) {
  let s = seedState(seed);
  let out = "";
  for (let k = 0; k < levelLength(level); k++) {
    s = step(s);
    out += tileOf(s);
  }
  return out;
}
export const verdictCode = (level, index, expected, chosen) =>
  chosen === expected ? (index + 1 === levelLength(level) ? (level === 7 ? 3 : 2) : 1) : 0;
const verdictName = (c) => ["wrong", "correct", "level", "victory"][c];

// Braid.Reflection
const PROMPTS = [
  [2, "Clarity", "What is one thing that feels true right now?"],
  [3, "Intention", "Start, middle, end. Which part feels heaviest?"],
  [5, "Emotion", "Name five textures in your current experience."],
  [7, "Structure", "If this feeling had a shape, how many sides would it have?"],
];
export const COPY = [
  "We can just sit. Your pace is law here.",
  "Breathe or write when ready...",
  "We may be nearing something unsayable. Would a brief pause help?",
  "Notice any shift, even subtle? No need to explain.",
  "This session will vanish in 24 hours.",
  "Voice can help me hear your pace. Shall we listen? You can turn this off anytime.",
];
export const GUIDE = [
  COPY[2],
  "Steady. The words arrived and settled. Nothing needs to happen next.",
  "There is ground under this. Stay as long as it feels right.",
  "Quiet and clear. You can rest here, or keep going, at your pace.",
  "Something is moving. It does not need a name yet.",
  "The thread loosens a little. That is allowed. We can follow it slowly.",
  "A few directions at once. Any one of them is enough for now.",
  "Many things at once. Feet on the floor can be a small anchor, if it helps.",
  "This is a lot. Silence is welcome here too.",
  "The edges feel far apart. Nothing has to be solved right now.",
];
export const guideIndex = (drift, variant) => (offersPause(drift) ? 0 : 1 + coherenceRank(drift) * 3 + (variant % 3));

// Braid.Logic
const overlap = (a, b, j) => sub(a + b, j);
const quorumWellFormed = (n, a, b, j) => j <= n && a <= j && b <= j && j <= a + b;
const quorumHolds = (f, a, b, j) => f + 1 <= overlap(a, b, j);
const ringAdmit = (cap, read, write) => (sub(cap, sub(write, read)) < 4 ? sub(cap, sub(write, read)) : 4);
const ringWrite = (cap, read, write) => write + ringAdmit(cap, read, write);

// Braid.Retention
export const shouldErase = (policy, age) => policy === 0 || (policy === 1 && 24 <= age) || (policy === 2 && 168 <= age);

// App.dispatch
const INVALID = "invalid request";
export function dispatch(bytes, front = frontier()) {
  const a = (i) => bytes[i] ?? 0;
  const op = a(0);
  const size = bytes.length;
  const rhythmAt = (k) => (k === 0 ? RELAX : front.rhythms[k - 1] ?? RELAX);
  const rhythmName = (k) => (k === 0 ? "relax 4-7-8" : k <= front.rhythms.length ? "frontier" : "relax 4-7-8");
  if (op === 68 && size === 3) return driftReply(driftStep(a(1), a(2)));
  if (op === 82 && size === 2) return driftReply(restStep(a(1)));
  if (op === 66 && size === 3) {
    const r = rhythmAt(a(1));
    const p = positionOf(r, a(2));
    return fields([phaseName(phaseAt(r, p)), decimal(countAt(r, p)), decimal(phaseLength(r, p)), decimal(cycleLength(r)), rhythmName(a(1))]);
  }
  if (op === 67 && size === 5) return fields([nextFlag(a(1), 1, 4, a(4)), nextFlag(a(2), 2, 5, a(4)), nextFlag(a(3), 3, 6, a(4))].map(decimal));
  if (op === 77 && size === 3) return levelLength(a(1)) === 0 ? INVALID : fields([decimal(levelLength(a(1))), sequenceText(a(2), a(1))]);
  if (op === 75 && size === 5) return a(3) < levelLength(a(1)) ? verdictName(verdictCode(a(1), a(3), tileAt(a(2), a(3)), a(4))) : INVALID;
  if (op === 80 && size === 2) {
    const [prime, category, text] = PROMPTS[a(1) % 4];
    return fields([decimal(prime), category, text]);
  }
  if (op === 71 && size === 3) {
    const i = guideIndex(a(1), a(2));
    return fields([decimal(i), GUIDE[i]]);
  }
  if (op === 87 && size === 2) return COPY[Math.min(a(1), 5)];
  if (op === 81 && size === 6) {
    return quorumWellFormed(a(1), a(3), a(4), a(5))
      ? fields([decimal(overlap(a(3), a(4), a(5))), quorumHolds(a(2), a(3), a(4), a(5)) ? "holds" : "violated"])
      : INVALID;
  }
  if (op === 70 && size === 4) {
    const [cap, read, write] = [a(1), a(2), a(3)];
    return read <= write && sub(write, read) <= cap
      ? fields([decimal(read), decimal(ringWrite(cap, read, write)), decimal(sub(ringWrite(cap, read, write), read))])
      : INVALID;
  }
  if (op === 83 && size === 3) return shouldErase(a(1), a(2)) ? "erase" : "keep";
  if (op === 78 && size === 1) return fields([decimal(front.rhythms.length), front.names.join(",")]);
  return INVALID;
}

// The requests whose answers the holo carries as its acceptance vectors: every
// mechanism, its edges, and the refusals. Frontier rhythms add one vector each.
export function requests(front = frontier()) {
  const R = (...xs) => Uint8Array.from(xs);
  const s = (c) => c.charCodeAt(0);
  const list = [
    R(s("D"), 50, 80), R(s("D"), 75, 200), R(s("D"), 10, 5), R(s("D"), 100, 255), R(s("D"), 25, 20),
    R(s("R"), 90), R(s("R"), 20),
    R(s("B"), 0, 0), R(s("B"), 0, 3), R(s("B"), 0, 4), R(s("B"), 0, 11), R(s("B"), 0, 18), R(s("B"), 0, 19), R(s("B"), 0, 200),
    R(s("C"), 0, 0, 0, 0), R(s("C"), 0, 0, 0, 1), R(s("C"), 1, 1, 1, 5), R(s("C"), 1, 1, 1, 7), R(s("C"), 0, 1, 0, 3),
    R(s("M"), 1, 7), R(s("M"), 7, 42), R(s("M"), 0, 1), R(s("M"), 4, 200),
    R(s("K"), 1, 7, 0, tileAt(7, 0)), R(s("K"), 1, 7, 0, (tileAt(7, 0) + 1) % 4), R(s("K"), 1, 7, 2, tileAt(7, 2)),
    R(s("K"), 7, 42, 22, tileAt(42, 22)), R(s("K"), 2, 9, 5, 0),
    R(s("P"), 0), R(s("P"), 1), R(s("P"), 2), R(s("P"), 3), R(s("P"), 9),
    R(s("G"), 10, 0), R(s("G"), 50, 1), R(s("G"), 75, 2), R(s("G"), 95, 0),
    R(s("W"), 0), R(s("W"), 4), R(s("W"), 5),
    R(s("Q"), 4, 1, 3, 3, 4), R(s("Q"), 10, 1, 3, 3, 6), R(s("Q"), 7, 2, 5, 5, 7), R(s("Q"), 4, 1, 3, 3, 9),
    R(s("F"), 8, 0, 0), R(s("F"), 8, 2, 7), R(s("F"), 8, 0, 8), R(s("F"), 8, 5, 3),
    R(s("S"), 0, 0), R(s("S"), 1, 23), R(s("S"), 1, 24), R(s("S"), 2, 200), R(s("S"), 3, 255),
    R(s("N")),
    R(), R(s("Z")), R(s("D"), 1),
  ];
  front.rhythms.forEach((_, i) => list.push(R(s("B"), i + 1, 0), R(s("B"), i + 1, 101)));
  list.push(new Uint8Array(64)); // the exact guest-allocation boundary
  return list;
}

const hex = (bytes) => Buffer.from(bytes).toString("hex");

export function vectorsTerm(front = frontier()) {
  let out = "(nil Foundation.View.Text.V1.Model::AcceptanceVector)";
  for (const request of requests(front).reverse()) {
    const response = Buffer.from(dispatch(request, front), "utf8");
    out = `(cons (record Foundation.View.Text.V1.Model::AcceptanceVector {request := 0x${hex(request)}, response := 0x${hex(response)}}) ${out})`;
  }
  return out;
}

export function writeVectors() {
  const model = readFileSync(MODEL, "utf8");
  const start = model.indexOf("acceptanceVectors := ") + "acceptanceVectors := ".length;
  const end = model.indexOf(", entryRoot :=");
  if (start < 21 || end < 0) throw new Error("App.lex.tex has no acceptanceVectors field");
  const next = model.slice(0, start) + vectorsTerm() + model.slice(end);
  if (next !== model) writeFileSync(MODEL, next);
  return next !== model;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const arg = process.argv[2];
  if (arg === "--write") console.log(writeVectors() ? "acceptance vectors rewritten" : "acceptance vectors current");
  else if (arg === "--check") {
    const model = readFileSync(MODEL, "utf8");
    if (!model.includes(`acceptanceVectors := ${vectorsTerm()}, entryRoot :=`)) {
      console.error("acceptance vectors are stale: run node scripts/reference.mjs --write");
      process.exit(1);
    }
    console.log("acceptance vectors current");
  } else {
    for (const r of requests()) console.log(hex(r).padEnd(14), JSON.stringify(dispatch(r)));
  }
}
