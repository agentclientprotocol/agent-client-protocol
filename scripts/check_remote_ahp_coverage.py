#!/usr/bin/env python3
"""Check that the ACP Remote parity RFD covers the pinned AHP surface."""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path


AUDITED_AHP_COMMIT = "b4016c0e02dd61393de268fa5e5f1a554d0ca611"

CLIENT_METHODS = (
    "initialize",
    "ping",
    "reconnect",
    "subscribe",
    "createSession",
    "disposeSession",
    "createChat",
    "disposeChat",
    "createTerminal",
    "disposeTerminal",
    "createResourceWatch",
    "listSessions",
    "resourceRead",
    "resourceWrite",
    "resourceList",
    "resourceCopy",
    "resourceDelete",
    "resourceMove",
    "resourceResolve",
    "resourceMkdir",
    "resourceRequest",
    "fetchTurns",
    "authenticate",
    "resolveSessionConfig",
    "sessionConfigCompletions",
    "completions",
    "invokeChangesetOperation",
    "listAutomationTriggerDefinitions",
    "runAutomation",
    "fetchAutomationRuns",
)

REVERSE_METHODS = (
    "resourceRead",
    "resourceWrite",
    "resourceList",
    "resourceCopy",
    "resourceDelete",
    "resourceMove",
    "resourceResolve",
    "resourceMkdir",
    "resourceRequest",
    "createResourceWatch",
)

CLIENT_NOTIFICATIONS = (
    "unsubscribe",
    "dispatchAction",
)

SERVER_NOTIFICATIONS = (
    "action",
    "root/sessionAdded",
    "root/sessionRemoved",
    "root/sessionSummaryChanged",
    "root/progress",
    "auth/required",
    "otlp/exportLogs",
    "otlp/exportTraces",
    "otlp/exportMetrics",
)

CHANNELS_AND_RESOURCES = (
    "ahp-root://",
    "ahp-session:/<id>",
    "ahp-chat:/<id>",
    "ahp-terminal:/<id>",
    "ahp-changeset:/<id>",
    "ahp-session:/<id>/annotations",
    "ahp-resource-watch:/<id>",
    "ahp-otlp:",
    "mcp://...",
    "ahp-automations://",
    "ahp-automation:/<id>",
    "ahp-automation-run:/<id>",
)

ACTIONS = (
    "root/agentsChanged",
    "root/activeSessionsChanged",
    "session/ready",
    "session/creationFailed",
    "session/chatAdded",
    "session/chatRemoved",
    "session/chatUpdated",
    "session/defaultChatChanged",
    "chat/turnStarted",
    "chat/delta",
    "chat/responsePart",
    "chat/toolCallStart",
    "chat/toolCallDelta",
    "chat/toolCallReady",
    "chat/toolCallConfirmed",
    "chat/toolCallComplete",
    "chat/toolCallResultConfirmed",
    "chat/toolCallContentChanged",
    "chat/toolCallAuthRequired",
    "chat/toolCallAuthResolved",
    "chat/turnComplete",
    "chat/turnCancelled",
    "chat/error",
    "chat/turnResume",
    "chat/activityChanged",
    "chat/workingDirectorySet",
    "chat/workingDirectoryRemoved",
    "session/titleChanged",
    "chat/usage",
    "chat/reasoning",
    "session/serverToolsChanged",
    "session/activeClientSet",
    "session/activeClientRemoved",
    "session/workingDirectorySet",
    "session/workingDirectoryRemoved",
    "session/workingDirectoryReplaced",
    "session/inputNeededSet",
    "session/inputNeededRemoved",
    "chat/pendingMessageSet",
    "chat/pendingMessageRemoved",
    "chat/queuedMessagesReordered",
    "chat/draftChanged",
    "chat/inputRequested",
    "chat/inputAnswerChanged",
    "chat/inputCompleted",
    "session/customizationsChanged",
    "session/customizationToggled",
    "session/customizationUpdated",
    "session/customizationRemoved",
    "session/mcpServerStateChanged",
    "session/mcpServerStartRequested",
    "session/mcpServerStopRequested",
    "chat/truncated",
    "chat/turnsLoaded",
    "session/isReadChanged",
    "session/isArchivedChanged",
    "session/activityChanged",
    "session/changesetsChanged",
    "session/configChanged",
    "session/metaChanged",
    "changeset/statusChanged",
    "changeset/fileSet",
    "changeset/fileRemoved",
    "changeset/filesReviewChanged",
    "changeset/contentChanged",
    "changeset/operationsChanged",
    "changeset/operationStatusChanged",
    "changeset/cleared",
    "annotations/set",
    "annotations/updated",
    "annotations/removed",
    "annotations/entrySet",
    "annotations/entryRemoved",
    "root/terminalsChanged",
    "root/configChanged",
    "terminal/data",
    "terminal/input",
    "terminal/resized",
    "terminal/claimed",
    "terminal/titleChanged",
    "terminal/cwdChanged",
    "terminal/exited",
    "terminal/cleared",
    "terminal/commandDetectionAvailable",
    "terminal/commandExecuted",
    "terminal/commandFinished",
    "resourceWatch/changed",
    "automation/createRequested",
    "automation/updateRequested",
    "automation/set",
    "automation/removed",
    "automationRun/lifecycleChanged",
    "automationRun/sessionSet",
    "automationRun/sessionRemoved",
    "automationRun/primarySessionChanged",
    "automationRun/cancelRequested",
)

