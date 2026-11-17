// EchoBraid's surface. Every decision on this page is the verified guest's:
// this script only encodes a request (an opcode byte and its argument bytes),
// hands it to `app.dispatch`, and shows the tab-separated answer. The opcodes
// are those of `dispatch` in app/src/App.lex.tex; the Lean kernel proved what
// each one may answer.
const decoder = new TextDecoder();
const store = {
  get(key, fallback) { try { return localStorage.getItem(key) ?? fallback; } catch { return fallback; } },
  set(key, value) { try { localStorage.setItem(key, value); } catch { /* private window: this page only */ } },
  drop(key) { try { localStorage.removeItem(key); } catch { /* nothing kept */ } },
};
const byte = (n) => Math.max(0, Math.min(255, Math.trunc(Number(n) || 0)));
const op = (c) => c.charCodeAt(0);
const el = (tag, text, className) => Object.assign(document.createElement(tag), { textContent: text, className: className ?? "" });

export default async function start({ app, reason, $ }) {
  const out = $("eb-out");
  const history = $("eb-history");
  if (!app) {
    out.textContent = `The guest did not load (${reason || "unknown"}); nothing here decides without it.`;
    out.className = "refuse";
    return;
  }

  async function ask(letter, ...args) {
    const reply = decoder.decode(await app.dispatch(Uint8Array.from([op(letter), ...args.map(byte)])));
    return reply === "invalid request" ? null : reply.split("\t");
  }
  function note(what, reply) {
    out.textContent = reply;
    out.className = "good";
    const row = el("li", "");
    row.append(el("span", what, "k"), el("span", reply, "v"));
    history.prepend(row);
    while (history.children.length > 30) history.lastChild.remove();
  }

  // Places: nonlinear, any order, no progress kept against the person.
  const places = [...document.querySelectorAll(".eb-place")];
  for (const go of document.querySelectorAll(".eb-go")) {
    go.addEventListener("click", () => {
      for (const b of document.querySelectorAll(".eb-go")) b.removeAttribute("aria-current");
      go.setAttribute("aria-current", "page");
      for (const p of places) p.hidden = p.dataset.place !== go.dataset.place;
      if (go.dataset.place !== "breathe") breath.stop();
    });
  }
  const show = (name) => document.querySelector(`.eb-go[data-place="${name}"]`)?.click();

  // Care: consent flags move only by the guest's nextFlag (EB-05).
  const flags = { mic: +store.get("eb.mic", "0"), voice: +store.get("eb.voice", "0"), notify: +store.get("eb.notify", "0") };
  const boxes = { mic: $("eb-mic"), voice: $("eb-voice"), notify: $("eb-notify") };
  const codes = { mic: [1, 4], voice: [2, 5], notify: [3, 6] };
  async function consent(action, what) {
    const r = await ask("C", flags.mic, flags.voice, flags.notify, action);
    [flags.mic, flags.voice, flags.notify] = r.map(Number);
    for (const k of Object.keys(flags)) { boxes[k].checked = flags[k] === 1; store.set(`eb.${k}`, String(flags[k])); }
    $("eb-dictate").hidden = !(flags.mic && recognition);
    note(`consent ${what}`, `listen ${r[0]}, read aloud ${r[1]}, reminders ${r[2]}`);
  }
  for (const k of Object.keys(boxes)) {
    boxes[k].checked = flags[k] === 1;
    boxes[k].addEventListener("change", () => consent(boxes[k].checked ? codes[k][0] : codes[k][1], `${boxes[k].checked ? "grant" : "revoke"} ${k}`));
  }
  $("eb-revoke").addEventListener("click", () => consent(7, "revoke all"));
  $("eb-consent-copy").textContent = (await ask("W", 5))[0];

  // Retention: the guest decides whether a kept reflection is erased (EB-09).
  const retention = $("eb-retention");
  retention.value = store.get("eb.retention", "1");
  async function enforce() {
    const savedAt = +store.get("eb.journal.at", "0");
    if (!savedAt) return;
    const hours = Math.min(255, Math.floor((Date.now() - savedAt) / 3_600_000));
    const [verdict] = await ask("S", +retention.value, hours);
    if (verdict === "erase") { store.drop("eb.journal"); store.drop("eb.journal.at"); journal.value = ""; note("retention", `erased after ${hours} h`); }
  }
  retention.addEventListener("change", () => { store.set("eb.retention", retention.value); enforce(); });
  const dyslexic = $("eb-dyslexic");
  dyslexic.checked = store.get("eb.dyslexic", "0") === "1";
  const applyFont = () => document.documentElement.classList.toggle("eb-dyslexic", dyslexic.checked);
  dyslexic.addEventListener("change", () => { store.set("eb.dyslexic", dyslexic.checked ? "1" : "0"); applyFont(); });
  applyFont();

  // Soft loop: drift, coherence and the pause offer are the guest's (EB-01..03).
  let drift = 10;
  let turns = 0;
  const log = $("eb-log");
  const say = (who, text) => { log.append(el("li", text, `eb-${who}`)); log.lastChild.scrollIntoView({ block: "nearest" }); };
  function meter(r) {
    drift = +r[0];
    $("eb-drift-bar").style.inlineSize = `${drift}%`;
    $("eb-drift").textContent = `Drift ${r[0]}, ${r[1]}`;
    $("eb-meter").setAttribute("aria-valuenow", r[0]);
    $("eb-meter").setAttribute("aria-valuetext", `Drift ${r[0]}, ${r[1]}`);
  }
  const speak = (text) => { if (flags.voice && "speechSynthesis" in window) speechSynthesis.speak(new SpeechSynthesisUtterance(text)); };
  $("eb-say").addEventListener("submit", async (event) => {
    event.preventDefault();
    const text = $("eb-text").value.trim();
    if (!text) return;
    $("eb-text").value = "";
    recognition?.stop();
    say("you", text);
    const r = await ask("D", drift, Math.min(255, text.length));
    meter(r);
    note("drift", r.join(" "));
    const [, reply] = await ask("G", drift, turns++);
    if (r[2] === "pause") {
      say("guide", reply);
      setTimeout(() => { show("breathe"); breath.begin(); }, 1500);
    } else {
      say("guide", reply);
      speak(reply);
    }
  });
  $("eb-pause").addEventListener("click", () => { show("breathe"); breath.begin(); });
  meter(["10", "stable"]);

  // Dictation runs only while the guest says listening was granted.
  const Recognition = window.SpeechRecognition || window.webkitSpeechRecognition;
  const recognition = Recognition ? new Recognition() : null;
  if (recognition) {
    recognition.continuous = true;
    recognition.interimResults = false;
    recognition.addEventListener("result", (event) => {
      for (const r of event.results) if (r.isFinal) $("eb-text").value += `${r[0].transcript} `;
    });
    $("eb-dictate").addEventListener("click", () => { if (flags.mic) recognition.start(); });
  }
  $("eb-dictate").hidden = !(flags.mic && recognition);

  // Breathe: phase and countdown at every tick are the guest's (EB-04, EB-10).
  const rhythm = $("eb-rhythm");
  const [count, names] = await ask("N");
  names.split(",").filter(Boolean).forEach((name, i) => rhythm.append(Object.assign(el("option", name), { value: String(i + 1) })));
  note("frontier", `${count} engine rhythm${count === "1" ? "" : "s"}`);
  const breath = {
    timer: 0,
    tick: 0,
    async step() {
      const [phase, left, length] = await ask("B", +rhythm.value, this.tick % 256);
      this.tick += 1;
      $("eb-phase").textContent = `${phase[0].toUpperCase()}${phase.slice(1)}`;
      $("eb-count").textContent = left;
      const orb = $("eb-orb");
      orb.dataset.phase = phase;
      orb.style.setProperty("--eb-phase-s", `${length}s`);
    },
    begin() { this.stop(); this.tick = 0; this.step(); this.timer = setInterval(() => this.step(), 1000); },
    stop() { clearInterval(this.timer); this.timer = 0; },
  };
  $("eb-breathe-go").addEventListener("click", () => breath.begin());
  $("eb-breathe-stop").addEventListener("click", async () => {
    breath.stop();
    $("eb-phase").textContent = "Ready";
    const r = await ask("R", drift);
    meter(r);
    say("guide", (await ask("W", 3))[0]);
    note("rest", r.join(" "));
    show("loop");
  });
  rhythm.addEventListener("change", () => { if (breath.timer) breath.begin(); });

  // Memory: the Rule 30 sequence and every verdict are the guest's (EB-06).
  const game = { level: 1, seed: 0, index: 0, busy: false, on: false };
  const tiles = [...document.querySelectorAll(".eb-tile")];
  const wait = (ms) => new Promise((done) => setTimeout(done, ms));
  async function playLevel() {
    game.busy = true;
    game.index = 0;
    const [length, sequence] = await ask("M", game.level, game.seed);
    $("eb-memory-state").textContent = `Level ${game.level} of 7, ${length} tiles`;
    await wait(800);
    for (const t of sequence) {
      tiles[+t].classList.add("lit");
      await wait(600);
      tiles[+t].classList.remove("lit");
      await wait(200);
    }
    game.busy = false;
  }
  $("eb-memory-go").addEventListener("click", () => {
    game.level = 1;
    game.seed = crypto.getRandomValues(new Uint8Array(1))[0];
    game.on = true;
    playLevel();
  });
  for (const tile of tiles) {
    tile.addEventListener("click", async () => {
      if (!game.on || game.busy) return;
      tile.classList.add("lit");
      setTimeout(() => tile.classList.remove("lit"), 300);
      const [verdict] = await ask("K", game.level, game.seed, game.index, +tile.dataset.tile);
      note("memory", verdict);
      if (verdict === "correct") game.index += 1;
      else if (verdict === "level") { game.level += 1; setTimeout(playLevel, 1000); }
      else if (verdict === "victory") { game.on = false; $("eb-memory-state").textContent = "All seven levels. Rest whenever you like."; }
      else { game.on = false; $("eb-memory-state").textContent = "That one slipped. Start again whenever you like."; }
    });
  }

  // Reflect: prompts are the guest's; the reflection stays on this device.
  const journal = $("eb-journal");
  journal.value = store.get("eb.journal", "");
  for (const b of document.querySelectorAll("[data-prompt]")) {
    b.addEventListener("click", async () => {
      const [prime, category, text] = await ask("P", +b.dataset.prompt);
      $("eb-prompt").textContent = `${category} (${prime}): ${text}`;
      journal.disabled = false;
      journal.focus();
    });
  }
  journal.addEventListener("input", () => { store.set("eb.journal", journal.value); store.set("eb.journal.at", String(Date.now())); });
  $("eb-burn").addEventListener("click", () => {
    if (!confirm("Clear this reflection? The words will fade, leaving space for new ones.")) return;
    journal.value = "";
    store.drop("eb.journal");
    store.drop("eb.journal.at");
  });
  // Sealing signs the reflection's SHA-256 with the visitor's Prism ID, the
  // identity afflom/stitch provides through the frame. The words never leave.
  $("eb-seal").addEventListener("click", async () => {
    const frame = document.querySelector("prism-frame");
    try {
      const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(journal.value));
      const { signature } = await frame.sign(new Uint8Array(digest));
      const hex = [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
      $("eb-sealed").textContent = `Sealed: sha256 ${hex.slice(0, 16)}… signature ${signature.slice(0, 16)}…`;
      note("seal", `reflection ${hex.slice(0, 12)} signed by this Prism ID`);
    } catch (error) {
      $("eb-sealed").textContent = `Not sealed: ${error?.message ?? "the Prism ID is not ready"}`;
    }
  });
  await enforce();

  // Logic: the guest answers the quorum and the ring buffer (EB-08).
  const quorum = async () => {
    const r = await ask("Q", $("eb-n").value, $("eb-f").value, $("eb-q1").value, $("eb-q2").value, $("eb-u").value);
    $("eb-quorum").textContent = r ? `Overlap ${r[0]}: the intersection property ${r[1]}` : "Not a possible pair of quorums";
    if (r) note("quorum", r.join(" "));
  };
  for (const id of ["eb-n", "eb-f", "eb-q1", "eb-q2", "eb-u"]) $(id).addEventListener("input", quorum);
  quorum();
  const ring = { cap: 8, read: 0, write: 0 };
  const drawRing = () => { $("eb-ring").textContent = `Capacity ${ring.cap}, read ${ring.read}, write ${ring.write}, holding ${ring.write - ring.read}`; };
  $("eb-enqueue").addEventListener("click", async () => {
    if (ring.write > 240) { ring.read = 0; ring.write = 0; }
    const r = await ask("F", ring.cap, ring.read, ring.write);
    [ring.read, ring.write] = [+r[0], +r[1]];
    drawRing();
    note("ring", r.join(" "));
  });
  $("eb-dequeue").addEventListener("click", () => { if (ring.read < ring.write) ring.read += 1; drawRing(); });
  drawRing();

  $("eb-welcome").textContent = (await ask("W", 0))[0];
  out.textContent = "Ready. Every answer here comes from the verified guest.";
  out.className = "good";
}
