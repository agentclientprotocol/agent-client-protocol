---
title: "Boolean Config Option Type"
---

- Author(s): [fscarponi](https://github.com/fscarponi)
- Champion: [benbrandt](https://github.com/benbrandt)

## Elevator pitch

Add a new `boolean` type to session configuration options, enabling agents to expose simple ON/OFF toggles (e.g., "Brave Mode", "Read Only", "Produce Report") as first-class config options alongside the existing `select` type.

## Status quo

Currently, `SessionConfigKind` only supports the `select` type, which allows agents to expose dropdown-style selectors with a list of named values. This works well for choosing models, modes, or reasoning levels.

However, there is no native way to represent a simple boolean on/off toggle. To expose a boolean option today, agents must use a `select` with two artificial options (e.g., "on"/"off"), and clients need custom, non-agnostic logic to detect that a particular select is actually a boolean toggle. This defeats the purpose of a standardized protocol.

## What we propose to do about it

- Add a `SessionConfigBoolean` struct with a `current_value: bool` field
- Add a `Boolean(SessionConfigBoolean)` variant to the `SessionConfigKind` enum, discriminated by `"type": "boolean"`
- Add a `SessionConfigOptionValue` internally-tagged enum so that `SetSessionConfigOptionRequest` can carry both string values (for `select`) and boolean values (for `boolean`) with an explicit `type` discriminator
- Provide convenience constructors and `From` impls for ergonomic usage
- Update documentation and regenerate schema files

## Shiny future

Clients can natively render boolean config options as toggle switches or checkboxes, without any custom logic. Agents can expose options like "Brave Mode", "Produce Report", or "Read Only" in a standardized way that any ACP-compliant client understands out of the box.

## Implementation details and plan

### Wire format: declaring a boolean option

In a `session/new` response (or any response containing `configOptions`):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "sessionId": "sess_abc123",
    "configOptions": [
      {
        "id": "brave_mode",
        "name": "Brave Mode",
        "description": "Skip confirmation prompts and act autonomously",
        "type": "boolean",
        "currentValue": true
      },
      {
        "id": "mode",
        "name": "Session Mode",
        "category": "mode",
        "type": "select",
        "currentValue": "code",
        "options": [
          { "value": "ask", "name": "Ask" },
          { "value": "code", "name": "Code" }
        ]
      }
    ]
  }
}
```

### Wire format: setting a boolean option

The `session/set_config_option` request carries a `type` discriminator alongside the `value`. The `type` field describes the _shape_ of the value, not the option kind.

When `type` is absent the value is treated as a `SessionConfigValueId` string, preserving backwards compatibility with existing clients:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "session/set_config_option",
  "params": {
    "sessionId": "sess_abc123",
    "configId": "brave_mode",
    "type": "boolean",
    "value": true
  }
}
```

For select options the `type` field can be omitted (defaults to `value_id`):

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "session/set_config_option",
  "params": {
    "sessionId": "sess_abc123",
    "configId": "mode",
    "value": "code"
  }
}
```

The response returns the full set of config options with current values, as with `select`:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "configOptions": [
      {
        "id": "brave_mode",
        "name": "Brave Mode",
        "description": "Skip confirmation prompts and act autonomously",
        "type": "boolean",
        "currentValue": true
      },
      {
        "id": "mode",
        "name": "Session Mode",
        "category": "mode",
        "type": "select",
        "currentValue": "code",
        "options": [..]
      }
    ]
  }
}
```

Key changes:

1. `SessionConfigBoolean` struct with `current_value: bool`
2. `Boolean(SessionConfigBoolean)` variant in `SessionConfigKind` (tagged via `"type": "boolean"`)
3. `SessionConfigOptionValue` enum using `#[serde(tag = "type")]` with a `#[serde(untagged)]` fallback variant — the same pattern as `AuthMethod`:
   - `Boolean { value: bool }` — matched when `type` is `"boolean"`
   - `ValueId { value: SessionConfigValueId }` — untagged fallback when `type` is absent or unrecognised
4. `SessionConfigOptionValue` is flattened (`#[serde(flatten)]`) onto `SetSessionConfigOptionRequest`, producing top-level `type` and `value` fields on the wire
5. The `value` field type change in `SetSessionConfigOptionRequest` is gated behind `#[cfg(feature = "unstable_boolean_config")]` — without the feature the field remains `SessionConfigValueId`
6. `From` impls (`&str`, `SessionConfigValueId`, `bool`) ensure ergonomic construction
7. Wire-level backward compatible: existing JSON payloads without a `type` field remain valid via the untagged fallback

### Client capabilities

Per the existing protocol design, clients that receive a config option with an unrecognized `type` should ignore it. Since the agent is required to have a default value for every option, the agent can function correctly even if the client doesn't render or interact with the boolean option. No new client capability negotiation is needed.

## Frequently asked questions

### What alternative approaches did you consider, and why did you settle on this one?

We considered reusing the existing `select` type with a convention (e.g., options named "on"/"off"), but this would require clients to implement non-agnostic detection logic, which contradicts the goal of a standardized protocol. A dedicated `boolean` type is cleaner and lets clients render the appropriate UI control without guessing.

### Is this a breaking change?

On the wire/JSON level: no. When `type` is absent the value is treated as a `SessionConfigValueId`, so existing payloads deserialize correctly. On the Rust API level: the type of `SetSessionConfigOptionRequest.value` changes, but this is gated behind `unstable_boolean_config`. Without the feature flag the stable API is unchanged. With the feature flag, `From` impls ensure source compatibility for users of the `new()` constructor.

## Revision history

- 2026-02-24: Initial proposal
- 2026-03-05: Updated to reflect final implementation — `flag` renamed to `boolean`, value type changed from untagged `String | Bool` enum to internally-tagged enum with `type` discriminator and untagged `ValueId` fallback, feature-gated behind `unstable_boolean_config`
