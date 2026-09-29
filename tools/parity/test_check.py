#!/usr/bin/env python3
"""Allowlist, path parsing, and golden-rewrite checks for tools/parity/check.py."""

from __future__ import annotations

import contextlib
import io
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import check  # noqa: E402


def broker_paths() -> list[str]:
    entries = check.load_divergences()
    paths: list[str] = []
    for entry in check.listed(entries, "broker_board.txt", "gamesession"):
        paths.extend(entry["paths"])
    return paths


def report_failed(errors: list[str]) -> int:
    entries = check.load_divergences()
    failed = [0]
    with contextlib.redirect_stdout(io.StringIO()):
        check.report("broker_board.txt", "gamesession", errors, entries, failed)
    return failed[0]


class AllowlistTest(unittest.TestCase):
    def test_broker_paths_are_only_offers_1_and_4(self) -> None:
        paths = broker_paths()
        self.assertTrue(paths)
        for path in paths:
            self.assertTrue(
                path.startswith("$.board.offers[1].") or path.startswith("$.board.offers[4]."),
                path,
            )
            self.assertNotIn("[*]", path)

    def test_known_offer_diffs_are_allowed(self) -> None:
        errors = [f"{path}: 'left' != 'right'" for path in broker_paths()]
        self.assertEqual(report_failed(errors), 0)

    def test_offers_0_change_fails(self) -> None:
        errors = [f"{path}: 'left' != 'right'" for path in broker_paths()]
        errors.append("$.board.offers[0].id: 'ret_spice_restock' != 'other'")
        self.assertEqual(report_failed(errors), 1)

    def test_offers_2_and_3_changes_fail(self) -> None:
        for index in (2, 3):
            errors = [f"{path}: 'left' != 'right'" for path in broker_paths()]
            errors.append(f"$.board.offers[{index}].reward_silver: 1 != 2")
            self.assertEqual(report_failed(errors), 1, index)


class ErrorPathTest(unittest.TestCase):
    def test_dollar_inside_a_value_does_not_move_the_path(self) -> None:
        line = "$.board.offers[0].title: 'pay $10' != 'pay $12'"
        self.assertEqual(check.error_path(line), "$.board.offers[0].title")

    def test_round_trip_prefix_is_stripped_only_at_the_start(self) -> None:
        line = "rust-save/python-load $.board.offers[1].title: 'Pay $10' != 'Pay $12'"
        self.assertEqual(check.error_path(line), "$.board.offers[1].title")

    def test_interior_dollar_is_not_a_prefix(self) -> None:
        line = "cost $ then $.board.offers[0].id: 1 != 2"
        self.assertNotEqual(check.error_path(line), "$.board.offers[0].id")
        self.assertFalse(check.error_path(line).startswith("$"))

    def test_dollar_in_an_allowlisted_value_stays_allowlisted(self) -> None:
        errors = [f"{path}: 'pay $1' != 'pay $2'" for path in broker_paths()]
        self.assertEqual(report_failed(errors), 0)


class WriteGoldenTest(unittest.TestCase):
    def test_write_golden_keeps_source(self) -> None:
        oracle = {
            "_source": "should-not-replace",
            "seed": 4,
            "board": {"offers": []},
            "log": [],
        }
        existing = {
            "_source": "python-oracle",
            "_note": "Python oracle snapshot",
            "seed": 0,
            "board": {"offers": [1]},
        }
        written = check.golden_to_write(oracle, existing)
        self.assertEqual(written["_source"], "python-oracle")
        self.assertEqual(written["_note"], "Python oracle snapshot")
        self.assertEqual(written["seed"], 4)
        self.assertNotIn("board", written)
        self.assertEqual(list(written)[:2], ["_source", "_note"])

    def test_write_golden_does_not_invent_source(self) -> None:
        written = check.golden_to_write({"seed": 1, "ledger": {}}, None)
        self.assertNotIn("_source", written)
        self.assertEqual(written, {"seed": 1})


if __name__ == "__main__":
    unittest.main()
