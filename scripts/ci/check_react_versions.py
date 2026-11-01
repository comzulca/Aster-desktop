"""T-097: reject split React/React DOM cohorts before frontend execution."""
import argparse
import copy
import json
from pathlib import Path
import re
import sys


def one(pattern, text):
    matches = re.findall(pattern, text, re.MULTILINE)
    if len(matches) != 1:
        raise ValueError("Missing, ambiguous or unsupported React lock representation")
    return matches[0]


def verify(manifest, lock):
    if not lock.startswith("lockfileVersion: '9.0'\n"):
        raise ValueError("Review the new pnpm lock format before admission")
    prefix, snapshots = lock.split("\nsnapshots:\n")
    importers, packages = prefix.split("\npackages:\n")
    root = one(r"^  \.:\n([\s\S]*)", importers)
    react_spec, version = one(r"^      react:\n        specifier: ([^\n]+)\n        version: ([^\n]+)$", root)
    dom_spec, dom = one(r"^      react-dom:\n        specifier: ([^\n]+)\n        version: ([^\n]+)$", root)
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError("Expected one reviewed stable React runtime version")
    declarations = manifest.get("dependencies", {})
    if not (declarations.get("react") == declarations.get("react-dom") == react_spec == dom_spec):
        raise ValueError("React runtime manifest/importer requirements must match")
    if dom != f"{version}(react@{version})":
        raise ValueError("React and React DOM must resolve to the exact same version")
    keys = re.findall(r"^  (react(?:-dom)?@[^\n]+):$", packages, re.MULTILINE)
    if len(keys) != 2 or set(keys) != {f"react@{version}", f"react-dom@{version}"}:
        raise ValueError("React package resolutions are missing, duplicated or split")
    one(rf"^  react@{re.escape(version)}: \{{\}}$", snapshots)
    dependency = one(rf"^  react-dom@{re.escape(version)}\(react@{re.escape(version)}\):\n    dependencies:\n      react: ([^\n]+)$", snapshots)
    if dependency != version:
        raise ValueError("React DOM snapshot uses a stale React dependency")


def self_test():
    version = "19.3.0"
    manifest = {"dependencies": {"react": "^19.3.0", "react-dom": "^19.3.0"}}
    lock = f"""lockfileVersion: '9.0'

importers:

  .:
    dependencies:
      react:
        specifier: ^19.3.0
        version: {version}
      react-dom:
        specifier: ^19.3.0
        version: {version}(react@{version})

packages:

  react@{version}:
    resolution: {{integrity: synthetic}}
  react-dom@{version}:
    resolution: {{integrity: synthetic}}

snapshots:

  react@{version}: {{}}
  react-dom@{version}(react@{version}):
    dependencies:
      react: {version}
"""
    verify(manifest, lock)
    rejected = 0
    for altered in [
        lock.replace("version: 19.3.0(react@19.3.0)", "version: 19.3.0(react@19.3.1)"),
        lock.replace("        version: 19.3.0\n", "        version: 19.3.1\n"),
        lock.replace("        specifier: ^19.3.0", "        specifier: ^19.3.1", 1),
        lock.replace("  react-dom@19.3.0:\n", "  react-dom@19.3.1:\n"),
        lock.replace("      react: 19.3.0\n", "      react: 19.3.1\n"),
        lock.replace("  react@19.3.0: {}", "  react@19.3.0: {}\n  react@19.3.0: {}"),
        lock.replace("lockfileVersion: '9.0'", "lockfileVersion: '10.0'"),
    ]:
        try:
            verify(manifest, altered)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("Split or incomplete React cohort admitted")
    changed = copy.deepcopy(manifest)
    changed["dependencies"]["react-dom"] = "^19.3.1"
    try:
        verify(changed, lock)
    except ValueError:
        rejected += 1
    else:
        raise AssertionError("Manifest split admitted")
    print(f"React cohort self-test PASS: aligned fixture and {rejected} negative controls")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            self_test()
        else:
            verify(json.loads((args.root / "package.json").read_text()), (args.root / "pnpm-lock.yaml").read_text())
            print("Locked React runtime cohort PASS")
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"React cohort FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)
