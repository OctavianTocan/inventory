# App API: what the native Android app talks to

The Kotlin app (`android/`, roadmap phase 9) uses the same server and library as the web front-end. It signs in for
a bearer token, reads cards as typed JSON from `/app/v1/*`, and writes through the existing write API
(`docs/write-api.md`) with the same token. The web keeps its HTML; this JSON comes from the same stored rows
(`cards.data` and `cards.extras`, documented on `render_card` in `rust/crates/core/src/render_card.rs`).

Status (2026-10-16): contract written; server read and sign-in endpoints implemented and proved by `tools/check-app-api.mjs`; app being built on branch `android-app`.

Implementation notes: `note` is the note's plain text. Read errors are `400` with `bad-limit`, `bad-cursor`, `missing-query` or `bad-request`; non-GET reads get `405`; unknown cards and spaces get `404 {"error":"not-found"}`. A space's cards omit trashed and grid-hidden cards. Search cursors are offset-based. Sign-out needs a valid bearer token (`204`, otherwise `401`). `GET /app/v1/me` exists only with sign-in on. It sends no `name` or `plan`: the stored `api_metadata` has no `/account.json` row, the recorded config script carries `account` with only `uid`, and the user settings hold neither. `colorScheme` and `features` come from the config script of `GET /everything`, which the server fills from the user's settings, the same source the web page reads. `APP_NONCE_MS` overrides the nonce lifetime for tests.

## Sign-in

Only with `AUTH=google`. The same people the web admits: `OWNER_EMAIL`, a verified address whose host is exactly
`ALLOWED_GOOGLE_DOMAIN` (unset means `octaviantocan.com`; empty disables domain admission), exact verified addresses
in the comma-separated `ALLOWED_GOOGLE_EMAILS` invitation list, and password users from
`mymind auth-user`. One function, `google_access` in `rust/crates/core/src/auth.rs`, decides for both the web callback
and `/app/v1/auth/google`. `email_verified` has to be boolean true. A subdomain or a lookalike suffix is rejected.

