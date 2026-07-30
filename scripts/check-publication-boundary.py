#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PATTERNS = (
    ("macOS user path", re.compile(r"/Users/[A-Za-z0-9._-]+/")),
    ("Linux user path", re.compile(r"/home/[A-Za-z0-9._-]+/")),
    ("Windows user path", re.compile(r"[A-Za-z]:\\\\Users\\\\[^\\\\\s]+\\\\")),
    ("file URI", re.compile(r"file:" + r"//")),
    ("private key material", re.compile(r"-----BEGIN (?:OPENSSH|RSA|EC|DSA) PRIVATE KEY-----")),
    ("OpenAI secret", re.compile(r"\bsk-(?:proj-)?[A-Za-z0-9_-]{16,}\b")),
    ("GitHub token", re.compile(r"\bgh[opusr]_[A-Za-z0-9]{20,}\b")),
    (
        "session transcript",
        re.compile(r"(?:session-[A-Za-z0-9_-]+\.jsonl|\.codex/" + r"sessions/)"),
    ),
    ("IPv4 address", re.compile(r"(?<![\w.-])(?:\d{1,3}\.){3}\d{1,3}(?![\w.-])")),
)


def tracked_files() -> list[Path]:
    result = subprocess.run(
        ["jj", "file", "list"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    paths: list[Path] = []
    for line in result.stdout.splitlines():
        path = ROOT / line
        if path.is_file():
            paths.append(path)
    return paths


def change_descriptions() -> str:
    result = subprocess.run(
        ["jj", "log", "-r", "::@", "--no-graph", "-T", 'description ++ "\n"'],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def scan_text(label: str, text: str) -> list[str]:
    findings: list[str] = []
    for name, pattern in PATTERNS:
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            findings.append(f"{label}:{line}: {name}")
    return findings


def self_test() -> None:
    cases = {
        "macOS user path": "/" + "Users/example/work/project",
        "private key material": "-----BEGIN " + "OPENSSH PRIVATE KEY-----",
        "OpenAI secret": "sk-" + "proj-0123456789abcdefghijklmnop",
        "GitHub token": "ghp" + "_0123456789abcdefghijklmnopqrstuv",
        "IPv4 address": ".".join(("192", "0", "2", "10")),
    }
    for expected, sample in cases.items():
        names = {finding.rsplit(": ", 1)[-1] for finding in scan_text("self-test", sample)}
        if expected not in names:
            raise SystemExit(f"publication boundary self-test failed: {expected}")
    if scan_text("self-test", "<repo-root> <private-host> <internal-ip>"):
        raise SystemExit("publication boundary self-test rejected approved placeholders")
    print("publication boundary self-test passed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    findings: list[str] = []
    for path in tracked_files():
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        findings.extend(scan_text(str(path.relative_to(ROOT)), text))
    findings.extend(scan_text("jj change descriptions", change_descriptions()))

    if findings:
        print("publication boundary check failed:", file=sys.stderr)
        for finding in findings:
            print(f"  {finding}", file=sys.stderr)
        raise SystemExit(1)

    print("publication boundary is clean")


if __name__ == "__main__":
    main()
