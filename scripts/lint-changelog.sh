#!/bin/bash
# Lint the [Unreleased] block of CHANGELOG.md for structural drift.
#
# Checks (all scoped to [Unreleased] only — released sections are frozen):
#   1. Duplicate subsection headings.
#        - `### Changed` twice  → fail
#        - `### Added — foo` twice (same topic) → fail
#        - Bare `### Added` mixed with `### Added — <topic>` → fail
#   2. Keep-a-Changelog ordering. Canonical order:
#        Breaking, Added, Changed, Deprecated, Removed, Fixed, Security, Dependencies
#      Topic-grouped `### Added — <topic>` all rank as Added.
#   3. Line length. Lines over 100 cols fail (fenced code blocks exempt).
#   4. Non-canonical heading names. Warns; does not fail.
#
# Usage: bash scripts/lint-changelog.sh [path/to/CHANGELOG.md]

set -e

CHANGELOG="${1:-CHANGELOG.md}"

if [ ! -f "$CHANGELOG" ]; then
  echo "lint-changelog: $CHANGELOG not found" >&2
  exit 2
fi

BLOCK=$(awk '/^## \[Unreleased\]/{b=1; next} /^## \[/{b=0} b' "$CHANGELOG")
if [ -z "$BLOCK" ]; then
  echo "lint-changelog: no [Unreleased] block in $CHANGELOG" >&2
  exit 2
fi

CANONICAL="Breaking Added Changed Deprecated Removed Fixed Security Dependencies"
fail=0

# 1. Duplicate subsection headings.
dupes=$(awk '
  /^### / {
    h = $0; sub(/^### /, "", h)
    if (h ~ / — /) {
      split(h, p, / — /); cat = p[1]; topic = p[2]
      if (++pair[cat "/" topic] == 2) print "duplicate topic subsection: ### " cat " — " topic
      topics[cat]++
    } else {
      if (++bare[h] == 2) print "duplicate subsection: ### " h
      bareset[h] = 1
    }
  }
  END {
    for (c in topics) if (c in bareset) print "mixed: bare ### " c " alongside ### " c " — <topic>"
  }
' <<<"$BLOCK")
if [ -n "$dupes" ]; then
  printf 'FAIL (duplicate/mixed subsections):\n%s\n\n' "$dupes" | sed 's/^\(.\)/  \1/' >&2
  fail=1
fi

# 2. Keep-a-Changelog ordering.
order_err=$(awk -v canon="$CANONICAL" '
  BEGIN { n = split(canon, a, " "); for (i=1; i<=n; i++) rank[a[i]] = i; last = 0; last_name = "" }
  /^### / {
    h = $0; sub(/^### /, "", h); sub(/ — .*/, "", h)
    r = rank[h]
    if (r == 0) next
    if (r < last) { printf "### %s appears after ### %s\n", h, last_name; exit 1 }
    last = r; last_name = h
  }
' <<<"$BLOCK") || true
if [ -n "$order_err" ]; then
  printf 'FAIL (order violation):\n  %s\n  Expected: %s\n\n' "$order_err" "$CANONICAL" >&2
  fail=1
fi

# 3. Line length (> 100 cols, skipping fenced code blocks).
long=$(awk '
  /^```/ { code = !code; next }
  !code && length > 100 {
    snippet = (length > 80) ? substr($0, 1, 77) "..." : $0
    printf "L%d (%d chars): %s\n", NR, length, snippet
  }
' <<<"$BLOCK")
if [ -n "$long" ]; then
  printf 'FAIL (lines over 100 columns in [Unreleased]):\n%s\n\n' "$long" | sed 's/^\(.\)/  \1/' >&2
  fail=1
fi

# 4. Non-canonical headings (warn only).
noncanon=$(awk -v canon="$CANONICAL" '
  BEGIN { n = split(canon, a, " "); for (i=1; i<=n; i++) k[a[i]] = 1 }
  /^### / {
    h = $0; sub(/^### /, "", h); sub(/ — .*/, "", h)
    if (!(h in k)) print h
  }
' <<<"$BLOCK" | sort -u)
if [ -n "$noncanon" ]; then
  printf 'WARN (non-canonical subsection names):\n%s\n  Canonical: %s\n  Or use "### Added — <topic>".\n\n' \
    "$noncanon" "$CANONICAL" | sed 's/^\(.\)/  \1/' >&2
fi

if [ "$fail" -ne 0 ]; then
  echo "✗ CHANGELOG [Unreleased] lint failed" >&2
  exit 1
fi

echo "✓ CHANGELOG [Unreleased] lint passed"
