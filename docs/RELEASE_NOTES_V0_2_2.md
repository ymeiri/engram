# engram v0.2.2 Release Notes

v0.2.2 is a Claude Code setup hotfix for Homebrew and global installs. It
supersedes v0.2.1 for users who run `engram setup` from a home-directory install
and then open Claude Code inside a project checkout.

## Fixed

- Generated Claude Code hook settings now try project-local hooks first and then
  fall back to `$HOME/.claude/hooks`, matching the default `engram setup`
  install root.
- Re-running setup removes the legacy Engram project-only hook entries, so
  upgraded settings do not keep invoking stale missing project hook paths.

## Install Or Upgrade

Homebrew users:

```bash
brew update
brew upgrade engram
engram --version
```

Direct tarball users:

```bash
version=0.2.2
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) target="aarch64-apple-darwin" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64 | Linux-arm64) target="aarch64-unknown-linux-gnu" ;;
  *) echo "unsupported platform for published tarballs" >&2; exit 1 ;;
esac
archive="engram-${version}-${target}.tar.gz"

curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}"
curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}.sha256"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c "${archive}.sha256"
else
  sha256sum -c "${archive}.sha256"
fi
tar -xzf "${archive}"
mkdir -p "$HOME/.local/bin"
install -m 755 "engram-${version}-${target}/engram" "$HOME/.local/bin/engram"
engram --version
```

After upgrade, rerun Claude Code setup so settings are rewritten with the new
fallback hook command:

```bash
engram setup --agent claude-code --write
```

Restart Claude Code after setup.

## Scope

- Supported release assets: Apple Silicon macOS, Intel macOS, Linux x64,
  Linux ARM64, and unsigned Windows x64.
- Linux assets remain glibc builds checked against a `GLIBC_2.35` maximum
  requirement.
- Native Claude prompt-bearing proof and live `/hooks` effective-hook visibility
  keep the same conservative support scope as v0.2.1 unless a release gate
  records fresh native Claude proof.
