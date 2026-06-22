# engram v0.2.1 Release Notes

v0.2.1 is a Linux install compatibility hotfix. It supersedes v0.2.0 for Linux
Homebrew and direct tarball installs.

## Fixed

- Rebuilt Linux x64 and Linux ARM64 release packages in an Ubuntu 22.04
  userspace so the published binaries no longer require `GLIBC_2.38` or
  `GLIBC_2.39`.
- Added package-smoke enforcement that fails Linux release artifacts requiring
  glibc symbols newer than `GLIBC_2.35`.

## Install Or Upgrade

Homebrew users:

```bash
brew update
brew upgrade engram
engram --version
```

Direct tarball users:

```bash
version=0.2.1
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
- Linux assets are glibc builds, not musl/static binaries. v0.2.1 is checked
  against a `GLIBC_2.35` maximum requirement.
- Native Claude prompt-bearing proof and live `/hooks` effective-hook visibility
  keep the same conservative support scope as v0.2.0 unless a release gate records
  fresh native Claude proof.
- v0.2.1 does not claim broad legacy deprecation, destructive cleanup, or
  unrestricted automated lifecycle mutation.
