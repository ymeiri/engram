# Homebrew Packaging

The beta Homebrew formula is rendered from the macOS Apple Silicon release tarball so the formula
uses the exact published artifact SHA-256.

Release-owner flow:

```bash
./scripts/package-release.sh
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

User install command after the tap is updated:

```bash
brew tap ymeiri/engram
brew install engram
```

The v0.2.0 beta Homebrew path is intentionally scoped to Apple Silicon macOS.
