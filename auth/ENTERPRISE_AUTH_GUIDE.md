# Enterprise Authentication & Multi-Tenancy Guide

Complete guide for setting up OAuth2, multi-tenancy, RBAC, and API key management in Inferno v0.8.0.

## Overview

Inferno provides enterprise-grade authentication and multi-tenancy support:
- **OAuth2 Integration**: Google, GitHub, Okta, Auth0, Azure AD
- **Multi-Tenancy**: Complete isolation of data, queues, caches, and resources
- **RBAC**: Fine-grained role-based access control
- **API Keys**: Per-tenant keys with rotation and expiration
- **Audit Logging**: Track all authentication and authorization events

## Architecture

```
User Login Request
        ↓
OAuth2 Provider (Google/GitHub/Okta/etc.)
        ↓
Inferno OAuth2 Handler
        ↓
JWT Token Generation (with tenant_id, roles)
        ↓
Session Management (secure cookie)
        ↓
Request Processing
        ↓
Tenant Isolation Layer
        ├─ Data Isolation (schema/db/collection)
        ├─ Queue Isolation (separate queues)
        ├─ Cache Isolation (namespaced caches)
        └─ Logging/Metrics (tagged with tenant_id)
        ↓
Authorization Check (RBAC)
        ↓
Rate Limiting (per-tenant quota)
        ↓
Response with tenant-scoped data
```

## OAuth2 Setup

### Quick Start (Google)

```bash
# 1. Create OAuth2 credentials at https://console.cloud.google.com/
# 2. Note your Client ID and Client Secret
# 3. Create Kubernetes Secret
kubectl create secret generic inferno-oauth2-secrets \
  --from-literal=OAUTH2_GOOGLE_CLIENT_ID="your-client-id" \
  --from-literal=OAUTH2_GOOGLE_CLIENT_SECRET="your-client-secret" \
  -n inferno-prod

# 4. Update ConfigMap to enable Google provider
kubectl apply -f auth/oauth2-config.yaml

# 5. Restart Inferno pods
kubectl rollout restart deployment/inferno -n inferno-prod
```

### Supported Providers

| Provider | Endpoint | Documentation |
|----------|----------|--------------|
| **Google** | accounts.google.com | https://developers.google.com/identity/protocols/oauth2 |
| **GitHub** | github.com/login/oauth | https://docs.github.com/en/developers/apps/building-oauth-apps |
| **Okta** | developer.okta.com | https://developer.okta.com/docs/reference/api/oidc/ |
| **Auth0** | auth0.com | https://auth0.com/docs/get-started/authentication-and-authorization-flow |
| **Azure AD** | microsoft.com | https://docs.microsoft.com/en-us/azure/active-directory/develop/ |

### Provider Configuration Files

Each provider has detailed setup documentation in `oauth2-config.yaml`:
- `GOOGLE_SETUP.md` - Google Cloud Console setup
- `GITHUB_SETUP.md` - GitHub OAuth App creation
- `OKTA_SETUP.md` - Okta application configuration
- `AUTH0_SETUP.md` - Auth0 tenant setup
- `AZURE_SETUP.md` - Azure AD app registration

### Configuration Structure

```yaml
providers:
  google:
    enabled: true
    client_id: "${OAUTH2_GOOGLE_CLIENT_ID}"
    client_secret: "${OAUTH2_GOOGLE_CLIENT_SECRET}"
    redirect_uri: "https://inferno.example.com/auth/callback/google"
    user_claim_mappings:
      user_id: "sub"
      email: "email"
      name: "name"
    tenant_claim: "hd"  # Google Workspace domain
```

## Multi-Tenancy Setup

### Enable Multi-Tenancy

```bash
# 1. Apply multi-tenancy configuration
kubectl apply -f auth/multi-tenancy-config.yaml

# 2. Configure in Helm values
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set auth.multiTenancy.enabled=true \
  --set auth.oauth2.enabled=true
```

### Tenant Identification

Tenants can be identified via multiple methods (checked in order):

**1. JWT Claim** (primary)
```json
{
  "sub": "user@example.com",
  "tenant_id": "tenant_acme",  // ← Tenant ID from JWT
  "roles": ["developer"]
}
```

**2. Request Header** (fallback)
```bash
curl -H "X-Tenant-ID: tenant_acme" https://inferno.example.com/inference
```

