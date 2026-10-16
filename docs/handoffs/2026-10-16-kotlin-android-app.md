---
artifact_contract: "ce-handoff/v1"
created_at: "2026-10-16T13:08:23Z"
title: "mymind native Kotlin Android app: phone connected, APK pulled, design decided"
summary: "Native Kotlin Android app matched 1:1 to mymind's app (roadmap phase 9): toolchain on Java 27, phone on wireless debugging, APK set pulled; user decided native Google sign-in without Cloudflare in the app, typed JSON card data, and the screenshot leeway."
keywords: ["mymind", "android", "kotlin", "native", "apk", "adb", "rea", "pixel-parity", "sign-in", "cloudflare-access", "phase-9"]
cwd: "/home/octaviantocan/Projects/20-29_Personal_Products_and_Platforms/25_Experiments_and_Labs/25.03-mymind-reconstruction"
resume_focus: "Build the native Kotlin Android app (roadmap phase 9) per docs/android-plan.md, starting with plan step 2 and the items under Still open"
repository: "github.com/OctavianTocan/mymind-reconstruction (private)"
repo_root_sha: "56f02615980e94ac90c0176f2be977d6f660708e"
branch: "main"
head: "9790df192f8ef0adc20196433789292c5398ed4a (this update is committed on top)"
---

# mymind native Kotlin Android app: handoff

The plan of record is `docs/android-plan.md`; this file only adds the session context around it. Read the plan's
"Decisions", "What our app talks to", "Sign-in" and "Screenshot match" sections, and `docs/accounts-plan.md`, first. Updated after the user
answered the open questions (same day); `git log -p` on this file shows the earlier version.

## The user's decisions (their own answers, 2026-10-16)

1. **A working app that matches mymind's Android app 1:1**, both at once.
2. **Every component native Kotlin**, no WebView shell. The user asked whether parity is checked visually or by
   code; visual is what they want ("if we build them in Kotlin, they can still look exactly the same").
3. **Fully separate from mymind's servers.** "We do not care about the live pages. We are reconstructing the app
   from scratch." Never fetch live mymind posing as the app or with its User-Agent; the real app is only the visual
   reference.
4. **Leeway for native niceties, agreed.** The user asked for tolerance for Kotlin-native niceties, then accepted my
   proposal: layout, colours and sizes match exactly; tiny text-edge differences are allowed; each native extra
   (back gesture, haptics, …) gets their yes or no one at a time.
5. **Tools:** install the latest of everything needed; set the newest Java as the default "if that works fine".
6. **Handoff location:** committed to `main` of this repo, not `/tmp`.
7. **Sign-in: native Google in the app, no Cloudflare in the app.** "I don't think the app should really need
   Cloudflare sign-in ... We just restrict the actual sign-in through Google and account creation to only me and the
   test user." Moments later they widened it: "We have the test user, my main email, and then my domain. Anyone
   from those is completely welcome", so any verified `@octaviantocan.com` Google account too, on the app and the
   website. They want a native screen with one-tap Google sign-in.
9. **No Cloudflare Access at all; the tunnel stays.** "We can actually fully drop Cloudflare ... in terms of access.
   We are still using it as a tunnel thing." The server's sign-in is the only gate; AGENTS.md rule 7 says so.
10. **A library per person** in a proper multi-user database, planned before the Kotlin app or in parallel. "The
    test user would actually just get the library that I have right now, just one-to-one", and the parity checks
    run as the test user. Re-verifying after `lib/` changes can wait; it is no reason to hold a change.
11. **Google Cloud setup by the agent**, "the same way you did the other setup" (agent browser signed in as the
    owner): the Android OAuth client, and test users or publishing for domain accounts.
8. **Card data:** asked why the app should get web markup at all; they want a format both apps render properly.
   Recorded as typed JSON for the app from the same stored data the web's HTML is made from (the web keeps HTML only
   because mymind's own front-end code requires it).
12. **A plain public sign-in page.** Asked what signed-out visitors should see once the hostname is public, the user
    accepted my proposal ("Yeah, that sounds good. Let's do that."): a plain page of our own through the tunnel,
    mymind's recorded page kept for loopback and verification. Built and deployed, and Access removed
    (`docs/accounts-plan.md`, "Done: removing Cloudflare Access").

Roadmap row 9 (`docs/roadmap.md`) records 1–3; `docs/accounts-plan.md` records 7 and 9–12.

## Still open

- **A library per person** (`docs/accounts-plan.md`, "Not built"). Until it exists, Google sign-in admits only
  `OWNER_EMAIL`; domain accounts are widened at the same time.
- **The numeric screenshot tolerance** that encodes the agreed leeway; to be chosen and proven on a known-good pair
  (plan "Screenshot match").

