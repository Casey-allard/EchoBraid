#!/usr/bin/env bash
# Project this repository's Rust workspace into a Prism project with UORust
# (afflom/UORust), relock it against the pinned SDK's LexLean and have the Lean
# kernel verify it. Run inside the SDK image; `--check` fails when the
# committed uorust/ differs from a fresh projection.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
tool=${UORUST:-/tmp/uorust/bin/uorust}
if [[ ! -x $tool ]]; then
  cargo install --locked --root /tmp/uorust --git https://github.com/afflom/UORust --tag v0.4.0 uorust
fi
out=$(mktemp -d)
"$tool" project "$root" --out "$out" --name EchoBraidRust --last-modified 2026-11-17T00:00:00Z
(cd "$out" && lexlean lock >/dev/null && lexlean fmt --check && lexlean verify)
rm -rf "$out/.lexlean" "$out/.lake"
if [[ ${1:-} == --check ]]; then
  diff -r "$out" "$root/uorust" && echo "uorust/: current and verified"
else
  rm -rf "$root/uorust" && cp -r "$out" "$root/uorust" && echo "uorust/: rewritten and verified"
fi
