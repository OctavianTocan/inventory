#!/usr/bin/env bash
# Uploads a staged downloads folder to the hosted site's R2 bucket. See --help.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
site=https://mymind.octaviantocan.com
worker="$root/rust/crates/worker"
wrangler=${WRANGLER:-$root/worker/node_modules/wrangler/bin/wrangler.js}

usage() {
  cat <<'EOF'
Usage: deploy/upload-downloads.sh <stage dir>

Uploads every file under <stage dir>/downloads/ to the bucket's downloads/ folder on the hosted site, replacing
objects of the same name. The Worker's /_admin/ routes exist only while its ADMIN_TOKEN secret is set, so the script
sets the secret from ADMIN_TOKEN (the environment, else .env), uploads with `mymind upload-hosted`, and deletes the
secret again on every exit, failed or not. Setting and deleting the secret each make a new Worker version with the
same code.

WRANGLER points at another wrangler.js.
EOF
}

die() { echo "upload-downloads: $*" >&2; exit 1; }

wrangler() { (cd "$worker" && WRANGLER_SEND_METRICS=false node "$wrangler" "$@"); }

close_admin() {
  wrangler secret delete ADMIN_TOKEN >/dev/null 2>&1 || true
  if wrangler secret list 2>/dev/null | grep -q '"ADMIN_TOKEN"'; then
    echo "upload-downloads: ADMIN_TOKEN is still set on the Worker; delete it with wrangler secret delete ADMIN_TOKEN" >&2
    exit 1
  fi
}

case "${1:-}" in
  --help|-h) usage; exit 0 ;;
  "") usage >&2; die "a stage directory is required" ;;
esac
stage=$(cd "$1" && pwd)
[ -d "$stage/downloads" ] || die "$stage/downloads does not exist"
if [ -z "${ADMIN_TOKEN:-}" ] && [ -f "$root/.env" ]; then
  set -a
  # shellcheck disable=SC1091
  . "$root/.env"
  set +a
fi
[ -n "${ADMIN_TOKEN:-}" ] || die "ADMIN_TOKEN is not set"

trap close_admin EXIT
printf '%s' "$ADMIN_TOKEN" | wrangler secret put ADMIN_TOKEN >/dev/null || die "could not set the ADMIN_TOKEN secret"
# The new secret takes a few seconds to reach every Cloudflare location.
for attempt in 1 2 3 4 5 6; do
  if "$root/rust/target/release/mymind" upload-hosted "$site" bucket --bucket "$stage" --prefix downloads/ --force; then
    exit 0
  fi
  [ "$attempt" = 6 ] && die "upload-hosted failed"
  sleep 10
done
