# Plan: a pixel-identical mymind Android app

Status (2026-10-16): steps 0 and 1 are done (the phone is reachable over wireless debugging and the installed APK set
is pulled). Nothing is decompiled or built yet.

## Decisions (the user, 2026-10-16)

- **A working app that matches the real app 1:1.** Both at once: it has to work, and every screen has to look
  exactly like mymind's Android app.
- **All components native Kotlin.** No WebView shell around mymind's pages; the UI is rebuilt in Kotlin. Parity is
  judged visually (screenshots of the real app against ours), so different code is fine.
- **Fully separate from mymind's servers.** Our app talks only to our own library (the hosted replica). Never fetch
  live mymind pages posing as the app or with its User-Agent. The real app on the phone or emulator is only the
  visual reference.
- **Leeway (agreed).** Layout, colours and sizes match exactly; tiny differences in text and edge anti-aliasing are
  allowed; each native extra (predictive back gesture, haptics, system share sheet, …) gets the user's yes or no one
  at a time before it ships.
- **Native sign-in, no Cloudflare Access anywhere.** A sign-in screen in the app with one-tap Google sign-in; Access
  is dropped for the whole host (the tunnel stays). Welcome: the owner's Google account, any Google account with a
  verified `@octaviantocan.com` address, and the test user (password), each with their own library. Nobody else, and
  no sign-up form. Details in "Sign-in" below.
- **Structured card data for the app.** The app gets cards as typed JSON, not web markup (see "What our app talks
  to").

Consequences for the steps below: steps 2–4 (decompile, run and capture the real app, point it at the replica so
both apps show the same data) produce the reference; step 5 is a native Kotlin app instead of WebViews.

## The phone

Samsung SM-F976B (foldable; folded, the cover screen is 1080x2520 at density 480, overridden to 450), Android 17
(API 37), with mymind 2.14 (code 280) from Play. It is on wireless debugging and already paired with the development
machine: `adb mdns services` lists it, then `adb connect <ip:port>` with the address shown (the port changes).
`android/apk/com.mymind.access/` holds the six pulled APKs (base, arm64_v8a, xxhdpi, en, es, ro; base.apk SHA-256
`8d6e6b06fdbd7413e82f49d50cada0c9eaf8a8073305362aaef01f281cf0e69a`), Git-ignored like everything decoded from
them.

## What our app talks to

The same server and library as the web front-end and the desktop app (a window around the hosted site): cards,
spaces, tags, search, writes, uploads, `/changes` and live updates on the `/hub` WebSocket (`docs/write-api.md`,
`lib/`).

Card content is already stored as structured data: each card's JSON (`data`, prose as a document tree) plus its
source metadata (`extras`, documented on `renderCard` in `lib/render-card.mjs`). The server turns that into HTML only
because mymind's own web code, which the replica serves unchanged for pixel parity, understands nothing else
(`lib/render-card.mjs`, `lib/render/views.mjs`). The app gets a typed JSON form of the same data from its own API
(below) and draws it natively: one source, two renderers. Define that JSON once (for example `docs/app-api.md`) so
the Kotlin models are generated or checked against it.

## Sign-in (decided 2026-10-16)

The user's choice: a native sign-in screen with one-tap Google sign-in, and no Cloudflare Access at all (the tunnel
stays). Option A (browser sign-in, then per-device Cloudflare tokens), C (passing the Access token through) and an
Access exemption for an app path were rejected. Accounts, per-user libraries and removing Access are planned in
`docs/accounts-plan.md`; the app builds on them.

- **Who can sign in (the user):** "the test user, my main email, and then my domain. Anyone from those is completely
  welcome." Google for `OWNER_EMAIL` and any verified `@octaviantocan.com` address, passwords only for users made
  with `tools/auth-user.mjs` (the `TEST_USER_*` account for automated tests). No sign-up form. Each person gets their
  own library.
- **In the app:** Android Credential Manager with Sign in with Google, `serverClientId` set to the existing Web client
  ID. Google also needs an Android OAuth client in the same Google Cloud project for the app's package name and the
  SHA-1 of each signing key (debug and release). The user authorized the agent to do this Google Cloud setup itself,
  the way the Web client was set up (the agent browser, signed in as the owner). A password form serves the test
  user.
