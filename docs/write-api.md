# mymind write side and read side: API spec for the SQLite backend

Scope: every client-to-server write the mymind web client (UI 0.109.0, `mirror/static/134/0.109.0/scripts`) sends,
the responses it expects, what the UI does next, and the read-side GET shapes the SQLite backend must generate.

Evidence legend: **[R]** a response was recorded in `replay/` or `capture/`; **[C]** inferred from client code only (the
request side is exact, the response side is what the client reads); **[?]** unknown, repeated under Open questions.

The original write recording was `PATCH /users/<uid>` returning `{ "status": 200 }`. Other write contracts below are
**[C]**; the read-side GETs in section 6 are mostly **[R]**. Today `server.mjs` answers every non-GET with `200 {}` and
logs `[write ignored]`. Examples are redacted: `<uid>` is the 22-char account id, `<id>` a card id.

An additional real URL save on 2026-10-15 records the hub and rendered `/changes` lifecycle in
`evidence/live-saves/2026-10-15-apk-link.jsonl`. Its APK has no resource text and its screenshot fails:

1. `object:created {id}`: Placeholder, flags 0, `placeholder processing`, the head-spin saving template, no title paragraph.
2. `object:updated {id,type:"WebPage",flags:2097152,bumped:"0001-01-01T00:00:00"}`: card flags 2097153,
   `web-page processing`, domain bubble, "Taking screenshot…" and the normal URL title paragraph.
3. `ai:failed {objectId}`, then `exception {id,type:"InvalidObjectException",message:"no resource text"}`.
4. `screenshot:failed {objectId}`: card flags 6291457, with exactly the same screenshot spinner markup.

Bit 2097152 identifies the fetched WebPage/pending-screenshot state in this trace; bit 4194304 is added on screenshot
failure. Their broader meanings are unproven. Bit 1 remains set even after both failures. Exact wall-clock delays,
details/mobile markup, successful screenshots/AI, early failures and failures with usable text were not recorded;
the implementation preserves the existing behavior for those outcomes. The local fixture checks read the recording
as their expectation and never copy private card data into source fixtures.

## 1. Conventions

### 1.1 Request headers

| Header | Value | Notes |
|---|---|---|
| `x-authenticity-token` | `v4.<a>.<b>.<c>` | Sent on every `api.js` request. Source: `meta#authenticityToken` in the shell HTML. |
| `Content-Type` | `application/json` (default), `application/prose+json` | prose body of `PUT /objects/:id/content`; `GET` of it uses `prose+json` as `Accept` |
| `Content-Type` | `<file mime>` | Raw attachment upload, and `text/markdown` for `POST /convert`. |
| `Accept` | `application/json` or `application/msgpack` or `application/prose+json` | msgpack for grid streams and `/changes`. |

`composer.js` calls `fetch` directly for note save (`x-authenticity-token`, `Content-Type`, `Accept: application/json`);
its attachment upload sends no token header (only `Content-Type`, `Accept`, `Content-Length`). A local server should not
make the token mandatory on attachments.

Token rotation: a response may carry `x-authenticity-token`; `api.js` stores it for the next request. The replica does
not validate tokens today, so rotation is optional.

### 1.2 Responses

- Success: 200/201 with JSON, or an empty body. `api.post` calls `response.json()` (an empty body throws, so every
  POST the client parses must return JSON). `api.delete` does not parse the body.
- Failure: any non-2xx. `api.ensureSuccess` throws; the client reads
  `Content-Type: application/problem+json` with `{ "type": "...", "status": 4xx, "detail": "...", "errors": {...}? }`.
- Header `x-object-quota: {"count":2271,"limit":50000}` is read after create, delete and `/changes`. When
  `count >= limit` the client refuses to create ("You are out of cards"). Send it on those responses; also exposed via
  `access-control-expose-headers: x-object-quota, x-checkpoint` (cards) or `x-object-quota, x-count` (changes).

### 1.3 Client timing

Title, note and prose saves debounce 1 s, so expect bursts of `PUT`/`PATCH` per card. Hub events are applied after
50 ms (3000 ms while a save is in flight); `mymind.synchronize()` (a `/changes` poll) also runs on window focus.

## 2. Identifiers, timestamps, flags

### 2.1 Id format [R from `capture/cards.json`]

22 chars base62 (`0-9A-Za-z`). Layout `<11 constant><2 kind><9 counter-ish>`. Kind prefix after the constant
`<P>`: card `0O`, tag `0y`, space `gl`, smart-space query `iV`, card note `9Z` (one sample,
`<P>9Z<9 chars>`).

- `<P>` is the 11-char prefix shared by every id of the account (take it from `capture/cards.json`; redacted here).
  Treat it as a shard/account prefix.
- Ids are roughly time-ordered, not strictly. Tags created together get consecutive ids (last char counts up in base62).
  A generator with the same prefix, a kind prefix and 9 base62 chars from a monotonic counter is fine; the client never
  parses ids.
- Account uid (`config.account.uid`, also in `/users/<uid>`): 22 chars, same alphabet.
- Media paths embed ids: `/media/<account uid>/<11-char card tail>/<22-char blob id>;<W>x<H>[-c].webp`.
- The card id is not inside the card `json`; it is only in the html `data-id`.

