# Security Policy

## Supported Versions

We release patches for security vulnerabilities in the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| 0.x.x   | :x:                |

## Reporting a Vulnerability

We take the security of Inferno seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

### How to Report

1. **Email**: Send details to [security@inferno.ai](mailto:security@inferno.ai)
2. **Subject**: Include "SECURITY" in the subject line
3. **Details**: Include as much information as possible:
   - Type of issue (e.g., buffer overflow, SQL injection, etc.)
   - Full paths of source file(s) related to the issue
   - The location of the affected source code (tag/branch/commit or direct URL)
   - Any special configuration required to reproduce the issue
   - Step-by-step instructions to reproduce the issue
   - Proof-of-concept or exploit code (if possible)
   - Impact of the issue, including how an attacker might exploit it

### What to Expect

- **Acknowledgment**: We'll acknowledge receipt within 48 hours
- **Initial Assessment**: We'll provide an initial assessment within 5 business days
- **Regular Updates**: We'll keep you informed of our progress
- **Resolution**: We'll work to resolve critical issues within 30 days

### Safe Harbor

We support safe harbor for security researchers who:
- Make a good faith effort to avoid privacy violations and disruptions
- Only interact with accounts you own or have explicit permission to access
- Do not access, modify, or delete data belonging to others
- Contact us before making any public disclosure

## Security Best Practices

### For Users

#### Model Security
- **Verify Model Sources**: Only use models from trusted sources
- **Checksum Validation**: Always verify model checksums before loading
- **Sandboxing**: Run Inferno in containerized environments when possible
- **File Permissions**: Restrict file permissions on model directories

#### Network Security
- **Firewall**: Use firewalls to restrict access to Inferno ports
- **TLS/SSL**: Enable HTTPS for production deployments
- **Authentication**: Always enable authentication for production use
- **Rate Limiting**: Configure rate limiting to prevent abuse

#### Configuration Security
- **Secrets Management**: Store API keys and secrets securely (not in config files)
- **Principle of Least Privilege**: Run with minimal required permissions
- **Regular Updates**: Keep Inferno updated to the latest version
- **Audit Logging**: Enable comprehensive audit logging

### Configuration Example

```toml
# Security-focused configuration
[auth]
enabled = true
jwt_secret_env = "INFERNO_JWT_SECRET"  # Store in environment variable
session_timeout_hours = 2
require_https = true

[rate_limiting]
enabled = true
requests_per_minute = 100
burst_size = 20

[audit]
enabled = true
encryption = true
compression = true
log_level = "info"

[server]
bind_address = "127.0.0.1"  # Don't bind to 0.0.0.0 unless necessary
cors_origins = ["https://your-domain.com"]
max_request_size_mb = 10
```

### For Developers

#### Secure Development
- **Input Validation**: Validate all user inputs
- **Error Handling**: Don't leak sensitive information in error messages
- **Dependencies**: Regularly audit dependencies with `cargo audit`
- **Static Analysis**: Use `cargo clippy` and additional security linters
- **Memory Safety**: Leverage Rust's memory safety guarantees

#### Testing
- **Fuzzing**: Use fuzzing to test input parsing and model loading
- **Integration Tests**: Test security features in realistic scenarios
- **Dependency Testing**: Test with different versions of dependencies
- **Environment Testing**: Test in various deployment environments

## Known Security Considerations

### Model Loading
- **File Validation**: Models are validated before loading
- **Memory Limits**: Configurable limits prevent memory exhaustion
- **Sandboxing**: Model execution can be sandboxed
- **Format Validation**: Strict validation of model file formats

### API Security
- **Authentication**: JWT and API key support
- **Rate Limiting**: Configurable rate limiting per endpoint
- **Input Sanitization**: All inputs are validated and sanitized
- **CORS**: Configurable CORS policies

### Cache Security
- **Encryption**: Cache contents can be encrypted at rest
- **Access Control**: Cache access follows authentication rules
- **Cleanup**: Automatic cleanup of sensitive cache entries
- **Integrity**: Cache integrity validation

## Vulnerability Disclosure Timeline

1. **Day 0**: Vulnerability reported
2. **Day 2**: Acknowledgment sent to reporter
3. **Day 5**: Initial assessment completed
4. **Day 30**: Fix developed and tested (for critical issues)
5. **Day 35**: Security advisory published
6. **Day 37**: Fix released to public

## Security Resources

- **Security Documentation**: [Security Guide](SECURITY_GUIDE.md)
- **Deployment Security**: [Secure Deployment Guide](DEPLOYMENT_SECURITY.md)
- **API Security**: [API Security Best Practices](API_SECURITY.md)
- **Audit Logs**: [Audit Logging Guide](AUDIT_GUIDE.md)

## Contact

- **Security Team**: [security@inferno.ai](mailto:security@inferno.ai)
- **General Issues**: [GitHub Issues](https://github.com/ringo380/inferno/issues)
- **Discussion**: [GitHub Discussions](https://github.com/ringo380/inferno/discussions)

Thank you for helping keep Inferno and our users safe!