- **On the server:** a sign-in endpoint takes the Google ID token and verifies it against Google's published keys
  (`aud` = Web client ID, issuer, expiry, `email_verified`, allowed email; `verifyIdToken` in `lib/auth.mjs` already
  does all but the domain check), or takes a password user's name and password with rate limiting, and returns an
  opaque app token stored hashed and revocable like `sessions`. Every other route accepts `Authorization: Bearer
  <token>` as well as the session cookie, including media. The app keeps the token encrypted with an Android
  Keystore key.

## Screenshot match

The web replica is matched at REA channel threshold 0: every pixel identical. The real app draws most screens as web
pages inside WebViews, while native Kotlin draws text and anti-aliased edges with Android's own renderer, so a native
screen can look identical to a person and still differ in edge pixels. Per the leeway decision, a screen matches when
layout, colours and sizes are identical and only text and edge anti-aliasing differ. Pick the compare tolerance that
encodes this (a channel threshold and a ratio of differing pixels, or a check that differences lie only along glyph
and shape edges), prove it on a known-good pair, and record it here.

## What the app is (evidence so far)

- Package `com.mymind.access`, v2.14 (code 280), min SDK 32 (Android 12L), target SDK 35, shipped as split APKs.
- It is a **native shell around WebViews that load the same server-rendered pages and ES modules as the web app.**
  The web bundle already contains the Android side of that contract:
  - `scripts/native/api.js` exposes `mymind.api` (`ui.update`, `cards.open`, `synchronize`, `objects.*`, `query/find`)
    for native code to call, and forwards messages between WebViews (`forwardMessage({target:'CardView'})`).
  - The page calls 46 `mymind.native.*` methods (`openCard`, `closeCard`, `openSheet('addToSpace')`, `openMenu`,
    `openImageViewer`, `openPdfViewer`, `chooseFile`, `generateFeedback`, `writeToClipboard`, `openView`, `signout`, …)
    and `mymind.bridge.sendMessage` (`app:idle`, `view:loaded`, `view:open`, `synchronizer:error`). The web bundle never
    defines `mymind.native`, `mymind.bridge` or `mymind.viewName`: the shell injects them.
  - Six native views: MainView, CardView, SettingsView, CollectionsView, EditorView, NewSpaceView.
  - The server picks the HTML by User-Agent (`<body class="android …">`, `mymind.config.platform = 'Android'`).
  - Card images use a `mymind://` scheme that the shell serves from a native image cache.
- Unknown until the APK is decompiled: the shell's language (Kotlin is likely), its exact User-Agent, the injected
  bridge script, and how much chrome (toolbar, sheets, menus, image/PDF viewers) is native.

Consequence: most of the Android UI is already reconstructed. The web replica plus the app's User-Agent renders the
same HTML/JS/CSS the WebViews show, and the verifier already proves phone rendering pixel-identical (`iphone` and
`android` profiles in `npm run verify`). What is left is the native shell: chrome, sheets, viewers, the bridge, and
the image cache.

