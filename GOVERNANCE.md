# Agent Client Protocol (ACP) Governance

Note: This document describes how ACP governance is structured today (Zed-led) and paints a picture of our desired evolutionary path (following MCP).

## Design and Governance Principles
- TODO enumerate principles
    - editor/agent decoupling
    - interoperability
    - _others_

## Current Governance Model (Zed-led)
- Authority
  - Zed acts as Lead Maintainer (BDFL-style) for the overall project.
- Code
  - Repositories are housed under a neutral Github Organization - [agentclientprotocol](https://github.com/agentclientprotocol).
  - Permissions model:
        TODO describe admin/maintainer roles and least-privilege setup.
- Delegated Component Stewardship
  - Companies and individuals with relevant expertise may contact Zed for component-specific administration rights
    - The goal of component-specific admins: allow companies/indiviuals with specific knowledge and skillsets to administer code they're best equipped to administer
    - Example: JetBrains as admins of the Kotlin SDK
  - Appointment mechanism: contact Zed via shared Slack channel or at hi@zed.dev
- Contribution and Change Control
    - TODO: admins can do whatever to their repos subject to
        - TODO: Non-breaking changes: admins may merge directly, subject to repo protections.
        - TODO: Breaking changes: review/objection window of 5 business days before merge.
    - TODO: propose changes via discussion -> then PR
        - TODO: PRs approved by admin
        - TODO: admins can self merge after 5 business day window
        - TODO: non-admins must have code merged by admin
- Conflict Resolution
  - TODO Escalation path
  - Final decision-maker: Zed during Phase 0

## Future Governance Model
- Target Structure (modeled after MCP)
  - Contributors
  - Maintainers (component-level)
  - Core Maintainers (project-level)
  - Lead Maintainer(s) / BDFL (initially Zed; will evolve)
  - Steering Committee

## Roles and Responsibilities
- Contributors
  - TODO
- Maintainers
  - TODO
- Lead Maintainers (BDFL)
  - TODO

## Versioning, Releases, and Deprecation
- Specification
  - TODO Versioning scheme
  - TODO Deprecation policy
- Migration Guides
  - TODO Required for breaking changes

## Security Policy and Vulnerability Disclosure
- Reporting channel:
    - TODO email/security.txt/GitHub advisories.

## Communication
- Public Channels
  - TODO GitHub (issues, discussions)

## Legal, Licensing, and Contributor Terms
- License
  - TODO Apache 2.0
- By contributing, you agree that your contributions will be licensed under the MIT License.
