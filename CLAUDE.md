# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands
- Build all crates: `cargo build`
- Run all tests: `cargo test`
- Run specific test: `cargo test test_name`
- Run tests for package: `cargo test --package tap-msg`
- Run benchmarks: `cargo bench`
- Run specific benchmark: `cargo bench --bench message_benchmark`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`

## Code Quality

### Before Committing
Always run these commands before committing:
```bash
cargo fmt --all
cargo clippy --all --all-targets
cargo test --all
```

### Comments
- Keep comments concise and in the present tense
- Describe what the code does, not historical context
- Avoid comments that explain "why" something was changed or "what used to be here"
- Remove commented-out code instead of leaving it with explanations

Bad:
```rust
// Note: This was changed because the old implementation had issues
// Previously we used XOR encryption but that was insecure
```

Good:
```rust
// Encrypt using AES-KW per RFC 3394
```

### Error Handling
- Use `?` operator for error propagation
- Provide meaningful error messages
- Don't panic in library code

### Testing
- Tests should handle expected failures gracefully
- Use `#[serial]` for tests that modify shared state (environment variables, files)
- Prefer in-memory storage for test isolation

## Code Style Guidelines
- Use Rust 2021 edition features
- Follow error handling pattern with thiserror and custom Result types
- Use builder pattern for complex objects
- Implement Validation trait for validatable types
- Document public APIs with doc comments and examples
- Use async/await for asynchronous code
- Maintain WASM compatibility where appropriate
- Prefer typed structs over raw JSON for message bodies
- Use namespaced errors with detailed messages
- Follow standard Rust naming conventions (snake_case for functions, CamelCase for types)
- Do not deprecate code. Just remove it.

## Message Implementation Guidelines
- Always use @tap-msg-derive/ macros when implementing new messages
- Messages defined in @tap-msg/src/message/ may have a transaction_id. This is not meant to be serialized directly but maps to a `thid` in the parent didcomm message or the `id` if an Initiator like a Transfer, Payment or Connect.

## Workflow Guidelines
- Always run the tests in ci before finishing a task
- When starting a new feature always create a new branch using the feat/xxx convention
- Always use TDD by writing tests first
- Confirm with the user that the tests are correct and commit them before implementing them
- Once the user tells you push to github and create a new pr using the `gh` tool, but always run the same ci tools first before pushing

# Planning

Plan individual implementation using single story point tasks in TASKS.md. Always review a PRD and think hard about it before planning the tasks. Start a section in TASKS.md with the name of the PRD in a `##` section followed by a link to its file. Use TDD with tasks to write failing tests before the tasks implementing the feature. Group tasks by numbered logical phases. Each task should be outlined as a markdown checkbox and each phase using `###` as a header. Don't include time estimate.

Ask the user to review the TASKS.md file. Once user approves it commit the changes to the repository. Always review the PRD linked in the section for the current task to understand what the overall requirements and context are.

# Workflow

Always follow the @TASKS.md file to ensure proper task management and progress tracking.

When done run the linter and formatter, then mark each individual task as done with an 'x' in the checkbox,  always ask the user to review the code before comitting.

## Release Guidelines
- When doing a release first create a new release branch "release/version"
- Use semantic versioning
- Until 1.0.0, only do minor releases which can have breaking changes
- Update all required versions in the workspace to the same version
- Do a full `cargo build`
- Commit and push to GitHub, creating a new PR
- Publish each crate one by one
- When doing a release please review git history and add a comprehensive changelog to CHANGELOG.md

## Security
- IMPORTANT!! Always perform real encryption, decryption, signing and signature verification. Never use simplified placeholders. Always use well tested cryptography libraries.
