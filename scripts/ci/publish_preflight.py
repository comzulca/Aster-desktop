#!/usr/bin/env python3
"""Read-only GitHub publication preflight for the Aster-desktop checkout.

This helper prints a deterministic manifest from Git-visible files; it never
contacts GitHub, mutates the repository, stages files, or opens credentials.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys

EXPECTED_JPG_SHA256 = "b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path, help="Local Git working tree root")
    parser.add_argument("--base64-index", type=int, help="Read the indexed file as base64 after rechecking its hash")
    parser.add_argument("--expected-sha256", help="Required SHA-256 for --base64-index")
    parser.add_argument("--base64-offset", type=int, default=0, help="Offset in the base64 string; use with --base64-index")
    parser.add_argument("--base64-length", type=int, help="Maximum number of base64 characters to print")
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    if not (root / ".git").exists():
        parser.error("root does not contain a .git repository")
    raw_paths = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
    ).split(b"\0")
    paths = sorted(set(raw_paths) - {b""})
    records = []
    for raw in paths:
        # Git path names can be arbitrary bytes on Linux; the connector needs
        # UTF-8 paths to preserve their identity. Fail instead of mangling.
        relative = raw.decode("utf-8", errors="strict")
        if relative.startswith("/") or "\\" in relative or any(
            component in ("", ".", "..") for component in relative.split("/")
        ):
            raise ValueError(f"Unsafe repository path {relative!r}")
        path = root / relative
        meta = path.lstat()
        if not stat.S_ISREG(meta.st_mode):
            raise ValueError(f"Expected a regular file, found {relative!r}")
        data = path.read_bytes()
        if path.stat().st_size != len(data):
            raise ValueError(f"File changed during read: {relative!r}")
        sha256 = hashlib.sha256(data).hexdigest()
        git_blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        mode = "100755" if meta.st_mode & stat.S_IXUSR else "100644"
        records.append(
            {"path": relative, "size": len(data), "sha256": sha256, "git_blob_sha1": git_blob, "mode": mode}
        )
    jpg = next((r for r in records if r["path"] == "public/assets/Aster_UI2.jpg"), None)
    if jpg is None or jpg["sha256"] != EXPECTED_JPG_SHA256:
        raise ValueError("Original Aster_UI2.jpg missing or altered")
    if args.base64_index is not None:
        if args.expected_sha256 is None:
            parser.error("--base64-index requires --expected-sha256")
        if args.base64_offset < 0 or (args.base64_length is not None and args.base64_length < 1):
            parser.error("base64 offset must be nonnegative and length must be positive")
        record = records[args.base64_index]
        if record["sha256"] != args.expected_sha256:
            raise ValueError("Manifest changed since index selection")
        import base64

        data = (root / record["path"]).read_bytes()
        if hashlib.sha256(data).hexdigest() != args.expected_sha256:
            raise ValueError("File changed during base64 read")
        encoded = base64.b64encode(data).decode("ascii")
        sys.stdout.write(encoded[args.base64_offset : args.base64_offset + args.base64_length if args.base64_length else None])
        return
    if args.expected_sha256 is not None:
        parser.error("--expected-sha256 requires --base64-index")
    if args.base64_offset != 0 or args.base64_length is not None:
        parser.error("base64 offset/length require --base64-index")
    manifest = {
        "root": str(root),
        "count": len(records),
        "total_bytes": sum(record["size"] for record in records),
        "files": records,
    }
    canonical = json.dumps(manifest["files"], sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    manifest["manifest_sha256"] = hashlib.sha256(canonical).hexdigest()
    json.dump(manifest, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
