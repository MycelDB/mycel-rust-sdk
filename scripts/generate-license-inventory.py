#!/usr/bin/env python3
"""Generate docs/legal/dependency-license-inventory.tsv for Cargo dependencies."""
from __future__ import annotations

import csv
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "legal" / "dependency-license-inventory.tsv"
FIRST_PARTY = {"mycel", "mycel-sdk", "mycel-proto"}


def main() -> None:
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"], cwd=ROOT, text=True))
    rows = []
    for package in sorted(metadata["packages"], key=lambda item: (item["name"], item["version"], item.get("source") or "")):
        if not package.get("source") or package["name"] in FIRST_PARTY:
            continue
        rows.append([
            "cargo",
            package["name"],
            package["version"],
            "cargo-lock",
            package.get("license") or "UNKNOWN",
            "",
            package.get("repository") or package.get("homepage") or package.get("documentation") or package.get("source") or "",
        ])

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(["Ecosystem", "Package", "Version", "Scope", "License", "License files", "Source"])
        writer.writerows(rows)


if __name__ == "__main__":
    main()
