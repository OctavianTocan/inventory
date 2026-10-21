#!/usr/bin/env python3
"""Release regressions; all subprocesses and artifacts stay in a temporary fixture."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[1]
CERT = b"synthetic test certificate"
DIGEST = hashlib.sha256(CERT).hexdigest()


class ReleaseTest(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="android-release-test-")
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        for directory in ("deploy", "android", "bin", "sdk/build-tools/37.0.0", "downloads", "temporary", "remote", "rust/crates/worker"):
            (self.root / directory).mkdir(parents=True)
        shutil.copy(REPO / "deploy/publish-android.sh", self.root / "deploy")
        helper = REPO / "deploy/android-release.py"
        if helper.exists():
            shutil.copy(helper, self.root / "deploy")
        (self.root / "android/CHANGELOG.md").write_text("## 0.4.0 (5), 2026-10-21\n- Card sheets.\n\n## 0.3.1 (4), 2026-10-18\n- Previous.\n")
        (self.root / "release.jks").write_bytes(b"synthetic test keystore")
        self.env = {"PATH": f"{self.root / 'bin'}:{os.environ['PATH']}", "HOME": str(self.root),
                    "TMPDIR": str(self.root / "temporary"),
                    "CI": "true", "ANDROID_HOME": str(self.root / "sdk"),
                    "ANDROID_RELEASE_PASSWORD": "synthetic-test-password", "MYMIND_RELEASE_STORE": str(self.root / "release.jks"),
                    "ANDROID_RELEASE_CERT_SHA256": DIGEST}
        self.executable("android/gradlew", """#!/usr/bin/env python3
from pathlib import Path
Path('built').touch()
target = Path('app/build/outputs/apk/release/app-release.apk')
target.parent.mkdir(parents=True, exist_ok=True)
target.write_bytes(b'synthetic APK')
""")
        self.executable("bin/keytool", f"#!/usr/bin/env python3\nimport os, sys\nif os.environ.get('KEYTOOL_FAIL'): sys.exit(1)\nif '-exportcert' in sys.argv: sys.stdout.buffer.write({CERT!r})\nelse: print('SHA256: {DIGEST}')\n")
        self.executable("sdk/build-tools/37.0.0/apksigner", f"#!/bin/sh\necho 'Signer #1 certificate SHA-256 digest: {DIGEST}'\n")
        self.executable("sdk/build-tools/37.0.0/aapt2", "#!/bin/sh\necho \"package: name='com.octaviantocan.mymind' versionCode='5' versionName='0.4.0'\"\n")

    def executable(self, name, text):
        path = self.root / name
        path.write_text(text)
        path.chmod(0o755)

    def publish(self, *args):
        return subprocess.run(["bash", str(self.root / "deploy/publish-android.sh"), *args],
                              cwd=self.root, env=self.env, capture_output=True, text=True)

    def test_corrupt_manifest_fails_before_build(self):
        (self.root / "downloads/manifest.json").write_text("{broken")
        result = self.publish("--local", str(self.root / "downloads"))
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertFalse((self.root / "android/built").exists(), result.stderr)
        self.assertIn("manifest", result.stderr)

    def remote(self, data=None):
        for path in (self.root / "remote").iterdir():
            path.unlink()
        self.env.update(CLOUDFLARE_ACCOUNT_ID="synthetic-account", CLOUDFLARE_API_TOKEN="synthetic-r2-token",
                        TEST_USER_NAME="synthetic-user", TEST_USER_PASSWORD="synthetic-password",
                        ANDROID_GOOGLE_WEB_CLIENT_ID="synthetic-google-client", REMOTE_ROOT=str(self.root / "remote"))
        self.env["WRANGLER"] = str(self.root / "wrangler.js")
        (self.root / "wrangler.js").touch()
        (self.root / "remote/manifest.json").write_text(json.dumps(data or self.other_downloads()))
        self.executable("bin/node", """#!/usr/bin/env python3
