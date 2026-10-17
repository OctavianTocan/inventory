# Android app downloads, self-updates and changelogs

The user, 2026-10-17: `/download` should also serve the mobile app so it is easy to install, the app should update
itself, and every new version must come with a changelog, so the user gets each version quickly.

Starts after Android round 3 (haptics, pull to refresh, uploads, Top of Mind, Serendipity) is reviewed and committed.

## What exists

- `/download` lists the packages in `downloads/manifest.json` (`{library, files: [{name, label, bytes, sha256,
  built}]}`) and serves `/download/<name>`, only with a session.
  - Production: the Worker reads R2 `downloads/` (`rust/crates/worker/src/lib.rs`, `download`). The sign-in gate
    in front of it is the Auth Durable Object.
  - Standalone server: `rust/crates/server/src/download.rs` checks only the `mm_session` cookie.
  - The page is `rust/crates/core/src/download_page.rs`, written for the Linux desktop tarball. Every file is sent
    as `application/gzip`.
- The desktop package is made by `deploy/package-app.sh`, which writes the manifest entry. Files reach R2 through
  `upload-hosted bucket --prefix downloads` and the Worker's `/_admin/bucket/` route (`ADMIN_TOKEN`).
- The app: `com.octaviantocan.mymind`, `minSdk 32`, `versionCode 1`, `versionName 0.1.0`. Only debug builds exist,
  signed with mini-001's `~/.android/debug.keystore`. Sign in with Google needs an Android OAuth client for each
  signing key's SHA-1; only the debug key has one ("mymind Android (debug, mini-001)" in `mymind-replica`).

## Decisions

1. **One release key.** Generate `~/.config/mymind/android-release.jks` outside the repository, with a random
   password saved in `.env` (`ANDROID_RELEASE_PASSWORD`; a PKCS12 keystore uses one password for the store and the
   key). Made 2026-10-17: alias `mymind`, RSA 4096, valid until 2126, certificate SHA-1
   `EE:DC:47:AA:A2:AE:1C:B8:AB:A2:01:B5:65:02:71:E0:7D:18:C1:15`.
   - Android installs an update only when it is signed with the same key, so every published build uses this key.
   - Losing the key means the next version needs an uninstall. The user should keep a copy somewhere safe.
   - Add a second Android OAuth client for its SHA-1, the same way the debug client was made (the user authorized
     that Google Cloud setup on 2026-10-16).
2. **The version comes from the changelog.** `android/CHANGELOG.md` holds the entries, newest first:

   ```markdown
   ## 0.2.0 (2), 2026-10-18
   - Downloads and installs new versions by itself.
   ```

   - Gradle reads `versionName` and `versionCode` from the top entry.
   - The build fails when the top entry is malformed or has no bullet, or when its `versionCode` is not higher than
     the next entry's.
   - So there is no way to raise the version without writing its changelog. The file also goes into the APK as an
     asset for the app's own changelog screen.
3. **The server knows about the Android package.**
   - Manifest entries gain `platform` (`android`), `versionName`, `versionCode`, `notes` (that version's changelog
     bullets) and `history` (every version's `versionName`, `versionCode`, `date` and `notes`, newest first, from
     the changelog). An entry without `platform: "android"` stays a Linux tarball.
   - `/download/<name>` sends an Android entry as `application/vnd.android.package-archive`, any other as
     `application/gzip`. The type follows from `platform`, so the manifest carries no content type.
   - The page lists the Android app with its version, size, sha256 and the latest notes, plus one line on allowing
     installs from the browser the first time. The Linux section stays as it is.
   - `GET /download/android.json` answers `{versionName, versionCode, url, bytes, sha256, built, notes, history:
     [{versionName, versionCode, date, notes}]}`, with `url` the package's path on the same server.
   - Only the newest APK is kept, because Android cannot downgrade without an uninstall. The history keeps every
     version's notes.
4. **The app's token works on `/download`.** The updater calls `/download/android.json` and the APK with
   `Authorization: Bearer <app token>`.
   - In the Worker, check that the Auth gate already accepts it; add it if not. (Checked: `Auth::handle` already
     lets an app token through on every path, `/download` included.)
   - In `download.rs`, accept an unexpired app token (`app_tokens`, hashed like sessions) alongside the cookie.
   - This changes sign-in code, so it gets a Codex read-only review before it ships (AGENTS.md rule 7).
