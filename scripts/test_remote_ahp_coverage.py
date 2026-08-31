"""Regression tests for the pinned AHP parity-ledger checker."""

from __future__ import annotations

import unittest

import check_remote_ahp_coverage as coverage


class RemoteAhpCoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document = coverage.PARITY_RFD.read_text(encoding="utf-8")

    def test_current_ledger_is_complete(self) -> None:
        self.assertEqual(coverage.inventory_errors(), [])
        self.assertEqual(coverage.document_errors(self.document), [])

    def test_missing_directional_matrix_is_rejected(self) -> None:
        start = self.document.index("### Reverse request methods")
        end = self.document.index("## Notification matrix", start)
        document = self.document[:start] + self.document[end:]

        errors = coverage.document_errors(document)

        self.assertTrue(any(error.startswith("reverse methods:") for error in errors))

    def test_inventory_token_dump_is_not_a_ledger(self) -> None:
        tokens = [f"`{coverage.AUDITED_AHP_COMMIT}`"]
        tokens.extend(
            f"`{item}`"
            for items in coverage.INVENTORIES.values()
            for item in items
        )

        errors = coverage.document_errors("\n".join(tokens))

        self.assertTrue(errors)

    def test_mapping_requires_nonempty_target(self) -> None:
        line = next(
            line
            for line in self.document.splitlines()
            if line.startswith("| `ping`")
        )
        cells = line.split("|")
        cells[-2] = " "
        document = self.document.replace(line, "|".join(cells), 1)

        errors = coverage.document_errors(document)

        self.assertIn(
            "client methods: `ping` lacks a status and nonempty ACP target",
            errors,
        )


if __name__ == "__main__":
    unittest.main()
