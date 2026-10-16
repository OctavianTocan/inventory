# SQLite and semantic search

`GET /search?q=design&wildcard=true` returns exactly `{elapsed,count,query,matches:[{id,score}]}`. Matches use
descending relevance by default; `sortBy=bumped asc|desc`, `created asc|desc` and `modified asc|desc` select
timestamp order. `wildcard=true` makes the final unquoted free-text term a prefix. An explicit `word*` also
matches prefixes; unquoted words otherwise require complete tokens. FTS uses Unicode tokenization and removes
diacritics, with no replay lookup in the search implementation.

The external-content FTS5 index is built from every existing card on first open. Persistent SQLite triggers
update it transactionally for card data/extras, annotations and current reader content, including capture and
AI writes. Search excludes trash, private cards, drafts and stubs. Index installation requires a writable
database even in API verification mode; always use a temporary copy when the data directory is shared.

## Syntax

| Form | Behavior |
| --- | --- |
| `type:image`, `type:post`, `type:video` | Exact type or parent aliases emitted by the client. |
| `tag:design system`, `label:"design system"`, `#design system` | Case-insensitive annotation identity; a multiword tag consumes adjacent words until an operator/field. |
| `site:github`, `site:example.org`, `domain:sub.example.org` | Domain labels, exact domains and their subdomains. |
| `"silver birch"`, `blue*`, `title:orchard`, `note:idea` | Exact phrase, prefix, or a specified indexed field. |
| `foo -bar`, `NOT bar`, `+foo`, `(foo OR bar) AND baz`, `foo && baz`, `foo || bar` | Intersection, exclusion, grouping and union; AND has precedence. |

`before:2026-01-01`, `after:2025-01-01`, `date:2025-10-15` and `created:[2025-01-01 TO 2026-01-01]` compare UTC
creation timestamps. `created:>=2025-01-01`, `bumped:` and `modified:` also accept comparisons/ranges. Date-only
equality covers that UTC day. `format:pdf`, `completed:true|false`, `action:read|watch|listen|play` support the
client's named queries; actions group the corresponding media types. `word^2` weights a term. Caller text is
quoted before SQLite MATCH; structured predicates use bound parameters.

`GET /search/suggestions` lists active entity types with counts, `score=count*3`, and `isAI:false`. With `q`,
it returns tag-name prefix completions grouped by case-insensitive identity. Manual tags score twice their
count; other tags score their count. The store can return more than the recorded autocomplete cap; the client
displays at most 20. All 17 recorded type suggestions match exactly. All 50 recorded `q=design` tag suggestions
have matching counts; 43 match every recorded field, with remaining differences in canonical spelling/order.

## Recorded comparison

Measured on the seeded library (2,271 visible cards) on 2026-10-15. These are informational comparisons, not a
claim of live ranking parity. `tools/check-search.mjs` checks that overlap never falls below these baselines.

| Query, wildcard=true | Live | SQLite | Overlap | Live only | SQLite only |
| --- | ---: | ---: | ---: | ---: | ---: |
| de | 1,648 | 1,607 | 1,585 | 63 | 22 |
| des | 1,004 | 945 | 933 | 71 | 12 |
| desi | 845 | 799 | 789 | 56 | 10 |
| desig | 840 | 796 | 786 | 54 | 10 |
| design | 840 | 796 | 786 | 54 | 10 |

For every query, the main difference is live-only matches without a matching token prefix in the stored
searchable fields. Most are social posts. Missing server-side extracted text or derived metadata is a likely
explanation, inferred from those missing fields; the recording does not establish which hidden field matched.
SQLite-only matches include URL paths and stored body text that live search does not match the same way.
Changing to arbitrary substrings adds many false positives and does not recover the missing `design` matches.

Counts of live matches supported by each stored field overlap with one another:

| Query | Title | Description | Tags | URL/domain/site | Body | OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| de | 358 | 877 | 1,160 | 117 | 334 | 194 |
| des | 216 | 437 | 681 | 29 | 171 | 88 |
| desi | 201 | 322 | 638 | 25 | 144 | 60 |
| desig | 201 | 319 | 638 | 25 | 144 | 60 |
| design | 201 | 319 | 638 | 25 | 144 | 60 |

The seed has no stored summaries, and neither stored card note contributes to these five queries. Both summary
and note text are indexed and tested using synthetic writes. The comparison tool reads replay only to compute
these statistics; request-time search uses SQLite exclusively.

## Smart spaces and semantic search

`POST /spaces` with `query.filters` computes initial members from those filters, ignoring stale supplied
`query.matches`. `GET /queries/:id/check` evaluates the same syntax, returns `{elapsed,added,removed}`, and
persists the current membership before the next space read. Manual additions and query exclusions survive
refresh. In `STORE_READONLY=1`, an exact recorded request is replayed; unrecorded checks preview the difference
without writing rows, changing the checkpoint or sending hub events.

`GET /embeddings/search?q=&modelId=&take=&filter=` returns `{elapsed,count,query,matches}`; `modelId` values from
the client map to the one hosted `gemini-embedding-2` model used by stored 768-float vectors. `filter` uses the
text-query syntax. `GET /embeddings/related/:id?take=` uses cosine over local vectors, excludes self and needs
no API key. Missing source vectors return empty matches. `take` defaults to 100, is bounded to 1,000, and can
be zero. Related scores can be negative cosine similarities.

`semantic=true&semanticBoost=2` on `/search` adds positive cosine similarity times the boost to lexical scores
and includes semantic-only positive matches. Explicit field constraints and negative clauses remain filters;
eligibility is re-evaluated after the hosted call. Boost zero uses text results alone. No key, provider errors,
or OFFLINE without an explicit loopback `GEMINI_BASE_URL` degrade to text or empty semantic matches.

## Backfill and checks

`node tools/embed-library.mjs [--db path] [--limit N]` only reads the database and prints an estimate. `--run`
enables hosted embedding and transactions, unless `STORE_READONLY=1` or offline policy forbids the endpoint.
The text builder and Gemini document formatting are shared with new-card analysis; existing vectors and cards
edited during the API call are skipped. Text-only batches do not load images. Empty documents are skipped
rather than inventing content. Completed batches survive a later error and are skipped on rerun.

Seed estimate: 2,162 cards with usable text, 109 skipped without text, approximately 217,522 input tokens using
four characters per token, and **$0.043504 USD**. The constant is $0.20 per million standard text input tokens,
from [Google's Gemini Embedding 2 pricing](https://ai.google.dev/gemini-api/docs/pricing#gemini-embedding-2),
checked 2026-10-15. Synchronous `batchEmbedContents` groups requests; this tool does not use the discounted
asynchronous Batch API. Actual usage is printed when Gemini returns token metadata.

Run `node tools/check-search.mjs --in-process` in a socket-restricted environment. It copies the seed and also
uses synthetic fixtures, exercises real request callbacks and capture/AI mutations, tests Gemini locally,
and prints recording/field comparisons. On the host run `node tools/check-search.mjs` and
`node tools/check-capture.mjs` for real HTTP and the previously skipped Chromium checks. They use temporary
databases and OS-assigned ports, never port 4321. Browser/REA verification remains a separate host operation.

If reads/store checks differ only on recordings while a live verification run rewrites replay, rerun first.
For stable tests against the new recordings, seed a fresh temporary database and use `TEST_SOURCE_DB` for HTTP
checks plus `--db` for `check-store`; do not replace the shared seed or its local edits.
