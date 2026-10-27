# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.27.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.26.0...schema-v1.27.0) - 2026-10-27

### Added

- *(unstable)* link session message views and keep identity off chunks ([#2465](https://github.com/agentclientprotocol/agent-client-protocol/pull/2465))

### Fixed

- *(unstable)* null clears session_message participant IDs and message references ([#2555](https://github.com/agentclientprotocol/agent-client-protocol/pull/2555))
- *(unstable-v2)* make ToolCallLocation.line 1-based ([#2546](https://github.com/agentclientprotocol/agent-client-protocol/pull/2546))

### Other

- *(schema)* mark DefaultOnNull lists with an x-deserialize hint ([#2569](https://github.com/agentclientprotocol/agent-client-protocol/pull/2569))
- *(schema)* note that titled multi-select items omit type, as in MCP ([#2538](https://github.com/agentclientprotocol/agent-client-protocol/pull/2538))
- *(schema)* ExtNotification can be sent by either side ([#2534](https://github.com/agentclientprotocol/agent-client-protocol/pull/2534))
- *(schema)* include -32800's range in the reserved error codes ([#2531](https://github.com/agentclientprotocol/agent-client-protocol/pull/2531))
- *(schema)* write versioned doc links in source and drop the generator's link rewriting ([#2525](https://github.com/agentclientprotocol/agent-client-protocol/pull/2525))
- *(schema)* use wire names instead of Rust identifiers in schema descriptions ([#2520](https://github.com/agentclientprotocol/agent-client-protocol/pull/2520))
- *(schema)* write the error pages for v1 and v2 ([#2504](https://github.com/agentclientprotocol/agent-client-protocol/pull/2504))
- *(deps)* bump the minor group with 9 updates ([#2437](https://github.com/agentclientprotocol/agent-client-protocol/pull/2437))

## [1.26.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.25.0...schema-v1.26.0) - 2026-10-16

### Added

- *(unstable)* add client system prompt schema ([#2361](https://github.com/agentclientprotocol/agent-client-protocol/pull/2361))

### Fixed

- *(unstable)* remove next edit suggestions ([#2266](https://github.com/agentclientprotocol/agent-client-protocol/pull/2266))

### Other

- *(unstable)* clarify that a forked session is active ([#2359](https://github.com/agentclientprotocol/agent-client-protocol/pull/2359))

## [1.25.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.24.1...schema-v1.25.0) - 2026-10-09

### Added

- *(schema)* stabilize session notices ([#2336](https://github.com/agentclientprotocol/agent-client-protocol/pull/2336))
- *(schema)* stabilize session compaction ([#2334](https://github.com/agentclientprotocol/agent-client-protocol/pull/2334))
- *(unstable)* report failures after prompt insertion with an error stop reason ([#2281](https://github.com/agentclientprotocol/agent-client-protocol/pull/2281))

### Fixed

- *(schema)* reject malformed values receivers act on ([#2333](https://github.com/agentclientprotocol/agent-client-protocol/pull/2333))

## [1.24.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.24.0...schema-v1.24.1) - 2026-09-30

### Fixed

- *(unstable)* allow MCP response extensions ([#2265](https://github.com/agentclientprotocol/agent-client-protocol/pull/2265))

## [1.24.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.23.0...schema-v1.24.0) - 2026-09-30

### Added

- *(unstable)* Initial RFD and schema for subagents ([#1992](https://github.com/agentclientprotocol/agent-client-protocol/pull/1992))
- *(unstable)* make MCP-over-ACP request-scoped ([#2223](https://github.com/agentclientprotocol/agent-client-protocol/pull/2223))

## [1.23.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.22.0...schema-v1.23.0) - 2026-09-18

### Added

- *(unstable)* add v1 session notice capability ([#2171](https://github.com/agentclientprotocol/agent-client-protocol/pull/2171))

## [1.22.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.21.0...schema-v1.22.0) - 2026-09-17

### Added

- *(schema)* stabilize tool call name ([#2166](https://github.com/agentclientprotocol/agent-client-protocol/pull/2166))
- *(unstable)* add session notices schema ([#2004](https://github.com/agentclientprotocol/agent-client-protocol/pull/2004))

## [1.21.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.20.0...schema-v1.21.0) - 2026-08-20

### Added

- *(unstable)* add session compaction updates ([#2002](https://github.com/agentclientprotocol/agent-client-protocol/pull/2002))
- *(schema)* stabilize terminal authentication ([#2000](https://github.com/agentclientprotocol/agent-client-protocol/pull/2000))
- *(schema)* stabilize elicitation ([#1779](https://github.com/agentclientprotocol/agent-client-protocol/pull/1779))

### Other

- *(rfd)* Move terminal auth to preview ([#1796](https://github.com/agentclientprotocol/agent-client-protocol/pull/1796))

## [1.20.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.19.1...schema-v1.20.0) - 2026-07-21

### Added

- *(unstable)* add tool call name ([#1752](https://github.com/agentclientprotocol/agent-client-protocol/pull/1752))

## [1.19.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.19.0...schema-v1.19.1) - 2026-07-20

### Fixed

- *(schema)* remove enum discriminators from invalid schemas ([#1612](https://github.com/agentclientprotocol/agent-client-protocol/pull/1612))

## [1.19.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.18.0...schema-v1.19.0) - 2026-07-06

### Added

- *(unstable)* Add descriptions to elicitation enum options ([#1397](https://github.com/agentclientprotocol/agent-client-protocol/pull/1397))

## [1.18.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.17.0...schema-v1.18.0) - 2026-07-06

### Added

- *(schema)* Stabilize boolean session config options ([#1604](https://github.com/agentclientprotocol/agent-client-protocol/pull/1604))
- *(unstable-v2)* Unify the ID naming conventions across the schema ([#1567](https://github.com/agentclientprotocol/agent-client-protocol/pull/1567))

### Fixed

- *(schema)* Reject malformed protocol fields ([#1583](https://github.com/agentclientprotocol/agent-client-protocol/pull/1583))
- *(unstable)* remove URL elicitation error ([#1574](https://github.com/agentclientprotocol/agent-client-protocol/pull/1574))
- *(unstable-v2)* Continue to make more enums future compatible ([#1571](https://github.com/agentclientprotocol/agent-client-protocol/pull/1571))

### Other

- *(schema)* Clean up generated documentation and make wording more consistent ([#1568](https://github.com/agentclientprotocol/agent-client-protocol/pull/1568))

## [1.17.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.16.0...schema-v1.17.0) - 2026-06-29

### Added

- *(schema)* Stabilize request cancellation ([#1549](https://github.com/agentclientprotocol/agent-client-protocol/pull/1549))

### Fixed

- Deserialization leniency part 2 ([#1526](https://github.com/agentclientprotocol/agent-client-protocol/pull/1526))
- *(unstable-v2)* make mcpServers optional in new sessions ([#1523](https://github.com/agentclientprotocol/agent-client-protocol/pull/1523))

### Other

- *(unstable-v2)* Clean up client schema types ([#1540](https://github.com/agentclientprotocol/agent-client-protocol/pull/1540))
- *(schema)* correct Implementation description ([#1518](https://github.com/agentclientprotocol/agent-client-protocol/pull/1518))

## [1.16.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.15.0...schema-v1.16.0) - 2026-06-24

### Added

- *(schema)* Stabilize model_config option category ([#1502](https://github.com/agentclientprotocol/agent-client-protocol/pull/1502))

## [1.15.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.14.0...schema-v1.15.0) - 2026-06-24

### Added

- *(unstable)* Add boolean config option capabilities ([#1490](https://github.com/agentclientprotocol/agent-client-protocol/pull/1490))

## [1.14.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.13.7...schema-v1.14.0) - 2026-06-18

### Added

- Add unstable model config category ([#1455](https://github.com/agentclientprotocol/agent-client-protocol/pull/1455))

### Other

- Clean up missing documentation on various schemas and builder ([#1454](https://github.com/agentclientprotocol/agent-client-protocol/pull/1454))
- various cleanups ([#1453](https://github.com/agentclientprotocol/agent-client-protocol/pull/1453))
- Clarify schema release versioning docs ([#1443](https://github.com/agentclientprotocol/agent-client-protocol/pull/1443))

## [1.13.7](https://github.com/agentclientprotocol/agent-client-protocol/releases/tag/schema-v1.13.7) - 2026-06-16

### Fixed

- *(schema)* Add missing _meta fields to protocol schemas ([#1440](https://github.com/agentclientprotocol/agent-client-protocol/pull/1440))
- *(rust)* Preserve JSON object key order ([#1393](https://github.com/agentclientprotocol/agent-client-protocol/pull/1393))

### Other

- Fix versioning for new schema git releases ([#1382](https://github.com/agentclientprotocol/agent-client-protocol/pull/1382))
- Setup separate publishes for JSON Schemas ([#1377](https://github.com/agentclientprotocol/agent-client-protocol/pull/1377))
