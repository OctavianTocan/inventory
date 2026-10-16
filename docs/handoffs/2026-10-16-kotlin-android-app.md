---
artifact_contract: "ce-handoff/v1"
created_at: "2026-10-16T13:08:23Z"
title: "mymind native Kotlin Android app: phone connected, APK pulled, sign-in open"
summary: "Native Kotlin Android app matched 1:1 to mymind's app (roadmap phase 9): decisions recorded, toolchain on Java 27, phone on wireless debugging, APK set pulled; sign-in design and screenshot tolerance still to discuss with the user."
keywords: ["mymind", "android", "kotlin", "native", "apk", "adb", "rea", "pixel-parity", "sign-in", "cloudflare-access", "phase-9"]
cwd: "/home/octaviantocan/Projects/20-29_Personal_Products_and_Platforms/25_Experiments_and_Labs/25.03-mymind-reconstruction"
resume_focus: "Build the native Kotlin Android app (roadmap phase 9) per docs/android-plan.md, starting with plan step 2 and the two open questions"
repository: "github.com/OctavianTocan/mymind-reconstruction (private)"
repo_root_sha: "56f02615980e94ac90c0176f2be977d6f660708e"
branch: "main"
head: "129e3f526e400baaa8f27120afc681bb109993ae (this handoff is committed on top)"
---

# mymind native Kotlin Android app: handoff

The plan of record is `docs/android-plan.md`; this file only adds the session context around it. Read the plan's
"Decisions", "Open: how the app signs in" and "Open: how strict the screenshot match is" sections first.

## The user's decisions (their own answers, 2026-10-16)

1. **A working app that matches mymind's Android app 1:1**, both at once.
2. **Every component native Kotlin**, no WebView shell. The user asked whether parity is checked visually or by
   code; visual is what they want ("if we build them in Kotlin, they can still look exactly the same").
3. **Fully separate from mymind's servers.** "We do not care about the live pages. We are reconstructing the app
   from scratch." Never fetch live mymind posing as the app or with its User-Agent; the real app is only the visual
   reference.
4. **Leeway for native niceties, to be discussed.** "Leave a bit of tolerance for things because there are niceties
   to Kotlin native stuff that we would not get without it ... this needs to be discussed probably." Which screens
   or behaviours may differ, and by how much, is not agreed yet.
5. **Tools:** install the latest of everything needed; set the newest Java as the default "if that works fine".
6. **Handoff location:** committed to `main` of this repo, not `/tmp`.

Roadmap row 9 (`docs/roadmap.md`) records 1–3.

## Open questions for the user (not decided)

- **How the app signs in.** The user's own framing: they understand the token should not be built into the app and
  guess the app calls an API to get a token after signing in; they want to talk it through. Three options are in
  the plan (A: browser sign-in once in a Custom Tab, then the server mints a per-device Cloudflare service token plus
  an app session, handed over by one-time code and kept in the Android Keystore; B: a Cloudflare Bypass for an app
  API path plus native Google sign-in, which needs the user to relax AGENTS.md rule 7; C: pass the Access token
  through, re-sign-in when it expires). A is my recommendation, not the user's choice.
- **Screenshot tolerance.** The web replica is matched at channel threshold 0. The real app draws most screens as
  WebView pages; native Kotlin rasterizes text and anti-aliased edges differently, so exact pixel equality is
  unlikely even where a person sees no difference. The user did not know what my earlier "text-matching risk"
  question meant; it is this. It merges with decision 4.

The user also asked what the downloaded app talks to. Answer given: the desktop app is a window around the hosted
site, whose JavaScript calls the server's existing HTTP API (`lib/`, `docs/write-api.md`); the Kotlin app will call
that same API. The only Android-specific gap is that card bodies reach the web as server-rendered HTML
(`lib/render-card.mjs`), so the app renders them natively from card JSON.

## Current state

| Piece | State |
|---|---|
| Android code | Not started. No Gradle project in the repo. |
| Phone | Samsung foldable on Android 17, wireless debugging on, paired with mini-001 (plan "The phone"). Reconnect with `adb mdns services`, then `adb connect <ip:port>`. |
| mymind APK set | Pulled to `android/apk/com.mymind.access/` (six splits; machine-local and Git-ignored, never commit or share). Not decompiled yet (plan step 2). |
| Reference captures | None yet (plan steps 3–4). |
| Toolchain | Installed and version-checked (plan "Tools on this machine"). Java default is Temurin 27 (mise global plus `org.gradle.java.home` in `~/.gradle/gradle.properties`); a throwaway probe app with AGP 9.4.1 and built-in Kotlin built with it. REA's jadx JAR is hash-verified but unused so far. |
| Hosted library | `https://mymind.octaviantocan.com` on mini-002 behind Cloudflare Access app "mymind" plus mymind's own sign-in; `node tools/check-hosted.mjs` passed 9/9 at c7ac205. |
| /download | Linux desktop build, for anyone with a mymind session on the hosted site (c7ac205). |

## How parity is proven today

`tools/drive.mjs` runs scripted steps against live and replica, screenshots each state and compares pairs with
`rea compare-web-screenshots` at channel threshold 0 (`tools/drive.mjs:241`, summary at `:322`). For Android the
plan uses `rea capture-adb-screen` on both apps showing the same data (plan step 4 points the unmodified real app at
our replica through mitmproxy) and the same compare, with the tolerance still to agree.

## References

- `docs/android-plan.md`: decisions, phone, API, sign-in options, evidence about the real app (six native views, 46
  `mymind.native.*` bridge calls, `mymind://` image scheme), steps 0–5, tools, risks.
- `desktop/src-tauri/src/main.rs:185-280`: how the desktop shell lets Cloudflare Access and Google sign-in pages
  load for the hosted library; the same two-gate problem the Android app has.
- `tools/check-hosted.mjs`: reaching the hosted site with the Access service token (sent only to that origin) and
  signing in as the test user.
- `deploy/download-server.mjs`: the Access-token plus `mm_session` checks a new app endpoint would follow.
- `AGENTS.md`: hard rules 1 (never change the mymind account), 4 (no bot-detection workarounds), 6 (no replica-code
  edits during drive runs), 7 (nothing public without sign-in); delegation table; long-running process rules.
- `README.md`: "Sign-in (remote access)", "Hosting", "Testing the hosted library".

## Machine-local state (mini-001)

- Repo `.env` holds `CF_ACCESS_CLIENT_ID/SECRET` (service token "mymind-bypass"), `TEST_USER_NAME/PASSWORD` and
  Google client values. Never print or commit them.
- `REA_JADX_MCP_JAR=~/.local/share/rea/jadx-headless-mcp-0.7.1-all.jar` must be exported for REA's APK code
  commands; it is in no shell profile.
- Verification run `mymind-verify-merge` (systemd user unit, started 14:30 CEST, log
  `evidence/drive/replica-merge.log`) was still running at 15:08 CEST; every finished suite was fully identical so
  far (desktop-dark-ui had 5 step errors). Until it ends, do not edit `server.mjs`, `tools/replay-store.mjs` or
  `lib/**/*.mjs`. The previous session will report it and run `npm run coverage`.
- Shell: zsh; stop processes by PID, never `pkill -f`; foreground `sleep` is blocked.

## Plausible path

1. Plan step 2: decompile the pulled APK (apktool, jadx) and record the shell's views, layouts, fonts, colours and
   network client in `docs/android-plan.md`.
2. Settle sign-in and tolerance with the user, then write a detailed build plan (for example with `ce-plan`).
3. Plan steps 3–4 on the emulator for reference captures, then build screens in Kotlin on a branch, each proven
   against the real app's screenshot.