import json, os, shutil, sys
from pathlib import Path
root = Path(os.environ['REMOTE_ROOT'])
action, key = sys.argv[4:6]
name = key.removeprefix('mymind/downloads/')
with (root / 'operations').open('a') as log: log.write(f'{action} {name}\\n')
if action == 'put' and os.environ.get('R2_FAIL_KEY') == name: sys.exit(1)
path = Path(sys.argv[sys.argv.index('--file') + 1])
if action == 'get':
    if os.environ.get('R2_CHANGED') and (root / 'mymind-0.4.0.apk').exists():
        data = json.loads((root / name).read_text())
        data['concurrent'] = True
        path.write_text(json.dumps(data))
    else: shutil.copyfile(root / name, path)
else: shutil.copyfile(path, root / name)
""")
        self.executable("bin/curl", """#!/usr/bin/env python3
import json, os, shutil, sys
from pathlib import Path
root = Path(os.environ['REMOTE_ROOT'])
path = sys.argv[-1].removeprefix('https://mymind.octaviantocan.com')
out = Path(sys.argv[sys.argv.index('--output') + 1])
with (root / 'http').open('a') as log: log.write(path + '\\n')
if os.environ.get('CURL_FAIL_PATH') == path: sys.exit(22)
if path == '/app/v1/auth/password': out.write_text('{"token":"synthetic-app-token"}')
elif path == '/app/v1/auth/signout': out.write_text('{}')
elif path == '/download/android.json':
    data = json.loads((root / 'manifest.json').read_text())
    entry = max((f for f in data['files'] if f.get('platform') == 'android'), key=lambda f: f['versionCode'])
    out.write_text(json.dumps({**entry, 'url': '/download/' + entry['name']}))
else:
    shutil.copyfile(root / path.removeprefix('/download/'), out)
    if os.environ.get('CURL_CORRUPT'): out.write_bytes(b'corrupt APK')
