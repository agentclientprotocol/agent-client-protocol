All paths in the protocol should be absolute

## Adding new methods

- Create the params and output structs in `agent-client-protocol-schema/src/{v1,v2}/{agent,client}.rs` under the corresponding section, with the fields the RFD or request defines. Follow the schema rules below for each field.
- If the protocol method name is `noun/verb`, use `verb_noun` for the user facing methods and structs.

  Example 1 (`noun/noun`):
  Protocol method: `terminal/output`
  Trait method name: `terminal_output`
  Request/Response structs: `TerminalOutputRequest` / `TerminalOutputResponse`
  Method names struct: `terminal_output: &'static str`

  Example 2 (`noun/verb`):
  Protocol method: `terminal/new`
  Trait method name: `new_terminal`
  Request/Response structs: `NewTerminalRequest` / `NewTerminalResponse`
  Method names struct: `terminal_new: &'static str`

- Add constants for the method names
- Add variants to {Agent|Client}{Request|Response} enums
- Add the method to the SideDocs functions in `schema-generator/src/main.rs`
- In `v2`, add an `x-state` hint to every new `SessionUpdate` variant, and to any other message from the Agent that changes what a Client holds for a session or about the Agent as a whole. Add an `x-scope` hint to the params of every new Agent-to-Client method. The `state_and_scope` tests in `schema-generator/src/main.rs` describe the formats and fail when a required hint is missing.
- Run `npm run generate` and fix any issues that appear
- Run `npm run check`
- Update the example agents and clients in tests and examples in both libraries

## Schema rules

- For any nullable field, explicitly define whether it is required or optional and whether `null` is equivalent to an omitted key before running schema generation.
- Use `DefaultOnError`/`VecSkipError` only when the fallback means less (no metadata, an unsupported capability, a missing display item). If the receiver acts on the value (runs, reads, connects, or applies something), keep it strict so a request fails with `-32602` instead of silently doing something else; use `DefaultOnNull` on such lists so `null` still means empty.
- A malformed value for a known discriminator must not fall through to an untagged fallback variant.
- `DefaultOnError`, `VecSkipError`, and the `x-deserialize-*` markers they add to the schema are SDK leniency toward non-conforming Clients and Agents. They are not protocol behavior and can change. Ignore them when you write or review the spec.
- `x-state` and `x-scope` are hints for SDK generators and intermediaries. They restate what the spec pages say a message changes and what it belongs to. They add no protocol behavior. When you change one of those rules, update the hint with the prose.

## Updating existing methods, their params, or output

- Update the mintlify docs and guides in the `docs` directory
- Make the same change in both the stable page and its `draft/` copy, unless the change is draft-only
- Run `npm run check` to make sure the json and zod schemas gets generated properly

## Tests

- The Node tests in `scripts/schema-tests` are only for JSON Schema validation. They load the published files in `schema/` and assert which values a JSON Schema validator accepts and rejects. Add a case there for a rule that the published schema states, such as a hand-written `if`/`then` or `oneOf`.
- Don't test serialization, deserialization, or other Rust behavior there. Those tests go next to the type in `agent-client-protocol-schema`. Tests of how the schema is generated and annotated go in `schema-generator`.

## Writing spec prose and schema descriptions

These apply to the `docs/protocol` pages and to Rust doc comments, which become the descriptions in the generated schema.

