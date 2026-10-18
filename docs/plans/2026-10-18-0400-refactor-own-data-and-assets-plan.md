---
title: Run on our own data and assets, without mymind recordings - Plan
type: refactor
date: 2026-10-18
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Run on our own data and assets, without mymind recordings

## Goal Capsule

- **Objective:** the product answers every request from its own database, storage and code, and the Android app
  and the Chrome extension ship only our own fonts, icons and pictures, so nothing depends on data recorded from
  mymind.
- **Means:** move old cards' files into our storage, replace each recorded answer with one we build, replace mymind's
  icons and pictures with Phosphor and our own, keep the fonts whose licence lets us, and replace the others, then delete `replay/`, `capture/` and the
  tools that made them.
- **Done when:** a fresh clone with `replay/` and `capture/` absent serves the whole library locally and on Cloudflare,
  the app and the extension build with no file from `android/decoded` or `extension/mymind`, and the two folders and
  their tools are gone from Git.

## Product Contract

### Summary

Every card keeps its text, tags, notes and files, but the files live in our storage and every page and answer comes
from our own code. The app and the extension switch to Phosphor icons and our own pictures and icon; they keep Nunito (open licence), and the commercial fonts give way to open ones. The web
interface itself (`mirror/`) is the one remaining mymind part, and it gets its own plan.

### Problem Frame

The replica was built to match mymind 1:1, so whatever was not rebuilt was answered from recordings of live mymind:
`replay/` (10,346 recorded answers, 4.3 GB) and `capture/` (2,263 card detail pages, 172 reader views, about 4,800
media files, sign-in pages, the page shell; 2.2 GB). The database already holds every card's text, tags, notes and
spaces, but:

- old cards' pictures, videos and PDF pages are read from `capture/media`, `capture/mediahost` and `capture/carbon`
  (`rust/crates/core/src/hosted.rs:246-299`, `rust/crates/server/src/replica.rs:474-552`), and only in the sizes
  mymind rendered; there are no originals;
- every page load is mymind's recorded HTML, patched (`rust/crates/core/src/api.rs:800`, `api.rs:1022`); there is no
  page generator of our own;
- old cards' detail views, `PinnedCard` templates and reader views come from recordings (`api.rs:848`), as do
  `/news/unread`, `/components/*`, `/serendipity`, `/account/blocks/*`, `/ui` and `/achievements/unseen`;
- read-only search, suggestions and smart-space checks prefer recordings (`api.rs:804`);
- local sign-in serves mymind's recorded pages (`capture/signin/`, `rust/crates/server/src/serve.rs:19-38`);
- the native server does not start without `capture/` (`replica.rs:180-219`);
- the Android build copies 57 mymind files from the decoded mymind app (`android/app/build.gradle.kts:11-67`), and the
  extension packages mymind's icons, pictures and Nunito (`extension/mymind/`).

Any request the recordings do not cover fails or shows a placeholder, and the library cannot grow past what was
recorded. The owner wants a proper setup: our data, our storage, our code.

### Requirements

Data and storage

- R1. Every old card's files (pictures, videos, PDF pages, link screenshots) are stored the way new uploads are
  (`data/media` locally, `libraries/<lib>/media/` and `renditions/` in R2) and the card points to them.
- R2. No card keeps a link to a mymind host (`media.mymind.host`, mymind CDN URLs); the 33 that do are moved like R1.
- R3. The `recorded` and `api_metadata` tables are dropped once nothing reads them.

Answers

- R4. Every page load (`/everything`, `/things/:id`, `/spaces`, `/serendipity`, `/trash`, `/account`,
  `/achievements`, `/bookshelf`) is rendered by our code from a page template we own in the repository.
- R5. Card detail, `PinnedCard` and reader views are rendered from the database for every card, old and new.
- R6. Search, suggestions and smart-space checks always come from the database.
- R7. Each remaining recorded route (`/news/unread`, `/components/*`, `/serendipity`, `/account/blocks/*`, `/ui`,
  `/achievements/unseen`, `/manifest.webmanifest`) gets an answer from our code, or is dropped when the interface
  works without it.
- R8. Local sign-in uses the replica's own sign-in page, as tunnel and Cloudflare sign-in already do.
- R9. The native server and the Worker start and serve with `replay/` and `capture/` absent.

Removal

- R10. `replay/`, `capture/`, `library/` bodies tied to them, and the tools that record or import them
  (`tools/capture.mjs`, `capture-signin.mjs`, `replay-store.mjs`, `cdp.mjs`, `mirror-static.mjs`,
  `mymind search-recordings`, the recording parts of `import-hosted`, `seed-db` and `bundle-library`) are deleted.
- R11. Tests that read the recordings either run on synthetic fixtures or are deleted when they only checked
  recordings.

Assets (the full list is in Asset inventory below)

