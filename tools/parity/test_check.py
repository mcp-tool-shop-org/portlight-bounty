#!/usr/bin/env python3
"""Allowlist, path parsing, and golden-rewrite checks for tools/parity/check.py."""

from __future__ import annotations

import contextlib
import io
import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import check  # noqa: E402


def broker_paths() -> list[str]:
    entries = check.load_divergences()
    paths: list[str] = []
    for entry in check.listed(entries, "broker_board.txt", "gamesession"):
        paths.extend(spec["path"] for spec in entry["paths"])
    return paths


def report_failed(errors: list[str], left=None, right=None, entries=None) -> int:
    if entries is None:
        entries = check.load_divergences()
    failed = [0]
    with contextlib.redirect_stdout(io.StringIO()):
        check.report("broker_board.txt", "gamesession", errors, entries, failed, left, right)
    return failed[0]


class AllowlistTest(unittest.TestCase):
    def test_broker_paths_are_leaves_on_offers_1_and_4(self) -> None:
        paths = broker_paths()
        self.assertTrue(paths)
        for path in paths:
            self.assertTrue(
                path.startswith("$.board.offers[1].") or path.startswith("$.board.offers[4]."),
                path,
            )
            self.assertNotIn("[*]", path)
            self.assertFalse(path.endswith(".tags"))
            self.assertNotEqual(path, "$.board.offers")

    def test_container_pattern_does_not_hide_an_offer_mutation(self) -> None:
        left = {"board": {"offers": [{"id": "kept", "reward_silver": 10}]}}
        right = {"board": {"offers": [{"id": "mutated", "reward_silver": 99}]}}
        errors = check.close(left, right, "$")
        pattern = "$.board.offers"
        self.assertIn(pattern, check.container_patterns(left, right, [pattern]))
        allowed, unexpected, _missing = check.partition(errors, [pattern])
        self.assertEqual(allowed, [])
        self.assertTrue(any(check.error_path(line) == "$.board.offers[0].id" for line in unexpected))
        self.assertFalse(check.path_matches("$.board.offers[0].id", pattern))
        self.assertFalse(check.path_matches("$.log[0].command", "$.log"))

    def test_offers_0_change_fails(self) -> None:
        errors = [f"{path}: 'left' != 'right'" for path in broker_paths()]
        errors.append("$.board.offers[0].id: 'ret_spice_restock' != 'other'")
        self.assertEqual(report_failed(errors), 1)

    def test_trailing_index_does_not_cross_a_field(self) -> None:
        self.assertTrue(check.path_matches("$.board.offers[1].tags[2]", "$.board.offers[1].tags[2]"))
        self.assertTrue(check.path_matches("$.board.offers[1].tags[2][0]", "$.board.offers[1].tags[2]"))
        self.assertFalse(check.path_matches("$.board.offers[1].tags[2].name", "$.board.offers[1].tags[2]"))
        self.assertFalse(check.path_matches("$.board.offers[10].id", "$.board.offers[1]"))

    def test_enemy_crew_pin_rejects_99(self) -> None:
        entries = check.load_divergences()
        pinned = next(
            spec
            for entry in check.listed(entries, "capture_prize.txt", "oracle")
            for spec in entry["paths"]
            if spec["path"].endswith("enemy_crew")
        )
        self.assertEqual(pinned["python"], 5)
        self.assertEqual(pinned["rust"], 0)
        spec = {"path": "$.enemy_crew", "python": pinned["python"], "rust": pinned["rust"]}
        fake = [{
            "script": "capture_prize.txt",
            "check": "oracle",
            "summary": "pin",
            "rust": "r",
            "python": "p",
            "paths": [spec],
        }]
        failed = [0]
        with contextlib.redirect_stdout(io.StringIO()):
            check.report(
                "capture_prize.txt",
                "oracle",
                ["$.enemy_crew: 5 != 99"],
                fake,
                failed,
                {"enemy_crew": 5},
                {"enemy_crew": 99},
            )
        self.assertEqual(failed[0], 1)
        self.assertFalse(check.values_equal(99, pinned["rust"]))

    def test_roundtrip_dropped_offer_is_not_allowlisted(self) -> None:
        offer = {"id": "kept", "reward_silver": 10}
        python = {"board": {"offers": [dict(offer) for _ in range(5)]}}
        rust = {"board": {"offers": [dict(offer) for _ in range(4)]}}
        errors = [f"rust-save/python-save {line}" for line in check.close(rust, python, "$")]
        self.assertTrue(any(check.error_path(line) == "$.board.offers[4]" for line in errors))

        def entry(path: str, python_value, rust_value) -> list[dict]:
            return [{
                "script": "broker_board.txt",
                "check": "roundtrip",
                "summary": "dropped offer",
                "rust": "session.rs",
                "python": "session.py",
                "paths": [{"path": path, "python": python_value, "rust": rust_value}],
            }]

        container = entry("$.board.offers", python["board"]["offers"], rust["board"]["offers"])
        failed = [0]
        container_out = io.StringIO()
        with contextlib.redirect_stdout(container_out):
            check.report("broker_board.txt", "roundtrip", errors, container, failed, python, rust)
        self.assertEqual(failed[0], 1, container_out.getvalue())
        self.assertIn("list or object", container_out.getvalue())

        bogus = entry("$.board.offers[4]", {"id": "present"}, {"id": "bogus"})
        failed = [0]
        pin_out = io.StringIO()
        with contextlib.redirect_stdout(pin_out):
            check.report("broker_board.txt", "roundtrip", errors, bogus, failed, python, rust)
        self.assertEqual(failed[0], 1, pin_out.getvalue())
        self.assertIn("not the pinned pair", pin_out.getvalue())
        self.assertIn("bogus", pin_out.getvalue())

        failed = [0]
        with contextlib.redirect_stdout(io.StringIO()):
            check.report("broker_board.txt", "roundtrip", errors, container, failed)
        self.assertEqual(failed[0], 1)

    def test_wildcard_pattern_is_readable(self) -> None:
        message = check.pattern_error("$.board.offers[*]")
        self.assertIsNotNone(message)
        assert message is not None
        self.assertIn("wildcard", message)
        self.assertIn("[*]", message)
        self.assertIsNone(check.pattern_error("$.board.offers[4].id"))
        entry = [{
            "script": "broker_board.txt",
            "check": "roundtrip",
            "summary": "wildcard",
            "rust": "r",
            "python": "p",
            "paths": [{"path": "$.board.offers[*].id", "python": "a", "rust": "b"}],
        }]
        failed = [0]
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            check.report(
                "broker_board.txt",
                "roundtrip",
                ["$.board.offers[0].id: 'a' != 'b'"],
                entry,
                failed,
                {"board": {"offers": [{"id": "a"}]}},
                {"board": {"offers": [{"id": "b"}]}},
            )
        self.assertEqual(failed[0], 1)
        text = out.getvalue()
        self.assertIn("wildcard", text)
        self.assertIn("[*]", text)
        self.assertNotIn("Traceback", text)


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