Not pursued (the user's decision above): an `android-app` profile in `tools/drive.mjs` that records live mymind with
the app's User-Agent.

## Tools on this machine

Installed 2026-10-16 on the development machine (mini-001):

| Tool | Version, source |
|---|---|
| Android SDK `~/Android/Sdk` | platforms 36 and 37, build-tools 36.0.0/37.0.0, emulator, platform-tools, cmdline-tools |
| AVDs | `Pixel_9_API_36` (google_apis, rootable), `fold`, `latest`, `pulls_demo` |
| Java 27 (default) | Temurin 27 through mise (global) and in `~/.gradle/gradle.properties` (`org.gradle.java.home`), OpenJDK 27 as the system `java`. A probe app (AGP 9.4.1, built-in Kotlin, compileSdk 36) built with it. Temurin 21 stays installed |
| `gradle` 9.8.1, `kotlin` 2.4.21 | pacman; latest stable Android Gradle Plugin is 9.4.1 |
| `jadx` 1.5.6, `mitmproxy` 12.2.3 | pacman |
| `apktool` 3.0.3 | AUR `android-apktool-bin` |
| `frida` 17.23.3, `objection` 1.12.5 | `uv tool install frida-tools` / `objection`, only if WebView debugging is off or pinning blocks step 4 |
| `jadx-headless-mcp` 0.7.1 | `~/.local/share/rea/jadx-headless-mcp-0.7.1-all.jar`, SHA-256 matches REA's audited release; export `REA_JADX_MCP_JAR` to that path for REA's APK code commands |
| scrcpy, KVM, Docker | already present |

REA (`npx -y rea-agents@6.4.0`): `list-adb-devices`, `pull-adb-package`, `inspect-adb-package`, `capture-adb-screen`,
`inspect-adb-display`, `decode-android-resources`, `inspect-android-package`, `inspect-android-class`,
`inspect-android-method`, `compare-web-screenshots`.

## Plan, best first

### 0. On the phone (done 2026-10-16)

Wireless debugging is on and the machine is paired. Reconnect after a phone restart or network change:

```bash
adb mdns services              # lists _adb-tls-connect with the current ip:port
adb connect <ip:port>
```

### 1. Pull the exact APK set you have installed (done 2026-10-16)

```bash
rea list-adb-devices
rea inspect-adb-package <serial> com.mymind.access
rea pull-adb-package <serial> com.mymind.access --output-directory android/apk   # all splits, SHA-256 each
rea inspect-adb-display <serial>                                                   # size + density for the AVD
```

Plain adb equivalent: `adb shell pm path com.mymind.access`, then `adb pull` each path.

### 2. Decompile and read the shell (1–2 h)

```bash
apktool d android/apk/com.mymind.access/base.apk -o android/decoded        # manifest, layouts, drawables, fonts, Lottie, dimens
jadx -d android/jadx android/apk/com.mymind.access/base.apk                # Kotlin/Java
grep -rE "addJavascriptInterface|addDocumentStartJavaScript|evaluateJavascript|setUserAgentString|shouldInterceptRequest|mymind://" android/jadx
```

Outputs we need: the User-Agent string, each view's start URL, the injected bridge JS, the `mymind://` handler,
and every native screen's layout and resources.

### 3. Run the real app on the emulator and record it (1–2 h)

```bash
emulator -avd Pixel_9_API_36 -gpu host            # or a new AVD sized to the phone's display
adb -e install-multiple android/apk/com.mymind.access/*.apk
```

On `google_apis` (userdebug) images every WebView is inspectable:
`adb forward tcp:9223 localabstract:webview_devtools_remote_$(adb shell pidof com.mymind.access)`.
Then the existing tooling works unchanged: `tools/drive.mjs` / `Side` can attach to `http://127.0.0.1:9223`
(Playwright `connectOverCDP`) to record what each WebView loads into `replay/` under the Android variant, and
`rea capture-adb-screen` captures the full screen including native chrome. If debugging is off: `adb root` and
Frida (`WebView.setWebContentsDebuggingEnabled(true)`).

### 4. Point the real app at the replica (half a day)

Start the emulator with `-http-proxy 127.0.0.1:8080`, run `mitmdump` with a small script that routes
`access.mymind.com` and `static.accelerator.net` to `localhost:4321`, and trust the mitm CA (system store on API 34+,
or disable pinning with objection). The unmodified app then renders from our replica: pixel-identical by
construction, and the reference for step 5.

### 5. Build our own app in native Kotlin

Every screen rebuilt in Kotlin from the captures and decoded resources (layouts, fonts, colours, dimens, Lottie),
reading our library's API. Superseded idea: six WebViews loading the replica with the recovered User-Agent and bridge.
Verify the same way as the web: freeze the status bar (demo mode), animator scales 0, the same library data on both
sides (step 4), `rea capture-adb-screen` on our app and on the real app for every screen, then
`rea compare-web-screenshots`.

### Emulator or phone?

Emulator for steps 3–5: root, inspectable WebViews, fixed resolution, easy interception. The phone is for pulling the
APK and taking reference screenshots (`rea capture-adb-screen`); an unrooted phone with a release build blocks DevTools
and traffic interception.

## Risks

- Play licensing / Play Integrity may refuse an emulator without Play Store; fallback is a Play Store image (not
  rootable) or the phone.
- Login on the emulator may need an email code.
- Certificate pinning on the native client is unknown until step 2.
- Re-signed APKs break licensing, billing and sign-in; keep any patched build for local study.
- mymind's APK, code and assets are theirs: don't redistribute them, and check their terms before going further.