- R12. The Android app uses only fonts we may ship (Nunito, and an open serif for titles in place of Louize and
  Avenir), Phosphor icons for all icons, has its own launcher icon, and its
  build no longer reads `android/decoded`.
- R13. The extension uses Nunito and Phosphor, has its own toolbar icon set, and no longer packages `extension/mymind/`.
- R14. mymind pictures in the app and the extension are replaced by our own or removed, as the inventory says.
- R15. The app's theme colours are our own (mymind pink `#EEB7C9` in `themes.xml` goes).

### Key Decisions

- KD1. Fonts mymind uses stay where we may use them: Nunito (SIL Open Font License, Google Fonts) stays, taken from
  Google Fonts instead of the decoded app. Louize (205TF) and Avenir (Linotype) are commercial, so titles use Instrument
  Serif (SIL OFL, Google Fonts) and the offline screen uses Nunito. (session-settled: user-directed — chosen over
  switching everything to Geist: "if the fonts they use are fonts we can also use, just use em")
- KD2. mymind's marketing and help pages are not rebuilt. (session-settled: user-directed — chosen over rebuilding
  them: "we were not using their marketing or help pages")
- KD3. The web interface (`mirror/`) stays until its own plan replaces it. (session-settled: user-approved — chosen
  over replacing it here, because building an interface is a separate, larger project)

### Success Criteria

- With `replay/` and `capture/` moved out of the checkout, `mymind serve` on the working library and the hosted site
  as the test user show every card with its picture, open every card's detail and reader view, and search works.
- No request to the server returns a placeholder image or "Detail view not captured" for a card that existed.
- `grep -r` for `capture/`, `replay/`, `Recorded`, `android/decoded` and `extension/mymind` in code finds nothing.

### Scope Boundaries

- Not in this plan: the new web interface (KD3); the website keeps mymind's interface code, fonts and icons until then.
- Not in this plan: renaming the product, the package `com.octaviantocan.mymind`, or the hostname.
- Not in this plan: mymind's marketing and help pages (KD2).
- The extension's own code copies mymind's look (class names `mymind-notification-*`, wording such as "Type here to
  add a mind note..."); restyling it beyond fonts, icons and pictures belongs with the web interface plan.

## Planning Contract

### Key Technical Decisions

- KTD1. The page template becomes a checked-in source file, not data: one template for the page shell (from today's
  `capture/shell.html`, with recorded card data stripped) under `rust/crates/core/templates/`, filled by our code from
  the database. It still loads `mirror/` until the interface plan replaces it. Rationale: the shell is the only way
  the mirrored interface boots, and a template we own and render is a proper source, while a recorded response is not.
- KTD2. Old media migrates with a one-off command, `mymind migrate-media`, run once locally and once against the
  bucket, not through lazy fallbacks. It copies each `capture/` file to the key the card's `blob.path` and rendition
  names imply, picks the largest rendition as the original when no original exists, rewrites the 33 mymind-host links,
  and writes a report of every card it could not complete. Rationale: a one-off copy leaves no runtime branch reading
  `capture/`; lazy fallback would keep the dependency.
- KTD3. Detail and reader views use `render_detail` for all cards. Before switching, a check compares the text in each
  recorded detail and reader page with what `render_detail` produces from the database, and any content that exists
  only in the recording (for example article reader text) is imported into the database first.
- KTD4. Routes in R7 are answered by small handlers returning the empty state the interface accepts (no news, no
  achievements, the account blocks for a self-hosted account), each checked in the browser; a route the interface
  never calls is dropped.
- KTD5. Phosphor icons enter Android as vector drawables converted from `@phosphor-icons/core` 2.1.1 SVGs (about 35,
  Regular weight), keeping every `painterResource` call; no community Compose library, since none is official.
  Nunito comes from Google Fonts as the variable TTF (`res/font/nunito.ttf`), keeping the existing
  `FontVariation.weight` code in `ui/theme/Fonts.kt`; Instrument Serif replaces Louize. The extension uses the same
  SVGs and a Nunito woff2 from Google Fonts. Licence texts ship next to the font files.
- KTD6. The Lottie tab-bar animations become still Phosphor icons and the loading animation a
  `CircularProgressIndicator`, so the Lottie dependency goes.

### High-Level Technical Design

Request order today and after (directional):

```
today:  sign-in -> Api (DB) -> recorded answer -> mirror/static -> capture media -> placeholder
after:  sign-in -> Api (DB + templates) -> mirror/static -> 404
```

### Asset inventory

Android (`android/app/build.gradle.kts:11-67` copies them from `android/decoded/res/`):

| Asset | Used in | Replacement |
|---|---|---|
| `nunito.ttf` (variable, weights 400/500/600/700, about 61 uses in 24 files) | `ui/theme/Fonts.kt` | Nunito from Google Fonts (OFL), same file kind and weights |
| `louize_regular.otf` (titles: `Tiles.kt`, `CardSheet.kt` 31 sp, `SettingsSheet.kt` 48 sp, `ShareSheet.kt` 45 sp, sign-in, `NewSpaceDialog`) | `Fonts.kt` | Instrument Serif (OFL) (KD1) |
| `avenir_roman.otf` | `OfflineScreen.kt:76` | Nunito |
| About 28 icon drawables (card menu, share sheet, submenus, upload strip, settings, add-media popup) | per screen | Phosphor, e.g. delete -> Trash, download -> DownloadSimple, tags -> Tag, sign out -> SignOut |
| 6 tab-bar Lottie files | `Tab.kt:14-16` | Phosphor icons (KTD6) |
| `load_spin.json` | `LoadingSpinner.kt:29` | `CircularProgressIndicator` |
| `splash_bg.png`, `mymind_splash_icon.png` | launch | our icon on a plain background |
| `share_background.png` | share sheet | removed (plain surface) |
| `offline_bg.jpg`, `offline_bg_dark.png`, `offline_notice.png` | offline screen | removed; Phosphor WifiSlash |
| `settings_background_light/dark.png`, `icon_pile_light/dark.png`, `mymind_badge.png` | settings | removed |
| `sad_mymind.xml` | share error | Phosphor SmileySad |
| `ic_upload_progress.png` | upload strip | Phosphor CircleNotch, spinning |
| `upgrade_dark/light.png` | upgrade sheet | removed with the upgrade content |
| launcher icon (none set today) | manifest | new adaptive icon (needs a design) |
| `themes.xml` `#EEB7C9` | theme | our accent colour (needs a choice) |

Not mymind's, kept: `svg_google.xml` (Google's sign-in logo), `baseline_*` Material icons.