### 2.2 Timestamps

| Where | Format |
|---|---|
| card json `created`, `bumped`, `deleted` | ISO, second precision: `2026-10-11T12:34:16Z` |
| card json `modified`, note `modified` | ISO, 6 fractional digits: `2026-10-11T12:34:20.065912Z` |
| detail html `data-modified`, `time[datetime]` | 7 fractional digits |
| space `created` | ISO, microsecond precision |
| `/changes` item `timestamp` | msgpack uint64 (0xcf), nanoseconds since epoch |
| `/changes` object dates | msgpack timestamp ext (type -1, `d7 ff` = timestamp64) |
| `x-checkpoint` | decimal string, ns since epoch, equals the newest card `modified` in ns |

Store `modified` as integer nanoseconds; derive both ISO forms and the checkpoint from it.

### 2.3 Card flags (bitmask, `json.flags`)

| Bit | Value | Meaning |
|---|---|---|
| 0 | 1 | analyzing (AI not finished) |
| 1 | 2 | readable (reader view; set on 170 of 173 Articles) |
| 3 | 8 | private (hidden from grid) |
| 9 | 512 | pinned, "Top of Mind" |
| 14 | 16384 | starter card |
| 16 / 17 / 18 | 65536 / 131072 / 262144 | completed / draft (`isCurrentDraft`) / stub |
| 20 to 23 | 1<<20 .. 1<<23 | [?] never read by the client. Capture: bit 20 on Screenshot 162, Image 55; bit 21 on WebPage 339, Article 173; bit 23 on XPost 375, InstagramReel 23 |

The grid hides a card when `isPrivate`, `isCurrentDraft` or `isStub`.

### 2.4 Tag flags

`{id, name, flags}`; `flags` is omitted in card json when 0.

| flags | Meaning | Count in capture |
|---|---|---|
| 2 | AI tag | 9709 |
| 65538 / 8194 / 4098 / 4102 | AI tag plus bit 16 / 13 / 12 / 12+2 | 3372 / 3130 / 18 / 25 |
| 8 | manual tag (what `POST /objects/:id/tags` creates) | 70 |
| missing | untyped; also the `highlight` tag | 1272 |

Bits above 2 and 8 are unexplained [?]. Use `8` for every user-added tag.

## 3. Data model the backend must hold

Suggested `node:sqlite` shape (one writer, WAL):

```
objects(id PK, type, flags, title, description, prose JSON, source_name, source_url, site_name, domain, ocr,
        blob JSON, format, author, object_names JSON, brand, note_id, note_prose JSON, note_modified_ns,
        extra JSON, created_ns, bumped_ns, modified_ns, deleted_ns NULL)
annotations(id PK, object_id, name, flags)                          -- a card's tag; DELETE /annotations/:id
spaces(id PK, name, color, position, created_us, query_id NULL, query_filters JSON NULL, query_checked NULL)
space_objects(space_id, object_id, score REAL, query_id NULL, excluded INT, PRIMARY KEY(space_id, object_id))
settings(uid PK, colorScheme, locale, timeZoneId, soundEnabled, onboardFlags INT)
changes(seq INTEGER PK, ts_ns, type, object_type, object_id)       -- feeds /changes?since=
uploads(id PK, token, name, size, received, blob_path, done)
revisions(id PK, object_id, created, is_current, body JSON)
shares(id PK, object_id, created, expires)
```

Card json keys (all optional except the first five): `type, flags, modified, bumped, created` always; then
`title, description, tags[], source{name,url}, siteName, domain, ocr, blob{...}, format, author, objectNames, prose,
brand, note{id,prose,modified}, deleted`.

Card `type` values and counts in the capture: XPost 932, WebPage 340, Repository 281, Screenshot 215, Article 173,
Image 141, Video 77, InstagramReel 36, Note 31, YouTubeVideo 24, Product 4, ThreadsPost 4, SoftwareApplication 3,
Document 3, Placeholder 2, Content 2, SubstackNote 1, RedditPost 1.

Note card json (redacted):

```json
{ "type": "Note", "flags": 0,
  "tags": [{ "id": "<P>0y6…", "name": "showreel", "flags": 2 }],
  "prose": { "type": "doc", "content": [{ "type": "paragraph", "content": [
      { "type": "text", "text": "Hello" }, { "type": "hardBreak" } ] }] },
  "modified": "2026-01-01T12:00:00.065912Z", "bumped": "2026-01-01T12:00:00Z", "created": "2026-01-01T12:00:00Z" }
```

`prose` is a TipTap/ProseMirror document. A card note is `note: { id, prose, modified }` inside the card json.

## 4. Writes

Every row below: method and path, body, response, UI follow-up. "Re-fetch" means the client issues the named GET.

### 4.1 Save a card

#### `POST /objects` [C]

One endpoint for every capture path. Headers: token; `Content-Type: application/json` (or `multipart/form-data` for
file paste). Response `{ "id": "<id>" }`, status **201** for a new card, **200** when the server matched an existing
card and bumped it. Also send `x-object-quota`.

