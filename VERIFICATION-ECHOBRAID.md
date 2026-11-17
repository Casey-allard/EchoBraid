# EchoBraid verification record

`VERIFICATION.md` is a universal policy file, byte-bound by `template.lock`;
this file is EchoBraid's own evidence, as `AGENTS.md` asks ("put generated or
product-specific verification evidence in project-owned files").

## What each gate checks

| Gate | Command | What it refuses |
| --- | --- | --- |
| Lean kernel, holo | `cd app && lexlean verify` (also inside `prismpm --project app verify`) | The holo's kernel-decided theorems (`frontierSafe` over the concrete frontier, 4-7-8 safety and cycle, the fixed retention policies), and any declaration whose axioms differ from what Lean observes |
| Lean kernel, proofs | `lexlean verify` at the root (`just proofs`) | Any of the 28 arithmetic theorems in `proofs/Laws` that does not hold over the `app/src` definitions it imports |
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

## Why two LexLean projects

A text application's `prismpm verify` compiles the generated Lean natively,
LexLean runtime included, within a fixed 600-second limit. A `linear_arithmetic`
proof makes the generated Lean import the runtime's arithmetic laws
(`LexLeanRuntime/Laws.lean`, 684 KB), which alone exceeded that limit both in
the sandbox this repository was written in and on a GitHub runner
(`PP5007: application-lean timed out`, pages run 44000420820). So the holo
(`app/`) carries the definitions, the dispatch, the acceptance vectors and the
theorems the kernel decides, and the root project proves the 28 arithmetic
theorems over the same `app/src` files with `lexlean verify`, which uses the
SDK's prebuilt runtime. Both run in `just vv`; `echobraid_eb_*` pins each
theorem's statement and that its file is an entrypoint of the project that
verifies it.

Local evidence for the split: `prismpm --project app verify` printed
`verified attestation: 36cca3a752acc0e2ec63a90dae12eaac0a06464994726e2de6d16f00fed5039a`,
the holo is 32 717 bytes, and `prism-code vectors` replayed 60 of 60 vectors
with identities verified. The root project printed `verified 15 modules`.
