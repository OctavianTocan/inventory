# Android app: every screen and state of the real mymind app

Written 2026-10-17 from the decompiled app (mymind 2.14, `com.mymind.access`). It is the checklist for
`docs/android-plan.md`: what the real app shows, what our app (`android/app`) has today, and which reference
screenshots to take on the phone.

## How to read this

Source tags point at local, Git-ignored trees. This file states facts and file names only; it copies no code and no
resources.

| Tag | Folder |
|---|---|
| `jadx/x/y.java:N` | `android/jadx/sources/x/y.java`, line N (R8-obfuscated Java from Kotlin and Compose) |
| `decoded/...` | `android/decoded/` (apktool output: manifest, `res/`) |
| `mirror/...` | `mirror/` (the web front-end the real app loads in its WebView) |

Rules for reading the numbers:

- Sizes are dp or sp as written in the code. The phone (Samsung SM-F976B cover screen) is 1080 x 2520 px at the
  450 dpi override, so 1 dp is 2.8125 px and the screen is 384 x 896 dp.
- Colours use `#RRGGBB`. "80%" and similar mean alpha.
- "Palette x" means field `x` of the theme class (`jadx/xf/a.java` dark, `jadx/xf/b.java` light), see the palette
  table. The code is obfuscated, so a role is only named when a usage site shows it.
- "Not determined" means the code was not read far enough or the value is not in the app. Nothing is guessed.
- Part 2 of this task already built the shell parts (theme, fonts, tab bar, spinner, tabs, card as its own view). The
  column "Ours today" says what is there now.

## Summary

The real app is a Compose shell around WebViews. The web pages (grid, spaces, serendipity, search, card page, note
editor, upgrade page) are the same front-end as `mirror/`. Native Compose draws the splash, sign-in, onboarding, tab
bar, card header, settings drawer, card menus, dialogs and error screens.

| # | Screen | Kind | Ours today |
|---|---|---|---|
| S01 | Splash | native | built (`ui/splash`) |
| S02 | Sign-in (Login, LoggingIn) | native | built (`ui/signin`), plus our password form |
| S03 | Onboarding, steps 0 to 7 | native + video | none |
| S04 | Welcome screen, guest plan screen | native | none |
| S05 | Main host (WebView, tab bar, loading, overlays) | native | `ui/shell` (`MainShell`, `ShellFrame`), tab bar and spinner compared |
| S06 | Everything tab | WebView | native (`ui/library/EverythingScreen.kt`), search bar compared, grid by geometry |
| S07 | Spaces tab | WebView | native (`ui/library/SpacesScreen.kt`), list and an opened space |
| S08 | Serendipity tab | WebView | native (`ui/library/SerendipityScreen.kt`) |
| S09 | Search | WebView | native overlay (`ui/library/EverythingScreen.kt`, `SearchBar.kt`) |
| S10 | Card view (native header over WebView) | native + WebView | header built and compared (`ui/card`), body is our own text view |
| S11 | Card menu | native | built and compared (`ui/menu`), Reader Mode inert |
| S12 | Select a space, Add tags | native | built (`ui/menu/CardSubmenus.kt`), not compared (no capture) |
| S13 | Image viewer | native | none |
| S14 | Settings drawer (control panel) | native | built and compared (`ui/settings`), data-changing rows do nothing |
| S15 | Account deletion | native | none |
| S16 | Icon picker | native | none |
| S17 | Upgrade view and upsell modal | WebView + native + Rive | none |
| S18 | Web error screen | native | built (`ui/errors`) |
| S19 | Offline screen | native | built (`ui/errors`) |
| S20 | API error dialog | native | built (`ui/errors`) |
| S21 | Photo source popup | native | built (`ui/upload/AddMediaPopup.kt`), opened by the + buttons, not compared (no capture) |
| S22 | Share sheet (share target) | native, own activity | none |
| S23 | Video player | native, own activity | none |
| S24 | Permission dialog, biometric prompt, quick-note notification | system + native | none |
| S25 | Note editor and note composer | WebView | none |
| S26 | Account closed | WebView | none |
| S27 | Dev screens (dev panel, account debug, wallpapers) | native | none |

27 screens: 16 native Compose (S01 to S05, S11 to S16, S18 to S21, S27), 6 WebView pages (S06 to S09, S25, S26), 2
mixed native header or modal plus WebView (S10, S17), and 3 that run in their own activity or in the system (S22,
S23, S24).

## Facts that hold for every screen

### Activity, window and navigation

- One main activity, `com.mymind.access.activities.MainActivity`, portrait only, `windowSoftInputMode=adjustResize`,
  predictive back off (`enableOnBackInvokedCallback=false`) (`decoded/AndroidManifest.xml:26,38`). Our manifest
  already locks portrait. Whether the real app draws edge to edge was not determined.
- The window background at launch is pink `#EEB7C9` (`onboardingBackground`, `decoded/res/values/styles.xml:9`). Ours
  sets it in the launch theme (`android/app/src/main/res/values/themes.xml`).
- WebView view types are `MainView`, `CardView`, `UpgradeView`, `AccountDebugView`, `EditorView`, `AccountClosedView`
  (`jadx/yf/z0.java`). `AccountClosedView` closes when the URL contains `/closed` (`jadx/yf/f.java`).
- WebView status is `Initializing`, `Loading`, `Ready`, `Error` (`jadx/bg/i0.java`). Launch state is `Splash`
  (initial), `Login`, `LoggingIn`, `Onboarding` or `Main` (`jadx/bg/k.java`, `bg/i.java`, `bg/h.java`, `bg/g.java`,
  `bg/f.java`).
- Messages the web page sends to the shell through `appInterface.postMessage` (`jadx/yf/i.java`), by handler
  (`jadx/b0/w3.java`, `jadx/aj/c.java:796`): `view:loaded` (MainView), `view:open` and `view:close` with name
  `SettingsView`, `view:refresh`, `card:open`, `card:deleted`, `view:close` (CardView), `view:onScroll` (CardView),
  `image:view`, `clipboard:write`, `browser:open`, `iap:checkout`, `noteComposer:peekOut`, `noteComposer:hide`,
  `noteEditor:change`, `noteEditor:focused`.
- The shell tells the page to navigate with `mymind.trigger({"type":"navigate","path":...})`: Everything
  `/everything` (`jadx/wf/a.java`), Serendipity `/clear` (`wf/b.java`), Spaces `/spaces` (`wf/c.java`).
- The web front-end has Android rules under `body.android` (`mirror/static/134/0.109.0/styles/app.css` lines 3545,
  3552, 3559, 9554, 9557, 16447): the card detail scroll area has no max height; `main` gets 30 px bottom padding;
  a keyboard placeholder has the height of CSS variable `--android-keyboard-inset-height`; the browser container has
  no min or max height; the editor bubble menu is hidden.

### Theme rule: native chrome follows the system, pages follow the account

- **Native parts follow the system mode.** The Compose theme picks the dark palette (`xf/h`) or the light one
  (`xf/i`) from the system night mode (`jadx/r8/c.java`, function b, through `xf/k.f21559a`); the activity sets
  the system bar style the same way (`jadx/com/mymind/access/activities/MainActivity.java:51`). This covers the tab
  bar, the card header icons, the card menu, the settings drawer and the loading layer.
- **Web pages follow the account.** The shell sends the system mode to the page as a `color-scheme` header
  (`jadx/yf/c0.java:233-243`), but the page uses the account's own `colorScheme` setting and falls back to the system
  only when that is `system` or missing (`mirror/static/134/0.109.0/scripts/base.js:23-56`). Library pages, the
  search overlay and card pages therefore follow the account.
- **The capture agrees.** The capture account is set to dark, so `02-everything-top-light.png`, `09-settings-top-light.png`
  and `14-card-article-menu-light.png` show dark pages with the light-system status bar icons, while their native
  parts use the light palette (the tab bar in `02-everything-top-light.png`, the menu in
  `14-card-article-menu-light.png`). In the light-system launch recording (`videos/`) the loading layer is light
  (240,241,246) and the tab bar (246,247,250) stays light after the dark page (23,24,33) appears; in the dark-system
  one both are dark (20,21,30).
- **Ours:** `ui/theme/MymindTheme.kt` keeps the system palette for chrome (`MymindTheme.chrome`) and `PageTheme(scheme)`
  for pages; `ShellFrame` draws the status-bar background and tab bar in the system palette and the page in the
  account's. `LibraryViewModel.scheme` reads `colorScheme` from the config script of the `/everything` page, as the
  web does, and falls back to `GET /app/v1/me` when the page cannot be read. The screenshot tests set the account to
  dark, like the capture.

### Matching the phone capture

- **Phone configuration.** `android/app/src/test/kotlin/com/octaviantocan/mymind/ui/Phone.kt`: 1080 x 2520 px, 450
  dpi (1 dp = 2.8125 px), font scale 1.08, status bar 113 px, gesture navigation bar 42 px (`device.txt` of the
  capture). Our tab bar lands on the pixels of `02-everything-top-dark.png` and `-light.png` (S05).
- **Text zoom.** The phone's font scale 1.08 reaches the WebView as text zoom: every CSS font size and every px line
  height is multiplied by 1.08, including sizes in `cqw`. So CSS `Npx` is written `N.sp` in ours (sp carries the
  font scale), and a `cqw` size becomes dp × 1.08 (the repository title at 10 cqw of a 177 dp tile is 19.1 dp on the
  phone). Proof: in `02-everything-top-dark.png` the two-line article headline (CSS `17px/1.06`) has a line pitch of
  55 px; 17 × 1.06 × 1.08 × 2.8125 is 54.7 px, while without the zoom it would be 50.7 px.
- **Display P3.** The capture PNGs are Display P3. Greys come out the same, saturated sRGB colours do not: `#FF5924`
  reads (236,101,57), `#F90019` reads (228,50,44), `#C00013` reads (176,36,33), `#FF002E` reads (234,51,58),
  `#2DBA4E` reads (93,183,91), `#1E9BF0` reads (74,153,233), which are those colours converted from sRGB to P3. The
  compare therefore converts our image to P3 before diffing (`compare.py --p3`); our code keeps the sRGB values.
- **Compare tolerance.** Per region, a pixel fails when any channel differs by more than 24; a region passes when at
  most 1% of its pixels fail. Proof (`evidence/android-compare/`, `results.jsonl`): the known-good pair, the tab bar
  of `02-everything-top-*.png` against that of `03-everything-scroll-1-*.png` (same bar, other page state), gives 0
  in both modes; our tab bar moved by 1 px fails at 1.52% and by 1 dp at 3.13% (`tolerance-shift-*`). So the
  tolerance absorbs anti-aliasing and P3 rounding but not a 1 px shift of text or icons.
