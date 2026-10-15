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


class ShortageReasonTest(unittest.TestCase):
    PATH = "$.board.offers[2].offer_reason"

    def errors(self, python: str, rust: str, path: str | None = None) -> list[str]:
        return check.close(python, rust, path or self.PATH)

    def test_real_pairs_pass_both_ways(self) -> None:
        for python, rust in [
            ("Shortage at Corsair's Rest \u2014 urgent demand for grain",
             "Shortage at Corsair's Rest - urgent demand for Grain"),
            ("Shortage at Ironhaven \u2014 urgent demand for iron",
             "Shortage at Ironhaven - urgent demand for Iron Ore"),
        ]:
            self.assertEqual(self.errors(python, rust), [])
            self.assertEqual(check.close(rust, python, self.PATH), [])

    def test_equal_strings_compare_as_normal(self) -> None:
        self.assertEqual(self.errors("Same text", "Same text"), [])

    def test_other_differences_fail(self) -> None:
        cases = [
            # A procurement reason that differs.
            ("Porto Novo needs grain delivered", "Porto Novo needs Grain delivered"),
            # A shortage with the wrong port.
            ("Shortage at Ironhaven \u2014 urgent demand for grain",
             "Shortage at Thornport - urgent demand for Grain"),
            # A shortage with the wrong good name.
            ("Shortage at Ironhaven \u2014 urgent demand for iron",
             "Shortage at Ironhaven - urgent demand for Iron"),
            # An unknown good id.
            ("Shortage at Ironhaven \u2014 urgent demand for unobtainium",
             "Shortage at Ironhaven - urgent demand for Unobtainium"),
        ]
        for python, rust in cases:
            self.assertEqual(len(self.errors(python, rust)), 1, (python, rust))

    def test_the_rewording_is_only_accepted_on_a_board_offer(self) -> None:
        python = "Shortage at Ironhaven \u2014 urgent demand for iron"
        rust = "Shortage at Ironhaven - urgent demand for Iron Ore"
        for path in ["$.offer_reason", "$.board.active[0].offer_reason", "$.board.offers[0].title"]:
            self.assertEqual(len(self.errors(python, rust, path)), 1, path)

    def test_a_changed_field_beside_it_still_fails(self) -> None:
        left = {"offers": [{"id": "a", "offer_reason": "Shortage at X \u2014 urgent demand for grain"}]}
        right = {"offers": [{"id": "b", "offer_reason": "Shortage at X - urgent demand for Grain"}]}
        errors = check.close(left, right, "$")
        self.assertEqual([check.error_path(line) for line in errors], ["$.offers[0].id"])


class AllowlistTest(unittest.TestCase):
    def test_broker_board_gamesession_allowlist_is_closed(self) -> None:
        entries = check.load_divergences()
        self.assertEqual(check.listed(entries, "broker_board.txt", "gamesession"), [])
        self.assertEqual(check.listed(entries, "broker_board.txt", "oracle"), [])

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


# The expected-divergence list is closed by name. UNTAGGED_PAIRS are the entries
# that existed before the balance fork and stay byte for byte as they were.
# TUNED_PAIRS are the Rust-tuned pairs `pin_tuned.py`'s sweep produced.
UNTAGGED_PAIRS = [
    ("capture_prize.txt", "oracle"),
    ("bounty_claim.txt", "oracle"),
    ("bounty_claim.txt", "roundtrip"),
    ("bounty_claim_sail.txt", "oracle"),
    ("bounty_claim_sail.txt", "roundtrip"),
    ("insurance_claim.txt", "oracle"),
    ("capture_decline.txt", "oracle"),
    ("capture_fleet_full.txt", "oracle"),
    ("capture_prize.txt", "oracle"),
    ("capture_too_few.txt", "oracle"),
    ("capture_too_many.txt", "oracle"),
    ("upgrade_naval.txt", "oracle"),
    ("boarding.txt", "oracle"),
    ("bounty_claim.txt", "oracle"),
    ("bounty_claim_sail.txt", "oracle"),
    ("capture_spare_prefix.txt", "oracle"),
    ("gear_armor.txt", "oracle"),
    ("injury_heal.txt", "oracle"),
    ("naval_combat.txt", "oracle"),
    ("victory_provenance.txt", "oracle"),
    ("victory_spare.txt", "oracle"),
    ("victory_takeall.txt", "oracle"),
    ("naval_evade_clip.txt", "oracle"),
]