**3. Hostname** (for SaaS)
```
tenant1.inferno.example.com → tenant_id: tenant1
tenant2.inferno.example.com → tenant_id: tenant2
```

**4. OAuth2 Domain** (for SSO)
```yaml
domain_extraction:
  enabled: true
  mappings:
    "acme.okta.com": "tenant_acme"
    "company.auth0.com": "tenant_company"
```

### Tenant Isolation Strategies

#### Data Isolation
```yaml
isolation:
  data:
    type: "schema"  # Options: schema, database, collection
    # Each tenant gets separate database schema
    # Example: tenant_acme.models, tenant_company.models
```

#### Queue Isolation
```yaml
isolation:
  queue:
    enabled: true
    namespace_prefix: "tenant_queue_"
    # Each tenant has separate request queue
    # Prevents cross-tenant interference
```

#### Cache Isolation
```yaml
isolation:
  cache:
    enabled: true
    namespace_prefix: "tenant_cache_"
    # Cache keys namespaced per tenant
    # Prevents cache poisoning attacks
```

#### Logging & Metrics Isolation
```yaml
isolation:
  logging:
    enabled: true
    include_tenant_id: true
    # Logs: {"tenant_id": "tenant_acme", "event": "inference"}

  metrics:
    enabled: true
    include_tenant_id: true
    # Metrics: inferno_inference_requests_total{tenant_id="tenant_acme"}
```

## RBAC Configuration

### Default Roles

| Role | Permissions | Use Case |
|------|-----------|----------|
| **admin** | Full access | Tenant administrators |
| **developer** | Create inferences, read models | Development teams |
| **analyst** | Read-only, audit trails | Data analysts |
| **service** | API key access (limited) | Service-to-service |
| **guest** | Health/models read-only | Unauthenticated users |

### Role Mapping

#### From JWT Claims
```json
{
  "sub": "user@example.com",
  "roles": ["developer", "analyst"],  // ← Multiple roles
  "tenant_id": "tenant_acme"
}
```

#### From Email Domain
```yaml
email_domain_mapping:
  "acme.com": "developer"
  "admin@acme.com": "admin"
  "analytics@acme.com": "analyst"
```

### Permission Scopes

| Scope | Description |
|-------|-------------|
| `inference:create` | Create inference requests |
| `inference:read` | Read own inference requests |
| `inference:read:tenant` | Read all tenant inferences |
| `models:create` | Upload models |
| `models:read` | List available models |
| `users:manage` | Manage tenant users |
| `audit:read` | View audit logs |
| `settings:update` | Modify tenant settings |

### Example: Custom Role

```yaml
roles:
  data_scientist:
    permissions:
      - "inference:create"
      - "inference:read:tenant"
      - "models:read"
      - "cache:read"
      - "metrics:read"
      # Cannot modify models, users, or settings
```

## Tenant Quotas & Rate Limiting

### Per-Tenant Quotas

```yaml
quotas:
  rate_limit:
    requests_per_second: 1000
    burst_size: 5000
    by_endpoint:
      "/inference": 500      # More restrictive
      "/batch": 100

  concurrent_requests:
    default: 100

  queue:
    max_pending: 10000
    timeout: 300

  models:
    max_loaded: 5
    max_model_size: 50

  storage:
    cache_size_limit: 100  # GB
    models_storage_limit: 500  # GB

  inference:
    max_tokens: 4096
    max_batch_size: 32
    max_concurrent: 50
```

### Quota Override (Premium Tenant)

```yaml
tenants:
  tenant_acme:
    name: "ACME Corporation"
    status: "active"
    quotas_override:
      rate_limit:
        requests_per_second: 5000  # 5x default
      concurrent_requests: 500      # 5x default
```

### Rate Limiting Example

```bash
# Default: 1000 req/s per tenant
for i in {1..1001}; do
  curl https://inferno.example.com/inference
done
# 1000 requests succeed, 1 request gets 429 Too Many Requests

# With burst: burst_size=5000
# Can accept up to 5000 requests in burst, then throttle to 1000 req/s
```

## API Key Management

### Generate API Key

