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
WHISPER = "https://raw.githubusercontent.com/ggml-org/whisper.cpp/5eaee0cc9024eb9e9a99469199920c8b7f5cb9ec"
ARCHIVE = "https://github.com/ggml-org/whisper.cpp/releases/download/b5907/whisper-bin-x64.zip"
MODEL = "ggml-base-q5_1.bin"
FIXTURE = ROOT / "reports/speech-fixture.wav"
FIXTURE_SIZE = 352078
FIXTURE_SHA256 = "59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e"
URLS = {
    MODEL: f"https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/{MODEL}",
    "whisper.cpp-LICENSE.txt": f"{WHISPER}/LICENSE",
    "whisper-model-LICENSE.txt": "https://raw.githubusercontent.com/openai/whisper/86098128c0b4f24f0e2aa2994de830614b474227/LICENSE",
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
                payload = download(ARCHIVE, 9168428, "ecca07b277d943e9b9f4712856a144b857271a6b333104e511e1988e80db582b")
                archive = zipfile.ZipFile(io.BytesIO(payload))
            with archive.open("Release/" + name) as item:
                data = validate(item.read(size + 1), size, digest)
        with tempfile.NamedTemporaryFile(dir=folder, delete=False) as staging:
            staging.write(data)
        os.replace(staging.name, path)
    verify(folder)
    public_fixture()
    print("Verified packaged speech resources:", len(ASSETS))


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
    args = parser.parse_args()
    if args.test_dir:
        probe(args.test_dir)
    else:
        prepare()
