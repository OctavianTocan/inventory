# The hosted agent container: one ThyMind account's Claude Code and Codex, run by the AI kit's agent runner
# (`mymind-agent-runner`, crates/ai) as plain processes, since this container is the sandbox. Built from the repository
# root (crates/worker/wrangler.jsonc sets the build context); Dockerfile.dockerignore keeps the context small.
FROM rust:1.99-bookworm AS build
WORKDIR /src
COPY rust/Cargo.toml rust/Cargo.lock ./
COPY rust/crates ./crates
COPY rust/test-support ./test-support
COPY vendor/ai-kit /vendor/ai-kit
RUN cargo build --release --locked -p mm-ai --bin mymind-agent-runner

FROM node:22-bookworm-slim
# The CLI versions come from cli-versions.env; a non-empty build argument overrides one for a trial build.
ARG CLAUDE_CODE_VERSION=
ARG CODEX_VERSION=

# git and ripgrep support the CLIs; flock (util-linux) guards login updates.
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates git ripgrep util-linux \
  && rm -rf /var/lib/apt/lists/*

COPY rust/agent/cli-versions.env /etc/cli-versions.env
RUN claude="${CLAUDE_CODE_VERSION}" codex="${CODEX_VERSION}" \
  && . /etc/cli-versions.env \
  && npm install -g "@anthropic-ai/claude-code@${claude:-$CLAUDE_CODE_VERSION}" "@openai/codex@${codex:-$CODEX_VERSION}" \
  && npm cache clean --force

# The runner runs as root and keeps the login homes in /homes, which only root can read; each CLI job runs as
# thymind-cli in its own job folder, so a run cannot read a saved login.
RUN userdel -r node 2>/dev/null || true \
  && groupadd -g 1001 thymind-cli \
  && useradd -M -d /nonexistent -s /usr/sbin/nologin -u 1001 -g thymind-cli thymind-cli

COPY --from=build /src/target/release/mymind-agent-runner /usr/local/bin/mymind-agent-runner
# The marker lets the AI kit's runner start; the native server never has it.
RUN touch /.thymind-agent-container && install -d -o root -g root -m 0700 /homes

ENV DISABLE_AUTOUPDATER=1 \
  MYMIND_AGENT_RUNNER=0.0.0.0:8080 \
  MYMIND_AGENT_HOMES=/homes \
  MYMIND_AGENT_CLI_USER=1001:1001
WORKDIR /
EXPOSE 8080
ENTRYPOINT ["mymind-agent-runner"]
