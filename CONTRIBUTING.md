# Contributing to engram

Thank you for your interest in contributing to engram!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/engram.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Run lints: `cargo clippy --all-targets`
7. Format code: `cargo fmt`
8. Commit and push
9. Open a Pull Request

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Test
cargo test

# Run clippy
cargo clippy --all-targets --all-features

# Format
cargo fmt
```

## Code Style

- Follow Rust conventions
- Use `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write tests for new functionality
- Document public APIs

## Pull Request Process

1. Update documentation if needed
2. Add tests for new features
3. Ensure CI passes
4. Request review from maintainers

## Reporting Issues

- Use the issue templates
- Provide reproduction steps
- Include relevant logs/errors
