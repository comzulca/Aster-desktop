"""Stage verified offline dictation resources or probe an extracted Windows payload."""

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile
import urllib.request
import zipfile


ROOT = Path(__file__).resolve().parents[1]
ASSETS = json.loads((ROOT / "src-tauri/speech-assets.json").read_text("utf-8"))
WHISPER_REVISION = "5eaee0cc9024eb9e9a99469199920c8b7f5cb9ec"
WHISPER = f"https://raw.githubusercontent.com/ggml-org/whisper.cpp/{WHISPER_REVISION}"
ARCHIVE = "https://github.com/ggml-org/whisper.cpp/releases/download/b5907/whisper-bin-x64.zip"
ARCHIVE_SHA256 = "ecca07b277d943e9b9f4712856a144b857271a6b333104e511e1988e80db582b"
MODEL_REVISION = "5359861c739e955e79d9a303bcbc70fb988958b1"
MODEL_LICENSE_REVISION = "86098128c0b4f24f0e2aa2994de830614b474227"
MODEL = "ggml-base-q5_1.bin"
FIXTURE = ROOT / "reports/speech-fixture.wav"
FIXTURE_SIZE = 352078
FIXTURE_SHA256 = "59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e"
URLS = {
    MODEL: f"https://huggingface.co/ggerganov/whisper.cpp/resolve/{MODEL_REVISION}/{MODEL}",
    "whisper.cpp-LICENSE.txt": f"{WHISPER}/LICENSE",
    "whisper-model-LICENSE.txt": f"https://raw.githubusercontent.com/openai/whisper/{MODEL_LICENSE_REVISION}/LICENSE",
}


def validate(data, size, digest):
    if len(data) != size or hashlib.sha256(data).hexdigest() != digest:
        raise ValueError("Unexpected speech resource length or SHA-256")
    return data


def download(url, size, digest):
    with urllib.request.urlopen(url, timeout=120) as response:
        if not response.url.startswith("https://"):
            raise ValueError("Unexpected download protocol")
        return validate(response.read(size + 1), size, digest)


