# Claude Code Guidelines for tap-rs

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
