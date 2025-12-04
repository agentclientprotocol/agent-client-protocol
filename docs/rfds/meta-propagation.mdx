---
title: "Meta Field Propagation Conventions"
---

- Author(s): [Adrian Cole](https://github.com/codefromthecrypt)
- Champion: [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

Document `params._meta` as the convention for propagating metadata from clients to agents, such as trace identifiers or correlation IDs. This aligns with [MCP](https://modelcontextprotocol.io/), enabling shared instrumentation since both protocols use stdio JSON-RPC transports.

## Status quo

ACP clients already propagate context to agents via `_meta`. For example, `requestId` is used for request correlation in [AionUi](https://github.com/iOfficeAI/AionUi/blob/main/src/common/codex/types/eventData.ts#L12-L16).

However, the [extensibility](../protocol/extensibility) documentation does not specify the `_meta` type or document its use for propagation. Without documentation, parties must coordinate ad hoc, which can lead to portability accidents (such as one side using `_meta.traceparent` and the other `_meta.otel.traceparent`). Documenting that propagated fields are root keys in `_meta` prevents this.

## What we propose to do about it

Update the [extensibility](../protocol/extensibility#the-_meta-field) documentation with two changes:

1. Add the type `{ [key: string]: unknown }` to the existing summary sentence. This type is compatible with MCP SDKs.
2. Add a new paragraph after the JSON example about propagation conventions.

## Shiny future

- Same instrumentation (OpenInference, etc.) works for both ACP and MCP.
- Observability tools can correlate traces across protocols.

## Implementation details

**Change 1**: Update the existing summary sentence in [extensibility](../protocol/extensibility#the-_meta-field):

```diff
-All types in the protocol include a `_meta` field that implementations can use to attach custom information.
+All types in the protocol include a `_meta` field with type `{ [key: string]: unknown }` that implementations can use to attach custom information.
```

**Change 2**: After the JSON example, before "Implementations **MUST NOT**", add:

> Clients may propagate fields to the agent for correlation purposes, such as `requestId`. The following root-level keys in `_meta` **SHOULD** be reserved for [W3C trace context](https://www.w3.org/TR/trace-context/) to guarantee interop with existing MCP implementations and OpenTelemetry tooling:
>
> - `traceparent`
> - `tracestate`
> - `baggage`

## FAQ

### Why document this now?

Clients already propagate context via `_meta`. Documenting prevents incompatible drift and enables shared tooling with MCP.

### Why reference MCP?

ACP and MCP are the two core agentic protocols, both using stdio JSON-RPC. Where `_meta` types are compatible, instrumentation code can be abstracted and reused for both:

Here are several MCP SDKs that propagate W3C trace-context in `_meta`:

- [MCP C# SDK](https://github.com/modelcontextprotocol/csharp-sdk) - native W3C trace-context propagation
- [OpenInference](https://github.com/Arize-ai/openinference) - Python and TypeScript MCP instrumentation (collaboration between Arize and Elastic)
- [curioswitch/mcp-go-sdk-otel](https://github.com/curioswitch/mcp-go-sdk-otel) - Go MCP instrumentation

## Revision history

- 2025-12-04: Implementation in extensibility docs
- 2025-11-28: Initial draft
