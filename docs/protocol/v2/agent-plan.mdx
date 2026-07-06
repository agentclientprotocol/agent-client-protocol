---
title: "Agent Plan"
description: "How Agents communicate their execution plans"
---

Plans are execution strategies for complex tasks that require multiple steps.

Agents may share plans with Clients through [`session/update`](/protocol/v2/prompt-lifecycle#3-agent-reports-output) notifications, providing real-time visibility into their thinking and progress.

`plan_update` carries a `plan` object with a `type` discriminator and a required `planId` so Clients can track multiple plans independently.

The plan content type is `items`. Additional plan operations remain unstable while the Plan Operations RFD is in progress.

## Creating Plans

When the language model creates an execution plan, the Agent **SHOULD** report it to the Client with `plan_update`.

### Item-Based Plans

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "sess_abc123def456",
    "update": {
      "sessionUpdate": "plan_update",
      "plan": {
        "type": "items",
        "planId": "plan-1",
        "entries": [
          {
            "content": "Analyze the existing codebase structure",
            "priority": "high",
            "status": "pending"
          },
          {
            "content": "Identify components that need refactoring",
            "priority": "high",
            "status": "pending"
          },
          {
            "content": "Create unit tests for critical functions",
            "priority": "medium",
            "status": "pending"
          }
        ]
      }
    }
  }
}
```

<ParamField path="plan.type" type="string" required>
  The plan content type.
</ParamField>

<ParamField path="plan.planId" type="PlanId" required>
  A unique identifier for this plan within the session.
</ParamField>

<ParamField path="plan.entries" type="PlanEntry[]" required>
  An array of [plan entries](#plan-entries) representing the tasks to be
  accomplished.
</ParamField>

## Extensibility

`plan.type` values can be custom or future variants when Clients can preserve or display them generically. Custom plan content types **MUST** begin with `_`; unknown non-underscore plan content types are reserved for future ACP variants.

Every plan content variant, including custom or future variants, **MUST** carry a `planId`.

## Plan Entries

Each plan entry represents a specific task or goal within the overall execution strategy:

<ParamField path="content" type="string" required>
  A human-readable description of what this task aims to accomplish
</ParamField>

<ParamField path="priority" type="PlanEntryPriority" required>
  The relative importance of this task.

- `high`
- `medium`
- `low`

Custom or future priority values can be used for display hints. Custom priorities **MUST** begin with `_`; unknown non-underscore priorities are reserved for future ACP variants.

</ParamField>

<ParamField path="status" type="PlanEntryStatus" required>
  The current [execution status](#status) of this task

- `pending`
- `in_progress`
- `completed`

Custom or future status values can be used when Clients can preserve or display a generic plan-entry state. Custom statuses **MUST** begin with `_`; unknown non-underscore statuses are reserved for future ACP variants.

</ParamField>

## Updating Plans

As the Agent progresses through the plan, it **SHOULD** report updates by sending more `session/update` notifications with the same `sessionUpdate: "plan_update"` structure and `planId`.

For item-based plans, the Agent **MUST** send a complete list of all plan entries in each update and their current status. The Client **MUST** replace the current contents of that plan completely.

### Dynamic Planning

Plans can evolve during execution. The Agent **MAY** add, remove, or modify plan entries as it discovers new requirements or completes tasks, allowing it to adapt based on what it learns.
