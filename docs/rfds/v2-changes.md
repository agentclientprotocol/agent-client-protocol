---
title: "v2 Changes"
---

Author(s): [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

> What are you proposing to change?

<!--
    Give a brief, high-level overview of what you plan to do and what problem you are solving. Feel free to use bullet points to help clarify the structure.
-->

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

## What we propose to do about it

> What are you proposing to improve the situation?

<!--
    Use this section to describe what you propose to do at a high-level.
    Don't give every detail, this should be the high-level summary.

    Note: This section is OPTIONAL when RFDs are first opened.
    You can also include multiple variants if you have different ideas of how to approach the problem, though these should be narrowed down as the RFD progresses.
-->

## Shiny future

> How will things will play out once this feature exists?

<!--
    Use this section to describe the "status quo" as it will play out once
    we have made these changes.

    Note: This section is OPTIONAL when RFDs are first opened.
-->

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

### Cleanup and Alignment

#### Clean up capabilities

#### Enum variant extension

Same as session config categories: \_ for extension, preserve non-underscore for future variants

#### Non-streaming Messages

#### Streaming Tool Calls

#### JSON-RPC Batch

### Behavior Changes

#### v2 Prompting

### New Features

#### Message IDs

#### Truncate/Edit

#### Fork

#### session/new: Provide starting messages

Potentially request config options?
Response provides available commands

#### Terminal Output

### Removals

#### Session modes (and unstable models)

Streaming + non-streaming as well

#### Richer Diffs

#### Plan Variants

#### Subagents

### Transports

RFD for HTTP + Websockets

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

<!--
    Keep this section up-to-date as discussion proceeds. The goal is to capture major points that came up on a PR or in a discussion forum -- and if they reoccur, to point people to the FAQ so that we can start the dialog from a more informed place.
-->

### What alternative approaches did you consider, and why did you settle on this one?

None. The idea came to me fully formed, like Athena springing from Zeus's head.

<!-- You...may want to adjust this. -->

## Revision history

<!-- If there have been major updates to this RFD, you can include the git revisions and a summary of the changes. -->