```bash
# Create API key for tenant
curl -X POST https://inferno.example.com/api/keys \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CI/CD Pipeline Key",
    "expiration_days": 90,
    "scopes": ["inference:create", "models:read"]
  }'

# Response:
{
  "key_id": "key_1a2b3c4d5e6f",
  "key_secret": "sk_prod_1a2b3c4d5e6f7g8h9i0j...",  // Only shown once!
  "expires_at": "2025-01-15T00:00:00Z",
  "created_at": "2024-10-15T10:30:00Z"
}
```

### Use API Key

```bash
# Use key in request header
curl https://inferno.example.com/inference \
  -H "Authorization: Bearer sk_prod_1a2b3c4d5e6f7g8h9i0j..." \
  -H "Content-Type: application/json" \
  -d '{"model": "llama-7b", "prompt": "Hello"}'
```

### Key Rotation

```bash
# Create new key (old key still works during grace period)
curl -X POST https://inferno.example.com/api/keys \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{"name": "CI/CD Pipeline Key (Rotated)"}'

# Wait 7 days (grace period)
# Revoke old key
curl -X DELETE https://inferno.example.com/api/keys/key_1a2b3c4d5e6f \
  -H "Authorization: Bearer $JWT_TOKEN"
```

### Key Expiration & Monitoring

```bash
# List keys with expiration status
curl https://inferno.example.com/api/keys \
  -H "Authorization: Bearer $JWT_TOKEN"

# Response shows expiration warnings for keys expiring <30 days
[
  {
    "key_id": "key_1a2b3c4d5e6f",
    "name": "CI/CD Pipeline Key",
    "expires_at": "2024-11-05T00:00:00Z",
    "days_until_expiration": 21,
    "status": "expiration_warning"  // ← Will expire soon
  }
]
```

## Helm Chart Integration

### Enable Enterprise Auth

```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set auth.oauth2.enabled=true \
  --set auth.oauth2.providers.google.enabled=true \
  --set auth.multiTenancy.enabled=true \
  --set auth.rbac.enabled=true \
  --set auth.apiKeys.enabled=true
```

### Values Configuration

```yaml
auth:
  oauth2:
    enabled: true
    session:
      ttl: 3600
      cookie:
        secure: true
        http_only: true
        same_site: "Strict"
    providers:
      google:
        enabled: true
        client_id: "${OAUTH2_GOOGLE_CLIENT_ID}"
        client_secret: "${OAUTH2_GOOGLE_CLIENT_SECRET}"
      github:
        enabled: false
      okta:
        enabled: false

  multiTenancy:
    enabled: true
    tenant_identification:
      jwt_claim: "tenant_id"
      header: "X-Tenant-ID"
    isolation:
      data: "schema"
      queue: true
      cache: true
      logging: true
      metrics: true
    quotas:
      rate_limit:
        requests_per_second: 1000
      concurrent_requests: 100

  rbac:
    enabled: true
    default_role: "developer"
    role_claim: "roles"

  apiKeys:
    enabled: true
    expiration_days: 365
    rotation_interval_days: 90
```

## Security Best Practices

### 1. Token Security

```yaml
# ✅ Good: Short-lived access tokens
access_token_ttl: 3600  # 1 hour

# ✅ Good: Secure refresh token exchange
refresh_token_ttl: 2592000  # 30 days (requires re-authentication)

# ✅ Good: Token signature validation
validate_signature: true

# ✅ Good: Token expiration check
validate_expiration: true

# ❌ Bad: Long-lived tokens without refresh
access_token_ttl: 2592000  # 30 days
```

### 2. Tenant Isolation

```yaml
# ✅ Good: Strict tenant isolation
isolation:
  data: "schema"      # Separate schemas per tenant
  queue: true         # Separate queues
  cache: true         # Namespace cache keys
  logging: true       # Tag logs with tenant_id

# ❌ Bad: Row-level security (susceptible to SQL injection)
SELECT * FROM models WHERE tenant_id = ?  # Could be bypassed

# ✅ Good: Schema-level security
SELECT * FROM tenant_acme.models  # Impossible to access other tenant's data
```

### 3. Rate Limiting

```yaml
# ✅ Good: Aggressive rate limiting
rate_limit:
  requests_per_second: 1000
  burst_size: 5000    # Temporary burst allowed

# ✅ Good: Per-endpoint limits
by_endpoint:
  "/inference": 500   # More restrictive
  "/batch": 100

# ❌ Bad: No rate limiting
# Vulnerable to DoS attacks
```