- **Methods.** "Pixel" means the region diff above on chrome (tab bar, search bar ends, card header, menu, settings
  text). "Geometry" means ink boxes, line boxes and baselines measured on both images (`evidence/android-compare/tools/`
  `bbox.py`, `lines.py`, `blobs.py`) for content whose text and pictures are the user's, which our samples cannot
  match.
- **Robolectric fonts.** Robolectric draws the variable `nunito.ttf` narrower than the phone: the search placeholder
  is 411 px wide in ours against 426 px in the capture. Widths of Nunito runs are compared on line boxes and
  baselines, not on pixels. On a device the same font renders at weight 400 as on the phone.

### Fonts (`jadx/xf/f.java`, `decoded/res/font`)

| Role | Font | Our `MymindFonts` |
|---|---|---|
| f21544a | `louize_regular.otf` | `louize` |
| f21545b | `nunito.ttf`, variable weight 400 | `nunito` |
| f21546c | `nunito.ttf` weight 500 | `nunitoMedium` |
| f21547d | `nunito.ttf` weight 600 | `nunitoSemiBold` |
| f21548e | `nunito.ttf` weight 700 | `nunitoBold` |
| f21549f | `avenir_roman.otf` | `avenir` |

Text styles that also pass a font weight (for example 500) do so on top of these families. Our `Fonts.kt` matches the
six faces.

### Palette (`jadx/xf/a.java` dark, `xf/b.java` light; 35 constructor values, read in order)

Values are the same in both themes unless two are shown. Roles are named only where a usage site was read.

| Field | Dark | Light | Seen used for |
|---|---|---|---|
| a | `#000000` | `#FFFFFF` | splash icon tint (`hg/a.java:1079`) |
| b | `#EEB7C9` | `#EEB7C9` | pink brand colour, launch window |
| c | gradient: transparent at 0.0 to `#F0F2F5` at 0.3 | same | background of a full-size container (`hg/a.java:229`) |
| d | 80% `#354154` | 80% `#354154` | text on the guest-plan screen |
| e | `#646C79` | `#646C79` | "Continue as guest" button colour (`hg/a.java:849`) |
| f | 80% `#323E51` | 80% `#323E51` | "Thank you for supporting" line |
| g | `#FF5924` | `#FF5924` | accent: selected tab, orange buttons, error retry |
| h | `#626C7A` | `#748297` | unselected tab, error screen text |
| i | `#14161E` | `#F0F2F5` | page background, default card header colour |
| j | `#1D1E28` | `#FFFFFF` | raised surface, dividers in the photo popup |
| k | `#14161E` | `#F0F2F5` | tab bar fade (`fg/n.java`) |
| l | `#4D505C` | `#B8BCC3` | card chip background in the menu (`fg/h0.java`) |
| m | `#14161E` | `#F1F3F7` | not seen used |
| n | `#F90019` | `#F90019` | danger red (settings danger zone) |
| p | `#14161E` | `#F0F2F5` | not seen used |
| q | `#FFFFFF` | `#2C1A20` | name in the settings greeting |
| r | 70% `#A7B4C6` | 80% `#FFFFFF` | settings label colour |
| s | `#FFFFFF` | 70% `#000000` | settings value colour |
| t | `#14161E` | `#FFFFFF` | not seen used |
| u | `#A7B4C6` | `#161B23` | upsell title (`cg/l.java:158`) |
| v | 80% `#626C7A` | 80% `#354154` | upsell body (`cg/l.java:216`) |
| w | `#1C1E26` | `#F4F6F9` | upsell row colours (`cg/l.java:322`) |
| x | `#748297` | `#8B99AF` | upsell row colours |
| y | MenuColors | MenuColors | menus and submenus (below) |
| A | 40% `#000000` | 50% `#FFFFFF` | settings switch |
| B | `#8493A9` | `#4C5F7C` | settings row label |
| C | SwitchColors | SwitchColors | switch track and thumb |
| D | `#B2C1DC` | `#4A5465` | `eg/f.java` (not read) |
| E | `#303140` | `#FFFFFF` | `eg/f.java` (not read) |
| F | `#1D1E28` | `#5B6779` | `db/a.java` (not read) |
| G | `#14161E` | `#DBE1EA` | `db/a.java` (not read) |
| H | `#FF5924` | `#FF5924` | not seen used |
| I | `#232733` | `#DFE5ED` | not seen used |
| J | `#FFFFFF` | `#FFFFFF` | not seen used |
| K | `#444B60` | `#FFFFFF` | not seen used |

Fixed fields outside the constructor: `z` `#748297` (hint and footer text in settings), `L` `#FF003D` (account
deletion background, `jadx/bg/a.java`), `o` ShareSheetColors (below).

Our `Colors.kt` holds only the page, tab bar and tab colours from this table. The rest is added screen by screen.

MenuColors (`jadx/xf/p.java`, 17 values in constructor order; light and dark are separate):

- Dark: `#14161E`, `#1C1E26`, 10% `#FF002E`, `#748297`, `#FF002E`, `#748297`, `#748297`, `#A7B4C6`, `#1C1E26`, 60%
  `#748297`, `#A7B4C6`, 60% `#748297`, `#FF5924`, `#1C1E26`, `#748297`, `#1C1E26`, `#748297`.
- Light: first value is white; then `#F4F6F9`, 10% `#FF002E`, `#353643`, `#FF002E`, `#748297`, `#748297`, one value
  that is white or black (not determined), `#F4F6F9`, 30% `#415777`, `#B3BCCA`, 60% `#51647D`, `#FF5924`,
  `#F4F6F9`, white or black (not determined), `#F4F6F9`, `#353643`.
- Roles seen: header text is value 6, close icon value 7, tag text value 8, tag field background value 9, space row
  background value 16, space row text value 17 (`jadx/kg/i.java:229,249,675,726`, `fg/h0.java:171,201-214`). The
  numbering of those usage sites is by field name, so the match to constructor order is likely but not proven.

ShareSheetColors (`jadx/xf/q.java`, same in both themes), 20 values by field letter a to t: `#F0F2F5` (sheet
background), `#C8D3E3`, `#121B28` (inactive tab), white (active tab), 13% `#534E48`, white, 12% `#313A47`, `#415777`,
black, 60% `#51647D`, `#B3BCCA`, `#A7B4C6`, `#A7B4C6`, `#DEE4ED`, `#747E8D`, white, `#A7B4C6`, `#424954`, black,
`#A7B4C6`. Only the sheet background and the two tab colours were matched to a usage site (`jadx/jg/n.java`); the
other roles were not determined.

### Animation facts that recur

- Default tween is 300 ms with the Compose default easing when the code calls `tween(ms)` without an easing
  (`v.e.n(ms, 6, null)`); `FastOutSlowIn` is named only where it appears.
- Tab icon colour: animated colour, default animation spec (`jadx/fg/n.java:129-235`).
- Lottie tab icons are 44 x 44 at 100 fps, 100 frames, 1.0 s (`decoded/res/raw/{everything,spaces,serendipity}_{light,dark}.json`).
- `load_spin.json` is 480 x 600 at 30 fps, 120 frames, 4.0 s (`decoded/res/raw/load_spin.json`).

### Haptics

The real app has haptics, so ours plays them where it does and of the same type, and adds none of its own (decided
in round 3: the user wanted haptics, and the code settles where the real app has them).

- **Native:** every tab tap plays the confirm haptic (`HapticFeedbackType.Confirm`, constant 16, `CONFIRM`;
  `jadx/db/a.java`, `eg/c.java`, `eg/f.java`). The image viewer's long press plays the long-press haptic
  (`jadx/b0/x2.java` case 3, `fg/n.java:2876`); S13 is not built.
- **From the page:** the web front-end calls `mymind.native.generateFeedback()`, and the native "Haptics" bridge plays
  the long-press haptic (`HapticFeedbackType.LongPress`, constant 0; `jadx/bg/t.java:127`, `bg/h0.java:70`,
  `fg/n.java:2334`, `aj/c.java` case 9). The page calls it (scripts under `mirror/static/134/0.109.0/scripts/`) on:
  the 500 ms card long press that opens the native card menu (`components/menu.js`); the space options toggle
  (`components/dropdown.js:67`); a tag's remove and a tag value tap (`tagging.js:507`, `:513`); a type or tag chip in
  the search suggestions (`components/SearchSuggestions.js:236`, `:260`); search clear and remove filter
  (`search.js:91`, `:562`); the Top of Mind drag reorder (`TopOfMind.js:202`, not built).
- **None** on pull to refresh (`components/pull-to-refresh.js` has no call) or on a delete confirm, so ours has none
  there either, although the round's fallback list named both.
- **Ours:** Compose `LocalHapticFeedback` only, which follows the system's touch feedback setting; no custom
  vibration. Tab bar (`ui/shell/TabBar.kt`, Confirm); card tile long press (`ui/library/Tiles.kt`, the long-press
  haptic that `combinedClickable` plays itself); the opened space's menu dots (`SpacesScreen.kt`); a tag's remove in
  the card view (`ui/card/CardSheet.kt`); the search chips and the search clear (`EverythingScreen.kt`,
  `SearchBar.kt`). Tested with a recording `HapticFeedback` (`InteractionTest`).

---

## S01 Splash

- **Reached:** every cold start while the launch state is `Splash` (initial).
- **Kind:** native Compose (`jadx/hg/a.java:996-1129`, `tf/c.java`, `bg/q.java:70-90`).
- **Ours today:** `ui/splash/SplashOverlay.kt` draws the three layers over the page until the state is `Main`.
  The launch theme sets the window background to the pink `#EEB7C9` (`res/values/themes.xml`), so the first frame
  before Compose is pink. `MymindRoot` maps our session state to the launch state: loading is Splash, signed out is
  Login (LoggingIn while the sign-in call runs), signed in is Main. Screenshots: `splash_{light,dark}.png`.
- **Layers (all shown until the state is `Main`; exit fade 500 ms):**
  1. `splash_bg` drawable, centred over the palette's onboarding background, drawn at its own size and shrunk to fit
     (`ContentScale.Inside`), not cropped; scale and blur animate with tween 1500 ms (labels "animate scale",
     "animate blur").
  2. `mymind_splash_icon`, 64 dp wide with the height following the picture (672 x 1052, so the icon isn't
     square), tinted with palette a; scale animates 1500 ms, alpha 500 ms.
  3. A bottom gradient from transparent at 40% to 75% white at 70% of the height; alpha animates 1500 ms.
