"""Release regression tests use only disposable repositories and a fake Cargo."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parent.parent


class ReleaseFlowTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ic-memory-release-test-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.work = self.base / "work"
        self.remote = self.base / "origin.git"
        self.work.mkdir()
        self.env = os.environ.copy()
        self.env.pop("PUBLISH_DRY_RUN", None)
        self.env.pop("MAKEFLAGS", None)
        self.env.pop("MFLAGS", None)
        self.env["GIT_TERMINAL_PROMPT"] = "0"
        self.env["PYTHONDONTWRITEBYTECODE"] = "1"
        self.command("git", "init", "--bare", str(self.remote))
        self.command("git", "init", "-b", "main")
        for name, value in [("user.name", "Release Fixture"), ("user.email", "fixture@example.invalid"),
                            ("commit.gpgsign", "false"), ("tag.gpgsign", "false"),
                            ("core.hooksPath", "/dev/null")]:
            self.command("git", "config", name, value)
        (self.work / "scripts").mkdir()
        shutil.copyfile(REPO / "scripts/release.py", self.work / "scripts/release.py")
        shutil.copyfile(REPO / "Makefile", self.work / "Makefile")
        # Exercise the production Make orchestration while replacing expensive
        # Rust validation with a controllable process boundary.
        with (self.work / "Makefile").open("a") as file:
            file.write('\nvalidate:\n\t@test "$${FAIL_VALIDATE:-0}" != 1\n')
        (self.work / "Cargo.toml").write_text(
            '[package]\nname = "ic-memory"\nversion = "0.12.3"\nrust-version = "1.88.0"\n'
            '[dependencies]\nother = "0.12.3"\n')
        (self.work / "README.md").write_text('# Fixture\n\nic-memory = "0.12.3"\n')
        (self.work / "CHANGELOG.md").write_text('# Changelog\n\n## 0.13.0\n\nFixture changes.\n\n## 0.12.3\n\nHistory.\n')
        (self.work / ".gitignore").write_text('target/\nCargo.lock\n')
        (self.work / "source.rs").write_text('// committed source\n')
        self.command("git", "add", ".")
        self.command("git", "commit", "-m", "Prepare source and release notes")
        self.source = self.command("git", "rev-parse", "HEAD").stdout.strip()
        self.command("git", "remote", "add", "origin", str(self.remote))
        self.command("git", "push", "-u", "origin", "main")
        (self.work / "Cargo.lock").write_text("original lockfile\n")
        fakebin = self.base / "bin"
        fakebin.mkdir()
        cargo = fakebin / "cargo"
        cargo.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
root = pathlib.Path.cwd()
(root / 'target').mkdir(exist_ok=True)
with (root / 'target/cargo.jsonl').open('a') as log:
    log.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1] in ('update', 'generate-lockfile'):
    (root / 'Cargo.lock').write_text('resolved lockfile\\n')
if sys.argv[1] == 'package' and os.environ.get('FAIL_PACKAGE') == '1':
    sys.exit(42)
''')
        cargo.chmod(0o755)
        self.env["PATH"] = str(fakebin) + os.pathsep + self.env["PATH"]

    def command(self, *args, success=True, env=None, cwd=None):
        result = subprocess.run(args, cwd=cwd or self.work, env=env or self.env,
                                text=True, capture_output=True)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def make(self, *targets, **kwargs):
        return self.command("make", "--no-print-directory", *targets, **kwargs)

    def calls(self):
        log = self.work / "target/cargo.jsonl"
        return [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []

    def assert_unmodified_version(self):
        self.assertEqual(self.command("git", "rev-parse", "HEAD").stdout.strip(), self.source)
        self.assertIn('version = "0.12.3"', (self.work / "Cargo.toml").read_text())
        self.assertIn('ic-memory = "0.12.3"', (self.work / "README.md").read_text())

    def test_minor_release_push_and_separate_publish(self):
        self.make("release-minor")
        head = self.command("git", "rev-parse", "HEAD").stdout.strip()
        self.assertNotEqual(head, self.source)
        self.assertEqual(self.command("git", "rev-parse", "HEAD^").stdout.strip(), self.source)
        self.assertEqual(self.command("git", "--git-dir", str(self.remote), "rev-parse", "refs/heads/main").stdout.strip(), head)
        self.assertEqual(self.command("git", "cat-file", "-t", "refs/tags/v0.13.0").stdout.strip(), "tag")
        manifest = (self.work / "Cargo.toml").read_text()
        self.assertIn('version = "0.13.0"', manifest)
        self.assertIn('other = "0.12.3"', manifest)
        self.assertEqual(self.command("git", "status", "--porcelain").stdout, "")
        self.assertFalse(any(call[0] == "publish" for call in self.calls()))
        self.make("publish-dry-run")
        self.assertEqual(self.calls()[-1], ["publish", "--locked", "--registry", "crates-io", "--dry-run"])
        self.make("publish", env={**self.env, "PUBLISH_DRY_RUN": "1"})
        self.assertEqual(self.calls()[-1][-1], "--dry-run")
        self.make("publish")  # Fake Cargo: no network/publication can occur.
        self.assertEqual(self.calls()[-1], ["publish", "--locked", "--registry", "crates-io"])
        self.make("release-commit", "release-push")  # Idempotent retry.
        self.assertEqual(self.command("git", "rev-parse", "HEAD").stdout.strip(), head)

    def test_patch_prepare_and_explicit_remaining_steps(self):
        path = self.work / "CHANGELOG.md"
        path.write_text(path.read_text().replace("0.13.0", "0.12.4"))
        self.command("git", "add", "CHANGELOG.md")
        self.command("git", "commit", "-m", "Prepare patch notes")
        self.source = self.command("git", "rev-parse", "HEAD").stdout.strip()
        self.make("patch")
        self.assertEqual(self.command("git", "rev-parse", "HEAD").stdout.strip(), self.source)
        self.assertIn('version = "0.12.4"', (self.work / "Cargo.toml").read_text())
        self.assertEqual(self.command("git", "tag", "--list").stdout, "")
        self.make("release-stage", "release-commit", "release-push")
        self.assertEqual(self.command("git", "log", "-1", "--format=%s").stdout.strip(), "Release 0.12.4")

    def test_dirty_source_and_wrong_changelog_fail_before_validation(self):
        (self.work / "untracked.txt").write_text("unfinished")
        self.make("minor", success=False)
        self.assert_unmodified_version()
        (self.work / "untracked.txt").unlink()
        self.make("patch", success=False)  # Draft is 0.13.0, not 0.12.4.
        self.assert_unmodified_version()
        (self.work / "source.rs").write_text("// staged edit\n")
        self.command("git", "add", "source.rs")
        self.make("minor", success=False)
        self.assert_unmodified_version()
        self.assertEqual(self.calls(), [])

    def test_validation_failure_never_bumps(self):
        self.make("minor", success=False, env={**self.env, "FAIL_VALIDATE": "1"})
        self.assert_unmodified_version()
        self.assertEqual(self.calls(), [])

    def test_final_package_failure_rolls_back_version_and_lockfile(self):
        self.make("minor", success=False, env={**self.env, "FAIL_PACKAGE": "1"})
        self.assert_unmodified_version()
        self.assertEqual((self.work / "Cargo.lock").read_text(), "original lockfile\n")
        self.assertFalse((self.work / "target/release-validation/prepared.json").exists())
        self.assertEqual(self.command("git", "status", "--porcelain").stdout, "")

    def test_staged_pollution_and_edited_release_surfaces_are_rejected(self):
        self.make("minor")
        self.make("release-stage")
        (self.work / "source.rs").write_text("// unvalidated change\n")
        self.command("git", "add", "source.rs")
        self.make("release-commit", success=False)
        self.assertEqual(self.command("git", "rev-parse", "HEAD").stdout.strip(), self.source)
        # Restore only disposable fixture state to isolate partial staging.
        self.command("git", "restore", "--source=HEAD", "--staged", "--worktree", "source.rs")
        with (self.work / "README.md").open("a") as file:
            file.write("unvalidated docs\n")
        self.make("release-commit", success=False)
        self.assertEqual(self.command("git", "tag", "--list").stdout, "")

    def test_stale_source_receipt_is_rejected(self):
        self.make("minor")
        self.command("git", "commit", "--allow-empty", "-m", "Source moved")
        self.make("release-stage", success=False)
        self.assertEqual(self.command("git", "tag", "--list").stdout, "")

    def test_remote_tag_collision_prevents_version_changes(self):
        self.command("git", "--git-dir", str(self.remote), "tag", "v0.13.0", "refs/heads/main")
        self.make("minor", success=False)
        self.assert_unmodified_version()
        self.assertEqual(self.calls(), [])

    def test_remote_ahead_prevents_version_changes(self):
        other = self.base / "other"
        self.command("git", "clone", "--branch", "main", str(self.remote), str(other))
        self.command("git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                     "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "Remote advanced", cwd=other)
        self.command("git", "push", "origin", "main", cwd=other)
        self.make("minor", success=False)
        self.assert_unmodified_version()
        self.assertEqual(self.calls(), [])

    def test_atomic_push_rejection_does_not_advance_remote_branch(self):
        hook = self.remote / "hooks/update"
        hook.write_text('#!/bin/sh\ncase "$1" in refs/tags/*) exit 1;; esac\n')
        hook.chmod(0o755)
        self.make("release-minor", success=False)
        self.assertEqual(self.command("git", "--git-dir", str(self.remote), "rev-parse", "refs/heads/main").stdout.strip(), self.source)
        self.assertEqual(self.command("git", "--git-dir", str(self.remote), "tag", "--list").stdout, "")
        hook.unlink()
        self.make("release-push")

    def test_publish_rejects_dirty_or_untagged_release(self):
        self.make("minor", "release-stage", "release-commit")
        calls = self.calls()
        (self.work / "untracked.txt").write_text("unfinished")
        self.make("publish", success=False)
        self.assertEqual(self.calls(), calls)
        (self.work / "untracked.txt").unlink()
        self.command("git", "tag", "-d", "v0.13.0")
        self.make("publish-dry-run", success=False)
        self.assertEqual(self.calls(), calls)
        self.make("release-commit")  # Recover a missing tag without a new commit.
        self.make("publish-dry-run")


if __name__ == "__main__":
    unittest.main()
