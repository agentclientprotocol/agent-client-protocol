---
title: "Represent deleted files in diff"
---

Author(s): [anna239](https://github.com/benbrandt)

## Elevator pitch

> What are you proposing to change?

Add flag `deleted` to [Diff](https://agentclientprotocol.com/protocol/tool-calls#diffs) entity type for the case of a deleted file.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

Currently, in Diff entity type `newText` is not nullable, so it's not possible to distinguish between a deleted file and empty file.

## What we propose to do about it

> What are you proposing to improve the situation?

Add flag `deleted` to [Diff](https://agentclientprotocol.com/protocol/tool-calls#diffs) entity type for the case of a deleted file.

**Current structure (cannot distinguish deleted file from empty file):**

```json
{
  "type": "diff",
  "path": "/home/user/project/src/config.json",
  "oldText": "{\n  \"debug\": false\n}",
  "newText": ""
}
```

**Proposed structure with `deleted` flag:**

```json
{
  "type": "diff",
  "path": "/home/user/project/src/config.json",
  "oldText": "{\n  \"debug\": false\n}",
  "newText": "",
  "deleted": true
}
```

Note: we would ideally make newText nullable, but that would break existing clients.

## Shiny future

> How will things will play out once this feature exists?

It is possible for the agent to distinguish between a deleted file and an empty file.

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

Adding the new field will be a non-breaking change, and clients that update can better distinguish between deleted and empty files.

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

**Do we need to represent moved files?**

An agent could represent that with a deleted file at the old path and a new file at the new path.

We need to rework the entire diff structure to handle more cases, and binary files, but in the meantime this provides a stop-gap until we can implement a more comprehensive solution.

### What alternative approaches did you consider, and why did you settle on this one?

We considered making newText nullable, but that would break existing clients.

## Revision history

2026-02-20: Initial draft
