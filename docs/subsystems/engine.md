---
title: The self-improvement engine
type: subsystem
created: 2026-11-17
updated: 2026-11-17
sources:
  - code: crates/engine/src/lib.rs
  - code: xtask/src/engine.rs
  - code: .github/workflows/engine.yml
  - code: frontier.toml
  - code: model/engine.toml
  - code: crates/conformance/tests/engine.rs
verified_at: 921760a
---

# The self-improvement engine

One run of `.github/workflows/engine.yml` is one engine tick, after Casey-allard/lexlean-r4's
engine. The decisions live in `crates/engine/src/lib.rs` and are pure; `xtask engine tick`
(`xtask/src/engine.rs`) does the file work. A tick: a 64-cell Rule 30 ring proposes 12
breathing rhythms through three strategies scheduled by Hebbian weight; Hessian sorting
ranks them by calm score minus the Frobenius norm of the score's Hessian; the best safe,
novel one is written into `app/src/Braid/Frontier.lex.tex`; the vectors are regenerated;
and `lexlean verify` must decide `frontierSafe` before anything is kept. A proved tick is
committed to main, tagged `v0.2.<tick>`, released, and `.github/workflows/pages.yml` redeploys. How the
model is built is on [[holo-build]].

State: `frontier.toml` (every proved rhythm), `model/engine.toml` (ring, generation,
weights, every attempt). `cargo xtask engine render` regenerates the frontier module
from `frontier.toml`; `engine status` prints both. The repository variable
`ECHOBRAID_ENGINE=off` stops the workflow.

> ⚠️ lexlean-r4 has no Hessian sorting: its engine ranks strategies by bounded Hebbian
> weights. EchoBraid keeps those and adds the Hessian ranking. Do not "restore parity" by
> removing either.

> ⚠️ `Rhythm::safe` mirrors the model's `rhythmSafe` clause for clause; `engine_eb_13`
> pins the clauses. Change both or neither. The engine's filter is a convenience; the
> kernel's `frontierSafe` is the guarantee (a planted 9-9-9-9 rhythm is refused by
> `decide`).

> ⚠️ `frontier_module` must emit LexLean's canonical formatting, axioms included
> (`frontierNamesFrom` carries `propext`), or `lexlean fmt --check` fails the tick. Check
> with `engine render` followed by `lexlean fmt --check`.

> ⚠️ The tick pushes with `GITHUB_TOKEN`, which does not trigger workflows, so the
> workflow dispatches `.github/workflows/pages.yml` and the next tick itself. Checkout must keep
> `persist-credentials: false` (the bootstrap audit refuses otherwise); the push builds
> its own auth header.
