---
title: "Streamable HTTP & WebSocket Transport"
---

- Author(s): [alexhancock](https://github.com/alexhancock), [jh-block](https://github.com/jh-block)
- Champion: [anna239](https://github.com/anna239)

## Elevator pitch

> What are you proposing to change?

ACP needs a standard remote transport. We propose a **single long-lived GET stream** for all server→client messages, with **POST** for client→server messages, and **WebSocket upgrade** as an alternative on the same endpoint. A single `/acp` endpoint supports two connectivity profiles:

- **Streamable HTTP (POST/GET/DELETE)** — Single long-lived SSE stream per connection for all server→client messages (responses and notifications). POST requests return immediately (202 Accepted, except `initialize`). Requires HTTP/2.
- **WebSocket upgrade (GET with `Upgrade: websocket`)** — persistent, full-duplex, low-latency bidirectional messaging.

Clients that support remote ACP over HTTP MUST support both Streamable HTTP and WebSocket. This allows servers to support only WebSocket if they choose, simplifying deployment.

Both profiles share the same JSON-RPC message format and ACP lifecycle as the existing **stdio** local subprocess transport.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

ACP only has stdio. There is no standard remote transport, which causes fragmentation as implementers invent their own HTTP layers, leading to incompatible SDKs and deployments.

## What we propose to do about it

> What are you proposing to improve the situation?

### 1. Adds an HTTP Transport

ACP adopts a streamable HTTP transport with three key characteristics:

1. **Single long-lived GET stream per connection** — All server→client messages (responses to requests and unsolicited notifications) are delivered via a single SSE stream opened with GET. This includes responses to client requests (correlated by JSON-RPC `id`), server-initiated notifications (no `id`), and server-to-client requests like `request_permission` (with `id`, client responds via POST). The GET stream is scoped to `Acp-Connection-Id` and delivers messages for all sessions within that connection. Session identity is carried in the JSON-RPC message body (`sessionId` field).

2. **POST requests return immediately (except initialize)** — Client→server messages are sent via POST. Most POST requests return `202 Accepted` immediately with an empty body. The actual response comes later on the GET stream, correlated by JSON-RPC `id`. The `initialize` request is special: it returns `200 OK` with a JSON response body containing capabilities and the `Acp-Connection-Id`. The `Acp-Connection-Id` is also included in the response header.

3. **Requires HTTP/2** — Streamable HTTP transport MUST use HTTP/2. This provides multiplexing for concurrent POST requests while maintaining a single long-lived GET stream, and improves efficiency for high-frequency message exchanges.

### 4. Adds WebSocket as a first-class upgrade on the same endpoint

A GET with `Upgrade: websocket` upgrades to a persistent bidirectional channel — same endpoint, same lifecycle model.

This is important for ACP, as it is more bidirectional in its nature as a protocol.

### 5. Requires cookie support on HTTP transports

Clients MUST accept, store, and return cookies set by the server on all HTTP-based transports (Streamable HTTP and WebSocket). Cookies MUST be sent on subsequent requests to the server for the duration of the connection. Clients MAY discard all cookies when a connection is terminated. This allows servers to rely on cookies for session affinity (e.g., sticky sessions behind a load balancer) and other small amounts of per-connection state.

### 6. Defines a unified routing model

| Method   | Upgrade Header?      | Behavior                                                                                                    |
| -------- | -------------------- | ----------------------------------------------------------------------------------------------------------- |
| `POST`   | —                    | Send JSON-RPC message. `initialize` returns 200 with JSON body. All others return 202 Accepted immediately. |
| `GET`    | No                   | Open connection-scoped SSE stream for all server→client messages. Requires `Acp-Connection-Id`.             |
| `GET`    | `Upgrade: websocket` | Upgrade to WebSocket for full-duplex messaging                                                              |
| `DELETE` | —                    | Terminate the connection                                                                                    |

### 7. Preserves the full ACP lifecycle

The `initialize` → `initialized` → messages → close lifecycle is identical regardless of transport. The `Acp-Connection-Id` binds requests to the initialized connection and its negotiated capabilities. Session identity is carried in JSON-RPC message bodies via the `sessionId` field.

## Shiny future

> How will things play out once this feature exists?

- **SDK implementers** get a clear, testable transport spec — Rust, TypeScript, and Python SDKs can all interoperate.
- **Desktop clients** use WebSocket for low-latency streaming; all clients support it as a baseline.
- **Cloud deployments** expose agents behind standard HTTP load balancers using the stateless-friendly HTTP mode, with cookie-based sticky sessions guaranteed by client support.
- **Proxy chains** can route ACP traffic over HTTP for multi-hop agent topologies.

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

### Transport Architecture

```
                         ┌─────────────────────────────────┐
                         │         /acp endpoint           │
                         └──────┬──────────┬───────────────┘
                                │          │
                    ┌───────────▼──┐  ┌────▼──────────────┐
                    │  HTTP State  │  │  WebSocket State   │
                    │(connections) │  │  (connections)     │
                    └───────┬──────┘  └────┬──────────────┘
                            │              │
                    ┌───────▼──────────────▼───────────────┐
                    │     ACP Agent (JSON-RPC handler)     │
                    │     serve(agent, read, write)        │
                    └─────────────────────────────────────┘
```

### Identity Model

ACP over Streamable HTTP uses two HTTP headers for connection and session identity, plus JSON-RPC message fields:

- **`Acp-Connection-Id`** (HTTP header) — Transport-level identifier returned by the server in the `initialize` response. Required on all HTTP requests after `initialize` and on the GET stream. Binds requests to an initialized connection and its negotiated capabilities.
- **`Acp-Session-Id`** (HTTP header) — Session-level identifier returned in the `session/new` response body. Required on all session-scoped POST requests (`session/prompt`, `session/cancel`, permission responses, etc.). Enables routing and debugging.
- **`sessionId`** (JSON-RPC field) — Session-level identifier also included in JSON-RPC `params` for session-scoped methods and in responses on the GET stream. A single connection may host multiple sessions, each with its own `sessionId`.

### Streamable HTTP Message Flow

```
Client                             Server
  │                                    │
  │  ═══ Connection Initialization ═══ │
  │                                    │
  │─── POST /acp ─────────────────────>│  { method: "initialize", id: 1 }
  │    Content-Type: application/json  │  (no Acp-Connection-Id header)
  │                                    │
  │<────── 200 OK ─────────────────────│  { id: 1, result: { capabilities, connectionId } }
  │    Acp-Connection-Id: <conn_id>    │  Response includes Acp-Connection-Id header
  │    Content-Type: application/json  │
  │                                    │
  │  ═══ Open GET Stream ═══           │
  │                                    │
  │─── GET /acp ──────────────────────>│  Open long-lived SSE stream
  │    Acp-Connection-Id: <conn_id>    │  for all server→client messages
  │    Accept: text/event-stream       │
  │              ┌─────────────────────│  (SSE stream open)
  │              │                     │
  │              │                     │
  │  ═══ Session Creation ═══          │
  │                                    │
  │─── POST /acp ─────────────────────>│  { method: "session/new", id: 2,
  │    Acp-Connection-Id: <conn_id>    │    params: { cwd, mcpServers } }
  │                                    │
  │<────── 202 Accepted ───────────────│  (returns immediately)
  │              │                     │
  │<─────────────│─ SSE event ─────────│  { id: 2, result: { sessionId: "sess_abc123" } }
  │              │                     │  (response comes on GET stream)
  │              │                     │
  │  ═══ Prompt Flow ═══               │
  │                                    │
  │─── POST /acp ─────────────────────>│  { method: "session/prompt", id: 3,
  │    Acp-Connection-Id: <conn_id>    │    params: { sessionId: "sess_abc123", prompt } }
  │    Acp-Session-Id: sess_abc123     │
  │                                    │
  │<────── 202 Accepted ───────────────│  (returns immediately)
  │              │                     │
  │<─────────────│─ SSE event ─────────│  notification: AgentMessageChunk (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: AgentThoughtChunk (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: ToolCall (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: ToolCallUpdate (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: AgentMessageChunk (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  { id: 3, result: { sessionId: "sess_abc123", ... } }
  │              │                     │  (response comes on GET stream)
  │              │                     │
  │  ═══ Permission Flow ═══           │
  │  (when tool requires confirmation) │
  │                                    │
  │─── POST /acp ─────────────────────>│  { method: "session/prompt", id: 4, ... }
  │    Acp-Connection-Id: <conn_id>    │
  │    Acp-Session-Id: sess_abc123     │
  │                                    │
  │<────── 202 Accepted ───────────────│
  │              │                     │
  │<─────────────│─ SSE event ─────────│  notification: ToolCall (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  { method: "request_permission", id: 99,
  │              │                     │    params: { sessionId: "sess_abc123", ... } }
  │              │                     │  (server-to-client request)
  │─── POST /acp ────────────────────>│  { id: 99, result: { outcome: "allow_once" } }
  │    Acp-Connection-Id: <conn_id>    │  (client response)
  │    Acp-Session-Id: sess_abc123     │
  │                                    │
  │<────── 202 Accepted ───────────────│
  │              │                     │
  │<─────────────│─ SSE event ─────────│  notification: ToolCallUpdate (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  { id: 4, result: { sessionId: "sess_abc123", ... } }
  │              │                     │  (response comes on GET stream)
  │              │                     │
  │  ═══ Cancel Flow ═══               │
  │                                    │
  │─── POST /acp ─────────────────────>│  { method: "session/prompt", id: 5, ... }
  │    Acp-Connection-Id: <conn_id>    │
  │    Acp-Session-Id: sess_abc123     │
  │                                    │
  │<────── 202 Accepted ───────────────│
  │              │                     │
  │<─────────────│─ SSE event ─────────│  notification: AgentMessageChunk (sessionId: "sess_abc123")
  │              │                     │
  │─── POST /acp ─────────────────────>│  { method: "session/cancel",
  │    Acp-Connection-Id: <conn_id>    │    params: { sessionId: "sess_abc123" } }
  │    Acp-Session-Id: sess_abc123     │
  │                                    │
  │<────── 202 Accepted ───────────────│  (notification, no id)
  │              │                     │
  │<─────────────│─ SSE event ─────────│  { id: 5, result: { sessionId: "sess_abc123", ... } }
  │              │                     │  (response comes on GET stream)
  │              │                     │
  │  ═══ Resume Session Flow ═══       │
  │  (new connection, existing session)│
  │                                    │
  │─── POST /acp ─────────────────────>│  { method: "initialize", id: 1 }
  │    (no Acp-Connection-Id)          │  New connection
  │<────── 200 OK ─────────────────────│  { id: 1, result: { capabilities, connectionId } }
  │    Acp-Connection-Id: <new_conn>   │
  │                                    │
  │─── GET /acp ──────────────────────>│  Open new GET stream
  │    Acp-Connection-Id: <new_conn>   │
  │              ┌─────────────────────│  (SSE stream open)
  │              │                     │
  │─── POST /acp ─────────────────────>│  { method: "session/load", id: 2,
  │    Acp-Connection-Id: <new_conn>   │    params: { sessionId: "sess_abc123", cwd } }
  │    Acp-Session-Id: sess_abc123     │
  │                                    │
  │<────── 202 Accepted ───────────────│
  │              │                     │
  │<─────────────│─ SSE event ─────────│  notification: UserMessageChunk (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: AgentMessageChunk (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: ToolCall (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  notification: ToolCallUpdate (sessionId: "sess_abc123")
  │<─────────────│─ SSE event ─────────│  { id: 2, result: { sessionId: "sess_abc123" } }
  │              │                     │  (response comes on GET stream)
  │              │                     │
  │  ═══ Connection Termination ═══    │
  │                                    │
  │─── DELETE /acp ───────────────────>│  Terminate connection
  │    Acp-Connection-Id: <conn_id>    │
  │<────────── 202 Accepted ───────────│
  │              ▼                     │  (GET stream closes)
```

#### Content Negotiation and Validation

- POST `Content-Type` **MUST** be `application/json` (415 otherwise).
- GET `Accept` **MUST** include `text/event-stream` (406 otherwise).
- POST requests for session-scoped operations **MUST** include both `Acp-Connection-Id` and `Acp-Session-Id` headers.
- Batch JSON-RPC requests return 501.
- HTTP/2 is **REQUIRED** for Streamable HTTP transport.

### WebSocket Request Flow

#### Connection Establishment (GET with Upgrade)

```
Client                                    Server
  │  GET /acp                               │
  │  Upgrade: websocket                     │
  │────────────────────────────────────────►│
  │  HTTP 101 Switching Protocols           │
  │  Acp-Connection-Id: <uuid>              │
  │◄────────────────────────────────────────│
  │  ══════ WebSocket Channel ══════════════│
```

A new connection is created on upgrade. The `Acp-Connection-Id` is returned in the upgrade response headers. The client must still send `initialize` as the first JSON-RPC message over the WebSocket to negotiate capabilities before creating sessions.

#### Bidirectional Messaging

All messages are WebSocket text frames containing JSON-RPC. Binary frames are ignored. On disconnect, the server cleans up the connection and any associated sessions.

### Unified Endpoint Routing

```
GET /acp
  ├── Has Upgrade: websocket? → WebSocket handler
  └── No → SSE stream handler
        ├── Missing Acp-Connection-Id? → 400 Bad Request
        ├── Unknown Acp-Connection-Id? → 404 Not Found
        └── Valid Acp-Connection-Id → Open connection-scoped SSE stream

POST /acp
  ├── Initialize request (no Acp-Connection-Id)? → Create connection, return 200 with JSON
  ├── No Acp-Connection-Id? → 400 Bad Request
  ├── Unknown Acp-Connection-Id? → 404 Not Found
  ├── Session-scoped request missing Acp-Session-Id? → 400 Bad Request
  └── Has valid Acp-Connection-Id (and Acp-Session-Id if required) → Forward to agent, return 202 Accepted

DELETE /acp
  ├── Has Acp-Connection-Id? → Terminate connection and all associated sessions, return 202
  └── No Acp-Connection-Id? → 400 Bad Request
```

### Connection and Session Model

```
Connection {
    connection_id:  String,                          // Acp-Connection-Id
    capabilities:   NegotiatedCapabilities,
    sessions:       HashMap<String, Session>,        // keyed by sessionId (JSON-RPC field)
    get_stream:     Option<SseStream>,               // Single GET stream for this connection
    to_agent_tx:    mpsc::Sender<String>,
    from_agent_rx:  Arc<Mutex<Receiver<String>>>,
    handle:         JoinHandle<()>,
}

Session {
    session_id:     String,                          // sessionId (JSON-RPC field)
    // session-specific state
}
```

The agent task is spawned once per connection. A single GET SSE stream delivers all server→client messages for that connection, regardless of which session they belong to. Sessions are identified by the `sessionId` field in JSON-RPC messages. The transport layer adapts channels to the wire format (SSE events for HTTP, text frames for WebSocket).

### Comparing to MCP Streamable HTTP

| MCP Requirement                            | ACP Implementation                          | Status               |
| ------------------------------------------ | ------------------------------------------- | -------------------- |
| POST for all client→server messages        | ✅                                          | Compliant            |
| Accept header validation (406)             | ✅                                          | Compliant            |
| Notifications/responses return 202         | ✅ (except `initialize` returns 200)        | Mostly compliant     |
| Requests return SSE stream                 | ❌ (single long-lived GET stream instead)   | Documented deviation |
| Session ID on initialize response          | ✅ (`Acp-Connection-Id`)                    | Compliant (renamed)  |
| Session ID required on subsequent requests | ✅ (`Acp-Connection-Id` + `Acp-Session-Id`) | Compliant (extended) |
| GET opens SSE stream                       | ✅ (single connection-scoped stream)        | Compliant            |
| DELETE terminates session                  | ✅ (terminates connection)                  | Compliant            |
| 404 for unknown sessions                   | ✅ (unknown connection IDs)                 | Compliant            |
| Batch requests                             | ❌ (returns 501)                            | Documented deviation |
| Resumability (Last-Event-ID)               | ❌                                          | Future work          |
| Protocol version header                    | ❌                                          | Future work          |

### Deviations from MCP Streamable HTTP

1. **Single long-lived GET stream**: MCP opens a new SSE stream for each request response. ACP uses a single long-lived GET stream per connection for all server→client messages. POST requests (except `initialize`) return 202 Accepted immediately, and responses arrive on the GET stream correlated by JSON-RPC `id`.
2. **Initialize returns JSON directly**: MCP's `initialize` returns an SSE stream. ACP's `initialize` returns `200 OK` with a JSON response body containing capabilities and `connectionId`. The `Acp-Connection-Id` is also included in the response header.
3. **HTTP/2 required**: ACP requires HTTP/2 for multiplexing concurrent POST requests alongside the long-lived GET stream.
4. **Two-header model**: ACP uses both `Acp-Connection-Id` (for connection identity) and `Acp-Session-Id` (for session identity on POST requests). MCP only uses `Mcp-Session-Id`. This allows ACP to distinguish connection-level state from session-level operations while supporting multiple concurrent sessions on one connection.
5. **WebSocket extension**: MCP doesn't define WebSocket. ACP adds it as a required client capability. Clients MUST support WebSocket, and servers MAY choose to only support WebSocket connections.
6. **Cookie support required**: Clients MUST handle cookies on HTTP transports for the duration of the connection, enabling sticky sessions and per-connection server state.
7. **No batch requests**: Returns 501. May be added later.
8. **No resumability yet in reference implementation**: SSE event IDs and `Last-Event-ID` resumption planned as follow-up.

### Implementation Plan

1. **Phase 1 — Specification** (this RFD): Define the transport spec and align terminology.
2. **Phase 2 — Reference Implementation** (in progress): Working implementation in Goose (`block/goose`).
3. **Phase 3 — SDK Support**: Add Streamable HTTP and WebSocket client support to Rust SDK (`sacp`), then TypeScript SDK.
4. **Phase 4 — Hardening**: Origin validation, `Acp-Protocol-Version`, SSE resumability, batch requests, security audit.

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

### Why not just use MCP Streamable HTTP as-is?

MCP opens a new SSE stream for each request response, which creates many long-lived connections and complicates load balancing. ACP uses a single long-lived GET stream per connection for all server→client messages, dramatically reducing connection count and simplifying sticky session routing. This is better suited for ACP's bidirectional, multi-session nature.

### How are sessions identified?

ACP uses `Acp-Connection-Id` in HTTP headers to identify the connection, and `sessionId` in JSON-RPC message bodies to identify sessions. A single connection may host multiple sessions. The single GET stream delivers messages for all sessions, and clients demux by the `sessionId` field in each message.

### Why add WebSocket support?

ACP is highly bidirectional with frequent streaming updates. WebSocket provides true bidirectional messaging with lower per-message overhead than HTTP. Clients MUST support WebSocket so that servers can choose to only support WebSocket connections, simplifying deployment. Streamable HTTP remains available as an additional option for environments where WebSocket is not viable on the server side (e.g., serverless).

### How does the server distinguish WebSocket from SSE on GET?

By inspecting the `Upgrade: websocket` header. This is standard HTTP behavior.

### Can a client have multiple sessions on one connection?

Yes. A client may call `session/new` multiple times within a single `Acp-Connection-Id`. Each returns a distinct `sessionId` in the response body. All messages for all sessions are delivered on the single GET stream. The client demuxes messages by the `sessionId` field in each JSON-RPC message.

### What alternative approaches did you consider, and why did you settle on this one?

- **Per-request SSE streams (like MCP)**: Rejected — creates too many long-lived connections, complicates load balancing, and wastes resources.
- **Separate endpoints** (`/acp/http`, `/acp/ws`): Rejected — single endpoint is simpler; WebSocket upgrade is natural HTTP.
- **WebSocket only**: Rejected — doesn't work through all proxies.
- **Session-scoped GET streams**: Rejected — still creates multiple long-lived connections per client. Connection-scoped stream with JSON-RPC demuxing is simpler.

### How does this interact with authentication?

Authentication (see auth-methods RFD) is orthogonal and layered on top via HTTP headers, query parameters, or WebSocket subprotocols. `Acp-Connection-Id` and `Acp-Session-Id` are transport-level identifiers, not auth tokens.

### What about the `Acp-Protocol-Version` header?

Clients SHOULD include it on all requests after initialization. Not yet implemented; part of Phase 4 hardening.

### Why require HTTP/2?

HTTP/2 provides multiplexing, allowing many concurrent POST requests alongside the long-lived GET stream on a single TCP connection. This is essential for efficient operation with the single-stream model. HTTP/1.1 would require separate TCP connections for each concurrent POST, defeating the efficiency gains.

## Revision history

- **2025-03-10**: Initial draft based on the RFC template and goose reference implementation.
- **2026-04-01**: Introduced a two-header identity model: `Acp-Connection-Id` (returned at `initialize`, binds to the connection) and `Acp-Session-Id` (returned at `session/new`, scopes to a session). This addresses feedback that the original single `Acp-Session-Id` conflated transport binding with ACP session identity, and enables session-scoped GET listener streams for targeted server-to-client event delivery. Removed connection-scoped GET streams — all GET SSE listeners now require both `Acp-Connection-Id` and `Acp-Session-Id`.
- **2026-04-15**: Minor edits
- **2026-04-23**: Major revision to single long-lived GET stream model. Changed from per-request SSE streams to a single connection-scoped GET stream for all server→client messages. POST requests (except `initialize`) now return 202 Accepted immediately. `initialize` returns 200 OK with JSON response body. Required HTTP/2 for multiplexing. This change makes the HTTP usage more similar to WebSocket and supports better the bidirectional nature of ACP.
