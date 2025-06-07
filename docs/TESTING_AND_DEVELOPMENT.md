# Testing and Development Guide

This guide explains how to run tests, examples, and benchmarks without affecting your production `~/.tap` directory.

## Environment Variables

TAP supports several environment variables to control where data is stored:

- **`TAP_HOME`**: Sets the TAP home directory (replaces `~/.tap`)
- **`TAP_TEST_DIR`**: Sets a test directory where `.tap` will be created
- **`TAP_ROOT`**: Alternative to `TAP_HOME`, used by some components

Priority order:
1. `TAP_HOME` (highest priority)
2. `TAP_ROOT`
3. `TAP_TEST_DIR`
4. `~/.tap` (default)

## Running Tests

All tests automatically use temporary directories to protect your production `~/.tap` folder:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Running Examples

### Method 1: Using the Helper Script

We provide a helper script that automatically sets up temporary storage:

```bash
# Run the key labels demo
./run-example-with-temp.sh key_labels_demo

# Run the key management example
./run-example-with-temp.sh key_management
```

### Method 2: Manual Environment Variables

You can manually set environment variables:

```bash
# Create a temporary directory
export TAP_HOME=$(mktemp -d)

# Run the example
cargo run --example key_labels_demo

# Clean up
rm -rf $TAP_HOME
```

### Method 3: In-Code Temporary Storage

Some examples already include code to use temporary storage:

```rust
use tempfile::TempDir;
use std::env;

fn main() -> Result<()> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    env::set_var("TAP_HOME", temp_dir.path());
    
    // Your code here...
    
    Ok(())
}
```

## Running Benchmarks

Benchmarks also use temporary storage automatically:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_name
```

## Integration Tests

Integration tests in the `tests/` directory automatically use temporary storage through the environment variable system.

## Development Tips

### Using Test Utilities

For unit tests, you can use the `test_utils` module:

```rust
#[cfg(test)]
mod tests {
    use tap_agent::test_utils::{setup_test_environment, reset_test_environment};
    
    #[test]
    fn my_test() {
        setup_test_environment();
        
        // Your test code here
        
        reset_test_environment();
    }
}
```

### Creating Temporary Storage in Examples

When writing new examples, always use temporary storage:

```rust
use tempfile::TempDir;
use std::env;

fn main() -> Result<()> {
    let temp_dir = TempDir::new()?;
    env::set_var("TAP_HOME", temp_dir.path());
    
    println!("Using temporary storage at: {:?}", temp_dir.path());
    println!("(This protects your production ~/.tap directory)");
    
    // Your example code here
    
    println!("\nExample completed! Your production ~/.tap directory was not affected.");
    Ok(())
}
```

## Verifying Storage Isolation

To verify that tests and examples are not using your production `~/.tap` directory:

1. Check that `~/.tap` modification time doesn't change when running tests
2. Set `TAP_HOME` to a known directory and verify files are created there
3. Use `strace` or `dtrace` to monitor file system calls

## Troubleshooting

### Tests Still Using ~/.tap

If tests are still accessing `~/.tap`, check:

1. Environment variables are set correctly
2. No hard-coded paths in the code
3. All storage operations use the storage module's path resolution

### Permission Errors

If you get permission errors with temporary directories:

1. Ensure `/tmp` has sufficient space
2. Check that temporary directories are being created with proper permissions
3. Try setting `TMPDIR` to a different location

## Best Practices

1. **Always use temporary storage** in tests and examples
2. **Never hard-code** `~/.tap` paths
3. **Document** when production storage is intentionally used
4. **Clean up** temporary directories after use
5. **Test isolation** - each test should use its own temporary directory