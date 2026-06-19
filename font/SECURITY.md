# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :x:                |

## Reporting a Vulnerability

We take the security of dx-font seriously. If you discover a security vulnerability, please follow these steps:

### 1. Do Not Open a Public Issue

Please do not report security vulnerabilities through public GitHub issues. This helps prevent malicious actors from exploiting the vulnerability before a fix is available.

### 2. Report Privately

Send a detailed report to the maintainers via:
- GitHub Security Advisories (preferred)
- Email to the project maintainers (see Cargo.toml for contact info)

### 3. Include Details

Your report should include:
- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)
- Your contact information for follow-up

### 4. Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: 1-7 days
  - High: 7-14 days
  - Medium: 14-30 days
  - Low: 30-90 days

## Security Considerations

### Network Security

dx-font makes HTTP requests to external font providers. Consider:

1. **Rate Limiting**: Built-in rate limiting prevents abuse
2. **Timeouts**: Configurable timeouts prevent hanging requests
3. **Retry Logic**: Exponential backoff prevents overwhelming servers
4. **TLS**: All requests use HTTPS when available

### File System Security

When downloading fonts:

1. **Path Validation**: Output paths are validated before writing
2. **File Verification**: Magic byte validation prevents malicious files
3. **Temporary Files**: Cleaned up on failure
4. **Permissions**: Respects system file permissions

### Dependency Security

We regularly audit dependencies for known vulnerabilities:

```bash
cargo audit
```

### Best Practices for Users

1. **Validate Downloads**: Always verify downloaded fonts before use
2. **Limit Permissions**: Run with minimal required permissions
3. **Network Isolation**: Consider network policies for production
4. **Cache Security**: Secure cache directories with appropriate permissions
5. **Input Validation**: Sanitize user input before passing to search queries

## Known Security Considerations

### 1. External API Dependencies

dx-font relies on external font provider APIs. We cannot guarantee:
- API availability
- Response integrity
- Provider security practices

**Mitigation**: Use caching and implement fallback strategies.

### 2. Downloaded Font Files

Fonts are binary files that could potentially contain malicious code when rendered.

**Mitigation**: 
- Magic byte verification
- Use fonts from trusted providers only
- Scan downloaded files with antivirus if needed

### 3. Cache Poisoning

Cached responses could be tampered with if cache directory is compromised.

**Mitigation**:
- Secure cache directory permissions
- Use short TTL for sensitive applications
- Implement cache validation

### 4. Denial of Service

Malicious actors could attempt to:
- Exhaust rate limits
- Fill disk with cache
- Trigger excessive retries

**Mitigation**:
- Built-in rate limiting
- Configurable cache TTL
- Maximum retry limits
- Timeout configuration

## Security Updates

Security updates will be:
1. Released as patch versions (0.2.x)
2. Documented in CHANGELOG.md
3. Announced in release notes
4. Tagged with `security` label

## Acknowledgments

We appreciate responsible disclosure and will acknowledge security researchers who report vulnerabilities (with permission).

## Questions?

For security-related questions that are not vulnerabilities, please open a regular GitHub issue with the `security` label.
