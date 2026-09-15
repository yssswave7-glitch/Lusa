# Lusa compatibility contract

Lusa is an offline Luau/Roblox compatibility runtime. Its goal is to make Roblox-oriented Luau tooling predictable outside Studio while preserving real Luau parser/runtime behavior.

## Compatibility tiers

**Language/runtime:** Lusa uses Lune's Luau VM and preserves direct file execution, syntax/compile diagnostics, native Luau libraries, `getfenv`/`setfenv`, `loadstring` where provided by the VM, `bit32`, `buffer`, coroutines, and the Lune task scheduler.

**Roblox datatypes and object model:** Lusa exposes the Roblox datatypes and `Instance` model implemented by Lune. A generated API registry supplements Lune's bundled reflection database with current class and service names from a Roblox API dump. Registered custom classes inherit from their declared superclass when possible.

**Offline shims:** Lusa implements selected engine-facing behavior when a deterministic standalone approximation is useful, such as `DataModel:IsLoaded()`, an offline wall-clock implementation of `Workspace:GetServerTimeNow()`, Roblox's protected Instance metatable sentinel, and stable `Players.PlayerAdded` / `Players.PlayerRemoving` signal handles. Offline player signals expose connection behavior but do not invent live players or fire without a live engine.

**Live-engine behavior:** replication, physics, rendering, security contexts, network ownership, live service state, authoritative server time, and other behaviors that require the Roblox engine are not simulated as if they were real.

**Executor-only behavior:** exploit-executor identity, hook state, fake C/native-closure provenance, stack-frame concealment, and similar non-Roblox surfaces are intentionally not fabricated.

## Tooling mode

`--raw` (or `LUSA_RAW=1`) skips Lusa's Roblox registry/bootstrap while retaining the Luau/Lune runtime. This is the recommended mode for a deobfuscator or trace harness that already constructs a target environment, because it avoids contaminating bootstrap observations with a second `game`/`workspace` model.

`--isolated` (or `LUSA_ISOLATED=1`) additionally denies filesystem/network/process APIs and file module loads, and removes inherited environment values. It retains task, datetime, serde, regex, and offline Roblox libraries for analysis harnesses. Luau JIT is enabled by default; `--no-jit` or `LUSA_LUAU_JIT=0` disables it.

Lusa also accepts common Luau CLI optimization/debug flags for subprocess compatibility and supports stdin for generated scripts. See [docs/TOOLING.md](docs/TOOLING.md).

## API registry limits

The generated registry improves recognition/creation of class and service names that may be newer than Lune's bundled reflection database. It does **not** automatically synthesize every method, event, property, security rule, or live return value from the API dump. Missing engine behavior should remain explicit rather than silently returning invented data.

## Security boundary

Official Lusa binaries expose Lune's standard libraries, including filesystem, network, process, and regex modules. This is intentional for tooling compatibility and is reported by `lusa --capabilities` as `"host_io": true`.

Isolated mode is defense in depth, not a complete sandbox. Lusa is **not** an OS security boundary. Execute untrusted code inside a separately hardened OS/container sandbox with filesystem, network, process, CPU, memory, and wall-clock controls enforced outside the Lusa process.