- **Start values:** scale 1.0, blur 0 dp, icon centred, icon scale 1.0, icon alpha 0, gradient alpha 0.
- **Target values by state:**

| State | Background scale | Blur | Icon position (x, y bias) | Icon scale | Icon alpha | Gradient alpha |
|---|---|---|---|---|---|---|
| Splash, Main | 1.0 | 15 dp | centre | 1.0 | as start | 0 |
| Login, LoggingIn | 1.5 | 30 dp | (0, -0.7) | 0.75 | 1 | 1 |
| Onboarding | 1.5 | 30 dp | (0, -0.5) | 0.75 | 1 | 0 |

- **Pictures:** `splash_bg.png` (1500 x 1696) and `mymind_splash_icon.png` (672 x 1052) are in
  `decoded/res/drawable-xxxhdpi`. Their look wasn't viewed (viewing sends the picture to a hosted model, which the
  rules forbid). The Gradle task `copyMymindResources` copies them into the git-ignored generated folder; never
  commit them. The screenshot tests draw plain bands and blocks in their place (`FlatPictures.kt`); the files
  `splash_real_*.png` use the shipped pictures and are for the owner to look at.
- **Not determined:** whether Android 12's own splash window shows first.

## S02 Sign-in

- **Reached:** launch state `Login` (signed out). `LoggingIn` while the Google call runs.
- **Kind:** native Compose over the splash background (`jadx/hg/a.java:74-206`, `bg/i.java`, `bg/h.java`).
- **Ours today:** `ui/signin/SignInScreen.kt` (`SignInOverlay`) puts `SignInContent` over the splash scene. Our
  sign-in methods stay: Google one-tap, and a password form for the test user (`PasswordForm.kt`, two white pills and
  an accent pill at the bottom, 18 sp Nunito). A failed sign-in opens the API error dialog (S20) titled "Sign in
  failed". Screenshots: `sign_in_{light,dark}.png`, `signing_in_{light,dark}.png`.
- **Keyboard (ours):** the window is edge to edge, so the keyboard does not resize it; `imePadding` lays the screen
  out above the keyboard, the password form sits on it, and when the centred title column would reach the form it
  moves up by the overlap while the splash logo fades out. Screenshots `sign_in_keyboard_{light,dark}` (a 986 px
  keyboard, the reference phone's); the emulator shot `evidence/android-compare/smoke2/shots/02-signin-keyboard-password-empty.png`
  shows the keyboard with the username typed and the password field empty.
- **Facts:**
  - Title "Access\nyour mind" (`access_your_mind`): Louize 64 sp, line height 50 sp, letter spacing -0.05 em, centred;
    column spaced 40 dp, horizontal padding 64 dp (`hg/a.java:199`). The line height is smaller than the type, so the
    dot of the "i" in "mind" touches the "s" above it.
  - A Google sign-in button shows when `showSignInButton` is true and a loading state shows while
    `showSigningInLoading` is true (`hg/a.java:74-132`). The button uses the default `fadeIn()` and `fadeOut()`. The
    loading indicator is 64 dp, a black arc on the accent track with round caps; it enters and leaves with a fade plus
    a scale from and to 0 on a spring of stiffness 400. Strings: "Sign in with Google", "Sign in", "Sign in failed",
    "LOGIN" (`decoded/res/values/strings.xml:165,245-247`).
  - Google pill: minimum height 48 dp (it grows with the text), corner 64 dp, white fill, the 24 dp `svg_google`
    logo before the text, text Nunito 18 sp black, padding 30 x 10 dp, row spacing 12 dp.
  - "Let's get started" pill (onboarding): minimum height 48 dp, corner 64 dp, white fill, 24 dp `arrow_right` icon
    after the text, text Nunito 18 sp, padding 30 x 10 dp, row spacing 12 dp (`hg/a.java:132-175`).
- **Not determined:** nothing that blocks the layout. The password form is our own and isn't in the real app.

## S03 Onboarding, steps 0 to 7

- **Reached:** launch state `Onboarding`, after sign-in on a new account.
- **Kind:** native Compose with `onboarding1.mp4` to `onboarding7.mp4` (`decoded/res/raw`), state in
  `jadx/hg/l.java`, `hg/i.java`, layout in `hg/a.java` (functions d, e, f) and `fg/b0.java:260-420`.
- **Ours today:** none.
- **Flow:** `OnboardingUiState(currentStep, isLoading, error)`; steps 0 to 7; pressing the main button goes to the
  next step and, on step 7, finishes onboarding (`hg/l.java`).
- **Page background** animates with the default spring (label "onboarding bgcolor"): step 0 transparent over the
  splash, step 1 `#394559`, steps 2 to 7 white (`hg/a.java:398-690`).
- **Videos:** steps 1 to 7 play `onboarding<step>.mp4` in a video view with 12 dp side padding and 30 dp rounded
  corners; content slides (tween 600 ms) and fades between steps (`hg/a.java:398-690`).
- **Texts** (Louize, letter spacing -0.05 em, centred), size, colour and line height by step:

| Step | String | Size | Colour | Line height | Main button |
|---|---|---|---|---|---|
| 0 | "Welcome to your new mind." | 60 sp | `#522857` | 0.85 em | Let's get started |
| 1 | "It's like your real mind, but enhanced with AI." | 40 sp | `#1F2E48` | 0.95 em | Continue |
| 2 | "Just save it. Artificial Intelligence takes care of the rest." | 35 sp | `#78374C` | 0.98 em | Love this! |
| 3 | "No folders. No collections. No wasted time organizing." | 32 sp | `#674E40` | 0.9 em | Okay cool |
| 4 | "Bookmark websites and images by sharing them to your mind." | 35 sp | `#1F2E48` | 0.9 em | Continue |
| 5 | "Save articles, quotes & highlights without breaking your focus." | 35 sp | `#78374C` | 0.94 em | Continue |
| 6 | "Keep all your notes, todos and ideas within reach." | 35 sp | `#253E37` | 0.9 em | Continue |
| 7 | "And remember, it's completely private" | 45 sp | `#522857` | 0.84 em | I'm ready, let's go |

  Step text sits at the top centre with 50 dp top padding (`fg/b0.java`, default branch).
- **Buttons** (`hg/a.java:217-332`; bottom area 152 dp tall, 12 dp side padding, palette c behind it):
  - Steps 0 to 6 next button: bottom end, 30 dp end and 40 dp bottom margin, height 44 dp, white fill, 8 dp border
    `#D4DCE7`, 25% black shadow, 64 dp corners, padding 30 x 10 dp, row spacing 8 dp, 16 dp `arrow_right` icon after
    the text, text Nunito 16 sp weight 500 `#748297`.
  - Step 7 button: bottom centre, 112 dp bottom margin, height 44 dp, palette g fill, 64 dp corners, padding 16 x 10
    dp, 16 dp `arrow_right`, text 16 sp weight 500, white.
  - "Skip": bottom start, 30 dp start and 40 dp bottom margin, capitals, Nunito 15 sp weight 500, `#748297`, letter
    spacing 0.05 em, line height 20 sp, tap area padding 30 x 10 dp.
- **Not determined:** the video frames; which step "Skip" jumps to; whether the carousel can be swiped.

## S04 Welcome screen and guest-plan screen

- **Reached:** the welcome screen (`hg/a.java:694-735`) is a layout with "Welcome to your new mind." and the "Let's
  get started" pill; where it is used was not determined. The guest-plan screen (`hg/a.java:736-882`) follows the end
  of onboarding for a guest account (`fg/n.java` overlays, `openUpgradeView`).
- **Kind:** native Compose. **Ours today:** none.
- **Welcome:** Louize 64 sp, line height 50 sp, letter spacing -0.05 em, centred, horizontal padding 64 dp, spacing
  40 dp, then the 48 dp pill from S02.
- **Guest plan:**
  - Picture `plans_guest.png` (710 x 1536, `decoded/res/drawable`).
  - Title "Upgrade for the full\nmymind experience": Louize 35 sp, line height 0.9 em.
  - Body "Join our thousands of members\nwho get advanced features,\ncustom icons and more cards.": Nunito
    weight 500 (f21546c) 18 sp, line height 1.3 em, palette d.
  - Button "See our plans": height 48 dp, 100 dp corners, palette g fill, white capitals, Nunito 700 (f21548e) 16
    sp, letter spacing 0.1 em, line height 1.4 em.
  - Button "Continue as guest": same size, transparent, border 20% of palette e.
  - Heart icon `ic_heart`, then "Thank you for supporting an independent company": Nunito weight 600 12 sp, line
    height 1.33 em, palette f.
- **Not determined:** how the plan picture is placed (fit, crop, size).

## S05 Main host

- **Reached:** launch state `Main` after the splash exit.
- **Kind:** native Compose host (`jadx/fg/n.java:1953-2290`).
- **Ours today:** `ui/shell/MainShell.kt`: `MainShell` holds the tab, the opened space and card, the card menu and
  the settings drawer; `ShellFrame` draws the page between the status bar and an 80 dp bottom gap (layer 2) in the
  account's scheme, with the tab bar over it in the system's. Screenshots: `tab_bar_{light,dark}`,
  `spinner_{light,dark}`, `S05_spinner_{everything,spaces,serendipity}_{light,dark}`.
- **Structure, back to front:**
  1. A Material 3 modal navigation drawer (not a bottom sheet) with a black 66% scrim; its drawer content is the
     settings panel (S14); its gestures are on only while the settings view is open (`profileScreenOpen`,
     `fg/b.java`, `fg/n.java:2120-2143`).
  2. The WebView host: top padding is the status-bar height, bottom padding 80 dp. The content blur animates with
     tween 750 ms `FastOutSlowIn`, driven by open overlays. The system Back button goes back in the WebView history
     when there is one.
  3. The tab bar container, 128 dp tall, with a vertical gradient from transparent at the top to palette k at 25%
     (`fg/n.java:129-235`).
  4. The card header (S10) when the state is `Main` and the WebView is `Ready`.
  5. The web error screen (S18) when the WebView is `Error`.
  6. A loading layer with the spinner (below) and an offline layer (S19).
  7. The overlay host for dialogs and full-screen views (S13, S15, S16, S17, S27).
- **Tab bar:** Material 2 bottom navigation, default 56 dp high, transparent, no elevation; label Nunito 13 sp, line
  height 18 sp; selected colour palette g, unselected palette h; 16 dp space above the icon; order Everything,
  Spaces, Serendipity (`jadx/fg/n.java`, `xf/h.java`, `xf/i.java`). The selected tab plays its Lottie once; other
  tabs sit on frame 0 (assumed, see "Not determined").
- **Loading spinner:** `load_spin.json` (480 x 600, 30 fps, 4 s), drawn by `jadx/jg/n.java` function h (line 515)
  in the loading layer (`jadx/hg/a.f9205b`). The layer covers the whole screen, so the spinner is centred on the
  screen, not on the page: across the frames of the launch recordings (`videos/`) its ink stays inside 503,1212 to
  578,1309 px, around the screen centre, in palette h of the system. Ours draws the Lottie 38 dp tall, looped, in the
  system's unselected-tab colour, offset from the page centre to the screen centre (`ui/library/CardGrid.kt`,
  `PageSpinner`); its ink box is the same 503,1212 to 578,1309.
