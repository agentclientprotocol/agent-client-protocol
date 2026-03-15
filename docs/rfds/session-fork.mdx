---
title: "Forking of existing sessions"
---

- Author(s): [@josevalim](https://github.com/josevalim)
- Champion: [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

> What are you proposing to change?

We propose adding the ability to "fork" a new session based on an existing one.
This will allow us to use the current conversation as context to generate pull
request descriptions, summaries, etc. without polluting the user history.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

Imagine we want to summarize the current conversation to use it in a future chat. If we send a message
asking for the summary, it will become part of its context, affecting future user interactions.
Therefore we want to be able to fork a session, issue additional messages, and then close the fork.

## What we propose to do about it

> What are you proposing to improve the situation?

To add a "session/fork" method.

## Shiny future

> How will things will play out once this feature exists?

We will be able to implement functionality that requires using the current chat
without polluting its history, ranging from summaries to potentially subagents.

I can also see this feature being extended in the future to support an optional
message ID, so the fork happens at a specific message, allowing clients to implement
functionality like editing previous messages and similar.

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

We propose to add a new "session/fork" method. Agents must declare this option is
available by returning `session: { fork : {} }` in its capabilities. The object is reserved
to declare future capabilities, such as forking from a specific message, a tool call, or similar.

Then the client would be able to request a fork of the given session:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "session/fork",
  "params": {
    "sessionId": "sess_789xyz",
    "cwd": "...",
    "mcpServers": [...]
  }
}
```

The request expects the same options as `session/load`, such as `cwd` and `mcpServers`.

Similarly, the agent would respond with optional data such as config options, the same as `session/load`.

Agents may reply with an error if forking of that specific session or with the given options is not supported,
for example if the agent does not support forking with a different working directory than the initial session.

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

**Q: Should a new method be introduced or should "session/new" be expanded?**

They must be different because they will effectively require different options.
For example, "session/new" has options such as capabilities and MCP which are not
recommended to be set when forking, as the context being forked was built with other
tools, and forking may accept a messageId for checkpoints.

**Q: Should fork only accept the `sessionId` or also other options, similar to `session/load`?**

Initially, we proposed to only accept the `sessionId`, but this would make it more difficult to
allow forking of inactive sessions in agents like `claude-agent-acp`, as Claude does not retain the
configured MCP servers of a session. Limiting fork to only already active sessions would limit its usefulness.

Moreover, allowing to pass different options also allows features like dynamically adding MCP servers
to existing sessions to work by forking them with the new options, if the agent supports it. If it
does not, the client can still show an appropriate error message.

### What alternative approaches did you consider, and why did you settle on this one?

None. This proposal is inspired by the abilities exposed in Claude Agent SDK. It must be validated against other agents too.

## Revision history

- 2025-11-17: Mentioned capabilities format, updated FAQ.
- 2025-11-20: Added request format and updated capabilities format.
- 2025-12-10: Adjust fork options to align with `session/load`.
