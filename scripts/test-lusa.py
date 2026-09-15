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
    (
        "tests/compat/protected_instance_pattern.luau",
        "LUSA_PROTECTED_INSTANCE_PATTERN_PASS",
    ),
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

# Official binaries expose Lune's standard host libraries for tooling
# compatibility. Lusa documents this explicitly instead of presenting it as a
# security boundary.
host_output = run_args(
    ["--raw", "-"],
    True,
    input_text=(
        'local ok, fs = pcall(require, "@lune/fs")\n'
        'assert(ok and fs ~= nil, "@lune/fs should be available in release builds")\n'
        'print("LUSA_HOST_IO_PASS")\n'
    ),
)
if "LUSA_HOST_IO_PASS" not in host_output:
    raise SystemExit("host library availability check failed")

# Isolated mode is a defense-in-depth profile for untrusted analysis. It keeps
# pure/runtime libraries needed by harnesses but denies filesystem, network,
# process, and file-module access.
isolated_output = run_args(
    ["--raw", "--isolated", "-"],
    True,
    input_text=(
        'local taskOk = pcall(require, "@lune/task")\n'
        'local fileOk = pcall(require, "./local-module")\n'
        'assert(taskOk, "isolated mode should retain @lune/task")\n'
        'assert(not fileOk, "isolated mode unexpectedly allowed file require")\n'
        'for _, name in {"fs", "net", "process", "stdio"} do\n'
        '    local ok = pcall(require, "@lune/" .. name)\n'
        '    assert(not ok, "isolated mode exposed @lune/" .. name)\n'
        'end\n'
        'print("LUSA_ISOLATED_PASS")\n'
    ),
)
if "LUSA_ISOLATED_PASS" not in isolated_output:
    raise SystemExit("--isolated check failed")

# Roblox setup runs before the guard is installed, while user code still sees
# the restricted require function.
roblox_isolated_output = run_args(
    ["--isolated", "-"],
    True,
    input_text=(
        'assert(game ~= nil and workspace ~= nil, "Roblox bootstrap missing")\n'
        'assert(not pcall(require, "@lune/net"), "isolated Roblox exposed net")\n'
        'print("LUSA_ROBLOX_ISOLATED_PASS")\n'
    ),
)
if "LUSA_ROBLOX_ISOLATED_PASS" not in roblox_isolated_output:
    raise SystemExit("Roblox --isolated check failed")

potassium_output = run_args(
    ["--raw", "-"],
    True,
    input_text=(
        "local name, version = identifyexecutor()\n"
        'assert(name == "potassium", "unexpected executor name")\n'
        'assert(version == "v2.4.8", "unexpected executor version")\n'
        "local aliasName, aliasVersion = getexecutorname()\n"
        'assert(aliasName == name and aliasVersion == version, "identity alias mismatch")\n'
        'assert(_LUSA_EXECUTOR_PROFILE == "potassium-emulated", "missing emulation marker")\n'
        'assert(debug.info(identifyexecutor, "s") == "[C]", "identity callback is not native")\n'
        'assert(getgenv() == getgenv(), "executor environment is unstable")\n'
        'assert(getrenv() == getrenv() and getrenv() ~= getgenv(), "environment separation failed")\n'
        'assert(getrenv().print == print, "runtime global lookup failed")\n'
        'assert(iscclosure(identifyexecutor), "identity callback is not a C closure")\n'
        'assert(islclosure(function() end), "Lua closure classification failed")\n'
        'assert(isexecutorclosure(function() end) and not isexecutorclosure(print), "executor closure classification failed")\n'
        'assert(checkcaller(), "caller classification failed")\n'
        'assert(isfunctionhooked == nil, "profile spoofed live hook state")\n'
        'print("LUSA_POTASSIUM_IDENTITY_PASS")\n'
    ),
)
if "LUSA_POTASSIUM_IDENTITY_PASS" not in potassium_output:
    raise SystemExit("default Potassium identity profile check failed")

executor_disabled_output = run_args(
    ["--raw", "--executor-profile=none", "-"],
    True,
    input_text=(
        'assert(identifyexecutor == nil, "executor identity was not disabled")\n'
        'assert(getgenv == nil and checkcaller == nil, "executor profile leaked APIs")\n'
        'print("LUSA_EXECUTOR_DISABLED_PASS")\n'
    ),
)
if "LUSA_EXECUTOR_DISABLED_PASS" not in executor_disabled_output:
    raise SystemExit("--executor-profile=none check failed")

for jit_flag in ("--jit", "--no-jit"):
    jit_output = run_args(
        ["--raw", jit_flag, "-"],
        True,
        input_text='print("LUSA_JIT_CONTROL_PASS")\n',
    )
    if "LUSA_JIT_CONTROL_PASS" not in jit_output:
        raise SystemExit(f"{jit_flag} check failed")

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
    "isolated_mode": True,
    "executor_identity_emulation": True,
    "executor_identity_default": "potassium",
    "jit_default": True,
    "roblox_bootstrap": True,
    "api_registry": True,
    "host_io": True,
    "sandbox": "external_required",
}
for key, expected in required.items():
    if capabilities.get(key) != expected:
        raise SystemExit(f"capability {key!r} expected {expected!r}, got {capabilities.get(key)!r}")

host_modules = set(capabilities.get("host_io_modules", []))
if not {"fs", "net", "process", "regex"}.issubset(host_modules):
    raise SystemExit(f"--capabilities host_io_modules incomplete: {sorted(host_modules)!r}")

if "-O2" not in capabilities.get("luau_cli_flags", []):
    raise SystemExit("--capabilities is missing -O2 compatibility")

if "potassium" not in capabilities.get("executor_identity_profiles", []):
    raise SystemExit("--capabilities is missing the Potassium identity profile")

print("ALL_LUSA_TESTS_PASS")
