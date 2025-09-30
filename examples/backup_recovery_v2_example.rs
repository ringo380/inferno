//! Backup & Recovery Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::backup_recovery_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Backup & Recovery Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Create full backup
    println!("Example 1: Create full backup");
    let cmd = BackupCreate::new(
        config.clone(),
        "full".to_string(),
        "/backups/full-backup".to_string(),
        true,
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Create incremental backup
    println!("Example 2: Create incremental backup");
    let cmd = BackupCreate::new(
        config.clone(),
        "incremental".to_string(),
        "/backups/incremental".to_string(),
        true,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Restore from backup
    println!("Example 3: Restore from backup");
    let cmd = BackupRestore::new(
        config.clone(),
        "backup-abc123".to_string(),
        "/restore/target".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Create backup schedule
    println!("Example 4: Create backup schedule");
    let cmd = BackupSchedule::new(
        config.clone(),
        "create".to_string(),
        Some("daily-full".to_string()),
        Some("0 2 * * *".to_string()),
        Some("full".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: List schedules
    println!("Example 5: List schedules");
    let cmd = BackupSchedule::new(config.clone(), "list".to_string(), None, None, None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: List backups
    println!("Example 6: List backups");
    let cmd = BackupList::new(
        config.clone(),
        Some("full".to_string()),
        Some("7d".to_string()),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Verify backup
    println!("Example 7: Verify backup");
    let cmd = BackupVerify::new(config.clone(), "backup-abc123".to_string(), true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Test disaster recovery
    println!("Example 8: Test disaster recovery");
    let cmd = BackupTest::new(
        config.clone(),
        "full".to_string(),
        Some("backup-abc123".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Export metadata
    println!("Example 9: Export metadata");
    let cmd = BackupExport::new(
        config.clone(),
        "backups-metadata.json".to_string(),
        "json".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    println!("=== All examples completed successfully ===");
    Ok(())
}
