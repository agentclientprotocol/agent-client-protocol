# Agent Client Protocol (ACP) Governance

Note: This document describes how ACP governance is structured today (Zed-led) and paints a picture of our desired evolutionary path (following MCP).

## Current Governance Model (Zed-led)

- Given the rapid iteration on the core protocol, Zed _currently_ leads ACP in a "Lead Maintainer"/"[BDFL](https://en.wikipedia.org/wiki/Benevolent_dictator_for_life)" capacity
- As the foundations of ACP harden and adoption increases, Zed _intends_ on transitioning ACP to a hierarchical structure with a steering committee, similar to [MCP](https://modelcontextprotocol.io/community/governance)

### Core Ownership

- All ACP repositories are housed under a neutral Github Organization - [agentclientprotocol](https://github.com/agentclientprotocol).
  - Zed owns and administers this organization, the main protocol repo, and all repositories not administered by other companies or contributors

### Component-Specific Admins

- Companies and individuals with relevant expertise may contact Zed for component-specific administration rights
  - The goal of component-specific admins: allowing companies/indiviuals with specific knowledge and skillsets to administer code they're best equipped to administer
    - Example: JetBrains administers [the Kotlin SDK](https://github.com/agentclientprotocol/kotlin-sdk)
  - Companies or individuals interested in administration may contact Zed via shared Slack channel or at hi@zed.dev
- Access to component administration is granted via _teams_
  - These teams have admin permissions to their specific repository/repositories, and write permissions to the [main protocol repository](https://github.com/agentclientprotocol/agent-client-protocol)
- Zed will provide suggested permissions/rule sets for component admins, but admins are free to update these permissions within their repository(ies) as they see fit

### Contribution and PR Control

- Contributors should follow a discussion -> PR model
  - Discussions should address what change is looking to be made, why, and summarize the hypothesized technical direction
  - PRs submitted without prior discussion may be redirected to create a discussion, especially for larger changes that need more input.
  - Repository admins (whether Zed, or specific component admins) are required code reviewers to merge any PRs
- Proposals for breaking changes to the **protocol** must be flagged for a 5 business day review/objection window before they are merged.
  - if no objections, the Zed team may self-merge after the window expires
- Non-breaking changes can be merged at any time. However, changes that would require additional implementation effort should have followed the discussion model above, and ideally without major objections.
- Repository admins, in **their own repository** may merge non-breaking and breaking changes at any time. It is acknowledged that library design may require changes over time, and each repository can follow its own method of gathering feedback and discussion before making breaking changes.

## Future Governance Model

- The current target structure for ACP is modeled after [MCP](https://modelcontextprotocol.io/community/governance)
  - Contributors
  - Maintainers (component-level)
  - Core Maintainers (project-level)
  - Lead Maintainer(s) / BDFL (initially Zed; will evolve)
  - Steering Committee
- _Unlike_ MCP, we have a hypothesis that companies will have a strong role in the future governance model of ACP, given its primary utility _today_ is for company interactivity. That's just a hypothesis, though, and as the protocol develops we're happy to hear alternate perspectives.

## Security Policy and Vulnerability Disclosure

- Zed will triage all potential security and vulnerability issues between the Zed team and other maintainers
- Reports can be submitted to security@zed.dev

## Communication

- Asynchronous communication lives in GitHub
  - Discussions about the protocol, future direction, governance, etc. should be in the form of a discussion
  - Bugs, feature requests, etc. should be submitted as an issue

## Legal, Licensing, and Contributor Terms

- By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
