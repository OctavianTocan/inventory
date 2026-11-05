# Live eval

`questions.xml` holds ten read-only questions about the `TEST_USER_*` account's library on
`mymind.octaviantocan.com`, in mcp-builder's evaluation format. Each needs more than one tool call (find the space or
card, then open it), and each answer is a fact inside a saved card, so it stays true until that card is edited.

`live.ts` signs in as the test user with the password route, gives the tools that app token directly (no OAuth), runs
them through the real MCP client and server, and signs the token out at the end. Only read tools run unless `--write`
is given.

```bash
set -a; . <(grep '^TEST_USER' ~/apps/mymind-reconstruction/.env); set +a
npx tsx eval/live.ts thymind_list_spaces '{}' thymind_search '{"query":"neovim","limit":5}'   # call tools by hand
npx tsx eval/live.ts --eval eval/questions.xml   # ask Claude Code (claude -p) each question with only these tools
```

`--eval` serves the tools on a free 127.0.0.1 port for the run and needs a signed-in `claude` on the machine; it
prints PASS or FAIL per question and the score.

## Results

5 Nov 2026: the answers were checked by hand with the tools, then a small agent (Haiku) given only the questions and
the read tools answered 10 of 10 right in 15 tool calls. The run found three things, all fixed in the same change:
pinned cards imported from mymind showed `pinned: false` (they carry the pin flag but no `pinPosition`),
descriptions kept HTML entities such as `&lsquo;`, and authors came as "by Name", so cards read "By by Name".
