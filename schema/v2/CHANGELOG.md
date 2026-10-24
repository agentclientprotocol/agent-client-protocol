# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0-alpha.10](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.9...schema-v2.0.0-alpha.10) - 2026-10-24

### Added

- *(unstable)* link session message views and keep identity off chunks ([#2465](https://github.com/agentclientprotocol/agent-client-protocol/pull/2465))
- *(unstable-v2)* define version mismatch errors ([#2412](https://github.com/agentclientprotocol/agent-client-protocol/pull/2412))

### Other

- *(deps)* bump the minor group with 9 updates ([#2437](https://github.com/agentclientprotocol/agent-client-protocol/pull/2437))

## [2.0.0-alpha.9](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.8...schema-v2.0.0-alpha.9) - 2026-10-16

### Added

- *(unstable-v2)* patch upsert _meta by top-level key ([#2372](https://github.com/agentclientprotocol/agent-client-protocol/pull/2372))
- *(unstable)* add client system prompt schema ([#2361](https://github.com/agentclientprotocol/agent-client-protocol/pull/2361))

### Fixed

- *(unstable)* remove next edit suggestions ([#2266](https://github.com/agentclientprotocol/agent-client-protocol/pull/2266))
- *(schema)* align v2 experimental feature compatibility ([#2397](https://github.com/agentclientprotocol/agent-client-protocol/pull/2397))

### Other

- *(unstable-v2)* define foreground state transitions ([#2411](https://github.com/agentclientprotocol/agent-client-protocol/pull/2411))
- *(v2)* clarify replay IDs and unsupported cursors ([#2398](https://github.com/agentclientprotocol/agent-client-protocol/pull/2398))
- *(v2)* align normative cancellation requirements ([#2390](https://github.com/agentclientprotocol/agent-client-protocol/pull/2390))
- *(unstable-v2)* clarify prompt insertion contract ([#2392](https://github.com/agentclientprotocol/agent-client-protocol/pull/2392))
- *(unstable-v2)* clarify foreground work lifecycle ([#2387](https://github.com/agentclientprotocol/agent-client-protocol/pull/2387))
- *(v2)* clarify prompt cancellation when closing sessions ([#2385](https://github.com/agentclientprotocol/agent-client-protocol/pull/2385))
- *(schema)* specify how git_patch writes paths ([#2362](https://github.com/agentclientprotocol/agent-client-protocol/pull/2362))
- *(unstable)* clarify that a forked session is active ([#2359](https://github.com/agentclientprotocol/agent-client-protocol/pull/2359))
- *(schema)* list full v2 session baseline on AgentCapabilities.session ([#2358](https://github.com/agentclientprotocol/agent-client-protocol/pull/2358))
- *(schema)* clarify v2 foreground work can start without a prompt ([#2357](https://github.com/agentclientprotocol/agent-client-protocol/pull/2357))

## [2.0.0-alpha.8](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.7...schema-v2.0.0-alpha.8) - 2026-10-09

### Added

- *(schema)* stabilize session notices ([#2336](https://github.com/agentclientprotocol/agent-client-protocol/pull/2336))
- *(schema)* stabilize session compaction ([#2334](https://github.com/agentclientprotocol/agent-client-protocol/pull/2334))
- *(unstable)* report failures after prompt insertion with an error stop reason ([#2281](https://github.com/agentclientprotocol/agent-client-protocol/pull/2281))

### Fixed

- *(schema)* reject malformed values receivers act on ([#2333](https://github.com/agentclientprotocol/agent-client-protocol/pull/2333))

### Other

- *(schema)* clarify v2 entity IDs are unique per entity type ([#2332](https://github.com/agentclientprotocol/agent-client-protocol/pull/2332))

## [2.0.0-alpha.7](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.6...schema-v2.0.0-alpha.7) - 2026-09-30

### Fixed

- *(unstable)* allow MCP response extensions ([#2265](https://github.com/agentclientprotocol/agent-client-protocol/pull/2265))

## [2.0.0-alpha.6](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.5...schema-v2.0.0-alpha.6) - 2026-09-30

### Added

- *(unstable-v2)* return available commands in session responses ([#2259](https://github.com/agentclientprotocol/agent-client-protocol/pull/2259))
- *(unstable)* Initial RFD and schema for subagents ([#1992](https://github.com/agentclientprotocol/agent-client-protocol/pull/1992))
- *(unstable)* make MCP-over-ACP request-scoped ([#2223](https://github.com/agentclientprotocol/agent-client-protocol/pull/2223))

## [2.0.0-alpha.5](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.4...schema-v2.0.0-alpha.5) - 2026-09-18

### Added

- *(unstable-v2)* return message ID on prompt insertion ([#2175](https://github.com/agentclientprotocol/agent-client-protocol/pull/2175))

## [2.0.0-alpha.4](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.3...schema-v2.0.0-alpha.4) - 2026-09-17

### Added

- *(schema)* stabilize tool call name ([#2166](https://github.com/agentclientprotocol/agent-client-protocol/pull/2166))
- *(unstable)* add session notices schema ([#2004](https://github.com/agentclientprotocol/agent-client-protocol/pull/2004))

## [2.0.0-alpha.3](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.2...schema-v2.0.0-alpha.3) - 2026-08-20

### Added

- *(unstable)* add session compaction updates ([#2002](https://github.com/agentclientprotocol/agent-client-protocol/pull/2002))
- *(schema)* stabilize terminal authentication ([#2000](https://github.com/agentclientprotocol/agent-client-protocol/pull/2000))
- *(schema)* stabilize elicitation ([#1779](https://github.com/agentclientprotocol/agent-client-protocol/pull/1779))

### Other

- *(rfd)* Move terminal auth to preview ([#1796](https://github.com/agentclientprotocol/agent-client-protocol/pull/1796))

## [2.0.0-alpha.2](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.1...schema-v2.0.0-alpha.2) - 2026-07-21

### Added

- *(unstable)* add tool call name ([#1752](https://github.com/agentclientprotocol/agent-client-protocol/pull/1752))
- *(unstable-v2)* add cancelled variant to tool call and plan item ([#1750](https://github.com/agentclientprotocol/agent-client-protocol/pull/1750))

## [2.0.0-alpha.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.0...schema-v2.0.0-alpha.1) - 2026-07-20

### Added

- *(unstable-v2)* add v2 semantic string types ([#1742](https://github.com/agentclientprotocol/agent-client-protocol/pull/1742))
- *(unstable-v2)* rename diff patch payload to text ([#1730](https://github.com/agentclientprotocol/agent-client-protocol/pull/1730))
- *(unstable-v2)* add v2 terminal output surface ([#1679](https://github.com/agentclientprotocol/agent-client-protocol/pull/1679))

### Fixed

- *(unstable-v2)* gate v2 auth methods on authMethods ([#1728](https://github.com/agentclientprotocol/agent-client-protocol/pull/1728))
- *(schema)* remove enum discriminators from invalid schemas ([#1612](https://github.com/agentclientprotocol/agent-client-protocol/pull/1612))

### Other

- *(unstable-v2)* clarify idle session semantics ([#1729](https://github.com/agentclientprotocol/agent-client-protocol/pull/1729))
- enable schema v2 releases ([#1632](https://github.com/agentclientprotocol/agent-client-protocol/pull/1632))
