#!/usr/bin/env python3
"""Reject the incompatible WinRT async cohort observed in PR #44."""

from __future__ import annotations

import argparse
import copy
from pathlib import Path
import sys
import tomllib


# A coordinated migration must review these pins with the native PDF factories.
REVIEWED = {"windows": "0.62.2", "windows-future": "0.3.2", "windows-core": "0.62.2"}


def verify(manifest: dict, lock: dict) -> None:
    dependencies = manifest.get("target", {}).get("cfg(windows)", {}).get("dependencies", {})
    for name in ("windows", "windows-future"):
        declaration = dependencies.get(name)
        if not isinstance(declaration, dict) or declaration.get("version") != "=" + REVIEWED[name]:
            raise ValueError(f"{name}: require the exact reviewed manifest pin")
    if dependencies["windows-future"].get("default-features") is not False:
        raise ValueError("windows-future: default features must remain disabled")
    if dependencies["windows-future"].get("features", []) != []:
        raise ValueError("windows-future: additional features require review")

    packages = lock.get("package", [])

    def package(name: str) -> dict:
        matches = [item for item in packages if item.get("name") == name]
        if len(matches) != 1:
            raise ValueError(f"{name}: missing or ambiguous locked package")
        found = matches[0]
        if name in REVIEWED and found.get("version") != REVIEWED[name]:
            raise ValueError(f"{name}: unreviewed locked version")
        return found

    def edge(owner: dict, dependency: str) -> dict:
        entries = owner.get("dependencies", [])
        matches = [item.split() for item in entries if isinstance(item, str) and item.split() and item.split()[0] == dependency]
        target = package(dependency)
        if len(matches) != 1 or matches[0] not in ([dependency], [dependency, target["version"]]):
            raise ValueError(f"{owner['name']}: missing or mismatched {dependency} edge")
        return target

    app = package("aster-desktop")
    windows = edge(app, "windows")
    future = edge(app, "windows-future")
    if edge(windows, "windows-future") is not future:
        raise ValueError("root and Windows async types must use the same locked package")
    if edge(windows, "windows-core") is not edge(future, "windows-core"):
        raise ValueError("Windows and async types must share the reviewed core ABI")


def self_test() -> None:
    manifest = {"target": {"cfg(windows)": {"dependencies": {
        "windows": {"version": "=0.62.2"},
        "windows-future": {"version": "=0.3.2", "default-features": False},
    }}}}
    lock = {"package": [
        {"name": "aster-desktop", "version": "0.2.4", "dependencies": ["windows", "windows-future"]},
        {"name": "windows", "version": "0.62.2", "dependencies": ["windows-core", "windows-future"]},
        {"name": "windows-future", "version": "0.3.2", "dependencies": ["windows-core"]},
        {"name": "windows-core", "version": "0.62.2"},
    ]}
    verify(manifest, lock)
    qualified = copy.deepcopy(lock)
    qualified["package"][0]["dependencies"] = ["windows 0.62.2", "windows-future 0.3.2"]
    verify(manifest, qualified)

    rejected = 0

    def reject(change) -> None:
        nonlocal rejected
        candidate_manifest, candidate_lock = copy.deepcopy(manifest), copy.deepcopy(lock)
        change(candidate_manifest, candidate_lock)
        try:
            verify(candidate_manifest, candidate_lock)
        except (ValueError, KeyError, TypeError):
            rejected += 1
            return
        raise AssertionError("An incompatible or incomplete WinRT fixture was admitted")

    def incompatible_upgrade(m, entries):
        m["target"]["cfg(windows)"]["dependencies"]["windows-future"]["version"] = "=0.100.0"
        entries["package"][0]["dependencies"][1] = "windows-future 0.100.0"
        entries["package"].extend([
            {"name": "windows-future", "version": "0.100.0", "dependencies": ["windows-core 0.100.0"]},
            {"name": "windows-core", "version": "0.100.0"},
        ])

    reject(incompatible_upgrade)
    reject(lambda m, entries: m["target"]["cfg(windows)"]["dependencies"]["windows"].update(version="0.62"))
    reject(lambda m, entries: m["target"]["cfg(windows)"]["dependencies"]["windows-future"].update(**{"default-features": True}))
    reject(lambda m, entries: m["target"]["cfg(windows)"]["dependencies"]["windows-future"].update(features=["std"]))
    reject(lambda m, entries: entries["package"][0].update(dependencies=["windows"]))
    reject(lambda m, entries: entries["package"][0].update(dependencies=["windows 0.100.0", "windows-future"]))
    reject(lambda m, entries: entries["package"][1].update(dependencies=["windows-core"]))
    reject(lambda m, entries: entries["package"][1].update(dependencies=["windows-core", "windows-future 0.100.0"]))
    reject(lambda m, entries: entries["package"][2].update(dependencies=["windows-core 0.100.0"]))
    reject(lambda m, entries: entries["package"][3].update(version="0.100.0"))
    reject(lambda m, entries: entries["package"].append({"name": "windows-core", "version": "0.100.0"}))
    reject(lambda m, entries: entries["package"].append(copy.deepcopy(entries["package"][2])))
    reject(lambda m, entries: entries["package"].pop(3))
    print(f"WinRT cohort self-test: 2 admitted fixtures, {rejected} rejected fixtures PASS")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            self_test()
        else:
            manifest = tomllib.loads((args.root / "src-tauri/Cargo.toml").read_text(encoding="utf-8"))
            lock = tomllib.loads((args.root / "src-tauri/Cargo.lock").read_text(encoding="utf-8"))
            verify(manifest, lock)
            print("Reviewed locked WinRT async cohort PASS")
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"WinRT cohort FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
