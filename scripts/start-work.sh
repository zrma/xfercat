#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/start-work.sh --work-id <id>

Creates docs/todo-<id>/spec.md and open-questions.md if they do not exist.
USAGE
}

work_id=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --work-id)
      work_id="${2:-}"
      shift 2
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

if [[ ! "$work_id" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
  echo "invalid or missing --work-id: $work_id" >&2
  exit 1
fi

todo_dir="docs/todo-$work_id"
mkdir -p "$todo_dir"

if [[ ! -f "$todo_dir/spec.md" ]]; then
  cat >"$todo_dir/spec.md" <<EOF
# Spec: $work_id

Status: planned

## Goal

- TODO

## Scope

- TODO

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | todo | \`scripts/check.sh\` | TODO |

## Required Evidence

- TODO

## Publication Impact

- tracked artifact와 local-only evidence 경계를 적는다.
- remote write가 없으면 명시한다.

## Out Of Scope

- TODO

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
EOF
fi

if [[ ! -f "$todo_dir/open-questions.md" ]]; then
  cat >"$todo_dir/open-questions.md" <<'EOF'
# Open Questions

현재 미결 항목 없음.
EOF
fi

echo "initialized $todo_dir"
