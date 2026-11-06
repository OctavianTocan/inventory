#!/usr/bin/env python3
"""Android version gate and fail-closed release publisher. Uses only the Python standard library."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SITE = "https://mymind.octaviantocan.com"
HEADING = re.compile(r"## (\d+\.\d+\.\d+) \((\d+)\), (\d{4}-\d{2}-\d{2})")


def fail(message):
    raise ValueError(message)


def history(text):
    entries = []
    for line in text.splitlines():
        line = line.rstrip()
        if line.startswith("## "):
            match = HEADING.fullmatch(line)
            if not match:
                fail(f"invalid changelog heading: {line}")
            name, code, date = match.groups()
            if not 0 < int(code) <= 2100000000:
                fail("versionCode must be between 1 and 2100000000")
            entries.append(dict(versionName=name, versionCode=int(code), date=date, notes=[]))
        elif line.startswith("- "):
            if not entries or not line[2:].strip():
                fail("changelog has an empty note or a note before its heading")
            entries[-1]["notes"].append(line[2:].strip())
    if not entries or any(not entry["notes"] for entry in entries):
        fail("changelog needs release entries with notes")
    for newer, older in zip(entries, entries[1:]):
        if newer["versionCode"] <= older["versionCode"] or version(newer["versionName"]) <= version(older["versionName"]):
            fail("changelog versionName and versionCode must both increase")
    return entries


def version(name):
    return tuple(map(int, name.split(".")))


def runtime_path(path):
    if not path.startswith("android/") or path.endswith("/CONTRACTS"):
        return False
    source = re.search(r"/src/([^/]+)/", path)
    runtime_source = source is not None and not source[1].lower().startswith(("test", "androidtest"))
    return (runtime_source or path.endswith((".gradle.kts", ".gradle"))
            or path in {"android/gradle.properties", "android/gradlew", "android/gradlew.bat"}
            or path.startswith("android/gradle/"))


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True).stdout


def gate(base, head):
    current = history(git("show", f"{head}:android/CHANGELOG.md").decode())[0]
    previous = history(git("show", f"{base}:android/CHANGELOG.md").decode())[0]
    paths = git("diff", "--name-only", "--no-renames", "-z", base, head).decode().split("\0")
    changed = any(runtime_path(path) for path in paths)
    if current["versionCode"] < previous["versionCode"] or version(current["versionName"]) < version(previous["versionName"]):
        fail("Android changelog versions must never decrease")
    if changed and (current["versionCode"] <= previous["versionCode"] or version(current["versionName"]) <= version(previous["versionName"])):
        fail("Android main source/build changed: add a newer versionName and versionCode to android/CHANGELOG.md")
    print("Android runtime version increase verified" if changed else "No Android runtime/build change; no version bump required")


def manifest(path, missing=False):
    if missing and not path.exists():
        return {"files": []}
    try:
        data = json.loads(path.read_text())
        if not isinstance(data, dict) or not isinstance(data.get("files"), list) or any(not isinstance(item, dict) for item in data["files"]):
            fail("manifest must be an object containing a files array of objects")
        for entry in data["files"]:
            if entry.get("platform") == "android":
                if type(entry.get("versionCode")) is not int or entry["versionCode"] <= 0:
                    fail("Android manifest versionCode is invalid")
                if not isinstance(entry.get("versionName"), str) or not re.fullmatch(r"\d+\.\d+\.\d+", entry["versionName"]):
                    fail("Android manifest versionName is invalid")
        return data
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot read valid manifest: {error}")


def published(data):
    entries = [entry for entry in data["files"] if entry.get("platform") == "android"]
    return max(entries, key=lambda entry: entry["versionCode"], default=None)


def required(name):
    value = os.environ.get(name, "").strip()
    if not value:
        fail(f"{name} is required")
    return value


def command(args, label, **kwargs):
    result = subprocess.run(args, capture_output=True, **kwargs)
    if result.returncode:
        fail(f"{label} failed (exit {result.returncode}); check its prerequisites")
    return result.stdout


def certificate(value):
    result = value.strip().replace(":", "").lower()
    if not re.fullmatch(r"[0-9a-f]{64}", result):
        fail("ANDROID_RELEASE_CERT_SHA256 must be a SHA-256 certificate fingerprint")
    return result


def signing(production):
    required("ANDROID_RELEASE_PASSWORD")
    store = Path(os.environ.get("MYMIND_RELEASE_STORE", str(Path.home() / ".config/mymind/android-release.jks")))
    if not store.is_file():
        fail("release keystore is missing; set MYMIND_RELEASE_STORE to the existing production key")
    alias = os.environ.get("MYMIND_RELEASE_ALIAS", "mymind")
    der = command(["keytool", "-exportcert", "-alias", alias, "-keystore", str(store), "-storepass:env", "ANDROID_RELEASE_PASSWORD"], "release key/password/alias validation")
    digest = hashlib.sha256(der).hexdigest()
    expected = required("ANDROID_RELEASE_CERT_SHA256") if production else os.environ.get("ANDROID_RELEASE_CERT_SHA256")
    if expected and digest != certificate(expected):
        fail("release keystore certificate does not match ANDROID_RELEASE_CERT_SHA256; keep the existing production key")
    sdk = Path(os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT") or str(Path.home() / "Android/Sdk"))
    tools = sorted((sdk / "build-tools").glob("*/apksigner"), key=lambda path: tuple(int(n) for n in re.findall(r"\d+", path.parent.name)))
    if not tools or not tools[-1].with_name("aapt2").is_file():
        fail("apksigner and aapt2 are required in Android SDK build-tools")
    return tools[-1], digest


def verify_apk(apk, entry, signer, digest):
    output = command([str(signer), "verify", "--print-certs", str(apk)], "APK signature verification").decode()
    certs = re.findall(r"certificate SHA-256 digest: ([0-9a-fA-F:]+)", output)
    if not certs or any(certificate(item) != digest for item in certs):
        fail("APK is not signed with the existing release key")
    output = command([str(signer.with_name("aapt2")), "dump", "badging", str(apk)], "APK version validation").decode()
    match = re.search(r"package: name='([^']+)' versionCode='(\d+)' versionName='([^']+)'", output)
    if not match or match.groups() != ("com.octaviantocan.mymind", str(entry["versionCode"]), entry["versionName"]):
        fail("APK application ID or version does not match the changelog")


def checksum(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


class Remote:
    def __init__(self, stage):
        self.stage = stage
        self.token = None
        for name in ("CLOUDFLARE_ACCOUNT_ID", "CLOUDFLARE_API_TOKEN", "TEST_USER_NAME", "TEST_USER_PASSWORD", "ANDROID_GOOGLE_WEB_CLIENT_ID"):
            required(name)
        self.wrangler = Path(os.environ.get("WRANGLER", str(ROOT / "rust/crates/worker/node_modules/wrangler/bin/wrangler.js")))
        if not self.wrangler.is_file():
            fail("pinned Wrangler is missing; run npm ci --prefix rust/crates/worker")

    def r2(self, action, key, path):
        args = ["node", str(self.wrangler), "r2", "object", action, f"mymind/downloads/{key}", "--file", str(path), "--remote", "--config", str(ROOT / "rust/crates/worker/wrangler.jsonc")]
        if action == "put":
            args += ["--content-type", "application/json" if key == "manifest.json" else "application/vnd.android.package-archive"]
        command(args, f"R2 {action} {key}", cwd=ROOT / "rust/crates/worker", env={**os.environ, "WRANGLER_SEND_METRICS": "false"})

    def request(self, path, output, method="GET", payload=None):
        args = ["curl", "--silent", "--show-error", "--fail", "--connect-timeout", "15", "--max-time", "300", "--request", method, "--output", str(output)]
        if self.token:
            headers = self.stage / "authorization"
            headers.write_text(f"Authorization: Bearer {self.token}\n")
            headers.chmod(0o600)
            args += ["--header", f"@{headers}"]
        if payload is not None:
            args += ["--header", "Content-Type: application/json", "--data-binary", "@-"]
        args += [SITE + path]
        command(args, f"authenticated download request {method} {path}", input=payload)

    def signin(self):
        output = self.stage / "signin.json"
        self.request("/app/v1/auth/password", output, "POST", json.dumps(dict(username=required("TEST_USER_NAME"), password=required("TEST_USER_PASSWORD"), device="android-release-ci")).encode())
        self.token = json.loads(output.read_text()).get("token")
        if not isinstance(self.token, str) or not self.token or re.search(r"[\r\n]", self.token):
            fail("test-user sign-in did not return an app token")
        output.unlink()

    def verify(self, entry, signer, digest):
        description = self.stage / "hosted-android.json"
        self.request("/download/android.json", description)
        live = json.loads(description.read_text())
        for field in ("versionCode", "versionName", "bytes", "sha256", "notes", "history"):
            if live.get(field) != entry.get(field):
                fail(f"hosted Android {field} does not match the release candidate")
        if live.get("url") != f"/download/{entry['name']}":
            fail("hosted Android URL does not match the release candidate")
        apk = self.stage / "hosted.apk"
        self.request(live["url"], apk)
        if apk.stat().st_size != entry["bytes"] or checksum(apk) != entry["sha256"]:
            fail("hosted APK does not match the candidate checksum/size")
        verify_apk(apk, entry, signer, digest)

    def close(self):
        if self.token:
            try:
                self.request("/app/v1/auth/signout", self.stage / "signout", "POST")
            except ValueError:
                print("android-release: test-user signout failed", file=sys.stderr)


def publish(local=None, if_needed=False):
    entries = history(Path(os.environ.get("MYMIND_CHANGELOG", str(ROOT / "android/CHANGELOG.md"))).read_text())
    latest = entries[0]
    with tempfile.TemporaryDirectory(prefix="android-release-") as temporary:
        stage = Path(temporary)
        remote = None
        try:
            if local:
                target = Path(local).resolve()
                data = manifest(target / "manifest.json", missing=True)
            else:
                remote = Remote(stage)
                remote.r2("get", "manifest.json", stage / "manifest.json")
                data = manifest(stage / "manifest.json")
            previous = published(data)
            equal = previous and latest["versionCode"] == previous["versionCode"]
            if previous and (latest["versionCode"] < previous["versionCode"] or version(latest["versionName"]) < version(previous["versionName"])):
                fail("release is older than the published Android version")
            if previous and (latest["versionCode"] <= previous["versionCode"] or version(latest["versionName"]) <= version(previous["versionName"])) and not (remote and if_needed and equal):
                fail("versionName and versionCode must both be higher than the published Android version")
            signer, digest = signing(production=remote is not None)
            if remote:
                remote.signin()
            if remote and if_needed and equal:
                if any(previous.get(field) != latest[field] for field in ("versionName", "notes")) or previous.get("history") != entries:
                    fail("published version already exists with different release notes/history; bump the version")
                if previous.get("name") != f"mymind-{latest['versionName']}.apk":
                    fail("published APK name does not match this version")
                remote.verify(previous, signer, digest)
                print(f"Already published and verified {latest['versionName']} ({latest['versionCode']}); no upload")
                return
            args = ["nice", "-n", "19", "./gradlew", "--no-daemon", "--max-workers=2", *shlex.split(os.environ.get("GRADLE_ARGS", "")), f"-PmymindChangelog={os.environ.get('MYMIND_CHANGELOG', str(ROOT / 'android/CHANGELOG.md'))}"]
            if remote:
                args += [f"-PmymindGoogleWebClientId={required('ANDROID_GOOGLE_WEB_CLIENT_ID')}"]
            # Gradle output is retained on the console; it must never receive passwords as command arguments.
            subprocess.run([*args, ":app:assembleRelease"], cwd=ROOT / "android", check=True)
            source = ROOT / "android/app/build/outputs/apk/release/app-release.apk"
            verify_apk(source, latest, signer, digest)
            name = f"mymind-{latest['versionName']}.apk"
            apk = stage / name
            shutil.copyfile(source, apk)
            entry = dict(latest, name=name, label="Android", platform="android", bytes=apk.stat().st_size,
                         sha256=checksum(apk), built=datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"), history=entries)
            revision = os.environ.get("ANDROID_RELEASE_REVISION")
            if revision:
                if not re.fullmatch(r"[0-9a-f]{40}", revision):
                    fail("ANDROID_RELEASE_REVISION must be a full Git commit SHA")
                entry["sourceCommit"] = revision
            updated = {**data, "files": [item for item in data["files"] if item.get("platform") != "android"] + [entry]}
            candidate = stage / "candidate-manifest.json"
            candidate.write_text(json.dumps(updated, indent=2) + "\n")
            if remote:
                remote.r2("put", name, apk)
                remote.r2("get", "manifest.json", stage / "latest-manifest.json")
                if manifest(stage / "latest-manifest.json") != data:
                    fail("live manifest changed during publication; refusing to overwrite it, retry after the other publisher finishes")
                remote.r2("put", "manifest.json", candidate)
                remote.verify(entry, signer, digest)
                print(f"Published and verified {name} at {SITE}/download")
            else:
                target.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(apk, target / name)
                pending = target / ".android-release-manifest.json"
                shutil.copyfile(candidate, pending)
                pending.replace(target / "manifest.json")
                print(f"Published {name} into {target}")
        finally:
            if remote:
                remote.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("gate", help="require a changelog increase for Android runtime/build changes")
    check.add_argument("--base", required=True)
    check.add_argument("--head", default="HEAD")
    release = commands.add_parser("publish", help="publish with the existing release key; production needs CI credentials documented in docs/android-releases.md")
    release.add_argument("--local", metavar="DOWNLOADS_DIR")
    release.add_argument("--if-needed", action="store_true", help="verify an identical published release instead of uploading it again")
    args = parser.parse_args()
    try:
        if args.command == "gate":
            gate(args.base, args.head)
        else:
            publish(args.local, args.if_needed)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"android-release: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
