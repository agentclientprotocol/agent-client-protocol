<a href="https://agentclientprotocol.com/" >
  <img alt="Agent Client Protocol" src="https://zed.dev/img/acp/banner-dark.webp">
</a>

# Agent Client Protocol

The Agent Client Protocol (ACP) standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

Learn more at [agentclientprotocol.com](https://agentclientprotocol.com/).

## Versioning

The published crate and schema package versions describe the Rust crate and JSON Schema artifacts themselves. They follow the compatibility expectations of those artifacts: Rust APIs, generated schema structure, artifact layout, and other details that downstream SDKs or code generators may consume.

**The current stable ACP protocol version is `1`.**

ACP wire compatibility is determined separately by the protocol version exchanged during `initialize` via `protocolVersion`. The `version` field in `schema/meta*.json` also describes the ACP protocol version that the corresponding schema represents.

This means two versions of the JSON Schema artifacts can describe the same wire-compatible ACP protocol version while having different schema structure for SDK generators. For example, a release might change how definitions are organized, named, or emitted in the JSON Schema in a way that affects downstream code generation without changing the JSON messages exchanged by ACP clients and agents.

Consumers should not infer wire compatibility from the crate or schema package version alone. Use the negotiated `protocolVersion` to determine the ACP wire protocol shape and breaking-compatibility level. Within a protocol version, use the exchanged capabilities to decide which optional ACP messages and features are supported. Use artifact versions to manage compatibility with this repository's Rust and schema outputs.

## Integrations

- [Schema](./schema/schema.json)
- [Agents](https://agentclientprotocol.com/overview/agents)
- [Clients](https://agentclientprotocol.com/overview/clients)
- Official Libraries
  - **Kotlin**: [`acp-kotlin`](https://github.com/agentclientprotocol/kotlin-sdk) - Supports JVM, other targets are in progress, see [samples](https://github.com/agentclientprotocol/kotlin-sdk/tree/master/samples/kotlin-acp-client-sample/src/main/kotlin/com/agentclientprotocol/samples)
  - **Java**: [`java-sdk`](https://github.com/agentclientprotocol/java-sdk) - See [examples](https://github.com/agentclientprotocol/java-sdk/tree/main/examples)
  - **Python**: [`python-sdk`](https://github.com/agentclientprotocol/python-sdk) - See [examples](https://github.com/agentclientprotocol/python-sdk/tree/main/examples)
  - **Rust**: [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol) - See [examples/agent.rs](https://github.com/agentclientprotocol/rust-sdk/blob/main/src/agent-client-protocol/examples/agent.rs) and [examples/client.rs](https://github.com/agentclientprotocol/rust-sdk/blob/main/src/agent-client-protocol/examples/client.rs)
  - **TypeScript**: [`@agentclientprotocol/sdk`](https://www.npmjs.com/package/@agentclientprotocol/sdk) - See [examples/](https://github.com/agentclientprotocol/typescript-sdk/tree/main/src/examples)
- [Community Libraries](https://agentclientprotocol.com/libraries/community)

## Contributing

ACP is a protocol intended for broad adoption across the ecosystem; we follow a structured process to ensure changes are well-considered. Read the [Contributing Guide](./CONTRIBUTING.md) for more information.

## Contribution Policy

This project does not require a Contributor License Agreement (CLA). Instead, contributions are accepted under the following terms:

> By contributing to this project, you agree that your contributions will be licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0). You affirm that you have the legal right to submit your work, that you are not including code you do not have rights to, and that you understand contributions are made without requiring a Contributor License Agreement (CLA).
