use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// Database connection pool type
pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

// Database models matching our existing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbModel {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub format: String,
    pub size: i64,
    pub checksum: String,
    pub status: String, // 'available', 'loading', 'loaded', 'error'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON metadata
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbBatchJob {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub status: String, // 'pending', 'running', 'completed', 'failed', 'cancelled'
    pub progress: f64,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub failed_tasks: i32,
    pub config: String, // JSON config
    pub results: Option<String>, // JSON results
    pub schedule: Option<String>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String, // 'info', 'success', 'warning', 'error'
    pub source: String, // 'system', 'inference', 'security', 'batch', 'model'
    pub priority: String, // 'low', 'medium', 'high', 'critical'
    pub read: bool,
    pub action_data: Option<String>, // JSON action data
    pub metadata: Option<String>, // JSON metadata
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSecurityEvent {
    pub id: String,
    pub event_type: String,
    pub severity: String, // 'low', 'medium', 'high', 'critical'
    pub description: String,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub api_key_id: Option<String>,
    pub metadata: Option<String>, // JSON metadata
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub permissions: String, // JSON array of permissions
    pub is_active: bool,
    pub usage_count: i64,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbEventLog {
    pub id: String,
    pub event_type: String,
    pub event_data: String, // JSON event data
    pub timestamp: DateTime<Utc>,
}

pub struct DatabaseManager {
    pool: DbPool,
}

impl DatabaseManager {
    pub async fn new(database_path: Option<PathBuf>) -> Result<Self> {
        let db_path = database_path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("inferno");
            std::fs::create_dir_all(&path).ok();
            path.push("inferno.db");
            path
        });

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::new(manager)?;

        let database = DatabaseManager { pool };
        database.initialize_schema().await?;

        Ok(database)
    }

    pub fn get_connection(&self) -> Result<DbConnection> {
        Ok(self.pool.get()?)
    }

    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.get_connection()?;

        // Models table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                format TEXT NOT NULL,
                size INTEGER NOT NULL,
                checksum TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'available',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Batch jobs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS batch_jobs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                model_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                progress REAL NOT NULL DEFAULT 0.0,
                total_tasks INTEGER NOT NULL DEFAULT 0,
                completed_tasks INTEGER NOT NULL DEFAULT 0,
                failed_tasks INTEGER NOT NULL DEFAULT 0,
                config TEXT NOT NULL,
                results TEXT,
                schedule TEXT,
                next_run TEXT,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                FOREIGN KEY (model_id) REFERENCES models (id)
            )",
            [],
        )?;

        // Notifications table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notifications (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                notification_type TEXT NOT NULL,
                source TEXT NOT NULL,
                priority TEXT NOT NULL,
                read BOOLEAN NOT NULL DEFAULT FALSE,
                action_data TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Security events table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                source_ip TEXT,
                user_agent TEXT,
                api_key_id TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // API keys table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                key_hash TEXT NOT NULL UNIQUE,
                key_prefix TEXT NOT NULL,
                permissions TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                usage_count INTEGER NOT NULL DEFAULT 0,
                last_used TEXT,
                expires_at TEXT,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Event log table (for event history)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_log (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for better performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_models_status ON models (status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_batch_jobs_status ON batch_jobs (status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications (read)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications (created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_security_events_created_at ON security_events (created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys (is_active)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_event_log_timestamp ON event_log (timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_event_log_type ON event_log (event_type)", [])?;

        Ok(())
    }

    // Helper function to convert Row to DbModel
    fn row_to_model(row: &Row) -> rusqlite::Result<DbModel> {
        Ok(DbModel {
            id: row.get("id")?,
            name: row.get("name")?,
            file_path: row.get("file_path")?,
            format: row.get("format")?,
            size: row.get("size")?,
            checksum: row.get("checksum")?,
            status: row.get("status")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
            metadata: row.get("metadata")?,
        })
    }

    // Helper function to convert Row to DbBatchJob
    fn row_to_batch_job(row: &Row) -> rusqlite::Result<DbBatchJob> {
        Ok(DbBatchJob {
            id: row.get("id")?,
            name: row.get("name")?,
            model_id: row.get("model_id")?,
            status: row.get("status")?,
            progress: row.get("progress")?,
            total_tasks: row.get("total_tasks")?,
            completed_tasks: row.get("completed_tasks")?,
            failed_tasks: row.get("failed_tasks")?,
            config: row.get("config")?,
            results: row.get("results")?,
            schedule: row.get("schedule")?,
            next_run: match row.get::<_, Option<String>>("next_run")? {
                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc)),
                None => None,
            },
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
            started_at: match row.get::<_, Option<String>>("started_at")? {
                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc)),
                None => None,
            },
            completed_at: match row.get::<_, Option<String>>("completed_at")? {
                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc)),
                None => None,
            },
        })
    }

    // Helper function to convert Row to DbNotification
    fn row_to_notification(row: &Row) -> rusqlite::Result<DbNotification> {
        Ok(DbNotification {
            id: row.get("id")?,
            title: row.get("title")?,
            message: row.get("message")?,
            notification_type: row.get("notification_type")?,
            source: row.get("source")?,
            priority: row.get("priority")?,
            read: row.get("read")?,
            action_data: row.get("action_data")?,
            metadata: row.get("metadata")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
        })
    }

    // Helper function to convert Row to DbSecurityEvent
    fn row_to_security_event(row: &Row) -> rusqlite::Result<DbSecurityEvent> {
        Ok(DbSecurityEvent {
            id: row.get("id")?,
            event_type: row.get("event_type")?,
            severity: row.get("severity")?,
            description: row.get("description")?,
            source_ip: row.get("source_ip")?,
            user_agent: row.get("user_agent")?,
            api_key_id: row.get("api_key_id")?,
            metadata: row.get("metadata")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
        })
    }

    // Helper function to convert Row to DbApiKey
    fn row_to_api_key(row: &Row) -> rusqlite::Result<DbApiKey> {
        Ok(DbApiKey {
            id: row.get("id")?,
            name: row.get("name")?,
            key_hash: row.get("key_hash")?,
            key_prefix: row.get("key_prefix")?,
            permissions: row.get("permissions")?,
            is_active: row.get("is_active")?,
            usage_count: row.get("usage_count")?,
            last_used: match row.get::<_, Option<String>>("last_used")? {
                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc)),
                None => None,
            },
            expires_at: match row.get::<_, Option<String>>("expires_at")? {
                Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc)),
                None => None,
            },
            created_by: row.get("created_by")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
        })
    }

    // Event log methods
    pub async fn log_event(&self, event_type: &str, event_data: &str) -> Result<()> {
        let conn = self.get_connection()?;
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO event_log (id, event_type, event_data, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![id, event_type, event_data, timestamp],
        )?;

        // Keep only the last 10000 events to prevent unbounded growth
        conn.execute(
            "DELETE FROM event_log WHERE id NOT IN (
                SELECT id FROM event_log ORDER BY timestamp DESC LIMIT 10000
            )",
            [],
        )?;

        Ok(())
    }

    pub async fn get_event_history(&self, limit: Option<usize>, event_type: Option<&str>) -> Result<Vec<DbEventLog>> {
        let conn = self.get_connection()?;
        let limit = limit.unwrap_or(100);

        let mut query = "SELECT id, event_type, event_data, timestamp FROM event_log".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(event_type) = event_type {
            query.push_str(" WHERE event_type = ?");
            params.push(Box::new(event_type.to_string()));
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&query)?;
        let event_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(DbEventLog {
                id: row.get("id")?,
                event_type: row.get("event_type")?,
                event_data: row.get("event_data")?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>("timestamp")?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
            })
        })?;

        let mut events = Vec::new();
        for event in event_iter {
            events.push(event?);
        }

        Ok(events)
    }

    // Model CRUD operations
    pub async fn create_model(&self, model: &DbModel) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO models (id, name, file_path, format, size, checksum, status, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                model.id,
                model.name,
                model.file_path,
                model.format,
                model.size,
                model.checksum,
                model.status,
                model.created_at.to_rfc3339(),
                model.updated_at.to_rfc3339(),
                model.metadata
            ],
        )?;
        Ok(())
    }

    pub async fn get_models(&self) -> Result<Vec<DbModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, file_path, format, size, checksum, status, created_at, updated_at, metadata
             FROM models ORDER BY created_at DESC"
        )?;

        let model_iter = stmt.query_map([], Self::row_to_model)?;
        let mut models = Vec::new();
        for model in model_iter {
            models.push(model?);
        }
        Ok(models)
    }

    pub async fn get_model_by_id(&self, id: &str) -> Result<Option<DbModel>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, file_path, format, size, checksum, status, created_at, updated_at, metadata
             FROM models WHERE id = ?1"
        )?;

        let mut model_iter = stmt.query_map([id], Self::row_to_model)?;
        match model_iter.next() {
            Some(model) => Ok(Some(model?)),
            None => Ok(None),
        }
    }

    pub async fn update_model_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE models SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub async fn delete_model(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM models WHERE id = ?1", [id])?;
        Ok(())
    }

    // Batch job CRUD operations
    pub async fn create_batch_job(&self, job: &DbBatchJob) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO batch_jobs (id, name, model_id, status, progress, total_tasks, completed_tasks, failed_tasks, config, results, schedule, next_run, created_at, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                job.id,
                job.name,
                job.model_id,
                job.status,
                job.progress,
                job.total_tasks,
                job.completed_tasks,
                job.failed_tasks,
                job.config,
                job.results,
                job.schedule,
                job.next_run.map(|t| t.to_rfc3339()),
                job.created_at.to_rfc3339(),
                job.started_at.map(|t| t.to_rfc3339()),
                job.completed_at.map(|t| t.to_rfc3339())
            ],
        )?;
        Ok(())
    }

    pub async fn get_batch_jobs(&self) -> Result<Vec<DbBatchJob>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, model_id, status, progress, total_tasks, completed_tasks, failed_tasks, config, results, schedule, next_run, created_at, started_at, completed_at
             FROM batch_jobs ORDER BY created_at DESC"
        )?;

        let job_iter = stmt.query_map([], Self::row_to_batch_job)?;
        let mut jobs = Vec::new();
        for job in job_iter {
            jobs.push(job?);
        }
        Ok(jobs)
    }

    pub async fn get_batch_job_by_id(&self, id: &str) -> Result<Option<DbBatchJob>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, model_id, status, progress, total_tasks, completed_tasks, failed_tasks, config, results, schedule, next_run, created_at, started_at, completed_at
             FROM batch_jobs WHERE id = ?1"
        )?;

        let mut job_iter = stmt.query_map([id], Self::row_to_batch_job)?;
        match job_iter.next() {
            Some(job) => Ok(Some(job?)),
            None => Ok(None),
        }
    }

    pub async fn update_batch_job_progress(&self, id: &str, progress: f64, completed_tasks: i32, failed_tasks: i32) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE batch_jobs SET progress = ?1, completed_tasks = ?2, failed_tasks = ?3 WHERE id = ?4",
            params![progress, completed_tasks, failed_tasks, id],
        )?;
        Ok(())
    }

    pub async fn update_batch_job_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.get_connection()?;
        let now = Utc::now().to_rfc3339();

        match status {
            "running" => {
                conn.execute(
                    "UPDATE batch_jobs SET status = ?1, started_at = ?2 WHERE id = ?3",
                    params![status, now, id],
                )?;
            }
            "completed" | "failed" | "cancelled" => {
                conn.execute(
                    "UPDATE batch_jobs SET status = ?1, completed_at = ?2 WHERE id = ?3",
                    params![status, now, id],
                )?;
            }
            _ => {
                conn.execute(
                    "UPDATE batch_jobs SET status = ?1 WHERE id = ?2",
                    params![status, id],
                )?;
            }
        }
        Ok(())
    }

    pub async fn delete_batch_job(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM batch_jobs WHERE id = ?1", [id])?;
        Ok(())
    }

    pub async fn get_active_batch_job_count(&self) -> Result<i64> {
        let conn = self.get_connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM batch_jobs WHERE status IN ('pending', 'running')",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // Notification CRUD operations
    pub async fn create_notification(&self, notification: &DbNotification) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO notifications (id, title, message, notification_type, source, priority, read, action_data, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                notification.id,
                notification.title,
                notification.message,
                notification.notification_type,
                notification.source,
                notification.priority,
                notification.read,
                notification.action_data,
                notification.metadata,
                notification.created_at.to_rfc3339()
            ],
        )?;

        // Keep only the last 1000 notifications
        conn.execute(
            "DELETE FROM notifications WHERE id NOT IN (
                SELECT id FROM notifications ORDER BY created_at DESC LIMIT 1000
            )",
            [],
        )?;

        Ok(())
    }

    pub async fn get_notifications(&self, limit: Option<usize>) -> Result<Vec<DbNotification>> {
        let conn = self.get_connection()?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, title, message, notification_type, source, priority, read, action_data, metadata, created_at
             FROM notifications ORDER BY created_at DESC LIMIT ?1"
        )?;

        let notification_iter = stmt.query_map([limit], Self::row_to_notification)?;
        let mut notifications = Vec::new();
        for notification in notification_iter {
            notifications.push(notification?);
        }
        Ok(notifications)
    }

    pub async fn mark_notification_as_read(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE notifications SET read = TRUE WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    pub async fn mark_all_notifications_as_read(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("UPDATE notifications SET read = TRUE", [])?;
        Ok(())
    }

    pub async fn delete_notification(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM notifications WHERE id = ?1", [id])?;
        Ok(())
    }

    pub async fn clear_all_notifications(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM notifications", [])?;
        Ok(())
    }

    pub async fn get_unread_notification_count(&self) -> Result<i64> {
        let conn = self.get_connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notifications WHERE read = FALSE",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // Security event CRUD operations
    pub async fn create_security_event(&self, event: &DbSecurityEvent) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO security_events (id, event_type, severity, description, source_ip, user_agent, api_key_id, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.id,
                event.event_type,
                event.severity,
                event.description,
                event.source_ip,
                event.user_agent,
                event.api_key_id,
                event.metadata,
                event.created_at.to_rfc3339()
            ],
        )?;

        // Keep only the last 5000 security events
        conn.execute(
            "DELETE FROM security_events WHERE id NOT IN (
                SELECT id FROM security_events ORDER BY created_at DESC LIMIT 5000
            )",
            [],
        )?;

        Ok(())
    }

    pub async fn get_security_events(&self, limit: Option<usize>) -> Result<Vec<DbSecurityEvent>> {
        let conn = self.get_connection()?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, event_type, severity, description, source_ip, user_agent, api_key_id, metadata, created_at
             FROM security_events ORDER BY created_at DESC LIMIT ?1"
        )?;

        let event_iter = stmt.query_map([limit], Self::row_to_security_event)?;
        let mut events = Vec::new();
        for event in event_iter {
            events.push(event?);
        }
        Ok(events)
    }

    pub async fn clear_security_events(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM security_events", [])?;
        Ok(())
    }

    // API key CRUD operations
    pub async fn create_api_key(&self, api_key: &DbApiKey) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO api_keys (id, name, key_hash, key_prefix, permissions, is_active, usage_count, last_used, expires_at, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                api_key.id,
                api_key.name,
                api_key.key_hash,
                api_key.key_prefix,
                api_key.permissions,
                api_key.is_active,
                api_key.usage_count,
                api_key.last_used.map(|t| t.to_rfc3339()),
                api_key.expires_at.map(|t| t.to_rfc3339()),
                api_key.created_by,
                api_key.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub async fn get_api_keys(&self) -> Result<Vec<DbApiKey>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, key_hash, key_prefix, permissions, is_active, usage_count, last_used, expires_at, created_by, created_at
             FROM api_keys ORDER BY created_at DESC"
        )?;

        let key_iter = stmt.query_map([], Self::row_to_api_key)?;
        let mut keys = Vec::new();
        for key in key_iter {
            keys.push(key?);
        }
        Ok(keys)
    }

    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<DbApiKey>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, key_hash, key_prefix, permissions, is_active, usage_count, last_used, expires_at, created_by, created_at
             FROM api_keys WHERE key_hash = ?1 AND is_active = TRUE"
        )?;

        let mut key_iter = stmt.query_map([key_hash], Self::row_to_api_key)?;
        match key_iter.next() {
            Some(key) => Ok(Some(key?)),
            None => Ok(None),
        }
    }

    pub async fn update_api_key_usage(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE api_keys SET usage_count = usage_count + 1, last_used = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub async fn deactivate_api_key(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE api_keys SET is_active = FALSE WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    pub async fn delete_api_key(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM api_keys WHERE id = ?1", [id])?;
        Ok(())
    }

    pub async fn get_security_metrics(&self) -> Result<(i64, i64, i64, i64, i64, i64)> {
        let conn = self.get_connection()?;

        let total_api_keys: i64 = conn.query_row(
            "SELECT COUNT(*) FROM api_keys",
            [],
            |row| row.get(0),
        )?;

        let active_api_keys: i64 = conn.query_row(
            "SELECT COUNT(*) FROM api_keys WHERE is_active = TRUE",
            [],
            |row| row.get(0),
        )?;

        let expired_api_keys: i64 = conn.query_row(
            "SELECT COUNT(*) FROM api_keys WHERE expires_at IS NOT NULL AND expires_at < ?1",
            [Utc::now().to_rfc3339()],
            |row| row.get(0),
        )?;

        let security_events_24h: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_events WHERE created_at > ?1",
            [Utc::now().checked_sub_signed(chrono::Duration::hours(24)).unwrap_or(Utc::now()).to_rfc3339()],
            |row| row.get(0),
        )?;

        let failed_auth_attempts_24h: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_events WHERE event_type = 'authenticationFailed' AND created_at > ?1",
            [Utc::now().checked_sub_signed(chrono::Duration::hours(24)).unwrap_or(Utc::now()).to_rfc3339()],
            |row| row.get(0),
        )?;

        let suspicious_activities_24h: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_events WHERE event_type = 'suspiciousActivity' AND created_at > ?1",
            [Utc::now().checked_sub_signed(chrono::Duration::hours(24)).unwrap_or(Utc::now()).to_rfc3339()],
            |row| row.get(0),
        )?;

        Ok((
            total_api_keys,
            active_api_keys,
            expired_api_keys,
            security_events_24h,
            failed_auth_attempts_24h,
            suspicious_activities_24h,
        ))
    }
}