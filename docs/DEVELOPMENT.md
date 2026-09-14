# Development

## Requirements

- Rust stable with edition 2024 support
- Python 3.9+ for the black-box test runner

## Local loop

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --release
python scripts/test-lusa.py target/release/lusa
```

On Windows, pass `target/release/lusa.exe` to the Python script.

## Adding Roblox compatibility

Prefer extending `src/roblox_bootstrap.luau` using Lune's public
`@lune/roblox` extension hooks (`registerClass`, `registerService`,
`implementMethod`, and `implementProperty`). Every new behavior should have a
small black-box regression test under `tests/compat`.

Do not fake third-party executor identity/provenance APIs as a substitute for an
engine implementation. If a test depends on an executor-only global, keep that
expectation in the executor-boundary category instead of labeling it Roblox
parity.