""")

    def other_downloads(self):
        return {"library": "https://library.example", "custom": {"keep": True}, "files": [
            {"name": "linux.tar.gz", "bytes": 9, "sha256": "existing-linux"},
            {"platform": "chrome-extension", "name": "extension.zip", "version": "1.2.3"}]}

    def operations(self):
        path = self.root / "remote/operations"
        return path.read_text().splitlines() if path.exists() else []

    def test_local_preserves_other_downloads_and_rejects_duplicate(self):
        original = self.other_downloads()
        path = self.root / "downloads/manifest.json"
        path.write_text(json.dumps(original))
        result = self.publish("--local", str(path.parent))
        self.assertEqual(result.returncode, 0, result.stderr)
        data = json.loads(path.read_text())
        self.assertEqual(data["files"][:-1], original["files"])
        self.assertEqual(data["custom"], original["custom"])
        self.assertEqual(data["library"], original["library"])
        self.assertEqual(data["files"][-1]["versionCode"], 5)
        (self.root / "android/built").unlink()
        result = self.publish("--local", str(path.parent))
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "android/built").exists())

    def test_production_preserves_live_manifest_and_uploads_apk_first(self):
        self.remote()
        result = self.publish("--if-needed")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.operations(), ["get manifest.json", "put mymind-0.4.0.apk", "get manifest.json", "put manifest.json"])
        data = json.loads((self.root / "remote/manifest.json").read_text())
        self.assertEqual(data["files"][:-1], self.other_downloads()["files"])
        self.assertEqual(data["custom"], self.other_downloads()["custom"])
        self.assertIn("/download/mymind-0.4.0.apk", (self.root / "remote/http").read_text())
        self.assertFalse((self.root / "downloads/manifest.json").exists())

    def test_retry_verifies_without_building_or_writing(self):
        self.remote()
        self.assertEqual(self.publish("--if-needed").returncode, 0)
        (self.root / "android/built").unlink()
        (self.root / "remote/operations").unlink()
        result = self.publish("--if-needed")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Already published and verified", result.stdout)
        self.assertFalse((self.root / "android/built").exists())
        self.assertEqual(self.operations(), ["get manifest.json"])

    def test_retry_rejects_changed_notes_and_downgrades(self):
        self.remote()
        self.assertEqual(self.publish("--if-needed").returncode, 0)
        changelog = self.root / "android/CHANGELOG.md"
        changelog.write_text(changelog.read_text().replace("Card sheets.", "Different notes."))
        result = self.publish("--if-needed")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("different release notes", result.stderr)
        changelog.write_text("## 0.3.1 (4), 2026-10-18\n- Previous.\n")
        result = self.publish("--if-needed")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("older", result.stderr)

    def test_missing_prerequisites_fail_without_build_or_upload(self):
        for name in ("CLOUDFLARE_ACCOUNT_ID", "CLOUDFLARE_API_TOKEN", "TEST_USER_NAME", "TEST_USER_PASSWORD", "ANDROID_GOOGLE_WEB_CLIENT_ID", "ANDROID_RELEASE_PASSWORD", "ANDROID_RELEASE_CERT_SHA256"):
            with self.subTest(name=name):
                self.remote()
                original = self.env.pop(name)
                result = self.publish()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(name, result.stderr)
                self.assertFalse((self.root / "android/built").exists())
                self.assertFalse(any(op.startswith("put ") for op in self.operations()))
                self.env[name] = original

    def test_wrong_signing_key_password_missing_store_and_apk_version(self):
        self.remote()
        for name, value in (("ANDROID_RELEASE_CERT_SHA256", "0" * 64), ("KEYTOOL_FAIL", "yes"), ("MYMIND_RELEASE_STORE", str(self.root / "missing.jks"))):
            with self.subTest(name=name):
                old = self.env.get(name)
                self.env[name] = value
                result = self.publish()
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((self.root / "android/built").exists())
                self.assertFalse(any(op.startswith("put ") for op in self.operations()))
                if old is None: self.env.pop(name)
                else: self.env[name] = old
        self.executable("sdk/build-tools/37.0.0/aapt2", "#!/bin/sh\necho \"package: name='com.octaviantocan.mymind' versionCode='4' versionName='0.3.1'\"\n")
        result = self.publish()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match the changelog", result.stderr)
        self.assertFalse(any(op.startswith("put ") for op in self.operations()))

    def test_apk_upload_failure_leaves_manifest_untouched(self):
        self.remote()
        original = (self.root / "remote/manifest.json").read_bytes()
        self.env["R2_FAIL_KEY"] = "mymind-0.4.0.apk"
        result = self.publish()
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("put manifest.json", self.operations())
        self.assertEqual((self.root / "remote/manifest.json").read_bytes(), original)

    def test_manifest_failure_or_concurrent_change_is_not_success(self):
        for setting in ("R2_FAIL_KEY", "R2_CHANGED"):
            with self.subTest(setting=setting):
                self.remote()
                original = (self.root / "remote/manifest.json").read_bytes()
                self.env[setting] = "manifest.json" if setting == "R2_FAIL_KEY" else "yes"
                result = self.publish()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual((self.root / "remote/manifest.json").read_bytes(), original)
                self.env.pop(setting)

    def test_authenticated_readback_failure_and_corruption_are_not_success(self):
        for setting, value in (("CURL_FAIL_PATH", "/download/android.json"), ("CURL_CORRUPT", "yes")):
            with self.subTest(setting=setting):
                self.remote()
                self.env[setting] = value
                result = self.publish("--if-needed")
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn("Published and verified", result.stdout)
                self.env.pop(setting)

    def test_runtime_changes_need_both_versions_but_tests_do_not(self):
        previous = "## 0.3.1 (4), 2026-10-18\n- Previous.\n"
        self.env["GIT_PREVIOUS"] = previous
        self.executable("bin/git", """#!/usr/bin/env python3
