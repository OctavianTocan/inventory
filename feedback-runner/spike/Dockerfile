# Throwaway image for the feedback spike (docs/feedback.md). Built from the repository root (wrangler.jsonc sets the
# context) so it reads the pinned CLI versions in rust/agent/cli-versions.env.
# The donor image ships sandbox-shim, which @cloudflare/sandbox's Files runs; keep its tag equal to the package version.
FROM docker.io/cloudflare/sandbox:1.0.3 AS sandbox-tools

FROM node:22-bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates git ripgrep python3 \
  && rm -rf /var/lib/apt/lists/*
COPY rust/agent/cli-versions.env /etc/cli-versions.env
# Claude Code's postinstall copies its native binary into place, so scripts must run.
RUN . /etc/cli-versions.env \
  && npm install -g "@anthropic-ai/claude-code@${CLAUDE_CODE_VERSION}" "@openai/codex@${CODEX_VERSION}" \
  && npm cache clean --force
RUN userdel -r node 2>/dev/null || true \
  && useradd -m -d /home/agent -u 1001 -s /bin/sh agent
COPY --from=sandbox-tools /usr/local/bin/sandbox-shim /usr/local/bin/sandbox-shim
WORKDIR /workspace
CMD ["sleep", "infinity"]
