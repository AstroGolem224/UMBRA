use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::errors::{AppError, Result};
use crate::types::{
    AppendRunEventInput, ChannelMessageKind, CreateDispatchRunInput, CreateOpsChannelInput,
    CreateOpsJobInput, DispatchRun, DispatchStatus, OpsChannel, OpsChannelMessage,
    OpsChannelMessagePage, OpsJob, OpsJobStatus, OpsRouteApproval, OpsRule, OpsSession,
    OpsSessionState, OpsSessionTemplate, RouteApprovalStatus, RunArtifact, RunArtifactKind,
    RunEvent, RunEventPage, RunEventType, RunOutcomeStatus, SaveOpsRuleInput,
    SaveOpsSessionTemplateInput, StartOpsSessionInput,
};

#[derive(Clone)]
pub struct WorkbenchStore {
    db_path: Arc<PathBuf>,
}

impl WorkbenchStore {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let store = Self {
            db_path: Arc::new(db_path),
        };
        store.ensure_schema()?;
        Ok(store)
    }

    pub fn list_runs(&self) -> Result<Vec<DispatchRun>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
             FROM dispatch_runs
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([], map_run_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_runs_for_channel(&self, channel_id: &str) -> Result<Vec<DispatchRun>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
             FROM dispatch_runs
             WHERE channel_id = ?1
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([channel_id], map_run_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_run(&self, run_id: &str) -> Result<Option<DispatchRun>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
             FROM dispatch_runs
             WHERE id = ?1",
            [run_id],
            map_run_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn list_events(&self, run_id: &str) -> Result<Vec<RunEvent>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, run_id, event_type, body, created_at
             FROM run_events
             WHERE run_id = ?1
             ORDER BY created_at ASC, id ASC",
        )?;
        let rows = stmt.query_map([run_id], map_event_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_events_page(
        &self,
        run_id: &str,
        before: Option<&str>,
        limit: usize,
    ) -> Result<RunEventPage> {
        let conn = self.connect()?;
        let limit = normalize_page_limit(limit);
        let page_size = (limit + 1) as i64;
        let mut items = if let Some(before) = before.filter(|value| !value.trim().is_empty()) {
            let mut stmt = conn.prepare(
                "SELECT id, run_id, event_type, body, created_at
                 FROM run_events
                 WHERE run_id = ?1 AND created_at < ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )?;
            let rows = stmt.query_map(params![run_id, before, page_size], map_event_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, run_id, event_type, body, created_at
                 FROM run_events
                 WHERE run_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![run_id, page_size], map_event_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let has_more = items.len() > limit;
        if has_more {
            items.truncate(limit);
        }
        items.reverse();
        let next_before = if has_more {
            items.first().map(|item| item.created_at.clone())
        } else {
            None
        };

        Ok(RunEventPage {
            items,
            next_before,
            has_more,
        })
    }

    pub fn list_artifacts(&self, run_id: &str) -> Result<Vec<RunArtifact>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, run_id, kind, label, value, created_at
             FROM run_artifacts
             WHERE run_id = ?1
             ORDER BY created_at ASC, id ASC",
        )?;
        let rows = stmt.query_map([run_id], map_artifact_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn create_run(
        &self,
        input: CreateDispatchRunInput,
        provider_id: String,
    ) -> Result<(DispatchRun, RunEvent)> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let timestamp = Utc::now().to_rfc3339();

        let run = DispatchRun {
            id: Uuid::new_v4().to_string(),
            parent_run_id: input.continue_from_run_id,
            channel_id: input.channel_id,
            source_message_id: input.source_message_id,
            job_id: input.job_id,
            session_id: input.session_id,
            mode: input.mode,
            agent_id: input.agent_id,
            provider_id,
            workspace_id: input.workspace_id,
            pm_task_id: input.pm_task_id,
            prompt: input.prompt,
            persona_id: input.persona_id,
            outcome_status: None,
            status: DispatchStatus::Queued,
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
        };

        tx.execute(
            "INSERT INTO dispatch_runs (id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                run.id,
                run.parent_run_id,
                run.channel_id,
                run.source_message_id,
                run.job_id,
                run.session_id,
                dispatch_mode_to_db(&run.mode),
                run.agent_id,
                run.provider_id,
                run.workspace_id,
                run.pm_task_id,
                run.prompt,
                run.persona_id,
                run.outcome_status
                    .as_ref()
                    .map(run_outcome_status_to_db),
                dispatch_status_to_db(&run.status),
                run.created_at,
                run.updated_at,
            ],
        )?;

        let first_event = RunEvent {
            id: Uuid::new_v4().to_string(),
            run_id: run.id.clone(),
            event_type: RunEventType::UserMessage,
            body: run.prompt.clone(),
            created_at: timestamp,
        };

        tx.execute(
            "INSERT INTO run_events (id, run_id, event_type, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                first_event.id,
                first_event.run_id,
                run_event_type_to_db(&first_event.event_type),
                first_event.body,
                first_event.created_at,
            ],
        )?;

        tx.commit()?;
        Ok((run, first_event))
    }

    pub fn cancel_run(&self, run_id: &str) -> Result<(DispatchRun, RunEvent)> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;

        let timestamp = Utc::now().to_rfc3339();
        let updated = tx.execute(
            "UPDATE dispatch_runs
             SET status = ?1, updated_at = ?2
             WHERE id = ?3",
            params![
                dispatch_status_to_db(&DispatchStatus::Cancelled),
                timestamp,
                run_id
            ],
        )?;

        if updated == 0 {
            return Err(AppError::TargetNotFound(run_id.into()));
        }

        let event = RunEvent {
            id: Uuid::new_v4().to_string(),
            run_id: run_id.to_string(),
            event_type: RunEventType::System,
            body: "run cancelled".into(),
            created_at: Utc::now().to_rfc3339(),
        };

        tx.execute(
            "INSERT INTO run_events (id, run_id, event_type, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.id,
                event.run_id,
                run_event_type_to_db(&event.event_type),
                event.body,
                event.created_at
            ],
        )?;

        let run = tx
            .query_row(
                "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
                 FROM dispatch_runs
                 WHERE id = ?1",
                [run_id],
                map_run_row,
            )
            .map_err(AppError::from)?;

        tx.commit()?;
        Ok((run, event))
    }

    pub fn set_run_status(&self, run_id: &str, status: DispatchStatus) -> Result<DispatchRun> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = tx.execute(
            "UPDATE dispatch_runs
             SET status = ?1, outcome_status = NULL, updated_at = ?2
             WHERE id = ?3",
            params![dispatch_status_to_db(&status), timestamp, run_id],
        )?;

        if updated == 0 {
            return Err(AppError::TargetNotFound(run_id.into()));
        }

        let run = tx
            .query_row(
                "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
                 FROM dispatch_runs
                 WHERE id = ?1",
                [run_id],
                map_run_row,
            )
            .map_err(AppError::from)?;

        tx.commit()?;
        Ok(run)
    }

    pub fn complete_run(
        &self,
        run_id: &str,
        status: DispatchStatus,
        outcome_status: Option<RunOutcomeStatus>,
    ) -> Result<DispatchRun> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = tx.execute(
            "UPDATE dispatch_runs
             SET status = ?1, outcome_status = ?2, updated_at = ?3
             WHERE id = ?4",
            params![
                dispatch_status_to_db(&status),
                outcome_status.as_ref().map(run_outcome_status_to_db),
                timestamp,
                run_id
            ],
        )?;

        if updated == 0 {
            return Err(AppError::TargetNotFound(run_id.into()));
        }

        let run = tx
            .query_row(
                "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
                 FROM dispatch_runs
                 WHERE id = ?1",
                [run_id],
                map_run_row,
            )
            .map_err(AppError::from)?;

        tx.commit()?;
        Ok(run)
    }

    pub fn append_event(&self, input: AppendRunEventInput) -> Result<(DispatchRun, RunEvent)> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;

        let timestamp = Utc::now().to_rfc3339();
        let updated = tx.execute(
            "UPDATE dispatch_runs
             SET updated_at = ?1
             WHERE id = ?2",
            params![timestamp, input.run_id],
        )?;

        if updated == 0 {
            return Err(AppError::TargetNotFound(input.run_id));
        }

        let event = RunEvent {
            id: Uuid::new_v4().to_string(),
            run_id: input.run_id.clone(),
            event_type: input.event_type,
            body: input.body.trim().to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        tx.execute(
            "INSERT INTO run_events (id, run_id, event_type, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.id,
                event.run_id,
                run_event_type_to_db(&event.event_type),
                event.body,
                event.created_at
            ],
        )?;

        let run = tx
            .query_row(
                "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
                 FROM dispatch_runs
                 WHERE id = ?1",
                [input.run_id],
                map_run_row,
            )
            .map_err(AppError::from)?;

        tx.commit()?;
        Ok((run, event))
    }

    pub fn replace_artifacts(
        &self,
        run_id: &str,
        artifacts: Vec<(RunArtifactKind, String, String)>,
    ) -> Result<Vec<RunArtifact>> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM run_artifacts WHERE run_id = ?1", [run_id])?;

        let timestamp = Utc::now().to_rfc3339();
        let mut created = Vec::with_capacity(artifacts.len());
        for (kind, label, value) in artifacts {
            let artifact = RunArtifact {
                id: Uuid::new_v4().to_string(),
                run_id: run_id.to_string(),
                kind,
                label,
                value,
                created_at: timestamp.clone(),
            };

            tx.execute(
                "INSERT INTO run_artifacts (id, run_id, kind, label, value, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    artifact.id,
                    artifact.run_id,
                    run_artifact_kind_to_db(&artifact.kind),
                    artifact.label,
                    artifact.value,
                    artifact.created_at,
                ],
            )?;
            created.push(artifact);
        }

        tx.commit()?;
        Ok(created)
    }

    pub fn list_channels(&self) -> Result<Vec<OpsChannel>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, workspace_id, default_agent_id, created_at, updated_at
             FROM ops_channels
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([], map_channel_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_channel(&self, channel_id: &str) -> Result<Option<OpsChannel>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, name, description, workspace_id, default_agent_id, created_at, updated_at
             FROM ops_channels
             WHERE id = ?1",
            [channel_id],
            map_channel_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn create_channel(&self, input: CreateOpsChannelInput) -> Result<OpsChannel> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let channel = OpsChannel {
            id: Uuid::new_v4().to_string(),
            name: input.name.trim().to_string(),
            description: input.description.trim().to_string(),
            workspace_id: input.workspace_id,
            default_agent_id: input.default_agent_id,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        conn.execute(
            "INSERT INTO ops_channels (id, name, description, workspace_id, default_agent_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                channel.id,
                channel.name,
                channel.description,
                channel.workspace_id,
                channel.default_agent_id,
                channel.created_at,
                channel.updated_at
            ],
        )?;
        Ok(channel)
    }

    pub fn list_channel_messages_page(
        &self,
        channel_id: &str,
        before: Option<&str>,
        limit: usize,
    ) -> Result<OpsChannelMessagePage> {
        let conn = self.connect()?;
        let limit = normalize_page_limit(limit);
        let page_size = (limit + 1) as i64;
        let mut items = if let Some(before) = before.filter(|value| !value.trim().is_empty()) {
            let mut stmt = conn.prepare(
                "SELECT id, channel_id, parent_message_id, run_id, job_id, session_id, agent_id, author_label, kind, body, created_at
                 FROM ops_channel_messages
                 WHERE channel_id = ?1 AND created_at < ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )?;
            let rows =
                stmt.query_map(params![channel_id, before, page_size], map_channel_message_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, channel_id, parent_message_id, run_id, job_id, session_id, agent_id, author_label, kind, body, created_at
                 FROM ops_channel_messages
                 WHERE channel_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![channel_id, page_size], map_channel_message_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let has_more = items.len() > limit;
        if has_more {
            items.truncate(limit);
        }
        items.reverse();
        let next_before = if has_more {
            items.first().map(|item| item.created_at.clone())
        } else {
            None
        };

        Ok(OpsChannelMessagePage {
            items,
            next_before,
            has_more,
        })
    }

    pub fn list_channel_messages(&self, channel_id: &str) -> Result<Vec<OpsChannelMessage>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, parent_message_id, run_id, job_id, session_id, agent_id, author_label, kind, body, created_at
             FROM ops_channel_messages
             WHERE channel_id = ?1
             ORDER BY created_at ASC, id ASC",
        )?;
        let rows = stmt.query_map([channel_id], map_channel_message_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn append_channel_message(
        &self,
        channel_id: &str,
        parent_message_id: Option<String>,
        run_id: Option<String>,
        job_id: Option<String>,
        session_id: Option<String>,
        agent_id: Option<String>,
        author_label: Option<String>,
        kind: ChannelMessageKind,
        body: String,
    ) -> Result<(OpsChannel, OpsChannelMessage)> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let timestamp = Utc::now().to_rfc3339();

        let updated = tx.execute(
            "UPDATE ops_channels
             SET updated_at = ?1
             WHERE id = ?2",
            params![timestamp, channel_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("channel {}", channel_id)));
        }

        let message = OpsChannelMessage {
            id: Uuid::new_v4().to_string(),
            channel_id: channel_id.to_string(),
            parent_message_id,
            run_id,
            job_id,
            session_id,
            agent_id,
            author_label,
            kind,
            body: body.trim().to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        tx.execute(
            "INSERT INTO ops_channel_messages (id, channel_id, parent_message_id, run_id, job_id, session_id, agent_id, author_label, kind, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                message.id,
                message.channel_id,
                message.parent_message_id,
                message.run_id,
                message.job_id,
                message.session_id,
                message.agent_id,
                message.author_label,
                channel_message_kind_to_db(&message.kind),
                message.body,
                message.created_at
            ],
        )?;

        let channel = tx
            .query_row(
                "SELECT id, name, description, workspace_id, default_agent_id, created_at, updated_at
                 FROM ops_channels
                 WHERE id = ?1",
                [channel_id],
                map_channel_row,
            )
            .map_err(AppError::from)?;
        tx.commit()?;
        Ok((channel, message))
    }

    pub fn recover_incomplete_runs(&self) -> Result<Vec<DispatchRun>> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let stranded = {
            let mut stmt = tx.prepare(
                "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
                 FROM dispatch_runs
                 WHERE status IN ('queued', 'working')
                 ORDER BY updated_at DESC, created_at DESC",
            )?;
            let rows = stmt.query_map([], map_run_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let mut recovered = Vec::with_capacity(stranded.len());
        for run in stranded {
            let timestamp = Utc::now().to_rfc3339();
            tx.execute(
                "UPDATE dispatch_runs
                 SET status = ?1, outcome_status = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![
                    dispatch_status_to_db(&DispatchStatus::Error),
                    run_outcome_status_to_db(&RunOutcomeStatus::NeedsInput),
                    timestamp,
                    run.id
                ],
            )?;
            let event = RunEvent {
                id: Uuid::new_v4().to_string(),
                run_id: run.id.clone(),
                event_type: RunEventType::System,
                body: "run recovered after app restart; previous provider process is no longer attached".into(),
                created_at: Utc::now().to_rfc3339(),
            };
            tx.execute(
                "INSERT INTO run_events (id, run_id, event_type, body, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    event.id,
                    event.run_id,
                    run_event_type_to_db(&event.event_type),
                    event.body,
                    event.created_at
                ],
            )?;
            let updated = tx.query_row(
                "SELECT id, parent_run_id, channel_id, source_message_id, job_id, session_id, mode, agent_id, provider_id, workspace_id, pm_task_id, prompt, persona_id, outcome_status, status, created_at, updated_at
                 FROM dispatch_runs
                 WHERE id = ?1",
                [run.id],
                map_run_row,
            )?;
            recovered.push(updated);
        }

        tx.commit()?;
        Ok(recovered)
    }

    pub fn list_jobs(&self, channel_id: &str) -> Result<Vec<OpsJob>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, source_message_id, title, summary, agent_id, workspace_id, pm_task_id, run_id, status, created_at, updated_at
             FROM ops_jobs
             WHERE channel_id = ?1
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([channel_id], map_job_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_job(&self, job_id: &str) -> Result<Option<OpsJob>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, channel_id, source_message_id, title, summary, agent_id, workspace_id, pm_task_id, run_id, status, created_at, updated_at
             FROM ops_jobs
             WHERE id = ?1",
            [job_id],
            map_job_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn create_job(&self, input: CreateOpsJobInput) -> Result<OpsJob> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let job = OpsJob {
            id: Uuid::new_v4().to_string(),
            channel_id: input.channel_id,
            source_message_id: input.source_message_id,
            title: input.title.trim().to_string(),
            summary: input.summary.trim().to_string(),
            agent_id: input.agent_id,
            workspace_id: input.workspace_id,
            pm_task_id: input.pm_task_id,
            run_id: None,
            status: OpsJobStatus::Open,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        conn.execute(
            "INSERT INTO ops_jobs (id, channel_id, source_message_id, title, summary, agent_id, workspace_id, pm_task_id, run_id, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                job.id,
                job.channel_id,
                job.source_message_id,
                job.title,
                job.summary,
                job.agent_id,
                job.workspace_id,
                job.pm_task_id,
                job.run_id,
                ops_job_status_to_db(&job.status),
                job.created_at,
                job.updated_at
            ],
        )?;
        Ok(job)
    }

    pub fn set_job_run(&self, job_id: &str, run_id: &str, status: OpsJobStatus) -> Result<OpsJob> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE ops_jobs
             SET run_id = ?1, status = ?2, updated_at = ?3
             WHERE id = ?4",
            params![run_id, ops_job_status_to_db(&status), timestamp, job_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("job {}", job_id)));
        }
        self.get_job(job_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("job {}", job_id)))
    }

    pub fn set_job_status(&self, job_id: &str, status: OpsJobStatus) -> Result<OpsJob> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE ops_jobs
             SET status = ?1, updated_at = ?2
             WHERE id = ?3",
            params![ops_job_status_to_db(&status), timestamp, job_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("job {}", job_id)));
        }
        self.get_job(job_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("job {}", job_id)))
    }

    pub fn list_route_approvals(&self, channel_id: &str) -> Result<Vec<OpsRouteApproval>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, message_id, agent_id, workspace_id, reason, status, created_at, updated_at
             FROM ops_route_approvals
             WHERE channel_id = ?1
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([channel_id], map_route_approval_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_route_approval(&self, approval_id: &str) -> Result<Option<OpsRouteApproval>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, channel_id, message_id, agent_id, workspace_id, reason, status, created_at, updated_at
             FROM ops_route_approvals
             WHERE id = ?1",
            [approval_id],
            map_route_approval_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn create_route_approval(
        &self,
        channel_id: &str,
        message_id: &str,
        agent_id: &str,
        workspace_id: &str,
        reason: &str,
    ) -> Result<OpsRouteApproval> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let approval = OpsRouteApproval {
            id: Uuid::new_v4().to_string(),
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
            agent_id: agent_id.to_string(),
            workspace_id: workspace_id.to_string(),
            reason: reason.trim().to_string(),
            status: RouteApprovalStatus::Pending,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        conn.execute(
            "INSERT INTO ops_route_approvals (id, channel_id, message_id, agent_id, workspace_id, reason, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                approval.id,
                approval.channel_id,
                approval.message_id,
                approval.agent_id,
                approval.workspace_id,
                approval.reason,
                route_approval_status_to_db(&approval.status),
                approval.created_at,
                approval.updated_at
            ],
        )?;
        Ok(approval)
    }

    pub fn resolve_route_approval(
        &self,
        approval_id: &str,
        status: RouteApprovalStatus,
    ) -> Result<OpsRouteApproval> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE ops_route_approvals
             SET status = ?1, updated_at = ?2
             WHERE id = ?3",
            params![route_approval_status_to_db(&status), timestamp, approval_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("route approval {}", approval_id)));
        }
        self.get_route_approval(approval_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("route approval {}", approval_id)))
    }

    pub fn list_rules(&self) -> Result<Vec<OpsRule>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, pattern, target_agent_id, workspace_id, enabled, requires_human_gate, last_triggered_at, created_at, updated_at
             FROM ops_rules
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([], map_rule_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn save_rule(&self, input: SaveOpsRuleInput) -> Result<OpsRule> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let rule_id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let existing = self.get_rule(&rule_id)?;
        let created_at = existing
            .as_ref()
            .map(|rule| rule.created_at.clone())
            .unwrap_or_else(|| timestamp.clone());
        conn.execute(
            "INSERT INTO ops_rules (id, name, pattern, target_agent_id, workspace_id, enabled, requires_human_gate, last_triggered_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               pattern = excluded.pattern,
               target_agent_id = excluded.target_agent_id,
               workspace_id = excluded.workspace_id,
               enabled = excluded.enabled,
               requires_human_gate = excluded.requires_human_gate,
               updated_at = excluded.updated_at",
            params![
                rule_id,
                input.name.trim(),
                input.pattern.trim(),
                input.target_agent_id,
                input.workspace_id,
                bool_to_db(input.enabled),
                bool_to_db(input.requires_human_gate),
                existing.as_ref().and_then(|rule| rule.last_triggered_at.clone()),
                created_at,
                timestamp,
            ],
        )?;
        self.get_rule(&rule_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("rule {}", rule_id)))
    }

    pub fn get_rule(&self, rule_id: &str) -> Result<Option<OpsRule>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, name, pattern, target_agent_id, workspace_id, enabled, requires_human_gate, last_triggered_at, created_at, updated_at
             FROM ops_rules
             WHERE id = ?1",
            [rule_id],
            map_rule_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn set_rule_triggered(&self, rule_id: &str) -> Result<OpsRule> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE ops_rules
             SET last_triggered_at = ?1, updated_at = ?2
             WHERE id = ?3",
            params![timestamp, timestamp, rule_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("rule {}", rule_id)));
        }
        self.get_rule(rule_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("rule {}", rule_id)))
    }

    pub fn list_session_templates(&self) -> Result<Vec<OpsSessionTemplate>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, workspace_id, agent_ids_json, auto_advance, requires_human_gate, created_at, updated_at
             FROM ops_session_templates
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([], map_session_template_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_session_template(&self, template_id: &str) -> Result<Option<OpsSessionTemplate>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, name, workspace_id, agent_ids_json, auto_advance, requires_human_gate, created_at, updated_at
             FROM ops_session_templates
             WHERE id = ?1",
            [template_id],
            map_session_template_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn save_session_template(
        &self,
        input: SaveOpsSessionTemplateInput,
    ) -> Result<OpsSessionTemplate> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let template_id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let existing = self.get_session_template(&template_id)?;
        let created_at = existing
            .as_ref()
            .map(|template| template.created_at.clone())
            .unwrap_or_else(|| timestamp.clone());
        let agent_ids_json = serde_json::to_string(&input.agent_ids)
            .map_err(|error| AppError::Other(format!("invalid session agent ids: {error}")))?;
        conn.execute(
            "INSERT INTO ops_session_templates (id, name, workspace_id, agent_ids_json, auto_advance, requires_human_gate, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               workspace_id = excluded.workspace_id,
               agent_ids_json = excluded.agent_ids_json,
               auto_advance = excluded.auto_advance,
               requires_human_gate = excluded.requires_human_gate,
               updated_at = excluded.updated_at",
            params![
                template_id,
                input.name.trim(),
                input.workspace_id,
                agent_ids_json,
                bool_to_db(input.auto_advance),
                bool_to_db(input.requires_human_gate),
                created_at,
                timestamp,
            ],
        )?;
        self.get_session_template(&template_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("session template {}", template_id)))
    }

    pub fn list_sessions(&self, channel_id: &str) -> Result<Vec<OpsSession>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, template_id, state, current_turn_index, awaiting_human_gate, created_at, updated_at
             FROM ops_sessions
             WHERE channel_id = ?1
             ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([channel_id], map_session_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<OpsSession>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, channel_id, template_id, state, current_turn_index, awaiting_human_gate, created_at, updated_at
             FROM ops_sessions
             WHERE id = ?1",
            [session_id],
            map_session_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn get_active_session_for_channel(&self, channel_id: &str) -> Result<Option<OpsSession>> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, channel_id, template_id, state, current_turn_index, awaiting_human_gate, created_at, updated_at
             FROM ops_sessions
             WHERE channel_id = ?1 AND state = 'active'
             ORDER BY updated_at DESC, created_at DESC
             LIMIT 1",
            [channel_id],
            map_session_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn start_session(&self, input: StartOpsSessionInput) -> Result<OpsSession> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let session = OpsSession {
            id: Uuid::new_v4().to_string(),
            channel_id: input.channel_id,
            template_id: input.template_id,
            state: OpsSessionState::Active,
            current_turn_index: 0,
            awaiting_human_gate: false,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        conn.execute(
            "INSERT INTO ops_sessions (id, channel_id, template_id, state, current_turn_index, awaiting_human_gate, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session.id,
                session.channel_id,
                session.template_id,
                ops_session_state_to_db(&session.state),
                session.current_turn_index as i64,
                bool_to_db(session.awaiting_human_gate),
                session.created_at,
                session.updated_at
            ],
        )?;
        Ok(session)
    }

    pub fn update_session_state(
        &self,
        session_id: &str,
        state: OpsSessionState,
    ) -> Result<OpsSession> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE ops_sessions
             SET state = ?1, updated_at = ?2
             WHERE id = ?3",
            params![ops_session_state_to_db(&state), timestamp, session_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("session {}", session_id)));
        }
        self.get_session(session_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("session {}", session_id)))
    }

    pub fn set_session_turn(
        &self,
        session_id: &str,
        current_turn_index: usize,
        awaiting_human_gate: bool,
    ) -> Result<OpsSession> {
        let conn = self.connect()?;
        let timestamp = Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE ops_sessions
             SET current_turn_index = ?1, awaiting_human_gate = ?2, updated_at = ?3
             WHERE id = ?4",
            params![current_turn_index as i64, bool_to_db(awaiting_human_gate), timestamp, session_id],
        )?;
        if updated == 0 {
            return Err(AppError::TargetNotFound(format!("session {}", session_id)));
        }
        self.get_session(session_id)?
            .ok_or_else(|| AppError::TargetNotFound(format!("session {}", session_id)))
    }

    fn ensure_schema(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS dispatch_runs (
               id TEXT PRIMARY KEY,
               parent_run_id TEXT NULL,
               channel_id TEXT NULL,
               source_message_id TEXT NULL,
               job_id TEXT NULL,
               session_id TEXT NULL,
               mode TEXT NOT NULL,
               agent_id TEXT NOT NULL,
               provider_id TEXT NOT NULL,
               workspace_id TEXT NOT NULL,
               pm_task_id TEXT NULL,
               prompt TEXT NOT NULL,
               persona_id TEXT NULL,
               outcome_status TEXT NULL,
               status TEXT NOT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS run_events (
               id TEXT PRIMARY KEY,
               run_id TEXT NOT NULL,
               event_type TEXT NOT NULL,
               body TEXT NOT NULL,
               created_at TEXT NOT NULL,
               FOREIGN KEY(run_id) REFERENCES dispatch_runs(id) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS run_artifacts (
               id TEXT PRIMARY KEY,
               run_id TEXT NOT NULL,
               kind TEXT NOT NULL,
               label TEXT NOT NULL,
               value TEXT NOT NULL,
               created_at TEXT NOT NULL,
               FOREIGN KEY(run_id) REFERENCES dispatch_runs(id) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS ops_channels (
               id TEXT PRIMARY KEY,
               name TEXT NOT NULL,
               description TEXT NOT NULL,
               workspace_id TEXT NOT NULL,
               default_agent_id TEXT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS ops_channel_messages (
               id TEXT PRIMARY KEY,
               channel_id TEXT NOT NULL,
               parent_message_id TEXT NULL,
               run_id TEXT NULL,
               job_id TEXT NULL,
               session_id TEXT NULL,
               agent_id TEXT NULL,
               author_label TEXT NULL,
               kind TEXT NOT NULL,
               body TEXT NOT NULL,
               created_at TEXT NOT NULL,
               FOREIGN KEY(channel_id) REFERENCES ops_channels(id) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS ops_jobs (
               id TEXT PRIMARY KEY,
               channel_id TEXT NOT NULL,
               source_message_id TEXT NOT NULL,
               title TEXT NOT NULL,
               summary TEXT NOT NULL,
               agent_id TEXT NOT NULL,
               workspace_id TEXT NOT NULL,
               pm_task_id TEXT NULL,
               run_id TEXT NULL,
               status TEXT NOT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               FOREIGN KEY(channel_id) REFERENCES ops_channels(id) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS ops_route_approvals (
               id TEXT PRIMARY KEY,
               channel_id TEXT NOT NULL,
               message_id TEXT NOT NULL,
               agent_id TEXT NOT NULL,
               workspace_id TEXT NOT NULL,
               reason TEXT NOT NULL,
               status TEXT NOT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               FOREIGN KEY(channel_id) REFERENCES ops_channels(id) ON DELETE CASCADE
             );
             CREATE TABLE IF NOT EXISTS ops_rules (
               id TEXT PRIMARY KEY,
               name TEXT NOT NULL,
               pattern TEXT NOT NULL,
               target_agent_id TEXT NULL,
               workspace_id TEXT NULL,
               enabled INTEGER NOT NULL,
               requires_human_gate INTEGER NOT NULL,
               last_triggered_at TEXT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS ops_session_templates (
               id TEXT PRIMARY KEY,
               name TEXT NOT NULL,
               workspace_id TEXT NOT NULL,
               agent_ids_json TEXT NOT NULL,
               auto_advance INTEGER NOT NULL,
               requires_human_gate INTEGER NOT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS ops_sessions (
               id TEXT PRIMARY KEY,
               channel_id TEXT NOT NULL,
               template_id TEXT NOT NULL,
               state TEXT NOT NULL,
               current_turn_index INTEGER NOT NULL,
               awaiting_human_gate INTEGER NOT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               FOREIGN KEY(channel_id) REFERENCES ops_channels(id) ON DELETE CASCADE,
               FOREIGN KEY(template_id) REFERENCES ops_session_templates(id) ON DELETE CASCADE
             );
             CREATE INDEX IF NOT EXISTS idx_dispatch_runs_updated_at ON dispatch_runs(updated_at DESC);
             CREATE INDEX IF NOT EXISTS idx_run_events_run_id_created_at ON run_events(run_id, created_at);
             CREATE INDEX IF NOT EXISTS idx_run_artifacts_run_id_created_at ON run_artifacts(run_id, created_at);
             CREATE INDEX IF NOT EXISTS idx_ops_channel_messages_channel_id_created_at ON ops_channel_messages(channel_id, created_at);
             CREATE INDEX IF NOT EXISTS idx_ops_jobs_channel_id_updated_at ON ops_jobs(channel_id, updated_at DESC);
             CREATE INDEX IF NOT EXISTS idx_ops_route_approvals_channel_id_updated_at ON ops_route_approvals(channel_id, updated_at DESC);
             CREATE INDEX IF NOT EXISTS idx_ops_sessions_channel_id_updated_at ON ops_sessions(channel_id, updated_at DESC);",
        )?;
        ensure_optional_column(&conn, "dispatch_runs", "parent_run_id", "TEXT NULL")?;
        ensure_optional_column(&conn, "dispatch_runs", "channel_id", "TEXT NULL")?;
        ensure_optional_column(&conn, "dispatch_runs", "source_message_id", "TEXT NULL")?;
        ensure_optional_column(&conn, "dispatch_runs", "job_id", "TEXT NULL")?;
        ensure_optional_column(&conn, "dispatch_runs", "session_id", "TEXT NULL")?;
        ensure_optional_column(&conn, "dispatch_runs", "pm_task_id", "TEXT NULL")?;
        ensure_optional_column(&conn, "dispatch_runs", "outcome_status", "TEXT NULL")?;
        Ok(())
    }

    fn connect(&self) -> Result<Connection> {
        if let Some(parent) = Path::new(&*self.db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Connection::open(&*self.db_path)?)
    }
}

fn map_run_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DispatchRun> {
    Ok(DispatchRun {
        id: row.get(0)?,
        parent_run_id: row.get(1)?,
        channel_id: row.get(2)?,
        source_message_id: row.get(3)?,
        job_id: row.get(4)?,
        session_id: row.get(5)?,
        mode: dispatch_mode_from_db(&row.get::<_, String>(6)?),
        agent_id: row.get(7)?,
        provider_id: row.get(8)?,
        workspace_id: row.get(9)?,
        pm_task_id: row.get(10)?,
        prompt: row.get(11)?,
        persona_id: row.get(12)?,
        outcome_status: row
            .get::<_, Option<String>>(13)?
            .as_deref()
            .map(run_outcome_status_from_db),
        status: dispatch_status_from_db(&row.get::<_, String>(14)?),
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn map_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunEvent> {
    Ok(RunEvent {
        id: row.get(0)?,
        run_id: row.get(1)?,
        event_type: run_event_type_from_db(&row.get::<_, String>(2)?),
        body: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn map_artifact_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunArtifact> {
    Ok(RunArtifact {
        id: row.get(0)?,
        run_id: row.get(1)?,
        kind: run_artifact_kind_from_db(&row.get::<_, String>(2)?),
        label: row.get(3)?,
        value: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn map_channel_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsChannel> {
    Ok(OpsChannel {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        workspace_id: row.get(3)?,
        default_agent_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_channel_message_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsChannelMessage> {
    Ok(OpsChannelMessage {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        parent_message_id: row.get(2)?,
        run_id: row.get(3)?,
        job_id: row.get(4)?,
        session_id: row.get(5)?,
        agent_id: row.get(6)?,
        author_label: row.get(7)?,
        kind: channel_message_kind_from_db(&row.get::<_, String>(8)?),
        body: row.get(9)?,
        created_at: row.get(10)?,
    })
}

fn map_job_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsJob> {
    Ok(OpsJob {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        source_message_id: row.get(2)?,
        title: row.get(3)?,
        summary: row.get(4)?,
        agent_id: row.get(5)?,
        workspace_id: row.get(6)?,
        pm_task_id: row.get(7)?,
        run_id: row.get(8)?,
        status: ops_job_status_from_db(&row.get::<_, String>(9)?),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn map_route_approval_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsRouteApproval> {
    Ok(OpsRouteApproval {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        message_id: row.get(2)?,
        agent_id: row.get(3)?,
        workspace_id: row.get(4)?,
        reason: row.get(5)?,
        status: route_approval_status_from_db(&row.get::<_, String>(6)?),
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn map_rule_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsRule> {
    Ok(OpsRule {
        id: row.get(0)?,
        name: row.get(1)?,
        pattern: row.get(2)?,
        target_agent_id: row.get(3)?,
        workspace_id: row.get(4)?,
        enabled: db_to_bool(row.get::<_, i64>(5)?),
        requires_human_gate: db_to_bool(row.get::<_, i64>(6)?),
        last_triggered_at: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn map_session_template_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsSessionTemplate> {
    let agent_ids_json: String = row.get(3)?;
    let agent_ids = serde_json::from_str(&agent_ids_json).unwrap_or_default();
    Ok(OpsSessionTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        workspace_id: row.get(2)?,
        agent_ids,
        auto_advance: db_to_bool(row.get::<_, i64>(4)?),
        requires_human_gate: db_to_bool(row.get::<_, i64>(5)?),
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_session_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OpsSession> {
    Ok(OpsSession {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        template_id: row.get(2)?,
        state: ops_session_state_from_db(&row.get::<_, String>(3)?),
        current_turn_index: row.get::<_, i64>(4)? as usize,
        awaiting_human_gate: db_to_bool(row.get::<_, i64>(5)?),
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn dispatch_mode_to_db(value: &crate::types::DispatchMode) -> &'static str {
    match value {
        crate::types::DispatchMode::Chat => "chat",
        crate::types::DispatchMode::Task => "task",
    }
}

fn dispatch_mode_from_db(value: &str) -> crate::types::DispatchMode {
    match value {
        "chat" => crate::types::DispatchMode::Chat,
        _ => crate::types::DispatchMode::Task,
    }
}

fn dispatch_status_to_db(value: &DispatchStatus) -> &'static str {
    match value {
        DispatchStatus::Draft => "draft",
        DispatchStatus::Queued => "queued",
        DispatchStatus::Working => "working",
        DispatchStatus::Done => "done",
        DispatchStatus::Error => "error",
        DispatchStatus::Cancelled => "cancelled",
    }
}

fn dispatch_status_from_db(value: &str) -> DispatchStatus {
    match value {
        "draft" => DispatchStatus::Draft,
        "working" => DispatchStatus::Working,
        "done" => DispatchStatus::Done,
        "error" => DispatchStatus::Error,
        "cancelled" => DispatchStatus::Cancelled,
        _ => DispatchStatus::Queued,
    }
}

fn run_outcome_status_to_db(value: &RunOutcomeStatus) -> &'static str {
    match value {
        RunOutcomeStatus::Succeeded => "succeeded",
        RunOutcomeStatus::Blocked => "blocked",
        RunOutcomeStatus::NeedsInput => "needs_input",
    }
}

fn run_outcome_status_from_db(value: &str) -> RunOutcomeStatus {
    match value {
        "blocked" => RunOutcomeStatus::Blocked,
        "needs_input" => RunOutcomeStatus::NeedsInput,
        _ => RunOutcomeStatus::Succeeded,
    }
}

fn run_event_type_to_db(value: &RunEventType) -> &'static str {
    match value {
        RunEventType::UserMessage => "user_message",
        RunEventType::System => "system",
        RunEventType::Stdout => "stdout",
        RunEventType::Stderr => "stderr",
        RunEventType::AgentMessage => "agent_message",
    }
}

fn run_event_type_from_db(value: &str) -> RunEventType {
    match value {
        "system" => RunEventType::System,
        "stdout" => RunEventType::Stdout,
        "stderr" => RunEventType::Stderr,
        "agent_message" => RunEventType::AgentMessage,
        _ => RunEventType::UserMessage,
    }
}

fn run_artifact_kind_to_db(value: &RunArtifactKind) -> &'static str {
    match value {
        RunArtifactKind::Summary => "summary",
        RunArtifactKind::File => "file",
        RunArtifactKind::Test => "test",
    }
}

fn run_artifact_kind_from_db(value: &str) -> RunArtifactKind {
    match value {
        "file" => RunArtifactKind::File,
        "test" => RunArtifactKind::Test,
        _ => RunArtifactKind::Summary,
    }
}

fn channel_message_kind_to_db(value: &ChannelMessageKind) -> &'static str {
    match value {
        ChannelMessageKind::User => "user",
        ChannelMessageKind::Agent => "agent",
        ChannelMessageKind::System => "system",
    }
}

fn channel_message_kind_from_db(value: &str) -> ChannelMessageKind {
    match value {
        "agent" => ChannelMessageKind::Agent,
        "system" => ChannelMessageKind::System,
        _ => ChannelMessageKind::User,
    }
}

fn ops_job_status_to_db(value: &OpsJobStatus) -> &'static str {
    match value {
        OpsJobStatus::Open => "open",
        OpsJobStatus::Running => "running",
        OpsJobStatus::Blocked => "blocked",
        OpsJobStatus::Done => "done",
    }
}

fn ops_job_status_from_db(value: &str) -> OpsJobStatus {
    match value {
        "running" => OpsJobStatus::Running,
        "blocked" => OpsJobStatus::Blocked,
        "done" => OpsJobStatus::Done,
        _ => OpsJobStatus::Open,
    }
}

fn route_approval_status_to_db(value: &RouteApprovalStatus) -> &'static str {
    match value {
        RouteApprovalStatus::Pending => "pending",
        RouteApprovalStatus::Approved => "approved",
        RouteApprovalStatus::Rejected => "rejected",
    }
}

fn route_approval_status_from_db(value: &str) -> RouteApprovalStatus {
    match value {
        "approved" => RouteApprovalStatus::Approved,
        "rejected" => RouteApprovalStatus::Rejected,
        _ => RouteApprovalStatus::Pending,
    }
}

fn ops_session_state_to_db(value: &OpsSessionState) -> &'static str {
    match value {
        OpsSessionState::Active => "active",
        OpsSessionState::Paused => "paused",
        OpsSessionState::Completed => "completed",
    }
}

fn ops_session_state_from_db(value: &str) -> OpsSessionState {
    match value {
        "paused" => OpsSessionState::Paused,
        "completed" => OpsSessionState::Completed,
        _ => OpsSessionState::Active,
    }
}

fn bool_to_db(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn db_to_bool(value: i64) -> bool {
    value != 0
}

fn normalize_page_limit(limit: usize) -> usize {
    limit.clamp(1, 200)
}

fn ensure_optional_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&pragma)?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    let has_column = columns
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .any(|name| name == column);
    if !has_column {
        let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        conn.execute(&alter, [])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::types::{
        CreateDispatchRunInput, DispatchMode, RunArtifactKind, RunEventType, RunOutcomeStatus,
    };

    use super::WorkbenchStore;

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "umbra-workbench-test-{}.sqlite3",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn creates_runs_and_initial_events() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");

        let (run, event) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Task,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: None,
                    prompt: "ship it".into(),
                    persona_id: None,
                    continue_from_run_id: None,
                },
                "custom".into(),
            )
            .expect("run");

        assert_eq!(run.status, crate::types::DispatchStatus::Queued);
        assert_eq!(event.event_type, RunEventType::UserMessage);
        let listed = store.list_runs().expect("list runs");
        assert_eq!(listed.len(), 1);
        let events = store.list_events(&run.id).expect("list events");
        assert_eq!(events.len(), 1);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cancel_run_appends_system_event() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");
        let (run, _) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Chat,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: None,
                    prompt: "hello".into(),
                    persona_id: Some("architect".into()),
                    continue_from_run_id: None,
                },
                "custom".into(),
            )
            .expect("run");

        let (cancelled, event) = store.cancel_run(&run.id).expect("cancel");
        assert_eq!(cancelled.status, crate::types::DispatchStatus::Cancelled);
        assert_eq!(event.event_type, RunEventType::System);
        assert_eq!(store.list_events(&run.id).expect("events").len(), 2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn set_run_status_updates_run() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");
        let (run, _) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Task,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: None,
                    prompt: "do work".into(),
                    persona_id: None,
                    continue_from_run_id: None,
                },
                "codex".into(),
            )
            .expect("run");

        let updated = store
            .set_run_status(&run.id, crate::types::DispatchStatus::Working)
            .expect("update");

        assert_eq!(updated.status, crate::types::DispatchStatus::Working);
        assert_eq!(updated.id, run.id);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn replace_artifacts_overwrites_previous_entries() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");
        let (run, _) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Task,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: None,
                    prompt: "artifact pass".into(),
                    persona_id: None,
                    continue_from_run_id: None,
                },
                "codex".into(),
            )
            .expect("run");

        let first = store
            .replace_artifacts(
                &run.id,
                vec![(RunArtifactKind::Summary, "result".into(), "done".into())],
            )
            .expect("first");
        assert_eq!(first.len(), 1);

        let second = store
            .replace_artifacts(
                &run.id,
                vec![(RunArtifactKind::File, "src/main.rs".into(), "M".into())],
            )
            .expect("second");
        assert_eq!(second.len(), 1);
        let listed = store.list_artifacts(&run.id).expect("artifacts");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].kind, RunArtifactKind::File);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn complete_run_persists_outcome_status() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");
        let (run, _) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Task,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: Some("task-123".into()),
                    prompt: "need review".into(),
                    persona_id: None,
                    continue_from_run_id: None,
                },
                "codex".into(),
            )
            .expect("run");

        let updated = store
            .complete_run(
                &run.id,
                crate::types::DispatchStatus::Done,
                Some(RunOutcomeStatus::NeedsInput),
            )
            .expect("complete");

        assert_eq!(updated.outcome_status, Some(RunOutcomeStatus::NeedsInput));
        assert_eq!(updated.pm_task_id.as_deref(), Some("task-123"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn run_event_pagination_returns_latest_window() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");
        let (run, _) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Task,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: None,
                    prompt: "paginate".into(),
                    persona_id: None,
                    continue_from_run_id: None,
                },
                "codex".into(),
            )
            .expect("run");

        for index in 0..4 {
            store
                .append_event(crate::types::AppendRunEventInput {
                    run_id: run.id.clone(),
                    event_type: RunEventType::System,
                    body: format!("event-{index}"),
                })
                .expect("append event");
        }

        let first_page = store
            .list_events_page(&run.id, None, 2)
            .expect("first page");
        assert_eq!(first_page.items.len(), 2);
        assert!(first_page.has_more);

        let second_page = store
            .list_events_page(&run.id, first_page.next_before.as_deref(), 2)
            .expect("second page");
        assert_eq!(second_page.items.len(), 2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn recover_incomplete_runs_marks_stranded_runs() {
        let path = temp_db_path();
        let store = WorkbenchStore::new(path.clone()).expect("store");
        let (run, _) = store
            .create_run(
                CreateDispatchRunInput {
                    mode: DispatchMode::Task,
                    agent_id: "forge".into(),
                    workspace_id: "core".into(),
                    channel_id: None,
                    source_message_id: None,
                    job_id: None,
                    session_id: None,
                    pm_task_id: None,
                    prompt: "recover me".into(),
                    persona_id: None,
                    continue_from_run_id: None,
                },
                "codex".into(),
            )
            .expect("run");

        store
            .set_run_status(&run.id, crate::types::DispatchStatus::Working)
            .expect("working");

        let recovered = store.recover_incomplete_runs().expect("recover");
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].status, crate::types::DispatchStatus::Error);
        assert_eq!(
            recovered[0].outcome_status,
            Some(RunOutcomeStatus::NeedsInput)
        );

        let events = store.list_events(&run.id).expect("events");
        assert!(events
            .iter()
            .any(|event| event.body.contains("recovered after app restart")));

        let _ = std::fs::remove_file(path);
    }
}