import os, sys
from pathlib import Path
if sys.argv[1] == 'show':
    print(os.environ['GIT_PREVIOUS'] if sys.argv[2].startswith('base:') else Path('android/CHANGELOG.md').read_text())
else: sys.stdout.write(os.environ['GIT_PATH'] + '\\0')
""")
        current = self.root / "android/CHANGELOG.md"
        new = current.read_text()
        for path in ("android/app/src/main/Card.kt", "android/app/src/release/AndroidManifest.xml", "android/app/src/debug/Card.kt", "android/app/src/main/res/values/strings.xml", "android/buildSrc/src/main/Changelog.kt", "android/app/build.gradle.kts", "android/gradle/libs.versions.toml", "android/gradlew"):
            with self.subTest(path=path):
                self.env["GIT_PATH"] = path
                current.write_text(previous)
                result = self.run_gate()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("newer versionName and versionCode", result.stderr)
                current.write_text(new)
                self.assertEqual(self.run_gate().returncode, 0)
        for path in ("android/app/src/test/CardTest.kt", "android/app/src/androidTest/CardTest.kt", "android/app/screenshots/card.png", "android/README.md"):
            self.env["GIT_PATH"] = path
            current.write_text(previous)
            self.assertEqual(self.run_gate().returncode, 0, path)
        self.env["GIT_PATH"] = "android/app/src/main/Card.kt"
        for heading in ("0.3.1 (5)", "0.4.0 (4)"):
            current.write_text(f"## {heading}, 2026-10-21\n- New.\n")
            self.assertNotEqual(self.run_gate().returncode, 0, heading)

    def run_gate(self):
        return subprocess.run(["python3", "-B", str(self.root / "deploy/android-release.py"), "gate", "--base", "base", "--head", "head"],
                              cwd=self.root, env=self.env, capture_output=True, text=True)

    def test_workflow_structure(self):
        try:
            import yaml
        except ImportError:
            self.skipTest("PyYAML unavailable; parse workflow structure with actionlint/PyYAML before shipping")
        release = yaml.safe_load((REPO / ".github/workflows/android-release.yml").read_text())
        verify = yaml.safe_load((REPO / ".github/workflows/android-verify.yml").read_text())
        # PyYAML uses YAML 1.1 and reads GitHub's unquoted `on` as True.
        triggers = release.get("on", release.get(True))
        upstream = triggers["workflow_run"]
        self.assertEqual(upstream["workflows"], [verify["name"]])
        self.assertEqual(upstream["branches"], ["main"])
        self.assertEqual(upstream["types"], ["completed"])
        self.assertIn("workflow_dispatch", triggers)
        self.assertEqual(release["permissions"], {"contents": "read"})
        self.assertFalse(release["concurrency"]["cancel-in-progress"])
        jobs = release["jobs"]
        self.assertEqual(set(jobs["publish"]["needs"]), {"revision", "manual-verify"})
        self.assertEqual(jobs["manual-verify"]["needs"], "revision")
        self.assertEqual(jobs["manual-verify"]["uses"], "./.github/workflows/android-verify.yml")
        selected = jobs["manual-verify"]["with"]["revision"]
        publish_checkout = next(step["with"] for step in jobs["publish"]["steps"] if step.get("uses", "").startswith("actions/checkout@"))
        self.assertEqual(selected, publish_checkout["ref"], "manual verification and publication must use the same selected commit")
        self.assertFalse(publish_checkout["persist-credentials"])
        for name in ("sha", "base"):
            self.assertIn(name, jobs["revision"]["outputs"])
        verify_triggers = verify.get("on", verify.get(True))
        self.assertIsNone(verify_triggers["pull_request"], "required PR check must not be path-filtered")
        required = verify_triggers["workflow_call"]["inputs"]
        self.assertTrue(required["revision"]["required"])
        self.assertTrue(required["base"]["required"])


if __name__ == "__main__":
    unittest.main()
