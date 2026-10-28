# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0-alpha.10](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.9...schema-v2.0.0-alpha.10) - 2026-10-27

### Added

- *(unstable-v2)* add optional terminal size to terminal_update ([#2550](https://github.com/agentclientprotocol/agent-client-protocol/pull/2550))
- *(unstable-v2)* make ResourceLink.size unsigned and define unknown tool kinds ([#2521](https://github.com/agentclientprotocol/agent-client-protocol/pull/2521))
- *(unstable-v2)* add _meta to DiffPatch, ToolCallPermissionSubject, Icon and the cancelled outcome ([#2517](https://github.com/agentclientprotocol/agent-client-protocol/pull/2517))
- *(unstable-v2)* disallow null request IDs ([#2499](https://github.com/agentclientprotocol/agent-client-protocol/pull/2499))
- *(unstable-v2)* add type discriminator to available commands ([#2494](https://github.com/agentclientprotocol/agent-client-protocol/pull/2494))
- *(unstable)* link session message views and keep identity off chunks ([#2465](https://github.com/agentclientprotocol/agent-client-protocol/pull/2465))
- *(unstable-v2)* define version mismatch errors ([#2412](https://github.com/agentclientprotocol/agent-client-protocol/pull/2412))

### Fixed

- *(unstable-v2)* parse nextCursor and listed session directories strictly ([#2567](https://github.com/agentclientprotocol/agent-client-protocol/pull/2567))
- *(unstable-v2)* don't clear replacement lists on a malformed value ([#2568](https://github.com/agentclientprotocol/agent-client-protocol/pull/2568))
- *(unstable-v2)* keep optional fields of unknown variants verbatim ([#2561](https://github.com/agentclientprotocol/agent-client-protocol/pull/2561))
- *(unstable-v2)* reject a non-object elicitation schema root type ([#2558](https://github.com/agentclientprotocol/agent-client-protocol/pull/2558))
- *(unstable)* null clears session_message participant IDs and message references ([#2555](https://github.com/agentclientprotocol/agent-client-protocol/pull/2555))
- *(unstable-v2)* make ToolCallLocation.line 1-based ([#2546](https://github.com/agentclientprotocol/agent-client-protocol/pull/2546))
- *(unstable-v2)* make terminal exitCode a signed integer ([#2545](https://github.com/agentclientprotocol/agent-client-protocol/pull/2545))
- *(schema)* require exactly one elicitation scope in v2 ([#2498](https://github.com/agentclientprotocol/agent-client-protocol/pull/2498))

### Other

- *(schema)* state null handling for every optional v2 field ([#2570](https://github.com/agentclientprotocol/agent-client-protocol/pull/2570))
- *(schema)* mark DefaultOnNull lists with an x-deserialize hint ([#2569](https://github.com/agentclientprotocol/agent-client-protocol/pull/2569))
- *(schema)* fix broken v2 doc anchors ([#2565](https://github.com/agentclientprotocol/agent-client-protocol/pull/2565))
- *(schema)* put the text-content Markdown rule on the content page ([#2562](https://github.com/agentclientprotocol/agent-client-protocol/pull/2562))
- *(schema)* use one description for every v2 _meta field ([#2560](https://github.com/agentclientprotocol/agent-client-protocol/pull/2560))
- *(schema)* define config option defaults via currentValue and state select invariants ([#2557](https://github.com/agentclientprotocol/agent-client-protocol/pull/2557))
- *(schema)* document the elicitation form-schema subset and unrenderable forms ([#2556](https://github.com/agentclientprotocol/agent-client-protocol/pull/2556))
- *(schema)* say permission tool_call subjects apply as tool_call_update upserts ([#2554](https://github.com/agentclientprotocol/agent-client-protocol/pull/2554))
- *(schema)* specify v2 diff baseline, path uniqueness and patch conflicts ([#2547](https://github.com/agentclientprotocol/agent-client-protocol/pull/2547))
- *(schema)* define tool call field defaults and null resets in v2 ([#2544](https://github.com/agentclientprotocol/agent-client-protocol/pull/2544))
- *(schema)* define switch_mode as changing the mode config option in v2 ([#2543](https://github.com/agentclientprotocol/agent-client-protocol/pull/2543))
- *(schema)* note that titled multi-select items omit type, as in MCP ([#2538](https://github.com/agentclientprotocol/agent-client-protocol/pull/2538))
- *(schema)* ExtNotification can be sent by either side ([#2534](https://github.com/agentclientprotocol/agent-client-protocol/pull/2534))
- *(schema)* include -32800's range in the reserved error codes ([#2531](https://github.com/agentclientprotocol/agent-client-protocol/pull/2531))
- *(schema)* write versioned doc links in source and drop the generator's link rewriting ([#2525](https://github.com/agentclientprotocol/agent-client-protocol/pull/2525))
- *(schema)* define which stop reason applies to v2 cancellations ([#2523](https://github.com/agentclientprotocol/agent-client-protocol/pull/2523))
- *(schema)* mark required v2 fields in the docs and rename ClientAuthCapabilities ([#2522](https://github.com/agentclientprotocol/agent-client-protocol/pull/2522))
- *(schema)* use wire names instead of Rust identifiers in schema descriptions ([#2520](https://github.com/agentclientprotocol/agent-client-protocol/pull/2520))
- *(schema)* define Client handling of unknown v2 session states and tool statuses ([#2510](https://github.com/agentclientprotocol/agent-client-protocol/pull/2510))
- *(schema)* define timeline placement for v2 entities ([#2509](https://github.com/agentclientprotocol/agent-client-protocol/pull/2509))
- *(schema)* define session/cancel as cancelling foreground work in v2 ([#2508](https://github.com/agentclientprotocol/agent-client-protocol/pull/2508))
- *(schema)* write the error pages for v1 and v2 ([#2504](https://github.com/agentclientprotocol/agent-client-protocol/pull/2504))
- *(schema)* define receiver behavior for unknown v2 variants ([#2500](https://github.com/agentclientprotocol/agent-client-protocol/pull/2500))
- *(schema)* define when session/set_config_option is available in v2 ([#2497](https://github.com/agentclientprotocol/agent-client-protocol/pull/2497))
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
