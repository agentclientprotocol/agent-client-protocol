---
title: "Request Cancellation Mechanism"
---

- Author(s): [Artem Bukhonov](https://github.com/nerzhulart)
- Champion: [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

Introduce a standardized per-request cancellation mechanism for the Agent Client Protocol, inspired by the [Language Server Protocol (LSP)](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#cancelRequest), to enable a more granular cancellation of requests where individual JSON-RPC requests can be cancelled one by one.

## Status quo

The JSON-RPC specification doesn't define any standard mechanism for request cancellation and leaves it up to the implementation. Currently, ACP has some ad-hoc cancellation mechanisms for specific features (like prompt turn cancellation via `session/cancel`), but lacks a general-purpose, per-request cancellation mechanism.

This creates the following inconveniences:

- cancellation should be handled for each feature separately
- some languages that support handy cancellation mechanisms (C#, Kotlin, etc.) can't implement general-purpose request cancellation using ACP low-level machinery, and rather developers should manually call per-feature cancellation methods

## What we propose to do about it

Implement an **optional** `$/cancel_request` notification method (inspired by the Language Server Protocol) to both Agent and Client that uses JSON-RPC 2.0 notification format, allowing either party (client or agent) to cancel any outstanding request by its ID.

The mechanism will be:

- **Optional**: Not all implementations may support this feature, but it is recommended for those that do.
- **Flexible**: Provides two response options when cancellation is received:
  1. An error response with the standard cancellation error code (-32800)
  2. A valid response with partial or cancelled data (when meaningful partial results exist)

This approach balances flexibility with standardization, allowing implementations to opt-in to cancellation support while providing predictable behavior when enabled.

## Shiny future

Once implemented, this enables:

- **SDK integration layer**: Default mechanism that ACP SDKs can automatically wire to native language cancellation (C# CancellationToken, Kotlin Job, Go context.Context, JavaScript AbortController, etc.)
- Individual JSON-RPC request cancellation without affecting other concurrent requests
- Universal fallback for any request when feature-specific cancellation methods don't exist
- Consistent cancellation behavior from both external `$/cancel_request` and internal cancellation triggers
- Standard error response (`-32800`) or partial results when requests are cancelled regardless of cancellation source

In a future version, we could potentially deprecate the `session/cancel` notification in favor of the more general approach, as it would still provide the same functionality but with more flexibility and consistency.

## Implementation details and plan

### Protocol Changes

#### Cancellation Method

Add the `$/cancel_request` notification method to the JSON-RPC protocol:

```typescript
interface CancelNotification {
  method: "$/cancel_request";
  params: {
    requestId: string | number; // ID of request to cancel
  };
}
```

### Cancellation Behavior

Either party can send `$/cancel_request` to cancel requests. Notifications whose methods start with '$/' are messages which are protocol implementation dependent and might not be implementable in all clients or agents. For example if the implementation uses a single threaded synchronous programming language then there is little it can do to react to a `$/cancel_request` notification. If an agent or client receives notifications starting with '$/' it is free to ignore the notification.

The **receiving party** is **NOT** required to:

- Perform special handling for unsupported cancellation requests
- Return custom errors for unsupported `$/cancel_request` notifications

#### Cancellation Processing

When a `$/cancel_request` notification is received by a supporting implementation, it:

- **MUST** cancel the corresponding request activity and all nested activities related to that request
- **MAY** finish sending any pending notifications before responding
- **MUST** send one of these responses for the original request:
  - A valid response with appropriate data (such as partial results or cancellation marker)
  - An error response with code [`-32800` (Request Cancelled)](./schema#errorcode)

#### Internal Cancellation

Requests can also be cancelled internally by the executing party without receiving `$/cancel_request`:

- **Client-side examples**: User closes IDE, switches to different project, file becomes unavailable
- **Agent-side examples**: LLM context limit reached, internal timeout, resource constraints

When internal cancellation occurs, the executing party **SHOULD**:

1. Send the same `-32800` (Cancelled) error response as if `$/cancel_request` was received
2. Ensure consistent behavior regardless of cancellation source

### Error Code

Add standard JSON-RPC error code `-32800` for cancelled requests:

- Code: `-32800`
- Message: "Request cancelled"
- Meaning: Execution of the method was aborted either due to a cancellation request from the caller or because of resource constraints or shutdown.

## Frequently asked questions

### What alternative approaches did you consider, and why did you settle on this one?

The core need is to add **granular cancellation** as a general mechanism for individual JSON-RPC requests, while **feature-specific cancellation methods** (like `session/cancel`) remain useful for cases requiring additional domain semantics.

We selected the **LSP-style `$/cancel_request`** approach because:

- Serves as a **default cancellation layer** that SDK implementations can easily map to native language cancellation mechanisms
- Proven pattern familiar to developers from LSP ecosystem
- Works across all JSON-RPC transports (HTTP, WebSocket, stdio, pipes)
- Provides universal fallback when feature-specific cancellation doesn't exist
- Complements existing feature-specific methods rather than replacing them

### How does this relate to existing cancellation mechanisms like `session/cancel`?

The `$/cancel_request` mechanism is complementary to feature-specific cancellation:

- `$/cancel_request`: Generic cancellation for any JSON-RPC request by ID
- `session/cancel`: Feature-specific cancellation with additional semantics (e.g., cancels entire prompt turn context, triggers specific cleanup logic)

Both mechanisms serve different purposes:

**Feature-specific methods** like `session/cancel` provide:

- Domain-specific semantics and behavior
- Structured cleanup for complex operations
- Context-aware cancellation logic

**Generic `$/cancel_request`** provides:

- Default cancellation layer that bridges programming language cancellation mechanisms (C# CancellationToken, Kotlin Job, Go context.Context, etc.) with ACP
- Universal fallback for any request when no feature-specific method exists
- Simple ID-based targeting for SDK convenience
- Standardized error responses

Implementations can use both: feature-specific methods for rich semantics, and `$/cancel_request` for simple per-request cancellation.

Note: it is possible that `session/cancel` could be replaced by the more generic `$/cancel_request` in future versions of the protocol.

#### Example: Cascading Cancellation Flow

```mermaid
sequenceDiagram
    participant Client
    participant Agent

    Note over Client,Agent: 1. Session prompt in progress
    Client->>Agent: session/prompt (id=1, "Analyze file X")
    Agent-->>Client: session/update (agent started processing)

    Note over Client,Agent: 2. Agent makes concurrent requests
    Agent->>Client: terminal/create (id=2, "grep pattern file.txt")
    Agent->>Client: session/request_permission (id=3, "read sensitive file")

    Note over Client,Agent: 3. Client cancels the prompt turn
    Client->>Agent: session/cancel (sessionId)

    Note over Client,Agent: 4. Agent cascades cancellation internally
    Agent->>Client: $/cancel_request (id=2) [terminal request]
  Agent->>Client: $/cancel_request (id=3) [permission request]

    Note over Client,Agent: 5. Client confirms individual cancellations
    Client->>Agent: response to id=2 (error -32800 "Cancelled")
    Client->>Agent: response to id=3 (error -32800 "Cancelled")

    Note over Client,Agent: 6. Agent completes prompt cancellation
    Agent->>Client: response to id=1 (stopReason: "cancelled")
```

### What happens if a request completes before the cancellation is processed?

If a request completes normally before the cancellation notification is processed, the implementation should:

1. Send the normal response (not a cancellation error)
2. Ignore the subsequent cancellation notification for that request ID

This ensures clients always receive meaningful responses and prevents race conditions.

### How should implementations handle cascading cancellation?

When a request is cancelled, implementations should:

1. Cancel the primary request activity
2. Propagate cancellation to any nested/child requests
3. Clean up resources associated with the entire request tree
4. Send cancellation responses for all affected requests

This ensures complete cleanup and prevents resource leaks.

## Revision history

- 2025-11-13: Initial version converted from PR #183
- 2025-12-05: Updated with current implementation.
- 2025-12-09: Mirror LSP behavior.