| Intent | Body |
|---|---|
| URL or any pasted text | `{ "content": "https://example.com/post", "source": "clipboard", "spaceId": "<id>?" }` |
| Note from composer | `{ "type": "Note", "prose": { "type": "doc", "content": [...] } }` |
| Note with title | `{ "type": "Note", "title": "...", "prose": {...}, "spaceId": "...", "source": "clipboard" }` |
| Wiki stub | `{ "title": "Idea", "type": "Note", "isStub": true }` |
| Image or PDF paste | multipart: part `metadata` = JSON blob `{ "source": "clipboard", "spaceId": "..." }`, part `blob` = the file |
| Uploaded file | `{ "type": "Media", "uploadId": "<blob id>", "spaceId": "<id>?" }` |

Server decisions the client does not make [?]: link versus text for `content`, link-metadata scraping (title,
description, siteName, domain, source, cover image, `type`), markdown-looking text becoming a `Note`.

Minimal local behavior that satisfies the client:

1. `content` is a single http(s) URL: insert an `objects` row with `type: "Placeholder"`, `flags: 0` (saving),
   `source.url`, `domain`, `title` = host; append a `changes` row; enrich asynchronously (see 5.2).
2. `content` is other text: `type: "Note"`, `prose` = one paragraph per line.
3. The same URL already present (not deleted): bump (`bumped = now`) and return **200** `{id}`.
4. Blob or `uploadId`: `type: "Image"` (image mime), `"Screenshot"`, `"Video"`, `"Document"` (pdf), set `blob`.

UI follow-up on 201: `mymind.prependCard(id)` does `GET /cards/<id>` (Accept msgpack), adds the card to the top of
the grid, fires `object:created`. On 200: "Bumped!" notice, `mymind.synchronize()` and re-sort by `bumped`. If a
`spaceId` was sent, the server must add the card to that space (also fire `space:updated`).

Draft flow (kept client-side in `localStorage.draft`): `POST /objects {type:"Note", isDraft:true, prose}` returns `{id}`;
`PATCH /objects/<id>/prose` (body is the doc itself) saves; `POST /objects/<id>/commit {prose}` (200/201) clears the
draft bit and the card appears in the grid. `composer.save()`: draft with id calls `commit`; existing card id calls
`PUT /objects/<id>/prose` (doc body, no wrapper); otherwise `POST /objects {type:"Note", prose}`. After a 200 the UI
calls `mymind.synchronize()` and bumps; after a 201 it calls `prependCard`.

### 4.2 Uploads (image, PDF, audio, video) [C]

Three steps, then the card is created from the upload id.

1. `POST /uploads/authorize[?count=N]` (N up to 100 per call; the client batches), no body. Returns one
   authorization object, or with `count` an **array** of them. The client reads `authorization.url` and passes the
   whole object to `Carbon.Upload` as `authorization`. Fields beyond `url` and a bearer `token` are [?].
2. `PUT <authorization.url>` (XHR, may be a different origin), body is the file bytes, chunked at 32 MiB:

   | Header | Value |
   |---|---|
   | `Content-Type` | file mime |
   | `Authorization` | `Bearer <authorization.token>` |
   | `x-upload-token` | Optional. `<authorization.token>` without `Bearer`. The server reads it first and ignores `Authorization` for the upload check, so a client whose `Authorization` carries an app token (`docs/app-api.md`) can still upload. A wrong value is refused even if `Authorization` is right. |
   | `x-file-name` | `encodeURI(file name)` |
   | `Content-Range` | `bytes <start>-<end>/<total>` |
   | `x-upload-id` | id from the first chunk's response, on later chunks |

   Final response JSON `{ "id": "<id>", "blob": { "id": "<blob id>" } }`. The client uses `result.blob.id` (saveFile) or
   `result.id` as `uploadId`.
3. `POST /objects { "type": "Media", "uploadId": "<blob id>", "spaceId": "<id>?" }` returns `{id}`; UI then
   `prependCard`. With an active space the client additionally sends `PUT /spaces/<spaceId>/objects/<cardId>` with `{}`.

Local implementation: make `authorize` return `{ "url": "/uploads/<uploadId>", "token": "<random>" }` (same origin),
accept the chunked `PUT` at `/uploads/<uploadId>` and append to a file, return `{id, blob:{id}}` on the last chunk.
Then `POST /objects` with `type: "Media"` links `blob` to a stored file and serves it at `/media/...` (see 6.8).

Client limits (from `config`): `maxUploadSize` 500000000, `maxQueueSize` 100; without the `video` feature, videos are
limited to 5 MB. Accepted: audio, image, video, `ai`, `pdf`. Unsupported format shows `#unsupported-file-type-notice`
(client side only).

Related binary writes:

| Call | Body | Response | Use |
|---|---|---|---|
| `POST /objects/<id>/attachments` | raw bytes, `Content-Type: <mime>` | `{ "path", "width", "height" }` | inline image in prose editor |
| `POST /objects/<id>/attachments` | JSON `{ "remoteUrl": "..." }` | same | remote image in prose |
| `POST /objects/<id>/attachments` | JSON `{ "uploadId": "..." }` | same | uploaded image in prose |
| `PUT /objects/<id>/<subresource>` | `{ "uploadId": "..." }` | any 2xx | replace screenshot/image (`<subresource>` name [?]) |

The response `{path,width,height}` becomes the editor's image node `{src: path, width, height}`.

