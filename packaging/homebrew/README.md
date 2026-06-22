# Homebrew Packaging

The Homebrew formula is rendered from the supported macOS and Linux release tarballs so the formula
uses exact published artifact SHA-256 values.

Release-owner flow:

```bash
# Run package smoke on each supported Homebrew host first:
#   aarch64-apple-darwin
#   x86_64-apple-darwin
#   x86_64-unknown-linux-gnu
#   aarch64-unknown-linux-gnu
./scripts/render-homebrew-formula.sh
```

The renderer writes:

```text
dist/homebrew/Formula/engram.rb
```

After publishing the matching GitHub release asset, copy that formula to the Homebrew tap:

```text
ymeiri/homebrew-engram/Formula/engram.rb
```

User install command after the tap is updated on macOS or Linux:

```bash
brew tap ymeiri/engram
brew install engram
```

The v0.2.0 Homebrew path supports Apple Silicon macOS, Intel macOS, Linux x64, and
Linux ARM64. Windows uses GitHub release zip assets instead of Homebrew.
