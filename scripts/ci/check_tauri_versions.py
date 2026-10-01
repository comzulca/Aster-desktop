#!/usr/bin/env python3
"""T-037: check the fixed root's resolved Rust/npm Tauri major/minor pair.

Read-only, standard-library guard; no imports from the inspected repository.
Alternate Tauri sources/registry routes in the manifest are denied. Cargo
configuration/source-replacement rules are outside these four fixed inputs.
Only pnpm 9's closed root-importer/API selection layout is supported. Transitive
YAML bodies are opaque: frozen pnpm installation validates the complete lock.
This structural check is not a package, license, runtime or acceptance gate.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import stat
import sys
import tomllib


LIMITS = {
    "package.json": 64 * 1024,
    "pnpm-lock.yaml": 2 * 1024 * 1024,
    "src-tauri/Cargo.toml": 64 * 1024,
    "src-tauri/Cargo.lock": 2 * 1024 * 1024,
}
MAX_LINES = 65536
MAX_LINE_BYTES = 8192
API = "@tauri-apps/api"
SEMVER = re.compile(r"(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})")
DEPENDENCY_KEY = re.compile(r"(?:'([A-Za-z0-9_@][A-Za-z0-9_.@/+\-]*)'|([A-Za-z0-9_@][A-Za-z0-9_.@/+\-]*)):")
PACKAGE_KEY = re.compile(r"(?:'([^'\s]+)'|([^'\s]+)):(?: (\{\}))?")
API_INTEGRITY = re.compile(r"    resolution: \{integrity: sha512-[A-Za-z0-9+/]{86}==\}")


class GateError(ValueError):
    """A bounded, sanitized structural denial."""


def require(condition: bool, reason: str) -> None:
    if not condition:
        raise GateError(reason)


def version(value: object) -> tuple[int, int, int]:
    require(isinstance(value, str), "locked version must be a stable numeric string")
    match = SEMVER.fullmatch(value)
    require(match is not None, "locked version is malformed or unsupported")
    return tuple(int(part) for part in match.groups())


def text_input(path: str, data: bytes) -> str:
    require(isinstance(data, bytes), "input must be bytes")
    require(0 < len(data) <= LIMITS[path], "input is empty or exceeds its byte limit")
    try:
        text = data.decode("utf-8")
    except UnicodeError as error:
        raise GateError("input is not strict UTF-8") from error
    require(not any(ord(char) < 32 and char not in "\r\n\t" for char in text), "input contains unsupported control characters")
    require("\x7f" not in text and "\ufeff" not in text, "input contains an unsupported marker")
    lines = text.split("\n")
    require(len(lines) <= MAX_LINES, "input exceeds its line limit")
    require(all(len(line.encode("utf-8")) <= MAX_LINE_BYTES for line in lines), "input exceeds its line-byte limit")
    require(all("\r" not in line.rstrip("\r") for line in lines), "input has unsupported line endings")
    return text


def unique_json_object(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        require(key not in result, "JSON contains duplicate keys")
        result[key] = value
    return result


def reject_json_constant(_: str) -> object:
    raise GateError("JSON contains a non-finite constant")


def parse_json(text: str) -> dict:
    try:
        value = json.loads(text, object_pairs_hook=unique_json_object, parse_constant=reject_json_constant)
    except (ValueError, RecursionError) as error:
        raise GateError("package manifest JSON is invalid or ambiguous") from error
    require(isinstance(value, dict), "package manifest must be an object")
    return value


def parse_toml(text: str) -> dict:
    try:
        return tomllib.loads(text)
    except (ValueError, RecursionError) as error:
        raise GateError("Cargo TOML is invalid or ambiguous") from error


def api_requirement(manifest: dict) -> str:
    require(isinstance(manifest.get("name"), str) and bool(manifest["name"]), "package manifest lacks a package name")
    occurrences = []
    for key in ("dependencies", "devDependencies", "optionalDependencies", "peerDependencies"):
        entries = manifest.get(key, {})
        require(isinstance(entries, dict), "package dependency section is not an object")
        if API in entries:
            occurrences.append((key, entries[API]))
    require(len(occurrences) == 1 and occurrences[0][0] == "dependencies", "API must occur exactly once in root dependencies")
    value = occurrences[0][1]
    require(isinstance(value, str) and 0 < len(value) <= 512, "API manifest requirement is invalid")
    return value


def validate_cargo_manifest(manifest: dict) -> None:
    package = manifest.get("package")
    require(isinstance(package, dict) and isinstance(package.get("name"), str) and bool(package["name"]), "Cargo manifest lacks a package name")
    dependencies = manifest.get("dependencies")
    require(isinstance(dependencies, dict) and "tauri" in dependencies, "Cargo root manifest lacks tauri")
    dependency = dependencies["tauri"]
    if isinstance(dependency, dict):
        require(not any(key in dependency for key in ("git", "path", "package", "workspace", "registry", "registry-index")), "Cargo tauri source, registry routing or alias is unsupported")
        dependency = dependency.get("version")
    require(isinstance(dependency, str) and 0 < len(dependency) <= 512, "Cargo tauri requirement is invalid")


def yaml_sections(text: str) -> dict[str, list[str]]:
    require(text.endswith("\n") and "\t" not in text, "pnpm lock has unsupported or truncated formatting")
    lines = [line.removesuffix("\r") for line in text.split("\n") if line.removesuffix("\r")]
    require(lines and lines[0] == "lockfileVersion: '9.0'", "unsupported pnpm lockfile version/layout")
    expected = ["settings", "importers", "packages", "snapshots"]
    sections = {}
    section = None
    for line in lines[1:]:
        require(line.rstrip() == line, "pnpm lock has unsupported trailing whitespace")
        if not line.startswith(" "):
            require(len(sections) < len(expected) and line == expected[len(sections)] + ":", "pnpm top-level layout is unknown or duplicated")
            section = expected[len(sections)]
            sections[section] = []
        else:
            require(section is not None, "pnpm body precedes a section")
            indentation = len(line) - len(line.lstrip(" "))
            require(indentation >= 2 and indentation % 2 == 0, "pnpm indentation is unsupported")
            sections[section].append(line)
    require(list(sections) == expected and all(sections.values()), "pnpm lock is missing or has an empty required section")
    require(sections["settings"] == ["  autoInstallPeers: true", "  excludeLinksFromLockfile: false"], "pnpm settings layout is unsupported")
    return sections


def root_importer(lines: list[str]) -> dict[str, dict[str, str]]:
    require(lines[0] == "  .:", "pnpm root importer is missing or unsupported")
    result = {}
    section = None
    dependency = None
    for line in lines[1:]:
        indentation = len(line) - len(line.lstrip(" "))
        value = line[indentation:]
        if indentation == 4:
            require(value in ("dependencies:", "devDependencies:"), "pnpm root dependency section is unsupported")
            section = value[:-1]
            require(section not in result, "pnpm root dependency section is duplicated")
            result[section] = {}
            dependency = None
        elif indentation == 6:
            require(section is not None, "pnpm dependency has no section")
            match = DEPENDENCY_KEY.fullmatch(value)
            require(match is not None, "pnpm dependency key/layout is unsupported")
            dependency = match[1] or match[2]
            require(not any(dependency in entries for entries in result.values()), "pnpm root dependency is duplicated")
            result[section][dependency] = {}
        elif indentation == 8:
            require(section is not None and dependency is not None, "pnpm dependency field has no entry")
            key, separator, scalar = value.partition(": ")
            require(separator == ": " and key in ("specifier", "version") and bool(scalar), "pnpm dependency field/layout is unsupported")
            require(not any(char in scalar for char in "#&*!{}[]\"'"), "pnpm dependency scalar is unsupported")
            fields = result[section][dependency]
            require(key not in fields, "pnpm dependency field is duplicated")
            fields[key] = scalar
        else:
            raise GateError("pnpm importer layout is unknown or duplicated")
    require("dependencies" in result and all(result.values()), "pnpm root importer lacks dependency entries")
    require(all(set(fields) == {"specifier", "version"} for entries in result.values() for fields in entries.values()), "pnpm dependency entry is incomplete")
    return result


def package_records(lines: list[str]) -> dict[str, tuple[bool, list[str]]]:
    records = {}
    current = None
    for line in lines:
        indentation = len(line) - len(line.lstrip(" "))
        if indentation == 2:
            match = PACKAGE_KEY.fullmatch(line[2:])
            require(match is not None, "pnpm package key/layout is unsupported")
            current = match[1] or match[2]
            require(current not in records, "pnpm package entry is duplicated")
            records[current] = (match[3] is not None, [])
        else:
            require(current is not None and not records[current][0], "pnpm package body/layout is unsupported")
            records[current][1].append(line)
    return records


def locked_api(text: str, requirement: str) -> str:
    sections = yaml_sections(text)
    importer = root_importer(sections["importers"])
    require(API in importer["dependencies"], "pnpm root importer lacks API")
    entry = importer["dependencies"][API]
    require(entry["specifier"] == requirement, "API importer specifier differs from root manifest")
    selected = entry["version"]
    version(selected)
    name = API + "@" + selected
    packages = package_records(sections["packages"])
    snapshots = package_records(sections["snapshots"])
    require(name in packages and name in snapshots, "API locked package/snapshot is missing")
    empty, body = packages[name]
    require(not empty and len(body) == 1 and API_INTEGRITY.fullmatch(body[0]) is not None, "API package resolution layout is unsupported")
    require(snapshots[name] == (True, []), "API snapshot layout is unsupported")
    return selected


def locked_rust(lock: dict) -> str:
    require(set(lock) == {"version", "package"}, "Cargo lock top-level layout is unsupported")
    require(type(lock.get("version")) is int and lock["version"] == 4, "Cargo lockfile version is unsupported")
    packages = lock.get("package")
    require(isinstance(packages, list) and 0 < len(packages) <= 4096, "Cargo lock package list is invalid or exceeds its limit")
    selected = []
    identities = set()
    for package in packages:
        require(isinstance(package, dict), "Cargo lock package entry is invalid")
        require(set(package).issubset({"name", "version", "source", "checksum", "dependencies"}), "Cargo lock package layout is unsupported")
        name, resolved = package.get("name"), package.get("version")
        require(isinstance(name, str) and 0 < len(name) <= 128 and isinstance(resolved, str) and 0 < len(resolved) <= 256, "Cargo lock package identity is incomplete or exceeds its limit")
        source = package.get("source", "")
        require(isinstance(source, str) and len(source) <= 2048, "Cargo lock source is invalid or exceeds its limit")
        dependencies = package.get("dependencies", [])
        require(isinstance(dependencies, list) and len(dependencies) <= 4096, "Cargo lock dependency list is invalid or exceeds its limit")
        require(all(isinstance(item, str) and 0 < len(item) <= 2048 for item in dependencies), "Cargo lock dependency identity is invalid or exceeds its limit")
        require(len(set(dependencies)) == len(dependencies), "Cargo lock dependency identity is duplicated")
        identity = (name, resolved, source)
        require(identity not in identities, "Cargo lock package identity is duplicated")
        identities.add(identity)
        if name == "tauri":
            require(source == "registry+https://github.com/rust-lang/crates.io-index", "Cargo tauri source is unsupported")
            checksum = package.get("checksum")
            require(isinstance(checksum, str) and re.fullmatch(r"[0-9a-f]{64}", checksum) is not None, "Cargo tauri checksum is invalid")
            selected.append(resolved)
    require(len(selected) == 1, "Cargo lock must resolve exactly one tauri package")
    version(selected[0])
    return selected[0]


def check_inputs(inputs: dict[str, bytes]) -> tuple[str, str]:
    require(set(inputs) == set(LIMITS), "fixed input inventory is incomplete or unexpected")
    texts = {path: text_input(path, inputs[path]) for path in LIMITS}
    requirement = api_requirement(parse_json(texts["package.json"]))
    validate_cargo_manifest(parse_toml(texts["src-tauri/Cargo.toml"]))
    api = locked_api(texts["pnpm-lock.yaml"], requirement)
    rust = locked_rust(parse_toml(texts["src-tauri/Cargo.lock"]))
    require(version(api)[:2] == version(rust)[:2], "locked Rust/npm Tauri major/minor mismatch")
    return rust, api


def read_inputs(root: Path) -> dict[str, bytes]:
    root = root.resolve(strict=True)
    require(root.is_dir(), "root is not a directory")
    inputs = {}
    for relative, limit in LIMITS.items():
        path = root / relative
        require(not path.is_symlink() and not path.is_junction(), "fixed input is a link or junction")
        resolved = path.resolve(strict=True)
        require(resolved.is_relative_to(root), "fixed input escapes the root")
        require(stat.S_ISREG(resolved.stat().st_mode), "fixed input is not a regular file")
        with resolved.open("rb") as handle:
            require(stat.S_ISREG(os.fstat(handle.fileno()).st_mode), "fixed input is not a regular file")
            inputs[relative] = handle.read(limit + 1)
    return inputs


def fixture(api: str = "2.12.0", rust: str = "2.12.1") -> dict[str, bytes]:
    yaml = (
        "lockfileVersion: '9.0'\n\nsettings:\n  autoInstallPeers: true\n  excludeLinksFromLockfile: false\n"
        "\nimporters:\n  .:\n    dependencies:\n      '@tauri-apps/api':\n"
        f"        specifier: ^2.0.0\n        version: {api}\n\npackages:\n  '@tauri-apps/api@{api}':\n"
        "    resolution: {integrity: sha512-" + "A" * 86 + "==}\n"
        f"\nsnapshots:\n  '@tauri-apps/api@{api}': {{}}\n"
    )
    return {
        "package.json": b'{"name":"fixture","dependencies":{"@tauri-apps/api":"^2.0.0"}}',
        "pnpm-lock.yaml": yaml.encode(),
        "src-tauri/Cargo.toml": b'[package]\nname = "fixture"\n[dependencies]\ntauri = {version="2"}\n',
        "src-tauri/Cargo.lock": (
            f'version = 4\n[[package]]\nname = "tauri"\nversion = "{rust}"\n'
            'source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "' + "0" * 64 + '"\n'
        ).encode(),
    }


def self_test() -> int:
    count = 0

    def run(inputs: dict[str, bytes], passes: bool) -> None:
        nonlocal count
        try:
            check_inputs(inputs)
        except GateError:
            require(not passes, "self-test rejected an aligned fixture")
        else:
            require(passes, "self-test admitted a denied fixture")
        count += 1

    for api, rust in (("2.11.1", "2.11.6"), ("2.12.0", "2.12.1"), ("2.12.1", "2.12.0"), ("3.0.0", "3.0.9")):
        run(fixture(api, rust), True)
    crlf = fixture()
    crlf["pnpm-lock.yaml"] = crlf["pnpm-lock.yaml"].replace(b"\n", b"\r\n")
    run(crlf, True)
    for api, rust in (("2.12.0", "2.11.6"), ("2.11.1", "2.12.1"), ("3.12.0", "2.12.1"), ("2.012.0", "2.12.1"), ("2.12", "2.12.1"), ("2.12.0(peer@1.0.0)", "2.12.1"), ("2.12.0-beta.1", "2.12.1"), ("2.12.0", "2.12.1+meta")):
        run(fixture(api, rust), False)
    mutations = [
        ("package.json", b'{"dependencies":{"@tauri-apps/api":"^2"}}'),
        ("package.json", b'{"name":"fixture","dependencies":{}}'),
        ("package.json", b'{"name":"fixture","dependencies":{"@tauri-apps/api":"^2"},"devDependencies":{"@tauri-apps/api":"^2"}}'),
        ("package.json", b'{"name":"fixture","name":"duplicate"}'),
        ("package.json", b'{"name":"fixture","dependencies":{"@tauri-apps/api":"^2.0.0","@tauri-apps/api":"^2.0.0"}}'),
        ("package.json", b'{"name":"fixture","value":NaN}'),
        ("package.json", b'{'),
        ("package.json", b'[]'),
        ("src-tauri/Cargo.toml", b'[package]\nname="fixture"\n[dependencies]\n'),
        ("src-tauri/Cargo.toml", b'[package]\nname="fixture"\n[dependencies]\ntauri={path="elsewhere",version="2"}\n'),
        ("src-tauri/Cargo.toml", b'[package]\nname="fixture"\n[dependencies]\ntauri={version="2",registry="alternate"}\n'),
        ("src-tauri/Cargo.toml", b'[package]\nname="fixture"\n[dependencies]\ntauri={version="2",registry-index="https://alternate.invalid/index"}\n'),
        ("src-tauri/Cargo.toml", b'[package]\nname="fixture"\nname="duplicate"\n'),
        ("src-tauri/Cargo.lock", b'version=4\npackage=[]\n'),
        ("src-tauri/Cargo.lock", b'version=4\n[[package]]\nname="tauri"\nversion="2.12.1"\n'),
        ("src-tauri/Cargo.lock", b'version=4\n[[package]\n'),
        ("src-tauri/Cargo.lock", fixture()["src-tauri/Cargo.lock"] + b'unknown="unsupported"\n'),
        ("src-tauri/Cargo.lock", fixture()["src-tauri/Cargo.lock"] + b'dependencies="not a list"\n'),
        ("src-tauri/Cargo.lock", fixture()["src-tauri/Cargo.lock"] + b'dependencies=["x","x"]\n'),
    ]
    for path, data in mutations:
        changed = fixture()
        changed[path] = data
        run(changed, False)
    replacements = [
        (b"lockfileVersion: '9.0'", b"lockfileVersion: '10.0'"),
        (b"importers:\n", b"importers: {}\n"),
        (b"  .:\n", b"  other:\n"),
        (b"  .:\n", b"  .:\n  .:\n"),
        (b"    dependencies:\n", b"    dependencies:\n    dependencies:\n"),
        (b"      '@tauri-apps/api':\n", b"      other:\n"),
        (b"        version: 2.12.0\n", b"        version: 2.12.0\n        version: 2.12.0\n"),
        (b"        version: 2.12.0\n", b""),
        (b"        version: 2.12.0\n", b"        version: '2.12.0'\n"),
        (b"        specifier: ^2.0.0\n", b"        specifier: ^2.12.0\n"),
        (b"      '@tauri-apps/api':\n", b"      '@tauri-apps/api':\n        specifier: ^2.0.0\n        version: 2.12.0\n      '@tauri-apps/api':\n"),
        (b"snapshots:\n", b"snapshots:\nunknown:\n"),
        (b"  '@tauri-apps/api@2.12.0': {}\n", b"  other@1.0.0: {}\n"),
        (b"  '@tauri-apps/api@2.12.0': {}\n", b"  '@tauri-apps/api@2.12.0': {}\n  '@tauri-apps/api@2.12.0': {}\n"),
        (b"    resolution: {integrity: sha512-", b"    resolution: {tarball: sha512-"),
        (b"        version: 2.12.0\n", b"        version: &alias 2.12.0\n"),
        (b"  .:\n", b"\t.:\n"),
    ]
    for before, after in replacements:
        changed = fixture()
        require(before in changed["pnpm-lock.yaml"], "self-test mutation anchor is absent")
        changed["pnpm-lock.yaml"] = changed["pnpm-lock.yaml"].replace(before, after)
        run(changed, False)
    for change in (b"[[package]]\nname = \"tauri\"\nversion = \"2.13.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"" + b"0" * 64 + b"\"\n", fixture()["src-tauri/Cargo.lock"].split(b"[[package]]", 1)[1]):
        changed = fixture()
        changed["src-tauri/Cargo.lock"] += change if change.startswith(b"[[package]]") else b"[[package]]" + change
        run(changed, False)
    for path, limit in LIMITS.items():
        for data in (b"", b" " * (limit + 1), b"\xff", b"\x00"):
            changed = fixture()
            changed[path] = data
            run(changed, False)
    for data in (fixture()["pnpm-lock.yaml"].split(b"\nsnapshots:")[0] + b"\n", fixture()["pnpm-lock.yaml"].rstrip(b"\n"), b"\n" * (MAX_LINES + 1), b"x" * (MAX_LINE_BYTES + 1) + b"\n"):
        changed = fixture()
        changed["pnpm-lock.yaml"] = data
        run(changed, False)
    missing = fixture()
    missing.pop("src-tauri/Cargo.toml")
    run(missing, False)
    unexpected = fixture()
    unexpected["extra"] = b"not read"
    run(unexpected, False)
    print(f"T-037 self-test PASS: {count} pure structural fixtures; no filesystem/process/network/write operations")
    return count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, help="repository root; only four fixed inputs are read")
    parser.add_argument("--self-test", action="store_true", help="run pure in-memory fixtures before any selected root check")
    args = parser.parse_args()
    if not args.self_test and args.root is None:
        parser.error("provide --self-test and/or --root")
    try:
        if args.self_test:
            self_test()
        if args.root is not None:
            rust, api = check_inputs(read_inputs(args.root))
            print(f"T-037 source compatibility PASS: tauri={rust}; {API}={api}; matching major/minor only")
    except (GateError, OSError, RuntimeError) as error:
        reason = str(error) if isinstance(error, GateError) else "fixed inputs could not be read safely"
        print("T-037 FAIL: " + reason, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
