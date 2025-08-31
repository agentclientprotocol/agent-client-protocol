<a href="https://agentclientprotocol.com/" >
  <img alt="Agent Client Protocol" src="https://zed.dev/img/acp/banner-dark.webp">
</a>

# Agent Client Protocol

The Agent Client Protocol (ACP) standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

The protocol is still under heavy development, and we aim to mature it as we get confidence in the design by implementing it in various settings.

Learn more at [agentclientprotocol.com](https://agentclientprotocol.com/).

## Libraries and Schema

- **Rust**: [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol) - See [example_agent.rs](./rust/example_agent.rs) and [example_client.rs](./rust/example_client.rs)
- **TypeScript**: [`@zed-industries/agent-client-protocol`](https://www.npmjs.com/package/@zed-industries/agent-client-protocol) - See [examples/](./typescript/examples/)
- **Go**: [`github.com/zed-industries/agent-client-protocol/go`](https://pkg.go.dev/github.com/zed-industries/agent-client-protocol/go) - See [example/](./go/example/) and the [Go README](./go/README.md)
- **JSON Schema**: [schema.json](./schema/schema.json)

## Contributing

ACP is a protocol intended for broad adoption across the ecosystem; we follow a structured process to ensure changes are well-considered.

### Pull Requests

Pull requests should intend to close [an existing issue](https://github.com/zed-industries/agent-client-protocol/issues).

### Issues

- **Bug Reports**: If you notice a bug in the protocol, please file [an issue](https://github.com/zed-industries/agent-client-protocol/issues/new?template=05_bug_report.yml) and we will be in touch.
- **Protocol Suggestions**: If you'd like to propose additions or changes to the protocol, please start a [discussion](https://github.com/zed-industries/agent-client-protocol/discussions/categories/protocol-suggestions) first. We want to make sure proposed suggestions align well with the project. If accepted, we can have a conversation around the implementation of these changes. Once that is complete, we will create an issue for pull requests to target.
