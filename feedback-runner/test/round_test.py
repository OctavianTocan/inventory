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
        job = {"engine": "claude", "model": "opus", "effort": "high"}
        argv = round_.agent_argv(job, "p")
        self.assertIn("--effort", argv)
        self.assertEqual(argv[-2:], ["--", "p"])


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
