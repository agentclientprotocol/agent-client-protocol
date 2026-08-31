# ACP v2 Support in Kotlin SDK — YouTrack Tickets

---

## Epic: ACP v2 Support in Kotlin SDK

**Description:**
ACP v2 is a consolidation release of the Agent Client Protocol. It redesigns the prompt lifecycle (the `session/prompt` response no longer ends the turn), turns all updates into ID-keyed upserts with omit/`null`/value patch semantics, removes the Client file system / terminal execution / session modes APIs, reorganizes capabilities, and makes the whole schema forward-compatible via open enums and tagged unions. The SDK must support v1 and v2 side by side: one protocol version is negotiated per connection via `initialize`, and v2 stays behind explicit version negotiation and feature flags while it's labeled draft.

Migration guide: https://agentclientprotocol.com/protocol/v2/migration • Overview: https://agentclientprotocol.com/protocol/v2/overview

**Checklist:**

- [ ] Version negotiation and v1/v2 side-by-side support
- [ ] Separate v2 schema and generated models
- [ ] Initialization: unified `info`/`capabilities` and capability reorganization
- [ ] Authentication: `auth/login` / `auth/logout`
- [ ] Prompt lifecycle: acknowledgment response and `state_update`
- [ ] Messages: required `messageId` and whole-message upserts
- [ ] Tool calls: single `tool_call_update` upsert and `tool_call_content_chunk`
- [ ] Agent-owned terminal display: `terminal_update` / `terminal_output_chunk`
- [ ] Structured diff content
- [ ] Permission requests: `title` / `description` / `subject`
- [ ] Plans: `plan_update` with `planId`
- [ ] Session setup: `session/resume` with `replayFrom` replaces `session/load`
- [ ] Session baseline methods: `session/list`, `session/close`, `session/resume` required
- [ ] Session delete: new capability path
- [ ] Session modes removed → config options
- [ ] MCP server configuration changes
- [ ] Cancellation confirmation via idle `state_update`
- [ ] Removal of Client `fs/*` and `terminal/*` surface
- [ ] Content blocks: MCP alignment and `icons`
- [ ] Slash commands: tagged-union `input`
- [ ] Elicitation capability changes
- [ ] Extensibility: open enums, `_` prefix, `_meta` and three-state patch fields
- [ ] Transports: JSON-RPC batch support on stdio

---

## 1. Version negotiation and v1/v2 side-by-side support

The negotiation mechanism is unchanged, but the SDK must now select a v1 or v2 protocol surface per connection after `initialize` (`protocolVersion: 2`). v2 support is additive: v1 must keep working, and v2 (still labeled draft) must be gated behind version negotiation and feature flags. Unstable v2 features are gated independently of `protocolVersion: 2`.

📄 https://agentclientprotocol.com/protocol/v2/migration

## 2. Separate v2 schema and generated models

v2 ships its own schemas (`schema/v2/schema.json` stable baseline, `schema/v2/schema.unstable.json` draft features). Keep v1 and v2 models and fixtures fully separate. Some schema type names changed without wire changes (e.g. `UpdateSessionNotification`, `CancelSessionNotification`, `LoginAuthRequest`/`LogoutAuthRequest`), and generic `id` fields were renamed to domain IDs (`methodId`, `configId`, `groupId`).

📄 https://agentclientprotocol.com/protocol/v2/schema

## 3. Initialization: unified `info`/`capabilities` and capability reorganization

Role-specific fields were replaced: both sides now send `capabilities` and a **required** `info` (with `name`, `title`, `version`). All support markers became objects (`{}` = supported, omitted/`null` = unsupported) instead of booleans. Session-scoped capabilities moved under `capabilities.session` (`session.prompt`, `session.mcp`, `session.delete`, `session.additionalDirectories`); `loadSession` and the `list`/`resume`/`close` markers were removed; Client `fs`/`terminal` capabilities were removed entirely.

📄 https://agentclientprotocol.com/protocol/v2/initialization

## 4. Authentication: `auth/login` / `auth/logout`

`authenticate` was renamed to `auth/login` and `logout` to `auth/logout`. Auth method descriptors renamed `id` → `methodId` and gained a **required** `type` discriminator (stable value: `agent`). The logout capability marker is gone: advertising non-empty `authMethods` obligates the Agent to implement both methods; if omitted/empty, Clients must not call either.

📄 https://agentclientprotocol.com/protocol/v2/authentication