MCP_METHODS = (
    "tools/list",
    "tools/call",
    "notifications/tools/list_changed",
    "resources/list",
    "resources/templates/list",
    "resources/read",
    "notifications/resources/list_changed",
    "logging/setLevel",
    "notifications/message",
    "sampling/createMessage",
)

INVENTORIES = {
    "client methods": CLIENT_METHODS,
    "reverse methods": REVERSE_METHODS,
    "client notifications": CLIENT_NOTIFICATIONS,
    "server notifications": SERVER_NOTIFICATIONS,
    "channels/resources": CHANNELS_AND_RESOURCES,
    "actions": ACTIONS,
    "MCP methods": MCP_METHODS,
}

EXPECTED_COUNTS = {
    "client methods": 30,
    "reverse methods": 10,
    "client notifications": 2,
    "server notifications": 9,
    "channels/resources": 12,
    "actions": 96,
    "MCP methods": 10,
}

SECTION_BOUNDS = {
    "client methods": (
        "## Canonical request-method matrix",
        "### Reverse request methods",
    ),
    "reverse methods": ("### Reverse request methods", "## Notification matrix"),
    "client notifications": ("## Notification matrix", "## MCP side-channel method matrix"),
    "server notifications": ("## Notification matrix", "## MCP side-channel method matrix"),
    "MCP methods": ("## MCP side-channel method matrix", "## Channel and state matrix"),
    "channels/resources": ("## Channel and state matrix", "## Complete state/action-family matrix"),
    "actions": (
        "## Complete state/action-family matrix",
        "### Message, response-part, attachment, and tool-result parity",
    ),
}

SUPPLEMENTAL_TABLES = {
    "capabilities": ("## Capability matrix", "## Version, schema, and compatibility matrix", 18),
    "version rules": ("## Version, schema, and compatibility matrix", "## Error matrix", 8),
    "errors": ("## Error matrix", "## Detailed host-feature parity", 14),
}

TOKEN_COLUMNS = {
    "client methods": 0,
    "reverse methods": 0,
    "client notifications": 0,
    "server notifications": 0,
    "MCP methods": 1,
    "channels/resources": 0,
    "actions": 1,
}

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
PARITY_RFD = REPOSITORY_ROOT / "docs" / "rfds" / "remote" / "ahp-parity.mdx"


