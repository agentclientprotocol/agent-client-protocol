<a href="https://agentclientprotocol.com/" >
  <img alt="Agent Client Protocol" src="https://zed.dev/img/acp/banner-dark.webp">
</a>

# Agent Client Protocol

The Agent Client Protocol (ACP) standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

Learn more at [agentclientprotocol.com](https://agentclientprotocol.com/).

## Integrations

### Editors

- [Zed](https://zed.dev/docs/ai/external-agents)
- Emacs via [agent-shell.el](https://github.com/xenodium/agent-shell)
- [JetBrains _(coming soon)_](https://blog.jetbrains.com/ai/2025/10/jetbrains-zed-open-interoperability-for-ai-coding-agents-in-your-ide/)
- [marimo notebook](https://github.com/marimo-team/marimo)
- [neovim](https://neovim.io)
  - through the [CodeCompanion](https://github.com/olimorris/codecompanion.nvim) plugin
  - through the [yetone/avante.nvim](https://github.com/yetone/avante.nvim) plugin

### Agents

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview)
  - [via Zed's SDK adapter](https://github.com/zed-industries/claude-code-acp)
- [Gemini](https://github.com/google-gemini/gemini-cli)
- [Goose](https://block.github.io/goose/docs/guides/acp-clients)
- [JetBrains Junie _(coming soon)_](https://www.jetbrains.com/junie/)
- [Stakpak](https://github.com/stakpak/agent?tab=readme-ov-file#agent-client-protocol-acp)
- [VT Code](https://github.com/vinhnx/vtcode/blob/main/README.md#zed-ide-integration-agent-client-protocol)

## Libraries and Schema

- **JSON Schema**: [`schema.json`](./schema/schema.json)
- **Kotlin**: [`acp-kotlin`](https://github.com/agentclientprotocol/acp-kotlin-sdk) – supports JVM, other targets are in progress
- **Rust**: [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol) - See [examples/agent.rs](./rust/examples/agent.rs) and [examples/client.rs](./rust/examples/client.rs)
- **TypeScript**: [`@agentclientprotocol/sdk`](https://www.npmjs.com/package/@agentclientprotocol/sdk) - See [examples/](./typescript/examples/)

### From the Community

- **Dart**: [`acp_dart`](https://github.com/SkrOYC/acp-dart)
- **Emacs**: [`acp.el`](https://github.com/xenodium/acp.el)
- **Go**: [`acp-go-sdk`](https://github.com/coder/acp-go-sdk)
- **Python**: [`agent-client-protocol-python`](https://github.com/PsiACE/agent-client-protocol-python)
- **React**: [`use-acp`](https://github.com/marimo-team/use-acp): hooks for connecting to Agent Client Protocol (ACP) servers.

## Contributing

ACP is a protocol intended for broad adoption across the ecosystem; we follow a structured process to ensure changes are well-considered.

### Pull Requests

Pull requests should intend to close [an existing issue](https://github.com/agentclientprotocol/agent-client-protocol/issues).

### Issues

- **Bug Reports**: If you notice a bug in the protocol, please file [an issue](https://github.com/agentclientprotocol/agent-client-protocol/issues/new?template=05_bug_report.yml) and we will be in touch.
- **Protocol Suggestions**: If you'd like to propose additions or changes to the protocol, please start a [discussion](https://github.com/agentclientprotocol/agent-client-protocol/discussions/categories/protocol-suggestions) first. We want to make sure proposed suggestions align well with the project. If accepted, we can have a conversation around the implementation of these changes. Once that is complete, we will create an issue for pull requests to target.

### License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