### 4.3 Edit a card

| Edit | Request | Response | UI follow-up |
|---|---|---|---|
| Title | `PATCH /objects/<id>` `{ "title": "New title" }` | any 2xx (JSON ok) | none; field keeps its value. Empty string means untitled; input has `maxlength=100`; blur or 1 s debounce. |
| Byline | `PATCH /objects/<id>/entity` `{ "author": { "name": "..." } }` | any 2xx | none |
| Card note | `PUT /objects/<id>/note` body = prose doc JSON | `{ "id": "<note id>" }` | UI stores `object.note = {id, prose}`; no re-fetch |
| Note-card body | `PUT /objects/<id>/prose` body = prose doc JSON | 200 | `mymind.synchronize()` |
| Page content (reader/editor) | `PUT /objects/<id>/content`, `Content-Type: application/prose+json`, 1 s debounce | any 2xx | none |
| Read content | `GET /objects/<id>/content`, `Accept: application/prose+json` | prose doc | |
| Full-model patch (native only) | `PATCH /objects/<id>` with the whole model | any 2xx | |
| Remove AI thumbnail | `POST /cards/<id>/remove-ai-thumbnail` `{}` | any 2xx | UI re-fetches `/cards/<id>/detail` |
| Generate AI thumbnail | `POST /cards/<id>/generate-ai-thumbnail` `{}` | any 2xx | same |
| Re-analyze (tl;dr) | `POST /cards/<id>/analyze` `{}` | any 2xx | wait for hub `object:analyzed` |

- The web UI has **no description editor**; description is server-written (scraped or AI). Whether `PATCH` accepts
  `description` is [?]; accept it harmlessly.
- Every edit must set `objects.modified_ns = now`, append a `changes` row (type 1), and emit hub `object:updated`.
  Editing does not change `bumped`.
- Revisions: `GET /objects/<id>/revisions` returns `[{ "id", "created", "isCurrent", "content": { "body": ... } }]`;
  `POST /objects/<id>/revisions/<revId>/restore` `{}` restores one.

### 4.4 Tags

| Action | Request | Response | UI follow-up |
|---|---|---|---|
| Add | `POST /objects/<id>/tags` `{ "name": "design" }` | the tag model `{ "id", "name", "flags": 8 }` | UI sets `model = response`; no re-fetch |
| List | `GET /objects/<id>/tags` | array of tag models | |
| Remove | `DELETE /annotations/<tagId>` (tag id, not name) | empty/any 2xx; body ignored | tag chip removed locally; on error notice "Couldn't remove tag" |
| Bulk add | `POST /batch/tag` `{ "objectIds": ["..."], "tag": "design" }` | [?] | grid re-sync |

- The client strips a leading `#` before sending.
- "Annotation" is the server term for a card-tag link. Tag ids are per card [R]: in the capture 17596 tag ids map to
  6747 names, each id has exactly one name, and a name such as `AI` has 323 different ids. So the id returned by
  `POST /objects/<id>/tags` identifies this card's tag, and `DELETE /annotations/<tagId>` removes only that one.
  Model it as `annotations(id PK, object_id, name, flags)`; name-level queries (`/tags/*`) use `GROUP BY name`.
- A name already on the card should return that card's existing tag (idempotent).
- Hub pushes to other tabs: `tag:created {id, name, objectId, flags}`, `tag:deleted {objectId, name}`.
- Side reads: `GET /tags/recent?take=10` returns `[{name}]` (add-tag autocomplete), `GET /tags/manual`,
  `GET /tags/suggest?q=`, `GET /tags/detail?name=` returns `{objects: [...]}`.

### 4.5 Highlights

A highlight is an ordinary annotation whose `name` is `highlight` (no `flags` key).

- Add: `POST /objects/<id>/tags {"name":"highlight"}`. The UI adds class `highlighted` to the detail body and the
  card root becomes `<mymind-card class="content highlighted" ...>` on next render (the capture has one such Content
  card; its json tag is `{ "id": "...", "name": "highlight" }`).
- Remove: `DELETE /annotations/<tagId>`; the UI removes the class.
- The server must render `highlighted` into the card html root class whenever the card has a `highlight` tag.
- The editor also has a TipTap `highlight` mark inside prose; that is document content and needs no endpoint.

### 4.6 Spaces

Space model returned everywhere (see 6.5): `{ id, name, color, position, objects: [{id, score, queryId?}], created, query? }`.

| Action | Request | Response | UI follow-up |
|---|---|---|---|
| Create manual | `POST /spaces` `{ "name": "Ideas", "color": "#a1b2c3", "position": <count+1> }` | full space model | navigates to `/spaces/<id>`, notice "Space created", fires `space:created` |
| Create smart | `POST /spaces` `{ "name", "color", "query": { "filters": ["type:image"], "matches": [{"id","score"}] } }` | full space model with `query` | same |
| Rename | `PATCH /spaces/<id>` `{ "name": "New" }` | any 2xx | notice; local model updated |
| Recolor | `PATCH /spaces/<id>` `{ "color": "#aabbcc" }` | any 2xx | notice |
| Reorder | `POST /spaces/reorder` `["<id>", "<id>", ...]` | any 2xx | `mymind.spaces.stale = true` then re-fetch `GET /spaces` |
| Delete | `DELETE /spaces/<id>` | any 2xx | removed locally, navigate to `/spaces`, notice "Deleted space" |
| Add card | `PUT /spaces/<spaceId>/objects/<cardId>` `{}` | any 2xx | local `space.objects` updated; card html unchanged |
| Remove card | `DELETE /spaces/<spaceId>/objects/<cardId>` | any 2xx | |
| Remove from smart space | `POST /queries/<queryId>/objects/<objectId>/exclude` `{}` | any 2xx | object has `queryId` in the space |
| Bulk add | `POST /batch/add-to-space` `{ "objectIds": [...], "spaceId": "..." }` | [?] | |

