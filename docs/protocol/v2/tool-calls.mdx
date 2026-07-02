---
title: "Tool Calls"
description: "How Agents report tool call execution"
---

Tool calls represent actions that language models request Agents to perform during [active session work](/protocol/v2/prompt-lifecycle). When an LLM determines it needs to interact with external systems—like reading files, running code, or fetching data—it generates tool calls that the Agent executes on its behalf.

Agents report tool calls through [`session/update`](/protocol/v2/prompt-lifecycle#3-agent-reports-output) notifications, allowing Clients to display real-time progress and results to users.

While Agents handle the actual execution, they may use Client-mediated
interactions like [permission requests](#requesting-permission) to provide a
richer, more integrated experience.

## Reporting

When the language model requests a tool invocation, the Agent **SHOULD** report it to the Client with a `tool_call_update`:

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "sess_abc123def456",
    "update": {
      "sessionUpdate": "tool_call_update",
      "toolCallId": "call_001",
      "title": "Reading configuration file",
      "kind": "read",
      "status": "pending"
    }
  }
}
```

<ParamField path="toolCallId" type="ToolCallId" required>
  A unique identifier for this tool call within the session
</ParamField>

<ParamField path="title" type="string">
  A human-readable title describing what the tool is doing. Agents **SHOULD**
  include the title the first time they report a `toolCallId`.
</ParamField>

<ParamField path="kind" type="ToolKind">
  The category of tool being invoked.

<Expandable title="kinds">
  - `read` - Reading files or data - `edit` - Modifying files or content -
  `delete` - Removing files or data - `move` - Moving or renaming files -
  `search` - Searching for information - `execute` - Running commands or code -
  `think` - Internal reasoning or planning - `fetch` - Retrieving external data
  - `other` - Other tool types (default)
</Expandable>

Tool kinds help Clients choose appropriate icons and optimize how they display tool execution progress.

Custom or future tool kinds can be used when Clients can fall back to generic tool display behavior. Custom tool kinds **MUST** begin with `_`. Unknown non-underscore tool kinds are reserved for future ACP variants.

</ParamField>

<ParamField path="status" type="ToolCallStatus">
  The current [execution status](#status) (defaults to `pending`)
</ParamField>

<ParamField path="content" type="ToolCallContent[]">
  [Content produced](#content) by the tool call
</ParamField>

<ParamField path="locations" type="ToolCallLocation[]">
  [File locations](#following-the-agent) affected by this tool call
</ParamField>

<ParamField path="rawInput" type="object">
  The raw input parameters sent to the tool
</ParamField>

<ParamField path="rawOutput" type="object">
  The raw output returned by the tool
</ParamField>

The `tool_call_update` notification is an upsert keyed by `toolCallId`. For
existing tool calls, fields other than `_meta` leave the previous value
unchanged when omitted, explicitly clear or unset the value when `null`, and
replace the previous value when concrete values are sent. For a new
`toolCallId`, omitted fields use Client defaults. `content` and `locations` are
replaced as whole arrays; send `[]` or `null` to clear them. For `_meta`, omit
the field to leave it unchanged or set it to `null` to clear it. Use
`tool_call_content_chunk` when a tool produces content incrementally and the
Client should append each item instead of replacing the whole `content`
collection.

## Updating

As tools execute, Agents send updates to report progress and results.

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "sess_abc123def456",
    "update": {
      "sessionUpdate": "tool_call_update",
      "toolCallId": "call_001",
      "status": "in_progress",
      "content": [
        {
          "type": "content",
          "content": {
            "type": "text",
            "text": "Found 3 configuration files..."
          }
        }
      ]
    }
  }
}
```

All fields except `toolCallId` are optional in updates. Only the fields being changed need to be included.

## Streaming Content

