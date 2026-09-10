#!/usr/bin/env python3
"""Single-crate adaptation of Canic's validate/bump/commit/tag/push flow."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
SURFACES = ("Cargo.toml", "README.md")
RECEIPT = Path("target/release-validation/prepared.json")


class ReleaseError(Exception):
    pass


def run(*args, capture=False):
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=capture)
    if result.returncode:
        detail = result.stderr.strip() if capture else ""
        raise ReleaseError(f"{' '.join(args)} failed ({result.returncode}) {detail}")
    return result.stdout.strip() if capture else None


def git(*args):
    return run("git", *args, capture=True)


def require(condition, message):
    if not condition:
        raise ReleaseError(message)


def tracked_text(path, revision=None):
    if revision is None:
        return (ROOT / path).read_text()
    # Preserve the final newline: git() intentionally strips command output.
    result = subprocess.run(
        ["git", "show", f"{revision}:{path}"], cwd=ROOT,
        text=True, capture_output=True, check=True,
    )
    return result.stdout


def package_version(revision=None):
    package = tomllib.loads(tracked_text("Cargo.toml", revision))["package"]
    require(package["name"] == "ic-memory", "expected the ic-memory package")
    version = package["version"]
    require(re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", version),
            "release versions must be stable major.minor.patch numbers")
    return version


def next_version(current, kind):
    major, minor, patch = map(int, current.split("."))
    if kind == "patch":
        return f"{major}.{minor}.{patch + 1}"
    return f"{major}.{minor + 1}.0"


def ensure_clean():
    require(not git("status", "--porcelain", "--untracked-files=all"),
            "commit source, changelog, and other changes before releasing; working tree/index must be clean")


def check_changelog(version, revision=None):
    changelog = tracked_text("CHANGELOG.md", revision)
    headings = re.findall(r"^## (.+)$", changelog, re.MULTILINE)
    require(headings and headings[0] == version and headings.count(version) == 1,
            f"CHANGELOG.md must start with exactly one '## {version}' release entry")
    entry = changelog.split(f"## {version}\n", 1)[1].split("\n## ", 1)[0]
    require(entry.strip(), f"CHANGELOG.md entry for {version} is empty")


def version_surfaces(source, version):
    previous = package_version(source)
    manifest = tracked_text("Cargo.toml", source)
    section = re.search(r"(?ms)^\[package\][^\n]*\n.*?(?=^\[|\Z)", manifest)
    require(section is not None, "missing [package] section")
    replaced, count = re.subn(
        rf'(?m)^(version\s*=\s*)"{re.escape(previous)}"',
        rf'\g<1>"{version}"', section[0],
    )
    require(count == 1, "expected one package.version assignment")
    manifest = manifest[:section.start()] + replaced + manifest[section.end():]
    readme, count = re.subn(
        rf'(?m)^ic-memory = "{re.escape(previous)}"$',
        f'ic-memory = "{version}"', tracked_text("README.md", source),
    )
    require(count == 1, f'README.md must contain one ic-memory = "{previous}" example')
    return {"Cargo.toml": manifest, "README.md": readme}


def local_tag(version):
    result = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", f"refs/tags/v{version}"],
        cwd=ROOT, text=True, capture_output=True,
    )
    if result.returncode == 1:
        return None
    require(result.returncode == 0, "cannot inspect local release tag")
    return result.stdout.strip()


def remote_ready(version, before_version=False):
    branch = git("symbolic-ref", "--quiet", "--short", "HEAD")
    remote_ref = f"refs/remotes/origin/{branch}"
    # Refresh the source branch without importing tags or changing the worktree.
    git("fetch", "--quiet", "--no-tags", "origin", f"+refs/heads/{branch}:{remote_ref}")
    git("merge-base", "--is-ancestor", remote_ref, "HEAD")
    tag_ref = f"refs/tags/v{version}"
    refs = git("ls-remote", "--refs", "origin", tag_ref)
    if refs:
        lines = [line.split() for line in refs.splitlines()]
        require(len(lines) == 1 and lines[0][1] == tag_ref, "unexpected remote tag response")
        require(not before_version and lines[0][0] == local_tag(version),
                f"remote tag v{version} already exists or conflicts")
    return branch


def check_surfaces(source, version, revision=None):
    expected = version_surfaces(source, version)
    check_changelog(version, revision)
    for path, text in expected.items():
        require(tracked_text(path, revision) == text, f"{path} differs from the validated version-only edit")
    args = ["diff", "--name-only", source]
    if revision:
        args.append(revision)
    args.append("--")
    changed = set(git(*args).splitlines())
    require(changed == set(SURFACES), "release candidate contains changes beyond the version surfaces")
    require(not git("ls-files", "--others", "--exclude-standard"), "release candidate has untracked files")
    return expected


def prepare(kind):
    ensure_clean()
    source = git("rev-parse", "HEAD")
    version = next_version(package_version(), kind)
    check_changelog(version)
    expected = version_surfaces(source, version)
    require(local_tag(version) is None, f"local tag v{version} already exists")
    remote_ready(version, before_version=True)
    run("make", "--no-print-directory", "validate")
    ensure_clean()
    require(git("rev-parse", "HEAD") == source, "source HEAD changed during validation")
    # Refresh after validation, immediately before changing version files.
    remote_ready(version, before_version=True)
    ensure_clean()
    require(git("rev-parse", "HEAD") == source, "source HEAD changed before version edits")
    backups = {path: (ROOT / path).read_bytes() if (ROOT / path).exists() else None
               for path in (*SURFACES, "Cargo.lock")}
    receipt = ROOT / RECEIPT
    receipt.unlink(missing_ok=True)
    try:
        for path, text in expected.items():
            (ROOT / path).write_text(text)
        run("cargo", "update", "--workspace", "--offline")
        # Verify the package with its final version as well as the source gate.
        run("cargo", "package", "--locked", "--allow-dirty")
        require(git("rev-parse", "HEAD") == source, "source HEAD changed during preparation")
        check_surfaces(source, version)
        receipt.parent.mkdir(parents=True, exist_ok=True)
        receipt.write_text(json.dumps({"source": source, "version": version}) + "\n")
    except BaseException:
        for path, contents in backups.items():
            if contents is None:
                (ROOT / path).unlink(missing_ok=True)
            else:
                (ROOT / path).write_bytes(contents)
        receipt.unlink(missing_ok=True)
        raise
    print(f"Prepared {version}. Review git diff, then make release-stage release-commit release-push.")


def prepared():
    try:
        receipt = json.loads((ROOT / RECEIPT).read_text())
        source, version = receipt["source"], receipt["version"]
    except (OSError, ValueError, KeyError) as error:
        raise ReleaseError("no prepared release; run make patch or make minor") from error
    require(git("rev-parse", "HEAD") == source, "prepared validation is for another source HEAD")
    require(version in (next_version(package_version(source), "patch"),
                        next_version(package_version(source), "minor")),
            "prepared version is not the next patch or minor")
    check_surfaces(source, version)
    return source, version


def stage():
    prepared()
    run("git", "add", "--", *SURFACES)


def release_commit():
    ensure_clean()
    version = package_version()
    head = git("rev-parse", "HEAD")
    source = git("rev-parse", "HEAD^")
    message = git("log", "-1", "--format=%B")
    require(message == f"Release {version}\n\nValidated-source: {source}",
            f"HEAD is not the validated Release {version} commit")
    check_surfaces(source, version, "HEAD")
    require(version in (next_version(package_version(source), "patch"),
                        next_version(package_version(source), "minor")),
            "release version is not the next patch or minor")
    return version, head


def check_tag(version, head):
    require(local_tag(version) is not None, f"annotated tag v{version} is missing")
    require(git("cat-file", "-t", f"refs/tags/v{version}") == "tag", "release tag must be annotated")
    require(git("rev-parse", f"refs/tags/v{version}^{{commit}}") == head, "release tag does not identify HEAD")


def commit():
    # A failed tag operation can be retried without creating a second commit.
    if git("log", "-1", "--format=%s") == f"Release {package_version()}":
        version, head = release_commit()
    else:
        source, version = prepared()
        require(not git("diff", "--name-only"), "release surfaces have unstaged changes; run make release-stage")
        require(set(git("diff", "--cached", "--name-only").splitlines()) == set(SURFACES),
                "stage exactly the release surfaces with make release-stage")
        require(local_tag(version) is None, f"local tag v{version} already exists")
        run("git", "commit", "-m", f"Release {version}\n\nValidated-source: {source}")
        version, head = release_commit()
    if local_tag(version) is None:
        run("git", "tag", "-a", f"v{version}", "-m", f"Release {version}")
    check_tag(version, head)


def push():
    version, head = release_commit()
    check_tag(version, head)
    branch = remote_ready(version)
    # Network inspection may take time: recheck local state before pushing.
    require(release_commit() == (version, head), "release changed during remote inspection")
    check_tag(version, head)
    run("git", "push", "--no-follow-tags", "--atomic", "origin",
        f"{head}:refs/heads/{branch}", f"refs/tags/v{version}:refs/tags/v{version}")


def publish(dry_run=False):
    version, head = release_commit()
    check_tag(version, head)
    if not (ROOT / "Cargo.lock").exists():
        run("cargo", "generate-lockfile")
    require(release_commit() == (version, head), "release changed during lockfile resolution")
    args = ["cargo", "publish", "--locked", "--registry", "crates-io"]
    if dry_run:
        args.append("--dry-run")
    run(*args)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["version", "msrv", "ensure-clean", "patch", "minor",
                                           "stage", "commit", "push", "publish"])
    parser.add_argument("--dry-run", action="store_true", help="verify publish without uploading")
    args = parser.parse_args()
    require(not args.dry_run or args.command == "publish", "--dry-run applies only to publish")
    if args.command == "version":
        print(package_version())
    elif args.command == "msrv":
        print(tomllib.loads(tracked_text("Cargo.toml"))["package"]["rust-version"])
    elif args.command in ("patch", "minor"):
        prepare(args.command)
    elif args.command == "publish":
        dry_run = os.environ.get("PUBLISH_DRY_RUN", "0")
        require(dry_run in ("0", "1"), "PUBLISH_DRY_RUN must be 0 or 1")
        publish(args.dry_run or dry_run == "1")
    else:
        {"ensure-clean": ensure_clean, "stage": stage, "commit": commit, "push": push}[args.command]()


if __name__ == "__main__":
    try:
        main()
    except (ReleaseError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"release: {error}", file=sys.stderr)
        sys.exit(1)
