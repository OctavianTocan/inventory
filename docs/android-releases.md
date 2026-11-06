# Android releases

`android/CHANGELOG.md` remains the source of `versionName`, `versionCode`, bundled notes and update history.
Its top entry is the newest release. Every merge to `main` that changes the app publishes one, so several threads
often want the same next number on the same day: check `main`'s top entry right before merging and take the next
number when another PR took yours.

The `Android verify` workflow compares a PR's checked-out merge revision with its base commit. Changes to Android
main sources, resources, Gradle build files, dependency versions or the wrapper require both a higher version name
and a higher version code. Tests, screenshots, documentation and `CONTRACTS` files alone need no bump (they are not built into the app). Versions cannot decrease. Verify also runs
`:app:lintVitalRelease`, the lint the release build runs: releases 0.6.68 to 0.6.81 passed verify and then failed in
`Android release` on a release-only lint error, until 0.6.82 (#108). A green verify on main is not a shipped release;
check the `Android release` run after merging.
Every PR runs the check so branch protection can require `android` (the job in `Android verify`) without leaving
unrelated PRs waiting on a paths-filtered workflow. **Configure this required check in the repository's main branch
protection/ruleset**; a workflow alone cannot prevent someone bypassing failed checks. This implementation does not
change repository protection settings.

After a successful `Android verify` **push on main**, `Android release` checks out that verification run's exact SHA.
It accepts only this repository's main push runs, confirms the commit belongs to main and publishes with the
existing signing key. A manual `Android release` dispatch must select **main**; it first reruns version policy,
release-tool regressions and the Android unit tests/debug build on the exact selected SHA. Main moving while a
run waits does not change the selected revision. Failed/cancelled verification cannot publish. Production release
runs share `android-production-release` concurrency with cancellation disabled.

## Required GitHub configuration

Configure these repository or organization Actions values before requesting a release. Their current existence
is **unknown**: secret enumeration in the implementation session returned HTTP 403. No credentials are committed,
configured, generated or replaced by this change. A requested release fails if a required value is empty.

| Kind | Name | Value |
| --- | --- | --- |
| Secret | `ANDROID_RELEASE_KEYSTORE_BASE64` | Base64 of the **existing production PKCS12 keystore**, encoded on one line; never a new key. |
| Secret | `ANDROID_RELEASE_PASSWORD` | Existing keystore/key password. |
| Secret | `CLOUDFLARE_API_TOKEN` | Account-scoped token permitted to read and write R2 objects in the existing `mymind` bucket (Workers R2 Storage Edit). It does not need Worker deployment or secret editing permissions. |
| Secret | `TEST_USER_NAME` | Existing password test user's username, admitted by the app's normal sign-in. |
| Secret | `TEST_USER_PASSWORD` | That user's existing password. |
| Variable | `ANDROID_RELEASE_CERT_SHA256` | SHA-256 fingerprint of the original production signing certificate, independently confirmed from the existing installed/published app or trusted release records. Hex, optionally colon-separated. Mandatory for production. |
| Variable | `ANDROID_GOOGLE_WEB_CLIENT_ID` | Existing Google OAuth web client ID used by the production Android app. |
| Variable | `CLOUDFLARE_ACCOUNT_ID` | Account containing the existing `mymind` R2 bucket. |
| Optional variable | `ANDROID_RELEASE_ALIAS` | Existing signing alias; defaults to `mymind`. |

The runner needs JDK 21, Android platform `android-37.0`, build-tools `37.0.0`, Node 22, Python 3 and curl.
The workflow installs the SDK and the repository's locked Wrangler dependency with
`npm ci --prefix rust/crates/worker --ignore-scripts`; it does not build or deploy the Rust Worker/server.
The keystore is decoded into a private runner temporary file and removed on exit. The publisher checks its
certificate against the mandatory pin before building. No signing fallback exists. Passwords and app bearer
tokens are never command arguments. Outside CI, the wrapper retains the existing local `.env` loading behavior.

## Publication and retries

Run the release workflow manually from the Actions page on main, or let main verification trigger it.
For an already configured operator machine, `bash deploy/publish-android.sh --if-needed` runs the same publisher.
`MYMIND_RELEASE_STORE` selects the existing keystore (default `~/.config/mymind/android-release.jks`),
`MYMIND_RELEASE_ALIAS` its alias, and `ANDROID_RELEASE_PASSWORD` its password. Production also requires the values
above except the base64 secret: the operator supplies the keystore path instead. `WRANGLER` can select an installed
`wrangler.js`; by default it uses the project's locked Worker dependency. `ANDROID_RELEASE_REVISION`, when set to a
full commit SHA, is recorded as `sourceCommit` in the manifest. The workflow sets it to the verified commit.

The publisher reads `mymind/downloads/manifest.json` from **live R2**, even on a fresh checkout without `downloads/`.
Missing, unreadable or malformed live manifests fail; this command does not initialize an empty production
manifest. It keeps every non-Android entry and all top-level metadata, replaces Android entries with the new
release and history, and uploads the APK **before** its manifest. It rereads the live manifest immediately before
replacing it and fails if another publisher changed it. It never enables an admin route or changes Worker secrets.
Existing app authentication still protects `/download`, `/download/android.json` and the APK.

Before publication, `keytool` validates the existing key/password/alias; `apksigner` verifies the APK and certificate,
and `aapt2` confirms application ID and changelog version. The test user then reads the hosted release description
and downloads the APK through the authenticated endpoint. Version, notes, history, URL, size and checksum must
match the **candidate**, and the downloaded APK must pass the same signing/version checks. A failed upload or
readback makes the job fail. A manifest-upload failure can leave an unlisted APK in R2; retrying is safe.

Without `--if-needed`, any existing equal/newer release is an error. With it, an equal version is accepted only
when the release name, notes and history agree and the hosted APK passes size, checksum, application ID, version
and pinned-certificate validation. It reports `Already published and verified` and performs no build or upload.
A downgrade, changed notes at the same version, invalid credentials or failed validation is an error. This allows
retries after publication succeeded but readback failed, and test-only commits to verify the existing release.

GitHub serializes Android release runs. Independent Linux/extension/manual publishers must also be coordinated:
Wrangler's R2 object upload has no compare-and-swap option, so the manifest reread detects intervening changes but
cannot atomically exclude a write between that read and the final upload. Do not run those publishers concurrently.

## Local verification

`python3 -B deploy/android-release-test.py` runs synthetic fixtures in temporary directories, with no build,
credentials, network or shared state. It exercises the real publisher and version gate with fake external tools.
If PyYAML is installed, it also parses both workflow files and checks their event/dependency/security structure;
otherwise that structural check is explicitly skipped. GitHub's event execution and the live R2 permissions still
need CI evidence.

`python3 -B deploy/android-release.py gate --base <base-sha> --head <head-sha>` checks committed revisions.
`bash deploy/publish-android.sh --local /tmp/android-downloads` builds and validates a signed APK, then writes a
local manifest while preserving other local downloads. It needs a keystore/password and Android SDK; it performs
no upload or sign-in. A synthetic key may be used **only for this local fixture**, never as a production replacement.
The production fingerprint is optional in local mode; if supplied, it is still enforced. `MYMIND_CHANGELOG`
and `GRADLE_ARGS` retain their local build overrides.

Repository tests and a green release job do not prove a phone installed the update. Confirm the authenticated
hosted release first, then the device's version and installed certificate. The app installs
only from its update screen, which shows over everything when the app opens while a newer release is offered; it
waits for uploads in flight to finish before installing.
