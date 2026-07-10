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

## Cursor Cloud specific instructions

This is a **protocol schema library** (not a runtime application). There are no long-running services to start. Development consists of editing Rust source, regenerating schemas, and validating with the CI check pipeline.

### Environment prerequisites

- **Rust**: stable + nightly toolchains (edition 2024 requires Rust >= 1.85). Nightly is needed for `rustfmt`.
- **Node.js**: latest LTS via nvm. npm is used for Prettier, Mintlify, and orchestration scripts.
- **typos-cli**: `cargo install typos-cli` — used by `npm run spellcheck`.

### Key commands (see `package.json` scripts)

| Command                       | Purpose                                                   |
| ----------------------------- | --------------------------------------------------------- |
| `npm run check`               | Full CI pipeline: clippy, format check, spellcheck, tests |
| `npm run generate`            | Regenerate JSON schemas from Rust types + format          |
| `cargo test --all-features`   | Run Rust unit + doc tests                                 |
| `cargo clippy --all-features` | Lint Rust code                                            |
| `npm run format:check`        | Verify Prettier + rustfmt formatting                      |
| `npm run format`              | Auto-fix formatting                                       |

### Gotchas

- `cargo fmt` uses **nightly** rustfmt (Cargo.toml uses edition 2024 features). If `cargo fmt` errors about unstable features, ensure `rustup component add rustfmt --toolchain nightly` has been run — `cargo fmt` automatically uses the nightly formatter when available.
- After changing Rust types in `src/`, always run `npm run generate` to regenerate schema files, then verify with `git diff --exit-code` that generated files are committed.
- The `npm run check` command is the single command to validate everything before pushing. It mirrors the CI pipeline.
