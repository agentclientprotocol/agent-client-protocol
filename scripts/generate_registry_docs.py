#!/usr/bin/env python3
"""
Generates registry documentation from the ACP Agent Registry CDN.

This script fetches the registry.json from the CDN and generates
an MDX page with agent cards for the documentation site.

Usage:
    python scripts/generate_registry_docs.py

Environment variables:
    REGISTRY_URL: Override the default CDN URL (optional)
"""

from __future__ import annotations

import json
import os
import re
import sys
import time
import urllib.request
from pathlib import Path

REGISTRY_URL = os.environ.get(
    "REGISTRY_URL",
    "https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json",
)
ICON_BASE_URL = "https://cdn.agentclientprotocol.com/registry/v1/latest"

ROOT = Path(__file__).resolve().parents[1]
DOCS_DIR = ROOT / "docs"
TEMPLATE_PATH = DOCS_DIR / "get-started" / "_registry_agents.mdx"
OUTPUT_PATH = DOCS_DIR / "get-started" / "registry.mdx"
PLACEHOLDER = "$$AGENTS_CARDS$$"

# SVG attribute mappings for JSX compatibility
SVG_ATTR_REPLACEMENTS = {
    "fill-rule": "fillRule",
    "clip-rule": "clipRule",
    "clip-path": "clipPath",
    "stroke-width": "strokeWidth",
    "stroke-linecap": "strokeLinecap",
    "stroke-linejoin": "strokeLinejoin",
    "stroke-miterlimit": "strokeMiterlimit",
    "stroke-dasharray": "strokeDasharray",
    "stroke-dashoffset": "strokeDashoffset",
    "stroke-opacity": "strokeOpacity",
    "fill-opacity": "fillOpacity",
    "stop-color": "stopColor",
    "stop-opacity": "stopOpacity",
    "vector-effect": "vectorEffect",
}


def _escape_html(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


def _escape_text(text: str) -> str:
    return _escape_html(text).replace("\n", " ")


def _sanitize_svg(svg: str) -> str:
    """Sanitize SVG for JSX embedding with currentColor support."""
    svg = svg.strip()
    # Remove XML declaration
    svg = re.sub(r"<\?xml[^?]*\?>\s*", "", svg)
    # Remove non-SVG elements (e.g. Inkscape metadata)
    svg = re.sub(r"<(defs|sodipodi:\w+|inkscape:\w+)\b[^>]*/>", "", svg)
    svg = re.sub(r"<(defs|sodipodi:\w+|inkscape:\w+)\b[^>]*>.*?</\1>", "", svg, flags=re.DOTALL)
    # Remove namespace and Inkscape/sodipodi attributes
    svg = re.sub(r'\s+xmlns:\w+="[^"]*"', "", svg)
    svg = re.sub(r'\s+(sodipodi|inkscape):\w+="[^"]*"', "", svg)
    # Remove existing width/height/class attributes
    svg = re.sub(r'\s(width|height)="[^"]*"', "", svg)
    svg = re.sub(r'\sclass="[^"]*"', "", svg)
    # Add JSX-compatible attributes
    svg = re.sub(
        r"<svg\b",
        (
            '<svg width="20" height="20" '
            'className="agent-icon" '
            'aria-hidden="true" focusable="false"'
        ),
        svg,
        count=1,
    )
    # Convert hyphenated attributes to camelCase for JSX
    for old, new in SVG_ATTR_REPLACEMENTS.items():
        svg = svg.replace(f"{old}=", f"{new}=")
    return svg


def _make_request(url: str, timeout: int = 30) -> bytes:
    """Make HTTP request with proper headers."""
    req = urllib.request.Request(url, headers={"User-Agent": "ACP-Registry-Docs/1.0"})
    with urllib.request.urlopen(req, timeout=timeout) as response:
        return response.read()


def _fetch_registry() -> dict:
    """Fetch registry.json from CDN."""
    print(f"Fetching registry from {REGISTRY_URL}...")
    data = json.loads(_make_request(REGISTRY_URL).decode("utf-8"))
    print(f"Fetched {len(data.get('agents', []))} agents")
    return data


def _fetch_icon_svg(agent_id: str, retries: int = 3) -> str | None:
    """Fetch and sanitize SVG icon from CDN.

    Returns the sanitized SVG string, or None if all retries fail.
    """
    url = f"{ICON_BASE_URL}/{agent_id}.svg"
    for attempt in range(1, retries + 1):
        try:
            svg = _make_request(url, timeout=10).decode("utf-8")
            return _sanitize_svg(svg)
        except Exception as e:
            print(f"Warning: Could not fetch icon for {agent_id} (attempt {attempt}/{retries}): {e}")
            if attempt < retries:
                time.sleep(2)
    return None


def _fetch_all_icons(agents: list[dict]) -> dict[str, str]:
    """Fetch icons for all agents from the CDN.

    Returns a dict mapping agent_id -> sanitized SVG string.
    Raises SystemExit if any icon fails to fetch after retries.
    """
    icons: dict[str, str] = {}
    failed: list[str] = []

    for agent in agents:
        agent_id = agent.get("id", "-")
        svg = _fetch_icon_svg(agent_id)
        if svg is None:
            failed.append(agent_id)
        else:
            icons[agent_id] = svg

    if failed:
        print(f"Error: Failed to fetch icons for {len(failed)} agent(s): {', '.join(failed)}")
        print("Aborting to prevent publishing docs with missing icons.")
        sys.exit(1)

    return icons


def _render_agent_cards(agents: list[dict], icons: dict[str, str]) -> str:
    """Render agent cards as MDX components."""
    # Sort agents by name
    agents = sorted(agents, key=lambda a: a.get("name", "").lower())

    lines: list[str] = ["<CardGroup cols={3}>"]

    for agent in agents:
        agent_id = agent.get("id", "-")
        name = agent.get("name", agent_id)
        version = _escape_text(agent.get("version", "-"))
        repository = agent.get("repository", "")
        icon_svg = icons.get(agent_id)

        lines.append("  <Card")
        lines.append(f'    title="{_escape_html(name)}"')
        if repository:
            lines.append(f'    href="{_escape_html(repository)}"')
        if icon_svg:
            lines.append("    icon={")
            for line in icon_svg.splitlines():
                lines.append(f"      {line}")
            lines.append("    }")
        lines.append("  >")
        version_text = version if version not in ("", "-") else "version unknown"
        lines.append('    <p class="text-xs mt-3">')
        lines.append(f"      <code>{_escape_text(version_text)}</code>")
        lines.append("    </p>")
        lines.append("  </Card>")

    lines.append("</CardGroup>")
    return "\n".join(lines)


def main() -> None:
    # Ensure output directory exists
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)

    # Check if template exists
    if not TEMPLATE_PATH.exists():
        print(f"Error: Template file not found at {TEMPLATE_PATH}")
        print("Please create the template file first.")
        raise SystemExit(1)

    # Phase 1: Fetch all data (registry + icons) — abort on any failure
    registry = _fetch_registry()
    agents = registry.get("agents", [])

    if not agents:
        print("Warning: No agents found in registry")

    icons = _fetch_all_icons(agents)

    # Phase 2: Render and write — only reached if all fetches succeeded
    template = TEMPLATE_PATH.read_text()
    cards = _render_agent_cards(agents, icons)
    output = template.replace(PLACEHOLDER, cards)

    # Write output
    OUTPUT_PATH.write_text(output)
    print(f"Generated {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
