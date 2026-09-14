<p align="center">
  <img src="assets/logo/lusa-wordmark.svg" alt="Lusa" width="640">
</p>

# Lusa

**Lusa** is a Roblox-focused Luau compatibility runner built on top of
[Lune](https://github.com/lune-org/lune). It keeps the small standalone-runtime
workflow (`lusa script.luau`) while installing a Roblox-like global environment
before the user script is compiled and executed.

The project exists for Luau tooling, compatibility testing, build pipelines, and
running Roblox-oriented code outside Studio where an offline model is useful.
It is not an exploit executor and does not fabricate executor identity, hook
state, native-function provenance, or hidden stack frames.

## Quick start

```bash
cargo build --release
./target/release/lusa script.luau
```

On Windows:

```powershell
cargo build --release
.\target\release\lusa.exe script.luau
```

The CLI also accepts the explicit form:

```bash
lusa run script.luau arg1 arg2
```

## What Lusa adds

Lusa currently bootstraps a `DataModel` and exposes the common Roblox globals
`game`, `workspace`, `Instance`, `Enum`, Roblox datatypes, `task`, and
`DateTime`. It adds an offline `Workspace:GetServerTimeNow()`, registers
`SlimAnimationReplicationService` when the bundled reflection database does not
know it, and implements `DataModel:IsLoaded()`.

The user script remains the direct `Runtime::run_file` entrypoint. This is
important because Luau parse/compile diagnostics keep the real script path and
line numbers instead of pointing into a generated wrapper.

## Compatibility contract

Lusa aims for **Luau and offline Roblox engine-surface compatibility**, not a
bit-for-bit simulation of a live Roblox client/server. A standalone process
cannot reproduce replication, physics, security contexts, engine scheduling,
live service state, or a server-synchronized clock without the Roblox engine.
See [COMPATIBILITY.md](COMPATIBILITY.md) for the detailed boundary.

Executor-only globals such as `identifyexecutor`, `isfunctionhooked`,
`islclosure`, and `getgenv` are intentionally absent. Those are not Roblox APIs.

## Tests

After building:

```bash
python scripts/test-lusa.py target/release/lusa
```

The black-box suite covers the Roblox bootstrap, core Luau/native semantics,
executor-boundary behavior, valid Luau, and intentionally invalid syntax.
GitHub Actions runs the same suite on Linux x64 and Windows x64 and uploads both
binaries as artifacts.

## Build artifacts

Push to `main`, create a pull request, or manually run **Lusa CI / x64 builds**
in GitHub Actions. The workflow creates:

- `lusa-linux-x86_64`
- `lusa-windows-x86_64`

Tags matching `v*` also publish the built binaries to a GitHub Release.

## Architecture

Lusa is deliberately a small compatibility layer instead of a hostile global
rename of every internal `@lune/*` package. It depends on Lune `0.10.5`, uses
Lune's real Luau VM/runtime, then installs Roblox-facing behavior in
`src/roblox_bootstrap.luau`. This keeps upstream library semantics intact and
makes future Lune updates easier to review.

## Branding

Lusa's purple-to-blue assets live under [`assets/logo`](assets/logo). The mark
uses a crescent/orbital visual language inspired by Lune while remaining a
separate Lusa identity.

## License and attribution

Lusa is distributed under MPL-2.0. It depends on Lune, which is also
MPL-2.0-licensed. See [LICENSE.txt](LICENSE.txt) and [NOTICE.md](NOTICE.md).
