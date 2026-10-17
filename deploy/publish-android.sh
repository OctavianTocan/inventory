#!/usr/bin/env bash
# Builds the signed Android release and publishes it where /download serves it. See --help.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
android="$root/android"
changelog=${MYMIND_CHANGELOG:-$android/CHANGELOG.md}
site=https://mymind.octaviantocan.com

usage() {
  cat <<'EOF'
Usage:
  deploy/publish-android.sh --local <downloads dir>
  deploy/publish-android.sh
  deploy/publish-android.sh --help

Reads the top entry of android/CHANGELOG.md, builds the release APK with the release key, checks that the APK is
signed by that key, and writes mymind-<versionName>.apk and manifest.json.

--local <dir>   Publishes into a local downloads directory. No upload and no network. The versionCode must be higher
                than the Android entry already in <dir>/manifest.json, if any.
(no option)     Publishes to production: builds into a scratch directory under android/build, uploads it with
                deploy/upload-downloads.sh, then reads /download/android.json back as the test user and compares the
                versionCode and the checksum of the downloaded APK. The versionCode must be higher than the
                published one.

The release key comes from MYMIND_RELEASE_STORE (default ~/.config/mymind/android-release.jks), MYMIND_RELEASE_ALIAS
(default mymind) and ANDROID_RELEASE_PASSWORD. The password is read from the environment, or from .env in the
repository when it is not set. MYMIND_CHANGELOG points at another changelog file. Extra Gradle arguments can be
passed in GRADLE_ARGS (for example -PmymindServer=...).

Every failed step prints a message and exits non-zero.
EOF
}

die() { echo "publish-android: $*" >&2; exit 1; }

load_password() {
  [ -n "${ANDROID_RELEASE_PASSWORD:-}" ] && return 0
  [ -f "$root/.env" ] || die "ANDROID_RELEASE_PASSWORD is not set and there is no .env"
  set -a
  # shellcheck disable=SC1091
  . "$root/.env"
  set +a
  [ -n "${ANDROID_RELEASE_PASSWORD:-}" ] || die "ANDROID_RELEASE_PASSWORD is not set"
}

# Prints "name code" of the top changelog entry; Gradle enforces the full rules at build time.
top_entry() {
  python3 -I - "$changelog" <<'EOF'
import re, sys
try:
    text = open(sys.argv[1], encoding="utf-8").read()
except OSError as error:
    sys.exit(f"cannot read {sys.argv[1]}: {error}")
for number, line in enumerate(text.splitlines(), 1):
    if line.startswith("## "):
        m = re.fullmatch(r"## (\d+\.\d+\.\d+) \((\d+)\), \d{4}-\d{2}-\d{2}", line)
        if not m:
            sys.exit(f"{sys.argv[1]}:{number}: heading must look like '## 0.2.0 (2), 2026-10-17'")
        print(m[1], m[2])
        sys.exit(0)
sys.exit(f"{sys.argv[1]}: no version entry")
EOF
}

# Prints the published Android versionCode in a manifest file, or 0.
manifest_code() {
  python3 -I - "$1" <<'EOF'
import json, sys
try:
    files = json.load(open(sys.argv[1]))["files"]
except (OSError, ValueError, KeyError):
    files = []
codes = [f.get("versionCode", 0) for f in files if f.get("platform") == "android"]
print(max(codes, default=0))
EOF
}

build_and_stage() {
  local out=$1 name=$2 code=$3
  mkdir -p "$out"
  (cd "$android" && nice -n 19 ./gradlew --max-workers=2 ${GRADLE_ARGS:-} -PmymindChangelog="$changelog" :app:assembleRelease) \
    || die "assembleRelease failed"
  local apk="$android/app/build/outputs/apk/release/app-release.apk"
  [ -f "$apk" ] || die "the release APK was not produced"
  verify_signature "$apk"
  find "$out" -maxdepth 1 -name 'mymind-*.apk' -delete
  cp "$apk" "$out/$name"
  write_manifest "$out" "$name" "$code"
}

