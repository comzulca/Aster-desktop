#!/usr/bin/env python3
"""Verify the closed locally modified codec inventory; never execute a codec."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tempfile

UPSTREAM_ARCHIVE = "27bc9d5b815bc103f142aa054f561d9187d191692ec7c2d1e2b4737f8dbd7296"
UPSTREAM_VCS = "31d81fed7551c8ccea456d9d8e2b1fd8bebb6995"
MODIFIED = {"src/bitstream.rs", "src/mcu.rs", "src/mcu_prog.rs"}
MAX_FILE = 2 * 1024 * 1024
MAX_FILES = 100
PROVENANCE = "ASTER-PROVENANCE.json"


def linked(path: Path) -> bool:
    return path.is_symlink() or (hasattr(path, "is_junction") and path.is_junction())


def read(path: Path, root: Path) -> bytes:
    info = path.lstat()
    if linked(path) or not stat.S_ISREG(info.st_mode) or info.st_size > MAX_FILE:
        raise ValueError("Vendor input is not a bounded regular file")
    if not path.resolve(strict=True).is_relative_to(root.resolve(strict=True)):
        raise ValueError("Vendor path escaped root")
    return path.read_bytes()


def entries(items: object) -> dict[str, str]:
    if not isinstance(items, list) or len(items) > MAX_FILES:
        raise ValueError("Vendor listing is not bounded")
    result = {}
    for item in items:
        if not isinstance(item, dict):
            raise ValueError("Vendor entry is not an object")
        path, digest = item.get("path"), item.get("sha256")
        if not isinstance(path, str) or not isinstance(digest, str):
            raise ValueError("Vendor entry has no path/hash")
        relative = PurePosixPath(path)
        if (relative.is_absolute() or str(relative) != path or ".." in relative.parts
                or ":" in path or "\\" in path or "\x00" in path or not relative.parts
                or not re.fullmatch(r"[0-9a-f]{64}", digest) or path in result):
            raise ValueError("Vendor path/hash is noncanonical or duplicated")
        result[path] = digest
    return result


def verify(project: Path) -> dict:
    root = project / "src-tauri" / "vendor" / "zune-jpeg-0.5.15"
    for ancestor in (project, project / "src-tauri", project / "src-tauri" / "vendor", root):
        if linked(ancestor) or not ancestor.is_dir():
            raise ValueError("Vendor root/ancestor is not a real directory")
    raw = read(root / PROVENANCE, root)
    value = json.loads(raw)
    if not isinstance(value, dict):
        raise ValueError("Vendor provenance is not an object")
    upstream = value.get("upstream", {})
    if not isinstance(upstream, dict):
        raise ValueError("Vendor upstream provenance is not an object")
    if (value.get("schemaVersion") != 1 or value.get("name") != "zune-jpeg"
            or value.get("version") != "0.5.15"
            or upstream.get("archiveSha256") != UPSTREAM_ARCHIVE
            or upstream.get("vcsRevision") != UPSTREAM_VCS
            or upstream.get("vcsPath") != "crates/zune-jpeg"):
        raise ValueError("Vendor provenance differs from the reviewed upstream")
    original = entries(upstream.get("files"))
    changes = entries(value.get("modifications"))
    additions = entries(value.get("addedFiles"))
    if (set(changes) != MODIFIED or not set(changes).issubset(original)
            or set(additions) != {"ASTER-PATCH.md"} or set(additions) & set(original)
            or PROVENANCE in original or not {"Cargo.toml", "LICENSE-MIT", "LICENSE-APACHE", "LICENSE-ZLIB"}.issubset(original)):
        raise ValueError("Vendor changed/added/source inventory differs from review")
    expected = {**original, **changes, **additions}
    actual = set()
    for directory, subdirectories, files in os.walk(root, followlinks=False):
        for child in subdirectories:
            if linked(Path(directory) / child):
                raise ValueError("Vendor directory link rejected")
        for child in files:
            file = Path(directory) / child
            name = file.relative_to(root).as_posix()
            actual.add(name)
            if len(actual) > MAX_FILES:
                raise ValueError("Vendor file inventory exceeded its bound")
            if name == PROVENANCE:
                continue  # The recorded manifest cannot hash itself.
            if name not in expected or hashlib.sha256(read(file, root)).hexdigest() != expected[name]:
                raise ValueError("Vendor file missing from provenance or hash changed: " + name)
    if actual != set(expected) | {PROVENANCE}:
        raise ValueError("Vendor file inventory is incomplete")
    return {"classification": "source-provenance-not-codec-correctness", "name": "zune-jpeg",
            "version": "0.5.15", "upstreamArchiveSha256": UPSTREAM_ARCHIVE,
            "provenanceSha256": hashlib.sha256(raw).hexdigest(), "files": len(actual),
            "modifiedFiles": sorted(changes)}


def self_test() -> None:
    digest = "0" * 64
    for path in ("../escape", "/absolute", "a\\b", "a:b", "a/./b", ""):
        try:
            entries([{"path": path, "sha256": digest}])
        except ValueError:
            pass
        else:
            raise AssertionError("Invalid vendor path accepted")
    try:
        entries([{"path": "a", "sha256": digest}, {"path": "a", "sha256": digest}])
    except ValueError:
        pass
    else:
        raise AssertionError("Duplicate vendor path accepted")
    with tempfile.TemporaryDirectory(prefix="aster-vendor-self-test-") as temporary:
        try:
            verify(Path(temporary))
        except (ValueError, OSError):
            pass
        else:
            raise AssertionError("Missing vendor tree accepted")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            self_test()
            print("Vendor provenance self-test: PASS")
        else:
            print(json.dumps(verify(args.root.resolve(strict=True)), sort_keys=True))
        return 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print("Vendor provenance FAIL: " + str(error))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
