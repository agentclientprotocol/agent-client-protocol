---
title: "Agent Telemetry Export"
---

- Author(s): [@codefromthecrypt](https://github.com/codefromthecrypt)
- Champion: [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

> What are you proposing to change?

Define how agents export telemetry (logs, metrics, traces) to clients without tunneling it over the ACP transport. Clients run a local telemetry receiver and pass standard OpenTelemetry environment variables when launching agents. This keeps telemetry out-of-band and enables editors to display agent activity, debug issues, and integrate with observability backends.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

ACP defines how clients launch agents as subprocesses and communicate over stdio. The [meta-propagation RFD](./meta-propagation) addresses trace context propagation via `params._meta`, enabling trace correlation. However, there is no convention for how agents should export the actual telemetry data (spans, metrics, logs).

Without a standard approach:

1. **No visibility into agent behavior** - Editors cannot display what agents are doing (token usage, tool calls, timing)
2. **Difficult debugging** - When agents fail, there's no structured way to capture diagnostics
3. **Fragmented solutions** - Each agent/client pair invents their own telemetry mechanism
4. **Credential exposure risk** - If agents need to send telemetry directly to backends, they need credentials

Tunneling telemetry over the ACP stdio transport is problematic:

- **Head-of-line blocking** - Telemetry traffic could delay agent messages
- **Implementation burden** - ACP would need to define telemetry message formats
- **Coupling** - Agents would need ACP-specific telemetry code instead of standard SDKs

## What we propose to do about it

> What are you proposing to improve the situation?

Clients that want to receive agent telemetry run a local OTLP (OpenTelemetry Protocol) receiver and inject environment variables when launching agent subprocesses:

```
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
OTEL_SERVICE_NAME=agent-name
```

Agents using OpenTelemetry SDKs auto-configure from these variables. The client's receiver can:

- Display telemetry in the editor UI (e.g., token counts, timing, errors)
- Forward telemetry to the client's configured observability backend
- Add client-side context before forwarding

This follows the [OpenTelemetry collector deployment pattern](https://opentelemetry.io/docs/collector/deployment/agent/) where a local receiver proxies telemetry to backends.

### Architecture

```
┌────────────────────────────────────────────────────────────┐
│ Client/Editor                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ ACP Handler  │    │OTLP Receiver │───▶│   Exporter   │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
└────────┬─────────────────────▲──────────────────┬──────────┘
         │ stdio               │ HTTP             │
         ▼                     │                  ▼
┌─────────────────────┐        │         ┌───────────────────┐
│ Agent Process       │        │         │ Observability     │
│  ┌──────────────┐   │        │         │ Backend           │
│  │ ACP Agent    │   │        │         └───────────────────┘
│  ├──────────────┤   │        │
│  │ OTEL SDK     │────────────┘
│  └──────────────┘   │
└─────────────────────┘
```

### Discovery

Environment variables must be set before launching the subprocess, but ACP capability exchange happens after connection. Options for discovery:

1. **Optimistic injection** - Clients inject OTEL environment variables unconditionally. Agents without OpenTelemetry support simply ignore them. This is pragmatic since environment variables are low-cost and OTEL SDKs handle misconfiguration gracefully.

2. **Registry metadata** - Agent registries (like the one proposed in PR #289) could include telemetry support in agent manifests, letting clients know ahead of time.

3. **Manual configuration** - Users configure their client to enable telemetry collection for specific agents.

## Shiny future

> How will things will play out once this feature exists?

1. **Editor integration** - Editors can show agent activity: token usage, tool call timing, model switches, errors
2. **Unified debugging** - When agents fail, structured telemetry is available for diagnosis
3. **End-to-end traces** - Combined with `params._meta` trace propagation, traces flow from client through agent to any downstream services
4. **No credential sharing** - Agents never see backend credentials; the client handles authentication
5. **Standard SDKs** - Agent authors use normal OpenTelemetry SDKs that work in any context, not ACP-specific code

## Implementation details

> Tell me more about your implementation. What is your detailed implementation plan?

### 1. Create `docs/protocol/observability.mdx`

Add a new protocol documentation page covering observability practices for ACP. This page will describe:

**For Clients/Editors:**

- Running an OTLP receiver to collect agent telemetry
- Injecting `OTEL_EXPORTER_*` environment variables when launching agent subprocesses
- Respecting user-configured `OTEL_*` variables (do not override if already set)
- Forwarding telemetry to configured backends with client credentials

**For Agent Authors:**

- Using OpenTelemetry SDKs with standard auto-configuration
- Recommended spans, metrics, and log patterns for agent operations
- How telemetry flows when `OTEL_*` variables are present vs absent

### 2. Update `docs/protocol/extensibility.mdx`

Add a section linking to the new observability doc, similar to how extensibility concepts relate to other protocol features. Add a brief mention that observability practices (telemetry export) are documented separately.

### 3. Update `docs/docs.json`

Add `protocol/observability` to the Protocol navigation group.

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

### How does this relate to trace propagation in `params._meta`?

They are complementary:

- **Trace propagation** (`params._meta` with `traceparent`, etc.) passes trace context so spans can be correlated
- **Telemetry export** (this RFD) defines where agents send the actual span/metric/log data

Both are needed for end-to-end observability.

### What if an agent doesn't use OpenTelemetry?

Agents without OTEL SDKs simply ignore the environment variables. No harm is done. Over time, as more agents adopt OpenTelemetry, the ecosystem benefits.

### What if the user already configured `OTEL_*` environment variables?

If `OTEL_*` variables are already set in the environment, clients should not override them. User-configured telemetry settings take precedence, allowing users to direct agent telemetry to their own backends when desired.

### Why not define ACP-specific telemetry messages?

This would duplicate OTLP functionality, add implementation burden to ACP, and force agent authors to use non-standard APIs. Using OTLP means agents work with standard tooling and documentation.

### What about agents that aren't launched as subprocesses?

This RFD focuses on the stdio transport where clients launch agents. For other transports (HTTP, etc.), agents would need alternative configuration mechanisms, which could be addressed in future RFDs.

### What alternative approaches did you consider, and why did you settle on this one?

1. **Tunneling telemetry over ACP** - Rejected due to head-of-line blocking concerns and implementation complexity
2. **Agents export directly to backends** - Rejected because it requires sharing credentials with agents
3. **File-based telemetry** - Rejected because it doesn't support real-time display and adds complexity

The environment variable approach:

- Uses existing standards (OTLP, OpenTelemetry SDK conventions)
- Keeps telemetry out-of-band from ACP messages
- Lets clients control where telemetry goes without exposing credentials
- Requires no changes to ACP message formats

## Revision history

- 2025-12-04: Initial draft
