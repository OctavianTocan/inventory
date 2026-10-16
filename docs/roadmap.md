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
| 8 | **Accounts.** Google sign-in (OIDC + PKCE, hand-rolled), sessions in SQLite, `_cid`/`_jwt` cookies like mymind. | Sign in, sign out, second account sees an empty library. |
| 9 | **Android.** Kotlin WebView shell per `docs/android-plan.md`. | Runs on the phone against the replica. |

Every table carries `user_id` from phase 1 so accounts slot in without a migration.
