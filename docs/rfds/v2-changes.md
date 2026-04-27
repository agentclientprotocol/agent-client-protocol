---
title: "v2 Changes"
---

Author(s): [@benbrandt](https://github.com/benbrandt)

## Elevator pitch

> What are you proposing to change?

With ACP, we aim to move fast while keeping breaking changes to a minimum. However, we've gotten to a point where there are enough changes we would like to do that would benefit from some core redesigns that will allow for extending the protocol with new features more easily.

We've also managed to add new features that has led to learnings that would benefit from consolidation and alignment in other areas of the protocol to smooth things out and make things more consistent.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

We have had a fairly successful time adding new features via new capabilities and adding in new features in a non-breaking way. But some of the learnings we have made will require breaking changes, and it feels like there are enough of these built up, or RFDs we are stuck due to required changes that now is a good time to do so.

## What we propose to do about it

> What are you proposing to improve the situation?

#### New Required Capabilities

#### Clean up capabilities

#### Enum variant extension

Same as session config categories: \_ for extension, preserve non-underscore for future variants

#### Non-streaming Messages

#### Streaming Tool Calls

#### JSON-RPC Batch

#### v2 Prompting

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

## Shiny future

> How will things will play out once this feature exists?

<!--
    Use this section to describe the "status quo" as it will play out once
    we have made these changes.

    Note: This section is OPTIONAL when RFDs are first opened.
-->

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

### v2 + v1 Schema publishing

### SDK Support

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
