#!/usr/bin/env python3
"""One feedback round inside its sandbox (docs/feedback.md, FeedbackSandbox). Started in the background by the
runner, which polls /workspace/round/status.json and reads result.json when it is done. Standard library only.

Steps: clone without credentials (the outbound handler adds them), run the agent as the agent user with a time limit,
read its final JSON line, add the CHANGELOG entry the release gate asks for, refuse to push a commit holding a sign-in
file's text, then push feedback/<id>."""
import datetime
import json
import os
import re
import subprocess
import sys
import time

ROUND = "/workspace/round"
REPO_DIR = "/workspace/repo"
HOME = "/home/agent"
AGENT = "agent"
AGENT_TIMEOUT = 45 * 60
STARTED = time.time()
ENV = {
    "HOME": HOME,
    "PATH": "/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin",
    "CARGO_HOME": f"{HOME}/.cargo",
    "RUSTUP_HOME": "/usr/local/rustup",
    "SSL_CERT_FILE": "/etc/ssl/certs/ca-certificates.crt",
    "GIT_SSL_CAINFO": "/etc/ssl/certs/ca-certificates.crt",
    "NODE_EXTRA_CA_CERTS": "/etc/cloudflare/certs/cloudflare-containers-ca.crt",
    "DISABLE_AUTOUPDATER": "1",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    "IS_SANDBOX": "1",
    "GIT_TERMINAL_PROMPT": "0",
}
PLAN_LIMIT = re.compile(r"usage limit|limit reached|hit your (usage )?limit|quota exceeded|out of credits", re.I)
SIGN_IN = re.compile(r"authentication_error|invalid_grant|oauth token (has )?expired|token expired|please run /login|not logged in|log in again|401 unauthorized", re.I)


def write(name, data):
    path = f"{ROUND}/{name}"
    with open(f"{path}.tmp", "w") as handle:
        json.dump(data, handle)
    os.replace(f"{path}.tmp", path)


def phase(name):
    write("status.json", {"phase": name, "at": int(time.time())})


def run(args, timeout=600, check=True, cwd=REPO_DIR, as_agent=True):
    """Runs a command (as the agent user unless told otherwise) and returns its result."""
    result = subprocess.run(
        (["setpriv", "--reuid", AGENT, "--regid", AGENT, "--init-groups"] if as_agent else []) + args,
        cwd=cwd, env=ENV, capture_output=True, text=True, timeout=timeout,
    )
    if check and result.returncode != 0:
        raise RuntimeError(f"{' '.join(args[:3])} failed: {(result.stderr or result.stdout)[-600:]}")
    return result


def git(*args, check=True, timeout=600):
    return run(["git", *args], check=check, timeout=timeout)


def agent_argv(job, prompt):
    """The CLI command line, built as ai-kit's process.rs builds it for the chosen model and effort."""
    model, effort = job.get("model"), job.get("effort")
    if job["engine"] == "claude":
        return ["claude", "-p", "--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions",
                "--no-session-persistence", "--max-turns", "200",
                *(["--model", model] if model else []),
                *(["--effort", effort] if effort and effort != "none" else []),
                "--", prompt]
    return ["codex", "exec", "--json", "--sandbox", "danger-full-access", "--skip-git-repo-check",
            *(["-m", model] if model else []),
            *(["-c", f'model_reasoning_effort="{effort}"'] if effort and effort != "none" else []),
            prompt]


def final_text(engine, stdout):
    """The agent's last message from its JSON event stream, and whether the CLI called it an error."""
    events = []
    for line in stdout.splitlines():
        try:
            events.append(json.loads(line))
        except ValueError:
            pass
    if engine == "claude":
        results = [e for e in events if e.get("type") == "result"]
        if results:
            return str(results[-1].get("result") or results[-1].get("subtype") or ""), bool(results[-1].get("is_error"))
        return "", True
    messages = [e["item"].get("text", "") for e in events
                if e.get("type") == "item.completed" and isinstance(e.get("item"), dict) and e["item"].get("type") == "agent_message"]
    failed = any(e.get("type") in ("error", "turn.failed") for e in events)
    errors = [json.dumps(e) for e in events if e.get("type") in ("error", "turn.failed")]
    return (messages[-1] if messages else " ".join(errors)), failed and not messages


def answer(text):
    """The agent's closing `{summary, description, no_change}` line, or an empty answer."""
    for line in reversed(text.strip().splitlines()):
        line = line.strip().strip("`")
        if line.startswith("{") and line.endswith("}"):
            try:
                data = json.loads(line)
                return {"summary": str(data.get("summary", ""))[:200], "description": str(data.get("description", ""))[:4000],
                        "no_change": bool(data.get("no_change"))}
            except ValueError:
                continue
    return {"summary": "", "description": "", "no_change": False}


def sign_in_strings():
    """Every string of 16 or more characters in the round's sign-in files."""
    found = set()

    def walk(value):
        if isinstance(value, str) and len(value) >= 16:
            found.add(value)
        elif isinstance(value, list):
            for item in value:
                walk(item)
        elif isinstance(value, dict):
            for item in value.values():
                walk(item)

    for path in (".claude/.credentials.json", ".claude.json", ".codex/auth.json"):
        try:
            with open(f"{HOME}/{path}") as handle:
                walk(json.load(handle))
        except (OSError, ValueError):
            pass
    return found