| Request | Body | Answer |
|---|---|---|
| `POST /app/v1/auth/nonce` | none | `200 {"nonce": "<base64url>", "expiresAt": "<ISO>"}`. Single use, valid 10 minutes. |
| `POST /app/v1/auth/google` | `{"idToken": "<JWT>", "nonce": "<from /nonce>", "device": "<label>"}` | `200` token answer; `401 {"error": "not-allowed"}` for a verified Google account without access; `400 {"error": "invalid-token"}` for anything else (bad signature, audience, issuer, expiry, unknown or reused nonce, unverified email). |
| `POST /app/v1/auth/password` | `{"username": "...", "password": "...", "device": "<label>"}` | `200` token answer; `401 {"error": "invalid"}`; `429 {"error": "locked"}` (the web form's limits, shared with it). |
| `POST /app/v1/auth/signout` | none, `Authorization: Bearer` | `204`; the token stops working. |
| `GET /app/v1/me` | none | `{"email": "...", "kind": "google" \| "password"}`. |
| `GET /app/v1/agents` | none | Connected agents: `{"agents": [{"id", "name", "connected", "used"}]}`, the account's live app tokens that Inventory's MCP server holds (device `MCP: <name>`), most recently used first. Sign-in on only. |
| `POST /app/v1/analytics/query` | `{"sql": "SELECT …"}` | Owner only (`OWNER_EMAIL`), `404` for everyone else: one bounded question over the usage events (`docs/analytics.md`). |
| `POST /app/v1/events` | `{batch, version, events: [...]}` | The phone's own usage events, at most 100, unknown names dropped; answers `{kept}`, plus `stop: true` while analytics is off (`docs/analytics.md`). |
| `GET /app/v1/analytics/daily?since&library` | none | Owner only, `404` for everyone else: the kept daily library totals (`docs/analytics.md`). |
| `DELETE /app/v1/agents/<id>` | none | Signs one of the account's agents out (`204`); `404` for an id that is not one of its agents. The MCP server drops its grant on that agent's next call. |

Token answer: `{"token": "<opaque base64url, 256 bits>", "expiresAt": "<ISO>", "user": {"email": "...", "kind": "google" | "password"}}`.

- The ID token comes from Android Credential Manager (Sign in with Google) with `serverClientId` set to the Web
  client ID, so its `aud` is `GOOGLE_CLIENT_ID`; the server checks it exactly as the web callback does (RS256 against
  Google's keys, issuer, audience, expiry, `email_verified`, nonce).
- Tokens are stored hashed (SHA-256) in `data/auth.db` table `app_tokens`, like `sessions`, with the device label.
  They expire 90 days after last use. Removing a password user ends their tokens too.
- Every route the session cookie opens also accepts `Authorization: Bearer <token>`: API, media and the `/hub`
  WebSocket. A missing or dead token gets `401` JSON, which tells the app to sign in again. An open `/hub`
  connection is closed before its next event or ping once its token or session has ended.
- Google's signing keys are refetched for an unknown key id at most once a minute, and parallel password guesses
  count against the lockout before the password is checked.
- Through the tunnel only the three sign-in POSTs are open without a token.

## Cards

Media URLs are paths on the server (`/media/...`); the app resolves them against the server origin and sends its
bearer token.

| Request | Answer |
|---|---|
| `GET /app/v1/cards?limit=50&cursor=<c>` | `{"cards": [Card], "next": "<cursor>" \| null}`, the grid's order (newest bumped first) and filter (no trashed or grid-hidden cards). `limit` 1–200. |
| `GET /app/v1/cards/<id>` | `Card`, or `404 {"error": "not-found"}`. |
| `GET /app/v1/trash?limit&cursor` | Same list shape, trashed cards, most recently deleted first. |
| `GET /app/v1/spaces` | `{"spaces": [Space]}` in the user's order. |
| `GET /app/v1/spaces/<id>/cards?limit&cursor` | Same list shape, the space's cards in its order. |
| `GET /app/v1/tags` | `{"tags": [{"name": "...", "count": 12}]}`, the same order as the web's tag list. |
| `GET /app/v1/search?q=<query>&semantic=1&limit&cursor` | Same list shape, mymind's query syntax (`docs/search.md`). With `semantic=1` the order also weighs each card's similarity to the query's words, and up to 20 cards close in meaning join without a matching word (`Search::hybrid`); without it, words only. |
| `GET /app/v1/top-of-mind?limit&cursor` | Same list shape, the pinned cards in pin order (`pinPosition`, then newest bumped first for pins without a position). Trashed and grid-hidden cards are left out. Paged by offset, like a space. |
| `GET /app/v1/changes?since=<token>&limit=100` | `{"cards": [Card], "removed": ["<id>"], "next": "<token>", "more": bool, "reset": bool, "spaces": [Space]}`: the cards changed since `since`, each once, oldest change first (see below). `limit` 1–200, default 100. |
| `GET /app/v1/sky?day=YYYY-MM-DD` | Back today and the sky (see below). `day` is the phone's date, default today in UTC; `400 {"error": "bad-day"}` for anything else, `503 {"error": "read-only"}` on a read-only library. |

### Syncing a copy of the library

`/app/v1/changes` lets the phone keep the whole library and fetch only what changed.

- Without `since`, the feed lists every card, oldest bumped first. Read pages until `more` is false, passing each answer's `next` as the following request's `since`.
- `cards` holds the changed cards, in the same shape as `/cards`. `pinPosition` comes from the pin order that Top of Mind uses. Cards hidden from the grid are included, and the phone filters them out as the grid does.
- `removed` holds the ids of cards trashed or deleted since. `spaces` (the `/spaces` list) is only on the last page.
- Any write to a card counts as a change: its row, tags, text, spaces or pin. `app_sync.rs` keeps the log with triggers, and it starts on the first request.
- `reset: true` means the log started again, for example after the library was replaced by an import, so the token's position no longer applies. Drop the copy, keep the cards in this answer, and carry on paging.
- Tokens also name the card shape (`CHANGES_FORMAT` in `app_api.rs`). When cards gain a field the phone's copy needs, as `spaceRanks` did, the format changes and older tokens answer `reset: true`, so a copy kept before fills again once.

### Back today and the sky

`/app/v1/sky` (`sky.rs`) brings old finds back, a few a day, and keeps the sky they light.

```json
{"day": "2026-11-05", "back": ["<id>", "<id>", "<id>"],
 "active": {"id": 3, "figure": "ship", "name": "The Voyager", "seed": "<id>", "cards": ["<seed>", "<id>", "..."],
            "lit": ["<seed>", "<id>"], "created": "<iso>", "completed": null},
 "done": [Constellation], "stars": [{"id": "<id>", "visited": "<iso>"}]}
```

- `back` holds up to three ids, the same all day: the day's first request picks and keeps them. They are old finds
  (saved, and last saved again, over three months ago; not opened in six months; visible in the grid): up to two
  dark stars of the active constellation, least recently offered first, then the finds nearest in meaning to the
  saves of the last two weeks (the 20 newest saves when none are that recent). A find offered and left alone waits a
  month before it is offered again, unless its constellation still needs it. Without embeddings the picks are old
  finds in an order that changes each day. A pick that goes to the trash leaves the list.
- A constellation is one recent save (the `seed`, lit from the start) and the five old finds nearest to it, `cards`
  in that order. One is active at a time; when none is, the request forms one around the save from the last two weeks
  whose five nearest old finds are nearest of all. OpenAI Decisions picks its `figure` from a fixed set not used yet
  (`FIGURES` in `sky.rs`: ship, cup, book, mountain, lantern, camera, harp, tree, house, compass, anvil, bird, wave,
  coin), and `name` comes with the figure ("The Voyager"). Without an OpenAI key the figure is picked by the seed's
  id. The app draws each figure.
- `lit` is the seed and every find opened since the constellation formed. It is finished (`completed`, then in
  `done`, oldest first) once every find still out of the trash is lit; the next request forms the next one.
- `stars` is every find opened from Back today or the sky, with its last visit, for the loose stars.
- The phone records an opened find with the push's `visit` change (below). Card bodies come from the phone's copy.

### Uploads from the app

The app runs the web flow (`docs/write-api.md` 4.2) with its app token as the normal `Authorization: Bearer` header on
every request:

1. `POST /uploads/authorize` gives `{"url": "/uploads/<id>", "token": "<upload token>"}`.
2. `PUT <url>` with the chunk headers of the web flow. Send the upload token in `x-upload-token`; `Authorization`
   stays the app token. Without a correct `x-upload-token` the answer is `401 Invalid upload authorization`, so the app
   token alone cannot write an upload.
3. `POST /objects {"type": "Media", "uploadId": "<id from the last chunk>"}` creates the card.

### Card

Fields are omitted when the stored card has no value; the app treats every field but `id`, `type`, `flags`,
`created`, `modified` and `bumped` as optional.

```json
{
  "id": "00000002UnE0O4HfdtehA4",
  "type": "XPost",
  "flags": 0,
  "created": "2026-10-15T19:19:50Z",
  "modified": "2026-10-15T19:20:19.220548Z",
  "bumped": "2026-10-15T19:19:50Z",
  "deleted": "2026-10-16T08:00:00Z",
  "title": "Metal Effect for SwiftUI",
  "description": "Metal effect is now available for SwiftUI…",
  "url": "https://x.com/Jakubantalik/status/2110761365681873237",
  "domain": "x.com",
  "siteName": "X",
  "author": "…",
  "note": "…",
  "tags": [{"id": "00000002UnE0y6CGaAn5FS", "name": "SwiftUI", "flags": 2}],
  "spaces": ["00000002UnEglwYYzqZA8E"],
  "spaceRanks": {"00000002UnEglwYYzqZA8E": 4},
  "pinPosition": 3,
  "cover": {"src": "/media/…;600x414.mp4", "width": 600, "height": 414, "type": "video", "dominantColor": "#010101", "thumbHash": "3wcKNJqAh4eAh4d4iHeAcAj4eA=="},
  "blob": {"path": "…", "type": "image/jpeg", "width": 820, "height": 1024, "dominantColor": "#fbfbfb"},
  "prose": {"type": "doc", "content": []},
  "post": {"bodyHtml": "<p>…</p>", "author": "by …", "hasVideo": true, "gallery": [{"src": "…", "width": 1, "height": 1}]},
  "app": {"distributor": "app-store", "srcset": "…", "width": 250, "height": 250},
  "product": {"price": "$99", "brand": "…"},
  "reddit": {"title": "…", "subreddit": "…", "comments": [{"author": "…", "body": "…"}]},
  "videoLength": 634,
  "article": {"title": "…", "preview": "…", "url": "https://x.com/i/article/…"},
  "state": "ready"
}
```

- `type`: one of `XPost`, `ThreadsPost`, `SubstackNote`, `BlueskyPost`, `MastodonPost`, `WebPage`, `Article`,
  `Repository`, `Image`, `Screenshot`, `Video`, `YouTubeVideo`, `InstagramReel`, `Note`, `Content`,
  `SoftwareApplication`, `Product`, `Document`, `RedditPost`, `Placeholder`. The app shows an unknown type as a plain
  titled card instead of failing.
- `url`: `extras.linkUrl`, else `data.source.url`. `title`: `extras.title` (full), else `data.title`.
- `cover`: `extras.cover`; for an uploaded image without one, the rendition `imageCover(blob)` picks for the web.
- `thumbHash` on `cover`, or on `blob` for a card without a cover: the [ThumbHash](https://evanw.github.io/thumbhash/)
  of the gallery picture in standard base64, a blurred copy in about 25 bytes that the phone draws until the picture
  loads (`rust/crates/core/src/thumb_hash.rs`). The server reads it from a WebP rendition of the picture right after a
  card's capture; pictures saved before fill in newest first, all at once on a local server's start and 50 after each
  `/app/v1/changes` request on Cloudflare. An empty string means the picture could not be read, and a missing one that
  it has not been read yet.
- `prose`: the stored document tree (Note, Content). `post.bodyHtml` is the source post's sanitized HTML, the one
  place HTML reaches the app.
- `state`: `ready`, `saving` (capture pending), `screenshot` (screenshot pending) or `failed` (capture failed).
- What a link's own site adds after the capture (`rust/crates/core/src/capture_more.rs`), each only where the site
  answered: `reddit.comments`, up to three top comments from Reddit's public JSON with the post's own words in
  `post`; `videoLength`, a YouTube video's length in seconds from its watch page; `article`, the X article an X post
  carries, from X's embed data, whose cover becomes the card's `cover`. A saved Bluesky or Mastodon post, read from the
  network's public API, becomes a `BlueskyPost` or `MastodonPost` with its words, author and pictures.
- `product.brand`: `data.brand` when the capture read one, else what `extras.product` holds.
- `spaceRanks`: the card's place in each of its `spaces`. A space lists its cards from the smallest, the order of
  `/spaces/<id>/cards`, so the phone can show a space from its copy.

### Space

`{"id": "...", "name": "Always Relevant", "color": "#ffdcd0", "position": -1790003960, "smart": false, "count": 41}`.
`smart` is true for a space defined by a query.

## Live updates and writes

- `/hub` WebSocket with `Authorization: Bearer`: the same SignalR JSON hub and events the web gets
  (`object:created`, `object:updated`, …). The app (`data/HubListener.kt`) listens while it is on screen and, for any
  invocation, reads the changes feed (`GET /app/v1/changes`) instead of each card, so one read covers a burst of
  events. It connects again after a drop, waiting up to a minute, and stops trying after a 401, 403 or 404.
- Writes: the endpoints in `docs/write-api.md`, with `Authorization: Bearer` instead of the session cookie.

### Pushing the phone's changes

`POST /app/v1/push` takes the changes waiting on the phone in one request, as in the Replicache and Zero push protocol
(`app_push.rs`). The phone uses it and falls back to the per-change endpoints above when the answer has no
`lastChangeId`: an older server answers a write it does not know with `{}` and applies nothing.

```json
{"clientId": "<the copy's id>", "changes": [{"id": 12, "name": "pin", "args": {"cardId": "<id>", "pinned": true}}]}
```

- `clientId` names the phone's copy (up to 64 characters); a new copy, as after sign-out, makes a new one. Change ids
  are positive whole numbers that grow with each change; the server keeps the last one it handled for each client.
- Each change runs in its own transaction with that record, so a change sent again (an answer lost on the way, a
  retry) is skipped rather than applied twice.
- Names and `args`: `pin` (`cardId`, `pinned`), `trash` (`cardId`), `note` (`cardId`, `text`, one paragraph per
  line), `add-tag` (`cardId`, `name`), `remove-tag` (`tagId`), `add-to-space` and `remove-from-space` (`cardId`,
  `spaceId`), `create-space` (`spaceId`, `name`, `color`), `delete-space` (`spaceId`), `create-card` (`cardId`,
  `content`, `source`, `note`) and `visit` (`cardId`: a find opened from Back today or the sky, which lights its star
  and can finish the constellation). Other fields are ignored.
- `create-space` and `create-card` keep the id the phone made, so the phone can show the space or card and change it
  before the server answers. The id must be 32 lowercase hex digits (a UUID without its dashes): letters and digits
  like the library's own ids, and its letters stop at `f`, so it never holds the kind code those ids carry and count
  from. An id already in use is turned down. The phone makes spaces and cards this way only after the server has
  answered a push; before that it uses `POST /spaces` and `POST /objects`.
- `create-card` saves a link or text as `POST /objects` does, and schedules its enrichment the same way. A link
  already saved is bumped instead and keeps its id; the phone's id becomes another name for it (`app_aliases`), so
  the phone's later changes to that id reach the saved card. `note` becomes the card's note unless a link saved before
  already has one.
- Answer: `{"lastChangeId": 12, "refused": [ids], "tagIds": {"<change id>": "<tag id>"}, "cardIds": {"<change id>":
  "<card id>"}}`. A change the library turns down (a card that is gone, a blank tag, a name the server does not know)
  still counts as handled and is listed in `refused`; the phone drops it and reads the card again. `tagIds` gives the
  id of each tag the push added and `cardIds` the id the library keeps for each card it saved; when that is not the
  phone's id, the phone drops its own card and the changes feed brings the saved one.
- A malformed body, or more than 100 changes, is a 400 and applies nothing. A database failure stops the push at that
  change; the ones before it stay applied and are skipped when the phone sends again.

## App updates

`/download` takes the app token as well as the session cookie (the Worker's sign-in gate, and
`mymind download-server` locally).

- `GET /download/android.json`: the newest Android package, `{versionName, versionCode, url, bytes, sha256, built,
  notes, history}`. `url` is the package's path on the same server, `notes` are that version's changelog lines, and
  `history` holds `versionName`, `versionCode`, `date` and `notes` of every version, newest first. 404 when no
  Android package is published.
- `GET /download/<name>.apk`: the package, sent as `application/vnd.android.package-archive`.

The manifest entry behind both is described in `docs/plans/2026-10-17-1935-feat-android-updates-plan.md`.

## Feedback

Feedback from the app becomes a pull request that an agent writes on the sender's own Claude Code or Codex plan and
the runner merges on green CI (`docs/feedback.md`). Every refusal is 403 (never 401) with `{"error": "<sentence>"}`.

- `POST /app/v1/feedback` with `{text, engine, model, effort, client_id}`: `engine` is `claude` (or `claude-code`) or
  `codex`, `model` an id from `GET /app/v1/ai/models`, `effort` one of `none`, `minimal`, `low`, `medium`, `high`,
  `xhigh`, `max`, and `client_id` 8 to 64 letters, digits, `-` or `_`, made once per send so a retry returns the
  same job. Answers `202 {id, state}`. 400 for a malformed body or text outside 1 to 4000 characters; 403 "Only Tavi
  can send feedback for now." (while `FEEDBACK_OPEN` is `owner`), "Sign in to Claude Code first." (or Codex), "That
  model isn't available any more.", "You already have feedback in progress.", "You've sent 5 today. Try again
  tomorrow." (UTC day), "Feedback is paused for now."
- `GET /app/v1/feedback/{id}`: `{state, title, detail, version}` for the caller's own job, 404 otherwise. `title` is
  the notification's line and `detail` its second line; `version` is set once the change shipped.
- `GET /app/v1/feedback` (owner only): `{jobs: [...]}`, the last 100 jobs with engine, model, effort, PR, rounds,
  state, reason and the last failed check's link.
- `POST /app/v1/feedback/pause` (owner only) with `{paused: true|false}`: the kill switch. Answers `{paused}`.

While a feedback round has a person's sign-in, their AI search and sign-in status answer 403 "Your AI is busy with
your feedback."