Details:

- The client rejects a duplicate space name case-insensitively before sending; the server need not.
- Manual space `position` is a small int; the server may ignore the client value and append. Smart or default spaces
  carry a large negative `position` equal to `-(created epoch seconds)`.
- A smart space `query` is `{ id, filters: ["type:image", "repo", "tag:x", "-tag:y"], checked: "<ISO>" }`. Filters use
  the search syntax of `/search/suggestions` (`type:`, `tag:`; `label:` is rewritten to `#`; leading `-` negates).
  Space objects from a query carry `{ id, score (float), queryId }`.
- Smart space sync, server mode: client calls `GET /queries/<id>/check`, expects
  `{ "elapsed": 7, "added": [], "removed": [] }`, and re-fetches `GET /spaces/<id>` if either list is non-empty.
  (Client-mode `PUT /queries/<id>/matches` is unused because `config.index.mode = "server"`.)
- Hub: `space:created`, `space:updated`, `space:deleted` (payload `{id}`) mark the space list stale.
- `/changes` carries spaces as items with `objectType 190`; the client then calls `spaces.invalidate`.
- Modal template: `GET /components/NewSpaceModal` (HTML partial, recorded).

### 4.7 Delete, restore, permanent delete

| Action | Request | Response | UI follow-up |
|---|---|---|---|
| Move to trash | `DELETE /objects/<id>` | any 2xx, body ignored; send `x-object-quota` | card removed from grid, notice "Card deleted and moved to trash.", then `trySynchronizeQuotas` |
| Bulk trash | `POST /batch/delete` `{ "objectIds": [...] }` | [?] | optimistic removal in select mode |
| Restore | `POST /objects/<id>/restore` (no body) | any 2xx | card removed from the trash grid (`mymind.trash.restore`); next normal sync brings it back |
| Permanent delete | **no client call found** | | |
| Empty trash | **no client call found** | | |

- Trash is a soft delete: set `deleted = now`, `modified = now`, append a `changes` row with `type 2`, emit hub
  `object:deleted`. Restore clears `deleted`, sets `modified = now`, appends a `type 1` change, hub `object:updated`.
- `GET /cards/trash` returns the deleted cards (see 6.3). The trash page shows onboarding flag 64 (trash intro).
- Permanent removal appears to be server-side expiry only; the trash UI offers restore and exit, nothing else. A
  reasonable local implementation: purge rows deleted more than 30 days ago on startup (retention value [?]).

### 4.8 Serendipity

| Action | Request | Response | UI follow-up |
|---|---|---|---|
| Open or restart | `GET /serendipity` | HTML partial (see 6.9) | UI mounts 18 cards |
| Keep | none (client only) | | card flies back, nothing sent |
| Forget | `DELETE /objects/<id>` | any 2xx | same as normal trash; card is removed from the deck |
| Finish deck | `POST /serendipity/complete` `{}` | any 2xx | unlocks outro |
| First-run intro | `POST /onboard/flags/256/set` | JSON | flag `1<<8` |

Keep has no server effect, so no table is needed. The deck is random: pick 18 non-deleted, non-private cards.

### 4.9 User settings

All use `PATCH /users/<uid>` with a JSON body; the only recorded write [R]:

```
PATCH /users/<uid>
x-authenticity-token: <token>
Content-Type: application/json

{ "colorScheme": "dark" }

200 application/json
{ "status": 200 }
```

| Setting | Body | Where |
|---|---|---|
| Color scheme | `{ "colorScheme": "light" \| "dark" \| "system" }` | `colorScheme.js` |
| Locale and timezone | `{ "locale": "en-US", "timeZoneId": "Europe/London" }` | `app.js` `onIdle`, sent only when different from `Intl` |
| Serendipity sound | `{ "soundEnabled": true }` | serendipity toggle |

- Persist into `settings`; the next shell HTML must echo `colorScheme`, `locale`, `timeZoneId` inside
  `<script id="config" type="application/json">`. Nothing else re-fetches.
- **Density is client-only**: `localStorage.cardWidth` (grid column width) and `mobileSingleColumnLayout`. No request.
- Onboarding flags: `POST /onboard/flags/<value>/set` and `/unset`, no body, JSON response (`{}` is fine). Values
  seen: 64 trash intro, 512 reader intro, 4096 markdown shortcuts, 256 serendipity. Stored as a bitmask echoed in
  `config.onboardFlags`.
- `GET /account.json`, `GET /account/blocks/subscription` (HTML) are read-only account views.

### 4.10 Other writes