class WriteGoldenTest(unittest.TestCase):
    def test_unchanged_snapshot_keeps_source_and_bytes(self) -> None:
        existing = (
            '{\n'
            '  "_source": "python-oracle",\n'
            '  "_note": "keep the note",\n'
            '  "seed": 4,\n'
            '  "captain": {\n'
            '    "name": "Ada"\n'
            '  }\n'
            '}\n'
        )
        oracle = {"seed": 4, "captain": {"name": "Ada"}, "board": {"offers": []}, "ledger": {}}
        self.assertEqual(check.golden_text(oracle, existing), existing)
        self.assertIn('"_source": "python-oracle"', check.golden_text(oracle, existing))
        self.assertIn('"_note": "keep the note"', check.golden_text(oracle, existing))

    def test_changed_snapshot_still_keeps_source(self) -> None:
        existing = (
            '{\n'
            '  "_source": "python-oracle",\n'
            '  "_note": "keep the note",\n'
            '  "seed": 4\n'
            '}\n'
        )
        oracle = {"seed": 5, "board": []}
        rendered = check.golden_text(oracle, existing)
        parsed = json.loads(rendered)
        self.assertEqual(parsed["_source"], "python-oracle")
        self.assertEqual(parsed["_note"], "keep the note")
        self.assertEqual(parsed["seed"], 5)
        self.assertNotIn("board", parsed)

    def test_rewrite_matches_committed_golden_bytes(self) -> None:
        try:
            import portlight  # noqa: F401
        except ImportError:
            self.skipTest("Python game is not importable")
        if not os.path.exists(check.rust_bin()):
            self.skipTest("portlight binary is missing")
        import tempfile

        capture = os.path.join(check.GOLDEN, "capture_prize.json")
        with tempfile.TemporaryDirectory(prefix="pl-golden-save-") as save_root:
            with tempfile.TemporaryDirectory(prefix="pl-golden-out-") as dest:
                self.assertEqual(os.listdir(dest), [])
                for name in check.scripts():
                    script = os.path.join(check.SCRIPTS, name)
                    oracle = check.run_oracle(script, save_root)
                    path = os.path.join(check.GOLDEN, name.replace(".txt", ".json"))
                    with open(path, "rb") as fh:
                        committed = fh.read()
                    rendered = check.golden_text(oracle, committed.decode("utf-8")).encode("utf-8")
                    out = os.path.join(dest, os.path.basename(path))
                    with open(out, "wb") as fh:
                        fh.write(rendered)
                    with open(out, "rb") as fh:
                        self.assertEqual(fh.read(), committed, name)
        with open(capture, "rb") as fh:
            text = fh.read().decode("utf-8")
        self.assertIn('"_source": "python-oracle"', text)
        self.assertIn('"_note":', text)


if __name__ == "__main__":
    unittest.main()
