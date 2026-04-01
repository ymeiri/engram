//! Integration tests for Layer 2: Session History.
//!
//! Tests session tracking, events, decisions, and rationale.

use engram_core::session::{EventType, SessionStatus};
use engram_index::SessionService;
use engram_store::{connect_and_init, StoreConfig};

// =============================================================================
// Test Fixtures
// =============================================================================

async fn setup_service() -> SessionService {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let service = SessionService::new(db);
    service.init().await.expect("Failed to init");
    service
}

// =============================================================================
// Session Lifecycle Tests
// =============================================================================

#[tokio::test]
async fn test_start_session() {
    let service = setup_service().await;

    let session = service
        .start_session(
            Some("claude-code"),
            Some("my-project"),
            Some("Implement auth"),
        )
        .await
        .expect("Failed to start session");

    assert_eq!(session.agent, Some("claude-code".to_string()));
    assert_eq!(session.project, Some("my-project".to_string()));
    assert_eq!(session.goal, Some("Implement auth".to_string()));
    assert_eq!(session.status, SessionStatus::Active);
}

#[tokio::test]
async fn test_start_minimal_session() {
    let service = setup_service().await;

    let session = service
        .start_session(None, None, None)
        .await
        .expect("Failed to start session");

    assert!(session.agent.is_none());
    assert!(session.project.is_none());
    assert!(session.goal.is_none());
    assert_eq!(session.status, SessionStatus::Active);
}

#[tokio::test]
async fn test_end_session() {
    let service = setup_service().await;

    let session = service
        .start_session(Some("agent"), Some("project"), Some("goal"))
        .await
        .unwrap();

    service
        .end_session(&session.id, Some("Completed the feature"))
        .await
        .expect("Failed to end session");

    let ended = service.get_session(&session.id).await.unwrap().unwrap();

    assert_eq!(ended.status, SessionStatus::Completed);
    assert_eq!(ended.summary, Some("Completed the feature".to_string()));
}

#[tokio::test]
async fn test_end_session_without_summary() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    service
        .end_session(&session.id, None)
        .await
        .expect("Failed to end session");

    let ended = service.get_session(&session.id).await.unwrap().unwrap();

    assert_eq!(ended.status, SessionStatus::Completed);
    assert!(ended.summary.is_none());
}

// =============================================================================
// Event Logging Tests
// =============================================================================

#[tokio::test]
async fn test_log_event() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    let event = service
        .log_event(
            &session.id,
            EventType::Decision,
            "Chose OAuth over API keys",
            Some("Better security"),
            Some("design-review"),
        )
        .await
        .expect("Failed to log event");

    assert_eq!(event.event_type, EventType::Decision);
    assert_eq!(event.content, "Chose OAuth over API keys");
    assert_eq!(event.context, Some("Better security".to_string()));
    assert_eq!(event.source, Some("design-review".to_string()));
}

#[tokio::test]
async fn test_all_event_types() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    let event_types = [
        (EventType::Decision, "Made a decision"),
        (EventType::Command, "Ran a command"),
        (EventType::FileChange, "Changed a file"),
        (EventType::ToolUse, "Used a tool"),
        (EventType::Error, "Got an error"),
        (EventType::Milestone, "Reached milestone"),
        (EventType::Observation, "Observed something"),
    ];

    for (event_type, content) in event_types {
        let event = service
            .log_event(&session.id, event_type.clone(), content, None, None)
            .await
            .expect(&format!("Failed to log {:?}", event_type));

        assert_eq!(event.event_type, event_type);
    }
}

#[tokio::test]
async fn test_custom_event_type() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    let event = service
        .log_event(
            &session.id,
            EventType::Custom("my-custom-type".to_string()),
            "Custom event content",
            None,
            None,
        )
        .await
        .expect("Failed to log custom event");

    match event.event_type {
        EventType::Custom(ref t) => assert_eq!(t, "my-custom-type"),
        _ => panic!("Expected custom event type"),
    }
}

#[tokio::test]
async fn test_get_session_with_events() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    // Log multiple events
    service
        .log_event(&session.id, EventType::Observation, "First", None, None)
        .await
        .unwrap();
    service
        .log_event(&session.id, EventType::Decision, "Second", None, None)
        .await
        .unwrap();
    service
        .log_event(&session.id, EventType::Milestone, "Third", None, None)
        .await
        .unwrap();

    let (_, events) = service
        .get_session_with_events(&session.id)
        .await
        .expect("Failed to get session with events");

    assert_eq!(events.len(), 3);
}

// =============================================================================
// Session Listing Tests
// =============================================================================