| Call | Body | Response | Use |
|---|---|---|---|
| `POST /objects/<id>/pin` | `{}` or `{ "position": 1 }` | any 2xx | Top of Mind pin (flag 512) |
| `POST /objects/<id>/unpin` | `{}` | any 2xx | unpin |
| `POST /objects/reorder-pins` | `["<id>", ...]` | any 2xx | reorder pinned |
| `POST /objects/<id>/complete`, `/uncomplete` | `{}` | any 2xx | completed bit 65536 |
| `POST /cards/<id>/shares` | `{}` | `{ "id", "url" }` | share link, 24 h expiry; link is `/shares/<id>` |
| `DELETE /cards/<id>/shares/<shareId>` | | any 2xx | revoke share |
| `POST /convert` | markdown text, `Content-Type: text/markdown`, `Accept: application/prose+json` | prose doc | paste markdown |
| `POST /promotions/<id>/dismiss` | `{}` | any 2xx | |
| `GET /achievements/<id>?dismiss=true` | | JSON | dismisses (a GET with a side effect) |
| `POST /image-embeddings/enable` | none | text | |
| `POST /jserrors` | error report JSON | any 2xx | accept and drop |

Read side companions: `GET /cards/<id>/markdown`, `/cards/<id>/download[?format=]`, `/cards/<id>/reader`
(recorded), `GET /assistant/ask?q=`, `/nlp?text=`, `/vibe/analyze?q=`, `/query/analyze?q=`,
`GET /starter-cards[?userAgent=iPhone]` (msgpack `{html,json}` stream), `GET /embeddings/related/<id>?take=100`,
`GET /embeddings/search?q=&modelId=&take=100`.

## 5. Keeping the UI in sync

### 5.1 Pull: `GET /changes?since=<ns>&render=true`

Initial `GET /cards` sets `mymind.checkpoint` from `x-checkpoint`. Each sync requests items newer than the checkpoint.

- Response `application/msgpack`; header `x-count: <n>`; body is empty when `x-count` is 0 [R]. Also send
  `x-object-quota` and `access-control-expose-headers: x-object-quota, x-count`.
- Body is a stream of msgpack maps, one per change, six keys [R]:

  | Key | Value |
  |---|---|
  | `type` | 1 created or updated [R]; 2 deleted [C] |
  | `timestamp` | uint64 (0xcf), ns; the client sets `checkpoint = BigInt(timestamp)` |
  | `objectType` | 9 card [R]; 190 space [C] |
  | `object` | map, below |
  | `cardHtml` | string, same html as `{html}` |
  | `cardData` | string, the card JSON as a string (same as `{json}`) |

  `object` keys: `id, title, summary?, flags?, mainEntity {@type, @id, title, author, cover{type,path,width,height,
  dominantColor}, published}, entityType, spaces: [{id}], tags: [{id,name,flags}], source {name,url}, bumped, created,
  modified` (dates as msgpack timestamp ext).
- Client rule: for each item, `type 2` removes the card from the grid; otherwise it replaces the card when the object
  is new, its `modified` differs from the rendered one, or the element has class `processing`. Then
  `space.onObjectAdded` for each id in `object.spaces`. Space changes (`objectType 190`) call `spaces.invalidate`.
- Recorded lifecycle of a new YouTube card: change 1 with `flags: 1`, 26 raw tags and the raw video description;
  about 6 s later change 2 with `summary` (contains `[[wikilinks]]`), 36 tags and the description replaced by the AI
  summary. A local backend can emit one change at create and a second after fake enrichment.
- Only `type 1` / `objectType 9` items were ever recorded; the payload of a `type 2` item (probably `object.id` only, empty
  `cardHtml`) and of space items is [?].

### 5.2 Push: SignalR hub `/hub`

WebSocket, JSON protocol, `skipNegotiation`. The replica currently answers the handshake (`{}` + 0x1e) and pings
`{"type":6}` + 0x1e every 15 s, never pushing. Messages are SignalR invocations
`{"type":1,"target":"<name>","arguments":[<payload>]}` + 0x1e. Server-to-client targets the client handles:

| Target | Payload | Client reaction |
|---|---|---|
| `object:created` / `object:updated` / `object:bumped` / `object:deleted` / `object:analyzed` | `{id}` | schedule `synchronize()`; `object:analyzed` also re-fetches `/objects/<id>` and `/cards/<id>/detail` |
| `ai:failed` | `{objectId}` [R, APK save] | clears analyzing state |
| `screenshot:completed` / `screenshot:failed` | `{objectId}` | re-fetch `/cards/<id>/detail` |
| `tag:created` | `{id, name, objectId, flags}` | update tag list of an open card |
| `tag:deleted` | `{objectId, name}` | same |
| `space:created` / `space:updated` / `space:deleted` | `{id}` | `spaces.stale = true` |
| `account:updated`, `iap:processed`, `session:created`, `browser-extension:installed` | varies | account refresh |
| `exception`, `info`, `ReceiveMessage(user, message)` | strings | notices |

The hub is optional: the client also runs `synchronize()` after its own saves and on focus, so a first implementation
may leave the hub silent.

## 6. Read side the backend must generate

All lists come from SQLite; the client never needs a replayed byte. Data to use as templates is in
`capture/cards.json` (2270 cards, `{html, json}`), `capture/details/<id>.html` (30 details) and `replay/`.

