//! The server inside a login's hosted agent container (`rust/agent/Dockerfile`): runs that login's Claude Code and
//! Codex for the Worker. It refuses to start outside the container.
//!
//! Environment: `MYMIND_AGENT_RUNNER` (listen address), `MYMIND_AGENT_HOMES` (login homes, default `/homes`),
//! `MYMIND_AGENT_LOGIN` (the login's volume, set by the Worker's `AgentHome`), `MYMIND_AGENT_CLI_USER` (`uid:gid`).

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
    mm_ai::configure();
    let address = var("MYMIND_AGENT_RUNNER").context("The agent runner needs MYMIND_AGENT_RUNNER")?.parse()?;
    let homes = var("MYMIND_AGENT_HOMES").unwrap_or_else(|| "/homes".into()).into();
    let login = var("MYMIND_AGENT_LOGIN").context("The agent runner needs MYMIND_AGENT_LOGIN")?;
    let cli = var("MYMIND_AGENT_CLI_USER")
        .as_deref()
        .and_then(|user| user.split_once(':'))
        .and_then(|(uid, gid)| Some((uid.parse().ok()?, gid.parse().ok()?)))
        .context("The agent runner needs MYMIND_AGENT_CLI_USER as uid:gid")?;
    ai_kit::runner::serve(address, homes, login, cli).await
}

#[cfg(target_arch = "wasm32")]
fn main() {}