TUNED_PAIRS = [
    ("boarding.txt", "oracle_wide"),
    ("boarding.txt", "roundtrip"),
    ("bounty_claim.txt", "oracle_wide"),
    ("bounty_claim.txt", "roundtrip"),
    ("bounty_claim_sail.txt", "oracle"),
    ("bounty_claim_sail.txt", "oracle_wide"),
    ("bounty_claim_sail.txt", "roundtrip"),
    ("captain_bounty_hunter_s1.txt", "oracle"),
    ("captain_bounty_hunter_s1.txt", "oracle_wide"),
    ("captain_bounty_hunter_s1.txt", "gamesession"),
    ("captain_bounty_hunter_s1.txt", "roundtrip"),
    ("captain_bounty_hunter_s42.txt", "oracle"),
    ("captain_bounty_hunter_s42.txt", "oracle_wide"),
    ("captain_bounty_hunter_s42.txt", "roundtrip"),
    ("captain_dockhand_s1.txt", "oracle"),
    ("captain_dockhand_s1.txt", "oracle_wide"),
    ("captain_dockhand_s1.txt", "gamesession"),
    ("captain_dockhand_s1.txt", "roundtrip"),
    ("captain_dockhand_s42.txt", "oracle"),
    ("captain_dockhand_s42.txt", "oracle_wide"),
    ("captain_dockhand_s42.txt", "roundtrip"),
    ("captain_merchant_prince_s1.txt", "oracle_wide"),
    ("captain_merchant_prince_s1.txt", "gamesession"),
    ("captain_merchant_prince_s1.txt", "roundtrip"),
    ("captain_merchant_prince_s42.txt", "oracle_wide"),
    ("captain_merchant_prince_s42.txt", "roundtrip"),
    ("captain_privateer_s1.txt", "oracle"),
    ("captain_privateer_s1.txt", "oracle_wide"),
    ("captain_privateer_s1.txt", "gamesession"),
    ("captain_privateer_s1.txt", "roundtrip"),
    ("captain_privateer_s42.txt", "oracle"),
    ("captain_privateer_s42.txt", "oracle_wide"),
    ("captain_privateer_s42.txt", "roundtrip"),
    ("captain_scholar_s1.txt", "oracle"),
    ("captain_scholar_s1.txt", "gamesession"),
    ("captain_scholar_s1.txt", "roundtrip"),
    ("captain_scholar_s42.txt", "oracle"),
    ("captain_scholar_s42.txt", "roundtrip"),
    ("captain_smuggler_s1.txt", "oracle"),
    ("captain_smuggler_s1.txt", "oracle_wide"),
    ("captain_smuggler_s1.txt", "gamesession"),
    ("captain_smuggler_s1.txt", "roundtrip"),
    ("captain_smuggler_s42.txt", "oracle"),
    ("captain_smuggler_s42.txt", "oracle_wide"),
    ("captain_smuggler_s42.txt", "roundtrip"),
    ("capture_decline.txt", "oracle_wide"),
    ("capture_decline.txt", "roundtrip"),
    ("capture_prize.txt", "oracle_wide"),
    ("capture_prize.txt", "roundtrip"),
    ("capture_spare_prefix.txt", "oracle_wide"),
    ("capture_spare_prefix.txt", "roundtrip"),
    ("capture_too_few.txt", "oracle_wide"),
    ("capture_too_few.txt", "roundtrip"),
    ("capture_too_many.txt", "oracle_wide"),
    ("capture_too_many.txt", "roundtrip"),
    ("event_foreign.txt", "oracle"),
    ("event_foreign.txt", "gamesession"),
    ("event_foreign.txt", "roundtrip"),
    ("g9.txt", "oracle"),
    ("g9.txt", "gamesession"),
    ("g9.txt", "roundtrip"),
    ("gear_armor.txt", "oracle_wide"),
    ("gear_armor.txt", "roundtrip"),
    ("hull_day20.txt", "oracle"),
    ("hull_day20.txt", "gamesession"),
    ("hull_day20.txt", "roundtrip"),
    ("injury_heal.txt", "oracle"),
    ("injury_heal.txt", "oracle_wide"),
    ("injury_heal.txt", "roundtrip"),
    ("naval_combat.txt", "oracle_wide"),
    ("naval_combat.txt", "roundtrip"),
    ("naval_evade_clip.txt", "oracle_wide"),
    ("naval_evade_clip.txt", "roundtrip"),
    ("smuggler.txt", "oracle"),
    ("smuggler.txt", "gamesession"),
    ("smuggler.txt", "roundtrip"),
    ("victory_provenance.txt", "oracle_wide"),
    ("victory_provenance.txt", "roundtrip"),
    ("victory_spare.txt", "oracle_wide"),
    ("victory_spare.txt", "roundtrip"),
    ("victory_takeall.txt", "oracle_wide"),
    ("victory_takeall.txt", "roundtrip"),
]


WIDE_ROOTS = ("board", "captain_memories", "active_bounties", "deferred_fees", "ledger")