As tools execute, Agents **MAY** stream individual content items with
`tool_call_content_chunk`:

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "sess_abc123def456",
    "update": {
      "sessionUpdate": "tool_call_content_chunk",
      "toolCallId": "call_001",
      "content": {
        "type": "content",
        "content": {
          "type": "text",
          "text": "Found 3 configuration files..."
        }
      }
    }
  }
}
```

<ParamField path="toolCallId" type="ToolCallId" required>
  The ID of the tool call this content belongs to
</ParamField>

<ParamField path="content" type="ToolCallContent" required>
  A single [content item](#content) produced by the tool call
</ParamField>

Clients apply `tool_call_update` and `tool_call_content_chunk` notifications in
the order they are received for each `toolCallId`. A
`tool_call_content_chunk` appends its `content` item to the current tool-call
content. A later `tool_call_update` with `content` replaces all content
currently stored for that tool call, including content accumulated from earlier
chunks. Later chunks append to that replacement content. A `tool_call_update`
with `content: []` or `content: null` clears the tool-call content. A
`tool_call_content_chunk`'s `_meta`, when present, is chunk-scoped.

## Requesting Permission

The Agent **MAY** request permission from the user before proceeding with an operation, such as executing a tool call, by calling the `session/request_permission` method:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "session/request_permission",
  "params": {
    "sessionId": "sess_abc123def456",
    "title": "Approve file edit?",
    "description": "Allow the agent to edit src/main.rs?",
    "subject": {
      "type": "tool_call",
      "toolCall": {
        "toolCallId": "call_001"
      }
    },
    "options": [
      {
        "optionId": "allow-once",
        "name": "Allow once",
        "kind": "allow_once"
      },
      {
        "optionId": "reject-once",
        "name": "Reject",
        "kind": "reject_once"
      }
    ]
  }
}
```

<ParamField path="sessionId" type="SessionId" required>
  The session ID for this request
</ParamField>

<ParamField path="title" type="string" required>
  Title shown with the permission prompt. This text is separate from any
  `toolCall` update and does not replace the tool-call title.
</ParamField>

<ParamField path="description" type="string">
  Optional explanation shown with the permission prompt. This text is separate
  from any `toolCall` update and does not replace tool-call content. Omitted or
  `null` means no separate permission description was provided.
</ParamField>

<ParamField path="subject" type="RequestPermissionSubject">
  Optional structured context about the operation requiring permission. For tool
  calls, use `type: "tool_call"` with a `toolCall` update containing details
  about the operation. Omitted or `null` means no structured subject was
  provided. Custom or future subject types may appear. Clients that do not
  understand a subject should preserve it when proxying and use the common
  prompt fields or decline according to policy.
</ParamField>

<ParamField path="options" type="PermissionOption[]" required>
  Available [permission options](#permission-options) for the user to choose
  from. Must contain at least one option.
</ParamField>

The Client responds with the user's decision:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "outcome": {
      "outcome": "selected",
      "optionId": "allow-once"
    }
  }
}
```

Clients **MAY** automatically allow or reject permission requests according to the user settings.

If the current active work gets [cancelled](/protocol/v2/prompt-lifecycle#cancellation), the Client **MUST** respond with the `"cancelled"` outcome:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "outcome": {
      "outcome": "cancelled"
    }
  }
}
```

