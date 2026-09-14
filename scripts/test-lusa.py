#!/usr/bin/env python3
"""Small black-box suite for a built Lusa binary."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BIN = ROOT / "target" / "release" / ("lusa.exe" if os.name == "nt" else "lusa")
BIN = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_BIN

if not BIN.exists():
    raise SystemExit(f"missing Lusa binary: {BIN}")


def run(rel: str, expect_success: bool) -> str:
    path = ROOT / rel
    proc = subprocess.run(
        [str(BIN), str(path)],
        text=True,
        capture_output=True,
        timeout=30,
        check=False,
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    ok = (proc.returncode == 0) if expect_success else (proc.returncode != 0)
    print(f"[{'PASS' if ok else 'FAIL'}] {rel} (exit={proc.returncode})")
    if not ok:
        print(output)
        raise SystemExit(1)
    return output


checks = [
    ("tests/compat/roblox_core.luau", "LUSA_ROBLOX_CORE_PASS"),
    ("tests/compat/native_semantics.luau", "LUSA_NATIVE_SEMANTICS_PASS"),
    ("tests/compat/executor_boundary.luau", "LUSA_EXECUTOR_BOUNDARY_PASS"),
    ("tests/syntax/valid.luau", "LUSA_VALID_SYNTAX_PASS"),
]

for file_name, marker in checks:
    output = run(file_name, True)
    if marker not in output:
        raise SystemExit(f"{file_name} did not emit {marker}")

for file_name in ["tests/syntax/invalid_unclosed.luau", "tests/syntax/invalid_type.luau"]:
    output = run(file_name, False)
    lower = output.lower()
    if not any(token in lower for token in ("syntax", "expected", "parse", "error")):
        raise SystemExit(f"{file_name} failed without a recognizable Luau diagnostic")
    if Path(file_name).name.lower() not in lower:
        print(f"[WARN] diagnostic did not contain basename {Path(file_name).name!r}")

version = subprocess.run(
    [str(BIN), "--version"],
    text=True,
    capture_output=True,
    timeout=10,
    check=False,
)
if version.returncode != 0 or "lusa" not in version.stdout.lower():
    raise SystemExit("--version check failed")

print("ALL_LUSA_TESTS_PASS")
