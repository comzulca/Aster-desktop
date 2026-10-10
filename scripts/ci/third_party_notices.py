#!/usr/bin/env python3
"""Build a bounded notice inventory from exact installed locked source trees.

The inventory deliberately distinguishes every installed build/source package
from the target-filtered normal dependency closure used by the packaged app.
It is evidence for review, not legal advice, a binary SBOM, or legal approval.
"""
from __future__ import annotations

from collections import deque
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
import tempfile
from typing import Any

NOTICE_NAME = re.compile(r"^(licen[cs]e|copying|notice)([._-].*)?$", re.I)
MAX_TEXT = 2 * 1024 * 1024
MAX_JSON = 64 * 1024 * 1024
MAX_TOTAL = 32 * 1024 * 1024
REVIEWED_TARGET = "x86_64-pc-windows-msvc"
OVERLAY_FIELDS = {
    "ecosystem",
    "name",
    "version",
    "declaredLicense",
    "repository",
    "vcsSha1",
    "pathInVcs",
    "noticeFile",
    "noticeByteCount",
    "noticeSha256",
    "originKind",
    "sourceUrl",
    "packageSourceAccessUrl",
    "upstreamRepositoryOriginal",
}
EXPECTED_OVERLAYS = {
    ("cargo", "selectors", "0.38.0"): {
        "ecosystem": "cargo",
        "name": "selectors",
        "version": "0.38.0",
        "declaredLicense": "MPL-2.0",
        "repository": "github.com/servo/stylo",
        "vcsSha1": "572ecba2d1600e7c3d490586692a209faf703baa",
        "pathInVcs": "selectors",
        "noticeFile": "Mozilla-MPL-2.0.txt",
        "noticeByteCount": 16726,
        "noticeSha256": "3f3d9e0024b1921b067d6f7f88deb4a60cbe7a78e76c64e3f1d7fc3b779b9d04",
        "originKind": "canonical-license-steward",
        "sourceUrl": "https://www.mozilla.org/media/MPL/2.0/index.txt",
        "packageSourceAccessUrl": "https://github.com/servo/stylo/tree/572ecba2d1600e7c3d490586692a209faf703baa/selectors",
        "upstreamRepositoryOriginal": False,
    },
    ("cargo", "webview2-com", "0.39.1"): {
        "ecosystem": "cargo",
        "name": "webview2-com",
        "version": "0.39.1",
        "declaredLicense": "MIT",
        "repository": "github.com/wravery/webview2-rs",
        "vcsSha1": "edc2caf886175ccaebe86078c9cfe1ae2a187328",
        "pathInVcs": "crates/webview2-com",
        "noticeFile": "webview2-rs-MIT.txt",
        "noticeByteCount": 1067,
        "noticeSha256": "0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73",
        "originKind": "upstream-repository-original",
        "sourceUrl": "https://raw.githubusercontent.com/wravery/webview2-rs/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE",
        "packageSourceAccessUrl": "https://github.com/wravery/webview2-rs/tree/edc2caf886175ccaebe86078c9cfe1ae2a187328/crates/webview2-com",
        "upstreamRepositoryOriginal": True,
    },
    ("cargo", "webview2-com-macros", "0.8.1"): {
        "ecosystem": "cargo",
        "name": "webview2-com-macros",
        "version": "0.8.1",
        "declaredLicense": "MIT",
        "repository": "github.com/wravery/webview2-rs",
        "vcsSha1": "dffa41a8a46d3f5565eefbff2de57d38d399f158",
        "pathInVcs": "crates/callback-macros",
        "noticeFile": "webview2-rs-MIT.txt",
        "noticeByteCount": 1067,
        "noticeSha256": "0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73",
        "originKind": "upstream-repository-original",
        "sourceUrl": "https://raw.githubusercontent.com/wravery/webview2-rs/dffa41a8a46d3f5565eefbff2de57d38d399f158/LICENSE",
        "packageSourceAccessUrl": "https://github.com/wravery/webview2-rs/tree/dffa41a8a46d3f5565eefbff2de57d38d399f158/crates/callback-macros",
        "upstreamRepositoryOriginal": True,
    },
    ("cargo", "webview2-com-sys", "0.39.1"): {
        "ecosystem": "cargo",
        "name": "webview2-com-sys",
        "version": "0.39.1",
        "declaredLicense": "MIT",
        "repository": "github.com/wravery/webview2-rs",
        "vcsSha1": "edc2caf886175ccaebe86078c9cfe1ae2a187328",
        "pathInVcs": "crates/bindings",
        "noticeFile": "webview2-rs-MIT.txt",
        "noticeByteCount": 1067,
        "noticeSha256": "0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73",
        "originKind": "upstream-repository-original",
        "sourceUrl": "https://raw.githubusercontent.com/wravery/webview2-rs/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE",
        "packageSourceAccessUrl": "https://github.com/wravery/webview2-rs/tree/edc2caf886175ccaebe86078c9cfe1ae2a187328/crates/bindings",
        "upstreamRepositoryOriginal": True,
    },
}


