"""Render acceptance vectors for app/src/App.lex.tex from (request bytes, response text) pairs on stdin.
Each input line: hex-request<TAB>response-with-\\t-escapes. Output: the LexLean cons-list."""
import sys
rows = []
for line in sys.stdin.read().splitlines():
    if not line.strip():
        continue
    req, resp = line.split("\t", 1)
    resp = resp.encode().decode("unicode_escape").encode()
    rows.append((req.strip().lower(), resp.hex()))
out = "(nil Foundation.View.Text.V1.Model::AcceptanceVector)"
for req, resp in reversed(rows):
    out = f"(cons (record Foundation.View.Text.V1.Model::AcceptanceVector {{request := 0x{req}, response := 0x{resp}}}) {out})"
print(out)