### 4. API Key Security

```bash
# ✅ Good: Strong key generation
openssl rand -base64 32  # 256-bit key

# ✅ Good: Key rotation every 90 days
rotation_interval_days: 90
grace_period_days: 7     # Grace period for clients

# ✅ Good: IP whitelist (optional)
ip_whitelist:
  enabled: true
  addresses:
    - "10.0.0.0/8"

# ❌ Bad: No key expiration
# Key exposure has no time-limit impact
```

### 5. Audit Logging

```yaml
audit:
  enabled: true
  track_creation: true     # Log API key creation
  track_usage: true        # Log every API key use
  track_rotation: true     # Log key rotation
  track_revocation: true   # Log key revocation

# Logs should be:
# - Immutable (write-only)
# - Tamper-evident
# - Regularly exported
# - Analyzed for anomalies
```

## Troubleshooting

### OAuth2 Not Working

```bash
# Check provider configuration
kubectl get configmap inferno-oauth2-config -n inferno-prod -o yaml

# Verify credentials
kubectl get secret inferno-oauth2-secrets -n inferno-prod
# Should show non-empty values for your provider

# Check pod logs
kubectl logs deployment/inferno -n inferno-prod | grep oauth2

# Verify redirect URI matches provider
# Provider: https://console.cloud.google.com/
# Inferno: https://inferno.example.com/auth/callback/google
```

### Multi-Tenancy Not Isolating

```bash
# Check tenant identification
kubectl get configmap inferno-multi-tenancy-config -n inferno-prod

# Verify JWT claim or header
curl -H "X-Tenant-ID: tenant_acme" -v https://inferno.example.com/inference

# Check logs for tenant_id
kubectl logs deployment/inferno -n inferno-prod | grep tenant_id

# Verify data isolation (schema level)
# Connect to database and verify schema names
psql -U inferno
\dn  # List schemas
# Should see: tenant_acme, tenant_company, etc.
```

### Rate Limiting Too Aggressive

```bash
# Check current quotas for tenant
curl https://inferno.example.com/quotas \
  -H "Authorization: Bearer $JWT_TOKEN"

# Override quota for specific tenant
kubectl edit configmap inferno-multi-tenancy-config -n inferno-prod
# Modify: tenants.tenant_acme.quotas_override

# Restart to apply
kubectl rollout restart deployment/inferno -n inferno-prod
```

## Performance Considerations

### Overhead

| Component | CPU Impact | Memory Impact |
|-----------|-----------|---------------|
| OAuth2 validation | <2% | <5Mi |
| Multi-tenancy isolation | <1% | <2Mi |
| RBAC authorization | <1% | <2Mi |
| API key management | <1% | <2Mi |
| Total | ~4% | ~10Mi |

### Optimization Tips

1. **Cache JWT validation results** (30-60 sec)
2. **Use Redis for distributed rate limiting**
3. **Pre-compute tenant quotas**
4. **Batch audit log writes**

## Migration Guide

### From Single-Tenant to Multi-Tenant

**Step 1: Enable multi-tenancy mode**
```yaml
multi_tenancy:
  enabled: true
  default_tenant: "legacy"  # All existing data
```

**Step 2: Migrate existing data to tenant schema**
```sql
-- Move existing data to tenant schema
ALTER SCHEMA public RENAME TO tenant_legacy;
CREATE SCHEMA public;  -- Create new schema for new tenants
```

**Step 3: Add tenant_id to JWT claims**
```json
{
  "sub": "user@example.com",
  "tenant_id": "legacy",  // ← Add this
  "roles": ["admin"]
}
```

**Step 4: Enable OAuth2 for new tenants**
```yaml
oauth2:
  enabled: true
  providers:
    google:
      enabled: true
```

**Step 5: Gradually migrate to new tenants**
- Create new tenants with OAuth2
- Migrate data gradually
- Decommission legacy schema after migration

## Support

- **GitHub**: https://github.com/ringo380/inferno
- **Issues**: https://github.com/ringo380/inferno/issues
- **Docs**: https://github.com/ringo380/inferno/wiki

---

**Version**: Inferno v0.8.0
**Last Updated**: 2024-Q4
**OAuth2 Spec**: RFC 6749
**JWT Spec**: RFC 7519
