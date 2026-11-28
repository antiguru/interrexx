"""Controls for the generated conformance report."""

import unittest

import conformance_summary as summary


REVISION = "# ootest revision 13624\n"
INVENTORY = {
    "suite-revision.txt": "13624\n",
    "differential.txt": "# included paths\n\nlang/one.rex\nlang/two.rex\nlang/three.rex\n",
    "unfiled.txt": "lang/four.rex\tRecorded Deviation\n",
    "builtin-status.txt": "LENGTH\timplemented\nBEEP\tlater-level\n",
    "deviations.tsv": "1\tactive\t-\t-\n2\twithdrawn\t-\t-\n",
    "suite-licences.txt": REVISION
    + "base/bif/LENGTH\tTEST_A\tUnspecified\tWording is unspecified\n"
    + "base/class/Stream\tTEST_B\tDeviation 1\tStream differs\n",
    "suite-not-run.txt": REVISION
    + "base/rexxutil/platform/windows/SysIni\tTEST_C\tNotRun\trestrictOS line 12\n"
    + "extensions/platform/windows/ole/OLEObject\tTEST_D\tNotRun\tplatform windows\n",
}
SUITE = (
    REVISION + "# groups base/class/Stream,API/oo/METHOD\n" + summary.SUITE_HEADER + "\n"
    "base/class/Stream\tTEST_A\t-\tpassed\tpassed\n"
    "base/class/Stream\tTEST_B\tDeviation(1)\tpassed\tnot started\n"
    "base/class/Stream\tTEST_C\t-\tpassed\tfailed\n"
    "base/class/Stream\tTEST_D\t-\tpassed\tblocked: timeout\n"
    "base/class/Stream\tTEST_E\t-\tpassed\tstarted\n"
    "API/oo/METHOD\tTEST_A\t-\tpassed\tblocked at load: no summary\n"
)


def corpus_shard(index, matched, total):
    return (
        "==============================================================================\n"
        "interrexx differential corpus report -- rust/corpus/differential.txt -- "
        f"shard {index} of 2\n"
        "mode: STRICT (the gate) -- REXX_CORPUS_GATE is set\n"
        f"{matched} of {total} matching\n"
    )


class ConformanceSummary(unittest.TestCase):
    def test_inventory_has_provenance_and_no_observed_results(self):
        report = summary.render(INVENTORY.__getitem__, "interrexx-sha", "oracle-sha")
        self.assertIn("Interrexx inventory commit `interrexx-sha`", report)
        self.assertIn("ooRexx pin `oracle-sha`", report)
        self.assertIn("ooTest pin r13624", report)
        self.assertIn("3 programs are listed", report)
        self.assertIn("1 active and 1 withdrawn", report)
        self.assertIn("| later-level | 1 | BEEP |", report)
        self.assertIn("| Built-in functions | 0 | 1 | 0 |", report)
        self.assertIn("| Streams | 1 | 0 | 0 |", report)
        self.assertIn("| RexxUtil | 0 | 0 | 1 |", report)
        self.assertIn("| Platform extensions | 0 | 0 | 1 |", report)
        self.assertIn("| RexxUtil | NotRun | restrictOS line 12 | 1 |", report)
        self.assertIn("| lang/four.rex | Recorded Deviation |", report)
        self.assertIn("No strict corpus log supplied.", report)
        self.assertIn("No ooTest per-test report supplied.", report)
        self.assertNotIn("Program comparisons reported", report)

    def test_suite_counts_use_observed_verdicts_per_engine(self):
        report = summary.render(INVENTORY.__getitem__, "revision", "pin", suite_report=SUITE)
        self.assertIn("selected groups (base/class/Stream,API/oo/METHOD)", report)
        self.assertIn("| Streams | interrexx | 4 | 1 | 1 | 1 | 1 | 0 | 1 |", report)
        self.assertIn("| Streams | ooRexx | 5 | 5 | 0 | 0 | 0 | 0 | 0 |", report)
        self.assertIn("| Native API | interrexx | 0 | 0 | 0 | 0 | 0 | 1 | 0 |", report)

    def test_unrecognised_or_stale_suite_report_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "does not match the ooTest pin"):
            summary.suite_results(SUITE, "13625")
        with self.assertRaisesRegex(ValueError, "unknown verdict"):
            summary.suite_results(SUITE.replace("blocked: timeout", "passed eventually"), "13624")

    def test_partial_corpus_counts_only_reported_shards(self):
        report = summary.render(
            INVENTORY.__getitem__, "revision", "pin", corpus_log=corpus_shard(0, 1, 2),
        )
        self.assertIn("cover 1 of 2 shards", report)
        self.assertIn("| 2 | 1 | 1 |", report)

    def test_corpus_reports_do_not_double_count_shards(self):
        first = corpus_shard(0, 1, 2)
        full = "\n".join(summary.corpus_results(first + corpus_shard(1, 1, 1), 3))
        self.assertIn("cover 2 of 2 shards", full)
        self.assertIn("| 3 | 2 | 1 |", full)
        with self.assertRaisesRegex(ValueError, "repeated or inconsistent shard indices"):
            summary.corpus_results(first + first, 3)
        with self.assertRaisesRegex(ValueError, "does not match the tracked inventory size"):
            summary.corpus_results(corpus_shard(0, 1, 1), 3)
        with self.assertRaisesRegex(ValueError, "no strict differential report"):
            summary.corpus_results(first.replace("mode: STRICT", "mode: REPORT"), 3)


if __name__ == "__main__":
    unittest.main()
