from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
CHECK = SCRIPTS.parent / ".ai-first/vcs.py"


class RepositoryTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="vcs-contract-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / "repo"
        self.root.mkdir()
        self.env = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        self.env.pop("AI_FIRST_VCS_SESSION", None)

    def run_command(self, *args, ok=True):
        result = subprocess.run(args, cwd=self.root, env=self.env, text=True, capture_output=True)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        return result

    def check(self, *args, ok=True):
        result = self.run_command("python3", str(CHECK), "--repo-root", str(self.root), *args, ok=False)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0)
        return result



@unittest.skipUnless(shutil.which("jj"), "jj is required for finalizer tests")
class FinalizerTests(RepositoryTest):
    def setUp(self):
        super().setUp()
        self.run_command("jj", "git", "init", "--colocate")
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.test"), ("signing.behavior", "drop")):
            self.run_command("jj", "config", "set", "--repo", key, value)
        (self.root / "baseline").write_text("baseline")
        self.run_command("jj", "describe", "-m", "test: baseline")
        self.run_command("jj", "bookmark", "create", "main", "-r", "@")
        self.run_command("jj", "new", "main")


    def value(self, revision, template):
        return self.run_command("jj", "log", "-r", revision, "--no-graph", "-T", template).stdout.strip()


    def prepare_finalizer(self, gate_exit=0):
        scripts = self.root / "scripts"
        scripts.mkdir()
        for name in ("finalize-change.sh",):
            shutil.copyfile(SCRIPTS / name, scripts / name)
        runtime = self.root / ".ai-first"
        runtime.mkdir()
        shutil.copyfile(SCRIPTS.parent / ".ai-first/vcs.py", runtime / "vcs.py")
        publication = scripts / "check-publication-boundary.py"
        publication.write_text("#!/bin/sh\nexit 0\n")
        publication.chmod(0o755)
        gate = scripts / "check.sh"
        gate.write_text(f"#!/bin/sh\nexit {gate_exit}\n")
        gate.chmod(0o755)
        config = self.base / "codex"
        config.mkdir()
        (config / "commit-attribution").write_text("Fixture <fixture@example.test>\n")
        self.env["CODEX_HOME"] = str(config)


    def finalize(self, *args, ok=True):
        return self.run_command("bash", "scripts/finalize-change.sh", "--message", "fix: fixture change", *args, ok=ok)


    def test_finalizer_keeps_main_and_opens_an_empty_working_copy(self):
        self.prepare_finalizer()
        old_main = self.value("main", "commit_id")
        self.finalize()
        self.assertEqual(self.value("main", "commit_id"), old_main)
        self.assertEqual(self.value("@", "empty"), "true")
        self.assertEqual(self.value("parents(@)", "description").splitlines()[-1], "Co-authored-by: Fixture <fixture@example.test>")
        self.check("--closeout", "--revision", "@-")


    def test_gate_failure_stops_before_description_and_bookmark_changes(self):
        self.prepare_finalizer(gate_exit=17)
        before = self.value("main", "commit_id")
        description = self.value("@", "description")
        self.assertNotEqual(self.finalize("--bookmark", "main", ok=False).returncode, 0)
        self.assertEqual(self.value("main", "commit_id"), before)
        self.assertEqual(self.value("@", "description"), description)


    def test_explicit_main_move_requires_a_linear_candidate(self):
        self.prepare_finalizer()
        self.finalize("--bookmark", "main")
        self.assertEqual(self.value("main", "commit_id"), self.value("@-", "commit_id"))
        self.check("--closeout")


    def test_failed_attribution_helper_does_not_fall_back(self):
        self.prepare_finalizer()
        helper = self.base / "codex/skills/vcs-jj/scripts/describe_with_attribution.sh"
        helper.parent.mkdir(parents=True)
        helper.write_text("#!/bin/sh\nexit 19\n")
        helper.chmod(0o755)
        before = self.value("@", "description")
        self.assertEqual(self.finalize(ok=False).returncode, 19)
        self.assertEqual(self.value("@", "description"), before)


    def test_finalizer_rejects_a_merge_before_moving_main(self):
        self.prepare_finalizer()
        main = self.value("main", "commit_id")
        left = self.value("@", "commit_id")
        self.run_command("jj", "new", "main")
        (self.root / "right").write_text("right")
        right = self.value("@", "commit_id")
        self.run_command("jj", "new", left, right)
        (self.root / "merge-edit").write_text("merge")
        result = self.finalize("--bookmark", "main", ok=False)
        self.assertIn("merge commits", result.stderr)
        self.assertEqual(self.value("main", "commit_id"), main)


    def test_finalizer_refuses_an_empty_change_without_creating_another_head(self):
        self.prepare_finalizer()
        self.run_command("jj", "describe", "-m", "test: local tools")
        self.run_command("jj", "new")
        before = self.value("@", "commit_id")
        result = self.finalize(ok=False)
        self.assertIn("empty working copy", result.stderr)
        self.assertEqual(self.value("@", "commit_id"), before)

    def test_finalizer_connects_the_started_session_before_main_move(self):
        self.prepare_finalizer()
        self.run_command("jj", "describe", "-m", "test: fixture tools")
        self.run_command("jj", "bookmark", "set", "main", "-r", "@")
        self.run_command("jj", "new", "main")
        state = self.base / "session.json"
        self.env["AI_FIRST_VCS_SESSION"] = str(state)
        self.run_command("python3", ".ai-first/vcs.py", "start")
        (self.root / "task-change").write_text("completed")
        self.finalize("--bookmark", "main")
        self.run_command("python3", ".ai-first/vcs.py", "finish")
        self.assertEqual(self.value("@", "empty"), "true")

    def test_missing_session_stops_finalizer_before_main_move(self):
        self.prepare_finalizer()
        self.env["AI_FIRST_VCS_SESSION"] = str(self.base / "missing.json")
        before = self.value("main", "commit_id")
        self.assertNotEqual(self.finalize("--bookmark", "main", ok=False).returncode, 0)
        self.assertEqual(self.value("main", "commit_id"), before)

    def test_native_start_rejects_missing_session_before_creating_packet(self):
        self.prepare_finalizer()
        shutil.copyfile(SCRIPTS / "start-work.sh", self.root / "scripts/start-work.sh")
        self.env["AI_FIRST_VCS_SESSION"] = str(self.base / "missing.json")
        result = self.run_command("bash", "scripts/start-work.sh", "--work-id", "fixture-task", ok=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "docs/todo-fixture-task").exists())


if __name__ == "__main__":
    unittest.main()
