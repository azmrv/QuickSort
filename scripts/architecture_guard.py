#!/usr/bin/env python3
"""Enforce the Clean Architecture Dependency Rule for the QuickSort workspace.

QuickSort follows Clean Architecture + DDD: dependencies must point strictly
inward (Domain <- Application <- Infrastructure <- Adapters). This script
verifies that rule by parsing every workspace crate's Cargo.toml and checking
its internal (project-local) dependencies against an allowed matrix.

Allowed internal-dependency matrix (crate -> allowed internal deps):
    quicksort-domain         -> []  (innermost layer, no internal deps)
    quicksort-application    -> [quicksort-domain]
    quicksort-infrastructure -> [quicksort-domain, quicksort-application]
    src-tauri                -> [quicksort-application, quicksort-infrastructure,
                                 quicksort-ipc-contract]
                                 (MUST NOT depend on quicksort-domain directly)
    context-menu-dll         -> [src-tauri, quicksort-ipc-contract]
    quicksort-ipc-contract   -> []  (no internal deps)

Note: the crate physically located in `src-tauri/` is published under the
Cargo package name `quicksort`, so `quicksort` is treated as an alias of the
`src-tauri` crate for both crate identification and dependency resolution.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

# Canonical (matrix) names of the internal crates we govern.
INTERNAL_CRATES = {
    "quicksort-domain",
    "quicksort-application",
    "quicksort-infrastructure",
    "quicksort-ipc-contract",
    "src-tauri",
    "context-menu-dll",
}

# Map a Cargo package name to its canonical (matrix) name.
PACKAGE_ALIASES = {
    "quicksort": "src-tauri",  # src-tauri/Cargo.toml publishes as `quicksort`
}

# Allowed internal-dependency matrix: canonical crate -> allowed canonical deps.
ALLOWED_DEPS: dict[str, list[str]] = {
    "quicksort-domain": [],
    "quicksort-application": ["quicksort-domain"],
    "quicksort-infrastructure": ["quicksort-domain", "quicksort-application"],
    "src-tauri": [
        "quicksort-application",
        "quicksort-infrastructure",
        "quicksort-ipc-contract",
    ],
    "context-menu-dll": ["src-tauri", "quicksort-ipc-contract"],
    "quicksort-ipc-contract": [],
}


def _canonical(name: str) -> str:
    """Return the canonical (matrix) name for a Cargo package/dependency name."""
    return PACKAGE_ALIASES.get(name, name)


def discover_crates(root: Path) -> list[tuple[str, dict]]:
    """Walk *root* for Cargo.toml files and return governed (crate, data) pairs.

    Only crates whose canonical name is part of the Dependency-Rule matrix are
    returned; other crates (e.g. legacy/unused crates) are not enforced.
    """
    governed: list[tuple[str, dict]] = []
    for cargo_toml in sorted(root.rglob("Cargo.toml")):
        # Skip build artifacts / dependency caches.
        if any(part in ("target", "node_modules") for part in cargo_toml.parts):
            continue
        try:
            with cargo_toml.open("rb") as fh:
                data = tomllib.load(fh)
        except (OSError, tomllib.TOMLDecodeError):
            continue
        package = data.get("package")
        if not isinstance(package, dict) or "name" not in package:
            continue  # virtual manifest (e.g. workspace root) -> not a crate
        canonical = _canonical(package["name"])
        if canonical in ALLOWED_DEPS:
            governed.append((canonical, data))
    return governed


def internal_deps(data: dict) -> set[str]:
    """Return the set of canonical internal deps declared in *data*."""
    deps: set[str] = set()
    for dep_name in data.get("dependencies", {}):
        canonical = _canonical(dep_name)
        if canonical in INTERNAL_CRATES:
            deps.add(canonical)
    return deps


def parse_root(argv: list[str]) -> Path:
    """Resolve the --root argument (default: parent of the scripts/ dir)."""
    default = Path(__file__).resolve().parent.parent
    args = argv[1:]
    if "--root" in args:
        idx = args.index("--root")
        if idx + 1 < len(args):
            return Path(args[idx + 1]).resolve()
    for arg in args:
        if arg.startswith("--root="):
            return Path(arg.split("=", 1)[1]).resolve()
    return default


def main(argv: list[str]) -> int:
    root = parse_root(argv)

    violations: list[str] = []
    checked = 0
    for crate, data in discover_crates(root):
        checked += 1
        allowed = set(ALLOWED_DEPS[crate])
        actual = internal_deps(data)
        for dep in sorted(actual - allowed):
            allowed_str = ", ".join(sorted(allowed)) or "none"
            violations.append(
                f"VIOLATION: {crate} depends on {dep} which is not allowed "
                f"(allowed: {allowed_str})"
            )

    if violations:
        for line in violations:
            print(line)
        return 1

    print(f"OK: Dependency Rule satisfied for {checked} crates")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