## 5. Prompt lifecycle: acknowledgment response and `state_update`

The biggest semantic change: `session/prompt` now returns an empty `{}` result immediately on acceptance instead of staying pending for the whole turn. The Agent then sends a `user_message` acknowledgment (with agent-owned `messageId`), a new `state_update` notification (`running` / `idle` / `requires_action`), and the `stopReason` moved from the prompt response to the idle `state_update`.

📄 https://agentclientprotocol.com/protocol/v2/prompt-lifecycle

## 6. Messages: required `messageId` and whole-message upserts

`messageId` is now **required** on all message chunks (`user_message_chunk`, `agent_message_chunk`, `agent_thought_chunk`); it was optional in v1. New whole-message upserts were added: `user_message`, `agent_message`, `agent_thought` carrying a full `content` array with three-state patch semantics (omitted = unchanged, `null`/`[]` = cleared, array = replaced); chunks always append.

📄 https://agentclientprotocol.com/protocol/v2/prompt-lifecycle

## 7. Tool calls: single `tool_call_update` upsert and `tool_call_content_chunk`

The `tool_call` (create) update was removed: the first `tool_call_update` for an unseen `toolCallId` now creates the tool call. Updates are explicit upserts with omit/`null`/value patch semantics; only `toolCallId` is required. New `tool_call_content_chunk` appends a single content item for streaming. The `status` enum adds `cancelled`, and `kind`/`status` are now extensible.

📄 https://agentclientprotocol.com/protocol/v2/tool-calls

## 8. Agent-owned terminal display

The `terminal` tool-call content changed ownership: it's now only a reference (`terminalId`) to a display-only terminal owned by the Agent. New session updates: `terminal_update` (upsert keyed by `terminalId` with patch fields `command`, `cwd`, base64 `output` replacement snapshot, `exitStatus`) and `terminal_output_chunk` (independently base64-encoded appended bytes). No input, kill, wait, or execution semantics.

📄 https://agentclientprotocol.com/protocol/v2/tool-calls

## 9. Structured diff content

The v1 diff (`path` + `oldText`/`newText`) was replaced by a required `changes` list of file operations (`add`/`delete`/`modify`/`move`/`copy` with optional `fileType`, `mimeType`) plus an optional renderable `patch` (`format: "git_patch"`, absolute paths). Clients must handle patch-less diffs; there is no mechanical mapping back to `oldText`/`newText`.

📄 https://agentclientprotocol.com/protocol/v2/tool-calls

## 10. Permission requests: `title` / `description` / `subject`

`session/request_permission` no longer takes a bare `toolCall`. It now requires a `title` (prompt copy), optional `description`, and an optional tagged-union `subject`: `tool_call` (a `ToolCallUpdate` upsert shape) or the new `command` subject (`command`, required absolute `cwd`, optional `toolCallId`/`terminalId`). Options and response shape are unchanged, but both unions are extensible; an unknown outcome must never be treated as approval.

📄 https://agentclientprotocol.com/protocol/v2/tool-calls

## 11. Plans: `plan_update` with `planId`

The flat `plan` update was replaced by `plan_update`, whose payload is a tagged union with required `planId` and `type` (stable type: `items`), enabling multiple plans per session. Entry shape is unchanged; entry `status` adds `cancelled`, and `priority`/`status` enums are now extensible. Each update replaces that plan's entries.

📄 https://agentclientprotocol.com/protocol/v2/agent-plan

## 12. Session setup: `session/resume` with `replayFrom` replaces `session/load`

`session/load` was removed; `session/resume` gained an optional `replayFrom` cursor (`{ "type": "start" }` replays full history as ordinary `session/update` notifications). `session/new` and `session/resume` share the same environment params (required absolute `cwd`, optional `additionalDirectories` and `mcpServers` — `mcpServers` was required in v1). Responses carry `configOptions` and no longer include `modes`.

📄 https://agentclientprotocol.com/protocol/v2/session-setup

## 13. Session baseline methods required

The individual `list`/`resume`/`close` capability markers are gone: advertising `capabilities.session` now commits the Agent to `session/new`, `session/list`, `session/resume`, `session/close`, `session/prompt`, `session/cancel`, and `session/update`. `session/list` also formalizes RFC 3339 `updatedAt` and title pushes via `session_info_update`.

📄 https://agentclientprotocol.com/protocol/v2/session-list

## 14. Session delete: new capability path

