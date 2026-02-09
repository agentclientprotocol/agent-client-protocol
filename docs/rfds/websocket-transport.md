---
title: "Native WebSocket Transport for Agent Communication Protocol"
---

Author(s): [steve02081504](https://github.com/steve02081504)

## Elevator pitch

The current Agent Communication Protocol (ACP) SDK exclusively uses stdio for transport, requiring a separate forwarding process when the agent backend operates over WebSocket. This proposal introduces native WebSocket transport support directly into the ACP SDK, eliminating the need for intermediate stdio-to-WebSocket bridging processes and enabling more flexible, resource-efficient deployment architectures.

## Status quo

Currently, the ACP SDK (`@agentclientprotocol/sdk`) is designed around stdio-based communication. When an agent backend is implemented as a web service (e.g., using Express.js with WebSocket endpoints), the following architecture is required:

1. **Backend Service**: Implements ACP protocol over WebSocket (e.g., `ws://localhost:8931/ws/acp`).
2. **Bridge Process**: A separate script or bin that:
   - Connects to the WebSocket endpoint.
   - Forwards all messages between stdio and WebSocket.
   - Handles buffering, encoding, and line-delimitation.
3. **IDE Client**: Launches the bridge process and communicates via stdio.

**[Example from fount framework:](https://github.com/steve02081504/fount/blob/master/src/public/parts/shells/ideIntegration/public/fount_ide_agent.mjs)**

```javascript
// fount_ide_agent.mjs - A dedicated forwarding process
const ws = new WebSocket(wsUrl, [apiKey]);
process.stdin.on('data', (chunk) => {
    buffer += chunk;
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';
    for (const line of lines)
        if (line.trim()) ws.send(line + '\n');
});
ws.onmessage = (event) => {
    writeOut(event.data.endsWith('\n') ? event.data : event.data + '\n');
};
```

**Problems with this approach:**

1. **Resource Overhead**: Each client connection requires an additional process, consuming:
   - ~3-70MB RAM depends on the runtime.
   - CPU cycles for message forwarding.
   - Disk I/O for process spawning and script loading.
2. **Architectural Complexity**: Introduces an unnecessary layer of indirection that must handle:
   - Connection lifecycle synchronization (stdio close ↔ WebSocket close).
   - Error propagation and retry logic.
   - Authentication token passing (via URL params or environment variables).
3. **Deployment Friction**: Deploying an ACP agent as a web service requires:
   - Hosting both the main backend **and** distributing the bridge script.
   - Managing separate versioning for the bridge script.
   - Documenting non-standard connection procedures.
4. **Latency**: Every message incurs an extra serialization/deserialization cycle and inter-process communication overhead.

**Current Workarounds:**
- **Custom Bridge Scripts**: Each project maintains its own stdio-to-WebSocket forwarder (e.g., [`fount_ide_agent.mjs`](https://github.com/steve02081504/fount/blob/master/src/public/parts/shells/ideIntegration/public/fount_ide_agent.mjs)).
- **Redundant Implementations**: Developers re-implement WebSocket-to-ndJSON stream adapters in multiple projects.
- **Suboptimal Architecture**: Backends that naturally operate over WebSocket (e.g., multi-user services like fount) are forced into a single-user stdio model.

## What we propose to do about it

We propose extending the ACP SDK to natively support WebSocket as a transport layer, alongside the existing stdio transport.

**Key Design Principles:**
- **Transport Abstraction**: Decouple the protocol implementation from the transport layer.
- **Backward Compatibility**: Existing stdio-based agents continue to work unchanged.
- **Minimal API Surface**: Introduce transport selection without complicating the core SDK.

**Proposed Client API:**

```typescript
// Current (stdio only)
const agent = new AgentClientConnection({
    command: ['node', 'agent.js'],
    args: ['--model', 'gpt-4']
});

// New (WebSocket)
const agent = new AgentClientConnection({
    transport: 'websocket',
    url: 'ws://localhost:8931/ws/acp',
    protocols: ['api-key-token'],  // Optional: WebSocket subprotocols for auth
    headers: { 'Authorization': 'Bearer TOKEN' }  // Optional: custom headers
});

// Alternative: Auto-detect via URL scheme
const agent = new AgentClientConnection({
    endpoint: 'ws://localhost:8931/ws/acp?charname=ZL-31',
    auth: { type: 'subprotocol', token: 'api-key-token' }
});
```

**Proposed Server API:**

```typescript
import { AgentSideConnection, webSocketStream } from '@agentclientprotocol/sdk';
import type { WebSocket } from 'ws';

// Helper to adapt WebSocket to ACP duplex stream
function createWebSocketTransport(ws: WebSocket) {
    return webSocketStream(ws);  // New SDK export
}

// Express.js + ws example
app.ws('/acp', authenticate, (ws, req) => {
    const stream = createWebSocketTransport(ws);
    const connection = new AgentSideConnection(
        (conn) => new MyAgent(conn, req.user),
        stream
    );
});
```

## Shiny future

In the shiny future, ACP agents will be truly transport-agnostic:

- **Cloud-Native Deployment**: A single agent backend can serve thousands of concurrent users over WebSocket without spawning child processes.
- **Unified Architecture**: Frameworks like [fount](https://github.com/steve02081504/fount) can expose ACP directly as an HTTP/WebSocket API without auxiliary bridge scripts.
- **Mobile & Web Support**: Browser-based IDEs and mobile apps can connect directly to ACP agents via WebSocket without WASM runtimes or proxy servers.
- **Firewall Friendly**: WebSocket operates over standard HTTP(S) ports, bypassing corporate firewalls that block arbitrary stdio protocols.
- **Built-in Load Balancing**: Standard web infrastructure (nginx, HAProxy, AWS ALB) can distribute ACP connections without custom tooling.
- **Reduced Friction**: Developers can deploy ACP agents to Vercel, Cloudflare Workers, or any serverless platform that supports WebSocket.

## Implementation details and plan

### Phase 1: Core Transport Abstraction (v1.0)

**Goal**: Separate protocol logic from transport implementation.

1. **Define Transport Interface:**

```typescript
interface Transport {
    readable: ReadableStream<Uint8Array>;
    writable: WritableStream<Uint8Array>;
    close(): void;
}
```

2. **Refactor Existing Code:**
   - Extract current stdio logic into `StdioTransport` class.
   - Make `AgentClientConnection` and `AgentSideConnection` accept a `Transport` parameter.

3. **Backward Compatibility:**
   - Keep existing constructor signatures as shortcuts to `StdioTransport`.

### Phase 2: WebSocket Transport (v1.1)

**Goal**: Implement WebSocket transport with feature parity to stdio.

1. **Client-Side:**
   - Implement `WebSocketTransport` class.
   - Add connection options: URL, auth headers, subprotocols.
   - Handle reconnection logic (optional).

2. **Server-Side:**
   - Export `webSocketStream(ws)` helper for common WebSocket libraries (`ws`, `uWebSockets.js`).
   - Document integration patterns for Express, Fastify, Hono.

3. **Testing:**
   - Integration tests with real WebSocket servers (Node.js `ws`, Deno native).
   - Load testing to verify no performance regression.

### Phase 3: Documentation & Ecosystem (v1.2)

1. **Reference Implementations:**
   - Provide example multi-user agent backend (Express + WebSocket).
   - Create browser-based ACP client demo.

2. **Migration Guide:**
   - Document how to migrate from bridge scripts to native WebSocket.
   - Update IDE integration examples (Zed, VS Code, Cursor).

3. **Best Practices:**
   - Recommend authentication strategies (subprotocols, JWT headers, session cookies).
   - Document scaling patterns (sticky sessions, Redis pub/sub for distributed agents).

### Phase 4 (Optional): Additional Transports (v2.0)

- **HTTP Streaming**: Support Server-Sent Events (SSE) for one-way agent-to-client updates.
- **IPC Sockets**: Unix domain sockets for same-machine communication without network overhead.
- **TCP Sockets**: Raw TCP for ultra-low-latency scenarios.

## Frequently asked questions

### What alternative approaches did you consider, and why did you settle on this one?

| Approach | Reason for Rejection |
|----------|----------------------|
| **HTTP Long Polling** | Inefficient for bidirectional streaming; high latency due to request overhead. |
| **Server-Sent Events (SSE)** | Unidirectional only; requires separate HTTP POST channel for client-to-agent messages. |
| **gRPC Streams** | Requires additional tooling (protobuf, gRPC libraries); overkill for JSON-RPC protocol. |
| **Custom Binary Protocol** | Breaks compatibility with existing ACP JSON-RPC schema; reinvents WebSocket. |
| **Keep Bridge Scripts** | Does not address resource overhead; shifts burden to agent developers. |

**WebSocket was chosen because:**
- It's a W3C standard with universal browser and server support.
- Provides full-duplex communication over a single connection.
- Works over HTTP(S) ports, compatible with existing web infrastructure.
- Minimal overhead compared to HTTP request/response cycles.
- Already widely used in similar protocols (Language Server Protocol over WebSocket, Debug Adapter Protocol).

### How does this affect existing stdio-based agents?

**Zero impact.** The current stdio-based API remains the default:

```javascript
// This continues to work exactly as before
const agent = new AgentClientConnection({
    command: ['node', 'agent.js']
});
```

The WebSocket transport is opt-in, activated only when explicitly configured.

### What about authentication and security?

ACP over WebSocket supports multiple authentication strategies:

1. **WebSocket Subprotocols**: Pass API key as subprotocol (e.g., `new WebSocket(url, ['api-key-token'])`).
2. **HTTP Headers**: Send `Authorization` header during WebSocket handshake.
3. **Query Parameters**: Embed tokens in URL (e.g., `ws://host/acp?token=...`).
4. **Session Cookies**: Use existing HTTP session authentication (recommended for web apps).

The SDK will document best practices for each scenario. TLS (wss://) should be used in production to encrypt all communication.

### Won't this make the SDK more complex?

The complexity is **encapsulated** within transport implementations. The core protocol logic (`initialize`, `newSession`, `prompt`) remains unchanged. Developers using the SDK only see:

```diff
- const agent = new AgentClientConnection({ command: ['node', 'agent.js'] });
+ const agent = new AgentClientConnection({ endpoint: 'ws://localhost:8931/acp' });
```

For advanced use cases, the transport abstraction provides clear extension points without coupling to SDK internals.

### How does this compare to Language Server Protocol (LSP)?

LSP faced a similar challenge and now supports:
- stdio (original transport).
- WebSocket (for cloud IDEs like VS Code for Web, GitHub Codespaces).
- IPC pipes (Windows named pipes, Unix domain sockets).

ACP should follow LSP's proven approach: **protocol-transport independence** enables the same agent implementation to work across desktop IDEs, web IDEs, mobile apps, and CLI tools.

### What about connection lifecycle and error handling?

The SDK will handle common WebSocket scenarios:

- **Connection Failures**: Throw errors with actionable messages (e.g., "Failed to connect to ws://host:port - ensure agent is running").
- **Unexpected Disconnects**: Emit `disconnect` events; optionally auto-reconnect with exponential backoff.
- **Protocol Errors**: Maintain existing JSON-RPC error handling; transport layer does not interpret message semantics.

Reconnection behavior will be configurable:

```javascript
const agent = new AgentClientConnection({
    endpoint: 'ws://localhost:8931/acp',
    reconnect: {
        enabled: true,
        maxAttempts: 5,
        backoff: 'exponential'  // 1s, 2s, 4s, 8s, 16s
    }
});
```

### Can I use this with serverless platforms?

Yes, with caveats:

- **Vercel/Netlify**: Serverless functions have time limits (10-60s). Long-lived agent sessions should use dedicated WebSocket servers (e.g., Vercel Edge Functions with Durable Objects).
- **AWS Lambda**: Use API Gateway WebSocket APIs to route connections to Lambda functions. Lambda's 15-minute limit supports most agent interactions.
- **Cloudflare Workers**: Workers + Durable Objects provide native WebSocket support with indefinite connection lifetimes.

The SDK will document these platform-specific patterns.

## Revision history

- **2026-02-09**: Initial draft by steve02081504.
