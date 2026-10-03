#!/usr/bin/env python3
"""Best-effort tracked-source credential guard; never print matching bytes.

This is one gate, not a substitute for GitHub secret protection or review. It
intentionally scans all tracked text, including documentation and workflows.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
MAX_SOURCE_BYTES = 4 * 1024 * 1024
SKIP_DIRS = {".git", ".pnpm-store", "node_modules", "target", "dist", "coverage", "reports"}
PATTERNS = {
    "Meta direct API token": re.compile(
        r"\b(?:LLM\|[0-9]{1,32}\|[A-Za-z0-9_-]{12,256}|LLM_[A-Za-z0-9_.-]{12,})"
    ),
    "NVIDIA token": re.compile(r"\bnvapi-[A-Za-z0-9_-]{20,}\b"),
    "GitHub token": re.compile(r"\b(?:gh[opsur]_[A-Za-z0-9_]{30,}|github_pat_[A-Za-z0-9_]{50,})\b"),
    "private key": re.compile(r"-----BEGIN (?:RSA |DSA |EC |OPENSSH |ENCRYPTED )?PRIVATE KEY-----"),
    "assigned credential": re.compile(
        r"(?im)^\s*(?:export\s+)?(?:META_API_KEY|NVIDIA_API_KEY|GITHUB_TOKEN|GH_TOKEN|API_KEY|SECRET_KEY)"
        r"\s*[:=]\s*['\"]?(?!\$|<|\*|YOUR_|REPLACE_|EXAMPLE_)(?P<value>[A-Za-z0-9_+/=-]{24,})"
    ),
}


def tracked_paths(include_untracked: bool = False) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", *( ["--others", "--exclude-standard"] if include_untracked else [] ), "-z"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if result.returncode == 0 and result.stdout:
        return [ROOT / Path(item.decode("utf-8", "surrogateescape")) for item in result.stdout.split(b"\0") if item]
    if os.environ.get("GITHUB_ACTIONS") == "true":
        raise RuntimeError("No tracked source files are available for the secret guard")
    # Local bootstrap before the first git commit; CI always uses the tracked list.
    return [p for p in ROOT.rglob("*") if p.is_file() and not SKIP_DIRS.intersection(p.relative_to(ROOT).parts)]


def findings_for(source: str) -> list[tuple[str, int]]:
    findings = []
    for label, pattern in PATTERNS.items():
        for match in pattern.finditer(source):
            if label == "Meta direct API token" and match.group() in {
                "LLM|0|synthetic-fixture-key-123", "LLM|0|synthetic-ascii-key-123",
                "LLM|0|SYNTHETIC_NOT_A_SECRET", "LLM_0_SYNTHETIC_NOT_A_SECRET",
            }:
                continue
            # One exact non-secret opaque fixture, never a prefix exception.
            # Assigned values and direct tokens share this exact whitelist.
            if label == "assigned credential" and match.group("value") == "LLM_0_SYNTHETIC_NOT_A_SECRET":
                continue
            findings.append((label, source.count("\n", 0, match.start()) + 1))
    return findings


def self_test() -> None:
    positive = "nvapi-" + "a" * 40 + "\n" + "-----BEGIN " + "PRIVATE KEY-----"
    labels = {label for label, _ in findings_for(positive)}
    assert labels == {"NVIDIA token", "private key"}
    assert findings_for("LLM|123|" + "a" * 40) == [("Meta direct API token", 1)]
    assert not findings_for("LLM|0|synthetic-fixture-key-123")
    assert findings_for("LLM|0|synthetic-fixture-key-123" + "-extra")
    assert not findings_for('api_key="$NVIDIA_API_KEY"\nNVIDIA_API_KEY="<enter on Windows>"')
    opaque_synthetic = "LLM_0_SYNTHETIC_NOT_A_SECRET"
    for opaque in ["LLM_" + "a" * 12, "LLM_" + "aA09_.-" * 40]:
        assert findings_for(opaque) == [("Meta direct API token", 1)]
        for assigned in [f'META_API_KEY="{opaque}"', f"API_KEY='{opaque}'"]:
            assert "Meta direct API token" in {label for label, _ in findings_for(assigned)}
    assert not findings_for(opaque_synthetic)
    assert not findings_for(f'META_API_KEY="{opaque_synthetic}"')
    for suffix in ["-extra", ".extra", "_extra", "a" * 300]:
        extended = opaque_synthetic + suffix
        assert findings_for(extended)
        assert findings_for(f'META_API_KEY="{extended}"')
    for non_token in ["LLM_" + "x" * 11, "llm_" + "x" * 12, "LLM_<enter in the native prompt>"]:
        assert not findings_for(non_token)


def scan(include_untracked: bool = False) -> int:
    errors = 0
    inspected = 0
    try:
        paths = sorted(set(tracked_paths(include_untracked)))
    except RuntimeError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    for path in paths:
        relative = path.relative_to(ROOT)
        if path.is_symlink():
            print(f"ERROR {relative}: tracked symlink needs manual inspection", file=sys.stderr)
            errors += 1
            continue
        try:
            size = path.stat().st_size
            if size > MAX_SOURCE_BYTES:
                # Raster assets are never executed or parsed as text by this scanner.
                if path.suffix.lower() in {".jpg", ".jpeg", ".png", ".ico", ".webp"}:
                    continue
                print(f"ERROR {relative}: source too large for secret scanner", file=sys.stderr)
                errors += 1
                continue
            data = path.read_bytes()
        except OSError:
            print(f"ERROR {relative}: cannot read tracked file", file=sys.stderr)
            errors += 1
            continue
        if b"\x00" in data:
            continue
        try:
            content = data.decode("utf-8")
        except UnicodeDecodeError:
            continue
        inspected += 1
        for label, line in findings_for(content):
            print(f"ERROR {relative}:{line}: possible {label}; inspect locally", file=sys.stderr)
            errors += 1
    print(f"Secret guard checked {inspected} UTF-8 source files; findings/errors: {errors}.")
    return 1 if errors else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--include-untracked", action="store_true", help="Include non-ignored local source before committing")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("Secret guard synthetic positive and placeholder checks passed.")
        return 0
    return scan(args.include_untracked)


if __name__ == "__main__":
    raise SystemExit(main())
