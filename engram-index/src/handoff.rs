//! Rolling handoff service backed by Memory OS handoff items.

use crate::error::IndexResult;
use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, MemoryItem, MemoryKind, MemoryScope, MemoryStatus,
    WriterProvenance,
};
use engram_core::session::{Event, EventType};
use engram_store::{Db, MemoryRepo, SessionRepo};
use serde::{Deserialize, Serialize};

/// Handoff update result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffUpdate {
    /// Dry-run mode.
    pub dry_run: bool,
    /// Previous active handoff ID, if any.
    pub previous_id: Option<Id>,
    /// Planned or written handoff item.
    pub item: MemoryItem,
    /// Whether the item was written.
    pub written: bool,
}

/// Handoff get result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffGet {
    /// Matching active handoff, if any.
    pub item: Option<MemoryItem>,
}

/// Handoff compile result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffCompile {
    /// Compiled Markdown handoff.
    pub content: String,
    /// Number of events used.
    pub event_count: usize,
    /// Update result when write mode was requested.
    pub update: Option<HandoffUpdate>,
}

/// Service for rolling handoffs.
#[derive(Clone)]
pub struct HandoffService {
    memory_repo: MemoryRepo,
    session_repo: SessionRepo,
}

impl HandoffService {
    /// Create a handoff service.
    pub fn new(db: Db) -> Self {
        Self {
            memory_repo: MemoryRepo::new(db.clone()),
            session_repo: SessionRepo::new(db),
        }
    }

    /// Initialize required schemas.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.memory_repo.init_schema().await?;
        self.session_repo.init_schema().await?;
        Ok(())
    }

    /// Get the latest active handoff for a project/session.
    pub async fn get(
        &self,
        project: Option<&str>,
        session_id: Option<Id>,
    ) -> IndexResult<HandoffGet> {
        Ok(HandoffGet {
            item: self.latest_handoff(project, session_id).await?,
        })
    }

    /// Update the rolling handoff.
    pub async fn update(
        &self,
        project: Option<String>,
        session_id: Option<Id>,
        content: String,
        next_actions: Vec<String>,
        writer: WriterProvenance,
        dry_run: bool,
    ) -> IndexResult<HandoffUpdate> {
        let previous_items = self.active_handoffs(project.as_deref(), session_id).await?;
        let scope = handoff_scope(project, session_id);
        let content = normalize_handoff_content(&content, &next_actions);
        let update_evidence = handoff_update_evidence(&scope);
        let mut item = MemoryItem::new(
            MemoryKind::Handoff,
            "Rolling handoff",
            content,
            scope,
            ClaimOrigin::AgentObserved,
            writer,
        )
        .with_tag("handoff")
        .with_tag("rolling")
        .with_evidence(update_evidence);
        if let Some(session_id) = session_id {
            item = item.with_evidence(
                EvidenceRef::new(EvidenceKind::SessionEvent, session_id.to_string())
                    .with_summary("Rolling handoff update for session"),
            );
        }
        for previous in &previous_items {
            item = item.with_superseded_item(previous.id);
        }

        let previous_id = previous_items.first().map(|item| item.id);

        if !dry_run {
            self.memory_repo.save_memory_item(&item).await?;
            for previous in previous_items {
                let superseded = previous
                    .with_status(MemoryStatus::Superseded)
                    .with_evidence(
                        EvidenceRef::new(
                            EvidenceKind::ToolCall,
                            format!("handoff(action=update):{}", item.id),
                        )
                        .with_summary(format!(
                            "Superseded by newer rolling handoff {} for the same scope.",
                            item.id
                        )),
                    );
                self.memory_repo.save_memory_item(&superseded).await?;
            }
        }

        Ok(HandoffUpdate {
            dry_run,
            previous_id,
            item,
            written: !dry_run,
        })
    }

    /// Compile a handoff from session events and optionally write it.
    pub async fn compile(
        &self,
        session_id: Id,
        project: Option<String>,
        writer: WriterProvenance,
        dry_run: bool,
    ) -> IndexResult<HandoffCompile> {
        let events = self.session_repo.get_events(&session_id).await?;
        let content = compile_events_to_handoff(&events);
        let update = if dry_run {
            None
        } else {
            Some(
                self.update(
                    project,
                    Some(session_id),
                    content.clone(),
                    Vec::new(),
                    writer,
                    false,
                )
                .await?,
            )
        };
        Ok(HandoffCompile {
            content,
            event_count: events.len(),
            update,
        })
    }

    async fn latest_handoff(
        &self,
        project: Option<&str>,
        session_id: Option<Id>,
    ) -> IndexResult<Option<MemoryItem>> {
        Ok(self
            .active_handoffs(project, session_id)
            .await?
            .into_iter()
            .next())
    }

    async fn active_handoffs(
        &self,
        project: Option<&str>,
        session_id: Option<Id>,
    ) -> IndexResult<Vec<MemoryItem>> {
        Ok(self
            .memory_repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await?
            .into_iter()
            .filter(|item| item.kind == MemoryKind::Handoff)
            .filter(|item| handoff_matches(item, project, session_id))
            .collect())
    }
}

