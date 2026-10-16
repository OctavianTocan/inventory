# Roadmap: from replica to working app

Today the replica shows a recorded library and ignores writes. The goal is a mymind you can use: saving, editing,
search and AI, in a desktop app, without changing a pixel of what already matches live.

| # | Phase | Proof it works |
|---|---|---|
| 1 | **Store and renderer.** SQLite (`data/mymind.db`, `lib/store.mjs`) seeded from the recordings; `lib/render-card.mjs` renders a card's HTML from its JSON the way mymind's server does. | `tools/check-render.mjs`: every recorded card re-rendered byte for byte. |
| 2 | **Reads from the store.** `GET /cards`, `/cards/:id`, trash, spaces, tags, `/changes` come from SQLite instead of replay/. | Narrow REA patch suites still pixel-identical. |
| 3 | **Writes.** Notes, titles, card notes, tags, highlights, spaces, pins, trash/restore, settings (`docs/write-api.md`), with the `/changes` log and `/hub` pushes so the grid updates live. | Scripted save/edit/delete flows in the browser; screenshots of each new state. |
| 4 | **Link capture and uploads.** Fetch page metadata, screenshot the page, detect the card type; image/PDF uploads into `data/media/`. | Saved links render like recorded ones of the same type. |
| 5 | **AI.** On save: `gemini-embedding-2` embedding (text and images), summary and tag suggestions from a cheap Gemini chat model, final tag picks by TypeSafe Jev from the library's tag vocabulary. Semantic search on `/embeddings/*`. | Tags and summaries on new cards; similar-card search. |
| 6 | **Search.** SQLite FTS5 plus mymind's query syntax (`tag:`, `site:`, `type:`, dates, `-x`, quotes). | Recorded searches return the same matches. |
| 7 | **Desktop app.** Tauri 2 window around the replica's browser UI (`desktop/`), starting the server when needed. No `mymind.native` bridge: on desktop it would hand views to native chrome that only exists in mymind's Mac app. | App launches, saves, drags files in. |
| 8 | **Accounts.** Owner-only Google sign-in on mymind's own sign-in page (OIDC + PKCE, hand-rolled), sessions in SQLite; off unless `AUTH=google` (`docs/accounts-plan.md`). Built. | `tools/check-auth.mjs` against a stand-in Google. |
| 9 | **Android.** Native Kotlin app, every screen pixel-identical to mymind's Android app, talking only to our library (`docs/android-plan.md`). | REA screenshots of each screen match the real app showing the same data. |
| 10 | **Hosted library.** This computer serves the library through a Cloudflare tunnel with sign-in on; the desktop and phone apps open it remotely. | Sign in from another machine; edits show up on both. |
| 11 | **Native desktop (GPUI).** Rust GPUI client once the web replica is the reference. | Same REA screenshots as the web replica. |
| 12 | **Browser extension.** Save pages and selections from Chrome. | Saved items render like ones saved from mymind's extension. |

Every table carries `user_id` from phase 1 so accounts slot in without a migration.

Order after phase 8, as the user set it: phone app, then GPUI, then the browser extension. The hosted library comes
first because the phone app needs a server it can reach.