- Say what the protocol does: name the method, field, or behavior. Skip sentences that only describe benefits ("enables rich interactions", "provides a flexible way", "seamless integration").
- Don't tack on `-ing` clauses ("…, allowing Clients to…", "…, providing visibility into…"). Write "Clients use it to…" or start a new sentence.
- Name concrete methods and capabilities instead of abstract nouns. Write "Agents that advertise `capabilities.session`" or "`auth/login` and `auth/logout`", not "the session method surface" or "the authentication surface".
- Use periods instead of semicolons or em dashes to join clauses. Use straight quotes.
- Prefer plain words: "use", not "leverage" or "utilize"; "to", not "in order to".
- Use sentence case for headings and for the titles of steps and tabs. Keep Agent, Client, and acronyms capitalized.
- On the `docs/protocol` pages and in doc comments, capitalize Agent and Client wherever the protocol role is meant. Lowercase is for other things, such as "the underlying agent" that an adapter wraps or "API client libraries".
- Use one term for one thing. Write "final status" for `completed`, `failed`, and `cancelled`, not "terminal status", which reads as the status of a terminal. On the v2 pages, write "foreground work", not "turn", which only v1 defines.
- Write error names as the Errors page lists them, in sentence case: `-32800` (Request cancelled), `-32600` (Invalid request).
- Keep shorthand such as `idle(cancelled)` to tables and diagrams. In prose, write "`idle` with `stopReason: "cancelled"`".
- Don't say what a receiver does with a malformed value ("is treated as omitted", "is skipped", "falls back to"). Say what a conforming sender sends. Rules that reject a value, such as `-32602` for malformed params, are protocol behavior and stay.
- Write a requirement with a capitalized keyword: MUST, MUST NOT, SHOULD, SHOULD NOT, or MAY. Use bold on the `docs/protocol` pages (`**MUST**`) and plain capitals in doc comments. A lowercase "must", "should", or "may" is not a requirement ([RFC 8174](https://datatracker.ietf.org/doc/html/rfc8174)). Use it only for advice, a possibility, or a description of data. Where a rule is stated in more than one place, use the same keyword in each.
- A keyword needs an implementation as its subject: an Agent, a Client, a sender, a receiver, or a transport. "The language model **MAY** respond" and "Cancellation **MAY** be done" are plain statements. Write them with "can".
- Don't write "may not". It reads as both "might not" and "is not allowed to". Write "might not" for a possibility and MUST NOT for a rule. Where a lowercase "may" grants a permission, write "can" or use the keyword.
- Bold only the keywords. Other all-caps words, such as MAJOR, are not requirements.
- A sentence that introduces a list of rules carries no keyword when the items have their own. "**MUST** follow these requirements:" turns every SHOULD below it into a MUST.
- Before you change a keyword, find every place that states the rule: the other pages, the `draft/` copy, the migration guide, the schema description, the accepted RFD, and the same page of the other protocol version. Change them together, or link to the rule instead of restating it.
- Don't describe how an SDK serializes a value ("empty lists are omitted when serializing"). Like receiver leniency, it is not protocol behavior.
- A doc comment is often repeated: on a field and its builder method, and on a type and the trait method that uses it. When you change one, search for the other copies.
- The "See protocol docs" link in a doc comment points at the page that owns the topic.
- A page under `draft/` links to other pages under `draft/`. A stable page never links to a draft page.
- In `<ParamField>` and `<ResponseField>` tags, name the element type of an array (`string[]`), as the sibling fields do.
- The section intros in `schema-generator/src/main.rs` are shared by the v1 and v2 schema pages unless they are behind a `cfg`. A change there changes both.
- Don't name the protocol version ("v1", "v2") in prose. State the rule on its own: "No Client capability is required", not "No Client capability is required in v2". The exceptions are the migration guide, which compares versions, and text about `protocolVersion` negotiation. Link targets, file paths, and examples are not prose.
- v1 is stable: rewording must not add, remove, or change a requirement. Keep every **MUST**/**SHOULD**/**MAY** and its scope as it was, and don't state behavior the v1 spec doesn't already define. Draft v2 pages can change freely.

Never write readme files related to the conversation unless explicitly asked to.

## Conventional Commits

This repository uses **Conventional Commits** for automated releases via release-plz. All commit messages should follow this format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Commit Types

- **feat:** A new feature (triggers minor version bump)
- **fix:** A bug fix (triggers patch version bump)
- **docs:** Documentation only changes
- **style:** Code style changes (formatting, missing semicolons, etc.)
- **refactor:** Code changes that neither fix bugs nor add features
- **perf:** Performance improvements
- **test:** Adding or updating tests
- **chore:** Maintenance tasks, dependency updates, etc.
- **ci:** CI/CD configuration changes
- **build:** Build system or external dependency changes

### Examples

```
feat(schema): add support for dynamic proxy chains
fix(unstable): resolve deadlock in message routing
docs(rfd): update README with installation instructions
chore: bump tokio to 1.40
```

### Scope Guidelines

Common scopes for this repository:

- `schema` - Anything that touches the actual generated schema
- `unstable` - Any changes that would only touch unstable features
- `unstable-v2` - Any changes that would only touch v2
- `rust` - Any changes that only touch Rust code and how Rust handles the data with no changes to the schema
- `rfd` - Any changes that only touch RFD documentation
