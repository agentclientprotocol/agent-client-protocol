# Changelog

## [0.9.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.8.0...v0.9.0) - 2025-12-01

### Added

- [**breaking**] Provide clearer schema for _meta properties ([#290](https://github.com/agentclientprotocol/agent-client-protocol/pull/290))

## [0.8.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.7.0...v0.8.0) - 2025-11-28

Some follow-up changes from 0.7.0. Most of the changes were in the Rust schema to make things a bit easier to work with.

However, there were some further cleanups to the JSON schema to remove some $ref indirection where possible to have the schema be a bit flatter.

There are also some fixes that were causing issues with code generators related to Extension methods, these now have concrete types in the schema as well.

**Rust**: There are some breaking changes to the `OutgoingMessage` types and other low-level RPC types to make them generate clearer JSON schema representations. Likely these are only used by SDKs, but they moved to tuple enum variants.

Also, rather than having free-floating `V0` and `V1` constants, these are now associated constants on the `ProtocolVersion` type itself.

### Fixed

- Broken doctest and test in CI ([#267](https://github.com/agentclientprotocol/agent-client-protocol/pull/267))

### Other

- Remove some nesting of the JSON schema ([#278](https://github.com/agentclientprotocol/agent-client-protocol/pull/278))
- Easier ids in constructors ([#275](https://github.com/agentclientprotocol/agent-client-protocol/pull/275))
- Exhaustive RPC types ([#272](https://github.com/agentclientprotocol/agent-client-protocol/pull/272))
- Easier `new` methods for ExtRequest + ExtNotification ([#271](https://github.com/agentclientprotocol/agent-client-protocol/pull/271))
- Protocol Version constants ([#270](https://github.com/agentclientprotocol/agent-client-protocol/pull/270))
- Cleanup Rust example from schema docs ([#269](https://github.com/agentclientprotocol/agent-client-protocol/pull/269))
- Introduce helper methods to get the corresponding method name of a ([#268](https://github.com/agentclientprotocol/agent-client-protocol/pull/268))

## 0.7.0 (2025-11-25)

This is a big release as we move towards a v1.0 release of the JSON Schema.

This should be the final form, we just want to go through the motions of upgrading all of the SDKs to verify no further changes are needed.

**NOTE: The Protocol version is already, and remains, `1`. This is just for the JSON Schema itself.** There are no breaking changes to the protocol, we just reworked the schema representation to be more compliant with code generation tooling for the various SDKs.

We also now have two variants of the schema attached to the release:

**Stable**

- schema.json
- meta.json

**Unstable**

- schema.unstable.json
- meta.unstable.json

As we have more [RFD](https://agentclientprotocol.com/rfds/about) implementations in progress, this will allow us to iterate on the schema without requiring SDKs to churn through the changes.

For SDK authors, it is important if you use the unstable version, to make sure the unstable features are behind a flag of some kind with clear direction to your users about the state of these features. But this will also allow teams to start testing the unstable features and provide feedback to the RFD authors.

### Rust

The Rust crate, `agent-client-protocol-schema` has major breaking changes. All exported type are now marked as `#[non_exhaustive]`. Since the schema itself is JSON, and we can introduce new fields and variants in a non-breaking way, we wanted to allow for the same behavior in the Rust library.

All enum variants are also tuple variants now, with their own structs. This made it nicer to represent in the JSON Schema, and also made sure we have `_meta` fields on all variants.

This upgrade will likely come with a lot of compilation errors, but ideally upgrading will be more painless in the future.

## 0.6.3 (2025-10-30)

### Protocol

- Add `discriminator` fields to the schema.json for tagged enums to aid with code generation in language tooling.

## 0.6.2 (2025-10-24)

### Protocol

Fix incorrectly named `_meta` field on `SetSessionModeResponse`

## 0.6.1 (2025-10-24)

### Protocol

- No changes

### Rust

- Make `Implementation` fields public

## 0.6.0 (2025-10-24)

### Protocol

- Add ability for agents and clients to provide information about their implementation https://github.com/agentclientprotocol/agent-client-protocol/pull/192

## 0.5.0 (2025-10-23)

### Protocol

- JSON Schema: More consistent inlining for enum representations to fix issues with code generation in language tooling.
- Provide more schema-level information about JSON-RPC format.
- Provide missing `_meta` fields on certain enum variants.

### Rust

- More consistent enum usage. Enums are always either newtype or struct variants within a single enum, not mixed.

## 0.4.11 (2025-10-20)

### Protocol

- No changes

### Rust

- Make id types easier to create and add `PartialEq` and `Eq` impls for as many types as possible.

## 0.4.10 (2025-10-16)

### Protocol

- No changes

### Rust

- Export `Result` type with a default of `acp::Error`

## 0.4.9 (2025-10-13)

- Fix schema publishing

## 0.4.8 (2025-10-13)

- Fix publishing

## 0.4.7 (2025-10-13)

### Protocol

- Schema uploaded to GitHub releases

### Rust

- SDK has moved to https://github.com/agentclientprotocol/rust-sdk
- Start publishing schema types to crates.io: https://crates.io/crates/agent-client-protocol-schema

## 0.4.6 (2025-10-10)

### Protocol

- No changes

### Rust

- Fix: support all valid JSON-RPC ids (int, string, null)

## 0.4.5 (2025-10-02)

### Protocol

- No changes

### Typescript

- **Unstable** initial support for model selection.

## 0.4.4 (2025-09-30)

### Protocol

- No changes

### Rust

- Provide default trait implementations for optional capability-based `Agent` and `Client` methods.

### Typescript

- Correctly mark capability-based `Agent` and `Client` methods as optional.

## 0.4.3 (2025-09-25)

### Protocol

- Defined `Resource not found` error type as code `-32002` (same as MCP)

### Rust

- impl `Agent` and `Client` for `Rc<T>` and `Arc<T>` where `T` implements either trait.

## 0.4.2 (2025-09-22)

### Rust

**Unstable** fix missing method for model selection in Rust library.

## 0.4.1 (2025-09-22)

### Protocol

**Unstable** initial support for model selection.

## 0.4.0 (2025-09-17)

### Protocol

No changes.

### Rust Library

- Make `Agent` and `Client` dyn compatible (you'll need to annotate them with `#[async_trait]`) [#97](https://github.com/agentclientprotocol/agent-client-protocol/pull/97)
- `ext_method` and `ext_notification` methods are now more consistent with the other trait methods [#95](https://github.com/agentclientprotocol/agent-client-protocol/pull/95)
  - There are also distinct types for `ExtRequest`, `ExtResponse`, and `ExtNotification`
- Rexport `serde_json::RawValue` for easier use [#95](https://github.com/agentclientprotocol/agent-client-protocol/pull/95)

### Typescript Library

- Use Stream abstraction instead of raw byte streams [#93](https://github.com/agentclientprotocol/agent-client-protocol/pull/93)
  - Makes it easier to use with websockets instead of stdio
- Improve type safety for method map helpers [#94](https://github.com/agentclientprotocol/agent-client-protocol/pull/94)
