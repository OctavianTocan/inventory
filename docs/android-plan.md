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
- **Some leeway, still to be agreed.** The user wants room for things native Kotlin does better than the original.
  Which screens or behaviours may differ, and by how much, is to be discussed with the user before building them.

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

The server's existing HTTP API, the same one the web front-end and the desktop app (a window around the hosted site)
already use: the card list and card details, spaces, tags, search, writes, uploads, `/changes` and live updates on
the `/hub` WebSocket (`docs/write-api.md`, `lib/`). The data side needs nothing new. The web gets card bodies as
HTML that `lib/render-card.mjs` renders from card JSON; the app draws them natively from the JSON instead.

## Open: how the app signs in (to discuss with the user)

Every request meets two gates: Cloudflare Access at the edge, then mymind's own session (Google for the owner,
password users). Google refuses sign-in inside embedded WebViews, and a service token compiled into an APK can be
extracted, so the app has to obtain its credentials after the person signs in. Options:

- **A. Sign in once in the browser, then per-device tokens (recommended).** The app opens the sign-in in a Custom
  Tab, a real browser where Google allows it: Cloudflare Access, then mymind's sign-in. The server then creates a
  Cloudflare service token for that device (Cloudflare API) and a mymind session, and hands both to the app through
  a one-time code on an app link; the app keeps them in the Android Keystore and sends them with every request. Each
  device can be revoked on its own, and the outer Access wall stays whole. Costs: the server holds a Cloudflare API
  token that can edit Access, and that first sign-in shows a browser page.
- **B. Open only the app's API path at the edge.** A Cloudflare Bypass policy for something like `/api/app/*`; the
  app signs in natively (Android Credential Manager for Google, or a password) and the server checks the Google ID
  token and issues an app session. Fully native and 1:1, but that path is reachable from the internet with only our
  own sign-in in front of it, which AGENTS.md rule 7 forbids unless the user agrees.
- **C. Pass the Access token through.** Like A, but the server hands the app the Access token from the browser
  sign-in and the app sends it as `cf-access-token`. No Cloudflare API credential on the server; the person signs in
  again whenever the Access session expires.

## Open: how strict the screenshot match is

The web replica is matched at REA channel threshold 0: every pixel identical. The real app draws most screens as web
pages inside WebViews, while native Kotlin draws text and anti-aliased edges with Android's own renderer, so a native
screen can look identical to a person and still differ in edge pixels. Agree with the user what tolerance counts as a
match, per screen if needed; it goes together with the leeway decision above.

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
