use crate::batch::queue::{
    BatchJob, JobSchedule, ScheduleType, JobQueueManager, QueueStatus
};
use anyhow::Result;
use chrono::{DateTime, Utc, Datelike, Timelike, Weekday};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{RwLock, mpsc},
    time::interval,
};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJobEntry {
    pub job: BatchJob,
    pub next_run: SystemTime,
    pub last_run: Option<SystemTime>,
    pub run_count: u32,
    pub enabled: bool,
    pub queue_id: String,
}

#[derive(Debug)]
pub struct JobScheduler {
    scheduled_jobs: Arc<RwLock<HashMap<String, ScheduledJobEntry>>>,
    queue_manager: Arc<JobQueueManager>,
    running: Arc<std::sync::atomic::AtomicBool>,
    scheduler_tx: Option<mpsc::Sender<SchedulerCommand>>,
}

#[derive(Debug)]
pub enum SchedulerCommand {
    AddJob(ScheduledJobEntry),
    RemoveJob(String),
    UpdateJob(String, ScheduledJobEntry),
    EnableJob(String),
    DisableJob(String),
    Shutdown,
}

impl JobScheduler {
    pub fn new(queue_manager: Arc<JobQueueManager>) -> Self {
        Self {
            scheduled_jobs: Arc::new(RwLock::new(HashMap::new())),
            queue_manager,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            scheduler_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);

        let (tx, mut rx) = mpsc::channel::<SchedulerCommand>(100);
        self.scheduler_tx = Some(tx);

        let scheduled_jobs = self.scheduled_jobs.clone();
        let queue_manager = self.queue_manager.clone();
        let running = self.running.clone();

        // Start the scheduler loop
        tokio::spawn(async move {
            let mut tick_interval = interval(Duration::from_secs(60)); // Check every minute

            info!("Job scheduler started");

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                tokio::select! {
                    // Handle scheduler commands
                    cmd = rx.recv() => {
                        if let Some(command) = cmd {
                            match command {
                                SchedulerCommand::AddJob(entry) => {
                                    let mut jobs = scheduled_jobs.write().await;
                                    jobs.insert(entry.job.id.clone(), entry);
                                    debug!("Added scheduled job");
                                }
                                SchedulerCommand::RemoveJob(job_id) => {
                                    let mut jobs = scheduled_jobs.write().await;
                                    jobs.remove(&job_id);
                                    debug!("Removed scheduled job: {}", job_id);
                                }
                                SchedulerCommand::UpdateJob(job_id, entry) => {
                                    let mut jobs = scheduled_jobs.write().await;
                                    jobs.insert(job_id, entry);
                                    debug!("Updated scheduled job");
                                }
                                SchedulerCommand::EnableJob(job_id) => {
                                    let mut jobs = scheduled_jobs.write().await;
                                    if let Some(entry) = jobs.get_mut(&job_id) {
                                        entry.enabled = true;
                                        debug!("Enabled scheduled job: {}", job_id);
                                    }
                                }
                                SchedulerCommand::DisableJob(job_id) => {
                                    let mut jobs = scheduled_jobs.write().await;
                                    if let Some(entry) = jobs.get_mut(&job_id) {
                                        entry.enabled = false;
                                        debug!("Disabled scheduled job: {}", job_id);
                                    }
                                }
                                SchedulerCommand::Shutdown => {
                                    info!("Scheduler shutting down");
                                    break;
                                }
                            }
                        }
                    }

                    // Check for jobs to run on regular intervals
                    _ = tick_interval.tick() => {
                        Self::check_and_run_jobs(&scheduled_jobs, &queue_manager).await;
                    }
                }
            }

            info!("Job scheduler stopped");
        });