`session/delete` itself is unchanged and remains optional, but its capability moved to `capabilities.session.delete` and is advertised as an object marker (`{}`) instead of a boolean.

📄 https://agentclientprotocol.com/protocol/v2/session-delete

## 15. Session modes removed → config options

The modes API was removed entirely: `modes` on session responses, `session/set_mode`, `current_mode_update`, and all `SessionMode*` types. Mode-like state is expressed via session config options: `id` renamed to `configId`, stable `category` values (`mode`, `model`, `model_config`, `thought_level`), values changed via `session/set_config_option` (response returns the full updated `configOptions`), and agent-initiated changes arrive as `config_option_update`.

📄 https://agentclientprotocol.com/protocol/v2/session-config-options

## 16. MCP server configuration changes

Every MCP server config now requires a `type` discriminator (v1 stdio configs had none). The deprecated `sse` transport was removed. Per-transport capabilities live under `capabilities.session.mcp` (`stdio` is now an explicit opt-in, `http` for remote). `args`, `env` (stdio) and `headers` (http) became optional.

📄 https://agentclientprotocol.com/protocol/v2/session-setup

## 17. Cancellation confirmation via idle `state_update`

`session/cancel` is unchanged as a notification, but confirmation moved: instead of responding to `session/prompt` with `stopReason: "cancelled"`, the Agent finishes pending updates and sends an idle `state_update` with `stopReason: "cancelled"`. Clients keep accepting tool-call updates after cancelling and answer pending permission requests with the `cancelled` outcome.

📄 https://agentclientprotocol.com/protocol/v2/cancellation

## 18. Removal of Client `fs/*` and `terminal/*` surface

`fs/read_text_file`, `fs/write_text_file`, all five `terminal/*` methods, and the `clientCapabilities.fs`/`terminal` capabilities were removed. Clients that want to expose file access or command execution to Agents do so via client-provided MCP servers passed in `mcpServers`. Remove these methods/handlers from the v2 SDK surface.

📄 https://agentclientprotocol.com/protocol/v2/migration

## 19. Content blocks: MCP alignment and `icons`

The five content block types are unchanged but realigned with the latest MCP spec: `resource_link` gains an optional `icons` array (required `src`, optional `mimeType`, `sizes`, `theme`), the `type` discriminator is extensible, base64 payloads are marked `contentEncoding: "base64"`, and `annotations.priority` is bounded to 0–1.

📄 https://agentclientprotocol.com/protocol/v2/content

## 20. Slash commands: tagged-union `input`

`available_commands_update` is unchanged, but a command's optional `input` spec is now a tagged union: the v1 untagged unstructured input gains a required `type: "text"` discriminator so richer input types can be added later. Unknown input types should be preserved and rendered with a plain-text fallback.

📄 https://agentclientprotocol.com/protocol/v2/slash-commands

## 21. Elicitation capability changes

Elicitation capability moved from `clientCapabilities.elicitation` to the unified `capabilities.elicitation`, and supported modes must now be advertised explicitly as `form` and `url` object markers (an empty object no longer implies form-only support). The data model tracks the latest MCP release; ACP retains `elicitationId` and `elicitation/complete` for URL flows.

📄 https://agentclientprotocol.com/protocol/v2/elicitation

## 22. Extensibility: open enums, `_` prefix, `_meta` and three-state patch fields

All enums and tagged unions are open: unknown values must be accepted and preserved (`_`-prefixed = implementation extensions; non-underscore unknowns reserved for future ACP versions). The SDK must model omitted vs `null` vs concrete values distinctly wherever patch semantics apply — a plain nullable type erases a protocol distinction. `_meta` keys `traceparent`, `tracestate`, `baggage` are reserved for W3C trace context. Known discriminators must still be parsed strictly.

📄 https://agentclientprotocol.com/protocol/v2/extensibility

## 23. Transports: JSON-RPC batch support on stdio

stdio remains the primary transport, but v2 explicitly follows JSON-RPC 2.0 batch behavior: a newline-delimited message may be a batch array; receivers may process entries concurrently, must not reply to notifications, and return per-entry `-32600` errors for invalid entries. Lifecycle-sensitive messages (`initialize`, `auth/login`, `session/new`, `session/resume`, `session/prompt`) should not be batched. A remote transport (HTTP/SSE + WebSocket) is a separate RFD, not part of core v2.

📄 https://agentclientprotocol.com/protocol/v2/transports
