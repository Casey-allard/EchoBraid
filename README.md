<p align="center">
  <a href="https://casey-allard.github.io/EchoBraid/"><b>App</b></a>
  ·
  <a href="frontier.toml">Engine frontier</a>
  ·
  <a href="docs/DESIGN.md">Design</a>
  ·
  <a href="https://uor.foundation/docs">Prism docs</a>
  ·
  <a href="https://github.com/afflom/me">afflom/me</a>
  ·
  <a href="AGENTS.md">AGENTS.md</a>
</p>

<p align="center">
  <a href="https://github.com/Casey-allard/EchoBraid/actions/workflows/bootstrap.yml"><img src="https://github.com/Casey-allard/EchoBraid/actions/workflows/bootstrap.yml/badge.svg?branch=main" alt="bootstrap"></a>
  <a href="https://github.com/Casey-allard/EchoBraid/actions/workflows/pages.yml"><img src="https://github.com/Casey-allard/EchoBraid/actions/workflows/pages.yml/badge.svg?branch=main" alt="pages"></a>
  <a href="https://github.com/Casey-allard/EchoBraid/actions/workflows/engine.yml"><img src="https://github.com/Casey-allard/EchoBraid/actions/workflows/engine.yml/badge.svg?branch=main" alt="engine"></a>
  <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License: MIT OR Apache-2.0">
</p>

# EchoBraid

A neurodivergent-affirming, ASD-aligned learning companion that keeps your
pace. A soft writing loop, guided breathing, a memory game, prime-numbered
reflection prompts and a formal-logic workbench. Every decision is made by a
WebAssembly guest whose rules are proved in Lean 4, and an autonomous engine
discovers, proves and ships new breathing rhythms one tick at a time.

## What is published

