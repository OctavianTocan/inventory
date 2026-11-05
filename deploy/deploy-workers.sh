#!/usr/bin/env bash
# Deploys Inventory's two Workers in order: feedback-runner, then mymind (docs/feedback.md, Deploying).
# Before deploying it lists every feedback commit since the last deploy and flags commits that touch sign-in code,
# Wrangler files (and so Durable Object migrations), the runner itself or the ai-kit pin, so Tavi sees them first.
# The last deployed commit is kept in deploy/.last-worker-deploy (local, ignored by Git).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
marker="deploy/.last-worker-deploy"
head="$(git rev-parse HEAD)"
flagged='^(rust/crates/core/src/auth\.rs|rust/crates/server/src/serve\.rs|rust/crates/server/src/download\.rs|.*wrangler\.jsonc|feedback-runner/|vendor/(pulls\.review|ai-kit))'

if [[ -f "$marker" ]]; then
  since="$(cat "$marker")"
  echo "Feedback commits since the last deploy ($since):"
  found=0
  while read -r commit; do
    [[ -z "$commit" ]] && continue
    if git log -1 --format=%B "$commit" | grep -q '^Feedback-Id: '; then
      found=1
      echo "  $(git log -1 --format='%h %s' "$commit")"
      git diff-tree --no-commit-id --name-only -r "$commit" | grep -E "$flagged" | sed 's/^/      FLAGGED: /' || true
    fi
  done < <(git rev-list --reverse "$since..$head")
  [[ $found == 0 ]] && echo "  none"
  echo "Any commit touching the flagged paths since then:"
  git diff --name-only "$since" "$head" | grep -E "$flagged" | sed 's/^/  /' || echo "  none"
else
  echo "No earlier deploy recorded; deploying $head."
fi

read -r -p "Deploy feedback-runner, then mymind, at $(git rev-parse --short HEAD)? [y/N] " answer
[[ "$answer" == "y" || "$answer" == "Y" ]] || { echo "Not deployed."; exit 1; }

(cd feedback-runner && npm ci && npx wrangler deploy)
(cd rust/crates/worker && npx wrangler deploy --config wrangler.jsonc)
echo "$head" > "$marker"
echo "Deployed $head."