5. **The updater.**
   - **When it checks:** on launch, on return to the app when the last check is over an hour old, and every 6 hours
     in the background with WorkManager.
   - **Download:** a newer `versionCode` is downloaded in the background into the app's cache with the app token,
     then checked against `bytes` and `sha256`. A mismatch is thrown away and retried at the next check.
   - **Install:** through a `PackageInstaller` session.
     - The app declares `REQUEST_INSTALL_PACKAGES` and `UPDATE_PACKAGES_WITHOUT_USER_ACTION`, and asks for
       `USER_ACTION_NOT_REQUIRED`, so Android can install without a prompt where it allows (the app updating itself,
       Android 12 and later).
     - Where Android still asks, the system prompt opens in the foreground. In the background, a notification
       carries the version, the notes and an Install action.
     - The first install from the browser always needs the user's tap, plus allowing installs from that browser once.
   - **Notes in the app:**
     - in the foreground, a sheet "What's new in 0.3.0" shows the notes before installing, with Install and Later;
     - after an update, the new version's notes show once;
     - Later lasts until the next launch, because the user wants every version quickly.
   - **Settings:** shows "Version 0.2.0 (2)", "Check for updates" and the full changelog from the bundled asset.
   - **Look:** these screens are ours, not mymind's, so they use the app's existing theme and components, plainly.
6. **Publishing: `deploy/publish-android.sh`.**
   1. It refuses when the top changelog `versionCode` is not higher than the published one.
   2. It builds `assembleRelease` with the release key from `.env`.
   3. It checks the APK's certificate with `apksigner verify --print-certs` against the release key.
   4. It writes the APK and the manifest entry to the downloads directory and uploads `downloads/` to R2.
   5. It reads `/download/android.json` back as `TEST_USER` and compares the version and sha256.

   Publishing a version is then one command. It is still run only on request, like a deploy.

## Proof

- **Rust:**
  - page, JSON and content-type tests;
  - the gate tests: the app token is accepted on `/download`, and no credential, a revoked token and a cookie
    from another site are refused, in the Worker's gate and in `download.rs`;
  - clippy and the Wasm checks.
- **Android:**
  - Robolectric tests for changelog parsing and the build check, version comparison, a sha256 mismatch, and the
    updater's states (none, downloading, ready, installing, failed);
  - Roborazzi shots of the update sheet, the notes after an update, and the Settings rows, in light and dark.
- **Emulator, end to end, against a local server.** Locally `/download` is the separate `mymind download-server`
  (its own port, the same `AUTH_DB`), not `mymind serve`, so the test build gets a `mymindDownloads` Gradle property
  for the updater's server; it defaults to `mymindServer`, which in production serves both.
  1. install 0.2.0 from the local `/download` page;
  2. publish 0.2.1 locally;
  3. the app downloads it, shows its notes and installs it, and reopens on 0.2.1 with the notes shown once.
- **Production:**
  - publish the first release;
  - `check-hosted` sees the Android entry, and the APK downloads as `TEST_USER` with a matching sha256;
  - Google sign-in works in the release build.

## Status

- 2026-10-17: step 1 is built. `mm_core::download_page` has `android_release`, `content_type` and the page's Android
  section; `mymind download-server` and the Worker serve `/download/android.json` and the APK type, and the
  download server accepts app tokens. Tests: `rust/crates/core/tests/download_android.rs` (page, JSON, type, the
  gate) and `rust/crates/server/tests/download_android.rs` (the token, expired, revoked and unknown credentials).
- 2026-10-17: steps 2 and 3 are built in `android/` and `deploy/publish-android.sh`. Unit tests 197, buildSrc tests 9,
  `assembleDebug` and `assembleRelease` (scratch key) pass; an emulator run updated 0.2.0 to 0.2.1 (foreground sheet)
  and to 0.2.2 (background notification action), silently both times. `publish-android.sh` without `--local` was never
  run. Changes from the plan: an extra `UpToDate` state, and the release build picks its network security config
  (`network_strict`, or `network_loopback` when pointed at an `http://` server).

## Order

1. The server side (main session, then a Codex review of the gate change), committed and deployed.
2. The release key and its Google OAuth client.
3. The changelog-driven version and the updater (the Android agent's next round), reviewed and committed.
4. The emulator end-to-end run, then the first production publish.
5. The user's one manual step: open `/download` on the phone, install, and allow installs from the browser once.
   Every later version arrives on its own.

Not included: an updater for the Linux desktop app (not asked for).
