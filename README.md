<p align="center">
  <img src="assets/logo/lusa-wordmark.svg" alt="Lusa" width="640">
</p>

# Lusa

**Lusa** is a Roblox-focused Luau compatibility runtime built on top of [Lune](https://github.com/lune-org/lune), with first-class support for tooling that repeatedly executes generated Luau: deobfuscators, trace/reconstruction harnesses, validators, build pipelines, and offline Roblox-oriented tests.

It keeps the convenient `lusa script.luau` workflow, uses Lune's real Luau VM, and can either install a Roblox-like environment or run in a clean **raw mode** when the surrounding tool provides its own mocks.

## Quick start

```bash
lusa script.luau
lusa -O2 script.luau
lusa --raw harness.luau
cat generated.luau | lusa --raw -
lusa --capabilities
```

The explicit Lune-style form is also supported:

```bash
lusa run script.luau arg1 arg2
```

## Built for Luau tooling

Lusa v0.2 accepts common Luau CLI optimization/debug flags (`-O0/-O1/-O2`, `-g0/-g1/-g2`), supports stdin, preserves direct file execution and real Luau diagnostics, and exposes a machine-readable capability report. `LUSA_RAW=1` lets a parent tool disable Lusa's Roblox bootstrap without changing the child command it constructs.

That makes Lusa easy to substitute into software that currently shells out to `luau` or `lune`. See [docs/TOOLING.md](docs/TOOLING.md) for concrete integration examples.

## Roblox compatibility

In the default Roblox mode, Lusa bootstraps a `DataModel` and exposes common Roblox globals such as `game`, `workspace`, `Instance`, `Enum`, Roblox datatypes, `task`, and `DateTime`. It includes offline shims such as `Workspace:GetServerTimeNow()` and `DataModel:IsLoaded()`.

Lusa also ships a generated API registry based on a current Roblox API dump. The v0.2 registry covers **830 Instance-descendant classes**, including **296 services**, so current class/service names can still be created or resolved when they are newer than Lune's bundled reflection database. The registry supplies class/service shape and inheritance; it does not pretend that a standalone process can implement every live-engine method or property.

The user script remains the direct `Runtime::run_file` entrypoint, so parser/compiler diagnostics keep the real script path and line numbers instead of pointing into a generated wrapper.

## Host access and sandboxing

Lusa is optimized for compatibility with real Luau tooling, so official release binaries include Lune's standard libraries, including filesystem, network, process, regex, task, and Roblox modules. `lusa --capabilities` reports this explicitly with `"host_io": true`.

That makes Lusa useful for trusted deobfuscation and reconstruction harnesses, but it also means **Lusa is not a security sandbox**. Unknown, hostile, or untrusted scripts should run inside a dedicated OS/container sandbox with filesystem, network, process, CPU, memory, and time limits enforced outside Lusa.

## Compatibility contract

Lusa aims for **Luau and offline Roblox engine-surface compatibility**, not a bit-for-bit simulation of a live Roblox client/server. A standalone process cannot reproduce replication, physics, security contexts, engine scheduling, live service state, or an authoritative Roblox server clock.

Executor-only globals such as `identifyexecutor`, `isfunctionhooked`, `islclosure`, and `getgenv` are intentionally not fabricated by Lusa. Those are not Roblox APIs, and spoofing them would make analysis results less trustworthy. See [COMPATIBILITY.md](COMPATIBILITY.md) for the detailed boundary.

## Tests and releases

```bash
cargo build --release
python scripts/test-lusa.py target/release/lusa
```

The black-box suite covers Roblox bootstrapping, current API registry names, native Luau behavior, executor-boundary behavior, syntax errors, Luau CLI flags, stdin, raw mode, machine capability reporting, and host-library availability. GitHub Actions runs the same suite on Linux x64 and Windows x64 before publishing release binaries.

## Architecture

Lusa remains a small compatibility layer rather than renaming or forking every internal `@lune/*` package. It depends on Lune `0.10.5`, installs its generated class/service registry and Roblox-facing bootstrap only when Roblox mode is active, and leaves raw mode available for tools that need to own the entire target environment.

## License and attribution

Lusa is distributed under MPL-2.0. It depends on Lune, which is also MPL-2.0-licensed. See [LICENSE.txt](LICENSE.txt) and [NOTICE.md](NOTICE.md).
