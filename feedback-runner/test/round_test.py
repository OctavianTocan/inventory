"""Tests for round.py's pure parts and its CHANGELOG step, run by test/round.test.ts."""
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
spec = importlib.util.spec_from_file_location("round", os.path.join(HERE, "..", "src", "round.py"))
round_ = importlib.util.module_from_spec(spec)
spec.loader.exec_module(round_)


class Answer(unittest.TestCase):
    def test_reads_the_closing_json_line(self):
        text = 'Done.\n{"summary": "Share sheet stays open", "description": "Raised the delay.", "no_change": false}'
        self.assertEqual(round_.answer(text), {"summary": "Share sheet stays open", "description": "Raised the delay.", "no_change": False})

    def test_empty_when_missing(self):
        self.assertEqual(round_.answer("no json here")["summary"], "")

    def test_final_text_of_each_engine(self):
        claude = json.dumps({"type": "result", "result": "ok", "is_error": False})
        self.assertEqual(round_.final_text("claude", f"noise\n{claude}\n"), ("ok", False))
        codex = json.dumps({"type": "item.completed", "item": {"type": "agent_message", "text": "hi"}})
        self.assertEqual(round_.final_text("codex", codex), ("hi", False))

    def test_argv_passes_model_and_effort_like_ai_kit(self):
        job = {"engine": "codex", "model": "gpt-6.1", "effort": "none"}
        self.assertEqual(round_.agent_argv(job, "p")[-3:], ["-m", "gpt-6.1", "p"])
        self.assertIn('model_providers.chatgpt_plan.env_key="ACCESS_TOKEN"', round_.agent_argv(job, "p"))
        job = {"engine": "claude", "model": "opus", "effort": "high"}
        argv = round_.agent_argv(job, "p")
        self.assertIn("--effort", argv)
        self.assertEqual(argv[-2:], ["--", "p"])


class ChatGptToken(unittest.TestCase):
    """chatgpt_token against a stand-in for OpenAI's token endpoint."""

    def setUp(self):
        self.dir = tempfile.mkdtemp()
        self.path = f"{self.dir}/chatgpt.json"

    def tearDown(self):
        shutil.rmtree(self.dir)

    def write(self, expires_in):
        login = {"client_id": "app_1", "host_id": "urn:uuid:h", "scope": "chatgpt.tokens.use.direct",
                 "expires_at": int((round_.time.time() + expires_in) * 1000),
                 "tokens": {"access_token": "a1", "refresh_token": "r1", "id_token": "i1"}}
        with open(self.path, "w") as handle:
            json.dump(login, handle)

    def test_a_token_that_outlasts_the_round_is_used_as_it_is(self):
        self.write(3 * 3600)
        self.assertEqual(round_.chatgpt_token(self.path, post=lambda *_: self.fail("refreshed")), "a1")

    def test_a_refresh_rotates_and_saves_the_login(self):
        self.write(1800)
        sent = []
        token = round_.chatgpt_token(self.path, post=lambda url, form: sent.append(form) or (200, {"access_token": "a2", "refresh_token": "r2", "expires_in": 3600}))
        self.assertEqual(token, "a2")
        self.assertEqual(sent[0]["refresh_token"], "r1")
        self.assertEqual(sent[0]["client_id"], "app_1")
        with open(self.path) as handle:
            saved = json.load(handle)
        self.assertEqual(saved["tokens"], {"access_token": "a2", "refresh_token": "r2", "id_token": "i1"})
        self.assertEqual(saved["host_id"], "urn:uuid:h")

    def test_a_refused_refresh_or_missing_file_is_signed_out(self):
        self.write(60)
        with self.assertRaises(round_.SignedOut):
            round_.chatgpt_token(self.path, post=lambda *_: (400, {"error": "invalid_grant"}))
        with self.assertRaises(round_.SignedOut):
            round_.chatgpt_token(f"{self.dir}/missing.json")


class Bump(unittest.TestCase):
    """bump_if_needed against a small copy of the repository's release script and CHANGELOG."""

    def setUp(self):
        self.dir = tempfile.mkdtemp()
        os.makedirs(f"{self.dir}/deploy")
        os.makedirs(f"{self.dir}/android/app/src/main")
        shutil.copy(f"{ROOT}/deploy/android-release.py", f"{self.dir}/deploy/")
        with open(f"{self.dir}/android/CHANGELOG.md", "w") as handle:
            handle.write("# Changelog\n\n## 0.6.53 (64), 2026-11-05\n- Search.\n")
        for args in (["init", "-q"], ["config", "user.email", "t@t"], ["config", "user.name", "t"], ["add", "-A"], ["commit", "-qm", "base"]):
            subprocess.run(["git", *args], cwd=self.dir, check=True)
        self.base = subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.dir, check=True, capture_output=True, text=True).stdout.strip()
        round_.REPO_DIR = self.dir
        round_.run_original = round_.run
        round_.run = lambda args, **kw: round_.run_original(args, **{**kw, "as_agent": False, "cwd": self.dir})

    def tearDown(self):
        round_.run = round_.run_original
        shutil.rmtree(self.dir)

    def commit(self, path):
        with open(f"{self.dir}/{path}", "w") as handle:
            handle.write("change\n")
        subprocess.run(["git", "add", "-A"], cwd=self.dir, check=True)
        subprocess.run(["git", "commit", "-qm", "change"], cwd=self.dir, check=True)

    def test_adds_the_next_entry_for_an_app_change(self):
        self.commit("android/app/src/main/Thing.kt")
        self.assertEqual(round_.bump_if_needed(self.base, "Share sheet stays open"), "0.6.54")
        with open(f"{self.dir}/android/CHANGELOG.md") as handle:
            self.assertRegex(handle.read(), r"## 0\.6\.54 \(65\), \d{4}-\d{2}-\d{2}\n- Share sheet stays open\n\n## 0\.6\.53")

    def test_adds_nothing_for_a_server_change(self):
        self.commit("server.rs")
        self.assertIsNone(round_.bump_if_needed(self.base, "x"))


if __name__ == "__main__":
    unittest.main()
