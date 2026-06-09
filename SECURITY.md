# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** open a public issue
2. Email the maintainers directly (or use GitHub's private vulnerability reporting)
3. Include details about the vulnerability
4. Allow reasonable time for a fix before disclosure

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.0-beta.x | ✅ |
| 0.1.x | Security fixes only when practical |

## Security Considerations

engram stores data locally. Users should:
- Protect the `~/.engram` directory
- Be cautious about what data is indexed
- Review MCP tool permissions
- Pre-warm embeddings if first-run network access is not acceptable:
  `engram warmup embeddings`