fn handoff_scope(project: Option<String>, session_id: Option<Id>) -> MemoryScope {
    if let Some(session_id) = session_id {
        return MemoryScope::Session { session_id };
    }
    project
        .map(MemoryScope::project)
        .unwrap_or(MemoryScope::Global)
}

fn handoff_update_evidence(scope: &MemoryScope) -> EvidenceRef {
    let target = match scope {
        MemoryScope::Global => "handoff(action=update,scope=global)".to_string(),
        MemoryScope::Project { project_name, .. } => {
            format!("handoff(action=update,project={project_name})")
        }
        MemoryScope::Session { session_id } => {
            format!("handoff(action=update,session_id={session_id})")
        }
        _ => "handoff(action=update)".to_string(),
    };
    EvidenceRef::new(EvidenceKind::ToolCall, target)
        .with_summary("Rolling handoff content was provided through the handoff update API.")
}

fn handoff_matches(item: &MemoryItem, project: Option<&str>, session_id: Option<Id>) -> bool {
    match (&item.scope, session_id, project) {
        (
            MemoryScope::Session {
                session_id: item_session,
            },
            Some(session_id),
            _,
        ) => *item_session == session_id,
        (MemoryScope::Project { project_name, .. }, None, Some(project)) => {
            project_name.eq_ignore_ascii_case(project)
        }
        (MemoryScope::Global, None, None) => true,
        _ => false,
    }
}

fn normalize_handoff_content(content: &str, next_actions: &[String]) -> String {
    let mut content = content.trim().to_string();
    if !next_actions.is_empty() && !content.to_lowercase().contains("next action") {
        content.push_str("\n\n## Next Actions\n");
        for action in next_actions {
            content.push_str(&format!("- {}\n", action.trim()));
        }
    }
    content
}

fn compile_events_to_handoff(events: &[Event]) -> String {
    let mut lines = vec!["# Handoff".to_string(), String::new()];
    append_event_section(
        &mut lines,
        "Decisions",
        events,
        &[EventType::Decision, EventType::Rule, EventType::Preference],
    );
    append_event_section(
        &mut lines,
        "Progress",
        events,
        &[
            EventType::Milestone,
            EventType::Observation,
            EventType::Plan,
            EventType::ToolResult,
        ],
    );
    append_event_section(
        &mut lines,
        "Validation",
        events,
        &[EventType::Test, EventType::Command],
    );
    append_event_section(
        &mut lines,
        "Risks And Limitations",
        events,
        &[EventType::Error, EventType::Limitation],
    );
    lines.push("## Next Actions".to_string());
    let handoff_updates = events
        .iter()
        .filter(|event| event.event_type == EventType::HandoffUpdate)
        .collect::<Vec<_>>();
    if handoff_updates.is_empty() {
        lines.push(
            "- Review the latest session events and continue with the current plan.".to_string(),
        );
    } else {
        for event in handoff_updates {
            lines.push(format!("- {}", event.content.trim()));
        }
    }
    lines.join("\n")
}

