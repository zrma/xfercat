#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REQUIRED_FILES = (
    "README.md",
    "AGENTS.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "docs/REPO_MANIFEST.yaml",
    "docs/agent-harness.md",
    "docs/HANDOFF.md",
    "docs/PRODUCT.md",
    "docs/ARCHITECTURE.md",
    "docs/PUBLICATION.md",
    "docs/status.md",
    "docs/roadmap.md",
    "docs/completed-milestones.md",
    "docs/decisions/0001-poc-runtime.md",
    "docs/decisions/0002-openssh-profile-import.md",
    "src/app.rs",
    "src/domain.rs",
    "src/executor.rs",
    "src/lib.rs",
    "src/main.rs",
    "src/openssh.rs",
    "src/transport.rs",
    "src/ui.rs",
    "tests/interaction.rs",
    "scripts/check.sh",
    "scripts/check-agent-harness-interface.sh",
    "scripts/check-publication-boundary.py",
    "scripts/start-work.sh",
    "scripts/finalize-change.sh",
)
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def fail(message: str) -> None:
    print(f"repository contract check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


for relative in REQUIRED_FILES:
    path = ROOT / relative
    if not path.is_file() or not path.read_text(encoding="utf-8").strip():
        fail(f"missing or empty {relative}")

manifest = (ROOT / "docs/REPO_MANIFEST.yaml").read_text(encoding="utf-8")
for exact in (
    "schema_version: 1",
    "name: xfercat",
    "publication_class: public",
    "remote_status: configured",
    "remote_visibility: public",
    "license_status: undecided",
    "local: jj",
    "push_requires_explicit_permission: true",
    "status: selected-for-poc",
    "core: Rust 2024",
    "interface: Ratatui TUI validation shell",
    "executable: xfercat",
):
    if exact not in manifest:
        fail(f"manifest is missing {exact!r}")

active_match = re.search(r"^active_work:\s+(.+)$", manifest, re.MULTILINE)
if active_match is None:
    fail("manifest does not declare active_work")
active_work = active_match.group(1).strip()
if active_work != "none":
    active_path = ROOT / active_work
    if not active_path.is_file():
        fail(f"active work does not exist: {active_work}")
    questions = active_path.parent / "open-questions.md"
    if not questions.is_file():
        fail(f"active work is missing {questions.relative_to(ROOT)}")

for markdown in sorted(ROOT.rglob("*.md")):
    text = markdown.read_text(encoding="utf-8")
    for raw_target in LINK.findall(text):
        target = raw_target.strip().split("#", 1)[0]
        if not target or "://" in target or target.startswith(("mailto:", "<")):
            continue
        resolved = (markdown.parent / target).resolve()
        try:
            resolved.relative_to(ROOT)
        except ValueError:
            fail(f"{markdown.relative_to(ROOT)} links outside the repository: {target}")
        if not resolved.exists():
            fail(f"{markdown.relative_to(ROOT)} has a broken link: {target}")

readme = (ROOT / "README.md").read_text(encoding="utf-8")
if "아직 구현되지 않았다" not in readme:
    fail("README must state that the functional client is not implemented")

print("repository contract is valid")