Extension (`extension/mymind/`, copied by `extension/tools/copy-mymind-assets.mjs`):

| Asset | Used in | Replacement |
|---|---|---|
| `icon_16/48/128/256.png` | `manifest.json:7-15` | our icon set (16, 32, 48, 128) |
| `logo.svg` | `notification.js:186,235` | our icon |
| loading start and finish GIFs | `worker.js:209,242` | Phosphor CircleNotch and CheckCircle |
| `nunito.woff2` | `drawer.css:2-3`, `notification.js:277`, `notification.css:12,166` | Nunito woff2 from Google Fonts; Instrument Serif where `notification.css:153` names Louize |
| `full-mind-bg.png`, `become-a-mastermind.svg` | upgrade block, `notification.js:~232-245` | removed with the block |
| `close.svg`, `plus-*.svg` | unused | deleted |
| `vendor/tiptap.min.js` | note editor | kept (Tiptap and ProseMirror, MIT) |

### Assumptions

- The largest recorded rendition is good enough as an old card's original; mymind's originals are not available.
- The interface accepts empty answers for news, achievements and account blocks (checked per route in U5).

## Implementation Units

- U1. **Media migration command.** Add `mymind migrate-media` in `rust/crates/server/src/tool_migrate_media.rs`
  (KTD2): local mode copies `capture/media|mediahost|carbon` into `data/media` and renditions; bucket mode does the
  same through the admin bucket routes; both rewrite mymind-host links and write a report. Tests:
  `rust/crates/server/tests/migrate_media.rs` on a synthetic library (a card with renditions only, a card with a
  `media.mymind.host` link, a missing file that lands in the report, a second run that changes nothing).
- U2. **Media from our storage only.** Remove the `capture/` fallbacks and placeholders from `hosted.rs:246-299`,
  `api.rs:756-759` and `replica.rs`; a missing file answers 404. Depends on U1 having run on both libraries. Tests:
  update `rust/crates/server/tests/media.rs`, `check_hosted_media.rs`, `rust/crates/core/tests/hosted_media.rs`.
- U3. **Page template.** Add the shell template (KTD1) and render it in `Api::shell`; page loads stop reading the
  recorded `/everything` or `shell`. Tests: page loads for each route in R4 render with the signed-in user's data and
  load `mirror/` scripts; checked in a browser locally.
- U4. **Detail, reader and PinnedCard from the database.** Run the KTD3 comparison, import missing content, then drop
  the recorded branches at `api.rs:848` and the `PinnedCard` and reader routes. Tests: `render_detail` for an old
  article card, an image card, a PDF card and a link card; reader view for an article card.
- U5. **Remaining routes and search.** Handlers for R7 (KTD4) and search, suggestions and checks always from the
  database (`api.rs:804`). Tests: each handler's answer; a browser pass through every page, the new-space modal and
  serendipity with no console errors.
- U6. **Own sign-in everywhere.** Local sign-in uses `plain_signin_page` (`auth.rs:306-317`); remove `FilePages` and
  `capture/signin`. Tests: update `rust/crates/core/tests/auth.rs` for local sign-in, account-not-found and sign-out.
