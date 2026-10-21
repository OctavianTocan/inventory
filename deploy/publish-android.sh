#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
if [[ "${1:-}" != --help && "${1:-}" != -h && "${CI:-}" != true && -f "$root/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  . "$root/.env"
  set +a
fi
exec python3 -B "$root/deploy/android-release.py" publish "$@"
