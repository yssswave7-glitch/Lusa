# Changelog

## v0.2.0

Tooling-focused runtime release.

- Accept Luau CLI compatibility flags `-O0/-O1/-O2` and `-g0/-g1/-g2`.
- Add stdin execution with `lusa -`.
- Add `--raw` and `LUSA_RAW=1` for trace/deobfuscation harnesses that provide their own environment.
- Add `--capabilities` JSON for automatic runtime discovery.
- Add a generated Roblox API registry covering 830 Instance-descendant classes, including 296 services, from the supplied current API dump.
- Add a registry generator for future Roblox API dump updates.
- Narrow official release builds to the Lune modules needed for analysis; direct host fs/net/process/regex modules are opt-in with the `host-io` Cargo feature.
- Harden Roblox bootstrap method registration so already-known upstream methods do not abort startup.
- Expand black-box CI coverage for tooling invocation, stdin/raw mode, registry compatibility, and host-access defaults.
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