- U7. **Start without recordings.** Delete `Replica` and `Recorded`, the replay copy into `api_metadata`
  (`read_metadata.rs`), drop both tables with a migration, and remove the recording parts of `import-hosted`,
  `seed-db`, `bundle-library` and `install-library`. Tests: `mymind serve` and the Worker under `wrangler dev` start on
  a checkout without `replay/` and `capture/`; `tools/check-reads.mjs`, `check-auth`, `check-app-api`,
  `check-download`, `node extension/tests/e2e.mjs` pass on a synthetic library.
- U8. **Tests off the recordings.** Move the tests that copy or hash `capture/`, `mirror/` and `replay/`
  (`rust/crates/server/tests/common/mod.rs:47`, `rust/crates/core/tests/support/recordings.rs`, `check_render.rs`,
  `check_store.rs`, `check_search.rs`, `site.rs`, `capture_html.rs`, `api.rs`) to synthetic fixtures; delete
  `tool_search_recordings.rs` and its test.
- U9. **Delete the recordings.** Run U1 on production, check the hosted site, then remove `replay/`, `capture/`, the
  tools in R10, and their docs in `README.md` and `AGENTS.md`; remove the bucket's `capture/` prefix after the hosted
  check passes. Git history keeps the old data; the working tree drops 6.5 GB.
- U10. **Android fonts, icons and pictures.** Add Nunito and Instrument Serif from Google Fonts and the Phosphor drawables (KTD5), update `ui/theme/Fonts.kt`
  and every screen in the inventory, replace the Lottie uses (KTD6), add the launcher icon and theme colour, delete
  `copyMymindResources` from `android/app/build.gradle.kts`. Tests: the Roborazzi screenshot tests in
  `android/app/src/test/` re-recorded and reviewed; `assembleRelease` with `android/decoded` moved away; an emulator
  pass through launch, library, card, share and settings.
- U11. **Extension fonts, icons and pictures.** Add the icon set, Nunito and Instrument Serif from Google Fonts and the Phosphor SVGs, remove the upgrade
  block and unused files, delete `extension/tools/copy-mymind-assets.mjs`, and make `deploy/publish-extension.sh`
  package only our files. Tests: `node extension/tests/units.mjs`, `security.mjs`, `e2e.mjs`, and a check that the zip
  holds nothing from `extension/mymind/`.

Order: U1 -> U2; U3, U4, U5, U6 in any order; U7 and U8 after them; U9 last. U10 and U11 are independent of the rest.

## Verification Contract

- Rust: `cd rust && nice -n 19 cargo test --offline -j 2 -- --test-threads=2`, `cargo clippy --all-targets -j 2`.
- Node checks: `tools/check-reads.mjs`, `check-auth.mjs`, `check-app-api.mjs`, `check-download.mjs`,
  `tools/check-hosted.mjs --browser` (as the test user, after each deploy).
- Android: `nice -n 19 ./gradlew --max-workers=2 testDebugUnitTest assembleRelease`, emulator pass.
- Extension: `node extension/tests/units.mjs`, `security.mjs`, `e2e.mjs`.
- Browser: every page of the local server and the hosted site, light and dark, at 1440 and 390 px, with no console
  error and no placeholder image.

## Definition of Done

- Every requirement R1-R15 holds and the Success Criteria checks pass on the local server and production.
- U1's report lists no card without its files, or the user has accepted each listed card.
- Each unit is committed and pushed on its own; production is deployed after U2, U3-U6 and U9, each followed by
  `check-hosted --browser`.

## Risks & Dependencies

- Old cards lose quality where mymind only gave a small rendition; U1's report names them.
- Content that exists only in recorded detail or reader pages would be lost without KTD3's comparison and import.
- The page template still carries mymind's markup until the interface plan; it moves from recorded data to source,
  not out of the product.
- A launcher icon, a toolbar icon set and an accent colour need a design decision before U10 and U11 finish.

## Sources & Research

- Recording reads and sizes: `rust/crates/core/src/hosted.rs`, `api.rs`, `recorded.rs`,
  `rust/crates/server/src/replica.rs`, `serve.rs`, `read_metadata.rs`, `tool_import_hosted.rs`; `replay/index.json`.
- Assets: `android/app/build.gradle.kts`, `android/app/src/main/kotlin/com/octaviantocan/mymind/ui/`,
  `extension/manifest.json`, `extension/tools/copy-mymind-assets.mjs`.
- Phosphor: github.com/phosphor-icons/core (MIT), `@phosphor-icons/core` 2.1.1, `@phosphor-icons/web` 2.1.2.
- Nunito and Instrument Serif: fonts.google.com (SIL OFL 1.1). Louize: 205TF, commercial. Avenir: Linotype, commercial.
