//! Security Command v2 Example
//!
//! Demonstrates security and access control management for authentication,
//! authorization, rate limiting, and IP access control.
//!
//! Run with: cargo run --example security_v2_example

use anyhow::Result;
use inferno::cli::security_v2::*;
use inferno::config::Config;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Security Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Show security status
    // ========================================================================
    println!("Example 1: Show Security Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let status = SecurityStatus::new(config.clone());");
    println!();
    println!("Output:");
    println!("  === Security Status ===");
    println!("  Security System: ✓ Enabled");
    println!("  ");
    println!("  Users: 5");
    println!("  Active Sessions: 3");
    println!("  Blocked IPs: 2");
    println!("  Rate Limiting: ✓ Active");

    println!("\n");

    // ========================================================================
    // Example 2: Create a user
    // ========================================================================
    println!("Example 2: Create a User");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let user = SecurityUserManage::new(");
    println!("      config.clone(),");
    println!("      \"create\".to_string(),");
    println!("      Some(\"user-123\".to_string()),");
    println!("      Some(\"john_doe\".to_string()),");
    println!("      Some(\"john@example.com\".to_string()),");
    println!("      Some(\"user\".to_string()),");
    println!("      None,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === User Management ===");
    println!("  Operation: create");
    println!("  ");
    println!("  Created user:");
    println!("    ID: user-123");
    println!("    Username: john_doe");
    println!("    Email: john@example.com");
    println!("    Role: user");

    println!("\n");

    // ========================================================================
    // Example 3: List users
    // ========================================================================
    println!("Example 3: List All Users");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = SecurityUserManage::new(");
    println!("      config.clone(),");
    println!("      \"list\".to_string(),");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === User Management ===");
    println!("  Operation: list");
    println!("  ");
    println!("  Users:");
    println!("    1. admin (admin) - active");
    println!("    2. user1 (user) - active");
    println!("    3. service-bot (service) - active");
    println!("  ");
    println!("  Total: 3 users");

    println!("\n");

    // ========================================================================
    // Example 4: Generate API key
    // ========================================================================
    println!("Example 4: Generate API Key");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let apikey = SecurityApiKey::new(");
    println!("      config.clone(),");
    println!("      \"generate\".to_string(),");
    println!("      Some(\"user-123\".to_string()),");
    println!("      Some(\"production-key\".to_string()),");
    println!("      None,");
    println!("      None,");
    println!("      Some(30),    // expires in 30 days");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === API Key Management ===");
    println!("  Operation: generate");
    println!("  ");
    println!("  Generated API key:");
    println!("    Key ID: key-abc123");
    println!("    Key: sk_live_abc123def456...");
    println!("    User: user-123");
    println!("    Name: production-key");
    println!("    Expires: in 30 days");

    println!("\n");

    // ========================================================================
    // Example 5: Generate JWT token
    // ========================================================================
    println!("Example 5: Generate JWT Token");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let token = SecurityToken::new(");
    println!("      config.clone(),");
    println!("      \"generate\".to_string(),");
    println!("      Some(\"user-123\".to_string()),");
    println!("      None,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === JWT Token Management ===");
    println!("  Operation: generate");
    println!("  ");
    println!("  Generated JWT token:");
    println!("    User: user-123");
    println!("    Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
    println!("    Expires: 1 hour");

    println!("\n");

    // ========================================================================
    // Example 6: Check rate limit status
    // ========================================================================
    println!("Example 6: Check Rate Limit Status");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let ratelimit = SecurityRateLimit::new(");
    println!("      config.clone(),");
    println!("      \"status\".to_string(),");
    println!("      Some(\"user-123\".to_string()),");
    println!("      Some(\"192.168.1.10\".parse().unwrap()),");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Rate Limit Management ===");
    println!("  Operation: status");
    println!("  ");
    println!("  Rate limit status:");
    println!("    Identifier: user-123");
    println!("    IP: 192.168.1.10");
    println!("    ");
    println!("    Per Minute: 50/60 (83%)");
    println!("    Per Hour: 500/1000 (50%)");
    println!("    Per Day: 5000/10000 (50%)");

    println!("\n");

    // ========================================================================
    // Example 7: Set custom rate limits
    // ========================================================================
    println!("Example 7: Set Custom Rate Limits");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let ratelimit = SecurityRateLimit::new(");
    println!("      config.clone(),");
    println!("      \"set\".to_string(),");
    println!("      Some(\"premium-user\".to_string()),");
    println!("      None,");
    println!("      Some(100),   // per minute");
    println!("      Some(5000),  // per hour");
    println!("      Some(50000), // per day");
    println!("      None,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Rate Limit Management ===");
    println!("  Operation: set");
    println!("  ");
    println!("  Updated rate limits:");
    println!("    Identifier: premium-user");
    println!("    Per Minute: 100");
    println!("    Per Hour: 5000");
    println!("    Per Day: 50000");

    println!("\n");

    // ========================================================================
    // Example 8: Block an IP address
    // ========================================================================
    println!("Example 8: Block an IP Address");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let ipcontrol = SecurityIpControl::new(");
    println!("      config.clone(),");
    println!("      \"block\".to_string(),");
    println!("      Some(\"203.0.113.45\".parse().unwrap()),");
    println!("      Some(\"Suspected spam activity\".to_string()),");
    println!("      false,");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === IP Access Control ===");
    println!("  Operation: block");
    println!("  ");
    println!("  Added IP to blocklist:");
    println!("    IP: 203.0.113.45");
    println!("    Reason: Suspected spam activity");

    println!("\n");

    // ========================================================================
    // Example 9: List IP access rules
    // ========================================================================
    println!("Example 9: List IP Access Rules");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let ipcontrol = SecurityIpControl::new(");
    println!("      config.clone(),");
    println!("      \"list\".to_string(),");
    println!("      None,");
    println!("      None,");
    println!("      false,");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === IP Access Control ===");
    println!("  Operation: list");
    println!("  ");
    println!("  Allowed IPs:");
    println!("    - 192.168.1.10");
    println!("    - 10.0.0.5");
    println!("  ");
    println!("  Blocked IPs:");
    println!("    - 203.0.113.45 (spam)");
    println!("    - 198.51.100.78 (abuse)");

    println!("\n");

    // ========================================================================
    // Example 10: View audit logs
    // ========================================================================
    println!("Example 10: View Audit Logs");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let audit = SecurityAudit::new(");
    println!("      config.clone(),");
    println!("      50,      // limit");
    println!("      Some(\"user-123\".to_string()),");
    println!("      None,");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Security Audit Logs ===");
    println!("  Limit: 50");
    println!("  User Filter: user-123");
    println!("  ");
    println!("  [2025-09-29T10:15:00Z] user-123 - login (success)");
    println!("  [2025-09-29T10:16:30Z] user-456 - api_key_generate (success)");
    println!("  [2025-09-29T10:17:45Z] user-789 - login (failure)");
    println!("  ");
    println!("  Total entries: 3");

    println!("\n");

    // ========================================================================
    // Example 11: Test security features
    // ========================================================================
    println!("Example 11: Test Security Features");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let test = SecurityTest::new(");
    println!("      config.clone(),");
    println!("      true,     // test auth");
    println!("      true,     // test rate limit");
    println!("      true,     // test validation");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Security Testing ===");
    println!("  ");
    println!("  Testing Authentication:");
    println!("    ✓ User authentication");
    println!("    ✓ API key validation");
    println!("    ✓ JWT token verification");
    println!("  ");
    println!("  Testing Rate Limiting:");
    println!("    ✓ Per-minute limits");
    println!("    ✓ Per-hour limits");
    println!("    ✓ Per-day limits");
    println!("  ");
    println!("  Testing Input Validation:");
    println!("    ✓ SQL injection prevention");
    println!("    ✓ XSS prevention");
    println!("    ✓ Path traversal prevention");
    println!("  ");
    println!("  All tests passed ✓");

    println!("\n");

    // ========================================================================
    // Example 12: Validation tests
    // ========================================================================
    println!("Example 12: Input Validation");
    println!("{}", "─".repeat(80));

    let invalid_operation = SecurityUserManage::new(
        config.clone(),
        "invalid".to_string(),
        None,
        None,
        None,
        None,
        None,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_operation), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid operation:");
            println!("  {}", e);
        }
    }

    println!();

    let missing_fields = SecurityUserManage::new(
        config.clone(),
        "create".to_string(),
        None,
        None,
        None,
        None,
        None,
    );

    match pipeline
        .execute(Box::new(missing_fields), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught missing required fields:");
            println!("  {}", e);
        }
    }

    println!();

    let zero_limit = SecurityAudit::new(config.clone(), 0, None, None, false);

    match pipeline
        .execute(Box::new(zero_limit), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid limit:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Security Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Security status monitoring");
    println!("✓ User management (create, list, delete, modify)");
    println!("✓ API key management");
    println!("✓ JWT token management");
    println!("✓ Rate limiting control");
    println!("✓ IP access control (allow/block lists)");
    println!("✓ Security audit logging");
    println!("✓ Security testing suite");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Operation types must be valid");
    println!("  - Required fields for each operation");
    println!("  - Audit limit: 1-1000");
    println!("  - API key expiration: 1-365 days");
    println!("  - Rate limit test requests: 1-10000");
    println!();
    println!("Use Cases:");
    println!("  - Authentication and authorization");
    println!("  - Access control management");
    println!("  - Rate limiting and throttling");
    println!("  - Security auditing and compliance");

    Ok(())
}