<ResponseField name="outcome" type="RequestPermissionOutcome" required>
  The user's decision. Known outcomes are `cancelled` when the [active work was
  cancelled](/protocol/v2/prompt-lifecycle#cancellation), and `selected` with an
  `optionId` for the selected permission option. Custom or future outcomes may
  appear; agents that do not understand the outcome MUST NOT treat it as
  approval.
</ResponseField>

### Permission Options

Each permission option provided to the Client contains:

<ParamField path="optionId" type="string" required>
  Unique identifier for this option
</ParamField>

<ParamField path="name" type="string" required>
  Human-readable label to display to the user
</ParamField>

<ParamField path="kind" type="PermissionOptionKind" required>
  A hint to help Clients choose appropriate icons and UI treatment for each option.

- `allow_once` - Allow this operation only this time
- `allow_always` - Allow this operation and remember the choice
- `reject_once` - Reject this operation only this time
- `reject_always` - Reject this operation and remember the choice

Custom or future permission option kinds can be used as UI hints. Custom kinds **MUST** begin with `_`. Unknown non-underscore kinds are reserved for future ACP variants. Clients that do not understand a kind should preserve the value and use a generic permission option treatment.

</ParamField>

## Status

Tool calls progress through different statuses during their lifecycle:

<ResponseField name="pending">
  The tool call hasn't started running yet because the input is either streaming
  or awaiting approval
</ResponseField>

<ResponseField name="in_progress">
  The tool call is currently running
</ResponseField>

<ResponseField name="completed">
  The tool call completed successfully
</ResponseField>

<ResponseField name="failed">The tool call failed with an error</ResponseField>

Custom or future status values can be used when Clients can display a generic progress state. Custom status values **MUST** begin with `_`; unknown non-underscore statuses are reserved for future ACP variants.

## Content

Tool calls can produce different types of content:

Tool call content `type` values can also be custom or future variants. Implementations should preserve unknown content payloads when storing, replaying, proxying, or forwarding tool calls, and otherwise render a generic content item or ignore the item if no safe display is available.

### Regular Content

Standard [content blocks](/protocol/v2/content) like text, images, or resources:

```json
{
  "type": "content",
  "content": {
    "type": "text",
    "text": "Analysis complete. Found 3 issues."
  }
}
```

### Diffs

File modifications shown as diffs. A diff always includes structured file
changes and can optionally include renderable patch text.

Clients can use `changes` to identify affected absolute paths and file
operations without parsing patch text. When `patch` is present, clients can
also render the patch text directly. Agents SHOULD provide `patch` whenever
feasible. Clients MUST handle diffs where `patch` is omitted or `null`.

```json
{
  "type": "diff",
  "changes": [
    {
      "operation": "modify",
      "path": "/home/user/project/src/config.json",
      "fileType": "text",
      "mimeType": "application/json"
    }
  ],
  "patch": {
    "format": "git_patch",
    "diff": "diff --git /home/user/project/src/config.json /home/user/project/src/config.json\n--- /home/user/project/src/config.json\n+++ /home/user/project/src/config.json\n@@ -1,3 +1,3 @@\n {\n-  \"debug\": false\n+  \"debug\": true\n }\n"
  }
}
```

<ParamField path="changes" type="DiffChange[]" required>
  Structured file changes described by this diff.
</ParamField>

<ParamField path="patch" type="DiffPatch">
  Optional renderable patch text. Agents SHOULD provide this whenever feasible.
  Omitted or `null` means no patch text was provided.
</ParamField>

<ParamField path="patch.format" type="string" required>
  Patch format. The ACP-defined value is `git_patch`.
</ParamField>

<ParamField path="patch.diff" type="string" required>
  Git patch text.
</ParamField>

<ParamField path="changes[].operation" type="string" required>
  File operation: `add`, `delete`, `modify`, `move`, or `copy`.
</ParamField>

<ParamField path="changes[].path" type="string" required>
  The absolute path after the operation. For deletes, this is the deleted path.
</ParamField>

<ParamField path="changes[].oldPath" type="string">
  The absolute path before the operation. Required for `move` and `copy`.
</ParamField>

<ParamField path="changes[].fileType" type="string">
  Optional file kind: `text`, `binary`, `directory`, or `symlink`.
</ParamField>

<ParamField path="changes[].mimeType" type="string">
  Optional MIME type for the file contents.
</ParamField>

Omit `patch` when there is no useful text patch, such as a same-path binary
update or symlink target change:

```json
{
  "type": "diff",
  "changes": [
    {
      "operation": "modify",
      "path": "/home/user/project/assets/logo.png",
      "fileType": "binary",
      "mimeType": "image/png"
    }
  ]
}
```

## Following the Agent

Tool calls can report file locations they're working with, enabling Clients to implement "follow-along" features that track which files the Agent is accessing or modifying in real-time.

```json
{
  "path": "/home/user/project/src/main.py",
  "line": 42
}
```

<ParamField path="path" type="string" required>
  The absolute file path being accessed or modified
</ParamField>

<ParamField path="line" type="number">
  Optional line number within the file
</ParamField>
