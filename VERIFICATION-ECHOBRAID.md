# EchoBraid verification record

`VERIFICATION.md` is a universal policy file, byte-bound by `template.lock`;
this file is EchoBraid's own evidence, as `AGENTS.md` asks ("put generated or
product-specific verification evidence in project-owned files").

## What each gate checks

| Gate | Command | What it refuses |
| --- | --- | --- |
| Lean kernel | `cd app && lexlean verify` (also inside `prismpm --project app verify`) | Any of the 33 theorems in `app/src` that does not hold, any declaration whose axioms differ from what Lean observes, and an engine frontier holding an unsafe rhythm (`frontierSafe` is decided over the concrete list) |
| Guest replay | `prismpm --project app verify` | Any acceptance vector whose response the compiled WebAssembly guest does not reproduce byte for byte; the vectors are written by `scripts/reference.mjs` |
| Vectors current | `node scripts/reference.mjs --check` | A model edit that left the acceptance vectors stale |
| Claim pins | `cargo test -p repo-conformance --test echobraid --test engine` | A weakened theorem statement, an extra axiom, a `dispatch` that stops deciding through the proved definition, a directive guide text, an engine whose safety filter drifts from `rhythmSafe`, a Rule 30 step that is not rule 30, a ranking that is not Hessian sorting, a tick admitting more than one or an unsafe rhythm, a Hebbian weight outside 1 to 16 |
| Meta-gate | `cargo test -p repo-conformance --test bdd` | A registered ID without a scenario or test, or the reverse |

## Planted defects

Each defect was planted, the gate run, the diagnostic recorded, and the file
restored; the restored tree then passed the same gate.

| Defect | Gate | Diagnostic |
| --- | --- | --- |
| `driftStepBounded` weakened to `<= 101` | `echobraid_eb_01` | `driftStepBounded in src/Braid/Resonance.lex.tex no longer states ((driftStep drift length) <= 100)` |
| `rhythmSafe` loosened to an exhale of up to 12 | `engine_eb_13` | `rhythmSafe no longer has ((project rhythm exhale) <=? 10)` |
| `driftStep` stops clamping a long message (`drift + 10`) | `lexlean verify` | `Lean rejected EchoBraid.Braid.Resonance (error): omega could not prove the goal` |
| An unsafe rhythm 9-9-9-9 written into the frontier | `lexlean verify` | ``Lean rejected EchoBraid.Braid.Frontier (error): Tactic `decide` proved that the proposition Breath.allSafe frontierRhythms = true is false`` |

Restored: `verified 12 modules; attestation 544e9b149e8d0b6b4cc65f5672dfb6bc622f32c69d03990b84afb02989f844a1`.

## A finding from the source material

The uploaded AxiomScan sample claimed that with `n ≥ 3f + 1` members, two
quorums of `2f + 1` always share `f + 1`. That holds only at `n = 3f + 1`. The
acceptance vector `Q 10 1 3 3 6` is a counterexample (two quorums of 3 in a
group of 10, overlap 0). The model proves the correct statement instead,
`quorumIntersection` (sizes summing past `n + f` force an overlap of at least
`f + 1`), with the classic case as its corollary `byzantineQuorum`. The Logic
place on the page lets a learner find the counterexample themselves.

## Where the full build runs

`lexlean verify` checks every theorem locally in under a minute, because the
SDK image carries the LexLean runtime prebuilt. `prismpm verify` also compiles
that runtime natively from source within a fixed 600-second limit; on the
4-core sandbox this repository was written in, the runtime's `Laws` module
alone took longer than that, so the full holo build, its guest replay and its
signed provenance are produced on GitHub's runners by `pages.yml` and
`build.yml`, as for every other Prism app with arithmetic proofs (afflom/me,
afflom/stitch, afflom/UORust, afflom/prism-devrel).
