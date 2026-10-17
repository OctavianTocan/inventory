#!/usr/bin/env bash
# Zips the Chrome extension and publishes it where /download serves it. See --help.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
extension="$root/extension"
site=https://mymind.octaviantocan.com

usage() {
  cat <<'EOF'
Usage:
  deploy/publish-extension.sh --local <downloads dir>
  deploy/publish-extension.sh
  deploy/publish-extension.sh --help

Zips extension/ (with the mymind assets in extension/mymind/, without tests/, tools/ and dot files) into
mymind-extension-<version>.zip, with everything under one mymind-extension/ folder, and writes its entry into
manifest.json. The version comes from extension/manifest.json and must be higher than the published one. Every file
the extension manifest names has to exist; extension/tools/copy-mymind-assets.mjs copies the mymind assets.

--local <dir>   Publishes into a local downloads directory. No upload and no network.
(no option)     Publishes to production: stages the zip and the manifest under rust/target/publish-extension, uploads
                them with deploy/upload-downloads.sh, then checks as the test user that /download lists the zip and
                that the downloaded zip matches its checksum. The published state is copied back to downloads/.

The test user (TEST_USER_NAME, TEST_USER_PASSWORD) comes from the environment, or from .env in the repository.
Every failed step prints a message and exits non-zero.
EOF
}

die() { echo "publish-extension: $*" >&2; exit 1; }

# Prints the extension's version after checking that every file its manifest names exists.
extension_version() {
  python3 -I - "$extension" <<'PYEOF'
import json, os, re, sys
base = sys.argv[1]
try:
    manifest = json.load(open(os.path.join(base, "manifest.json"), encoding="utf-8"))
except (OSError, ValueError) as error:
    sys.exit(f"cannot read extension/manifest.json: {error}")
version = manifest.get("version", "")
if not re.fullmatch(r"\d+(\.\d+){0,3}", version):
    sys.exit(f"extension/manifest.json: version {version!r} is not a Chrome version")
named = list(manifest.get("icons", {}).values()) + list(manifest.get("action", {}).get("default_icon", {}).values())
named += [manifest.get("background", {}).get("service_worker"), manifest.get("options_ui", {}).get("page")]
named += [path for group in manifest.get("web_accessible_resources", []) for path in group.get("resources", [])]
missing = [path for path in named if path and not os.path.isfile(os.path.join(base, path))]
if missing:
    sys.exit("missing in extension/: " + ", ".join(missing) + " (run node extension/tools/copy-mymind-assets.mjs)")
print(version)
PYEOF
}

# Prints the published extension version in a manifest file, or 0.
manifest_version() {
  python3 -I - "$1" <<'PYEOF'
import json, sys
try:
    files = json.load(open(sys.argv[1]))["files"]
except (OSError, ValueError, KeyError):
    files = []
versions = [f.get("version", "0") for f in files if f.get("platform") == "chrome-extension"]
print(max(versions, key=lambda v: [int(part) for part in v.split(".")], default="0"))
PYEOF
}

# Succeeds when version $1 is higher than version $2.
newer() {
  python3 -I -c 'import sys; key = lambda v: [int(p) for p in v.split(".")] + [0] * (4 - len(v.split("."))); sys.exit(0 if key(sys.argv[1]) > key(sys.argv[2]) else 1)' "$1" "$2"
}

zip_and_stage() {
  local out=$1 name=$2 version=$3
  mkdir -p "$out"
  find "$out" -maxdepth 1 -name 'mymind-extension-*.zip' -delete
  python3 -I - "$extension" "$out/$name" <<'PYEOF'
import os, sys, zipfile
base, target = sys.argv[1:3]
skip = {"tests", "tools"}
count = 0
with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as archive:
    for folder, dirs, files in os.walk(base):
        top = os.path.relpath(folder, base) == "."
        dirs[:] = sorted(d for d in dirs if not d.startswith(".") and not (top and d in skip))
        for file in sorted(files):
            if file.startswith("."):
                continue
            path = os.path.join(folder, file)
            archive.write(path, os.path.join("mymind-extension", os.path.relpath(path, base)))
            count += 1
print(f"zipped {count} files")
PYEOF
  write_manifest "$out" "$name" "$version"
}

