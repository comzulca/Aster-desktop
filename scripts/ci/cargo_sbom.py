#!/usr/bin/env python3
"""Produce a CycloneDX dependency graph and license inventory from Cargo metadata.

The report describes resolved *source dependencies*. It does not prove which
components are in a packaged installer or whether a declared license is valid.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import sys
from urllib.parse import quote
import uuid


def stable_ref(package_id: str) -> str:
    return "cargo:" + hashlib.sha256(package_id.encode("utf-8")).hexdigest()


def make_reports(metadata: dict) -> tuple[dict, dict]:
    packages = metadata.get("packages")
    resolution = metadata.get("resolve")
    if not isinstance(packages, list) or not packages or not isinstance(resolution, dict):
        raise ValueError("Cargo metadata must contain packages and a resolved dependency graph")
    root_id = resolution.get("root") or (metadata.get("workspace_members") or [None])[0]
    by_id = {pkg["id"]: pkg for pkg in packages}
    if root_id not in by_id:
        raise ValueError("Cargo metadata has no resolvable workspace root")
    app = by_id[root_id]
    if app.get("license") != "Apache-2.0":
        raise ValueError("Application root must explicitly declare Apache-2.0")
    root_ref = stable_ref(root_id)
    components = []
    licenses = []
    for package_id, pkg in sorted(by_id.items(), key=lambda item: (item[1]["name"], item[1]["version"], item[0])):
        source_type = (pkg.get("source") or "path").split("+", 1)[0]
        declared_license = pkg.get("license") or "UNKNOWN"
        licenses.append(
            {"name": pkg["name"], "version": pkg["version"], "declaredLicense": declared_license, "sourceType": source_type}
        )
        if package_id == root_id:
            continue
        component = {
            "type": "library",
            "bom-ref": stable_ref(package_id),
            "name": pkg["name"],
            "version": pkg["version"],
            "properties": [
                {"name": "cargo:declared-license", "value": declared_license},
                {"name": "cargo:source-type", "value": source_type},
            ],
        }
        if declared_license != "UNKNOWN":
            # Cargo permits the legacy slash separator as an alternative to OR.
            # CycloneDX 1.5 requires a single SPDX expression in this field.
            component["licenses"] = [{"expression": re.sub(r"\s*/\s*", " OR ", declared_license)}]
        if source_type == "registry":
            component["purl"] = "pkg:cargo/" + quote(pkg["name"], safe="") + "@" + quote(pkg["version"], safe="")
        components.append(component)
    references = set(by_id)
    dependencies = []
    for node in resolution.get("nodes", []):
        if node["id"] not in references:
            raise ValueError("Dependency node references an unknown package")
        # Cargo metadata's deps array includes renamed dependencies; `pkg` is
        # the resolved package identity and is the only edge needed here.
        targets = {dep["pkg"] for dep in node.get("deps", [])}
        if not targets.issubset(references):
            raise ValueError("Dependency edge references an unknown package")
        dependencies.append({"ref": stable_ref(node["id"]), "dependsOn": sorted(stable_ref(target) for target in targets)})
    report = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": "urn:uuid:" + str(uuid.uuid5(uuid.NAMESPACE_URL, root_ref)),
        "version": 1,
        "metadata": {"component": {"type": "application", "bom-ref": root_ref, "name": app["name"], "version": app["version"], "licenses": [{"expression": app["license"]}]}},
        "components": components,
        "dependencies": sorted(dependencies, key=lambda item: item["ref"]),
    }
    return report, {"source": "cargo metadata --locked --format-version 1", "packages": licenses}


def main() -> int:
    if len(sys.argv) != 4:
        print("Usage: cargo_sbom.py METADATA_JSON SBOM_JSON LICENSES_JSON", file=sys.stderr)
        return 2
    try:
        metadata = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8-sig"))
        sbom, licenses = make_reports(metadata)
        for file_name, contents in ((sys.argv[2], sbom), (sys.argv[3], licenses)):
            Path(file_name).write_text(json.dumps(contents, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    except (OSError, KeyError, ValueError, TypeError) as exc:
        print(f"Rust SBOM generation failed: {exc}", file=sys.stderr)
        return 1
    print(f"Rust SBOM includes {len(sbom['components'])} resolved dependencies.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
