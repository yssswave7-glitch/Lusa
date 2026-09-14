#!/usr/bin/env python3
"""Black-box compatibility suite for a built Lusa binary."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BIN = ROOT / "target" / "release" / ("lusa.exe" if os.name == "nt" else "lusa")
BIN = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_BIN

if not BIN.exists():
    raise SystemExit(f"missing Lusa binary: {BIN}")


def run_args(
    args: list[str],
    expect_success: bool,
    *,
    input_text: str | None = None,
    env: dict[str, str] | None = None,
) -> str:
    proc = subprocess.run(
        [str(BIN), *args],
        text=True,
        input=input_text,
        capture_output=True,
        timeout=30,
        check=False,
        env=env,
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    ok = (proc.returncode == 0) if expect_success else (proc.returncode != 0)
    label = " ".join(args)
    print(f"[{'PASS' if ok else 'FAIL'}] {label} (exit={proc.returncode})")
    if not ok:
        print(output)
        raise SystemExit(1)
    return output


def run_file(rel: str, expect_success: bool, *extra: str) -> str:
    path = ROOT / rel
    return run_args([*extra, str(path)], expect_success)


checks = [
    ("tests/compat/roblox_core.luau", "LUSA_ROBLOX_CORE_PASS"),
    ("tests/compat/native_semantics.luau", "LUSA_NATIVE_SEMANTICS_PASS"),
    ("tests/compat/tooling_runtime.luau", "LUSA_TOOLING_RUNTIME_PASS"),
    ("tests/compat/executor_boundary.luau", "LUSA_EXECUTOR_BOUNDARY_PASS"),
    ("tests/compat/api_registry.luau", "LUSA_API_REGISTRY_PASS"),
    ("tests/syntax/valid.luau", "LUSA_VALID_SYNTAX_PASS"),
]

for file_name, marker in checks:
    output = run_file(file_name, True)
    if marker not in output:
        raise SystemExit(f"{file_name} did not emit {marker}")

# Luau CLI compatibility: deobfuscators and validators commonly pass -O2.
output = run_file("tests/syntax/valid.luau", True, "-O2")
if "LUSA_VALID_SYNTAX_PASS" not in output:
    raise SystemExit("-O2 compatibility run did not execute the valid script")

for file_name in ["tests/syntax/invalid_unclosed.luau", "tests/syntax/invalid_type.luau"]:
    output = run_file(file_name, False, "-O2")
    lower = output.lower()
    if not any(token in lower for token in ("syntax", "expected", "parse", "error")):
        raise SystemExit(f"{file_name} failed without a recognizable Luau diagnostic")
    if Path(file_name).name.lower() not in lower:
        print(f"[WARN] diagnostic did not contain basename {Path(file_name).name!r}")

# stdin mode is useful for generated harnesses and pipelines.
stdin_output = run_args(
    ["--raw", "-"],
    True,
    input_text='print("LUSA_STDIN_PASS")\n',
)
if "LUSA_STDIN_PASS" not in stdin_output:
    raise SystemExit("stdin execution check failed")

# Raw mode should not install Roblox globals.
raw_output = run_args(
    ["--raw", "-"],
    True,
    input_text=(
        'assert(getfenv(0).game == nil, "raw mode unexpectedly installed game")\n'
        'print("LUSA_RAW_PASS")\n'
    ),
)
if "LUSA_RAW_PASS" not in raw_output:
    raise SystemExit("--raw check failed")

# Environment-only raw mode lets an existing subprocess wrapper switch Lusa
# behavior without changing the child command it constructs.
child_env = dict(os.environ)
child_env["LUSA_RAW"] = "1"
env_raw_output = run_args(
    ["-"],
    True,
    input_text=(
        'assert(getfenv(0).game == nil, "LUSA_RAW did not disable bootstrap")\n'
        'print("LUSA_ENV_RAW_PASS")\n'
    ),
    env=child_env,
)
if "LUSA_ENV_RAW_PASS" not in env_raw_output:
    raise SystemExit("LUSA_RAW environment compatibility check failed")

# Released binaries intentionally omit direct host fs/net/process modules.
safe_output = run_args(
    ["--raw", "-"],
    True,
    input_text=(
        'local ok = pcall(require, "@lune/fs")\n'
        'assert(not ok, "@lune/fs should not be available in the default build")\n'
        'print("LUSA_SAFE_MODULES_PASS")\n'
    ),
)
if "LUSA_SAFE_MODULES_PASS" not in safe_output:
    raise SystemExit("safe module boundary check failed")

version = subprocess.run(
    [str(BIN), "--version"],
    text=True,
    capture_output=True,
    timeout=10,
    check=False,
)
if version.returncode != 0 or "lusa" not in version.stdout.lower():
    raise SystemExit("--version check failed")

caps = subprocess.run(
    [str(BIN), "--capabilities"],
    text=True,
    capture_output=True,
    timeout=10,
    check=False,
)
if caps.returncode != 0:
    raise SystemExit("--capabilities check failed")
try:
    capabilities = json.loads(caps.stdout)
except json.JSONDecodeError as exc:
    raise SystemExit(f"--capabilities did not return JSON: {exc}") from exc

required = {
    "direct_file": True,
    "stdin": True,
    "raw_mode": True,
    "roblox_bootstrap": True,
    "api_registry": True,
    "host_io": False,
}
for key, expected in required.items():
    if capabilities.get(key) != expected:
        raise SystemExit(f"capability {key!r} expected {expected!r}, got {capabilities.get(key)!r}")

if "-O2" not in capabilities.get("luau_cli_flags", []):
    raise SystemExit("--capabilities is missing -O2 compatibility")

print("ALL_LUSA_TESTS_PASS")
