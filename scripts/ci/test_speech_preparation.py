"""Offline regression checks for speech resource preparation."""

import runpy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


speech = runpy.run_path(str(Path(__file__).resolve().parents[1] / "prepare-speech.py"))


class SpeechPreparationTests(unittest.TestCase):
    def test_probe_and_inventory_reject_linked_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = root / "payload"
            (payload / "speech").mkdir(parents=True)
            alias = root / "alias"
            alias.symlink_to(payload, target_is_directory=True)
            for folder in (alias, alias / "speech"):
                with self.subTest(folder=folder):
                    with self.assertRaisesRegex(ValueError, "Symlinks|Reparse"):
                        speech["probe"](folder)
                    with self.assertRaisesRegex(ValueError, "Symlinks|Reparse"):
                        speech["payload_inventory"](folder, "0" * 40, "0" * 64)

    def test_failed_publication_preserves_existing_file_and_removes_temporary(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            target = folder / "resource"
            target.write_bytes(b"original")
            with patch("os.replace", side_effect=PermissionError), self.assertRaises(PermissionError):
                speech["publish_bytes"](target, b"replacement")
            self.assertEqual(target.read_bytes(), b"original")
            self.assertEqual(list(folder.iterdir()), [target])
            speech["publish_bytes"](target, b"replacement")
            self.assertEqual(target.read_bytes(), b"replacement")


if __name__ == "__main__":
    unittest.main()
