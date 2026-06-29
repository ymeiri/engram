# engram v0.2.3 Release Notes

v0.2.3 is a Claude Code setup hotfix for Homebrew and global installs. It
supersedes v0.2.2 for users who expect `engram setup --agent claude-code --write`
to configure both Claude hooks and the required Claude MCP server entry.

## Fixed

- Claude Code guided setup now registers the user-scope Claude MCP server named
  `engram` automatically:

  ```bash
  claude mcp add -s user engram -- /path/to/engram serve
  ```

- If a user-scope Claude MCP server named `engram` already exists, setup replaces
  it with the current Engram binary path. This repairs stale entries that point to
  a missing or older binary.
- Homebrew-installed Engram resolves the stable Homebrew `opt` binary path when
  registering Claude MCP, so Claude Code does not keep a versioned Cellar path
  after upgrades.

## Install Or Upgrade

Homebrew users:

```bash
brew update
brew upgrade ymeiri/engram/engram
engram --version
```

After upgrade, rerun Claude Code setup:

```bash
engram setup --agent claude-code --write
```

Restart Claude Code after setup. No separate `claude mcp add` command is required
for the guided Claude Code setup path.

Direct tarball users:

```bash
version=0.2.3
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

## Scope

- Supported release assets: Apple Silicon macOS, Intel macOS, Linux x64,
  Linux ARM64, and unsigned Windows x64.
- Linux assets remain glibc builds checked against a `GLIBC_2.35` maximum
  requirement.
- Claude Code runtime enforcement still depends on Claude Code loading the
  generated hooks after restart.
