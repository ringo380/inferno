# API Security Best Practices

## Authentication

### JWT Authentication

Include the JWT token in requests:

```bash
curl -H "Authorization: Bearer <your-jwt-token>" \
     https://inferno.example.com/api/v1/models
```

### API Key Authentication

Include the API key in requests:

```bash
curl -H "X-API-Key: <your-api-key>" \
     https://inferno.example.com/api/v1/models
```

### Obtaining Tokens

```bash
# Login to get JWT token
curl -X POST https://inferno.example.com/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "admin", "password": "your-password"}'

# Response
{
  "token": "eyJ...",
  "expires_at": "2024-01-02T00:00:00Z"
}
```

## API Key Management

### Creating API Keys

```bash
curl -X POST https://inferno.example.com/api/v1/auth/api-keys \
     -H "Authorization: Bearer <admin-token>" \
     -H "Content-Type: application/json" \
     -d '{
       "name": "production-key",
       "permissions": ["read_models", "run_inference"],
       "expires_in_days": 90
     }'
```

### API Key Permissions

Available permissions:
- `read_models` - List and view model information
- `write_models` - Upload and modify models
- `delete_models` - Delete models
- `run_inference` - Execute model inference
- `manage_cache` - Manage cache operations
- `read_metrics` - View system metrics
- `write_config` - Modify configuration
- `manage_users` - User management
- `view_audit_logs` - View audit logs
- `use_streaming` - Use streaming inference
- `manage_queue` - Manage job queue

### Revoking API Keys

```bash
curl -X DELETE https://inferno.example.com/api/v1/auth/api-keys/<key-id> \
     -H "Authorization: Bearer <admin-token>"
```

## Rate Limiting

### Rate Limit Headers

Responses include rate limit information:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1704067200
```

### Handling Rate Limits

When rate limited (HTTP 429), implement exponential backoff:

```python
import time
import requests

def make_request_with_retry(url, headers, max_retries=5):
    for attempt in range(max_retries):
        response = requests.get(url, headers=headers)

        if response.status_code == 429:
            retry_after = int(response.headers.get('Retry-After', 60))
            time.sleep(retry_after * (2 ** attempt))
            continue

        return response

    raise Exception("Max retries exceeded")
```

## Input Validation

### Request Size Limits

- Maximum request body: 10MB (configurable)
- Maximum prompt length: 10,000 characters (configurable)

### Content Types

Always specify the content type:

```bash
curl -X POST https://inferno.example.com/api/v1/inference \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer <token>" \
     -d '{"prompt": "Hello, world!", "model": "llama-7b"}'
```

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "AUTHENTICATION_FAILED",
    "message": "Invalid API key",
    "details": null
  }
}
```

### Common Error Codes

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 401 | AUTHENTICATION_FAILED | Invalid or missing credentials |
| 403 | PERMISSION_DENIED | Insufficient permissions |
| 429 | RATE_LIMITED | Too many requests |
| 400 | VALIDATION_ERROR | Invalid input |
| 500 | INTERNAL_ERROR | Server error |

## CORS Configuration

### Allowed Origins

Configure allowed origins in your config:

```toml
[server]
cors_origins = ["https://your-frontend.com"]
cors_methods = ["GET", "POST", "DELETE"]
cors_headers = ["Authorization", "Content-Type", "X-API-Key"]
```

## Secure Communication

### TLS Requirements

- Minimum TLS version: 1.2
- Recommended: TLS 1.3
- Always use HTTPS in production

### Certificate Validation

When making requests, always validate certificates:

```python
# Good - validates certificates
requests.get("https://inferno.example.com", verify=True)

# Bad - disables certificate validation
# requests.get("https://inferno.example.com", verify=False)
```

## Logging and Monitoring

### Request Logging

All API requests are logged with:
- Timestamp
- Client IP
- User ID (if authenticated)
- Endpoint
- Response status
- Response time

### Sensitive Data

Sensitive data is automatically redacted from logs:
- API keys
- JWT tokens
- Passwords
- Email addresses

## Best Practices Checklist

- [ ] Always use HTTPS
- [ ] Rotate API keys regularly
- [ ] Use minimum required permissions
- [ ] Implement rate limiting on client side
- [ ] Handle errors gracefully
- [ ] Log and monitor API usage
- [ ] Validate all inputs
- [ ] Keep credentials out of version control