class ClosedListTest(unittest.TestCase):
    def entries(self) -> list[dict]:
        return check.load_json(check.DIVERGENCES)["entries"]

    def test_untagged_entries_are_the_original_pairs(self) -> None:
        pairs = [(e["script"], e["check"]) for e in self.entries() if "category" not in e]
        self.assertEqual(pairs, UNTAGGED_PAIRS)

    def test_tuned_entries_are_the_generated_pairs(self) -> None:
        pairs = [(e["script"], e["check"]) for e in self.entries() if check.is_tuned(e)]
        self.assertEqual(pairs, TUNED_PAIRS)
        self.assertEqual(len(self.entries()), len(UNTAGGED_PAIRS) + len(TUNED_PAIRS))

    def test_tuned_paths_are_exact_leaves_with_both_values(self) -> None:
        for entry in self.entries():
            if not check.is_tuned(entry):
                continue
            self.assertIn(entry["check"], ("oracle", "oracle_wide", "gamesession", "roundtrip"))
            for spec in entry["paths"]:
                self.assertIsNone(check.pattern_error(spec["path"]), spec["path"])
                self.assertTrue({"path", "python", "rust"} <= set(spec), spec["path"])
                self.assertFalse(isinstance(spec["python"], (dict, list)) and "$missing" not in spec["python"])

    def test_wide_keys_are_pinned_only_under_oracle_wide(self) -> None:
        for entry in self.entries():
            if entry["check"] != "oracle":
                continue
            for spec in entry["paths"]:
                root = spec["path"][2:].replace("[", ".").split(".")[0]
                self.assertNotIn(root, WIDE_ROOTS, f"{entry['script']} {spec['path']}")

    def test_base_content_drops_only_the_tuned_entries(self) -> None:
        base = check.load_divergences(base_content=True)
        self.assertEqual([(e["script"], e["check"]) for e in base], UNTAGGED_PAIRS)
        self.assertEqual(len(check.load_divergences()), len(UNTAGGED_PAIRS) + len(TUNED_PAIRS))

    def test_every_oracle_tuned_script_has_a_python_oracle_golden(self) -> None:
        for script, kind in TUNED_PAIRS:
            if kind != "oracle":
                continue
            golden = check.load_json(os.path.join(check.GOLDEN, script.replace(".txt", ".json")))
            self.assertEqual(golden.get("_source"), "python-oracle", script)


class WideSplitTest(unittest.TestCase):
    def test_wide_errors_are_split_from_narrow(self) -> None:
        errors = [
            "$.board.offers[0].id: 'a' != 'b'",
            "$.ledger[2].kind: 'x' != 'y'",
            "$.captain.silver: 1 != 2",
            "$.boardwalk: 1 != 2",
            "$: keys ['culture']",
        ]
        narrow, wide = check.split_wide(errors)
        self.assertEqual(wide, errors[:2])
        self.assertEqual(narrow, errors[2:])

    def test_a_wide_pin_matches_under_oracle_wide_only(self) -> None:
        left = {"board": {"offers": [{"id": "a"}]}}
        right = {"board": {"offers": [{"id": "b"}]}}
        entries = [{
            "script": "x.txt", "check": "oracle_wide", "category": "rust-tuned",
            "summary": "s", "rust": "r", "python": "p",
            "paths": [{"path": "$.board.offers[0].id", "python": "a", "rust": "b"}],
        }]
        narrow, wide = check.split_wide(check.close(left, right, "$"))
        failed = [0]
        with contextlib.redirect_stdout(io.StringIO()):
            check.report("x.txt", "oracle", narrow, entries, failed, left, right)
            check.report("x.txt", "oracle_wide", wide, entries, failed, left, right)
        self.assertEqual(failed[0], 0)
        # The same difference with a changed Rust value is not the pinned pair.
        moved = {"board": {"offers": [{"id": "other"}]}}
        _, wide = check.split_wide(check.close(left, moved, "$"))
        failed = [0]
        with contextlib.redirect_stdout(io.StringIO()):
            check.report("x.txt", "oracle_wide", wide, entries, failed, left, moved)
        self.assertEqual(failed[0], 1)

    def test_an_unlisted_wide_difference_fails(self) -> None:
        left = {"board": {"offers": [{"id": "a"}]}}
        right = {"board": {"offers": [{"id": "b"}]}}
        _, wide = check.split_wide(check.close(left, right, "$"))
        failed = [0]
        with contextlib.redirect_stdout(io.StringIO()):
            check.report("x.txt", "oracle_wide", wide, [], failed, left, right)
        self.assertEqual(failed[0], 1)

    def test_an_unknown_category_is_refused(self) -> None:
        import tempfile

        entry = {"script": "x.txt", "check": "oracle", "category": "other", "summary": "s",
                 "rust": "r", "python": "p", "paths": [{"path": "$.a", "python": 1, "rust": 2}]}
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "d.json")
            with open(path, "w", encoding="utf-8") as fh:
                json.dump({"note": "n", "entries": [entry]}, fh)
            old = check.DIVERGENCES
            check.DIVERGENCES = path
            try:
                with self.assertRaises(SystemExit):
                    check.load_divergences()
            finally:
                check.DIVERGENCES = old


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