- **Compared** (pixel, P3, threshold 24, max 1%, region x 12 to 1068, y 2262 to 2478; the frame edges and the home
  indicator are left out): tab bar against `02-everything-top-*.png` 0.039% dark, 0.056% light; against
  `04-spaces-top-*.png` 0.074% and 0.078%; against `05-serendipity-top-*.png` 0.052% and 0.064%. Spinner: geometry
  (ink box above) and colour. Files: `evidence/android-compare/S05-tab-bar-*`.
- **Tab bar during search:** the capture README says it is hidden while search is focused; in
  `07-search-focus-*.png` the keyboard covers that area. Ours keeps it under the keyboard (edge to edge, no IME
  padding on the shell). Whether the keyboard or the focus hides it was not determined. Unselected tab icons sit on the first Lottie frame in every tab shot (`02-`, `04-`, `05-`), matching
  ours.

## S06 Everything tab

- **Reached:** the first tab; `/everything`.
- **Kind:** WebView (the page in `mirror/`).
- **Ours today:** native, from `/app/v1/cards` (`ui/library/EverythingScreen.kt`, `SearchBar.kt`, `CardGrid.kt`,
  `Tiles.kt`, `CardKinds.kt`, `CardFeed.kt`, `LibraryViewModel.kt`): the search bar with the hamburger (opens S14) and
  the + button, the Top of Mind row (pinned cards), then the two-column masonry grid that loads the next page near
  its end. Tiles follow the web card classes: picture, web page, article, post (coloured edge per platform), note,
  repository, video. A long press on a tile opens the card menu (S11) without opening the card, as the page's 500 ms
  long press does. Screenshots: `S06_everything_{top,scrolled,account_system}_{light,dark}`,
  `S06_top_of_mind_{light,dark}`, `S06_pull_refresh_{light,dark}`, `S06_upload_strip{,_details}_{light,dark}`.
- **Top of Mind:** the row reads `GET /app/v1/top-of-mind` (pin order, paged; the row loads the next page near its
  end) instead of picking pinned cards out of the library. Adding or removing a pin from the card menu reads it again.
  A row resting at its start stays at its start when a new first pin arrives, so the new pin shows (`TopOfMindRow`;
  the lazy row would otherwise keep its first key in place and push the new pin off to the left).