def guard_path(path):
    if path.exists() and getattr(path.lstat(), "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT:
        raise ValueError("Reparse points cannot supply speech resources")
    if path.is_symlink():
        raise ValueError("Symlinks cannot supply speech resources")


def public_fixture():
    guard_path(FIXTURE.parent)
    guard_path(FIXTURE)
    if FIXTURE.exists():
        if not FIXTURE.is_file() or FIXTURE.stat().st_size != FIXTURE_SIZE:
            raise ValueError("Unexpected public audio fixture")
        return validate(FIXTURE.read_bytes(), FIXTURE_SIZE, FIXTURE_SHA256)
    data = download(f"{WHISPER}/samples/jfk.wav", FIXTURE_SIZE, FIXTURE_SHA256)
    with tempfile.NamedTemporaryFile(dir=FIXTURE.parent, delete=False) as staging:
        staging.write(data)
    os.replace(staging.name, FIXTURE)
    return data


def verify(folder):
    guard_path(folder)
    if {entry.name for entry in folder.iterdir()} != set(ASSETS):
        raise ValueError("Unexpected speech directory inventory")
    for name, spec in ASSETS.items():
        path = folder / name
        guard_path(path)
        if not path.is_file() or path.stat().st_size != spec["bytes"]:
            raise ValueError("Unexpected speech resource")
        validate(path.read_bytes(), spec["bytes"], spec["sha256"])


def prepare():
    folder = ROOT / "reports/speech"
    guard_path(folder.parent)
    guard_path(folder)
    folder.mkdir(parents=True, exist_ok=True)
    archive = None
    for name, spec in ASSETS.items():
        path = folder / name
        guard_path(path)
        if path.exists():
            continue
        size, digest = spec["bytes"], spec["sha256"]
        if name in URLS:
            data = download(URLS[name], size, digest)
        else:
            if archive is None:
                payload = download(ARCHIVE, 9168428, ARCHIVE_SHA256)
                archive = zipfile.ZipFile(io.BytesIO(payload))
            with archive.open("Release/" + name) as item:
                data = validate(item.read(size + 1), size, digest)
        with tempfile.NamedTemporaryFile(dir=folder, delete=False) as staging:
            staging.write(data)
        os.replace(staging.name, path)
    verify(folder)
    public_fixture()
    print("Verified packaged speech resources:", len(ASSETS))


def payload_inventory(folder, source_revision, installer_sha256):
    """Describe the verified, extracted files; this is not an advisory scan."""
    for value, length in ((source_revision, 40), (installer_sha256, 64)):
        if len(value) != length or any(char not in "0123456789abcdef" for char in value):
            raise ValueError("Invalid source or installer identity")
    verify(folder)
    direct = {
        MODEL: ("whisper-model", MODEL_REVISION),
        "whisper.cpp-LICENSE.txt": ("whisper.cpp-license", WHISPER_REVISION),
        "whisper-model-LICENSE.txt": ("whisper-model-license", MODEL_LICENSE_REVISION),
    }
    binaries = {
        "whisper-cli.exe": "whisper.cpp",
        "whisper.dll": "whisper.cpp",
        "ggml.dll": "GGML",
        "ggml-base.dll": "GGML",
        "ggml-cpu-x64.dll": "GGML",
        "ggml-cpu-haswell.dll": "GGML",
    }
    if set(ASSETS) != set(direct) | set(binaries) or set(direct) != set(URLS):
        raise ValueError("Missing or unexpected speech provenance")
    resources = []
    for name, spec in sorted(ASSETS.items()):
        if name in direct:
            component, revision = direct[name]
            origin = {"url": URLS[name], "revision": revision}
        else:
            component = binaries[name]
            origin = {
                "url": ARCHIVE,
                "revision": WHISPER_REVISION,
                "release_tag": "b5907",
                "archive_sha256": ARCHIVE_SHA256,
                "archive_member": f"Release/{name}",
            }
        resources.append({
            "path": f"speech/{name}",
            "component": component,
            "byte_count": spec["bytes"],
            "sha256": spec["sha256"],
            "origin": origin,
        })
    return {
        "schema_version": 1,
        "classification": "verified-extracted-nsis-speech-inventory",
        "source_revision": source_revision,
        "installer_sha256": installer_sha256,
        "resources": resources,
    }


def probe(folder):
    folder = folder.resolve(strict=True)
    verify(folder)
    wav = public_fixture()
    system_root = os.environ["SystemRoot"]
    result = subprocess.run(
        [str(folder / "whisper-cli.exe"), "-m", MODEL, "-f", "-", "-otxt", "-of", "-", "-l", "auto", "-nt", "-np", "-ng", "-t", "4"],
        input=wav, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, cwd=folder,
        env={"SystemRoot": system_root, "PATH": str(Path(system_root) / "System32")},
        timeout=120, check=True,
    )
    verify(folder)
    words = " ".join(result.stdout.decode("utf-8").lower().split())
    if "country can do for you" not in words:
        raise ValueError("Speech fixture transcript mismatch")
    print("PASS: packaged CPU engine transcribed the upstream public-domain fixture.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--test-dir", type=Path, help="Probe the exact supplied directory without changing its contents")
    parser.add_argument("--inventory-dir", type=Path, help="Print provenance for a verified extracted speech directory")
    parser.add_argument("--source-revision")
    parser.add_argument("--installer-sha256")
    args = parser.parse_args()
    if args.inventory_dir is not None:
        if args.test_dir is not None or not args.source_revision or not args.installer_sha256:
            parser.error("--inventory-dir requires --source-revision and --installer-sha256, without --test-dir")
        result = payload_inventory(args.inventory_dir, args.source_revision, args.installer_sha256)
        print(json.dumps(result, indent=2, sort_keys=True))
    elif args.source_revision is not None or args.installer_sha256 is not None:
        parser.error("Source and installer identity require --inventory-dir")
    elif args.test_dir:
        probe(args.test_dir)
    else:
        prepare()
