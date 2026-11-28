"""Controls for the test entry preflight."""

from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import test_preflight as preflight


class Prerequisites(unittest.TestCase):
    def test_empty_checkout_reports_native_suite_and_docs_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            missing = preflight.missing_inputs(Path(directory))
        self.assertIn("build/lib/librxregexp.so", missing)
        self.assertIn("ootest/ooTest.frm", missing)
        self.assertIn("oodocs/rexxref/en-US/fundclasses.xml", missing)

    def test_complete_inputs_pass_and_missing_leaf_is_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in (*preflight.NATIVE, *preflight.SUITE_FILES, *preflight.DOCS):
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.touch()
            for path in preflight.SUITE_DIRS:
                (root / path).mkdir(parents=True, exist_ok=True)
            self.assertEqual(preflight.missing_inputs(root), [])
            (root / "build/lib/librxmath.so").unlink()
            self.assertEqual(preflight.missing_inputs(root), ["build/lib/librxmath.so"])

    def test_oracle_sibling_cannot_satisfy_checkout_local_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            library = parent / "oracle/build/lib/librxregexp.so"
            library.parent.mkdir(parents=True)
            library.touch()
            root = parent / "worktree"
            root.mkdir()
            self.assertIn("build/lib/librxregexp.so", preflight.missing_inputs(root))

    def test_standalone_needs_no_native_library(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in (*preflight.SUITE_FILES, *preflight.DOCS):
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.touch()
            for path in preflight.SUITE_DIRS:
                (root / path).mkdir(parents=True, exist_ok=True)
            self.assertEqual(preflight.missing_inputs(root, standalone=True), [])
            self.assertIn("build/lib/librxmath.so", preflight.missing_inputs(root))

    def test_standalone_still_reports_a_missing_suite_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            missing = preflight.missing_inputs(Path(directory), standalone=True)
        self.assertIn("ootest/ooTest.frm", missing)
        self.assertIn("oodocs/rexxref/en-US/fundclasses.xml", missing)
        self.assertNotIn("build/lib/librxmath.so", missing)

    def test_missing_tool_blocks_command_with_all_standalone_inputs_present(self):
        stderr = StringIO()
        with (
            patch.object(preflight, "missing_inputs", return_value=[]),
            patch.object(preflight.shutil, "which",
                         side_effect=lambda name: None if name == "c++" else f"/bin/{name}"),
            patch.object(preflight.os, "execvp") as execute,
            redirect_stderr(stderr),
        ):
            self.assertEqual(preflight.main(["--standalone", "--", "cargo", "nextest"]), 1)
        self.assertIn("Rust test tools missing from PATH: c++", stderr.getvalue())
        execute.assert_not_called()


if __name__ == "__main__":
    unittest.main()
