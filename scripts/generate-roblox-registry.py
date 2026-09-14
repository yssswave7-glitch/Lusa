#!/usr/bin/env python3
"""Generate Lusa's offline Roblox class/service registry from an API dump."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path

CHUNK_COUNT = 12


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("api_dump", type=Path)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("src"),
        help="directory for roblox_api_registry_XX.luau files",
    )
    args = parser.parse_args()

    raw = args.api_dump.read_bytes()
    data = json.loads(raw)
    classes = {entry["Name"]: entry for entry in data["Classes"]}

    def instance_depth(name: str) -> int | None:
        depth = 0
        seen: set[str] = set()
        current = name
        while current in classes and current not in seen:
            if current == "Instance":
                return depth
            seen.add(current)
            current = classes[current].get("Superclass")
            depth += 1
        return None

    services: list[str] = []
    regular: list[tuple[int, str, str]] = []

    for entry in data["Classes"]:
        name = entry["Name"]
        depth = instance_depth(name)
        if depth is None or name == "Instance":
            continue
        superclass = entry.get("Superclass") or "Instance"
        if "Service" in (entry.get("Tags") or []):
            services.append(name)
        else:
            regular.append((depth, name, superclass))

    services = sorted(set(services), key=str.lower)
    regular.sort(key=lambda item: (item[0], item[1].lower()))

    actions: list[tuple[str, str, str | None]] = [
        ("service", name, None) for name in services
    ]
    actions.extend(("class", name, superclass) for _, name, superclass in regular)

    source_hash = hashlib.sha256(raw).hexdigest()
    per_chunk = max(1, math.ceil(len(actions) / CHUNK_COUNT))
    args.output_dir.mkdir(parents=True, exist_ok=True)

    for index in range(CHUNK_COUNT):
        start = index * per_chunk
        end = min(len(actions), start + per_chunk)
        chunk = actions[start:end]
        lines = [
            "--!strict",
            "-- Generated from a Roblox API dump for Lusa's offline class/service registry.",
            f"-- Source SHA-256: {source_hash}",
            (
                f"-- Chunk {index + 1}/{CHUNK_COUNT}; total Instance descendants: "
                f"{len(actions)}; services: {len(services)}."
            ),
            "",
            'local roblox = require("@lune/roblox")',
            "",
        ]
        for kind, name, superclass in chunk:
            if kind == "service":
                lines.append(f'pcall(roblox.registerService, "{name}")')
            else:
                lines.append(
                    f'pcall(roblox.registerClass, "{name}", "{superclass}")'
                )
        lines.append("")
        out = args.output_dir / f"roblox_api_registry_{index:02}.luau"
        out.write_text("\n".join(lines), encoding="utf-8")

    print(
        f"wrote {CHUNK_COUNT} registry chunks with {len(actions)} "
        f"classes/services ({len(services)} services)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
