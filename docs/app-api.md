# App API: what the native Android app talks to

The Kotlin app (`android/`, roadmap phase 9) uses the same server and library as the web front-end. It signs in for
a bearer token, reads cards as typed JSON from `/app/v1/*`, and writes through the existing write API
(`docs/write-api.md`) with the same token. The web keeps its HTML; this JSON comes from the same stored rows
(`cards.data` and `cards.extras`, documented on `renderCard` in `lib/render-card.mjs`).

Status (2026-10-16): contract written; server read and sign-in endpoints implemented and proved by `tools/check-app-api.mjs`; app being built on branch `android-app`.

Implementation notes: `note` is the note's plain text. Read errors are `400` with `bad-limit`, `bad-cursor`, `missing-query` or `bad-request`; non-GET reads get `405`; unknown cards and spaces get `404 {"error":"not-found"}`. A space's cards omit trashed and grid-hidden cards. Search cursors are offset-based. Sign-out needs a valid bearer token (`204`, otherwise `401`). `GET /app/v1/me` exists only with sign-in on. `APP_NONCE_MS` overrides the nonce lifetime for tests.

## Sign-in

Only with `AUTH=google`. The same people the web admits: today `OWNER_EMAIL` by Google, and password users from
`tools/auth-user.mjs`. One function decides who is allowed for both the web and the app, so widening it to verified
`@octaviantocan.com` addresses (with per-person libraries, `docs/accounts-plan.md`) happens in one place.

| Request | Body | Answer |
|---|---|---|
| `POST /app/v1/auth/nonce` | none | `200 {"nonce": "<base64url>", "expiresAt": "<ISO>"}`. Single use, valid 10 minutes. |
| `POST /app/v1/auth/google` | `{"idToken": "<JWT>", "nonce": "<from /nonce>", "device": "<label>"}` | `200` token answer; `401 {"error": "not-allowed"}` for a verified Google account without access; `400 {"error": "invalid-token"}` for anything else (bad signature, audience, issuer, expiry, unknown or reused nonce, unverified email). |
| `POST /app/v1/auth/password` | `{"username": "...", "password": "...", "device": "<label>"}` | `200` token answer; `401 {"error": "invalid"}`; `429 {"error": "locked"}` (the web form's limits, shared with it). |
| `POST /app/v1/auth/signout` | none, `Authorization: Bearer` | `204`; the token stops working. |
| `GET /app/v1/me` | none | `{"email": "...", "kind": "google" \| "password"}`. |

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
| `GET /app/v1/search?q=<query>&limit&cursor` | Same list shape, mymind's query syntax (`docs/search.md`). |

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
  "pinPosition": 3,
  "cover": {"src": "/media/…;600x414.mp4", "width": 600, "height": 414, "type": "video", "dominantColor": "#010101"},
  "blob": {"path": "…", "type": "image/jpeg", "width": 820, "height": 1024, "dominantColor": "#fbfbfb"},
  "prose": {"type": "doc", "content": []},
  "post": {"bodyHtml": "<p>…</p>", "author": "by …", "hasVideo": true, "gallery": [{"src": "…", "width": 1, "height": 1}]},
  "app": {"distributor": "app-store", "srcset": "…", "width": 250, "height": 250},
  "product": {"price": "$99", "brand": "…"},
  "reddit": {"title": "…", "subreddit": "…"},
  "state": "ready"
}
```

- `type`: one of `XPost`, `ThreadsPost`, `SubstackNote`, `WebPage`, `Article`, `Repository`, `Image`, `Screenshot`,
  `Video`, `YouTubeVideo`, `InstagramReel`, `Note`, `Content`, `SoftwareApplication`, `Product`, `Document`,
  `RedditPost`, `Placeholder`. The app shows an unknown type as a plain titled card instead of failing.
- `url`: `extras.linkUrl`, else `data.source.url`. `title`: `extras.title` (full), else `data.title`.
- `cover`: `extras.cover`; for an uploaded image without one, the rendition `imageCover(blob)` picks for the web.
- `prose`: the stored document tree (Note, Content). `post.bodyHtml` is the source post's sanitized HTML, the one
  place HTML reaches the app.
- `state`: `ready`, `saving` (capture pending), `screenshot` (screenshot pending) or `failed` (capture failed).

### Space

`{"id": "...", "name": "Always Relevant", "color": "#ffdcd0", "position": -1790003960, "smart": false, "count": 41}`.
`smart` is true for a space defined by a query.

## Live updates and writes

- `/hub` WebSocket with `Authorization: Bearer`: the same events the web gets (`object:created`, `object:updated`,
  …). The app re-reads `GET /app/v1/cards/<id>` for each changed id.
- Writes: the endpoints in `docs/write-api.md`, with `Authorization: Bearer` instead of the session cookie.