- **The app**, at <https://casey-allard.github.io/EchoBraid/>: a Prism app page
  that loads `app.holo`, checks its fingerprint and replays its examples in
  your browser before anything runs, then opens the EchoBraid surface. It is
  listed in the Prism index as `holo://echobraid`, so it appears in the
  address bar of every Prism app, [afflom/me](https://github.com/afflom/me)'s
  included.
- **The holo**: one content-addressed archive holding the LexLean model, its
  Lean proofs, the compiled guest, its acceptance vectors and its signed build
  provenance. `npx prism-code inspect app.holo` reads it without running it.
- **A release per engine tick**: `v0.2.<tick>`, with the tick's ranking table.

## How it is made

1. **Modelled in LexLean.** `app/src` holds the model: one module per
   mechanism and an `App` module whose `dispatch` routes a one-byte opcode to
   them. 33 theorems say what each may answer: 5 the kernel decides inside the
   holo, and 28 arithmetic theorems in `proofs/Laws`, a root LexLean project
   over the same `app/src` files.
2. **Proved in Lean 4.** `lexlean verify` (in `app/` and at the root) checks
   every theorem with Lean and leanchecker under an exact axiom policy.
3. **Built by PrismPM.** `prismpm --project app verify`, in the SDK image
   `prismpm.lock` pins, compiles the guest, replays every acceptance vector in
   it, checks the proofs again and writes the holo. `pages.yml` signs its
   provenance and publishes the page.
4. **Improved by the engine.** `engine.yml` runs a tick: Rule 30 proposes
   rhythms, Hessian sorting ranks them, the best safe and novel one is written
   into `app/src/Braid/Frontier.lex.tex`, and only if the Lean kernel proves
   `frontierSafe` is it committed, released and redeployed.

## The model

| Module | Mechanism | Theorems (`proofs/Laws/<Module>Laws` unless decided in the holo) |
| --- | --- | --- |
| `Braid.Resonance` | Semantic drift, coherence, the pause offer, rest | `driftStepBounded`, `restStepBounded`, `restNeverRaisesDrift`, `pauseOnlyWhenDivergent`, `divergentDecides`, `coherenceMonotone` |
| `Braid.Breath` | Breathing rhythms, the 4-7-8 cycle, `rhythmSafe` | `countPositive`, `countWithinPhase`, `phaseBounded`, `relaxCycleIsNineteen`, `relaxIsSafe` |
| `Braid.Frontier` | Rhythms the engine proved (written by the engine) | `frontierSafe` |
| `Braid.Consent` | Microphone, voice and reminder consent | `noSilentGrant`, `revokeAllClears`, `flagIsBinary`, `revokeWins` |
| `Braid.Memory` | Prime-length memory game on a Rule 30 ring | `tileBounded`, `levelLengthBounded`, `victoryOnlyAtFinalLevel`, `wrongTileEndsRound`, `seedNeverDegenerate` |
| `Braid.Reflection` | Prime prompts, the original copy, guide replies | `pauseComesFirst`, `guideIndexBounded`, `promptPrimeBounded`, `promptPrimeNotFour`, `promptPrimeNotSix` |
| `Braid.Logic` | Quorum intersection, ring buffer | `quorumIntersection`, `byzantineQuorum`, `ringStaysOrdered`, `ringNeverOverflows` |
| `Braid.Retention` | How long a reflection is kept | `dayPolicyErases`, `immediatePolicyAlwaysErases`, `keepPolicyNeverErases` |
| `App` | The byte protocol, the application record and its acceptance vectors | |

The protocol, one opcode letter then argument bytes: `D` drift, `R` rest,
`B` breath, `C` consent, `M` memory sequence, `K` memory move, `P` prompt,
`G` guide, `W` copy, `Q` quorum, `F` ring buffer, `S` retention, `N`
frontier. The claims are registered in `model/ids.toml` (`EB-01` to `EB-13`)
and listed in [CONFORMANCE.md](CONFORMANCE.md).

## The engine

The engine follows [lexlean-r4](https://github.com/Casey-allard/lexlean-r4)'s
autonomous engine. Each tick:

| Step | What happens | Where |
| --- | --- | --- |
| Abduction | A 64-cell Rule 30 ring proposes 12 rhythms through three strategies (direct, mutate the best, crossover) | `crates/engine` `Driver`, `Strategy` |
| Hessian sorting | Each candidate is ranked by its calm score minus the Frobenius norm of the score's Hessian, so a rhythm that stays calm when timing slips wins | `hessian_sort` |
| Admission | The best safe, novel rhythm above the floor | `plan` |
| Deduction | Written into `Braid.Frontier`; the vectors are regenerated; `lexlean verify` must decide `frontierSafe` | `xtask engine tick --write --prove` |
| Learning | Hebbian strategy weights move by the outcome, within 1 to 16 | `Weights::learn` |
| Release | Commit to main, tag `v0.2.<tick>`, redeploy the page, chain the next tick | `.github/workflows/engine.yml` |

`frontier.toml` lists every proved rhythm and `model/engine.toml` holds the
ring, the weights and every attempt. Set the repository variable
`ECHOBRAID_ENGINE` to `off` to stop it.

## Built on

| Part | What EchoBraid takes from it |
| --- | --- |
| [UOR-Foundation/template](https://github.com/UOR-Foundation/template) | The repository standards, the gate, the SDK lock, the page and the workflows |
| [LexLean](https://github.com/UOR-Foundation/LexLean) | The modelling language and the verifier |
| [PrismPM](https://github.com/UOR-Foundation/PrismPM) | The holo build, guest and replay |
| [afflom/stitch](https://github.com/afflom/stitch) | The Prism ID (`identity/`, pinned by `identity/stitch.lock`), used to seal reflections |
| [afflom/prism-devrel](https://github.com/afflom/prism-devrel) | This README's structure, `model/charter.md` and `model/experience.toml` |
| [afflom/UORust](https://github.com/afflom/UORust) | `uorust/`: the Rust workspace projected into a verified Prism project |
| [afflom/me](https://github.com/afflom/me) | Discovery through the shared Prism index |

## Everyday commands

| Command | What it does |
| --- | --- |
| `just vv` | The whole gate, in the SDK image |
| `cd app && lexlean verify` | The holo's model and its kernel-decided theorems |
| `lexlean verify` | The 28 arithmetic theorems in `proofs/Laws` |
| `prismpm --project app verify` | Build the holo, replay the vectors, check the proofs |
| `node scripts/reference.mjs --write` | Regenerate the acceptance vectors after a model change |
| `cargo xtask engine tick --write --prove` | One engine tick, locally |
| `cargo xtask engine status` | The frontier and the weights |
| `scripts/uorust.sh` | Re-project the Rust workspace with UORust |
| `just model-write` | Regenerate `CONFORMANCE.md` |
| `scripts/sdk.sh '<command>'` | Run any of these in the pinned SDK image |

Rules for agents and contributors are in [AGENTS.md](AGENTS.md); the
gate evidence and the planted defects are in
[VERIFICATION-ECHOBRAID.md](VERIFICATION-ECHOBRAID.md).

Built from [UOR-Foundation/template](https://github.com/UOR-Foundation/template).
Licensed under MIT or Apache-2.0, at your option.