### 6.1 `GET /cards` (grid) [R]

`Content-Type: application/msgpack`. Body: concatenated msgpack values, one `{ "html": "<mymind-card ...>", "json":
"<card JSON as string>" }` map per visible card, ordered `bumped` DESC (the capture's order). Headers:
`x-object-quota: {"count":N,"limit":50000}`, `x-checkpoint: <max modified ns>`,
`access-control-expose-headers: x-object-quota, x-checkpoint`. Exclude `deleted`, private, draft and stub cards.

### 6.2 `POST /cards/grid`, `GET /cards/<id>`

- `POST /cards/grid` body: JSON array of ids. Response: msgpack stream of `{html,json}` in the requested order. Used to
  hydrate a list from search or space matches.
- `GET /cards/<id>` with `Accept: application/msgpack`: one `{html,json}` value. With `Accept: application/json`:
  the card json object only. With `?template=PinnedCard`: HTML with `x-partial: PinnedCard`.
- `GET /objects/<id>` returns JSON with `title`, `tags`, `entityType`; used after `object:analyzed`.

### 6.3 `GET /cards/trash` [R]

Same msgpack `{html,json}` stream, only deleted cards. The html root gets class `deleted`:
`<mymind-card class="x-post post deleted" data-id="<id>"><div class="hover" on-click="card:restore"></div>...`
(`card:open` becomes `card:restore`, and the link bubble is omitted). json is the normal card json plus
`"deleted": "<ISO seconds>"`, and `modified` equals the delete time.

### 6.4 Card html templates

`html` is server-rendered, and the SQLite backend must render it. Root `<mymind-card class="<type classes>"
data-id="<id>">`; add `deleted` / `highlighted` classes when applicable. Root class by `type` [R]:

| Type | Class |
|---|---|
| XPost, ThreadsPost, SubstackNote | `x-post post`, `threads-post post`, `substack-note post` |
| WebPage, Article, Repository, Image, Screenshot | `web-page`, `article`, `repository`, `image`, `screenshot` |
| Video, YouTubeVideo, InstagramReel | `video`, `youtube-video video`, `instagram-reel` |
| Note, Content | `note content`, `content` |
| SoftwareApplication, Product, Document, RedditPost | `software-application`, `product`, `document`, `reddit-post` |
| Placeholder | `placeholder unavailable` |

Inner structure (samples, redacted):

```html
<!-- Image -->
<mymind-card class="image" data-id="<id>">
  <div class="hover" on-click="card:open"></div>
  <mymind-image style="aspect-ratio:800/999;background-color:#fbfbfb"><img class="lazy" data-src="/media/..."/></mymind-image>
  <p class="title">Title</p>
</mymind-card>

<!-- Note -->
<mymind-card class="note content" data-id="<id>">
  <div class="hover" on-click="card:open"></div>
  <div class="inner body prose "><p>text<br /><br />more</p></div>
</mymind-card>
```

Repository adds a link bubble in `.hover` (`<a class="link-bubble" href="<url>" on-click="card:link" target="_blank">
<span>Repository</span></a>`) and `.inner.github` (icon, name, label). Article uses `.article-card-top` (logo plus `<h3>` title) and a `mymind-image`; XPost uses `.inner > .body.x` with
`<i class="platform">`, `<p>` paragraphs, and an optional `mymind-image` or `<video autoplay muted playsinline>`.
Rule for the build: group `capture/cards.json` by `type`, normalize the data-bearing parts (title, url, media path,
aspect-ratio, paragraphs) and turn each type's most common html into a template. For a type without a template, fall
back to the `image` or `web-page` form.

### 6.5 `GET /spaces`, `GET /spaces/<id>` [R]

`application/json; charset=utf-8`. `/spaces` is an array sorted by `position` ascending; `/spaces/<id>` is one item.

```json
{ "id": "<P>gl...", "name": "Design", "color": "#e8c4a0", "position": 4,
  "objects": [{ "id": "<P>0O...", "score": 1 }],
  "created": "2026-04-05T21:10:44.123456Z" }
```

Smart space adds `"query": { "id": "<P>iV...", "filters": ["type:image"], "checked": "2026-10-15T14:00:37Z" }`;
its objects carry `queryId` and a float `score`. Manual spaces use `score: 1`.

### 6.6 `GET /search`, suggestions, tags [R]

Search runs on the server because `config.index.mode = "server"`.

```
GET /search?q=design&wildcard=true
{ "elapsed": 1, "count": 840, "query": "design*", "matches": [{ "id": "<id>", "score": 12.3 }] }
```

- Params: `q`, `wildcard=true|false` (true appends `*` to `query`), `sortBy`, `semantic=true&semanticBoost=2`.
- `matches` is ordered by score DESC; the client then hydrates with `POST /cards/grid`. Support `type:<kind>`,
  `tag:<name>` and `-` negation in `q`; use FTS5 over title, description, ocr, tags, prose text, source url.
- `GET /search/suggestions` (no `q`): array of 17 `{ type: "type", text, queryText: "type:x-post", count, score,
  isAI }`. With `?q=design`: tag suggestions `{ type: "tag", text: "design", queryText: "tag:design", count, score,
  isAI }`.
- `GET /tags/recent?take=10` -> `[{ "name": "design" }]`.
- `GET /tags/manual` -> msgpack, one array `[{ "name", "count", "flags": 0, "modified" }]`.
- `GET /queries/<id>/check` -> `{ "elapsed": 7, "added": [], "removed": [] }`.

### 6.7 `GET /cards/<id>/detail` [R]

Partial HTML (`x-partial` header). Structure of every recorded detail:

- `<mymind-card-header>`: close button, title, options dropdown ("Visit original source").
- `<mymind-card-detail class="<type> default-surface screenshot-mode">` containing:
  - `<script class="card-data" type="application/json">{ "id", "type", "share": null }</script>`;
  - `.guts` with the screenshot (`browser-container`) or content;
  - `<mymind-card-sidebar data-object-id="<id>" data-modified="<7-digit ISO>">` with
    - `<mymind-editable data-action="/objects/<id>">` (title input, `maxlength=100`),
    - the `<time datetime>` line and the source link,
    - `<mymind-tldr data-watchable="true|false"><p>summary</p></mymind-tldr>`, only when the card has a summary
      (`true` for `youtube-video` and `instagram-reel`, which the client labels TLDW),
    - `<mymind-tag-cloud data-object-id="<id>">`: add-tag form plus `<mymind-tag-list tags='[...]' object-type
      object-type-name object-type-slug>`,
    - `<mymind-card-note>` with `<mymind-editor data-type="note" data-card-id="<id>" data-action="/objects/<id>/note">`,
    - bottom actions `card#<id>:addToSpace|share|delete`, a share link `/shares/<id>`, and `mymind-space-chooser`.

The action URLs in this html are the write endpoints of section 4; keep them identical when templating. Only 30 details
exist on disk, so the backend needs a detail template (derive from `capture/details/*.html`) plus per-type guts.

### 6.8 Media

Card html references images as `/media/<uid>/<card tail>/<blob id>;<W>x<H>[-c].webp`. Recorded bytes exist only for
the first 300 cards (`replay/`); everything else is a placeholder. For new uploads store the file and serve it from a
path of the same shape; the content type served for it is [?].

### 6.9 `GET /serendipity`, `/ui`, small reads

- `/serendipity`: HTML partial (about 30 KB), mostly static: `<script id="sound-effects">` JSON, a
  `serendipity-outro` section, and `.circles > .circle-1 > .inner-circle` containing 18
  `<div class="circle"><div class="card"><div class="card-rotate"><div class="card-anim"><mymind-card ...>` blocks, each
  followed by `<span class="button expand-button">`. Reuse the recorded body as the template and substitute 18 cards.
- `/ui` -> `{ "environmentName": "Production", "version": "0.109.0" }`.
- `/news/unread` -> `{ "count": 0 }`; `/news` -> HTML; `/achievements/unseen` -> `[]`.
- `/manifest.webmanifest`, `/account/blocks/subscription` (HTML snippet) are static.

## 7. Implementation notes for the SQLite backend

1. Replace the `server.mjs` non-GET stub with a router keyed by `(method, pattern)` from section 4; keep `200 {}` for
   unknown paths so the client never breaks.
2. One transaction per write: mutate rows, bump `modified_ns` (strictly increasing, it is the `x-checkpoint`), insert a
   `changes` row, then push hub events if the hub is implemented.
3. `GET /cards` and `/changes` share one html renderer; card html must change when title, highlight tag, deleted state
   or media change, otherwise the client's `modified`-based diff does not repaint.
4. Never forward a write to `access.mymind.com`.

## 8. Open questions

1. `POST /uploads/authorize`: exact authorization fields besides `url` (token name, expiry, host) and the exact
   chunk-PUT response for non-final chunks.
2. `POST /objects` full response body beyond `{id}`, and the exact rule that returns 200 versus 201 (URL
   normalization, content hash for files).
3. How `content` is classified as URL versus text, which scraper fills `title`, `description`, `siteName`, `domain`,
   `source`, cover, `type`, and which hosts map to special types (XPost, YouTubeVideo, Repository, ...).
4. Permanent delete and trash retention: no client call exists; is there server-side expiry and how long?
5. Whether `PATCH /objects/<id>` accepts `description`, `tags`, `flags`, and what it does with unknown keys.
6. Names of the `PUT /objects/<id>/<subresource>` routes used to replace a screenshot or image.
7. Meaning of card flag bits 20 to 23 and tag flag bits above 8, and of the unrecorded `x-magic` header.
8. `POST /batch/tag`, `/batch/delete`, `/batch/add-to-space`: response shapes (the client only needs 2xx).
9. `POST /convert` input handling and exact prose output for markdown constructs.
10. Payload of `type 2` (deleted) and space (`objectType 190`) items in `/changes`.
11. How the server decides the card `position` for new manual spaces (client sends `count + 1`).
12. What triggers a hub `object:bumped` versus `object:updated`, and whether the server pushes anything for settings.
13. Whether `x-object-quota.count` includes trashed cards, and the `storage` quota semantics shown in `config.quotas`.
14. The mobile no-JS `<form method="POST" action="/objects">` encoding (form-encoded fields and its redirect).