fn append_event_section(
    lines: &mut Vec<String>,
    title: &str,
    events: &[Event],
    event_types: &[EventType],
) {
    lines.push(format!("## {title}"));
    let mut count = 0;
    for event in events
        .iter()
        .filter(|event| event_types.contains(&event.event_type))
    {
        lines.push(format!("- {}", event.content.trim()));
        count += 1;
    }
    if count == 0 {
        lines.push("- None recorded.".to_string());
    }
    lines.push(String::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::memory::{Harness, ModelIdentity};
    use engram_core::session::Session;
    use engram_store::{connect_and_init, StoreConfig};
    use time::{Duration, OffsetDateTime};

    async fn service() -> HandoffService {
        let db = connect_and_init(&StoreConfig::memory()).await.unwrap();
        let service = HandoffService::new(db);
        service.init_schema().await.unwrap();
        service
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    async fn seed_project_handoff(
        service: &HandoffService,
        project: &str,
        title: &str,
        updated_at: OffsetDateTime,
    ) -> MemoryItem {
        let mut item = MemoryItem::new(
            MemoryKind::Handoff,
            title,
            format!("# {title}"),
            MemoryScope::project(project),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_tag("handoff")
        .with_tag("rolling");
        item.created_at = updated_at;
        item.updated_at = updated_at;
        service.memory_repo.save_memory_item(&item).await.unwrap();
        item
    }

    #[tokio::test]
    async fn update_dry_run_does_not_write() {
        let service = service().await;
        let update = service
            .update(
                Some("engram".to_string()),
                None,
                "# Handoff".to_string(),
                vec!["Continue implementation".to_string()],
                writer(),
                true,
            )
            .await
            .unwrap();

        let get = service.get(Some("engram"), None).await.unwrap();

        assert!(update.dry_run);
        assert!(!update.written);
        assert!(update.item.content.contains("Next Actions"));
        assert!(update
            .item
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ToolCall
                && evidence.target.contains("project=engram")));
        assert!(get.item.is_none());
    }

    #[tokio::test]
    async fn update_dry_run_does_not_supersede_previous_handoff() {
        let service = service().await;
        let base = OffsetDateTime::now_utc();
        let older = seed_project_handoff(&service, "engram", "Older Handoff", base).await;
        let newest = seed_project_handoff(
            &service,
            "engram",
            "Newest Handoff",
            base + Duration::seconds(1),
        )
        .await;

        let update = service
            .update(
                Some("engram".to_string()),
                None,
                "# Planned Handoff".to_string(),
                vec!["Continue from the planned handoff".to_string()],
                writer(),
                true,
            )
            .await
            .unwrap();
        let stored_older = service
            .memory_repo
            .get_memory_item(&older.id)
            .await
            .unwrap()
            .unwrap();
        let stored_newest = service
            .memory_repo
            .get_memory_item(&newest.id)
            .await
            .unwrap()
            .unwrap();
        let active_items = service.active_handoffs(Some("engram"), None).await.unwrap();

        assert!(!update.written);
        assert_eq!(update.previous_id, Some(newest.id));
        assert_eq!(update.item.supersedes.len(), 2);
        assert!(update.item.supersedes.contains(&newest.id));
        assert!(update.item.supersedes.contains(&older.id));
        assert_eq!(stored_older.status, MemoryStatus::Active);
        assert_eq!(stored_newest.status, MemoryStatus::Active);
        assert_eq!(active_items.len(), 2);
    }

    #[tokio::test]
    async fn update_write_supersedes_all_previous_project_handoffs() {
        let service = service().await;
        let base = OffsetDateTime::now_utc();
        let older = seed_project_handoff(&service, "engram", "Older Handoff", base).await;
        let newest = seed_project_handoff(
            &service,
            "engram",
            "Newest Handoff",
            base + Duration::seconds(1),
        )
        .await;

        let update = service
            .update(
                Some("engram".to_string()),
                None,
                "# New Handoff".to_string(),
                vec!["Continue from the new handoff".to_string()],
                writer(),
                false,
            )
            .await
            .unwrap();
        let stored_older = service
            .memory_repo
            .get_memory_item(&older.id)
            .await
            .unwrap()
            .unwrap();
        let stored_newest = service
            .memory_repo
            .get_memory_item(&newest.id)
            .await
            .unwrap()
            .unwrap();
        let latest = service.get(Some("engram"), None).await.unwrap();

        assert!(update.written);
        assert_eq!(update.previous_id, Some(newest.id));
        assert_eq!(update.item.supersedes.len(), 2);
        assert!(update
            .item
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ToolCall
                && evidence.target.contains("project=engram")));
        assert!(update.item.supersedes.contains(&newest.id));
        assert!(update.item.supersedes.contains(&older.id));
        assert_eq!(stored_older.status, MemoryStatus::Superseded);
        assert_eq!(stored_newest.status, MemoryStatus::Superseded);
        assert!(stored_older
            .evidence
            .iter()
            .any(|evidence| evidence.target.contains(&update.item.id.to_string())));
        assert!(stored_newest
            .evidence
            .iter()
            .any(|evidence| evidence.target.contains(&update.item.id.to_string())));
        assert_eq!(latest.item.unwrap().id, update.item.id);
    }

    #[tokio::test]
    async fn update_write_leaves_other_project_handoff_active() {
        let service = service().await;
        let other_project = service
            .update(
                Some("other-project".to_string()),
                None,
                "# Other Project Handoff".to_string(),
                vec!["Continue other project".to_string()],
                writer(),
                false,
            )
            .await
            .unwrap()
            .item;

        let update = service
            .update(
                Some("engram".to_string()),
                None,
                "# Engram Handoff".to_string(),
                vec!["Continue Engram".to_string()],
                writer(),
                false,
            )
            .await
            .unwrap();
        let stored_other = service
            .memory_repo
            .get_memory_item(&other_project.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(update.previous_id, None);
        assert!(update.item.supersedes.is_empty());
        assert_eq!(stored_other.status, MemoryStatus::Active);
    }

    #[tokio::test]
    async fn compile_uses_session_events() {
        let service = service().await;
        let session = Session::new().with_project("engram");
        service.session_repo.save_session(&session).await.unwrap();
        service
            .session_repo
            .add_event(&Event::new(
                session.id,
                EventType::Decision,
                "agent",
                "Use a soft harness contract.",
            ))
            .await
            .unwrap();

        let compiled = service
            .compile(session.id, Some("engram".to_string()), writer(), true)
            .await
            .unwrap();

        assert!(compiled.content.contains("Use a soft harness contract."));
        assert!(compiled.content.contains("Next Actions"));
        assert!(compiled.update.is_none());
    }

    #[tokio::test]
    async fn compile_write_supersedes_previous_session_handoff() {
        let service = service().await;
        let session = Session::new().with_project("engram");
        service.session_repo.save_session(&session).await.unwrap();
        service
            .session_repo
            .add_event(&Event::new(
                session.id,
                EventType::HandoffUpdate,
                "agent",
                "Continue from compiled session context.",
            ))
            .await
            .unwrap();
        let previous = service
            .update(
                None,
                Some(session.id),
                "# Previous Session Handoff".to_string(),
                vec!["Continue from previous session handoff".to_string()],
                writer(),
                false,
            )
            .await
            .unwrap()
            .item;

        let compiled = service
            .compile(session.id, Some("engram".to_string()), writer(), false)
            .await
            .unwrap();
        let update = compiled.update.unwrap();
        let stored_previous = service
            .memory_repo
            .get_memory_item(&previous.id)
            .await
            .unwrap()
            .unwrap();
        let latest = service.get(None, Some(session.id)).await.unwrap();

        assert_eq!(update.previous_id, Some(previous.id));
        assert!(update.item.supersedes.contains(&previous.id));
        assert!(update
            .item
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ToolCall
                && evidence.target.contains(&session.id.to_string())));
        assert!(update
            .item
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::SessionEvent
                && evidence.target == session.id.to_string()));
        assert_eq!(stored_previous.status, MemoryStatus::Superseded);
        assert_eq!(latest.item.unwrap().id, update.item.id);
    }
}