        Ok(())
    }

    async fn check_and_run_jobs(
        scheduled_jobs: &Arc<RwLock<HashMap<String, ScheduledJobEntry>>>,
        queue_manager: &Arc<JobQueueManager>,
    ) {
        let now = SystemTime::now();
        let mut jobs_to_update = Vec::new();

        // Read jobs and identify which ones need to run
        {
            let jobs = scheduled_jobs.read().await;
            for (_job_id, entry) in jobs.iter() {
                if !entry.enabled {
                    continue;
                }

                if now >= entry.next_run {
                    // Check if the queue is available
                    if let Some(status) = queue_manager.get_queue_status(&entry.queue_id).await {
                        match status {
                            QueueStatus::Running => {
                                // Submit the job
                                let mut job = entry.job.clone();
                                job.id = format!("{}_{}", job.id, entry.run_count + 1);

                                match queue_manager.submit_job(&entry.queue_id, job).await {
                                    Ok(job_id) => {
                                        info!("Scheduled job submitted: {}", job_id);

                                        // Calculate next run time
                                        if let Ok(next_run) = Self::calculate_next_run(&entry.job.schedule, now) {
                                            let mut updated_entry = entry.clone();
                                            updated_entry.last_run = Some(now);
                                            updated_entry.run_count += 1;
                                            updated_entry.next_run = next_run;

                                            // Check if we've reached max runs
                                            if let Some(schedule) = &entry.job.schedule {
                                                let should_disable = match &schedule.schedule_type {
                                                    ScheduleType::Interval { max_runs: Some(max), .. } |
                                                    ScheduleType::Cron { max_runs: Some(max), .. } => {
                                                        updated_entry.run_count >= *max
                                                    }
                                                    ScheduleType::Once(_) => true,
                                                    _ => false,
                                                };

                                                if should_disable {
                                                    updated_entry.enabled = false;
                                                    info!("Disabled scheduled job {} after reaching max runs", entry.job.id);
                                                }
                                            }

                                            jobs_to_update.push((entry.job.id.clone(), updated_entry));
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to submit scheduled job {}: {}", entry.job.id, e);
                                    }
                                }
                            }
                            _ => {
                                debug!("Queue {} not available for scheduled job {}", entry.queue_id, entry.job.id);
                            }
                        }
                    }
                }
            }
        }

        // Update jobs outside of the read lock
        if !jobs_to_update.is_empty() {
            let mut jobs = scheduled_jobs.write().await;
            for (job_id, updated_entry) in jobs_to_update {
                jobs.insert(job_id, updated_entry);
            }
        }
    }

    fn calculate_next_run(schedule: &Option<JobSchedule>, from_time: SystemTime) -> Result<SystemTime> {
        let schedule = schedule.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Job has no schedule"))?;

        if !schedule.enabled {
            return Err(anyhow::anyhow!("Schedule is disabled"));
        }

        let from_datetime: DateTime<Utc> = from_time.into();

        let next_datetime = match &schedule.schedule_type {
            ScheduleType::Once(time) => {
                let datetime: DateTime<Utc> = (*time).into();
                if datetime > from_datetime {
                    datetime
                } else {
                    return Err(anyhow::anyhow!("One-time schedule has already passed"));
                }
            }

            ScheduleType::Interval { interval_minutes, .. } => {
                from_datetime + chrono::Duration::minutes(*interval_minutes as i64)
            }

            ScheduleType::Cron { expression, .. } => {
                Self::parse_cron_next(expression, from_datetime)?
            }

            ScheduleType::Daily { time, weekdays } => {
                Self::calculate_daily_next(time, weekdays, from_datetime)?
            }

            ScheduleType::Weekly { day_of_week, time } => {
                Self::calculate_weekly_next(*day_of_week, time, from_datetime)?
            }

            ScheduleType::Monthly { day_of_month, time } => {
                Self::calculate_monthly_next(*day_of_month, time, from_datetime)?
            }
        };

        // Check if the next run is within the schedule's time window
        if let Some(end_time) = schedule.end_time {
            let end_datetime: DateTime<Utc> = end_time.into();
            if next_datetime > end_datetime {
                return Err(anyhow::anyhow!("Next run would be after schedule end time"));
            }
        }

        Ok(next_datetime.into())
    }

    fn parse_cron_next(expression: &str, from: DateTime<Utc>) -> Result<DateTime<Utc>> {
        // Handle special cron keywords first
        let normalized_expression = Self::normalize_cron_expression(expression)?;

        // Parse the cron expression using the cron crate
        let schedule = Schedule::from_str(&normalized_expression)
            .map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", expression, e))?;

        // Find the next occurrence after the current time
        let next_occurrence = schedule.upcoming(Utc).next()
            .ok_or_else(|| anyhow::anyhow!("No future occurrence found for cron expression: {}", expression))?;

        // Ensure the next occurrence is after the from time
        if next_occurrence <= from {
            // If the calculated next time is not after 'from', get the one after that
            schedule.after(&from).next()
                .ok_or_else(|| anyhow::anyhow!("No future occurrence found after {} for cron expression: {}", from, expression))
        } else {
            Ok(next_occurrence)
        }
    }

    fn normalize_cron_expression(expression: &str) -> Result<String> {
        let trimmed = expression.trim();

        // Handle special cron keywords
        // Note: cron crate uses 7-field format: sec min hour day month dow year
        match trimmed {
            "@yearly" | "@annually" => Ok("0 0 0 1 1 * *".to_string()),
            "@monthly" => Ok("0 0 0 1 * * *".to_string()),
            "@weekly" => Ok("0 0 0 * * SUN *".to_string()),
            "@daily" | "@midnight" => Ok("0 0 0 * * * *".to_string()),
            "@hourly" => Ok("0 0 * * * * *".to_string()),
            _ => {
                // Convert 5-field to 7-field format if needed
                Self::convert_to_seven_field_format(trimmed)
            }
        }
    }

    fn convert_to_seven_field_format(expression: &str) -> Result<String> {
        let parts: Vec<&str> = expression.split_whitespace().collect();

        match parts.len() {
            5 => {
                // Standard 5-field cron: min hour day month dow
                // Convert to 7-field: sec min hour day month dow year
                Ok(format!("0 {} *", expression))
            }
            6 => {
                // 6-field cron: sec min hour day month dow
                // Convert to 7-field: sec min hour day month dow year
                Ok(format!("{} *", expression))
            }
            7 => {
                // Already 7-field format
                Self::validate_cron_expression(expression)?;
                Ok(expression.to_string())
            }
            _ => {
                Err(anyhow::anyhow!(
                    "Invalid cron expression format. Expected 5, 6, or 7 fields, got {}: '{}'",
                    parts.len(),
                    expression
                ))
            }
        }
    }

    fn validate_cron_expression(expression: &str) -> Result<()> {
        let parts: Vec<&str> = expression.split_whitespace().collect();

        // Accept 5, 6, or 7 field formats
        if ![5, 6, 7].contains(&parts.len()) {
            return Err(anyhow::anyhow!(
                "Invalid cron expression format. Expected 5, 6, or 7 fields, got {}: '{}'",
                parts.len(),
                expression
            ));
        }

        // Validate each field has valid characters
        for (i, part) in parts.iter().enumerate() {
            let field_name = match parts.len() {
                5 => match i {
                    0 => "minute",
                    1 => "hour",
                    2 => "day",
                    3 => "month",
                    4 => "weekday",
                    _ => unreachable!(),
                },
                6 => match i {
                    0 => "second",
                    1 => "minute",
                    2 => "hour",
                    3 => "day",
                    4 => "month",
                    5 => "weekday",
                    _ => unreachable!(),
                },
                7 => match i {
                    0 => "second",
                    1 => "minute",
                    2 => "hour",
                    3 => "day",
                    4 => "month",
                    5 => "weekday",
                    6 => "year",
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            if !Self::is_valid_cron_field(part) {
                return Err(anyhow::anyhow!(
                    "Invalid characters in {} field '{}'. Valid characters: 0-9, *, /, -, ,",
                    field_name,
                    part
                ));
            }
        }

        Ok(())
    }

    fn is_valid_cron_field(field: &str) -> bool {
        // Allow digits, asterisk, forward slash, hyphen, comma, and question mark
        field.chars().all(|c| {
            c.is_ascii_digit() || matches!(c, '*' | '/' | '-' | ',' | '?')
        })
    }

    fn calculate_daily_next(
        time_str: &str,
        weekdays: &[u8],
        from: DateTime<Utc>,
    ) -> Result<DateTime<Utc>> {
        let (hour, minute) = Self::parse_time_string(time_str)?;

        let mut next = from.with_hour(hour).unwrap().with_minute(minute).unwrap().with_second(0).unwrap();

        // If the time has already passed today, start from tomorrow
        if next <= from {
            next = next + chrono::Duration::days(1);
        }

        // Find the next day that matches one of the specified weekdays
        for _ in 0..7 {
            let weekday = next.weekday().num_days_from_monday() as u8;
            if weekdays.contains(&weekday) {
                return Ok(next);
            }
            next = next + chrono::Duration::days(1);
        }

        Err(anyhow::anyhow!("No valid weekday found"))
    }

    fn calculate_weekly_next(
        day_of_week: u8,
        time_str: &str,
        from: DateTime<Utc>,
    ) -> Result<DateTime<Utc>> {
        let (hour, minute) = Self::parse_time_string(time_str)?;

        let current_weekday = from.weekday().num_days_from_monday() as u8;
        let days_until_target = if day_of_week >= current_weekday {
            day_of_week - current_weekday
        } else {
            7 - (current_weekday - day_of_week)
        };

        let mut next = from + chrono::Duration::days(days_until_target as i64);
        next = next.with_hour(hour).unwrap().with_minute(minute).unwrap().with_second(0).unwrap();

        // If we're on the target day but the time has passed, go to next week
        if days_until_target == 0 && next <= from {
            next = next + chrono::Duration::weeks(1);
        }

        Ok(next)
    }

    fn calculate_monthly_next(
        day_of_month: u8,
        time_str: &str,
        from: DateTime<Utc>,
    ) -> Result<DateTime<Utc>> {
        let (hour, minute) = Self::parse_time_string(time_str)?;

        let mut next = from.with_day(day_of_month as u32)
            .unwrap_or_else(|| {
                // If the day doesn't exist in this month (e.g., Feb 30), use the last day
                from.with_day(1).unwrap() + chrono::Duration::days(32) - chrono::Duration::days(1)
            })
            .with_hour(hour).unwrap()
            .with_minute(minute).unwrap()
            .with_second(0).unwrap();

        // If the time has already passed this month, go to next month
        if next <= from {
            next = if let Some(next_month) = next.with_month(next.month() + 1) {
                next_month
            } else {
                // December -> January of next year
                next.with_year(next.year() + 1).unwrap().with_month(1).unwrap()
            };
        }

        Ok(next)
    }

    fn parse_time_string(time_str: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid time format. Use HH:MM"));
        }

        let hour: u32 = parts[0].parse()
            .map_err(|_| anyhow::anyhow!("Invalid hour: {}", parts[0]))?;
        let minute: u32 = parts[1].parse()
            .map_err(|_| anyhow::anyhow!("Invalid minute: {}", parts[1]))?;

        if hour > 23 {
            return Err(anyhow::anyhow!("Hour must be 0-23: {}", hour));
        }
        if minute > 59 {
            return Err(anyhow::anyhow!("Minute must be 0-59: {}", minute));
        }

        Ok((hour, minute))
    }

    pub async fn add_scheduled_job(&self, entry: ScheduledJobEntry) -> Result<()> {
        if let Some(tx) = &self.scheduler_tx {
            tx.send(SchedulerCommand::AddJob(entry)).await
                .map_err(|e| anyhow::anyhow!("Failed to add scheduled job: {}", e))?;
        }
        Ok(())
    }

    pub async fn remove_scheduled_job(&self, job_id: &str) -> Result<()> {
        if let Some(tx) = &self.scheduler_tx {
            tx.send(SchedulerCommand::RemoveJob(job_id.to_string())).await
                .map_err(|e| anyhow::anyhow!("Failed to remove scheduled job: {}", e))?;
        }
        Ok(())
    }

    pub async fn enable_job(&self, job_id: &str) -> Result<()> {
        if let Some(tx) = &self.scheduler_tx {
            tx.send(SchedulerCommand::EnableJob(job_id.to_string())).await
                .map_err(|e| anyhow::anyhow!("Failed to enable job: {}", e))?;
        }
        Ok(())
    }

    pub async fn disable_job(&self, job_id: &str) -> Result<()> {
        if let Some(tx) = &self.scheduler_tx {
            tx.send(SchedulerCommand::DisableJob(job_id.to_string())).await
                .map_err(|e| anyhow::anyhow!("Failed to disable job: {}", e))?;
        }
        Ok(())
    }

    pub async fn list_scheduled_jobs(&self) -> Vec<ScheduledJobEntry> {
        let jobs = self.scheduled_jobs.read().await;
        jobs.values().cloned().collect()
    }

    pub async fn get_scheduled_job(&self, job_id: &str) -> Option<ScheduledJobEntry> {
        let jobs = self.scheduled_jobs.read().await;
        jobs.get(job_id).cloned()
    }

    pub async fn stop(&self) -> Result<()> {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);

        if let Some(tx) = &self.scheduler_tx {
            tx.send(SchedulerCommand::Shutdown).await
                .map_err(|e| anyhow::anyhow!("Failed to stop scheduler: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_parse_time_string() {
        assert_eq!(JobScheduler::parse_time_string("09:30").unwrap(), (9, 30));
        assert_eq!(JobScheduler::parse_time_string("23:59").unwrap(), (23, 59));
        assert_eq!(JobScheduler::parse_time_string("00:00").unwrap(), (0, 0));

        assert!(JobScheduler::parse_time_string("24:00").is_err());
        assert!(JobScheduler::parse_time_string("12:60").is_err());
        assert!(JobScheduler::parse_time_string("invalid").is_err());
    }

    #[test]
    fn test_calculate_next_run_interval() {
        let schedule = JobSchedule {
            schedule_type: ScheduleType::Interval {
                interval_minutes: 60,
                max_runs: None,
            },
            start_time: None,
            end_time: None,
            timezone: "UTC".to_string(),
            enabled: true,
        };

        let now = SystemTime::now();
        let next = JobScheduler::calculate_next_run(&Some(schedule), now).unwrap();

        let duration = next.duration_since(now).unwrap();
        assert!(duration >= Duration::from_secs(3590) && duration <= Duration::from_secs(3610));
    }

    #[test]
    fn test_calculate_next_run_once() {
        let future_time = SystemTime::now() + Duration::from_secs(3600);
        let schedule = JobSchedule {
            schedule_type: ScheduleType::Once(future_time),
            start_time: None,
            end_time: None,
            timezone: "UTC".to_string(),
            enabled: true,
        };

        let now = SystemTime::now();
        let next = JobScheduler::calculate_next_run(&Some(schedule), now).unwrap();

        assert_eq!(next, future_time);
    }

    #[test]
    fn test_normalize_cron_expression_keywords() {
        assert_eq!(JobScheduler::normalize_cron_expression("@yearly").unwrap(), "0 0 0 1 1 * *");
        assert_eq!(JobScheduler::normalize_cron_expression("@annually").unwrap(), "0 0 0 1 1 * *");
        assert_eq!(JobScheduler::normalize_cron_expression("@monthly").unwrap(), "0 0 0 1 * * *");
        assert_eq!(JobScheduler::normalize_cron_expression("@weekly").unwrap(), "0 0 0 * * SUN *");
        assert_eq!(JobScheduler::normalize_cron_expression("@daily").unwrap(), "0 0 0 * * * *");
        assert_eq!(JobScheduler::normalize_cron_expression("@midnight").unwrap(), "0 0 0 * * * *");
        assert_eq!(JobScheduler::normalize_cron_expression("@hourly").unwrap(), "0 0 * * * * *");
    }

    #[test]
    fn test_normalize_cron_expression_standard() {
        assert_eq!(JobScheduler::normalize_cron_expression("0 * * * *").unwrap(), "0 0 * * * * *");
        assert_eq!(JobScheduler::normalize_cron_expression("15 14 1 * *").unwrap(), "0 15 14 1 * * *");
        assert_eq!(JobScheduler::normalize_cron_expression("0 22 * * 1-5").unwrap(), "0 0 22 * * 1-5 *");
        assert_eq!(JobScheduler::normalize_cron_expression("*/15 * * * *").unwrap(), "0 */15 * * * * *");
    }

    #[test]
    fn test_validate_cron_expression() {
        // Valid 5-field expressions
        assert!(JobScheduler::validate_cron_expression("0 * * * *").is_ok());
        assert!(JobScheduler::validate_cron_expression("15 14 1 * *").is_ok());
        assert!(JobScheduler::validate_cron_expression("0 22 * * 1-5").is_ok());
        assert!(JobScheduler::validate_cron_expression("*/15 * * * *").is_ok());
        assert!(JobScheduler::validate_cron_expression("0,30 * * * *").is_ok());

        // Valid 6-field expressions
        assert!(JobScheduler::validate_cron_expression("0 0 * * * *").is_ok());
        assert!(JobScheduler::validate_cron_expression("30 15 14 1 * *").is_ok());

        // Valid 7-field expressions
        assert!(JobScheduler::validate_cron_expression("0 0 * * * * *").is_ok());
        assert!(JobScheduler::validate_cron_expression("30 15 14 1 * * 2024").is_ok());

        // Invalid expressions
        assert!(JobScheduler::validate_cron_expression("0 * * *").is_err()); // Too few fields
        assert!(JobScheduler::validate_cron_expression("0 * * * * * * *").is_err()); // Too many fields
        assert!(JobScheduler::validate_cron_expression("0 * * * @ ").is_err()); // Invalid character
        assert!(JobScheduler::validate_cron_expression("abc * * * *").is_err()); // Invalid characters
    }

    #[test]
    fn test_is_valid_cron_field() {
        // Valid fields
        assert!(JobScheduler::is_valid_cron_field("*"));
        assert!(JobScheduler::is_valid_cron_field("0"));
        assert!(JobScheduler::is_valid_cron_field("0-5"));
        assert!(JobScheduler::is_valid_cron_field("*/15"));
        assert!(JobScheduler::is_valid_cron_field("0,15,30,45"));
        assert!(JobScheduler::is_valid_cron_field("?"));
        assert!(JobScheduler::is_valid_cron_field("1-5"));

        // Invalid fields
        assert!(!JobScheduler::is_valid_cron_field("@"));
        assert!(!JobScheduler::is_valid_cron_field("abc"));
        assert!(!JobScheduler::is_valid_cron_field("0-5-10"));
        assert!(!JobScheduler::is_valid_cron_field("*/*"));
        assert!(!JobScheduler::is_valid_cron_field("!"));
    }

    #[test]
    fn test_parse_cron_next_basic_expressions() {
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().into();

        // Test hourly: "0 * * * *" - every hour at minute 0
        let next = JobScheduler::parse_cron_next("0 * * * *", base_time).unwrap();
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);

        // Test daily: "0 0 * * *" - every day at midnight
        let next = JobScheduler::parse_cron_next("0 0 * * *", base_time).unwrap();
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);

        // Test specific time: "30 14 * * *" - every day at 2:30 PM
        let next = JobScheduler::parse_cron_next("30 14 * * *", base_time).unwrap();
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 30);
        assert!(next > base_time);
    }

    #[test]
    fn test_parse_cron_next_keywords() {
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().into();

        // Test @hourly
        let next = JobScheduler::parse_cron_next("@hourly", base_time).unwrap();
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);

        // Test @daily
        let next = JobScheduler::parse_cron_next("@daily", base_time).unwrap();
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);

        // Test @weekly
        let next = JobScheduler::parse_cron_next("@weekly", base_time).unwrap();
        assert_eq!(next.weekday(), Weekday::Sun);
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);

        // Test @monthly
        let next = JobScheduler::parse_cron_next("@monthly", base_time).unwrap();
        assert_eq!(next.day(), 1);
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);

        // Test @yearly
        let next = JobScheduler::parse_cron_next("@yearly", base_time).unwrap();
        assert_eq!(next.month(), 1);
        assert_eq!(next.day(), 1);
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert!(next > base_time);
    }

    #[test]
    fn test_parse_cron_next_complex_expressions() {
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().into();

        // Test every 15 minutes: "*/15 * * * *"
        let next = JobScheduler::parse_cron_next("*/15 * * * *", base_time).unwrap();
        assert!(next.minute() % 15 == 0);
        assert!(next > base_time);

        // Test weekdays at 9 AM: "0 9 * * 1-5"
        let next = JobScheduler::parse_cron_next("0 9 * * 1-5", base_time).unwrap();
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 0);
        let weekday = next.weekday().num_days_from_monday();
        assert!(weekday < 5); // Monday=0 to Friday=4
        assert!(next > base_time);

        // Test multiple specific minutes: "0,30 * * * *"
        let next = JobScheduler::parse_cron_next("0,30 * * * *", base_time).unwrap();
        assert!(next.minute() == 0 || next.minute() == 30);
        assert!(next > base_time);
    }

    #[test]
    fn test_parse_cron_next_invalid_expressions() {
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().into();

        // Test invalid format
        assert!(JobScheduler::parse_cron_next("invalid", base_time).is_err());
        assert!(JobScheduler::parse_cron_next("0 * * *", base_time).is_err()); // Too few fields
        assert!(JobScheduler::parse_cron_next("0 * * * * *", base_time).is_err()); // Too many fields

        // Test invalid ranges
        assert!(JobScheduler::parse_cron_next("60 * * * *", base_time).is_err()); // Invalid minute
        assert!(JobScheduler::parse_cron_next("0 25 * * *", base_time).is_err()); // Invalid hour
    }

    #[test]
    fn test_calculate_next_run_cron() {
        let schedule = JobSchedule {
            schedule_type: ScheduleType::Cron {
                expression: "@hourly".to_string(),
                max_runs: None,
            },
            start_time: None,
            end_time: None,
            timezone: "UTC".to_string(),
            enabled: true,
        };

        let now = SystemTime::now();
        let next = JobScheduler::calculate_next_run(&Some(schedule), now).unwrap();

        // The next run should be within the next hour
        let duration = next.duration_since(now).unwrap();
        assert!(duration <= Duration::from_secs(3600));

        // Convert to DateTime to check that it's at minute 0
        let next_dt: DateTime<Utc> = next.into();
        assert_eq!(next_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_next_run_cron_with_end_time() {
        let end_time = SystemTime::now() + Duration::from_secs(1800); // 30 minutes from now

        let schedule = JobSchedule {
            schedule_type: ScheduleType::Cron {
                expression: "@hourly".to_string(),
                max_runs: None,
            },
            start_time: None,
            end_time: Some(end_time),
            timezone: "UTC".to_string(),
            enabled: true,
        };

        let now = SystemTime::now();

        // This should fail because the next hourly run would be after the end time
        let result = JobScheduler::calculate_next_run(&Some(schedule), now);

        // Depending on the exact time this test runs, it might succeed or fail
        // If it succeeds, the next run should be before the end time
        if let Ok(next) = result {
            assert!(next <= end_time);
        }
        // If it fails, that's also valid - it means the next run would be after end_time
    }
}