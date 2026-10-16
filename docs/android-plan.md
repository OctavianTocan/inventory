# Plan: a pixel-identical mymind Android app

Status: research only. Nothing below has been run against a phone or an APK yet.

## Decisions (the user, 2026-10-16)

- **A working app that matches the real app 1:1.** Both at once: it has to work, and every screen has to look
  exactly like mymind's Android app.
- **All components native Kotlin.** No WebView shell around mymind's pages; the UI is rebuilt in Kotlin. Parity is
  judged visually (screenshots of the real app against ours), so different code is fine.
- **Fully separate from mymind's servers.** Our app talks only to our own library (the hosted replica). Never fetch
  live mymind pages posing as the app or with its User-Agent. The real app on the phone or emulator is only the
  visual reference.
- The user turns on USB debugging on their phone later (step 0).

Consequences for the steps below: steps 0–4 (pull, decompile, run and capture the real app, point it at the replica
so both apps show the same data) still produce the reference; step 5 becomes a native Kotlin app instead of
WebViews. Open questions: how a native app signs in to the hosted library behind Cloudflare Access and mymind's
Google sign-in (Google refuses OAuth inside embedded WebViews; a native client needs an Android OAuth client or a
system-browser handoff), and how close native text rendering gets to the real app's WebView text at a pixel
threshold of 0.

## What the app is (evidence so far)

- Package `com.mymind.access`, v2.14 (code 280), min Android 12, target SDK 35, shipped as split APKs.
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
| Java | Temurin 21 through mise (Gradle's daemon uses it via `org.gradle.java.home`); pacman also pulled in OpenJDK 27 as Gradle's dependency, not first on `PATH` |
| `gradle` 9.8.1, `kotlin` 2.4.21 | pacman |
| `jadx` 1.5.6, `mitmproxy` 12.2.3 | pacman |
| `apktool` 3.0.3 | AUR `android-apktool-bin` |
| `frida` 17.23.3, `objection` 1.12.5 | `uv tool install frida-tools` / `objection`, only if WebView debugging is off or pinning blocks step 4 |
| `jadx-headless-mcp` 0.7.1 | `~/.local/share/rea/jadx-headless-mcp-0.7.1-all.jar`, SHA-256 matches REA's audited release; export `REA_JADX_MCP_JAR` to that path for REA's APK code commands |
| scrcpy, KVM, Docker | already present |

REA (`npx -y rea-agents@6.4.0`): `list-adb-devices`, `pull-adb-package`, `inspect-adb-package`, `capture-adb-screen`,
`inspect-adb-display`, `decode-android-resources`, `inspect-android-package`, `inspect-android-class`,
`inspect-android-method`, `compare-web-screenshots`.

## Plan, best first

### 0. On the phone (you, 5 min)

1. Settings → About phone → tap **Build number** 7 times.
2. Developer options → turn on **USB debugging** (or **Wireless debugging**, then `adb pair <ip:port>` with the code).
3. Plug in, accept the RSA prompt on the phone.

### 1. Pull the exact APK set you have installed (10 min)

```bash
rea list-adb-devices
rea inspect-adb-package <serial> com.mymind.access
rea pull-adb-package <serial> com.mymind.access --output-directory android/apk   # all splits, SHA-256 each
rea inspect-adb-display <serial>                                                   # size + density for the AVD
```

Plain adb equivalent: `adb shell pm path com.mymind.access`, then `adb pull` each path.

### 2. Decompile and read the shell (1–2 h)

```bash
apktool d android/apk/base.apk -o android/decoded        # manifest, layouts, drawables, fonts, Lottie, dimens
jadx -d android/jadx android/apk/base.apk                # Kotlin/Java
grep -rE "addJavascriptInterface|addDocumentStartJavaScript|evaluateJavascript|setUserAgentString|shouldInterceptRequest|mymind://" android/jadx
```

Outputs we need: the User-Agent string, each view's start URL, the injected bridge JS, the `mymind://` handler,
and every native screen's layout and resources.

### 3. Run the real app on the emulator and record it (1–2 h)

```bash
emulator -avd Pixel_9_API_36 -gpu host            # or a new AVD sized to the phone's display
adb -e install-multiple android/apk/*.apk
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