- **Pull to refresh** (Everything, the search results, S07's list and an opened space): Material 3
  `PullToRefreshBox` (`ui/library/PullRefresh.kt`), its indicator under the search bar or the 60 dp header, the
  accent arrow on the page surface. It reads the first page again; when that page starts the list already shown
  (same cards, same order, and either more pages or no more cards than shown) nothing changes, so the scroll
  position stays; otherwise the list is replaced by the new first page and paging starts over. Pulling Everything
  reads Top of Mind again too. A failed read keeps the list. The page's own pull to refresh
  (`components/pull-to-refresh.js`: a downward swipe of at least 150 px with at most 100 px sideways while the page is
  at the top) shows a 35 px Lottie in the top bar (`mymind-syncing-progress-bar`, at least 2.1 s) and synchronises;
  ours keeps the Material indicator the round asked for, so its look differs. No haptic, as on the page.
- **Uploads** (the + here and inside an opened space): the + opens S21. "Photo library" opens the system picker for
  several images and videos (`PickMultipleVisualMedia`), "Take photo" the camera into a cache file shared through
  `FileProvider` (`<package>.provider`, `res/xml/file_paths.xml`), as the real app does (`jadx/yf/j.java`,
  `fg/l0.java`, `rf/b.java`). Each file then goes through the web flow (`docs/write-api.md` 4.2, `docs/app-api.md`
  "Uploads from the app"): `POST /uploads/authorize?count=N` (at most 100 files a call); `PUT` of each `url` in
  32 MiB pieces with `content-type`, `content-range`, `x-file-name` (encodeURI), the upload token in
  `x-upload-token`, `x-upload-id` from the second piece, and the app's own bearer token; then
  `POST /objects {"type":"Media","uploadId"}` (with `spaceId` inside a space, followed by
  `PUT /spaces/{s}/objects/{c}`). Files go one at a time (`ui/library/UploadQueue.kt`). A new card is put first in
  Everything; inside a space the space and the space list are read again. Empty files and files over 500 MB (the
  server's limits) fail without a request. The system photo picker names its files by media id (`20.jpg`), not by
  the original name, so those are the names the queue and the server get.
- **Upload queue strip** (`ui/upload/UploadStrip.kt`, the real app's `jadx/a0/i.java` case 10): shown while the queue
  holds files and the tab is not Serendipity, fading in and out; 8 dp from the sides and 120 dp from the bottom,
  palette j, 8 dp corners, shadow black 20% (y 8, blur 24); a tap opens or closes the list. Header at least 48 dp,
  16 dp side padding (24 dp and 20 dp top and bottom with the list), the turning `ic_upload_progress` at 18 dp and
  "N item(s) in your upload queue" in Nunito 16/20 sp, -0.01 em, palette h. List rows: at least 48 dp, 12 dp in from
  the strip, palette m, 4 dp corners; the kind icon (`ic_image`, `ic_video`, `ic_pdf`, `ic_text`) at 20 dp, red
  (palette n) on a failed file; "Image saved on yyyy.M.d" or the error in red; the turning icon at 14 dp while the
  file is sent, a 32 dp red delete button on a failed file that removes it from the list.
- **Facts:** the page is the mymind web page with `body.android` rules (above). The search bar shows the hamburger
  icon (class `hamburger-enabled`, `mirror/.../app.css:18028-18053`, 25 x 17 px mask, 60% opacity); a tap on it sends
  `view:open` with name `SettingsView` (`mirror/static/134/0.109.0/scripts/search.js:97-105`). The bar has a shadow
  (light `8px 17px 43px rgb(54 58 64 / 15%)`, dark `rgb(0 0 0 / 50%)` from `dark.css`) and sits above the + button
  (`.upload`, 55 px square, 6 px corners, `plus.svg` at 21 px). The capture shows both on Android
  (`02-everything-top-*.png`).
- **Compared:**
  - Search bar ends, pixel (P3, 24, 1%): hamburger area 0.23% dark, 0.25% light; + button area 0.072% dark, 0.089%
    light (0.52% and 0.54% before the square's edges were rounded to whole pixels as the browser paints them, with
    the plus at the page's exact centre) (`S06-search-bar-*`).
  - Grid, geometry against `02-everything-top-*.png`: columns x 28 to 526 and
    554 to 1052 px, the same; 10 dp gaps, the same; an article tile 497 px tall with a 197 px picture, the same;
    article headline line pitch 54 against 55 px; a repository tile 478 against 479 px, its green edge 11 px, title
    to label baseline 68 px and label width 126 px, the same, label 2 px lower; Top of Mind tiles, x and size the same.
- **Remaining differences:** no platform logos, GitHub logo or article site logo (the post X mark is a text glyph);
  the column each card lands in can differ from the web masonry; Robolectric's narrower Nunito (see Facts).
- **Uploads, ours against the real app:** the real app queues uploads in WorkManager (`UploadBlobWorker`), so they
  outlive the app; ours sends them from the view model, so closing the app stops them. The real app's failure text
  falls back to "Unknown error"; ours shows the API error text, and its own for an empty or oversized file. A photo
  taken with the camera is sent as `image/jpeg` from a `.jpg` cache file. Both the `spaceId` field and the space
  `PUT` are sent inside a space (the `PUT` is idempotent). Ours drops a file from the queue once its card exists, so
  the strip fades out after the last one (reading "All items uploaded" while it fades); when the real app drops
  finished work was not determined.
- **Inert:** the composer peek (`noteComposer:peekOut`; S25 is not determined).
- **Not determined:** where the class `hamburger-enabled` is set in the page.

## S07 Spaces tab

- **Reached:** second tab; `mymind.trigger navigate /spaces` (`jadx/wf/c.java`).
- **Kind:** WebView.
- **Ours today:** native, from `/app/v1/spaces` and `/app/v1/spaces/{id}/cards` (`ui/library/SpacesScreen.kt`): the
  60 dp "All Spaces" header, two columns of space stacks (the first card over three tilted blank cards seeded by the
  space id, `--stack-shadow`, the space's 14 dp ring and name), the orange + button 50 dp above the end of the page,
  and an opened space (back arrow, name, menu dots, the card grid). The + on the list opens the web's new space
  modal (`NewSpaceDialog.kt`: "Create new space", "Name your new space", "NEXT STEP", then "Pick a color" with the
  web's 16 colours and "FINISH & SAVE") and creates the space with `POST /spaces {name,color}`. The menu dots of an
  opened space open a menu with "Delete space" (`DELETE /spaces/{id}`) behind a confirm dialog in our own words,
  because the web's delete modal text is not recorded. The list and an opened space refresh on a pull (S06), and the
  + inside an opened space uploads into that space (S06, "Uploads"). Screenshots: `S07_spaces_top_{light,dark}`,
  `S07_space_open_{light,dark}`, `S07_new_space_{name,color}_{light,dark}`, `S07_pull_refresh_{light,dark}`,
  `S08_upload_strip_space_{light,dark}` (the strip over an opened space).
- **Compared** against `04-spaces-top-*.png` and `20-space-top-light.png`:
  - Header title, geometry: ink x 415 to 666, y 174 to 226 in the capture against 416 to 664, 176 to 227 in ours
    (baseline 214 against 215).
  - Opened space, pixel: back arrow region 0.66%, menu dots 0 (`S07-space-header-light`).
  - + button, pixel inside its rounded corners: 0.165% dark, 0.159% light (1.08% and 1.09% before); a region 10 px
    wider on each side 0.327% and 0.335% (`S07-spaces-fab-*`, `S07-spaces-fab-wide-*`). Compose rounds the 15 dp and
    50 dp padding to whole pixels where the web page places the square at 140.6 px; ours now draws the square at the
    page's fractional position with its edges on the nearest pixel, and the plus at the exact centre.
  - Page background in system light: the capture's dark page is (23,25,32) there and (20,22,29) in system dark;
    ours is (20,22,30) in both (under the threshold).
- **Remaining differences:** the space stacks hold our sample cards, so only their geometry is comparable; the
  modal shows the spaces icon where the web shows its logo; the space menu is a Material dropdown without the web's
  "Space Details" (smart spaces only); renaming a space (`PATCH /spaces/{id}` exists) is not built because the
  editable header is not.
- **Not determined:** the route of an opened space inside the web page (ours keeps it in the tab's state).

## S08 Serendipity tab

- **Reached:** third tab; `navigate /clear` (`jadx/wf/b.java`).
- **Kind:** WebView.
- **Ours today:** native (`ui/library/SerendipityScreen.kt`, `SerendipityModel` in `LibraryViewModel.kt`): the deck the
  web page deals, in its order: the `data-id` of each `mymind-card` in the partial `GET /serendipity` page (`Accept:
  text/html` and `x-partial: 1`, as the deck script asks through `util.getPartialHTML`; the document request gets the
  page shell without the deck), read with the app's bearer token, each card then read from `/app/v1/cards/{id}` (cards
  that fail or are in the trash are left out); one card at a time; Forget (moves the card to the trash, keeps it when
  the server refuses) and Keep in two 100 dp round buttons; a dot per card; at the end `POST /serendipity/complete`
  once. An empty deck says so in our words ("Nothing to rediscover right now."), and a tap deals again. The tile is
  drawn as the page draws it: post text at the base size, no coloured edge (`.inner` clips it with `overflow: hidden;
  max-height: 55vh`), no title. The page reaches under the status bar. Screenshots: `S08_serendipity_{light,dark}`,
  `S05_spinner_serendipity_{light,dark}`.
- **Compared**, geometry against `05-serendipity-top-light.png`: buttons x 259 to 821, y 1712 to 1993 against 1711 to
  1992 in ours; the dot row y 2035 to 2050, x 293 to 784 against 2035 to 2049, 295 to 784.
  `05-serendipity-top-dark.png` has a bright card picture under the buttons, so it gives no ink boxes.
- **Remaining differences:** the background is a still colour wash where the page animates a WebGL gradient; the
  button outlines are plain; the card's title under a non-post card is not drawn; "All done" is ours, not the web
  outro.
- **Not determined:** the entry animation (no screen recording of the tab entry was read for it).

## S09 Search

- **Reached:** tap the search bar on the Everything tab (the icon area opens settings instead).
- **Kind:** WebView (`mymind-search-bar`, `mirror/.../search.js`), with the system keyboard. The page uses
  `--android-keyboard-inset-height` for a placeholder that fills the keyboard area (`app.css:3559`).
- **Ours today:** a native overlay on the Everything tab (`ui/library/EverythingScreen.kt`, `SearchModel` in
  `LibraryViewModel.kt`): focusing the bar shows the type chips of the library, most used first, from
  `GET /search/suggestions` (the 17 types of `SearchType` until that answers); typing asks
  `GET /search/suggestions?q=` (or `GET /tags/suggest?q=` after a `#`) for tags and searches `/app/v1/search` after a
  pause. A picked chip becomes the filter (`type:<slug>`, or `tag:"name"` for a tag), shown as an accent chip above
  the results with the tag suggestions after it; tapping it lifts the filter. Back or clearing closes the overlay.
  Screenshots: `S09_search_{focused,results,tag_filter,same_vibe}_{light,dark}`.
- **Compared**, geometry against `07-search-focus-*.png`: chip row pitch the same. The capture's chips carry an icon
  each, so they are wider and wrap into other rows than ours.
- **Not determined:** the result layout (the capture has no typed search, by the capture rules).

## S10 Card view

- **Reached:** tap a card. The web page sends `card:open`; the shell opens a second WebView of type `CardView` over
  the main one (`jadx/b0/w3.java` case 6, `fg/m.java`).
- **Kind:** native header plus WebView page. **Ours today:** `ui/card/CardSheet.kt` draws the opened card as a
  sheet over the library and the tab bar, 24 dp under the status bar with 10 dp top corners (as in
  `12-card-article-top-*.png`), with the native header (`ui/card/CardHeader.kt`, `CardHeaderState` for the scroll
  rule) over our own text view of the card (Louize 28 sp title, Nunito 15 sp body); the real body is a WebView page.
  The header title is read-only in ours, because editing a title would change card data. Screenshots:
  `S10_card_{rest,hidden,shown,tags}_{light,dark}`, `card_header_{rest,hidden,shown}_{light,dark}`. The card's tags
  show as pills under the text; the × on a pill removes the tag (`DELETE /annotations/{id}`, the tag id from the
  card's `tags`). The plan said there is no native top bar; there is one (below).
- **Compared** (pixel, P3, 24, 1%) against `12-card-article-top-*.png` (rest), `13-card-article-scroll-down-*.png`
  (hidden) and `13-card-article-scroll-up-*.png` (shown), with the header icons as blocks of the shipped drawable
  sizes: rest, top band 0.149% dark, 0.062% light, the band under it 0; shown 0.149% and 0.056%, band 0; hidden
  0.38% and 0.12% (`S10-card-*`). Geometry: caret ink x 68 to 113, menu dots x 968 to 1012 and y 266 to 277, title
  from x 182 to 900, the sheet's top edge at y 182, all the same in both.
- **Remaining differences:** the body is ours, not the card web page (other serif sizes and layout). In system light
  the real header at rest draws its title and icons in the light chrome's dark ink over the dark card page, so they
  are dim (`12-card-article-top-light.png`); ours contrasts them with the page. That looks like a bug of the real
  app and is not copied.
- **Header** (`jadx/fg/n.java`, function d, lines 235-1633):
  - 64 dp tall, over the top of the page.
  - Left: 64 x 64 dp box with `ic_caret_down` (closes the card). Centre: single-line title field, Nunito 18 sp,
    weight 1 (fills the row); the title is editable. Right: 64 x 64 dp box with `ic_horizontal_menu` when the card
    has menu items.
  - Header colour is the card's own colour (`f6811f` hex) or palette i. The card layer's background is the same
    colour, so the blurred header background matches what is under it. Icon and text colour is white or black,
    picked by which contrasts more.
  - Scroll behaviour from the web page's `view:onScroll` value (`b0/w3.java`), which is the scroll position: at 50 or
    less the header is at offset 0, alpha 1, no background; when the position drops by more than 5 (the page moves
    back toward the top) the header shows with its background; when it grows by more than 5 (the page moves on) the
    header hides (offset -64 dp, alpha 0). Offset animates with tween 200 ms `FastOutSlowIn` (label "headerOffset");
    alpha is 1 at offset 0, 0 at offset -12 or lower, linear between; the background is a blur (20) when its alpha
    is above 0 (label "headerBgAlpha").
- **Back:** the caret and the system Back both close the card (`view:close CardView`, `card:deleted`).
- **Page content:** the card detail web page (images, notes, tags, similar cards) from `mirror/`.
- **Not determined:** the web page layout by card type beyond the tops captured (`12-card-article-top-*`,
  `12-card-note-top-dark`, `18-card-note-top-light`, `18-card-video-top-light`); image, PDF and product cards were
  not captured.

## S11 Card menu

- **Reached:** tap the menu icon in the card header, or long-press a card in a grid (the page's 500 ms long press).
- **Kind:** native Compose (`jadx/fg/h0.java`, `fg/l0.java`).
- **Ours today:** `ui/menu/CardMenu.kt` (`CardMenuSheet`, `CardMenuContent`, `CardMenuAction`, `MenuColors`) and
  `ui/menu/CardActions.kt` (`cardMenuItems`, `CardActions`), opened from `MainShell`. The menu is a rounded card
  (22 dp corners) 16 dp from the screen edges and above the navigation bar, with a drag handle and 52 dp rows; the
  screen behind is blurred 8 dp under a scrim. Screenshots: `S11_card_menu_{article,note,video}_{light,dark}`, and
  `S11_card_menu_real_*` with the shipped icons (written for the owner, not viewed by the agent).
- **Facts:**
  - A preview chip with the card's picture and title: corner 24 dp, padding 6 dp, picture 18 dp corners, text Nunito
    13 sp, background palette l.
  - A sheet title row 40 dp tall, text Nunito 18 sp, line height 34 sp, MenuColors header colour, side padding 20 dp.
  - Item rows: 52 dp tall, 16 dp corners, 24 dp side padding, icon then label with 10 dp spacing, label Nunito 17
    sp; icon per action: add tags, add to space, copy to clipboard, visit source, download, same vibe, delete, remove
    from space, top of mind, remove from top of mind, reader mode (drawables `addtagsicon`, `addtospaceicon`,
    `copytoclipboardicon`, `visitsourceicon`, `downloadicon`, `samevibeicon`, `deleteicon`, `removefromspaceicon`,
    `topofmindicon`, `removetopofmindicon`, `readermodeicon` in `decoded/res`). Delete and remove rows use the
    MenuColors danger values.
  - Settled by the capture: the container is a bottom sheet card, not a popup (`14-card-article-menu-*.png`,
    `14-card-note-menu-dark.png`, `18-card-video-menu-light.png`); the rows and their order for an article, a note
    and a video are the ones `cardMenuItems` gives. `18-card-note-menu-light.png` shows the note editor, not the
    menu.
- **Compared:** rows of all three menus by geometry: row boxes y 1408 to 2433 the same. Pixel (P3, 24, 1%) with the
  shipped icons, against `14-card-article-menu-*.png`, three regions (top rows, icon column, label column): 0, 0, 0
  dark; 0.015%, 0.012%, 0.001% light (`S11-card-menu-article-*`). Icon and label ink boxes are the same to the pixel.
- **Locked rows:** the plan features come from the `features` list of the config script in the `/everything` page
  (`GET /everything` with `Accept: text/html`), as on the web; Same Vibe is locked without `sameVibe`, and Reader Mode
  shows (locked) only without `reader`. The same script gives the account's `colorScheme`; `GET /app/v1/me` is the
  fallback when the page cannot be read.
- **Wired to the API** (`data/Api.kt`; every call carries the app's bearer token): Visit source (opens `url`), Copy
  (the Android client still copies the note text of a note, quote or content card, so formatting is lost and an
  image card's Copy does nothing; the Rust backend now serves `GET /cards/{id}/markdown` to cookie, app-token and
  extension sessions, with an inferred Markdown format; wiring Android to it remains client work), Download
  (`GET /media/{path}`,
  the original file, saved to Downloads through MediaStore as `<card id>.<format>` like the web's name, then the real
  app's toast "Successfully downloaded"), Same Vibe (`/app/v1/search?q=similar:<id>` on the Everything tab under a
  "Same Vibe" chip, the query alone as on the web), Add tags and Add to space (open S12), Top of Mind add and remove
  (`POST /objects/{id}/pin`, `/unpin`, then the Top of Mind row is read again), Remove from space
  (`DELETE /spaces/{space}/objects/{id}`), Delete (`DELETE /objects/{id}`, the trash). Each updates the open lists;
  a refused write shows the API error dialog (S20). Unit tests use a fake library (`ui/menu/CardActionsTest`) and a
  mock server (`MymindApiTest`).
- **Inert:** Reader Mode (no route serves the reader text, and S17 is not built) and locked rows.
- **Not determined:** the open animation of the real sheet; ours uses the Material 3 sheet's slide.

## S12 Select a space, Add tags

- **Reached:** from the card menu items "Add to space" and "Add tags".
- **Kind:** native Compose (`jadx/kg/i.java`, `fg/l0.java`, `jg/n.java`).
- **Ours today:** `ui/menu/CardSubmenus.kt` (`AddTagsPage`, `SelectSpacePage`, `SubmenuHeader`, `tagChoices`,
  `addTag`), drawn in the card menu's card (`MenuCard`), which stays above the keyboard. Screenshots:
  `S12_add_tags_{empty,recent,typed,added}_{light,dark}`, `S12_select_space{,_empty}_{light,dark}`.
  - Add tags: the field takes focus on open; IME Next or a tapped suggestion adds a chip, a tapped chip goes again;
    "Finish & Save" posts each tag, the text left in the field included (`POST /objects/{id}/tags {name}`, answer
    `{id,name,flags}`), and closes the menu. Choices: all tags (`GET /app/v1/tags`) and the recent ones
    (`GET /tags/recent?take=5`, the real app's call in `jadx/of/b.java`).
  - Select a space: one row per space (`GET /app/v1/spaces`); a tap puts the card in (`PUT
    /spaces/{space}/objects/{id}`) or takes it out (`DELETE`) at once, from the card's `spaces` ids. Without spaces,
    "Create Your First Space" opens the new space modal (S07).
- **Facts:**
  - Header title: capitals, `submenu_select_a_space` ("Select a space") or `submenu_add_tags` ("Add tags"),
    Nunito weight 700 14 sp, line height 21 sp, with a 24 dp `close` icon at the end.
  - Space row: width fill, height 48 dp, 16 dp corners, padding 20 dp, 10 dp spacing, a 20 dp ring in the space's
    colour that fills, with the white `spaces_checkmark` fading in, when selected; label Nunito 17 sp, line height
    23 sp, one line with ellipsis. The list is at most 280 dp tall and scrolls.
  - Empty list text: "It looks like you don't have any spaces to\nadd this card to unfortunately." (Nunito 15 sp,
    line height 20 sp) under the 49 dp spaces icon.
  - Tag field: Nunito 17 sp weight 400, line height 24 sp, 16 dp corners, 56 dp tall with the chevron at the end;
    help text "Hit next to add separate tags.\nTap on added tags to remove them." (15 sp / 20 sp), replaced by the
    chips once a tag is added.
  - Suggestions (`jg/n.java`): with text, the tags containing it (case ignored), the first 5, sorted by name; without
    text, the recent tags sorted, under "RECENT TAGS". Typing opens the list; the chevron toggles it.
  - Bottom button (`fg/l0.java:107-150`): height 44 dp, 100 dp corners, side padding 20 dp, top 8, bottom 20,
    MenuColors fill, label "Finish & Save" or "Create Your First Space", Nunito 16 sp. Select a space has no button
    when there are spaces.
  - Blur behind the sheet is set by `updateBlurDp`.
- **Compared:** not; the phone capture has no S12 screen.
- **Not determined:** animation, sheet height, the scrim. Ours lets the Add tags page scroll when the keyboard leaves
  too little room for it.

## S13 Image viewer

- **Reached:** tap an image in the card page; the page sends `image:view`.
- **Kind:** native (`jadx/fg/n.java` function j, state `image viewer`). **Ours today:** none.
- **Not determined:** layout, gestures, background colour, close control. Take a screenshot.

## S14 Settings drawer (control panel)

- **Reached:** tap the hamburger icon in the Everything search bar. The page sends `view:open SettingsView`; the shell
  opens the drawer (S05 layer 1). Back or swiping closes it (`view:close SettingsView`).
- **Kind:** native Compose, `jadx/ig/o.java`, hosted by `fg/b0.java:140-205`. It is a Material 3 modal navigation
  drawer, not a bottom sheet: the sheet is 90% of the screen width (Material 3 caps it at 360 dp) and full height, with
  the library's default shape, tonal elevation and window insets, over a black 66% scrim. Gestures are on only while
  it is open. All text in it is drawn with `BasicText`, so no Material text style adds line height or letter spacing.
- **Ours today:** `ui/settings` (`SettingsDrawer`, `SettingsSheet`, `SettingsRows`, `GlassBackdrop`), wrapped around
  the content of `MainShell`. It opens from the hamburger in our search bar (S06), as in the real app. The account
  line shows the email from `GET /app/v1/me` (`{"email","kind"}`). Rows that would change account data are drawn and
  do nothing. Screenshots: `settings_{light,dark}`, `settings_scrolled_{light,dark}` (flat stand-ins for the
  pictures), and `settings_real_*` with the shipped pictures (written for the owner, not viewed by the agent).
- **Compared** (pixel, P3, 24, 1%, on `settings_real_*`, numbers only) against `09-settings-top-*.png`: the greeting
  0, the swipe hint row 0 and the "YOUR ACCOUNT" label region 0, in both modes. The label region failed at 4.5% dark
  and 3.6% light while the greeting's empty name line had no height, which put the label 1 px low; the line now holds
  a zero-width space, as a line with a name would.
  Against `10-settings-bottom-*.png`, geometry: rows at y 1547 to 1682 and 1705 to 1840, the danger zone y 2021 to
  2246 at x 90 to 881, all the same; a pixel diff of the whole region fails (21 to 24%) because the glass rows show
  the blurred background picture at another scroll offset (`S14-settings-bottom-*`).
- **Remaining differences:** no name in the greeting and no plan block (the API has neither); an extra row under
  the account block in `09-settings-top-*.png` is not built, because its label is not in the native strings and its
  source is not in the `jadx/ig/` code (`ig/v` is `NeedsUpgrade(buttonTitle)`), so it was not determined.
- **Update rows (ours, not mymind's):** after the sign-out row: "Version X (N)", "Check for updates" with its state as
  trailing text, "Allow mymind to install updates" while Android blocks installs, and "Changelog" (opens the bundled
  changelog). The same state drives the "What's new" sheet and the after-update sheet (`ui/update/`).
- **Background:** picture `settings_background_light.png` (464 x 1027) or `settings_background_dark.png` (518 x 1218)
  (`decoded/res/drawable`, chosen in `jadx/xf/l.java`, `xf/m.java`), scaled 1.15 (night) or 1.5 (day) with animation.
- **Content.** The first screenful is a column of fixed height (the window height minus the status and navigation
  bar insets) with 32 dp side padding. The rows below it sit in the outer vertical scroll. Fonts follow
  `jadx/xf/f.java`.
  1. Greeting "You have a\nbeautiful mind,\n<name>": Louize 48 sp, line height 0.95 em, letter spacing -0.05 em. The
     first span uses palette q; the name span is white. Ours leaves the name empty (open question).
  2. 32 dp gap. "YOUR ACCOUNT" label (13 sp, palette r, letter spacing 0.06 em) above the value (20 sp, palette s, line
     height 1.4 em, letter spacing -0.01 em). 20 dp gap. "YOUR PLAN" label and value in the same style. Ours draws the
     plan block only when a plan is known, which it never is yet.
  3. For a guest plan: an upgrade button (label not determined), 48 dp tall, 100 dp corners, palette g, white
     capitals, 16 sp (f21548e), letter spacing 0.1 em. Not built.
  4. A flexible gap, then the icon picker row: the `icon_pile_*` picture (596 x 253), full width inside the side
     padding, 16 dp corners, opens S16. Ours is clickable and does nothing.
  5. Hint "Swipe down for more options" with `ic_hand` on a centred row spaced 8 dp: Nunito 13 sp, line height 18 sp,
     palette z; its alpha is 1 minus the scroll offset in pixels over 100, so it is gone at 100 px.
  6. 20 dp gap. Dev builds only: Wallpapers, Control panel, Debug panel rows, 8 dp apart. Not built.
  7. Rows, 8 dp apart, each 48 dp tall with 10 dp corners, start padding 16 dp and end padding 8 dp. The label is
     Nunito 500 16 sp, palette B, letter spacing -0.01 em. The trailing control is a standard Material 3 switch
     with the colours of `SwitchColors` (`jadx/xf/r.java`), not a 24 dp custom one, or an icon: `help` and `sign_out`
     are tinted with the unchecked thumb colour and followed by an 8 dp spacer. Rows: biometrics switch (only when
     biometrics can authenticate; not built), quick-note notification switch (drawn off, does nothing), "Get help or
     send feedback" (opens `https://mymind.helpscoutdocs.com`; ours does nothing), "Sign out of mymind" (ours signs
     out).
  8. Each row is a glass row (Haze): the style says blur 24 dp, the block overrides it to 5 dp, tint alpha 0.35 on
     the light theme and 0.55 on the dark theme of colour A, background colour A. Ours draws the row's own copy of the
     backdrop picture, blurred 5 dp, under that tint. The noise factor isn't drawn.
  9. Danger zone block: 10 dp corners, fill `#F90019`, padding 6 dp; the centred label "DANGER ZONE" is Nunito 500
     11 sp `#67000A`, line height 15 sp, letter spacing 0.48 em; inside, a 48 dp button, 6 dp corners, fill `#C00013`,
     "DELETE MY ACCOUNT" white Nunito 700 14 sp, capitals, letter spacing 0.1 em. The button opens S15. Ours is
     drawn and does nothing.
  10. Footer: a vertical column with "mymind® v<version>" (`ig/o.java:126`, Nunito 12 sp palette z) above the 24 dp
     `mymind_badge`, 8 dp apart; 16 dp spacers. Ours prints `versionName (versionCode)`.
- **Not determined:** the sheet's corner radius and tonal elevation (library defaults are used), whether the default
  window insets pad the sheet so its picture stops short of the system bars, row separators, the real names behind
  palette A, B, C values on screen.

## S15 Account deletion

- **Reached:** settings, "DELETE MY ACCOUNT" button. Not captured (see Capture list).
- **Kind:** native (`jadx/bg/a.java`). **Ours today:** none.
- **Facts:** background `#FF003D`; 32 dp side padding; "DANGER ZONE" Nunito 16 sp (f21548e), letter spacing 0.25 em,
  line height 15 sp; "Deleting is permanent" Louize 50 sp; "Closing your account will permanently delete all your
  data. There is no return." 20 sp; "If you are 100% sure, please type DELETE below to confirm." 20 sp with the word
  DELETE emphasised; an 8 dp-corner text field (16 sp); the confirm button ("Delete account", 14 sp capitals, letter
  spacing 0.1 em) is at 50% alpha until the word is typed.
- **Not determined:** the look of the button and field; navigation after success ("Your account has been
  successfully deleted").

## S16 Icon picker

- **Reached:** settings, icon row.
- **Kind:** native (`jadx/fg/n.java` function k, `fg/k0.java`). **Ours today:** none.
- **Facts:** 26 launcher icons: the default `ic_mellow_mind` and 25 activity aliases (`decoded/AndroidManifest.xml`).
  Alias icon names: monument_mind, connoisseur, hygge_mind, riddler, enlightened_mind, romantic, stoic_mind, old_soul,
  muse, hologaze, astral_walker, dark_souls, artist, og, minimalist, eclipse, zen, optimist, wise_mind, type_lover,
  dreamer, crystalized, skeuobaby, og_dark, og_light. Foreground and background layers are
  `decoded/res/drawable-xxxhdpi/ic_<name>_fg.png` and `_bg.png`. A dialog "Change icon?" says "Selecting a custom icon
  will close the app to apply the change." (`decoded/res/values/strings.xml:58-59`).
- **Not determined:** grid layout, sizes, dialog layout. Each pick changes the launcher icon, so none are tapped.

## S17 Upgrade view and upsell modal

- **Reached:** the settings upgrade button or "See our plans". The server can also push an upsell (`showUpsell`).
- **Kind:** upgrade view is a WebView (`UpgradeView`) with a Rive animation `upgrade_your_mind.riv`
  (`jadx/sf/g.java`); the upsell modal is native (`jadx/cg/l.java`). Play Billing runs from `iap:checkout`.
  **Ours today:** none.
- **Upsell modal facts:** title Louize 32 sp, line height 1.02 em, letter spacing -0.05 em, palette u; a picture from
  a server URL (day or night variant picked with the system theme), full width, with aspect from the server body
  text Nunito 16 sp, line height 1.3 em, letter spacing -0.05 em, palette v; 8 and 16 dp gaps; feature rows with
  icons (`upsell_icon_play` for play) coloured palette g or w/x; 24 dp gaps.
- **Not determined:** the rest of the modal, the Rive artboard, the plan page layout. Not captured (billing).

## S18 Web error screen

- **Reached:** the WebView status becomes `Error` (page failed to load).
- **Kind:** native (`jadx/rf/b.java:166-246`).
- **Ours today:** `ui/errors/WebErrorScreen.kt`, shown over the library when the first page load fails and there are
  no cards. Screenshots: `web_error_{light,dark}.png`.
- **Facts:** full screen in palette i; centred column with 32 dp padding. "Something went wrong" Nunito 20 sp,
  palette h, centred; 8 dp gap; "We couldn't load your content.\nPlease check your connection and try again." 14 sp,
  70% of palette h, both Nunito 400 (`rf/b.java:225-228`); 24 dp gap; retry button with 8 dp corners, palette g fill,
  white content colour, label "Retry" in 16 sp with padding 16 x 4 dp (`jadx/a0/b.java:325`).
- **Not determined:** nothing that blocks the layout.

## S19 Offline screen

- **Reached:** the device is offline.
- **Kind:** native (`jadx/fg/n.java:2404-2480`).
- **Ours today:** `ui/errors/OfflineScreen.kt`, drawn over everything while `data/Connectivity.kt` reports no
  network. Screenshots: `offline_{light,dark}.png`.
- **Facts:** full-screen clickable background, which absorbs taps: `offline_bg.jpg` (light, 375 x 812) or
  `offline_bg_dark.png` (dark, 750 x 1624) (chosen in `xf/m.java`, `xf/l.java`), cropped to fill; image
  `offline_notice.png` (704 x 492, `drawable-nodpi`) with 96 dp side padding; text "Please reconnect to the
  internet\nto use the mymind app." in avenir (f21549f) 18 sp, line height 24 sp, letter spacing -0.01 em, white,
  centred, 48 dp side padding; the column is centred on the screen, spaced 16 dp.
- **Not determined:** whether the clickable background shows a ripple or is disabled (ours has no ripple).

## S20 API error dialog

- **Reached:** a native API call fails (`jadx/rf/b.java:60-164`, state `jadx/g1`-style two-string holder).
- **Kind:** Material dialog with title and message from the error, confirm button.
- **Ours today:** `ui/errors/ApiErrorDialog.kt`, a default Material 3 alert dialog with an "OK" button (`ok` in
  `strings.xml`). Sign-in failures use it with the title "Sign in failed" (`sign_in_failed`). Screenshots:
  `api_error_dialog_{light,dark}.png`.
- **Texts that exist:** "Something went wrong, code %s.\nTry again or contact help@mymind.com", "Ouch, that hurt.",
  "Whoa, that's big." (big file), "That's too much!" (more than 20 items), "We can't get in!" (PDF), "Please log
  in!" (`decoded/res/values/strings.xml`).
- **Not determined:** dialog layout (default Material 3 alert dialog colours were not found).

## S21 Photo source popup

- **Reached:** the page's file input, which the + on Everything and inside a space opens: the WebView's
  `onShowFileChooser` (`jadx/yf/j.java`) shows the popup (`fg/l0.java` calling `rf/b.java`), not the system chooser.
- **Kind:** native (`jadx/rf/b.java:248-385`). **Ours today:** `ui/upload/AddMediaPopup.kt`, opened by our + buttons
  (`MainShell`), Back closes it. "Photo library" opens the system photo picker for several images and videos, "Take
  photo" the camera (row shown only when a camera app answers `IMAGE_CAPTURE`, declared in the manifest's
  `<queries>`). What follows is in S06 ("Uploads"). Screenshots: `S21_add_popup{,_no_camera}_{light,dark}`. Not
  compared: the capture has no shot of it.
- **Facts:** black 70% scrim over the whole screen (tap closes). The popup is aligned top end, width 60% of the
  parent, 16 dp padding, palette i fill, 8 dp corners. Rows: "Photo library" with `baseline_photo_library_24`, then a
  divider (palette j), then "Take photo" with `baseline_camera_alt_24` (second row only when a camera app exists).
  Row text Nunito 16 sp weight 500, palette h; icon palette h.
- **Not determined:** whether the 60% is a fraction of the screen or of an intrinsic width (ours: of the screen).

## S22 Share sheet

- **Reached:** from another app's share menu (not from launch).
- **Kind:** own activity `feature.share.NewShareSheetActivity` (`decoded/AndroidManifest.xml`; layout in
  `jadx/jg/n.java`, `jg/*`). **Ours today:** none.
- **Facts:**
  - Background picture `share_background.png` (1290 x 1934, `drawable-xxxhdpi`). Title Louize 38 sp.
  - Three tabs with icon and label: Tags, Notes, Spaces. Label Nunito 20 sp, line height 28 sp, letter spacing 0.02
    em; icon `tags`, `notes`, `spaces`; tab row spaced 8 dp, 4 dp between icon and label.
  - Notes panel: top 16 dp corners.
  - Tag input: Nunito 17 sp weight 400, line height 24 sp, 8 dp corners, 20 dp padding; hint "Hit next to add separate
    tags." (15 sp / 20 sp).
  - Space row: 48 dp tall, 8 dp corners, label Nunito 18 sp, line height 32 sp, `checkmark` icon at the end.
  - Save button at the bottom centre; success and failure states: failure shows `sad_mymind` with Louize 45 sp and
    Nunito 22 sp (line height 26 sp) and 17 sp buttons.
  - An upload strip at the bottom: slides in, height 80 dp up to 152 dp with the keyboard (`jg/n.java:681`).
- **Not determined:** all other states; share runs outside the main app, so it is not captured (sharing is not
  allowed during capture).

## S23 Video player

- **Reached:** the shell starts it with extra `videoUrl` when a video is played (`jadx/cg/h.java:136`, new task flag).
  Own activity `VideoPlayerActivity`, landscape, no history, excluded from recents (`decoded/AndroidManifest.xml:223`).
  It closes at once when `videoUrl` is missing.
- **Kind:** native Compose in its own activity (`jadx/com/mymind/access/activities/VideoPlayerActivity.java`,
  `jadx/ze/d.java`). **Ours today:** none.
- **Facts:** a full-size box with a black background and centred content; the player is a 16:9 box (aspect
  1.7777778) that starts playing at once (`ze/d.java:61-100`).
- **Not determined:** the controls drawn by the player component (`jadx/ze/*`), the buffering look.

## S24 Permission dialog, biometric prompt, quick-note notification

- Notification permission dialog (title "Allow quick note notifications", text "Quick note uses notifications so you
  can start writing right from your status bar.", buttons "Allow" and "Not now"): app dialog, layout not
  determined.
- Biometric prompt: system prompt, title "Biometric authentication to enter mymind". Labels "Use biometrics" and "Use
  biometrics or screen lock" exist.
- Quick-note notification: title "Have an idea?", action "Tap to start writing", input hint "Type a note", saved text
  "Saved to mymind", empty text "Ready when inspiration strikes." (`decoded/res/values/strings.xml:183-189`). Layout
  is the system notification; not determined.
- **Ours today:** none.

## S25 Note editor and note composer

- **Reached:** from the card page or the Everything tab's note button (messages `noteComposer:peekOut`,
  `noteComposer:hide`, `noteEditor:change`, `noteEditor:focused`).
- **Kind:** WebView `EditorView` (`jadx/eg/*`, `b0/w3.java` case 4). The editor bubble menu is hidden on Android
  (`app.css:16447`).
- **Ours today:** none. **Not determined:** layout; capture needs typing, so it is not captured.

## S26 Account closed

- **Reached:** the WebView loads a URL containing `/closed`; the shell then closes the view (`jadx/yf/f.java`).
- **Ours today:** none. Not reachable without deleting; not captured.

## S27 Dev screens

- Dev panel (`fg/n.java` function f), account debug (`fg/n.java` function a, padding top 16), wallpapers viewer. They
  open from dev-only rows in settings. Not part of a release look; not determined and not captured.

---

## Library screens: API gaps, real-app bugs, things left out

The app reads the web routes that already carry data before calling anything a gap: `GET /everything` (config
script: `colorScheme`, `features`, `isGuest`), `GET /search/suggestions`, `GET /tags/suggest`, `GET /app/v1/tags`,
`GET /tags/recent`, `GET /media/{path}`, `GET /account.json` (quotas only), and the `/app/v1` card and space JSON
(`url`, `tags` with ids, `spaces`, `cover` and `blob` with `dominantColor`, `pinPosition`). Not gaps, then: the
theme, the plan features, search suggestions, tags, space membership, `source.url` (the card's `url`), a space's
preview card (the first card of `/app/v1/spaces/{id}/cards`, the same data the web's space list shows) and a card
colour. No route carries these (listed, not added):

- The user's name (S14 greeting) and plan name (S14 "YOUR PLAN"): not in `/app/v1/me`, `/account.json` or the
  page config.
- `GET /cards/{id}/markdown` (the web's Copy) is served by Rust, including for app bearer tokens; its formatting is
  inferred from the web client, not a recorded response. Ours still copies the note text and should use it.
- `GET /cards/{id}/download` (the web's Download) remains unserved. Ours downloads the original from `/media/`.
- The reader text for Reader Mode.
- Platform and site logos, and the card web page itself.

Closed in round 3 by the server additions in `docs/app-api.md`: the Top of Mind list (`GET /app/v1/top-of-mind`),
uploads with the app's token (the upload token moves to `x-upload-token`), and the Serendipity deck (the ids in the
partial `GET /serendipity` page, which the app's token can read).

Behaviour of the real app that looks like a bug, noted and not copied:

- The search bar vanishes after switching tabs and coming back to Everything, until a cold start
  (`06-everything-return-light.png`, `06-everything-return-2-light.png`, capture README "Surprises").
- The search overlay stays after Back closes the keyboard, and a second Back leaves the app (`08-after-back-1-*.png`).
  Ours closes the overlay on that second Back.
- At rest in system light, the card header ink is dim over the dark card page (`12-card-article-top-light.png`, S10).

Left out until the user says yes (native extras or flows the real app has that need data changes):

- No predictive back. Haptics and pull to refresh are built (round 3, the user's yes): haptics only where the real
  app has them ("Haptics" above), pull to refresh as a Material 3 indicator (S06).
- No Trash screen, so no pull to refresh there; no Top of Mind reorder by drag, and no image viewer (S13).
- Uploads run in the view model, not in a WorkManager job, so they stop when the app process ends (S06).
- Inert: the composer peek (S25 not determined), Reader Mode (no reader text route), and renaming a space.
- The tab bar stays drawn while search is focused; the capture README says the real one hides, and with the
  keyboard up the keyboard covers it in both.

## Things in `docs/android-plan.md` that the code contradicts

1. Tab order: the real app has Everything, Spaces, Serendipity (`jadx/fg/n.java`); the plan lists Everything,
   Serendipity, Spaces. Ours now follows the real order.
2. "No native top bar": the card view has a native 64 dp header over the WebView (S10).
3. Serendipity opens `/clear`, not a path named serendipity (`jadx/wf/b.java`).
4. Settings is a native Material 3 navigation drawer opened from the web page's hamburger icon, not a tab, a
   bottom sheet or a screen of its own.
5. The window background at launch is pink `#EEB7C9`. Ours sets it in the launch theme.
6. The pictures `splash_bg`, `mymind_splash_icon`, `plans_guest`, `settings_background_*`, `offline_bg*`,
   `offline_notice`, `share_background`, `upgrade_dark`, `upgrade_light`, `icon_pile_*` and the vector `sad_mymind` are
   all in `decoded/res/drawable*`. They can reach the app through the Gradle copy task into the git-ignored
   generated folder, like the Lottie files. They must not be committed or sent to a hosted model, so their look is
   not described here.
7. The real default launcher icon is `ic_mellow_mind` (`decoded/AndroidManifest.xml:26`); 25 alias activities carry
   the other icons.

## Capture list

Navigation-only taps for reference screenshots on the phone (Samsung SM-F976B cover screen, 1080 x 2520, density 450).
The agent reads the screen and a UI dump before each tap and taps a named element. Never tapped: anything that
saves, deletes, edits, tags, pins, types, changes a setting, shares, buys or signs out. Every tap is logged. The user
keeps the phone unlocked and on the charger, with notifications cleared, and the app signed in. Take every light
screenshot first, then ask the user to switch the phone to dark mode (a device setting, the user's step) and repeat
step 21.

Naming: `screens/<step>-<name>-<light|dark>.png`.

| Step | Action | Screen | Shot |
|---|---|---|---|
| 1 | Close the app from recents, start it from the launcher; screen-record the first 4 s | S01, S05 loading | frames at 0.1 s, 0.5 s, 1.5 s, 3 s (splash, loading spinner) |
| 2 | Wait for the Everything tab | S06 at rest | `everything-top` |
| 3 | Scroll down one screen; scroll down five screens; scroll back to the top | S06 | `everything-scroll-1`, `everything-scroll-5` |
| 4 | Tap the Spaces tab | S07 | `spaces-top`; scroll one screen: `spaces-scroll-1` |
| 5 | Tap the Serendipity tab | S08 | `serendipity-top`; screen-record the entry |
| 6 | Tap the Everything tab again | S06 | confirms the tab icon animation (screen record 2 s) |
| 7 | Tap the search bar text area (not the icon); do not type | S09 | `search-focus` (with keyboard) |
| 8 | Press Back to close search | S06 | none |
| 9 | Tap the hamburger icon in the search bar | S14 | `settings-top` |
| 10 | Scroll the settings drawer by swiping up in steps without tapping any row | S14 | `settings-scroll-1`, `settings-scroll-2`, `settings-bottom` |
| 11 | Press Back | S06 | none |
| 12 | Tap a card that is an article (first one visible) | S10 | `card-article-top` |
| 13 | Scroll down 3 screens, then up one screen | S10 | `card-article-scroll-down` (header hidden), `card-article-scroll-up` (header shown with blur) |
| 14 | Tap the menu icon in the header | S11 | `card-menu` |
| 15 | Press Back | S10 | none |
| 16 | Tap an image inside the card | S13 | `image-viewer` |
| 17 | Press Back; tap the caret to close the card | S06 | none |
| 18 | Repeat steps 12 to 17 for an image card, a note card, a PDF card and a video card | S10, S11 | `card-<type>-top`, `card-<type>-menu` |
| 19 | From the video card, tap play, then press Back | S23 | `video-player` (landscape) |
| 20 | Spaces tab: tap the first space | space page | `space-top`, `space-scroll-1`; press Back |
| 21 | Dark mode: repeat steps 1 to 17 | all | same names with `-dark` |

Not captured on purpose, and why:

- Select a space, Add tags, icon picker, account deletion, billing and upsell, share sheet, note editor, photo popup:
  each needs a tap that saves, changes, types, buys, shares or opens a deletion flow. Their facts above come from the
  code only.
- Signed-out screens (splash with sign-in, onboarding, welcome, guest plan): need the user to sign out of the real
  app once; the user's call (`docs/android-plan.md`, "Reference screenshots").
- Offline screen: needs the phone to go offline (a device setting). Ask the user first.
- Web error screen, API error dialog: need a failure; not forced.

## Open questions, listed once

- How `plans_guest` and `share_background` are placed: fit, crop and size (S04, S22). The pictures themselves are
  in `decoded/res` and weren't viewed.
- Whether the keyboard or the search focus hides the real tab bar (S05).
- Open animation of the card menu and of the S12 pages inside it.
- Controls of the video player (`jadx/ze/*`) and the full-screen image viewer.
- Role names of several palette fields (m, p, t, D to K) and of the `y` MenuColors light values that are white or
  black.
- The web pages' own look at 384 dp width: read `mirror/` and the phone screenshots together.

Open questions from the native screens built in `android/app`:

- Password form (S02): the real app has none. It is ours, for the test user, and stays unless the user says to drop
  it.
- Settings greeting name (S14): the third line of the greeting is left empty. The code shows a name; the source of
  the name isn't known, and our server returns only the email.
- Plan (S14): the source of "YOUR PLAN" and the `free_plan` fallback aren't known, so the block is left out.
- Offline background (S19): ripple and enabled state of the clickable layer.
- Hamburger trigger (S14): settled. The icon is in the search bar (`02-everything-top-*.png`); ours is there too and
  its region passes the pixel compare (S06).
- Card header colour (S10): the real source is the card's own colour (`f6811f` hex). Ours uses the colour passed to
  `CardView`, which is the page colour; `/app/v1/cards` carries `cover.dominantColor` and `blob.dominantColor`, but
  whether the real header uses them is not determined (the S10 capture's header matches the page colour).
- Card header and the status bar (S10): settled. The card sheet starts 24 dp under the status bar
  (`12-card-article-top-*.png`), and the header sits at its top; ours matches (S10 compare).
- Settings sheet insets (S14): the greeting and the swipe hint land on the capture's pixels (S14 compare), so the
  content insets match; whether the picture reaches under the system bars was not checked.
- Settings sheet surface (S14): the library's tonal elevation and corner radius are used as they are.
- Glass rows (S14): the effective blur is 5 dp (the block override) against 24 dp in the style. The noise factor is
  left out. The transform origin the real code sets on the backdrop layer isn't reproduced.
- Omitted settings rows (S14): biometrics, the dev rows, the upsell list and the upgrade button. The quick-note
  switch doesn't ask for the notification permission.
