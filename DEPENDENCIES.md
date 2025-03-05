# TAP-rs Dependencies Guide

This document provides important information about the project dependencies and version constraints.

## Critical Dependencies

### UUID (v0.8.2)

**⚠️ IMPORTANT: DO NOT UPGRADE BEYOND v0.8.2 ⚠️**

The TAP-rs project requires UUID version 0.8.2 specifically because:

1. The `didcomm` crate (v0.4.1) has a direct dependency on this version
2. Newer versions of UUID have breaking API changes (e.g., `Uuid::v4()` became `Uuid::new_v4()`)
3. These breaking changes affect the WASM bindings and TypeScript integration

#### Usage Guidelines:

- Always use `uuid::Uuid::new_v4()` for generating UUIDs (not `v4()`)
- All crates in the workspace should reference the UUID dependency via:
  ```toml
  uuid = { workspace = true }
  ```
- When creating new crates or scripts that generate Cargo.toml files, ensure they use UUID v0.8.2
- If you need to update `didcomm` in the future, carefully test UUID compatibility

#### History:

The project initially used newer UUID versions in some crates, which led to build failures and API inconsistencies across the codebase. In March 2025, the project standardized on UUID v0.8.2 workspace-wide to resolve these issues.

### Other Version-Sensitive Dependencies

- **didcomm** (v0.4.1): Core dependency for DIDComm message handling
- **getrandom**: Should have the "js" feature enabled for WASM support

## Updating Dependencies

When updating dependencies, please follow these guidelines:

1. Use `cargo tree -e features` to check for potential version conflicts
2. Test the build process for all target platforms, especially WASM
3. Run the full test suite to ensure compatibility
4. Document any special version requirements in this file

## WASM-Specific Considerations

For WASM compatibility, certain dependencies require special features:
- uuid: needs "wasm-bindgen" feature
- getrandom: needs "js" feature
- web-sys and js-sys: carefully manage feature flags based on browser API needs
