# Plan: a pixel-identical mymind Android app

Status: research only. Nothing below has been run against a phone or an APK yet.

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

Fastest next step once the APK is decompiled: add an `android-app` profile to `tools/drive.mjs` with the app's exact
User-Agent (and the injected bridge stubbed), so the replica records and serves the HTML mymind sends the app itself.

## Tools on this machine

| Have | Missing (install when we start) |
|---|---|
| Android SDK in `~/Android/Sdk` (adb 37, emulator 37, build-tools 36/37 with apksigner, zipalign, aapt2) | `jadx` (`sudo pacman -S jadx`) |
| AVDs `Pixel_9_API_36` (google_apis, rootable) and `fold` | `apktool` (`yay -S android-apktool-bin`) |
| KVM (`/dev/kvm`, AMD SVM), 30 GB RAM, 600+ GB free | `mitmproxy` (`sudo pacman -S mitmproxy`) |
| scrcpy, Java 21, Docker | `frida-tools` (`yay -S frida-tools`), only if WebView debugging is off |
| REA: `pull-adb-package`, `inspect-adb-package`, `capture-adb-screen`, `inspect-adb-display`, `start-adb-app`, `read-adb-logcat`, `decode-android-resources`, `inspect-android-package`, `search-android-classes`, `inspect-android-method` | REA's APK code commands also need the `jadx-headless-mcp` JAR via `REA_JADX_MCP_JAR` |

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

### 5. Build our own shell (2–5 days)

A small Kotlin app: six WebViews loading `http://10.0.2.2:4321` with the recovered User-Agent, the recovered bridge
(`mymind.native`, `mymind.bridge`, `mymind.viewName`), a `mymind://` request interceptor backed by `replay/`, and the
native chrome rebuilt from the decoded resources. Verify the same way as the web: freeze the status bar (demo mode),
animator scales 0 or a fixed virtual clock in the WebViews, `rea capture-adb-screen` on our app and on the real app
(step 4) for every screen, then `rea compare-web-screenshots`.

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
