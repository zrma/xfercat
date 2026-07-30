#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/finalize-change.sh --verify-only
  scripts/finalize-change.sh --message "<type>: <summary>" [--bookmark <name>]

Runs the canonical local gate. With --message, writes a jj change description
with the configured Codex attribution. A local bookmark moves only when named.
This script never pushes.
USAGE
}

message=""
bookmark=""
verify_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --message)
      message="${2:-}"
      shift 2
      ;;
    --bookmark)
      bookmark="${2:-}"
      shift 2
      ;;
    --verify-only)
      verify_only=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$verify_only" -eq 1 ]]; then
  [[ -z "$message" && -z "$bookmark" ]] ||
    { echo "--verify-only cannot change description or bookmark" >&2; exit 2; }
  scripts/check.sh
  echo "local verification passed; no description, bookmark, or remote changed"
  jj status
  exit 0
fi

if [[ ! "$message" =~ ^(feat|fix|perf|refactor|docs|test|build|ci|chore|revert):\ .+ ]]; then
  echo "invalid --message: expected '<type>: <summary>'" >&2
  exit 1
fi

scripts/check.sh

codex_config="${CODEX_HOME:-$HOME/.codex}/config.toml"
attribution=$(sed -nE 's/^[[:space:]]*commit_attribution[[:space:]]*=[[:space:]]*"([^"]*)"[[:space:]]*$/\1/p' \
  "$codex_config" | head -n 1)
if [[ -z "$attribution" ]]; then
  echo "commit_attribution is not configured" >&2
  exit 1
fi

helper="${CODEX_HOME:-$HOME/.codex}/skills/vcs-jj/scripts/describe_with_attribution.sh"
if [[ -x "$helper" ]]; then
  "$helper" -r @ -- "$message"
else
  trailer="Co-authored-by: $attribution"
  jj describe -r @ --message "$message"$'\n\n'"$trailer"
fi

description=$(jj log -r @ --no-graph -T 'description')
DESCRIPTION="$description" TRAILER="Co-authored-by: $attribution" python3 - <<'PY'
import os

lines = [line.rstrip() for line in os.environ["DESCRIPTION"].splitlines() if line.strip()]
trailer = os.environ["TRAILER"]
if lines.count(trailer) != 1 or not lines or lines[-1] != trailer:
    raise SystemExit("attribution verification failed")
PY

if [[ -n "$bookmark" ]]; then
  jj bookmark set "$bookmark" -r @
fi

scripts/check-publication-boundary.py
echo "local change finalized; no remote was changed"
jj status
