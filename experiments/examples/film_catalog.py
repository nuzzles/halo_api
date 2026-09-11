"""Load the single catalog shared by downloads, probes, and replay categories."""

import csv
from pathlib import Path
import re
from uuid import UUID

EXPERIMENTS_ROOT = Path(__file__).resolve().parents[1]
CATALOG = EXPERIMENTS_ROOT / "films.csv"


def load_catalog(path=CATALOG):
    with path.open(newline="") as file:
        rows = list(csv.DictReader(file))
    ids, labels = set(), set()
    for row in rows:
        if not all(row.get(k) for k in ("category", "group", "slug", "match_id", "description", "analysis_profile")):
            raise ValueError("Catalog contains an incomplete row")
        if any(not re.fullmatch(r"[a-zA-Z0-9-]+", row[k]) for k in ("group", "slug")):
            raise ValueError("Unsafe catalog directory")
        if str(UUID(row["match_id"])) != row["match_id"]:
            raise ValueError("Catalog match ID must be a canonical UUID")
        label = row["group"] + "/" + row["slug"]
        if row["match_id"] in ids or label in labels:
            raise ValueError("Duplicate catalog match ID or storage label")
        if row["analysis_profile"] not in ("controlled", "octagon", "bandit", "oddball", "decoded", "pending"):
            raise ValueError("Unknown analysis profile")
        ids.add(row["match_id"])
        labels.add(label)
    return rows


if __name__ == "__main__":
    from collections import Counter
    rows = load_catalog()
    print(f"{len(rows)} unique films in {CATALOG}")
    for category, count in Counter(row["category"] for row in rows).items():
        print(f"{category}: {count}")
