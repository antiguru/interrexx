"""Controls for the standalone filter's name check."""

import unittest

import check_standalone_filter as check

FILTER = """
not (
  binary_id(=a::wholly)
  | test(=everywhere)
  | (binary_id(=a::mixed) & test(=one))
)
"""
SUITES = {
    "a::wholly": {"x", "y"},
    "a::mixed": {"one", "two", "everywhere"},
    "a::other": {"everywhere"},
}


class Names(unittest.TestCase):
    def test_entries_are_read_in_all_three_shapes(self):
        self.assertEqual(
            check.entries(FILTER),
            [("a::wholly", None), (None, "everywhere"), ("a::mixed", "one")],
        )

    def test_a_filter_of_existing_names_passes(self):
        self.assertEqual(check.unresolved(check.entries(FILTER), SUITES), [])

    def test_a_misspelt_test_is_reported(self):
        found = check.entries(FILTER.replace("=one", "=onee"))
        self.assertEqual(check.unresolved(found, SUITES), [("a::mixed", "onee")])

    def test_a_misspelt_binary_is_reported(self):
        found = check.entries(FILTER.replace("a::wholly", "a::wholy"))
        self.assertEqual(check.unresolved(found, SUITES), [("a::wholy", None)])

    def test_a_test_in_the_wrong_binary_is_reported(self):
        found = check.entries(FILTER.replace("a::mixed", "a::other"))
        self.assertEqual(check.unresolved(found, SUITES), [("a::other", "one")])

    def test_a_test_in_no_binary_is_reported(self):
        found = check.entries(FILTER.replace("=everywhere", "=nowhere"))
        self.assertEqual(check.unresolved(found, SUITES), [(None, "nowhere")])

    def test_an_unknown_entry_shape_is_refused(self):
        with self.assertRaises(ValueError):
            check.entries(FILTER.replace("| test(=everywhere)", "| test(/every/)"))

    def test_a_missing_wrapper_is_refused(self):
        with self.assertRaises(ValueError):
            check.entries("binary_id(=a::wholly)")


if __name__ == "__main__":
    unittest.main()
