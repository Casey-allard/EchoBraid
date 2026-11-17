---
title: EchoBraid design
type: decision
created: 2026-11-17
updated: 2026-11-17
sources:
  - code: app/src/App.lex.tex
  - code: crates/engine/src/lib.rs
  - code: identity/stitch.lock
  - code: bootstrap/register.mjs
  - code: scripts/uorust.sh
verified_at: 921760a
---

# EchoBraid design

How the uploaded EchoBraid concept (an AI Studio React app, its inner `app/`
and the nested Universal Logic app) became a Prism application: modelled in
LexLean, proved in Lean 4, built by PrismPM into a holo, served on GitHub
Pages, and improved by an autonomous engine.

## Principles carried over

The concept's own words set the rules: "We can just sit. Your pace is law
here." "Never be directive." "Do not offer solutions, just resonance."
"Prioritizes silence, consent, and nonlinear navigation." Each became a
proved property or a structural choice:

| Concept rule | In EchoBraid |
| --- | --- |
| Pace is law, nonlinear navigation | Six places in any order; no progress is scored or kept against the learner |
| Never directive, gentle, under 50 words | Ten fixed guide texts (`guideText`); `EB-07` pins that each is short and contains no instruction |
| If distressed, invite a pause | `offersPause` (drift > 80) forces the pause offer as the guide's only answer (`pauseComesFirst`, `EB-02`) |
| Sensors are opt-in, can be turned off anytime | `nextFlag`: no flag turns on without its own grant, revoke-all clears every flag (`EB-05`) |
| "This session will vanish in 24 hours" (promised, never built) | `shouldErase`: the 24-hour policy erases once 24 hours have passed (`EB-09`), enforced on every load |

## Mechanisms

| Concept source | Mechanism | Model | Change from the concept |
| --- | --- | --- | --- |
| concept: app/services/resonanceEngine.ts, calculateDrift | Semantic drift, ±0.1 by message length | `Braid.Resonance.driftStep` (percent, ±10) | The random ±0.05 noise is gone: a proved guest is deterministic |
| `getCoherenceState` (never used by the concept) | Stable / drifting / divergent at 0.3 / 0.7 | `coherenceRank` | Now drives the reply and the meter; proved monotone (`EB-03`) |
| SoftLoop pause at drift > 0.8 | Pause offer | `offersPause` | The concept's 0.8 pause and 0.7 divergence disagreed; the model proves pause ⇒ divergent |
| Closing the breath overlay, drift − 0.3 | Rest | `restStep` | Proved never to raise drift |
| `generateSystemResponse` (Gemini) | Guide reply | `guideIndex`, `guideText` | Deterministic reflection chosen by coherence and turn; no network, no model call |
| `generateSpeech` (Gemini TTS) | Read aloud | the browser's `speechSynthesis` | Only while the guest says voice was granted |
| `BreathOverlay` 4-7-8, 1 s ticks | Breath cycle | `Braid.Breath` (`phaseAt`, `countAt`) | Generalised to any rhythm; countdown proved in range (`EB-04`) |
| Home: Simon game, levels 3, 5, 7, 11, 13, 17, 23 | Memory | `Braid.Memory` | `Math.random` replaced by a 16-cell Rule 30 ring seeded per game |
| `PRIME_PROMPTS` 2, 3, 5, 7 | Reflect prompts | `Braid.Reflection.prompt*` | Proved prime-indexed |
| HoloScriptor / Journal "incinerate" | Clear | the page clears local storage | Plus sealing with the Prism ID (`frame.sign`) |
| AxiomScan `byzantine_consensus` | Quorum overlap | `Braid.Logic.quorumIntersection` | The sample claim is false for n > 3f + 1; the model proves the correct premise |
| AxiomScan / Analyze `lockless_ring_buffer` | Ring buffer | `ringWrite` | Proved never to overflow or reorder |
| `SessionConfig.dataRetention` (never enforced) | Retention | `Braid.Retention.shouldErase` | Enforced |
| Universal Logic `bringToLife` (Gemini, sketch to app) | (not carried) | | A generative call cannot be proved; EchoBraid's growth comes from the engine instead |

## Architecture decisions

1. **One holo, one byte protocol.** The guest exposes one `dispatch`: the
   first byte is an opcode letter, the rest are argument bytes, and the reply
   is tab-separated UTF-8. A text application (`prismpm/text-application/1`)
   carries it, its acceptance vectors and its proofs. The page's own surface
   (`site/card.*`) only encodes requests and shows replies.
2. **Hand-written LexLean, not a spec.json.** The guided prism/spec.json
   path covers apps that decide one answer from one line of fields. EchoBraid
   has thirteen mechanisms, so it is modelled directly in LexLean, as the
   template, afflom/me and lexlean-r4 are.
3. **The engine proposes; only the kernel admits.** The engine (Rust,
   `crates/engine`) is pure and deterministic. It can write candidates into
   `Braid.Frontier`, but a tick stands only if `lexlean verify` decides
   `frontierSafe` over the new list. lexlean-r4's engine ranks strategies with
   bounded Hebbian weights; EchoBraid keeps those and adds the Hessian sorting
   asked for: candidates are ranked by calm score minus the Frobenius norm of
   the score's Hessian, so robust rhythms win over sharp optima.
4. **Identity from Stitch, unchanged.** The template's `identity/prism-id.mjs`
   is generated from afflom/stitch and pinned by `identity/stitch.lock`; the
   frame (pinned by `frame.lock`) mounts it. EchoBraid uses it through
   `frame.sign` to seal reflections, and the frame's relay (ntfy) carries
   the Prism ID's device linking and sync.
5. **Discoverable from afflom/me without editing it.** Every Prism app's
   address bar, afflom/me's included, lists the apps in the Prism index at
   registry.uor.foundation. `.github/workflows/pages.yml` registers EchoBraid there on each
   deploy (`bootstrap/register.mjs`, GitHub's OIDC token, no secret), and the
   index lists it as `holo://echobraid` once its live holo verifies and its
   examples replay.
6. **UORust for the Rust.** The engine, the gates and the conformance suite
   are Rust. `scripts/uorust.sh` projects the workspace with afflom/UORust
   into `uorust/`, a Prism project the Lean kernel verifies; `.github/workflows/uorust.yml`
   keeps it current.
7. **Arithmetic proofs beside the holo, not in it.** PrismPM compiles a text
   application's Lean natively within a fixed time limit, and `linear_arithmetic`
   pulls the LexLean runtime's arithmetic laws into that compile, past the
   limit. The holo keeps what the kernel decides (the frontier, 4-7-8 safety);
   `proofs/Laws` proves the rest over the same source files. The frontier is
   written as indexed functions (`frontierAt`, `frontierName`), because the
   exporter renders no static list of records.
