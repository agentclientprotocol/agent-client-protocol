---
title: "Session Delete"
---

Author(s): [@chazcb](https://github.com/chazcb)
Champion: [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

> What are you proposing to change?

Add a capability-gated `session/delete` method so clients can remove sessions from `session/list`. This complements `session/list` by giving users control over which sessions appear in their session history.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

The [`session/list` RFD](/rfds/session-list) introduced the ability for clients to enumerate sessions. However, there's no standard way to remove sessions from this list. Without `session/delete`, users have no control over their session history—old sessions accumulate, and clients must implement non-standard deletion mechanisms or rely on agent-specific cleanup policies.

## What we propose to do about it

> What are you proposing to improve the situation?

Add a `session/delete` JSON-RPC method that is capability-gated. Agents advertise support via `sessionCapabilities.delete` in their initialization response.

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "protocolVersion": 1,
    "agentCapabilities": {
      "sessionCapabilities": {
        "delete": {}
      }
    }
  }
}
```

### Method

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "session/delete",
  "params": {
    "sessionId": "sess_abc123def456"
  }
}
```

### Request Parameters

| Field       | Type        | Required | Description           |
| ----------- | ----------- | -------- | --------------------- |
| `sessionId` | `SessionId` | Yes      | The session to delete |

### Response

On success, returns an empty result:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {}
}
```

### Semantics

- **Capability-gated**: Agents MUST NOT accept `session/delete` calls unless they advertised `sessionCapabilities.delete` at initialization.
- **Removes from list**: The primary effect is that deleted sessions no longer appear in `session/list` results.
- **Implementation-defined storage behavior**: Agents may implement soft delete (mark as hidden) or hard delete (remove data). The protocol does not prescribe which.
- **Implementation-defined load behavior**: Agents may choose what happens when a client calls `session/load` on a deleted session—return the session anyway, return an error, or any other behavior. The protocol does not prescribe which.
- **Idempotent**: Deleting an already-deleted session (or a session that never existed) SHOULD succeed silently rather than error.

## Alternatives considered

### Automatic lifecycle policies only

Rely on agents to implement their own session retention policies (e.g., delete sessions older than 30 days) without exposing user control.

**Tradeoffs**: Users have no control over which sessions are kept. A session the user wants to keep might be deleted, or a session the user wants gone might persist.

### Add a `hidden` flag to sessions

Instead of delete, allow users to mark sessions as hidden. They'd still exist but not appear in `session/list` by default.

**Tradeoffs**: More complex—requires filter parameters on `session/list` to show/hide hidden sessions. For most use cases, delete is simpler and matches user expectations.

### Batch deletion

Support deleting multiple sessions in one call via a `sessionIds` array.

**Tradeoffs**: Could be added later as an extension. Single-session delete covers the common case and keeps the initial implementation simple.

## Shiny future

> How will things play out once this feature exists?

Users can manage their session history, and all ACP clients can offer this using the same protocol method rather than implementing their own mechanisms.

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

1. **Schema**: Add `session/delete` method definition, `DeleteSessionRequest` and `DeleteSessionResponse` types.
2. **Capabilities**: Add `sessionCapabilities.delete` capability flag.
3. **Protocol**: Add `session/delete` to method tables.
4. **Docs**: Update session management docs to include deletion.

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

### Why not prescribe soft vs hard delete?

Different agents have different storage architectures and compliance requirements. Some may need to retain data for auditing; others may want to free storage immediately. The protocol focuses on the user-facing behavior (removed from list) and leaves storage decisions to implementers.

### Why not prescribe behavior for loading deleted sessions?

Similar reasoning—some agents may want to allow "undelete" by loading a soft-deleted session, others may want a clean error. The protocol provides the deletion mechanism; agents decide the semantics that fit their use case.

### Should delete require confirmation?

No. Confirmation UX is a client concern. The protocol provides the delete operation; clients can add confirmation dialogs, undo functionality, or other UX patterns as they see fit.

### What if the session is currently active?

Agents may reject deletion of active sessions or handle it however they choose. This is implementation-defined. A reasonable approach is to allow deletion—the session simply won't appear in future `session/list` calls.

### Why is this a separate RFD from session/list?

The [`session/list` RFD](/rfds/session-list#what-about-session-deletion) explicitly deferred deletion to keep scope focused. Now that `session/list` is established, `session/delete` is a natural complement.

## Revision history

- **2025-02-03**: Fixed capability example to use agent capability (initialize response)
- **2025-01-24**: Initial draft