write_manifest() {
  python3 -I - "$1" "$2" "$3" <<'PYEOF'
import datetime, hashlib, json, os, sys
out, name, version = sys.argv[1:4]
data = open(os.path.join(out, name), "rb").read()
entry = {
    "name": name, "label": "Chrome extension", "platform": "chrome-extension", "bytes": len(data),
    "sha256": hashlib.sha256(data).hexdigest(),
    "built": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
    "version": version,
}
path = os.path.join(out, "manifest.json")
try:
    manifest = json.load(open(path))
except (OSError, ValueError):
    manifest = {"files": []}
manifest["files"] = [f for f in manifest.get("files", []) if f.get("platform") != "chrome-extension"] + [entry]
with open(path, "w") as handle:
    json.dump(manifest, handle, indent=2)
    handle.write("\n")
print(f"{name}: {entry['bytes']} bytes, sha256 {entry['sha256']}")
PYEOF
}

# Production only: the steps below talk to the hosted site and are skipped by --local.
production_publish() {
  local name=$1 version=$2
  local stage="$root/rust/target/publish-extension"
  [ -f "$root/.env" ] && { set -a; . "$root/.env"; set +a; }
  [ -n "${TEST_USER_NAME:-}" ] && [ -n "${TEST_USER_PASSWORD:-}" ] || die "TEST_USER_NAME and TEST_USER_PASSWORD are required"
  [ -f "$root/downloads/manifest.json" ] || die "downloads/manifest.json is missing; it is the published state the upload starts from"
  local published
  published=$(manifest_version "$root/downloads/manifest.json")
  newer "$version" "$published" || die "version $version is not higher than the published $published"
  rm -rf "$stage"
  mkdir -p "$stage/downloads"
  cp "$root/downloads/manifest.json" "$stage/downloads/"
  zip_and_stage "$stage/downloads" "$name" "$version"
  "$root/deploy/upload-downloads.sh" "$stage" || die "the upload failed"
  local token
  token=$(app_token) || die "could not sign in as the test user"
  authed "$token" -f "$site/download" | grep -qF "/download/$name\"" || die "the hosted /download does not list $name"
  local want got kind
  want=$(python3 -I -c 'import json,sys; print([f for f in json.load(open(sys.argv[1]))["files"] if f.get("platform") == "chrome-extension"][0]["sha256"])' "$stage/downloads/manifest.json")
  got=$(authed "$token" -f "$site/download/$name" | sha256sum | cut -d' ' -f1) || die "could not download the zip"
  [ "$want" = "$got" ] || die "the downloaded zip does not match its checksum"
  kind=$(authed "$token" -f -o /dev/null -w '%{content_type}' "$site/download/$name") || die "could not read the zip's type"
  [ "$kind" = application/zip ] || die "the zip is sent as $kind"
  authed "$token" -f -X POST "$site/app/v1/auth/signout" >/dev/null || true
  # downloads/ is the source later uploads start from, so it gets the published state.
  find "$root/downloads" -maxdepth 1 -name 'mymind-extension-*.zip' -delete
  cp "$stage/downloads/$name" "$stage/downloads/manifest.json" "$root/downloads/"
  echo "published $name to $site"
}

# curl with the app token as a header read from a pipe, so the token never shows in the process list.
authed() {
  local token=$1
  shift
  curl -sS -H @<(printf 'Authorization: Bearer %s\n' "$token") "$@"
}

# curl rather than urllib: Cloudflare's browser integrity check refuses urllib's requests (error 1010).
app_token() {
  python3 -I -c 'import json, os; print(json.dumps({"username": os.environ["TEST_USER_NAME"], "password": os.environ["TEST_USER_PASSWORD"], "device": "publish-extension"}))' \
    | curl -fsS -X POST -H 'Content-Type: application/json' --data-binary @- "$site/app/v1/auth/password" \
    | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["token"])'
}

mode=production
dir=
case "${1:-}" in
  --help|-h) usage; exit 0 ;;
  --local) mode=local; dir=${2:-}; [ -n "$dir" ] || die "--local needs a downloads directory" ;;
  "") ;;
  *) usage >&2; die "unknown option $1" ;;
esac

version=$(extension_version) || die "the extension is not ready to publish"
name="mymind-extension-$version.zip"

if [ "$mode" = local ]; then
  mkdir -p "$dir"
  published=$(manifest_version "$dir/manifest.json")
  newer "$version" "$published" || die "version $version is not higher than the published $published"
  zip_and_stage "$dir" "$name" "$version"
  echo "published $name into $dir"
else
  production_publish "$name" "$version"
fi
