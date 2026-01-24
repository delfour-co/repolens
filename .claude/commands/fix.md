# Fix Command

Apply fixes for issues found during audit.

## Steps

1. Run audit to identify issues: `cargo run -- plan`
2. Review proposed actions
3. Apply fixes: `cargo run -- apply`
4. Verify fixes were applied correctly

## Usage

```bash
# Dry run first
cargo run -- apply --dry-run

# Apply fixes
cargo run -- apply
```
