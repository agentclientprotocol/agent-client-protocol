---
title: "ACP v2 Proposal"
---

Author(s): [@benbrandt](https://github.com/benbrandt)

This is a tracking RFD for the collection of RFDs that require breaking changes to the protocol and should make up ACP v2.

## Elevator pitch

> What are you proposing to change?

With ACP, we aim to move fast while keeping breaking changes to a minimum. However, we've gotten to a point where there are enough changes we would like to do that would benefit from some core redesigns that will allow for extending the protocol with new features more easily.

We've also managed to add new features that has led to learnings that would benefit from consolidation and alignment in other areas of the protocol to smooth things out and make things more consistent.

## Status quo

> How do things work today and what problems does this cause? Why would we change things?

We have had a fairly successful time adding new features via new capabilities and adding in new features in a non-breaking way. But some of the learnings we have made will require breaking changes, and it feels like there are enough of these built up, or RFDs we are stuck due to required changes that now is a good time to do so.

## What we propose to do about it

> What are you proposing to improve the situation?

### Current Draft RFDs

Current RFDs accepted as Drafts that are targeting v2 release

- [New Prompt Lifecycle](./prompt.md)
- [Message IDs](../message-id.md)
  - [Fork from specified IDs](../session-fork.mds)
- [Remote Transports](../streamable-http-websocket-transport.mdx)

Other RFDs will progress separately and are not dependent on breaking changes (specifically the new prompt lifecycle) and can land in either or both v1 and v2.

### RFDs to be Written

Changes under consideration that still need to be drafted or moved to draft:

- Capabilities: clean up naming and organization, as well as make some more capabilities required.
- Enum variant extension: Same as session config categories: \_ for extension, preserve non-underscore for future variants
- Streaming/Non-streaming consistency: Offer both options for both messages and tool calls
  - Also Terminal Output type for streaming terminal output from an agent
  - Expand diff types
- JSON-RPC Batch messages: clearer guidance on how SDK support should work for these
- Truncate/Edit support
- session/new changes:
  - Providing starting message history
  - Response can provide available commands
  - Potentially config options
- Remove session modes and (unstable) session models. These are replaces by session config options
- Plan variants
- Subagent support

## Shiny future

> How will things will play out once this feature exists?

There is a lof of work to do, especially on the SDK side, to support both versions, but it is likely that we should be able to allow Agents specifically to target v2 apis and gracefully fallback to v1 messages for v1 clients, to avoid huge support issues.

However, once all of this work is in place, it should be much easier to make additional breaking changes in the future when necessary, we've been kind of letting this build up given the effort required for the entire ecosystem, but the ACP maintainers will be charting a course forward to make this as smooth as possible!

## Implementation details and plan

> Tell me more about your implementation. What is your detailed implementation plan?

### v2 + v1 Schema publishing

I have created a [draft of the v2 schema](https://github.com/agentclientprotocol/agent-client-protocol/pull/1099), which is currently a direct duplicate of v1.

This also has the necessary conversion types that are needed for Rust at least to convert between the two. But this has a nice side-effect of a clear diff of how the schema will change and also what conversion is necessary. So the plan is to start proposing draft RFDs with the relevant schema changes where possible for approval.

Once we have more pieces in place, we can start publishing both schemas to assist SDK developers to start figuring out how to support this. **This should be done in an opt-in, off by default, clearly labeled unstable way for SDK consumers**. There will likely be bumps as we figure out the necessary plumbing and we shouldn't be shipping v2 in production without feature flags prior to a more stable release as we align all of the necessary pieces.

### SDK Support

With the needed breaking changes, as much as possible I am targeting having a consistent API surface for Agents, since they will want to target v2 apis but still support v1 clients. Because of how the version negotiation works, if we can achieve the same thing for clients that will be great, but if not, they will at least be provided clear version entrypoints.

## Frequently asked questions

> What questions have arisen over the course of authoring this document or during subsequent discussions?

## Revision history

2026-05-06: Initial draft
