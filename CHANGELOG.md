# Changelog

## v0.3.1

Bounded Potassium compatibility.

- Enable the Potassium compatibility profile by default; add `--executor-profile=none` / `LUSA_EXECUTOR_PROFILE=none` as an explicit opt-out.
- Return `potassium` and `v2.4.8` from native `identifyexecutor` and `getexecutorname` callbacks.
- Add stable `getgenv` / `getrenv`, closure classifiers, `checkcaller`, `isourthread`, and an offline `newcclosure` wrapper.
- Keep live hook state, injection, network/filesystem/input APIs, RakNet, hidden properties, and stack concealment outside the emulated profile.

## v0.3.0

Protection and performance update.

- Enable Luau JIT by default, matching Lune's CLI, with explicit CLI/environment controls to disable it.
- Optimize the native release binary for execution speed instead of minimum binary size.
- Add an opt-in isolated analysis profile that strips inherited environment values and denies filesystem/network/process APIs and file module loads while preserving pure/runtime libraries.
- Compile the generated Roblox API registry, bootstrap, and version setup as one cold runtime chunk to reduce scheduler and standard-library injection overhead.
- Add isolated-mode and JIT-control regression coverage.
- Integrate the updated Luraph ENV LOG harness with `--raw --isolated` execution and Lusa-first discovery.
- Build Linux x64 in a Debian 11 container so its glibc baseline remains compatible with Debian 11 and newer.

## v0.2.1

Roblox Instance fidelity patch.

- Match Roblox's protected Instance metatable result: `getmetatable(instance)` now returns `"The metatable is locked"` instead of mlua's generic `false` sentinel.
- Keep the compatibility callback native (`debug.info(getmetatable, "s") == "[C]`) and preserve non-Instance metatable behavior.
- Add stable offline `Players.PlayerAdded` and `Players.PlayerRemoving` signal surfaces with `Connect`, `Once`, and disconnectable connection handles.
- Add regression coverage for protected Instance metatables, native provenance, non-Instance metatable locks, and Players signal identity/connections.
- Commit the Cargo lockfile so release dependency resolution is reproducible.

## v0.2.0

Tooling-focused runtime release.

- Accept Luau CLI compatibility flags `-O0/-O1/-O2` and `-g0/-g1/-g2`.
- Add stdin execution with `lusa -`.
- Add `--raw` and `LUSA_RAW=1` for trace/deobfuscation harnesses that provide their own environment.
- Add `--capabilities` JSON for automatic runtime discovery.
- Add a generated Roblox API registry covering 830 Instance-descendant classes, including 296 services, from the supplied current API dump.
- Add a registry generator for future Roblox API dump updates.
- Explicitly report Lune host-I/O availability in `--capabilities`; official binaries prioritize tooling compatibility and are not a security sandbox.
- Harden Roblox bootstrap method registration so already-known upstream methods do not abort startup.
- Expand black-box CI coverage for tooling invocation, stdin/raw mode, registry compatibility, native Luau behavior, and host-library availability.
- Add dedicated tooling/deobfuscator integration documentation.

## v0.1.0

Initial Lusa release.

- Roblox-focused Luau compatibility runner built on Lune 0.10.5.
- Direct `lusa <script.luau> [args...]` CLI.
- Roblox-style `game`, `workspace`, `Instance`, `Enum`, datatype, `task`, and `DateTime` globals.
- Offline compatibility shims including `Workspace:GetServerTimeNow()` and `DataModel:IsLoaded()`.
- Luau syntax/runtime compatibility tests.
- Linux x64 and Windows x64 release builds.
- Purple Lusa branding.
