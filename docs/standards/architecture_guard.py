#!/usr/bin/env python3
import os
import sys

"""
QuickSort Architecture Guard
Enforces strict compile-time Hexagonal/DDD layer isolation rules.
"""

LAYER_RULES = {
    "crates/quicksort-domain": {
        "forbidden_dependencies": [
            "tauri", "windows", "winreg", "winctx", "serde_json",
            "serde", "uuid", "chrono", "anyhow", "thiserror", "tokio",
            "application", "infrastructure", "adapters"
        ],
        "description": "Domain layer must be entirely pure. No external crates allowed."
    },
    "crates/quicksort-application": {
        "forbidden_dependencies": [
            "tauri", "windows", "winreg", "winctx", "infrastructure", "adapters"
        ],
        "description": "Application use cases cannot depend on concrete implementations or UI bindings."
    },
}

def check_cargo_toml(file_path, forbidden_deps):
    if not os.path.exists(file_path):
        return []
    violations = []
    with open(file_path, "r", encoding="utf-8") as f:
        in_deps_block = False
        for line in f:
            line = line.strip()
            if line.startswith("[dependencies]") or line.startswith("[dev-dependencies]"):
                in_deps_block = True
                continue
            if line.startswith("[") and in_deps_block:
                in_deps_block = False
            if in_deps_block and line and not line.startswith("#"):
                dep_name = line.split("=")[0].strip().replace('"', '').replace("'", "")
                if dep_name in forbidden_deps:
                    violations.append(dep_name)
    return violations

def main():
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    has_violations = False

    print("🛡️ Running QuickSort Architectural Layer Compliance Verification...")

    for layer_path, rules in LAYER_RULES.items():
        target = os.path.join(repo_root, layer_path, "Cargo.toml")
        violations = check_cargo_toml(target, rules["forbidden_dependencies"])
        if violations:
            has_violations = True
            print(f"\n❌ ARCHITECTURE VIOLATION in layer [{layer_path}]:")
            print(f"   Reason: {rules['description']}")
            print(f"   Illegal dependencies found: {violations}")

    if has_violations:
        print("\n💥 Build rejected. Please fix dependency directions to align with ADR-002.")
        sys.exit(1)
    else:
        print("✅ Success! All workspace crate layers conform perfectly to inward-pointing dependency rule boundaries.")
        sys.exit(0)

if __name__ == "__main__":
    main()