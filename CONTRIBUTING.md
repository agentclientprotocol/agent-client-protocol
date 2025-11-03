# Welcome!

Thank you for your interest in contributing to Agent Client Protocol! We welcome new contributors and are glad that you want to contribute to our project. This document explains how to get involved, what we expect from contributors, and how we work together.

## Ways to contribute

We welcome many different types of contributions including:

- New features
- Builds, CI/CD
- Bug fixes
- Documentation
- Issue Triage
- Answering questions on Slack/Mailing List
- Web design
- Communications / Social Media / Blog Posts
- Release management

## Getting Started

To get started, make sure you have the following installed:

- [Rust](https://www.rust-lang.org/)
- [Node.js](https://nodejs.org/)

The schema files in the `/schema` directory are generated from the Rust code in the `rust` directory. You can always generate the latest schema files by running `npm run generate` which will also format the files for you.

Spellchecking is done via `npm run spellcheck`.

Tests are run via `cargo test`.

Unreleased features will be behind an `unstable` feature flag in the Rust crate.

If you notice a bug in the protocol, please file [an issue](https://github.com/agentclientprotocol/agent-client-protocol/issues/new?template=05_bug_report.yml) and we will be in touch.

## Coding Standards

For our Rust code, we use [rustfmt](https://github.com/rust-lang/rustfmt) and [clippy](https://github.com/rust-lang/rust-clippy) to enforce a consistent style and catch common mistakes.

For other files, like docs and schema files, we use [prettier](https://prettier.io/) to enforce a consistent style.

Our CI jobs will make sure all new changes fit these coding standards.

New features should be documented before being stabilized and released to users.

Feel free to add tests for any portion of the schema files that would benefit, we'll make sure these run in CI as well.

## RFD Process

Before a significant change or addition to the protocol is made, you should likely open an RFD (Request for Dialog) following [our RFD process](https://agentclientprotocol.com/rfds/about) to get feedback on your proposed changes before you do a lot of implementation work. This helps ensure that your changes align with the project's goals and should help avoid the frustration of doing work that may not be accepted.

## Pull Request Process

1. Ensure your branch is up to date with the main branch.
2. Open a pull request with a clear title and description.
3. At least one maintainer must review and approve the change before merging.
4. Maintainers may request changes or clarifications – this is part of the process.
5. Once approved, a maintainer will merge your pull request.

## Asking For Help

If you’re unsure about anything, feel free to reach out! The best way to reach us with a question when contributing is to ask on:

- The original GitHub issue or discussion thread
- Slack

## Code of Conduct

All contributors are expected to follow the project’s [Code of Conduct](CODE_OF_CONDUCT.md).

Please treat others with respect, foster an inclusive environment, and help keep this a welcoming project for everyone.

## Contribution Policy

This project does not require a Contributor License Agreement (CLA). Instead, contributions are accepted under the following terms:

> By contributing to this project, you agree that your contributions will be licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0). You affirm that you have the legal right to submit your work, that you are not including code you do not have rights to, and that you understand contributions are made without requiring a Contributor License Agreement (CLA).