#[tokio::test]
async fn test_list_sessions() {
    let service = setup_service().await;

    service
        .start_session(Some("agent1"), Some("project1"), None)
        .await
        .unwrap();
    service
        .start_session(Some("agent2"), Some("project1"), None)
        .await
        .unwrap();
    service
        .start_session(Some("agent1"), Some("project2"), None)
        .await
        .unwrap();

    // List all
    let all = service
        .list_sessions(None, None, None, None)
        .await
        .expect("Failed to list");
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_list_sessions_by_status() {
    let service = setup_service().await;

    let s1 = service.start_session(None, None, None).await.unwrap();
    let _s2 = service.start_session(None, None, None).await.unwrap();
    service.end_session(&s1.id, None).await.unwrap();

    let active = service
        .list_sessions(Some(&SessionStatus::Active), None, None, None)
        .await
        .expect("Failed to list");
    assert_eq!(active.len(), 1);

    let completed = service
        .list_sessions(Some(&SessionStatus::Completed), None, None, None)
        .await
        .expect("Failed to list");
    assert_eq!(completed.len(), 1);
}

#[tokio::test]
async fn test_list_sessions_by_agent() {
    let service = setup_service().await;

    service
        .start_session(Some("claude"), None, None)
        .await
        .unwrap();
    service
        .start_session(Some("claude"), None, None)
        .await
        .unwrap();
    service
        .start_session(Some("cursor"), None, None)
        .await
        .unwrap();

    let claude_sessions = service
        .list_sessions(None, Some("claude"), None, None)
        .await
        .expect("Failed to list");
    assert_eq!(claude_sessions.len(), 2);
}

#[tokio::test]
async fn test_list_sessions_by_project() {
    let service = setup_service().await;

    service
        .start_session(None, Some("webapp"), None)
        .await
        .unwrap();
    service
        .start_session(None, Some("webapp"), None)
        .await
        .unwrap();
    service
        .start_session(None, Some("mobile"), None)
        .await
        .unwrap();

    let webapp_sessions = service
        .list_sessions(None, None, Some("webapp"), None)
        .await
        .expect("Failed to list");
    assert_eq!(webapp_sessions.len(), 2);
}

#[tokio::test]
async fn test_list_sessions_with_limit() {
    let service = setup_service().await;

    for i in 0..10 {
        service
            .start_session(Some(&format!("agent-{}", i)), None, None)
            .await
            .unwrap();
    }

    let limited = service
        .list_sessions(None, None, None, Some(5))
        .await
        .expect("Failed to list");
    assert_eq!(limited.len(), 5);
}

#[tokio::test]
async fn test_get_active_sessions() {
    let service = setup_service().await;

    let s1 = service.start_session(None, None, None).await.unwrap();
    let _s2 = service.start_session(None, None, None).await.unwrap();
    let _s3 = service.start_session(None, None, None).await.unwrap();

    // End one
    service.end_session(&s1.id, None).await.unwrap();

    let active = service
        .get_active_sessions(None)
        .await
        .expect("Failed to get active");
    assert_eq!(active.len(), 2);
}

// =============================================================================
// Event Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_events() {
    let service = setup_service().await;

    let s1 = service.start_session(None, None, None).await.unwrap();
    let s2 = service.start_session(None, None, None).await.unwrap();

    // Log events in both sessions
    service
        .log_event(
            &s1.id,
            EventType::Decision,
            "Chose OAuth for authentication",
            None,
            None,
        )
        .await
        .unwrap();
    service
        .log_event(
            &s1.id,
            EventType::Observation,
            "Found caching issue",
            None,
            None,
        )
        .await
        .unwrap();
    service
        .log_event(
            &s2.id,
            EventType::Decision,
            "Decided to use OAuth library",
            None,
            None,
        )
        .await
        .unwrap();

    // Search across sessions
    let results = service
        .search_events("OAuth", None)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.content.contains("OAuth")));
}

#[tokio::test]
async fn test_search_events_with_limit() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    for i in 0..20 {
        service
            .log_event(
                &session.id,
                EventType::Observation,
                &format!("Observation number {}", i),
                None,
                None,
            )
            .await
            .unwrap();
    }

    let results = service
        .search_events("Observation", Some(5))
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 5);
}

// =============================================================================
// Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_session_stats() {
    let service = setup_service().await;

    let s1 = service.start_session(None, None, None).await.unwrap();
    let s2 = service.start_session(None, None, None).await.unwrap();
    let _s3 = service.start_session(None, None, None).await.unwrap();

    // End one, abandon one
    service.end_session(&s1.id, None).await.unwrap();

    // Log some events
    service
        .log_event(&s2.id, EventType::Decision, "Decision 1", None, None)
        .await
        .unwrap();
    service
        .log_event(&s2.id, EventType::Decision, "Decision 2", None, None)
        .await
        .unwrap();
    service
        .log_event(&s2.id, EventType::Error, "Error 1", None, None)
        .await
        .unwrap();

    let stats = service.stats().await.expect("Failed to get stats");

    assert_eq!(stats.total_sessions, 3);
    assert_eq!(stats.completed_sessions, 1);
    assert_eq!(stats.active_sessions, 2);
    assert_eq!(stats.total_events, 3);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_session_not_found() {
    let service = setup_service().await;

    let fake_id = engram_core::id::Id::new();
    let result = service.get_session(&fake_id).await.expect("Query failed");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_log_event_to_ended_session() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();
    service.end_session(&session.id, None).await.unwrap();

    // Should still be able to log events (implementation dependent)
    let result = service
        .log_event(
            &session.id,
            EventType::Observation,
            "After ended",
            None,
            None,
        )
        .await;

    // Just ensure it doesn't panic
    let _ = result;
}

#[tokio::test]
async fn test_long_event_content() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    let long_content = "X".repeat(50000);

    let event = service
        .log_event(
            &session.id,
            EventType::Observation,
            &long_content,
            None,
            None,
        )
        .await
        .expect("Failed with long content");

    assert_eq!(event.content.len(), 50000);
}

#[tokio::test]
async fn test_special_characters_in_content() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    let special_content = r#"Contains "quotes", 'single quotes', newlines
and tabs	and special chars: <>&"#;

    let event = service
        .log_event(
            &session.id,
            EventType::Observation,
            special_content,
            None,
            None,
        )
        .await
        .expect("Failed with special chars");

    assert_eq!(event.content, special_content);
}

#[tokio::test]
async fn test_many_events_in_session() {
    let service = setup_service().await;

    let session = service.start_session(None, None, None).await.unwrap();

    // Log 100 events
    for i in 0..100 {
        service
            .log_event(
                &session.id,
                EventType::Observation,
                &format!("Event {}", i),
                None,
                None,
            )
            .await
            .expect(&format!("Failed to log event {}", i));
    }

    let (_, events) = service.get_session_with_events(&session.id).await.unwrap();
    assert_eq!(events.len(), 100);
}
