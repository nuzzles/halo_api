"""The active catalog retains natural-end controls and both Arena matches."""

import csv
import unittest

from film_catalog import EXPERIMENTS_ROOT, load_catalog


class ActiveCatalog(unittest.TestCase):
    def test_retained_original_captures_and_descriptions_survive(self):
        catalog = {row["match_id"]: row for row in load_catalog()}
        original_count = 0
        for path in (EXPERIMENTS_ROOT / "archive/manifests").glob("*.tsv"):
            with path.open() as file:
                for row in csv.reader(file, delimiter="\t"):
                    if not row or row[0].startswith("#"):
                        continue
                    group, slug, match_id, description = row
                    if group == "force-end":
                        self.assertNotIn(match_id, catalog)
                        continue
                    current = catalog[match_id]
                    self.assertEqual((current["group"], current["slug"]), (group, slug))
                    self.assertTrue(current["description"].startswith(description))
                    self.assertNotEqual(current["analysis_profile"], "pending")
                    original_count += 1
        self.assertEqual(original_count, 19)
        self.assertGreaterEqual(len(catalog), 23)

    def test_both_ranked_matches_share_category_and_have_separate_profiles(self):
        catalog = {row["match_id"]: row for row in load_catalog()}
        bandit = catalog["ec02ed9d-346e-4eb0-a491-77e6b557847c"]
        oddball = catalog["4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73"]
        self.assertEqual(bandit["category"], "Ranked Arena gameplay")
        self.assertEqual(oddball["category"], bandit["category"])
        self.assertEqual(oddball["analysis_profile"], "oddball")
        self.assertIn("Oddball", oddball["description"])


if __name__ == "__main__":
    unittest.main()
