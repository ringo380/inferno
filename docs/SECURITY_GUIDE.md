# Inferno Security Guide

## Overview

This guide covers security configuration and best practices for running Inferno in production environments.

## Required Environment Variables

### Authentication Secrets

Inferno requires the following environment variables for secure operation:

```bash
# REQUIRED: JWT secret for token signing (minimum 32 characters)
export INFERNO_JWT_SECRET="your-secure-random-secret-at-least-32-chars"

# REQUIRED: Admin password for initial setup (minimum 12 characters)
export INFERNO_ADMIN_PASSWORD="your-secure-admin-password"
```

**Important**: These variables are mandatory when authentication is enabled. The application will refuse to start without them.

### Generating Secure Secrets

```bash
# Generate a secure JWT secret
openssl rand -base64 48

# Generate a secure admin password
openssl rand -base64 16
```

## Security Configuration

### Configuration File (config.toml)

```toml
[security]
# Enable authentication (recommended for production)
auth_enabled = true

# Token expiration (hours)
token_expiry_hours = 24

# Enable API key authentication
api_key_enabled = true

# Rate limiting
rate_limiting_enabled = true
max_requests_per_minute = 60
max_requests_per_hour = 1000

# Input validation
input_validation_enabled = true
max_input_length = 10000

# Output sanitization (removes sensitive data from responses)
output_sanitization_enabled = true

# TLS enforcement
tls_required = true
min_tls_version = "1.2"
```

## Authentication Methods

### JWT Tokens

Inferno uses JWT tokens for session-based authentication:

1. Authenticate with username/password to receive a JWT token
2. Include the token in the `Authorization: Bearer <token>` header
3. Tokens expire based on `token_expiry_hours` configuration

### API Keys

For programmatic access, API keys are recommended:

1. Generate an API key through the CLI or API
2. Include the key in the `X-API-Key` header
3. API keys can have specific permissions and expiration dates

## Rate Limiting

Rate limiting protects against abuse:

- **Per-minute limit**: Prevents burst attacks
- **Per-hour limit**: Prevents sustained abuse
- **Per-IP limits**: Additional protection against distributed attacks

Configure limits based on your expected usage patterns.

## Input Validation

Inferno validates all inputs to prevent injection attacks:

- Maximum input length enforcement
- Pattern detection for common attack vectors
- Path traversal prevention

## Audit Logging

Enable audit logging for security monitoring:

```toml
[security]
audit_logging_enabled = true
```

Audit logs capture:
- Authentication attempts (success/failure)
- API key creation/revocation
- User management operations
- Security-relevant configuration changes

## Network Security

### TLS Configuration

Always use TLS in production:

```toml
[server]
tls_enabled = true
tls_cert_path = "/path/to/cert.pem"
tls_key_path = "/path/to/key.pem"
```

### Firewall Rules

Restrict access to Inferno ports:

```bash
# Allow only specific IPs
iptables -A INPUT -p tcp --dport 8080 -s trusted.ip.address -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -j DROP
```

## Model Security

### Model Verification

Inferno validates models before loading:

- GGUF magic bytes verification
- File size limits
- Checksum validation (when provided)

### Sandboxing

For maximum security, run Inferno in a sandboxed environment:

```bash
# Using Docker
docker run --security-opt=no-new-privileges \
           --cap-drop=ALL \
           --read-only \
           inferno:latest
```

## Security Checklist

Before deploying to production:

- [ ] Set `INFERNO_JWT_SECRET` environment variable (32+ characters)
- [ ] Set `INFERNO_ADMIN_PASSWORD` environment variable (12+ characters)
- [ ] Enable TLS/HTTPS
- [ ] Configure rate limiting
- [ ] Enable audit logging
- [ ] Restrict network access with firewall rules
- [ ] Use a dedicated user with minimal permissions
- [ ] Regular security updates
- [ ] Monitor audit logs for suspicious activity