def summarize(items: list[str], limit: int = 8) -> str:
    """Render a bounded list for concise CI failures."""
    shown = ", ".join(items[:limit])
    remaining = len(items) - limit
    if remaining > 0:
        return f"{shown} (+{remaining} more)"
    return shown


def inventory_errors() -> list[str]:
    """Validate the hardcoded oracle before checking the document."""
    errors: list[str] = []
    for category, items in INVENTORIES.items():
        expected = EXPECTED_COUNTS[category]
        if len(items) != expected:
            errors.append(f"{category}: expected {expected} entries, found {len(items)}")

        duplicates = sorted(item for item, count in Counter(items).items() if count > 1)
        if duplicates:
            errors.append(f"{category}: duplicate entries: {summarize(duplicates)}")

    return errors


def section(document: str, start: str, end: str) -> str | None:
    """Return one heading-bounded section, or None if either boundary is absent."""
    start_index = document.find(start)
    if start_index < 0:
        return None
    end_index = document.find(end, start_index + len(start))
    if end_index < 0:
        return None
    return document[start_index:end_index]


def table_rows(section_text: str) -> list[list[str]]:
    """Parse data rows from the simple Markdown tables used by the parity ledger."""
    rows: list[list[str]] = []
    for line in section_text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if not cells or all(not cell or set(cell) <= {"-", ":"} for cell in cells):
            continue
        if any(cell in {"ACP status", "AHP status"} for cell in cells):
            continue
        rows.append(cells)
    return rows


def mapped_row_errors(category: str, items: tuple[str, ...], rows: list[list[str]]) -> list[str]:
    """Require one status-bearing, nonempty target row per pinned surface entry."""
    errors: list[str] = []
    token_column = TOKEN_COLUMNS[category]
    for item in items:
        token = f"`{item}`"
        matches = [
            row for row in rows if len(row) > token_column and token in row[token_column]
        ]
        if len(matches) != 1:
            errors.append(f"{category}: {token} appears in {len(matches)} mapping rows, expected 1")
            continue

        row = matches[0]
        if len(row) < 3 or "**" not in row[-2] or not row[-1]:
            errors.append(f"{category}: {token} lacks a status and nonempty ACP target")
    return errors


def document_errors(document: str) -> list[str]:
    """Validate the pinned surface ledger's category, row, status, and target coverage."""
    errors: list[str] = []

    commit_token = f"`{AUDITED_AHP_COMMIT}`"
    if commit_token not in document:
        errors.append(f"audit commit: missing inline token {commit_token}")

    for category, items in INVENTORIES.items():
        start, end = SECTION_BOUNDS[category]
        section_text = section(document, start, end)
        if section_text is None:
            errors.append(f"{category}: missing section boundaries {start!r} / {end!r}")
            continue
        errors.extend(mapped_row_errors(category, items, table_rows(section_text)))

    for name, (start, end, expected_rows) in SUPPLEMENTAL_TABLES.items():
        section_text = section(document, start, end)
        if section_text is None:
            errors.append(f"{name}: missing section boundaries {start!r} / {end!r}")
            continue
        rows = table_rows(section_text)
        if len(rows) != expected_rows:
            errors.append(f"{name}: expected {expected_rows} mapping rows, found {len(rows)}")
        for row_index, row in enumerate(rows, start=1):
            if len(row) < 3 or "**" not in row[-2] or not row[-1]:
                errors.append(f"{name}: row {row_index} lacks a status and nonempty ACP target")

    return errors


def main() -> int:
    errors = inventory_errors()

    try:
        document = PARITY_RFD.read_text(encoding="utf-8")
    except OSError as error:
        errors.append(f"cannot read {PARITY_RFD}: {error}")
    else:
        errors.extend(document_errors(document))

    if errors:
        print("AHP coverage check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    total = sum(EXPECTED_COUNTS.values())
    print(
        f"AHP surface inventory OK: {total} pinned entries across "
        f"{len(INVENTORIES)} categories at {AUDITED_AHP_COMMIT[:12]}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
