use crate::batch::queue::{
    BatchJob, JobSchedule, ScheduleType, JobQueueManager, QueueStatus
};
use anyhow::Result;
use chrono::{DateTime, Utc, TimeZone, Datelike, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{RwLock, mpsc},
    time::{interval, sleep_until, Instant},
};
use tracing::{debug, error, info, warn};

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
            for (job_id, entry) in jobs.iter() {
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
        // Simple cron parser for basic expressions
        // Format: "minute hour day month weekday"
        // This is a simplified implementation

        let parts: Vec<&str> = expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(anyhow::anyhow!("Invalid cron expression format"));
        }

        // For now, just implement a basic hourly schedule
        // TODO: Implement full cron parsing
        match expression {
            "0 * * * *" => Ok(from.with_minute(0).unwrap().with_second(0).unwrap() + chrono::Duration::hours(1)),
            "0 0 * * *" => Ok(from.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap() + chrono::Duration::days(1)),
            _ => {
                warn!("Complex cron expressions not fully implemented, defaulting to hourly");
                Ok(from + chrono::Duration::hours(1))
            }
        }
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
}