# Audit Command

Run a repository audit using RepoLens.

## Steps

1. Check if the project compiles: `cargo check`
2. Run the audit: `cargo run -- plan`
3. Display results in terminal format
4. Suggest fixes for any issues found

## Usage

```bash
cargo run -- plan --format terminal
```
