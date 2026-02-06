---
title: "Agent Message Clear Mechanism for Non-Monotonic Streaming"
---

Author(s): [steve02081504](https://github.com/steve02081504)

## Elevator pitch

The current Agent Communication Protocol (ACP) treats `agent_message_chunk` as a strictly append-only operation. This proposal introduces a new session update type, `agent_message_clear`, which allows agents to clear the accumulated streamed content. This enables support for non-monotonic streaming scenarios—such as iterative refinement, post-processing, and speculative decoding—without causing UI clutter or redundant message history.

## Status quo

Currently, the `agent_message_chunk` session update is strictly cumulative. The client concatenates all received chunks into the agent's message with no mechanism to clear or replace what has already been streamed. This is problematic for agents where content generation is non-monotonic:

1.  **Post-processing / reformatting**: Backends applying progressive Markdown rendering or XML tag balancing may need to reformat intermediate text.
2.  **Delegating agent frameworks**: Frameworks like [fount](https://github.com/steve02081504/fount) receive full accumulated states rather than deltas. When internal processing occurs (e.g., hiding tool-calls), the new state is no longer a simple prefix of the old one.
3.  **Iterative Refinement & Diffusion-based Models**: These models refine entire text blocks over multiple steps rather than generating tokens linearly.
4.  **Speculative decoding with rollback**: Inference engines may need to retract tokens if a drafted branch is rejected.
5.  **Display vs. Context Divergence**: An agent might stream a simplified preview and wish to replace it with a final richly-formatted version.

**Current Workarounds:**
*   **Separator & Re-push**: Agents append a visual separator (like `---`) and re-send the entire text. This causes **UI clutter**, **flicker**, and **massive context waste** in transcripts.
*   **Silent Freezing**: Agents stop streaming until the text "catches up" to the previous length, making the UI feel laggy or frozen.

## What we propose to do about it

We propose adding a new session update type: `agent_message_clear`. 

This update instructs the client to immediately clear the accumulated streamed content for the current agent message. Subsequent `agent_message_chunk` updates will then start appending from an empty state.

**Key characteristics:**
- **Simplicity**: It follows the existing "full-replacement" semantics found in `plan` updates and `tool_call_update` content.
- **Minimal Scope**: It introduces a single new `SessionUpdate` variant without changing existing logic.
- **Graceful Degradation**: Clients that do not recognize this update will ignore it, defaulting to the current (concatenated) behavior.

## Shiny future

In the shiny future, agent UIs will be fluid and reactive. 
- **Clean Recovery**: If an agent makes a "mistake" in its stream or needs to reformat a table on the fly, it can do so invisibly to the user's permanent history.
- **Next-Gen Support**: ACP will natively support diffusion-style language models that "paint" or "de-noise" a response over time.
- **Framework Harmony**: Developers using high-level frameworks (like `fount`) can push full-state updates safely, knowing the protocol handles the "reset" efficiently.
- **Optimized Context**: Chat transcripts will remain concise and readable, free of "Separator & Re-push" artifacts.

## Implementation details and plan

### Protocol Change
Add `agent_message_clear` to the `SessionUpdate` variant list.

**JSON-RPC Example:**
```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "sess_abc123",
    "update": {
      "sessionUpdate": "agent_message_clear"
    }
  }
}
```

### Usage Pattern
1. Agent sends `agent_message_chunk` with text "Drafting...".
2. Agent determines the draft needs replacement.
3. Agent sends `agent_message_clear`.
4. Agent sends `agent_message_chunk` with the final, polished content.

### Implementation Plan
1.  **Phase 1**: Update the ACP schema to include `agent_message_clear`.
2.  **Phase 2**: Implement handling in reference clients to clear the local buffer for the active message.
3.  **Phase 3 (Optional)**: Consider adding `agent_thought_clear` if agents require similar non-monotonic behavior for internal reasoning blocks.

## Frequently asked questions

### What alternative approaches did you consider, and why did you settle on this one?

Several designs were evaluated:

| Design | Reason for Rejection |
|---|---|
| **`agent_message_replace`** | While atomic, it requires a new content structure and results in very large payloads for long messages if sent frequently. |
| **`replace: true` flag** | Overloads the existing chunk type and makes partial-replace semantics ambiguous. |
| **Offset-based patching** | While bandwidth-efficient, it introduces significant complexity for both agent developers and client implementers (similar to Operational Transforms). |

`agent_message_clear` was chosen because it is the most consistent with existing ACP patterns (like `plan` updates) and provides the best balance between implementation simplicity and functional power.

### How does this affect backward compatibility?
It is fully backward-compatible. A legacy client will simply ignore the unknown `agent_message_clear` notification. The result is that the user sees the old text and new text concatenated, which is identical to the current "Separator" workaround.

### Does this replace the entire message history?
No. It only clears the "accumulated streamed content" of the **current** active agent message in the session. It does not affect previously finalized messages in the chat history.

## Revision history

- **2026-02-07**: Initial draft by steve02081504.
