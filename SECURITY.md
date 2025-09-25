# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | :white_check_mark: |
| 0.3.x   | :white_check_mark: |
| 0.2.x   | :x:                |
| 0.1.x   | :x: (yanked)       |

## Reporting a Vulnerability

We take the security of the Icarus CDK seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Please do NOT:
- Open a public GitHub issue for security vulnerabilities
- Post about the vulnerability on social media or forums
- Exploit the vulnerability for anything other than verification

### Please DO:
- Email us at security@icarus.ai with details of the vulnerability
- Include steps to reproduce if possible
- Allow us reasonable time to respond and fix the issue before public disclosure

### What to include in your report:
- Type of vulnerability (e.g., remote code execution, injection, cross-site scripting)
- Full paths of source file(s) related to the vulnerability
- Location of the affected source code (tag/branch/commit or direct URL)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability, including how an attacker might exploit it

### What to expect:
- **Initial Response**: We will acknowledge receipt of your vulnerability report within 48 hours
- **Assessment**: We will confirm the vulnerability and determine its impact within 7 days
- **Fix Timeline**: We aim to release a fix within 30 days, depending on complexity
- **Disclosure**: We will coordinate public disclosure with you after the fix is released
- **Recognition**: We will credit you for the discovery (unless you prefer to remain anonymous)

## Security Considerations for Canister Development

When building MCP tools with Icarus CDK, consider these security best practices:

### Authentication & Authorization
- Always validate caller principals in update methods
- Implement proper access control for sensitive operations
- Use the IC's built-in Internet Identity for user authentication when appropriate

### Input Validation
- Validate all inputs from MCP clients before processing
- Implement rate limiting for resource-intensive operations
- Sanitize data before storage or processing

### Stable Storage Security
- Encrypt sensitive data before storing in stable memory
- Implement proper access controls for stored data
- Regular audit of data retention policies

### Bridge Security
- Keep the bridge CLI updated to the latest version
- Use secure communication channels between Claude Desktop and canisters
- Validate all canister responses before forwarding to MCP clients

### Dependency Management
- Regularly update dependencies to patch known vulnerabilities
- Audit third-party crates for security issues
- Use `cargo audit` to check for known vulnerabilities

## Security Updates

Security updates will be released as patch versions (e.g., 0.4.1 for 0.4.x series) and announced through:
- GitHub Security Advisories
- Release notes
- Project Discord/communication channels

Users are strongly encouraged to update to the latest patch version of their major.minor series.

## Contact

For security concerns, contact: security@icarus.ai

For general questions about security practices, please open a discussion in the GitHub repository.