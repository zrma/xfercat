#!/bin/sh
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

scripts/check-agent-harness-interface.sh
scripts/check-repository-contract.py
scripts/check-publication-boundary.py --self-test
scripts/check-publication-boundary.py

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build

for snapshot in connections openssh openssh-empty profile-add profile-edit workspace rename review live-review results; do
  cargo run --quiet -- --snapshot "$snapshot" >/dev/null
done

sh -n scripts/check.sh
sh -n scripts/check-agent-harness-interface.sh
bash -n scripts/start-work.sh
bash -n scripts/finalize-change.sh

python3 - <<'PY'
import ast
from pathlib import Path

for path in sorted(Path("scripts").glob("*.py")):
    ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
print("python syntax is valid")
PY

printf 'xfercat repository checks passed\n'
