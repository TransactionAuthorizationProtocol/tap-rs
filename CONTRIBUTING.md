# Contributing to TAP-RS

Thank you for considering contributing to TAP-RS! This document outlines the process for contributing to the project and provides guidelines for documentation, code style, and testing.

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

1. Ensure the bug was not already reported by searching on GitHub under [Issues](https://github.com/notabene/tap-rs/issues).
2. If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/notabene/tap-rs/issues/new). Be sure to include a title and clear description, as much relevant information as possible, and a code sample or an executable test case demonstrating the expected behavior that is not occurring.

### Suggesting Enhancements

1. First, read the [documentation](./docs) to see if the enhancement is already supported.
2. Look at the [PRD](./prds/v1.md) to see if the enhancement is already planned.
3. [Open a new issue](https://github.com/notabene/tap-rs/issues/new) with a clear title and description of the suggested enhancement.

### Pull Requests

1. Fork the repository.
2. Create a new branch for your feature or bugfix: `git checkout -b feature/my-new-feature` or `git checkout -b fix/issue-123`.
3. Make your changes, following the [coding guidelines](#coding-guidelines).
4. Write or update tests for your changes.
5. Run the test suite to ensure all tests pass: `cargo test`.
6. Commit your changes: `git commit -am 'Add some feature'`.
7. Push to the branch: `git push origin feature/my-new-feature`.
8. Submit a pull request.

## Coding Guidelines

### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `cargo fmt` to format your code according to the standard style.
- Use `cargo clippy` to catch common mistakes and non-idiomatic code.
- Write clear and concise code with appropriate comments.

### Testing

- Write unit tests for all new features and bug fixes.
- Ensure that all tests pass before submitting a pull request.
- Consider adding integration tests for complex features.
- For WASM code, include browser-based tests when appropriate.

### Documentation Guidelines

Documentation is a critical part of the TAP-RS project. All contributions should include appropriate documentation updates.

#### API Documentation

- Document all public API items using Rust doc comments.
- Include examples in doc comments to demonstrate usage.
- Use `cargo doc` to build and review documentation locally.

#### Markdown Documentation

- Update relevant markdown documentation in the `docs` directory.
- Follow a consistent style in markdown files:
  - Use ATX-style headers (`# Header 1`, `## Header 2`, etc.).
  - Use fenced code blocks with appropriate language identifiers.
  - Use relative links to other documents in the repository.
  - Include a table of contents for longer documents.

#### Tutorial Guidelines

When writing or updating tutorials in the `docs/tutorials` directory:

1. **Clear Structure**: Start with an introduction, followed by prerequisites, then step-by-step instructions.
2. **Code Examples**: Include complete, working code examples.
3. **Explanations**: Explain why certain approaches are used, not just how to use them.
4. **Progressive Complexity**: Start simple and gradually introduce more complex concepts.
5. **Common Pitfalls**: Highlight common mistakes and how to avoid them.

#### API Reference Guidelines

When writing or updating API reference documentation in the `docs/api` directory:

1. **Complete Coverage**: Document all public types, functions, and methods.
2. **Parameter Descriptions**: Clearly describe all parameters and return values.
3. **Examples**: Include practical examples showing common usage patterns.
4. **Cross-References**: Link to related API items when appropriate.
5. **Error Handling**: Document possible errors and how to handle them.

## Development Workflow

### Setting Up Development Environment

1. Install Rust (at least version 1.71.0) and Cargo.
2. Clone the repository: `git clone https://github.com/notabene/tap-rs.git`.
3. Change into the project directory: `cd tap-rs`.
4. Build the project: `cargo build`.

### Working with WebAssembly

For WASM development:

1. Install `wasm-pack`: `cargo install wasm-pack`.
2. Build the WASM package: `cd tap-wasm && wasm-pack build --target web`.
3. For testing in Node.js: `cd tap-wasm && wasm-pack build --target nodejs`.

### Benchmarking

- Run benchmarks to measure performance: `cargo bench`.
- Compare benchmark results before and after your changes.

## Release Process

1. Update version numbers in `Cargo.toml` files according to [Semantic Versioning](https://semver.org/).
2. Update the CHANGELOG.md file with changes since the last release.
3. Create a new GitHub release with a tag matching the version number.

## License

By contributing to TAP-RS, you agree that your contributions will be licensed under the same [MIT License](LICENSE) that covers the project.
