# Lusa compatibility contract

Lusa uses Lune's embedded Luau runtime and Roblox datatype/Instance model, then
adds an offline Roblox-style bootstrap. The goal is to make Roblox-oriented
Luau tooling behave predictably outside Studio without claiming that a normal
process is a real Roblox client or server.

## Supported target

| Area | Status | Notes |
| --- | --- | --- |
| Luau parser / VM | Native Lune/Luau | User files are executed through `Runtime::run_file` |
| Syntax diagnostics | Native embedded Luau | File path and line information are preserved |
| `game` / `workspace` | Supported offline model | Backed by Lune's `DataModel` / `Workspace` Instances |
| Roblox datatypes | Supported where Lune exposes them | `Instance`, `Enum`, `Vector3`, `CFrame`, etc. |
| `task` | Supported | Uses Lune's Luau scheduler/task library |
| `GetServerTimeNow` | Offline approximation | Wall-clock seconds; not server synchronized |
| `SlimAnimationReplicationService` | Registered stub | Instance/service identity only |
| `CoreGui` ancestry / `GetDebugId` | Covered by tests | Uses Lune's Instance model |
| Live replication | Not available | Requires Roblox engine/network state |
| Physics / rendering | Not available | Requires Roblox engine |
| Security context / capabilities | Not equivalent | Standalone process is not Roblox |
| Executor APIs | Intentionally absent | Not Roblox APIs |

## About "1:1"

A standalone runner can match Luau language behavior and selected engine APIs,
but it cannot honestly guarantee 1:1 live-Roblox behavior for APIs whose values
come from the engine, network, scheduler, security context, or live place state.

Lusa therefore treats parity as a testable matrix instead of spoofing signals.
When a missing legitimate Roblox API matters, implement it in
`src/roblox_bootstrap.luau` (or upstream Lune) and add a regression test under
`tests/compat`.

## Executor / anti-analysis boundary

`identifyexecutor`, `getgenv`, `isfunctionhooked`, `islclosure`, and similar
functions are supplied by third-party executors, not by Roblox. Lusa does not
forge them, does not falsify `[C]` provenance for Lua callbacks, and does not
hide stack frames to defeat environment checks.

The rest of a compatibility test can still exercise ordinary Luau semantics
such as `pcall`, `rawget`, `debug.info` on actual VM builtins, frozen standard
libraries, `bit32`, and string functions. Those are covered separately in
`tests/compat/native_semantics.luau`.
