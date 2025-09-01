# Security Policy

## Supported Versions

Currently supported versions for security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :x: (yanked)       |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please follow these steps:

### 1. Do NOT Create a Public Issue

Security vulnerabilities should **never** be reported via public GitHub issues.

### 2. Report Privately

Send details to the maintainers via:
- GitHub Security Advisories (preferred)
- Direct message to repository maintainers

### 3. Include in Your Report

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Affected versions
- Possible fixes or mitigations (if known)

### 4. Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 5 business days
- **Resolution Target**: Based on severity
  - Critical: 7 days
  - High: 14 days
  - Medium: 30 days
  - Low: 90 days

## Security Best Practices

When using Icarus SDK:

### Canister Security

1. **Access Control**: Always implement proper authentication
   ```rust
   if !is_authorized(caller()) {
       return Err("Unauthorized".to_string());
   }
   ```

2. **Input Validation**: Validate all inputs
   ```rust
   if content.len() > MAX_CONTENT_SIZE {
       return Err("Content too large".to_string());
   }
   ```

3. **Resource Limits**: Implement limits to prevent DoS
   ```rust
   if memory_usage() > MAX_MEMORY {
       return Err("Memory limit exceeded".to_string());
   }
   ```

### Key Management

- Never hard-code secrets in your code
- Use environment variables for sensitive configuration
- Rotate keys regularly
- Use ICP's identity system for authentication

### Data Protection

- Encrypt sensitive data before storage
- Implement proper access controls
- Audit data access patterns
- Consider data retention policies

### Network Security

- Use HTTPS for all external communications
- Validate all external data sources
- Implement rate limiting
- Monitor for suspicious activity

## Known Security Considerations

### Internet Computer Specific

1. **Cycle Management**: Monitor cycle consumption to prevent drain attacks
2. **Upgrade Safety**: Test upgrades thoroughly to prevent data loss
3. **Inter-Canister Calls**: Validate responses from other canisters
4. **Subnet Boundaries**: Understand data visibility across subnets

### MCP Bridge Security

1. **Authentication**: The bridge requires proper authentication to canister
2. **Transport Security**: Use secure transport for MCP communications
3. **Input Sanitization**: All MCP inputs are sanitized before processing
4. **Rate Limiting**: Implement rate limiting in production

## Security Updates

Security updates are released as patch versions (e.g., 0.2.x).

To stay updated:
```bash
cargo update icarus
cargo audit
```

## Security Tools

Recommended security tools:

- **cargo-audit**: Check for known vulnerabilities
  ```bash
  cargo install cargo-audit
  cargo audit
  ```

- **cargo-deny**: Check dependencies for security issues
  ```bash
  cargo install cargo-deny
  cargo deny check
  ```

## Compliance

The Icarus SDK is licensed under BSL-1.1, which includes:
- Protection against creating competing MCP marketplaces
- Allowed use for building and deploying MCP tools
- Security patches are provided for supported versions

## Security Hall of Fame

We appreciate security researchers who help keep Icarus secure:
- (Your name could be here!)

## Contact

For urgent security matters, contact the maintainers directly via GitHub.

---

*This security policy is subject to change. Last updated: September 2025*