def bump_if_needed(base, summary):
    """Adds the CHANGELOG entry the release gate asks for as its own commit; returns the new version or None."""
    gate = run(["python3", "deploy/android-release.py", "gate", "--base", base, "--head", "HEAD"], check=False)
    if gate.returncode == 0:
        return None
    if "add a newer versionName" not in gate.stderr:
        raise RuntimeError(f"release gate failed: {gate.stderr[-600:]}")
    path = f"{REPO_DIR}/android/CHANGELOG.md"
    with open(path) as handle:
        lines = handle.read().split("\n")
    heading = re.compile(r"## (\d+)\.(\d+)\.(\d+) \((\d+)\), \d{4}-\d{2}-\d{2}")
    first = next(i for i, line in enumerate(lines) if heading.fullmatch(line.rstrip()))
    major, minor, patch, code = map(int, heading.fullmatch(lines[first].rstrip()).groups())
    name = f"{major}.{minor}.{patch + 1}"
    today = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    note = (summary or "A change from feedback.").replace("\n", " ").strip()
    lines[first:first] = [f"## {name} ({code + 1}), {today}", f"- {note}", ""]
    with open(path, "w") as handle:
        handle.write("\n".join(lines))
    git("add", "android/CHANGELOG.md")
    git("commit", "-q", "-m", f"Android {name} ({code + 1})")
    run(["python3", "deploy/android-release.py", "gate", "--base", base, "--head", "HEAD"])
    return name


def main():
    with open(f"{ROUND}/input.json") as handle:
        job = json.load(handle)
    with open(f"{ROUND}/prompt.txt") as handle:
        prompt = handle.read()
    branch = f"feedback/{job['id']}"
    result = {"done": True, "outcome": "error", "summary": "", "description": "", "head": None, "base": None,
              "bumped": None, "error": None}
    try:
        phase("clone")
        git("clone", "--depth", "1", f"https://github.com/{job['repo']}", REPO_DIR, timeout=900)
        git("config", "user.name", "ThyMind feedback")
        git("config", "user.email", "feedback@thymind.invalid")
        if job["round"] > 1 or job.get("conflict"):
            git("fetch", "--depth", "200", "origin", f"+refs/heads/{branch}:refs/remotes/origin/{branch}", "+refs/heads/main:refs/remotes/origin/main", timeout=900)
            git("switch", "-c", branch, f"origin/{branch}")
        else:
            git("switch", "-c", branch)
        base = git("rev-parse", "origin/main").stdout.strip() if job.get("conflict") else (job.get("base") or git("rev-parse", "HEAD").stdout.strip())
        git("fetch", "--depth", "1", "origin", base, check=False)
        result["base"] = base
        if job.get("conflict"):
            git("merge", "--no-edit", "origin/main", check=False)
        start = git("rev-parse", "HEAD").stdout.strip()

        phase("agent")
        try:
            agent = run(agent_argv(job, prompt), timeout=AGENT_TIMEOUT, check=False)
        except subprocess.TimeoutExpired:
            result["outcome"] = "timeout"
            return result
        with open(f"{ROUND}/agent.jsonl", "w") as handle:
            handle.write(agent.stdout[-2_000_000:])
        text, failed = final_text(job["engine"], agent.stdout)
        said = answer(text)
        result.update(summary=said["summary"], description=said["description"])
        if agent.returncode != 0 or failed:
            blob = f"{text}\n{agent.stderr[-4000:]}"
            if PLAN_LIMIT.search(blob):
                result["outcome"] = "plan_limit"
            elif SIGN_IN.search(blob):
                result["outcome"] = "signin"
            else:
                result["error"] = f"the agent exited with {agent.returncode}"
            return result

        phase("check")
        if git("status", "--porcelain").stdout.strip() and not job.get("conflict"):
            # Work left uncommitted is kept: the agent was told to commit.
            git("add", "-A")
            git("commit", "-q", "-m", said["summary"] or "Feedback change")
        if os.path.exists(f"{REPO_DIR}/.git/MERGE_HEAD"):
            result["error"] = "the merge with main was left unfinished"
            return result
        if git("rev-list", "--count", f"{start}..HEAD").stdout.strip() == "0" and not job.get("conflict"):
            result["outcome"] = "no_commits"
            return result
        result["bumped"] = bump_if_needed(base, said["summary"])
        diff = git("diff", base, "HEAD").stdout
        if any(secret in diff for secret in sign_in_strings()):
            result["error"] = "a commit holds a sign-in file's text"
            return result

        phase("push")
        git("push", "origin", f"HEAD:refs/heads/{branch}", timeout=900)
        result["head"] = git("rev-parse", "HEAD").stdout.strip()
        result["outcome"] = "pushed"
        return result
    except Exception as error:  # noqa: BLE001 - every failure ends the round with its reason
        result["error"] = str(error)[-1000:]
        return result


if __name__ == "__main__":
    final = main()
    final["seconds"] = int(time.time() - STARTED)
    write("result.json", final)
    phase("done")
    sys.exit(0)