def is_link(path: Path) -> bool:
    return path.is_symlink() or (hasattr(path, "is_junction") and path.is_junction())


def read_regular(path: Path, root: Path, limit: int) -> bytes:
    root = root.resolve(strict=True)
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or is_link(path) or info.st_size > limit:
        raise ValueError("Input is not a bounded regular file")
    resolved = path.resolve(strict=True)
    if not resolved.is_relative_to(root):
        raise ValueError("Input escaped its approved root")
    descriptor = os.open(resolved, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
    with os.fdopen(descriptor, "rb") as stream:
        before = os.fstat(stream.fileno())
        if (before.st_dev, before.st_ino, before.st_size) != (
            info.st_dev,
            info.st_ino,
            info.st_size,
        ):
            raise ValueError("Input identity changed")
        data = stream.read(limit + 1)
        after = os.fstat(stream.fileno())
        if len(data) > limit or (before.st_size, before.st_mtime_ns) != (
            after.st_size,
            after.st_mtime_ns,
        ):
            raise ValueError("Input changed or exceeds its bound")
    return data


def read_json(path: Path, root: Path, limit: int = MAX_JSON) -> dict[str, Any]:
    value = json.loads(read_regular(path, root, limit).decode("utf-8-sig"))
    if not isinstance(value, dict):
        raise ValueError("JSON root must be an object")
    return value


def write_output(path: Path, data: bytes) -> None:
    parent = path.parent.resolve(strict=True)
    if is_link(parent) or not parent.is_dir():
        raise ValueError("Output parent is not a normal directory")
    if path.exists() or path.is_symlink():
        info = path.lstat()
        if not stat.S_ISREG(info.st_mode) or is_link(path):
            raise ValueError("Output path is not a regular file")
    descriptor = os.open(
        path,
        os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_NOFOLLOW", 0),
        0o600,
    )
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def collect(root: Path) -> list[tuple[str, str]]:
    root = root.resolve(strict=True)
    texts: list[tuple[str, str]] = []
    for child in sorted(root.iterdir()):
        if child.is_file() and NOTICE_NAME.fullmatch(child.name):
            text = read_regular(child, root, MAX_TEXT).decode("utf-8-sig")
            texts.append((child.name, text))
        elif child.is_dir() and NOTICE_NAME.fullmatch(child.name):
            if is_link(child) or not child.resolve(strict=True).is_relative_to(root):
                raise ValueError("Notice directory link rejected")
            for file in sorted(child.glob("*")):
                if file.is_file():
                    text = read_regular(file, root, MAX_TEXT).decode("utf-8-sig")
                    texts.append((f"{child.name}/{file.name}", text))
    return texts


def load_reviewed_overlays(project: Path) -> tuple[dict[tuple[str, str, str], dict[str, Any]], str, int]:
    overlay_root_path = project / "scripts" / "ci" / "notices"
    if is_link(overlay_root_path):
        raise ValueError("Notice overlay root link rejected")
    overlay_root = overlay_root_path.resolve(strict=True)
    manifest_path = overlay_root / "manifest.json"
    manifest_bytes = read_regular(manifest_path, overlay_root, MAX_TEXT)
    manifest = json.loads(manifest_bytes.decode("utf-8-sig"))
    if not isinstance(manifest, dict) or set(manifest) != {
        "schemaVersion",
        "classification",
        "legalConclusion",
        "entries",
    }:
        raise ValueError("Notice overlay manifest shape is not exact")
    if (
        manifest["schemaVersion"] != 1
        or manifest["classification"] != "reviewed-pinned-source-notice-overlays"
        or not str(manifest["legalConclusion"]).startswith("UNASSESSED")
        or not isinstance(manifest["entries"], list)
    ):
        raise ValueError("Notice overlay manifest header is invalid")
    overlays: dict[tuple[str, str, str], dict[str, Any]] = {}
    referenced_files = {"README.md", "manifest.json"}
    for entry in manifest["entries"]:
        if not isinstance(entry, dict) or set(entry) != OVERLAY_FIELDS:
            raise ValueError("Notice overlay entry shape is not exact")
        key = (entry["ecosystem"], entry["name"], entry["version"])
        if key in overlays or key not in EXPECTED_OVERLAYS or entry != EXPECTED_OVERLAYS[key]:
            raise ValueError("Notice overlay entry is duplicate, unknown, or changed")
        notice_file = entry["noticeFile"]
        if not isinstance(notice_file, str) or Path(notice_file).name != notice_file:
            raise ValueError("Notice overlay filename is invalid")
        raw = read_regular(overlay_root / notice_file, overlay_root, MAX_TEXT)
        if len(raw) != entry["noticeByteCount"] or hashlib.sha256(raw).hexdigest() != entry["noticeSha256"]:
            raise ValueError("Notice overlay bytes do not match their reviewed size/hash")
        entry_with_text = dict(entry)
        entry_with_text["text"] = raw.decode("utf-8-sig")
        overlays[key] = entry_with_text
        referenced_files.add(notice_file)
    if set(overlays) != set(EXPECTED_OVERLAYS):
        raise ValueError("Notice overlay manifest is incomplete")
    children = list(overlay_root.iterdir())
    if any(not child.is_file() or is_link(child) for child in children):
        raise ValueError("Notice overlay directory contains a non-regular entry")
    actual_files = {child.name for child in children}
    if actual_files != referenced_files:
        raise ValueError("Notice overlay directory contains an unreviewed or missing file")
    read_regular(overlay_root / "README.md", overlay_root, MAX_TEXT).decode("utf-8-sig")
    return overlays, hashlib.sha256(manifest_bytes).hexdigest(), len(manifest_bytes)


def bind_reviewed_overlay(
    package: dict[str, Any], overlay: dict[str, Any]
) -> tuple[str, str, str]:
    if (
        normalized_license(package["license"]) != normalized_license(overlay["declaredLicense"])
        or package["repository"] != overlay["repository"]
        or package["ecosystem"] != "cargo"
    ):
        raise ValueError("Notice overlay package license/repository identity changed")
    vcs = read_json(package["directory"] / ".cargo_vcs_info.json", package["directory"], MAX_TEXT)
    git = vcs.get("git")
    if (
        not isinstance(git, dict)
        or git.get("sha1") != overlay["vcsSha1"]
        or vcs.get("path_in_vcs") != overlay["pathInVcs"]
    ):
        raise ValueError("Notice overlay package VCS provenance changed")
    return (
        overlay["noticeFile"],
        overlay["text"],
        f"reviewed-overlay:{overlay['originKind']}:{overlay['noticeFile']}",
    )


def normalized_license(value: Any) -> str:
    if isinstance(value, dict):
        value = value.get("type")
    return re.sub(r"\s+", "", str(value or "UNKNOWN")).upper()


def repository_value(value: Any) -> str:
    if isinstance(value, dict):
        value = value.get("url")
    text = str(value or "").strip().lower()
    for prefix in ("git+", "git://", "https://", "http://", "ssh://git@", "git@"):
        if text.startswith(prefix):
            text = text[len(prefix) :]
    text = text.replace("github.com:", "github.com/").rstrip("/")
    if text.endswith(".git"):
        text = text[:-4]
    return text


def cargo_required_ids(metadata: dict[str, Any]) -> set[str]:
    resolve = metadata.get("resolve")
    if not isinstance(resolve, dict) or not isinstance(resolve.get("root"), str):
        raise ValueError("Cargo metadata has no resolved root")
    nodes = {node["id"]: node for node in resolve.get("nodes", [])}
    required: set[str] = set()
    queue = deque([resolve["root"]])
    while queue:
        package_id = queue.popleft()
        if package_id in required:
            continue
        required.add(package_id)
        node = nodes.get(package_id)
        if not isinstance(node, dict):
            raise ValueError("Cargo resolve graph references an unknown package")
        for dependency in node.get("deps", []):
            dep_kinds = dependency.get("dep_kinds", [])
            if any(kind.get("kind") is None for kind in dep_kinds):
                queue.append(dependency["pkg"])
    return required


def npm_required_roots(project: Path, node_base: Path) -> set[Path]:
    required: set[Path] = set()
    root_manifest = read_json(project / "package.json", project, MAX_TEXT)
    queue: deque[tuple[Path, dict[str, Any], bool]] = deque([(project, root_manifest, True)])
    while queue:
        directory, manifest, is_project_root = queue.popleft()
        dependencies = manifest.get("dependencies") or {}
        optional = manifest.get("optionalDependencies") or {}
        if not isinstance(dependencies, dict) or not isinstance(optional, dict):
            raise ValueError("npm dependency maps must be objects")
        names = sorted(set(dependencies) | set(optional))
        for name in names:
            candidate = (directory / "node_modules" / name) if is_project_root else (directory.parent / name)
            try:
                resolved = candidate.resolve(strict=True)
            except OSError:
                if name in optional:
                    continue
                raise ValueError(f"Installed production npm dependency is missing: {name}")
            if not resolved.is_relative_to(node_base):
                raise ValueError("Installed npm dependency escaped the pnpm store")
            if resolved in required:
                continue
            package_manifest = read_json(resolved / "package.json", resolved, MAX_TEXT)
            required.add(resolved)
            queue.append((resolved, package_manifest, False))
    return required


def npm_installed_packages(project: Path) -> tuple[list[dict[str, Any]], set[Path]]:
    node_base_path = project / "node_modules" / ".pnpm"
    if is_link(node_base_path):
        raise ValueError("pnpm store root link rejected")
    node_base = node_base_path.resolve(strict=True)
    required = npm_required_roots(project, node_base)
    packages: list[dict[str, Any]] = []
    seen: set[Path] = set()
    for store in sorted(node_base.iterdir()):
        module = store / "node_modules"
        if not module.is_dir() or is_link(module):
            continue
        for child in sorted(module.iterdir()):
            candidates = sorted(child.iterdir()) if child.name.startswith("@") and child.is_dir() else [child]
            for candidate in candidates:
                resolved = candidate.resolve(strict=True)
                if (
                    not resolved.is_relative_to(node_base)
                    or resolved in seen
                    or not (resolved / "package.json").is_file()
                ):
                    continue
                seen.add(resolved)
                manifest = read_json(resolved / "package.json", resolved, MAX_TEXT)
                name = manifest.get("name")
                version = manifest.get("version")
                if not isinstance(name, str) or not isinstance(version, str):
                    raise ValueError("Installed npm package lacks name/version")
                packages.append(
                    {
                        "ecosystem": "npm",
                        "name": name,
                        "version": version,
                        "license": str(manifest.get("license") or "UNKNOWN"),
                        "repository": repository_value(manifest.get("repository")),
                        "directory": resolved,
                        "source": f"https://www.npmjs.com/package/{name}/v/{version}",
                        "required": resolved in required,
                    }
                )
    if required - seen:
        raise ValueError("Production npm dependency was not inventoried")
    return packages, required


def cargo_source_identity(package: dict[str, Any], root_id: str, project: Path) -> str | None:
    source = package.get("source")
    if source:
        return str(source)
    if package["id"] == root_id:
        return None
    # A local patch is third-party source too: never omit it merely because
    # Cargo's registry source/checksum is absent. Admit only the reviewed patch.
    expected = project / "src-tauri" / "vendor" / "zune-jpeg-0.5.15"
    directory = Path(package["manifest_path"]).parent
    if (
        package["name"] != "zune-jpeg"
        or package["version"] != "0.5.15"
        or is_link(directory)
        or any(is_link(ancestor) for ancestor in (
            project / "src-tauri", project / "src-tauri" / "vendor", expected))
        or directory.resolve(strict=True) != expected.resolve(strict=True)
    ):
        raise ValueError("Unreviewed non-root Cargo path dependency")
    read_regular(directory / "Cargo.toml", expected, MAX_TEXT)
    return "locally-modified-vendored:src-tauri/vendor/zune-jpeg-0.5.15; see ASTER-PROVENANCE.json"


def source_packages(
    metadata: dict[str, Any], required_metadata: dict[str, Any], project: Path
) -> list[dict[str, Any]]:
    required_cargo = cargo_required_ids(required_metadata)
    root_id = metadata.get("resolve", {}).get("root")
    if not isinstance(root_id, str):
        raise ValueError("Cargo inventory has no exact application root")
    packages: list[dict[str, Any]] = []
    for package in metadata.get("packages", []):
        source = cargo_source_identity(package, root_id, project)
        if source is not None:
            directory = Path(package["manifest_path"]).parent.resolve(strict=True)
            packages.append(
                {
                    "ecosystem": "cargo",
                    "name": package["name"],
                    "version": package["version"],
                    "license": package.get("license") or "UNKNOWN",
                    "repository": repository_value(package.get("repository")),
                    "directory": directory,
                    "source": source,
                    "required": package["id"] in required_cargo,
                }
            )
    npm_packages, _ = npm_installed_packages(project)
    packages.extend(npm_packages)
    if not packages:
        raise ValueError("Installed source inventory is empty")
    return sorted(packages, key=lambda item: (item["ecosystem"], item["name"], item["version"], item["source"]))


def generate(
    metadata: dict[str, Any], required_metadata: dict[str, Any], project: Path, target: str
) -> tuple[bytes, dict[str, Any]]:
    if target != REVIEWED_TARGET:
        raise ValueError("Target is not the reviewed Windows packaging target")
    packages = source_packages(metadata, required_metadata, project)
    overlays, overlay_manifest_sha256, overlay_manifest_byte_count = load_reviewed_overlays(project)
    direct: dict[int, list[tuple[str, str, str]]] = {}
    for index, package in enumerate(packages):
        direct[index] = [(name, text, "package-root") for name, text in collect(package["directory"])]

    # Some workspaces publish one canonical license file at the repository root
    # rather than duplicating it into every crate/package. Reuse only an exact
    # installed peer from the same ecosystem, normalized repository and declared
    # license. The donor and hash remain explicit in the evidence.
    selected: dict[int, list[tuple[str, str, str]]] = {}
    used_overlays: dict[tuple[str, str, str], int] = {}
    for index, package in enumerate(packages):
        texts = direct[index]
        if not texts and package["repository"] and normalized_license(package["license"]) != "UNKNOWN":
            for donor_index, donor in enumerate(packages):
                if (
                    donor_index != index
                    and donor["ecosystem"] == package["ecosystem"]
                    and donor["repository"] == package["repository"]
                    and normalized_license(donor["license"]) == normalized_license(package["license"])
                    and direct[donor_index]
                ):
                    texts = [
                        (
                            name,
                            text,
                            f"installed-repository-peer:{donor['name']}@{donor['version']}/{name}",
                        )
                        for name, text, _ in direct[donor_index]
                    ]
                    break
        overlay_key = (package["ecosystem"], package["name"], package["version"])
        if not texts and overlay_key in overlays:
            overlay = overlays[overlay_key]
            texts = [bind_reviewed_overlay(package, overlay)]
            used_overlays[overlay_key] = used_overlays.get(overlay_key, 0) + 1
        selected[index] = texts
    if set(used_overlays) != set(overlays) or any(count != 1 for count in used_overlays.values()):
        raise ValueError("Every reviewed notice overlay must bind exactly one installed locked package")

    root_id = metadata["resolve"]["root"]
    app = next((package for package in metadata["packages"] if package["id"] == root_id), None)
    if not app or app["name"] != "aster-desktop" or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", app["version"]):
        raise ValueError("Notice title has no valid exact application version")
    sections = [
        f"Aster {app['version']} — third-party source notices\n\n",
        "Generated without network access from exact installed locked Rust and npm source packages. "
        "The complete inventory includes build/development sources; requiredForPackagedApp marks the "
        f"normal dependency closure filtered for {target} plus installed production npm dependencies. "
        "Reviewed overlays are pinned to exact package/VCS/file hashes; canonical-license-steward "
        "texts are labeled separately from upstream-repository originals. "
        "This evidence is not a binary SBOM, legal advice, or legal approval.\n",
    ]
    records: list[dict[str, Any]] = []
    for index, package in enumerate(packages):
        texts = selected[index]
        text_records = []
        sections.append(
            f"\n{'=' * 72}\n{package['ecosystem']}: {package['name']} {package['version']}\n"
            f"Declared license: {package['license']}\nSource identity/page: {package['source']}\n"
            f"Required for packaged app graph: {'yes' if package['required'] else 'no'}\n"
        )
        if not texts:
            sections.append(
                "No standalone official notice text was found in the installed package or an exact "
                "installed repository/license peer; source review remains required.\n"
            )
        for filename, text, origin in texts:
            encoded = text.encode("utf-8")
            text_records.append(
                {
                    "name": filename,
                    "sha256": hashlib.sha256(encoded).hexdigest(),
                    "byteCount": len(encoded),
                    "origin": origin,
                }
            )
            sections.append(f"\n--- {filename} ({origin}) ---\n{text}\n")
        records.append(
            {
                "ecosystem": package["ecosystem"],
                "name": package["name"],
                "version": package["version"],
                "declaredLicense": package["license"],
                "repository": package["repository"] or None,
                "source": package["source"],
                "requiredForPackagedApp": bool(package["required"]),
                "texts": text_records,
            }
        )

    body = "".join(sections).encode("utf-8")
    if len(body) > MAX_TOTAL:
        raise ValueError("Notice inventory exceeds its bound")
    required_missing = [
        f"{record['ecosystem']}:{record['name']}@{record['version']}"
        for record in records
        if record["requiredForPackagedApp"] and not record["texts"]
    ]
    required_unknown = [
        f"{record['ecosystem']}:{record['name']}@{record['version']}"
        for record in records
        if record["requiredForPackagedApp"] and normalized_license(record["declaredLicense"]) == "UNKNOWN"
    ]
    non_required_missing = [
        f"{record['ecosystem']}:{record['name']}@{record['version']}"
        for record in records
        if not record["requiredForPackagedApp"] and not record["texts"]
    ]
    report = {
        "schemaVersion": 2,
        "classification": "installed-locked-source-notice-inventory",
        "cargoFilterPlatform": target,
        "sourceInventoryScope": "all installed locked Cargo and pnpm package source trees from unfiltered metadata, including other-target/build/development packages",
        "packagedAppRequirementScope": "Cargo normal dependency closure after target filtering plus installed production npm dependency closure",
        "payloadEvidenceScope": "this source notice inventory proves no installer payload; separate exact manifest-bound CI extraction and passive verification are required for the declared NSIS resources and their byte identities",
        "legalConclusion": "UNASSESSED — this inventory is not legal advice or legal approval",
        "overlayManifestSha256": overlay_manifest_sha256,
        "overlayManifestByteCount": overlay_manifest_byte_count,
        "reviewedOverlays": [dict(EXPECTED_OVERLAYS[key]) for key in sorted(EXPECTED_OVERLAYS)],
        "packageCount": len(records),
        "requiredPackageCount": sum(record["requiredForPackagedApp"] for record in records),
        "directMissingTextCount": sum(not direct[index] for index in range(len(records))),
        "repositoryPeerCoveredCount": sum(
            not direct[index]
            and bool(selected[index])
            and selected[index][0][2].startswith("installed-repository-peer:")
            for index in range(len(records))
        ),
        "reviewedOverlayCoveredCount": sum(
            not direct[index]
            and bool(selected[index])
            and selected[index][0][2].startswith("reviewed-overlay:")
            for index in range(len(records))
        ),
        "canonicalLicenseStewardCoveredCount": sum(
            not direct[index]
            and bool(selected[index])
            and selected[index][0][2].startswith("reviewed-overlay:canonical-license-steward:")
            for index in range(len(records))
        ),
        "missingTextCount": sum(not record["texts"] for record in records),
        "requiredMissingTextCount": len(required_missing),
        "nonRequiredMissingTextCount": len(non_required_missing),
        "requiredUnknownLicenseCount": len(required_unknown),
        "unresolvedRequiredPackages": required_missing,
        "unresolvedNonRequiredPackages": non_required_missing,
        "unknownLicenseRequiredPackages": required_unknown,
        "noticeByteCount": len(body),
        "noticeSha256": hashlib.sha256(body).hexdigest(),
        "packages": records,
    }
    return body, report


def has_blocking_gaps(report: dict[str, Any]) -> bool:
    return bool(report["requiredMissingTextCount"] or report["requiredUnknownLicenseCount"])


def self_test() -> None:
    def must_fail(label: str, operation: Any) -> None:
        try:
            operation()
        except (OSError, ValueError, KeyError, TypeError, UnicodeError):
            return
        raise AssertionError(f"negative fixture did not fail: {label}")

    source_overlays = Path(__file__).resolve(strict=True).parent / "notices"
    with tempfile.TemporaryDirectory(prefix="aster-notice-self-test-") as temporary:
        root = Path(temporary)

        wrong_hash_project = root / "wrong-hash"
        shutil.copytree(source_overlays, wrong_hash_project / "scripts" / "ci" / "notices")
        manifest_path = wrong_hash_project / "scripts" / "ci" / "notices" / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["entries"][0]["noticeSha256"] = "0" * 64
        write_output(manifest_path, (json.dumps(manifest, indent=2) + "\n").encode())
        must_fail("wrong overlay hash", lambda: load_reviewed_overlays(wrong_hash_project))

        missing_mapping_project = root / "missing-mapping"
        shutil.copytree(source_overlays, missing_mapping_project / "scripts" / "ci" / "notices")
        manifest_path = missing_mapping_project / "scripts" / "ci" / "notices" / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["entries"].pop()
        write_output(manifest_path, (json.dumps(manifest, indent=2) + "\n").encode())
        must_fail("missing overlay mapping", lambda: load_reviewed_overlays(missing_mapping_project))

        crate = root / "crate"
        crate.mkdir()
        vcs_path = crate / ".cargo_vcs_info.json"
        expected = dict(EXPECTED_OVERLAYS[("cargo", "webview2-com", "0.39.1")])
        expected["text"] = "fixture"
        package = {
            "ecosystem": "cargo",
            "license": "MIT",
            "repository": "github.com/wravery/webview2-rs",
            "directory": crate,
        }
        wrong_vcs = {"git": {"sha1": "0" * 40}, "path_in_vcs": "crates/webview2-com"}
        write_output(vcs_path, (json.dumps(wrong_vcs) + "\n").encode())
        must_fail("wrong overlay VCS", lambda: bind_reviewed_overlay(package, expected))

        must_fail("wrong target", lambda: generate({}, {}, root, "x86_64-unknown-linux-gnu"))
        root_package = {"id": "root", "source": None}
        if cargo_source_identity(root_package, "root", root) is not None:
            raise AssertionError("application root must not become a third-party notice")
        vendored = root / "src-tauri" / "vendor" / "zune-jpeg-0.5.15"
        vendored.mkdir(parents=True)
        write_output(vendored / "Cargo.toml", b"synthetic fixture only\n")
        local_package = {
            "id": "local", "source": None, "name": "zune-jpeg", "version": "0.5.15",
            "manifest_path": str(vendored / "Cargo.toml"),
        }
        if not cargo_source_identity(local_package, "root", root).startswith("locally-modified-vendored:"):
            raise AssertionError("local patched third-party source was omitted")
        must_fail("unknown path dependency", lambda: cargo_source_identity(
            {**local_package, "name": "unreviewed"}, "root", root))
        must_fail("wrong local version", lambda: cargo_source_identity(
            {**local_package, "version": "0.5.16"}, "root", root))
        must_fail("outside local dependency", lambda: cargo_source_identity(
            {**local_package, "manifest_path": str(crate / "Cargo.toml")}, "root", root))
        if not has_blocking_gaps({"requiredMissingTextCount": 1, "requiredUnknownLicenseCount": 0}):
            raise AssertionError("required missing-text fixture did not block")
        if has_blocking_gaps({"requiredMissingTextCount": 0, "requiredUnknownLicenseCount": 0}):
            raise AssertionError("zero-gap fixture unexpectedly blocked")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        print("Notice inventory self-test: PASS")
        return 0
    if len(sys.argv) != 7:
        print(
            "Usage: third_party_notices.py CARGO_METADATA TARGET_CARGO_METADATA PROJECT OUTPUT_TEXT OUTPUT_REPORT TARGET_TRIPLE",
            file=sys.stderr,
        )
        return 2
    try:
        metadata_path = Path(sys.argv[1])
        project = Path(sys.argv[3]).resolve(strict=True)
        if is_link(project) or not project.is_dir():
            raise ValueError("Project root is not a normal directory")
        metadata = read_json(metadata_path, metadata_path.parent.resolve(strict=True))
        required_metadata_path = Path(sys.argv[2])
        required_metadata = read_json(
            required_metadata_path, required_metadata_path.parent.resolve(strict=True)
        )
        body, report = generate(metadata, required_metadata, project, sys.argv[6])
        write_output(Path(sys.argv[4]), body)
        report_bytes = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        write_output(Path(sys.argv[5]), report_bytes)
        print(
            f"Notice inventory: {report['packageCount']} packages; "
            f"{report['requiredPackageCount']} in packaged-app graph; "
            f"{report['requiredMissingTextCount']} required without official standalone text"
        )
        if has_blocking_gaps(report):
            unresolved = report["unresolvedRequiredPackages"] + report["unknownLicenseRequiredPackages"]
            print(
                "Notice inventory gate failed; unresolved required packages: " + ", ".join(unresolved),
                file=sys.stderr,
            )
            return 1
    except (OSError, ValueError, KeyError, TypeError, UnicodeError) as error:
        print(f"Notice inventory failed: {type(error).__name__}: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
