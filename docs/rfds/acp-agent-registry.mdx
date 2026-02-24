---
title: "ACP Agent Registry"
---

**Author:** [@ignatov](https://github.com/ignatov)
**Champion:** [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

ACP needs a single, trusted registry of agents so clients can discover integrations, understand their capabilities, and configure them automatically. This RFD proposes (1) a canonical manifest format that every agent must publish, (2) a dedicated `agentclientprotocol/registry` repo where maintainers contribute those manifests, and (3) tooling that aggregates and publishes a searchable catalog for editors and other clients.

## Status quo

There is no canonical listing of ACP-compatible agents. Information lives in scattered READMEs or proprietary feeds, which makes it hard to:

- Let users discover agents directly inside ACP-aware clients.
- Ensure protocol-version compatibility or capability coverage.
- Keep metadata consistent (hosting model, license, etc.).

Every editor builds bespoke manifests or scrapes GitHub, leading to duplication and stale data.

## Agent manifest format (core proposal)

Each agent advertises itself via a manifest stored as `<id>/agent.json` in the registry repo.

Fields marked with **\*** are required:

| Field                 | Description                                                                                                                                                                                                                                                                      |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id` **\***           | Unique agent identifier. Lowercase letters, digits, and hyphens; must start with a letter (pattern: `^[a-z][a-z0-9-]*$`). Also the folder name in the registry repo.                                                                                                             |
| `name` **\***         | Human-readable display name.                                                                                                                                                                                                                                                     |
| `version` **\***      | Semantic version of the agent release (e.g. `1.0.0`).                                                                                                                                                                                                                            |
| `description` **\***  | Brief description of the agent's functionality and purpose.                                                                                                                                                                                                                      |
| `distribution` **\*** | Object describing how to obtain and run the agent. Supports three distribution types: `binary` (platform-specific archives), `npx` (Node packages), and `uvx` (Python packages). At least one distribution type must be provided. See [Distribution](#distribution) for details. |
| `repository`          | Source code repository URL.                                                                                                                                                                                                                                                      |
| `authors`             | Array of author/organization names.                                                                                                                                                                                                                                              |
| `license`             | SPDX license identifier or `"proprietary"`.                                                                                                                                                                                                                                      |
| `icon`                | Path to icon file (relative path or absolute URL). Must be SVG format, 16×16, monochrome using `currentColor` (enables light/dark theme adaptation). See [Icon requirements](#icon-requirements).                                                                                |

### Distribution

The `distribution` object supports three mutually independent strategies. An agent may provide one or more:

#### `binary`

Platform-specific archive downloads. Keyed by `<os>-<arch>` targets:

| Target            | OS      | Architecture |
| ----------------- | ------- | ------------ |
| `darwin-aarch64`  | macOS   | ARM64        |
| `darwin-x86_64`   | macOS   | x86-64       |
| `linux-aarch64`   | Linux   | ARM64        |
| `linux-x86_64`    | Linux   | x86-64       |
| `windows-aarch64` | Windows | ARM64        |
| `windows-x86_64`  | Windows | x86-64       |

When using `binary` distribution, builds **must be provided for all three operating systems** (darwin, linux, windows). CI will reject entries that only cover a subset.

Each target is an object with:

| Field     | Required | Description                         |
| --------- | -------- | ----------------------------------- |
| `archive` | Yes      | URL to download archive             |
| `cmd`     | Yes      | Command to execute after extraction |
| `args`    | No       | Array of command-line arguments     |
| `env`     | No       | Object of environment variables     |

#### `npx` / `uvx`

Package-manager-based distribution (Node via `npx`, Python via `uvx`). Each is an object with:

| Field     | Required | Description                               |
| --------- | -------- | ----------------------------------------- |
| `package` | Yes      | Package name (with optional version spec) |
| `args`    | No       | Array of command-line arguments           |
| `env`     | No       | Object of environment variables           |

### Icon requirements

Icons must meet the following requirements to pass CI validation:

- **SVG format** — only `.svg` files are accepted.
- **16×16 dimensions** — via `width`/`height` attributes or `viewBox`.
- **Monochrome using `currentColor`** — all `fill` and `stroke` values must use `currentColor` or `none`. Hardcoded colors (e.g. `fill="#FF5500"`, `fill="red"`) are rejected.

Using `currentColor` lets icons adapt automatically to the client's light or dark theme.

### Example: binary distribution

```jsonc
{
  "id": "someagent",
  "name": "SomeAgent",
  "version": "1.0.0",
  "description": "Agent for code editing",
  "repository": "https://github.com/example/someagent",
  "authors": ["Example Team"],
  "license": "MIT",
  "icon": "icon.svg",
  "distribution": {
    "binary": {
      "darwin-aarch64": {
        "archive": "https://github.com/example/someagent/releases/latest/download/someagent-darwin-arm64.zip",
        "cmd": "./someagent",
        "args": ["acp"],
      },
      "darwin-x86_64": {
        "archive": "https://github.com/example/someagent/releases/latest/download/someagent-darwin-x64.zip",
        "cmd": "./someagent",
        "args": ["acp"],
      },
      "linux-aarch64": {
        "archive": "https://github.com/example/someagent/releases/latest/download/someagent-linux-arm64.zip",
        "cmd": "./someagent",
        "args": ["acp"],
      },
      "linux-x86_64": {
        "archive": "https://github.com/example/someagent/releases/latest/download/someagent-linux-x64.zip",
        "cmd": "./someagent",
        "args": ["acp"],
      },
      "windows-x86_64": {
        "archive": "https://github.com/example/someagent/releases/latest/download/someagent-windows-x64.zip",
        "cmd": "./someagent.exe",
        "args": ["acp"],
        "env": {
          "SOMEAGENT_MODE_KEY": "",
        },
      },
    },
  },
}
```

### Example: package distribution

```jsonc
{
  "id": "pyagent",
  "name": "PyAgent",
  "version": "2.1.0",
  "description": "A Python-based ACP agent",
  "repository": "https://github.com/example/pyagent",
  "license": "Apache-2.0",
  "distribution": {
    "uvx": {
      "package": "pyagent@latest",
      "args": ["--mode", "acp"],
    },
  },
}
```

## Registry schema

The aggregated `registry.json` file conforms to the registry schema and contains:

| Field     | Description                                                                                               |
| --------- | --------------------------------------------------------------------------------------------------------- |
| `version` | Registry schema version (semver, e.g. `1.0.0`).                                                           |
| `agents`  | Array of agent entries (each following the agent manifest schema above, sourced from `agent.json` files). |

## Authentication requirements

To be listed in the registry, an agent **must support at least one** of the following authentication methods:

- **Agent Auth** — the agent handles the OAuth flow independently (opens the user's browser, runs a local callback server, exchanges the authorization code for tokens).
- **Terminal Auth** — the agent provides an interactive terminal-based setup experience (launched with additional args/env specified in the auth method).

CI verifies this by checking that the agent returns an `authMethods` array in its `initialize` response, with at least one method. See the [ACP auth methods RFD](./auth-methods) for the full specification.

## What we propose to do about it

1. **Manifest spec** (above) becomes normative; we publish the JSON Schema and validator script so maintainers can lint locally.
2. **Registry repository** `github.com/agentclientprotocol/registry`:
   - Structure: `<id>/agent.json`, optional `icon.svg`, optional `README.md`.
   - CI validates manifests on every PR: schema compliance, slug uniqueness, icon format (16×16 SVG, monochrome `currentColor`), URL accessibility for all distribution URLs, authentication support via ACP handshake, and binary OS coverage.
   - Push to `main` triggers a build that aggregates all entries into `registry.json` and publishes versioned + `latest` GitHub releases.
3. **Aggregated outputs**:
   - `registry.json`: deterministic list of all agents with icons copied to `dist/<id>.svg`.
4. **Distribution & search**:
   - Clients fetch `registry.json` from `https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json`.
   - Static site offers filters for deployment model, license, and distribution type.

## Shiny future

- Agent maintainers make PRs to update their manifests; CI keeps data clean.
- Automated version updates run hourly, checking npm, PyPI, and GitHub releases for new versions of registered agents and opening PRs automatically.
- Editors/clients can bootstrap ACP support by fetching one JSON file and filtering locally.
- The ACP website displays the same data for humans, ensuring consistency.
- Package-based distribution (`npx`, `uvx`) lowers the barrier for agents that don't need platform-specific binaries.

## Implementation details and plan

**Phase 1 – Spec & repo bootstrap**

- Finalize JSON Schema and documentation.
- Create registry repo with CI (GitHub Actions) that validates on PRs and publishes on merge.
- Seed with reference agents.
- Implement automated version update workflow (hourly cron via GitHub Actions).
- Enforce authentication requirements via CI handshake verification.

## Revision history

- 2025-11-28: Initial draft.
- 2025-12-16: Minors.
- 2026-02-04: Updated to match latest schema — removed `schema_version`, `homepage`, `capabilities`, and `auth` fields; added `icon` field; restructured `distribution` into `binary`, `npx`, and `uvx` types.