verify_signature() {
  local apk=$1 apksigner store alias
  apksigner=$(ls -d "${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"/build-tools/*/apksigner 2>/dev/null | sort -V | tail -1 || true)
  [ -n "$apksigner" ] || die "apksigner not found in the Android SDK build-tools"
  store=${MYMIND_RELEASE_STORE:-$HOME/.config/mymind/android-release.jks}
  alias=${MYMIND_RELEASE_ALIAS:-mymind}
  local apk_cert key_cert
  apk_cert=$("$apksigner" verify --print-certs "$apk" 2>&1 | sed -n 's/.*certificate SHA-256 digest: //p' | head -1) \
    || die "apksigner verify failed"
  [ -n "$apk_cert" ] || die "apksigner did not report a certificate"
  key_cert=$(keytool -list -alias "$alias" -keystore "$store" -storepass:env ANDROID_RELEASE_PASSWORD 2>/dev/null \
    | sed -n 's/.*SHA-256)*: //p' | tr -d ':' | tr 'A-F' 'a-f' | head -1)
  [ -n "$key_cert" ] || die "keytool could not read the certificate of the release key"
  [ "$apk_cert" = "$key_cert" ] || die "the APK is not signed with the release key"
  echo "signature ok"
}

write_manifest() {
  python3 -I - "$1" "$2" "$3" "$changelog" <<'EOF'
import datetime, hashlib, json, os, re, sys
out, name, code, changelog = sys.argv[1:5]
history, current = [], None
for line in open(changelog, encoding="utf-8").read().splitlines():
    m = re.fullmatch(r"## (\d+\.\d+\.\d+) \((\d+)\), (\d{4}-\d{2}-\d{2})", line)
    if m:
        current = {"versionName": m[1], "versionCode": int(m[2]), "date": m[3], "notes": []}
        history.append(current)
    elif line.startswith("- ") and current:
        current["notes"].append(line[2:].strip())
data = open(os.path.join(out, name), "rb").read()
entry = {
    "name": name, "label": "Android", "platform": "android", "bytes": len(data),
    "sha256": hashlib.sha256(data).hexdigest(),
    "built": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
    "versionName": history[0]["versionName"], "versionCode": history[0]["versionCode"],
    "notes": history[0]["notes"], "history": history,
}
path = os.path.join(out, "manifest.json")
try:
    manifest = json.load(open(path))
except (OSError, ValueError):
    manifest = {"files": []}
manifest["files"] = [f for f in manifest.get("files", []) if f.get("platform") != "android"] + [entry]
with open(path, "w") as handle:
    json.dump(manifest, handle, indent=2)
    handle.write("\n")
print(f"{name}: {entry['bytes']} bytes, sha256 {entry['sha256']}")
EOF
}

# Production only: the steps below talk to the hosted site and are skipped by --local.
production_publish() {
  local name=$1 code=$2
  local stage="$android/build/publish-stage"
  [ -f "$root/.env" ] && { set -a; . "$root/.env"; set +a; }
  [ -n "${TEST_USER_NAME:-}" ] && [ -n "${TEST_USER_PASSWORD:-}" ] || die "TEST_USER_NAME and TEST_USER_PASSWORD are required"
  local token published
  token=$(app_token) || die "could not sign in as the test user"
  published=$(published_code "$token") || die "could not read the published release"
  [ "$code" -gt "$published" ] || die "versionCode $code is not higher than the published $published"
  rm -rf "$stage"
  mkdir -p "$stage/downloads"
  if [ -d "$root/downloads" ]; then cp "$root/downloads/manifest.json" "$stage/downloads/" 2>/dev/null || true; fi
  build_and_stage "$stage/downloads" "$name" "$code"
  "$root/deploy/upload-downloads.sh" "$stage" || die "the upload failed"
  local live
  live=$(authed "$token" -f "$site/download/android.json") || die "could not read android.json back"
  [ "$(printf '%s' "$live" | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["versionCode"])')" = "$code" ] \
    || die "the hosted versionCode is not $code"
  local want got
  want=$(printf '%s' "$live" | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["sha256"])')
  got=$(authed "$token" -f "$site/download/$name" | sha256sum | cut -d' ' -f1) || die "could not download the APK"
  [ "$want" = "$got" ] || die "the downloaded APK does not match its checksum"
  authed "$token" -f -X POST "$site/app/v1/auth/signout" >/dev/null || true
  # downloads/ is the source the Linux packager and later uploads start from, so it gets the published state.
  mkdir -p "$root/downloads"
  find "$root/downloads" -maxdepth 1 -name 'mymind-*.apk' -delete
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
  python3 -I -c 'import json, os; print(json.dumps({"username": os.environ["TEST_USER_NAME"], "password": os.environ["TEST_USER_PASSWORD"], "device": "publish-android"}))' \
    | curl -fsS -X POST -H 'Content-Type: application/json' --data-binary @- "$site/app/v1/auth/password" \
    | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["token"])'
}

published_code() {
  local status body
  body=$(authed "$1" -o - -w '\n%{http_code}' "$site/download/android.json") || return 1
  status=${body##*$'\n'}
  [ "$status" = 404 ] && { echo 0; return 0; }
  [ "$status" = 200 ] || return 1
  printf '%s' "${body%$'\n'*}" | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["versionCode"])'
}

mode=production
dir=
case "${1:-}" in
  --help|-h) usage; exit 0 ;;
  --local) mode=local; dir=${2:-}; [ -n "$dir" ] || die "--local needs a downloads directory" ;;
  "") ;;
  *) usage >&2; die "unknown option $1" ;;
esac

read -r version code < <(top_entry) || die "the changelog is not readable"
name="mymind-$version.apk"
load_password

if [ "$mode" = local ]; then
  mkdir -p "$dir"
  published=$(manifest_code "$dir/manifest.json")
  [ "$code" -gt "$published" ] || die "versionCode $code is not higher than the published $published"
  build_and_stage "$dir" "$name" "$code"
  echo "published $name into $dir"
else
  production_publish "$name" "$code"
fi
