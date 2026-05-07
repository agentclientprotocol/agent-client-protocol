All paths in the protocol should be absolute

## Adding new methods

- Create empty params and output structs in src/client.rs or src/agent.rs under the corresponding section. I'll add the fields myself.
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
- Add the method to src/bin/generate.rs SideDocs functions
- Run `npm run generate` and fix any issues that appear
- Run `npm run check`
- Update the example agents and clients in tests and examples in both libraries

## Schema rules

- For any nullable field, explicitly define whether it is required or optional and whether `null` is equivalent to an omitted key before running schema generation.

## Updating existing methods, their params, or output

- Update the mintlify docs and guides in the `docs` directory
- Run `npm run check` to make sure the json and zod schemas gets generated properly

Never write readme files related to the conversation unless explicitly asked to.
