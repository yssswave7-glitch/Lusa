# Tooling integration

Lusa is designed to be a drop-in Luau runtime for offline tooling that repeatedly executes generated or transformed Luau: deobfuscators, trace harnesses, reconstructors, syntax validators, and build/test pipelines.

## Why Lusa fits this workload

Lusa executes files directly (`lusa file.luau`) and also accepts the Lune-style `run` subcommand. It accepts the common Luau CLI compatibility flags `-O0`, `-O1`, `-O2`, `-g0`, `-g1`, and `-g2`, so tools that invoke `luau -O2 generated.luau` do not need a special command builder. These flags are accepted for command-line compatibility; Lusa still uses the Luau VM/compiler bundled by Lune.

Generated programs can also be streamed over stdin:

```bash
cat generated.luau | lusa --raw -
```

`lusa --capabilities` prints a small JSON object intended for automatic runtime detection.

## Raw and isolated modes for harnesses

Trace/deobfuscation harnesses often construct their own `game`, `Instance`, mock services, proxy objects, and callback scheduler. Preloading a second Roblox environment would change the bootstrap they are trying to observe, so use raw mode for those harnesses:

```bash
lusa --raw harness.luau
```

If the parent program controls the child command and only appends a script path, set the environment variable instead:

```bash
LUSA_RAW=1 lusa harness.luau
```

Raw mode keeps the Luau/Lune runtime and its standard libraries, including `@lune/task`, but skips Lusa's `game` / `workspace` / Roblox API registry bootstrap.

For unknown inputs, combine it with the defense-in-depth isolated profile:

```bash
lusa --raw --isolated harness.luau
```

This removes inherited process environment values and denies filesystem/network/process APIs and file module loads while retaining task, datetime, serde, regex, and offline Roblox libraries. It complements rather than replaces OS/container limits.

## `env.py`-style trace harnesses

The updated ENV LOG harness recognizes `--lusa` and invokes this profile directly:

```bash
python env.py --lusa /path/to/lusa sample.lua
```

Or set the runtime path entirely through the environment:

```bash
LUSA_BIN=/path/to/lusa python env.py sample.lua
```

Other harnesses can invoke `lusa --raw --isolated <runner>` directly or set `LUSA_RAW=1` and `LUSA_ISOLATED=1`.

## Validator / deobfuscator integration

Tools that expose a `--luau` runtime option can point it directly at Lusa:

```bash
python deobfuscator.py --luau /path/to/lusa input.lua
```

A validator command such as:

```text
/path/to/lusa -O2 scratch.luau
```

is accepted unchanged.

## Roblox mode

Without `--raw` or `LUSA_RAW=1`, Lusa installs its offline Roblox environment. The checked-in registry is generated from a Roblox API dump and registers current `Instance` descendants and services that may be newer than Lune's bundled reflection database. This improves class/service-name compatibility while deliberately not inventing live engine state, networking, physics, security contexts, or class-specific behavior that the standalone process cannot reproduce.

Regenerate it from a newer API dump with:

```bash
python scripts/generate-roblox-registry.py roblox-api.json
```

## Host access boundary

Official Lusa release binaries prioritize tooling compatibility and expose Lune's standard libraries, including direct filesystem, network, process, and regex modules. `lusa --capabilities` reports `"host_io": true` and identifies the exposed host-I/O module families.

Normal mode is useful for trusted analysis harnesses. Isolated mode reduces the exposed in-process surface, but **Lusa is not a hardened OS security sandbox**. Run hostile or unknown code inside a dedicated OS/container sandbox with process, filesystem, network, CPU, memory, and wall-clock restrictions enforced outside the Lusa process.

## Compatibility boundary

Lusa does not spoof exploit-executor identities or fabricate hook state, native/C-closure provenance, or hidden stack frames. Analysis harnesses should model any non-Roblox executor surface explicitly in their own target environment when that behavior is part of the artifact being studied.