The user also asked what the downloaded app talks to. Answer given: the desktop app is a window around the hosted
site, whose JavaScript calls the server's existing HTTP API (`lib/`, `docs/write-api.md`); the Kotlin app uses the
same server, with a bearer token instead of the session cookie.

## Current state

| Piece | State |
|---|---|
| Android code | Not started. No Gradle project in the repo. |
| Phone | Samsung foldable on Android 17, wireless debugging on, paired with mini-001 (plan "The phone"). Reconnect with `adb mdns services`, then `adb connect <ip:port>`. |
| mymind APK set | Pulled to `android/apk/com.mymind.access/` (six splits; machine-local and Git-ignored, never commit or share). Not decompiled yet (plan step 2). |
| Reference captures | None yet (plan steps 3–4). |
| Toolchain | Installed and version-checked (plan "Tools on this machine"). Java default is Temurin 27 (mise global plus `org.gradle.java.home` in `~/.gradle/gradle.properties`); a throwaway probe app with AGP 9.4.1 and built-in Kotlin built with it. REA's jadx JAR is hash-verified but unused so far. |
| Hosted library | `https://mymind.octaviantocan.com` on mini-002, public through the tunnel with the server's sign-in as the only gate (owner-only Google, password users); signed-out visitors get the plain page. `node tools/check-hosted.mjs --browser` passed 11/11 at 9790df1. |
| Accounts | One shared library; per-person libraries are planned in `docs/accounts-plan.md`, not built. Cloudflare Access is deleted. |
| Verification | Coverage is stale since 9790df1 changed `lib/auth.mjs`; the user said re-running can wait. `npm run coverage` lists the runs to redo. |
| /download | Linux desktop build, for anyone with a mymind session on the hosted site (c7ac205). |

## How parity is proven today

`tools/drive.mjs` runs scripted steps against live and replica, screenshots each state and compares pairs with
`rea compare-web-screenshots` at channel threshold 0 (`tools/drive.mjs:241`, summary at `:322`). For Android the
plan uses `rea capture-adb-screen` on both apps showing the same data (plan step 4 points the unmodified real app at
our replica through mitmproxy) and the same compare, with the tolerance still to agree.

## References

- `docs/android-plan.md`: decisions, phone, API, sign-in options, evidence about the real app (six native views, 46
  `mymind.native.*` bridge calls, `mymind://` image scheme), steps 0–5, tools, risks.
- `desktop/src-tauri/src/main.rs:185-280`: how the desktop shell lets the Google sign-in pages load for the hosted
  library. Its Cloudflare Access handling (`is_access_login`) is now unused and harmless.
- `lib/auth.mjs`: the sign-in gate the app's bearer-token endpoint will join, including the plain public page.
- `tools/check-hosted.mjs`: signs in to the hosted site as the test user, no Cloudflare token.
- `deploy/download-server.mjs`: the `mm_session` check.
- `AGENTS.md`: hard rules 1 (never change the mymind account), 4 (no bot-detection workarounds), 6 (no replica-code
  edits during drive runs), 7 (nothing public without sign-in); delegation table; long-running process rules.
- `README.md`: "Sign-in (remote access)", "Hosting", "Testing the hosted library".

## Machine-local state (mini-001)

- Repo `.env` holds `TEST_USER_NAME/PASSWORD`, `OWNER_EMAIL`, the Google client values and the API keys; the
  `CF_ACCESS_*` values are deleted. Never print or commit them.
- `REA_JADX_MCP_JAR=~/.local/share/rea/jadx-headless-mcp-0.7.1-all.jar` must be exported for REA's APK code
  commands; it is in no shell profile.
- Verification run `mymind-verify-merge` finished 15:13 CEST: `npm run coverage` reports ready, 917 surfaces, 0 open
  reasons. `android-cards` showed 34/36 because its live video capture predates the "watched" button loading; the
  later `android-cards@media` live capture is byte-identical to the replica. No drive run is going now.
- Shell: zsh; stop processes by PID, never `pkill -f`; foreground `sleep` is blocked.

## Plausible path

1. Plan step 2: decompile the pulled APK (apktool, jadx) and record the shell's views, layouts, fonts, colours and
   network client in `docs/android-plan.md`.
2. Write detailed plans (for example with `ce-plan`), in parallel if useful: accounts (per-person libraries, the test
   user's copy, verification as the test user) and the app (JSON card format, bearer-token
   sign-in endpoint, architecture, screen order). Bring the still-open items above to the user.
3. Plan steps 3–4 on the emulator for reference captures, then build screens in Kotlin on a branch, each proven
   against the real app's screenshot.
