---
title: Building the holo
type: subsystem
created: 2026-11-17
updated: 2026-11-17
sources:
  - code: app/lexlean.toml
  - code: lexlean.toml
  - code: app/src/App.lex.tex
  - code: app/src/Braid/Frontier.lex.tex
  - code: app/src/Braid/Memory.lex.tex
  - code: scripts/reference.mjs
  - code: scripts/sdk.sh
  - code: Justfile
  - code: xtask/src/audit.rs
  - code: .github/workflows/pages.yml
  - code: .github/workflows/try-page.yml
verified_at: 921760a
---

# Building the holo

The holo is built from `app/` by `prismpm --project app verify` (the `app` recipe,
`Justfile:62`) inside the SDK image `prismpm.lock` pins; `scripts/sdk.sh` runs any
command there the way CI does. EchoBraid is a **text application**: one `dispatch`
(`EchoBraid.App.dispatch`) from request bytes to a UTF-8 reply, plus acceptance vectors
the compiled WebAssembly guest must reproduce byte for byte. The page (`site/card.*`)
only encodes requests; see [[engine]] for how the model grows on its own.

## Two LexLean projects, one source

`app/lexlean.toml` is the holo: definitions, dispatch, vectors and the theorems the
kernel decides (`frontierSafe`, 4-7-8 safety, the fixed retention cases). The root
`lexlean.toml` is a second project whose source roots are `app/src` and `proofs`
(`lexlean.toml:5`); `proofs/Laws/*` prove the 28 arithmetic theorems over the very
`app/src` modules the guest runs. `just vv` runs both (`proofs` at `Justfile:108`).

> ⚠️ Never put a `linear_arithmetic` proof in `app/src`. It makes the generated Lean
> import the LexLean runtime's `Laws.lean` (684 KB), and PrismPM compiles a text
> application natively within a fixed 600 s (`PP5007: application-lean timed out`).
> That limit was missed in the sandbox and on a GitHub runner. `lexlean verify` uses a
> prebuilt runtime, which is why the root project can carry those proofs.

> ⚠️ LexLean refuses source roots outside a project's directory (`../app/src`), so the
> proofs project has to live at the repository root, not in `proofs/`.

## What PrismPM's exporter will not render

> ⚠️ No zero-argument definition may return a list of records ("static list bindings
> require closed allocation-free constants"). The frontier is generated as indexed
> functions (`frontierAt`, `frontierName`, `frontierSafeFrom`) for that reason; see
> `crates/engine/src/lib.rs:506`.

> ⚠️ A function that returns a list cannot feed another call ("returns a list; its
> result cannot be used as an intermediate value"). Build text by recursion over strings
> instead, as `tilesFrom` does (`app/src/Braid/Memory.lex.tex:80`). A list literal passed
> straight to `prim join`, or a list parameter, is fine.

> ⚠️ The vectors must include one request exactly `guestAllocationMaximum` bytes long,
> and `guestAllocationMaximum` must be at least `requestMaximum` (`PP2009`). EchoBraid
> uses 32 and 64 and a 64-byte zero request.

## Vectors come from an oracle

`scripts/reference.mjs` mirrors the model line for line and writes the vectors into
`App.lex.tex` (`writeVectors`, `scripts/reference.mjs:211`; the request list is at
`:175`). `prismpm verify` replays them in the guest, so a divergence between oracle and
model fails the build. `just vectors` (`Justfile:100`) fails when they are stale.

> ⚠️ After any model edit run `node scripts/reference.mjs --write`, then
> `npx prism-code axioms --write` (with `prismpm.lock` in the project directory):
> every declaration must state exactly the axioms Lean observes, and the verify reports
> only the first wrong one per run.

## Language surface traps

> ⚠️ `\title{}` text must be a lexicon phrase; EchoBraid's is in
> `app/language/echobraid.domain`. Parameterless definitions are called in parentheses,
> `(invalid)`. `*` is not notation (use `prim multiply` or addition), there is no Prop
> disjunction (state it as a Bool `||` equal to `true`), and `definitions:` lists must be
> sorted.

> ⚠️ `linear_arithmetic` splits `if` only in the goal. A theorem whose `if` sits in a
> hypothesis fails; state it in contrapositive form with the decision in the goal.

## Gates around the page

`.github/workflows/pages.yml` builds, signs and then runs Lighthouse (100 on mobile), `screens.mjs` (every
window size class and theme, no page scroll, 48 px touch targets) and `weight.mjs` (app
under 120 KiB) before it deploys. They can all run locally against a built holo by
copying it to `site/app.holo` and following the workflow's "Assemble the page" step.

> ⚠️ The app surface is the largest square that fits; at 720x400 it is small. Keep the
> place navigation on one scrolling row and let each place scroll inside itself.

> ⚠️ `audit-prism` (`xtask/src/audit.rs:968`) refuses any commit that adds a non-Prism
> file or grows non-Prism lines over its parent. Shrink as much as you add, or put the
> behaviour in LexLean.

> ⚠️ The try page's acceptance (`.github/workflows/try-page.yml`) drives the template's Calculator page behind `#input`;
> it is limited to the template repository (`.github/workflows/try-page.yml:24`).
