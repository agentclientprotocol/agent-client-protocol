Add [acp-cj](https://atomgit.com/ystyle/acp-cj) to the Community Libraries list.

## What is acp-cj?

**acp-cj** is an Agent Client Protocol (ACP) framework for the **Cangjie** programming language (Huawei's native programming language). It provides:

- **Agent side (server)**: `AcpAgent` — implements the ACP Agent interface, can be spawned as a subprocess by any ACP client (e.g. Zed, opencode)
- **Client side**: `AcpClient` — connects to any ACP agent, handling handshake, session, prompt turns, and streaming notifications
- **Bridge example**: `pi_bridge` — combines `AcpAgent` + `AcpClient` to proxy requests to a real LLM agent
- Full ACP **v1** protocol support, built on a `jsonrpc` + `jsonvalue` layered architecture

## Repository

https://atomgit.com/ystyle/acp-cj (AtomGit, primary home of the Cangjie ecosystem)
