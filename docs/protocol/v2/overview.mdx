---
title: "Overview"
description: "How the Agent Client Protocol works"
---

The Agent Client Protocol allows [Agents](#agent) and [Clients](#client) to communicate by exposing methods that each side can call and sending notifications to inform each other of events.

## Communication Model

The protocol follows the [JSON-RPC 2.0](https://www.jsonrpc.org/specification) specification with two types of messages:

- **Methods**: Request-response pairs that expect a result or error
- **Notifications**: One-way messages that don't expect a response

ACP follows JSON-RPC 2.0 batch behavior. See
[Transports](/protocol/v2/transports#json-rpc-batch-messages) for batch
handling requirements.

## Message Flow

A typical flow follows this pattern:

<Steps>
<Step title="Initialization Phase">

- Client → Agent: `initialize` to establish connection
- Client → Agent: `auth/login` if required by the Agent

</Step>

<Step title="Session Setup - either:">

- Client → Agent: `session/new` to create a new session
- Client → Agent: `session/resume` to resume an existing session

</Step>

<Step title="Prompt Lifecycle">
   - Client → Agent: `session/prompt` to send user message
   - Agent → Client: `session/prompt` response once the prompt is accepted
   - Agent → Client: `session/update` notifications for accepted messages, state updates, and progress updates
   - Agent → Client: Permission requests as needed
   - Client → Agent: `session/cancel` to interrupt processing if needed
   - Active work completes when the Agent sends an idle `state_update` session update with a stop reason
</Step>
</Steps>

## Agent

Agents are programs that use generative AI to autonomously modify code. They typically run as subprocesses of the Client.

### Baseline Methods

<ResponseField
  name="initialize"
  post={[<a href="/protocol/v2/schema#initialize">Schema</a>]}
>
  [Negotiate versions and exchange capabilities.](/protocol/v2/initialization).
</ResponseField>

<ResponseField
  name="auth/login"
  post={[<a href="/protocol/v2/schema#auth%2Flogin">Schema</a>]}
>
  Authenticate with the Agent (if required).
</ResponseField>

<ResponseField
  name="auth/logout"
  post={[<a href="/protocol/v2/schema#auth%2Flogout">Schema</a>]}
>
  [End the current authenticated
  state](/protocol/v2/authentication#logging-out).
</ResponseField>

<ResponseField
  name="session/new"
  post={[<a href="/protocol/v2/schema#session%2Fnew">Schema</a>]}
>
  [Create a new conversation
  session](/protocol/v2/session-setup#creating-a-session).
</ResponseField>

<ResponseField
  name="session/prompt"
  post={[<a href="/protocol/v2/schema#session%2Fprompt">Schema</a>]}
>
  [Send user prompts](/protocol/v2/prompt-lifecycle#1-user-message) to the
  Agent.
</ResponseField>

<ResponseField
  name="session/list"
  post={[<a href="/protocol/v2/schema#session%2Flist">Schema</a>]}
>
  [List known sessions](/protocol/v2/session-list).
</ResponseField>

<ResponseField
  name="session/resume"
  post={[<a href="/protocol/v2/schema#session%2Fresume">Schema</a>]}
>
  [Resume an existing session, optionally replaying
  history](/protocol/v2/session-setup#resuming-sessions).
</ResponseField>

<ResponseField
  name="session/close"
  post={[<a href="/protocol/v2/schema#session%2Fclose">Schema</a>]}
>
  [Close an active session](/protocol/v2/session-setup#closing-active-sessions).
</ResponseField>

### Notifications

<ResponseField
  name="session/cancel"
  post={[<a href="/protocol/v2/schema#session%2Fcancel">Schema</a>]}
>
  [Cancel ongoing operations](/protocol/v2/prompt-lifecycle#cancellation) (no
  response expected).
</ResponseField>

## Client

Clients provide the interface between users and agents. They are typically code editors (IDEs, text editors) but can also be other UIs for interacting with agents. Clients manage the environment, handle user interactions, and control access to resources.

### Baseline Methods

<ResponseField
  name="session/request_permission"
  post={[<a href="/protocol/v2/schema#session%2Frequest_permission">Schema</a>]}
>
  [Request user authorization](/protocol/v2/tool-calls#requesting-permission)
  for operations such as tool calls.
</ResponseField>

### Notifications

<ResponseField
  name="session/update"
  post={[<a href="/protocol/v2/schema#session%2Fupdate">Schema</a>]}
>
  [Send session updates](/protocol/v2/prompt-lifecycle#3-agent-reports-output)
  to inform the Client of changes (no response expected). This includes [message
  updates and chunks](/protocol/v2/content), [tool calls, updates, and content
  chunks](/protocol/v2/tool-calls), [plans](/protocol/v2/agent-plan), [available
  commands updates](/protocol/v2/slash-commands#advertising-commands), and
  [config option updates](/protocol/v2/session-config-options#from-the-agent).
</ResponseField>

## Argument requirements

- All file paths in the protocol **MUST** be absolute.
- Line numbers are 1-based

## Error Handling

All methods follow standard JSON-RPC 2.0 [error handling](https://www.jsonrpc.org/specification#error_object):

- Successful responses include a `result` field
- Errors include an `error` object with `code` and `message`
- Notifications never receive responses (success or error)

## Conventions

Unless explicitly defined otherwise in the schema, ACP-defined JSON object property keys use `camelCase`. String values carried by discriminator fields use `snake_case`. The JSON-RPC envelope fields (`jsonrpc`, `id`, `method`, `params`, `result`, and `error`) follow the JSON-RPC 2.0 specification.

Selected enum-like fields and tagged unions can define custom or future fallbacks. For those fields, `_`-prefixed values are reserved for implementation-specific extensions, while unknown non-underscore values are reserved for future ACP variants.

## Extensibility

The protocol provides built-in mechanisms for adding custom functionality while maintaining compatibility:

- Add custom data using `_meta` fields
- Create custom methods by prefixing their name with underscore (`_`)
- Advertise custom capabilities during initialization

Learn about [protocol extensibility](/protocol/v2/extensibility) to understand how to use these mechanisms.

## Next Steps

- Learn about [Initialization](/protocol/v2/initialization) to understand version and capability negotiation
- Understand [Session Setup](/protocol/v2/session-setup) for creating and loading sessions
- Review the [Prompt Lifecycle](/protocol/v2/prompt-lifecycle)
- Explore [Extensibility](/protocol/v2/extensibility) to add custom features
