# EchoBraid -- documentation

EchoBraid: a neurodivergent-affirming learning companion, modelled in LexLean, proved in Lean 4, shipped as a Prism holo, and improved by an autonomous engine.

This directory is a wiki maintained by an LLM: linked markdown pages, nothing outside the
repository. The rules are in [`CLAUDE.md`](CLAUDE.md).

## Start here

1. [`../README.md`](../README.md) -- what is published and how it is made
2. [[DESIGN]] -- how the uploaded concept became the model, and why
3. [[holo-build]] -- building and verifying the holo, and the PrismPM limits that shape the model
4. [[engine]] -- the self-improvement engine, one tick at a time

This wiki grows as work touches new areas; `/project-docs-setup --complete` fills the rest.

---

## concepts

None yet; pages are added as work touches this area.

## architecture

None yet; pages are added as work touches this area.

## subsystems

- [[holo-build]] -- the two LexLean projects, the exporter's limits, vectors, page gates
- [[engine]] -- Rule 30, Hessian sorting, Hebbian weights, the tick workflow

## decisions

- [[DESIGN]] -- how the uploaded concept became the model, and the architecture decisions

## reference

None yet; pages are added as work touches this area.

## operations

None yet; pages are added as work touches this area.

## history

None yet; pages are added as work touches this area.

---

## Using this wiki

```
/docs-sync     right after a deploy or release -- reads only what changed since verified_at
/docs-lint     integrity check (orphans, broken links, broken sources, stale, unverified)
/docs-query    ask the docs. If it is not there, the answer says so
/docs-status   commits not yet reflected, stale pages, recent work
```

Every page carries `verified_at`, the commit it was checked against. Change code, run
`/docs-sync`.
