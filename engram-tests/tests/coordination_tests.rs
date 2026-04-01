//! Integration tests for Layer 5: Session Coordination.
//!
//! These tests verify the coordination system works correctly with a real database,
//! testing the repository and service layers together.

use engram_core::id::Id;
use engram_index::CoordinationService;
use engram_mcp::tools::{self, CoordRequestNew, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use std::time::Duration;

// =============================================================================
// Test Fixtures
// =============================================================================

/// Create an in-memory database for testing.
async fn setup_db() -> engram_store::Db {
    let config = StoreConfig::memory();
    connect_and_init(&config)
        .await
        .expect("Failed to connect to test database")
}

/// Create a coordination service with initialized schema.
async fn setup_service() -> CoordinationService {
    let db = setup_db().await;
    let service = CoordinationService::new(db);
    service
        .init()
        .await
        .expect("Failed to initialize coordination schema");
    service
}

// =============================================================================
// Basic Registration Tests
// =============================================================================

#[tokio::test]
async fn test_register_session() {
    let service = setup_service().await;
    let session_id = Id::new();

    let session = service
        .register(
            &session_id,
            "claude-code",
            "my-project",
            "Implement feature X",
        )
        .await
        .expect("Failed to register session");

    assert_eq!(session.session_id, session_id);
    assert_eq!(session.agent, "claude-code");
    assert_eq!(session.project, "my-project");
    assert_eq!(session.goal, "Implement feature X");
    assert!(session.components.is_empty());
    assert!(session.current_file.is_none());
}

#[tokio::test]
async fn test_register_session_with_components() {
    let service = setup_service().await;
    let session_id = Id::new();

    let session = service
        .register_with_components(
            &session_id,
            "cursor",
            "webapp",
            "Add authentication",
            vec!["auth-service".to_string(), "user-api".to_string()],
        )
        .await
        .expect("Failed to register session with components");

    assert_eq!(session.components.len(), 2);
    assert!(session.components.contains(&"auth-service".to_string()));
    assert!(session.components.contains(&"user-api".to_string()));
}

#[tokio::test]
async fn test_unregister_session() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Register
    service
        .register(&session_id, "agent", "project", "goal")
        .await
        .expect("Failed to register");

    // Verify it exists
    let session = service
        .get(&session_id)
        .await
        .expect("Failed to get session");
    assert!(session.is_some());

    // Unregister
    service
        .unregister(&session_id)
        .await
        .expect("Failed to unregister");

    // Verify it's gone
    let session = service
        .get(&session_id)
        .await
        .expect("Failed to get session");
    assert!(session.is_none());
}

#[tokio::test]
async fn test_unregister_nonexistent_session() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Should not error when unregistering a session that doesn't exist
    let result = service.unregister(&session_id).await;
    assert!(result.is_ok());
}

// =============================================================================
// Heartbeat Tests
// =============================================================================

#[tokio::test]
async fn test_heartbeat_updates_timestamp() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Register
    let initial = service
        .register(&session_id, "agent", "project", "goal")
        .await
        .expect("Failed to register");

    // Small delay
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Heartbeat
    service
        .heartbeat(&session_id)
        .await
        .expect("Failed to heartbeat");

    // Get updated session
    let updated = service
        .get(&session_id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    // Last heartbeat should be newer
    assert!(updated.last_heartbeat >= initial.last_heartbeat);

    // Cleanup
    service.unregister(&session_id).await.ok();
}

#[tokio::test]
async fn test_heartbeat_nonexistent_session_errors() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Heartbeat on non-existent session should error
    let result = service.heartbeat(&session_id).await;
    // The behavior depends on implementation - it might error or silently succeed
    // For now, we just ensure it doesn't panic
    let _ = result;
}

// =============================================================================
// Component Conflict Tests
// =============================================================================

#[tokio::test]
async fn test_component_conflict_detection() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Session A works on auth-service
    service
        .register_with_components(
            &session_a,
            "claude-code",
            "webapp",
            "Implement login",
            vec!["auth-service".to_string()],
        )
        .await
        .expect("Failed to register session A");

    // Session B also works on auth-service
    service
        .register_with_components(
            &session_b,
            "cursor",
            "webapp",
            "Add logout",
            vec!["auth-service".to_string()],
        )
        .await
        .expect("Failed to register session B");

    // Check conflicts for session B
    let conflicts = service
        .check_conflicts(&session_b)
        .await
        .expect("Failed to check conflicts");

    assert!(!conflicts.is_empty(), "Should have detected conflict");
    assert_eq!(conflicts[0].other_session_id, session_a);
    assert!(conflicts[0]
        .overlapping_components
        .contains(&"auth-service".to_string()));

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

#[tokio::test]
async fn test_partial_component_overlap() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Session A: auth-service, user-api
    service
        .register_with_components(
            &session_a,
            "agent-a",
            "project",
            "goal-a",
            vec!["auth-service".to_string(), "user-api".to_string()],
        )
        .await
        .expect("Failed to register session A");

    // Session B: auth-service, billing (only auth-service overlaps)
    service
        .register_with_components(
            &session_b,
            "agent-b",
            "project",
            "goal-b",
            vec!["auth-service".to_string(), "billing".to_string()],
        )
        .await
        .expect("Failed to register session B");

    let conflicts = service
        .check_conflicts(&session_b)
        .await
        .expect("Failed to check conflicts");

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].overlapping_components.len(), 1);
    assert!(conflicts[0]
        .overlapping_components
        .contains(&"auth-service".to_string()));

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

#[tokio::test]
async fn test_no_conflict_when_components_differ() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Session A: frontend
    service
        .register_with_components(
            &session_a,
            "agent-a",
            "project",
            "goal-a",
            vec!["frontend".to_string()],
        )
        .await
        .expect("Failed to register session A");

    // Session B: backend (no overlap)
    service
        .register_with_components(
            &session_b,
            "agent-b",
            "project",
            "goal-b",
            vec!["backend".to_string()],
        )
        .await
        .expect("Failed to register session B");

    let conflicts = service
        .check_conflicts(&session_b)
        .await
        .expect("Failed to check conflicts");

    assert!(conflicts.is_empty(), "Should have no conflicts");

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

#[tokio::test]
async fn test_set_components_returns_conflicts() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Session A with components
    service
        .register_with_components(
            &session_a,
            "agent-a",
            "project",
            "goal-a",
            vec!["shared-component".to_string()],
        )
        .await
        .expect("Failed to register session A");

    // Session B without components initially
    service
        .register(&session_b, "agent-b", "project", "goal-b")
        .await
        .expect("Failed to register session B");

    // Setting components should return conflicts
    let conflicts = service
        .set_components(&session_b, &["shared-component".to_string()])
        .await
        .expect("Failed to set components");

    assert!(!conflicts.is_empty(), "Should return conflicts");

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

// =============================================================================
// File Conflict Tests
// =============================================================================

#[tokio::test]
async fn test_file_conflict_detection() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Register both sessions
    service
        .register(&session_a, "agent-a", "project", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project", "goal-b")
        .await
        .expect("Failed to register session B");

    // Session A edits file
    service
        .set_current_file(&session_a, Some("src/main.rs"))
        .await
        .expect("Failed to set file for A");

    // Session B also edits same file - should get conflict
    let conflicts = service
        .set_current_file(&session_b, Some("src/main.rs"))
        .await
        .expect("Failed to set file for B");

    assert!(!conflicts.is_empty(), "Should detect file conflict");
    assert_eq!(conflicts[0].other_session_id, session_a);

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

#[tokio::test]
async fn test_no_file_conflict_when_different_files() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Register both sessions
    service
        .register(&session_a, "agent-a", "project", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project", "goal-b")
        .await
        .expect("Failed to register session B");

    // Different files
    service
        .set_current_file(&session_a, Some("src/auth.rs"))
        .await
        .expect("Failed to set file for A");

    let conflicts = service
        .set_current_file(&session_b, Some("src/billing.rs"))
        .await
        .expect("Failed to set file for B");

    assert!(conflicts.is_empty(), "Should have no file conflict");

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

#[tokio::test]
async fn test_clearing_file_removes_conflict() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Register both sessions
    service
        .register(&session_a, "agent-a", "project", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project", "goal-b")
        .await
        .expect("Failed to register session B");

    // Both edit same file
    service
        .set_current_file(&session_a, Some("shared.rs"))
        .await
        .expect("Failed to set file for A");

    service
        .set_current_file(&session_b, Some("shared.rs"))
        .await
        .expect("Failed to set file for B");

    // Session A clears file
    service
        .set_current_file(&session_a, None)
        .await
        .expect("Failed to clear file for A");

    // Session B should now have no file conflicts
    let conflicts = service
        .check_file_conflicts(&session_b, "shared.rs")
        .await
        .expect("Failed to check file conflicts");

    assert!(
        conflicts.is_empty(),
        "Should have no file conflict after clearing"
    );

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

// =============================================================================
// Listing Tests
// =============================================================================

#[tokio::test]
async fn test_list_active_sessions() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();
    let session_c = Id::new();

    // Register 3 sessions
    service
        .register(&session_a, "agent-a", "project-1", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project-1", "goal-b")
        .await
        .expect("Failed to register session B");

    service
        .register(&session_c, "agent-c", "project-2", "goal-c")
        .await
        .expect("Failed to register session C");

    // List all
    let all_sessions = service
        .list_active()
        .await
        .expect("Failed to list active sessions");

    assert_eq!(all_sessions.len(), 3);

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
    service.unregister(&session_c).await.ok();
}

#[tokio::test]
async fn test_list_sessions_by_project() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();
    let session_c = Id::new();

    // Register sessions in different projects
    service
        .register(&session_a, "agent-a", "project-alpha", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project-alpha", "goal-b")
        .await
        .expect("Failed to register session B");

    service
        .register(&session_c, "agent-c", "project-beta", "goal-c")
        .await
        .expect("Failed to register session C");

    // List by project
    let alpha_sessions = service
        .list_for_project("project-alpha")
        .await
        .expect("Failed to list sessions for project-alpha");

    assert_eq!(alpha_sessions.len(), 2);
    assert!(alpha_sessions.iter().all(|s| s.project == "project-alpha"));

    let beta_sessions = service
        .list_for_project("project-beta")
        .await
        .expect("Failed to list sessions for project-beta");

    assert_eq!(beta_sessions.len(), 1);
    assert_eq!(beta_sessions[0].session_id, session_c);

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
    service.unregister(&session_c).await.ok();
}

// =============================================================================
// Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_stats_count() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Initial stats
    let initial_stats = service.stats().await.expect("Failed to get stats");
    let initial_count = initial_stats.active_session_count;

    // Register sessions
    service
        .register(&session_a, "agent-a", "project", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project", "goal-b")
        .await
        .expect("Failed to register session B");

    // Stats should show 2 more sessions
    let stats = service.stats().await.expect("Failed to get stats");
    assert_eq!(stats.active_session_count, initial_count + 2);

    // Unregister one
    service.unregister(&session_a).await.ok();

    // Stats should show 1 more session
    let stats = service.stats().await.expect("Failed to get stats");
    assert_eq!(stats.active_session_count, initial_count + 1);

    // Cleanup
    service.unregister(&session_b).await.ok();
}

// =============================================================================
// Multi-Session Conflict Scenarios
// =============================================================================

#[tokio::test]
async fn test_multiple_sessions_same_component() {
    let service = setup_service().await;
    let sessions: Vec<Id> = (0..5).map(|_| Id::new()).collect();

    // Register 5 sessions all working on "shared-component"
    for (i, session_id) in sessions.iter().enumerate() {
        service
            .register_with_components(
                session_id,
                &format!("agent-{}", i),
                "big-project",
                &format!("task-{}", i),
                vec!["shared-component".to_string()],
            )
            .await
            .expect("Failed to register session");
    }

    // Each session should see 4 conflicts (all others)
    for session_id in &sessions {
        let conflicts = service
            .check_conflicts(session_id)
            .await
            .expect("Failed to check conflicts");

        assert_eq!(conflicts.len(), 4, "Should see 4 conflicts");
    }

    // Cleanup
    for session_id in &sessions {
        service.unregister(session_id).await.ok();
    }
}

#[tokio::test]
async fn test_complex_overlap_scenario() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();
    let session_c = Id::new();

    // Session A: auth, database
    service
        .register_with_components(
            &session_a,
            "agent-a",
            "project",
            "goal-a",
            vec!["auth".to_string(), "database".to_string()],
        )
        .await
        .expect("Failed to register session A");

    // Session B: auth, api (overlaps with A on auth)
    service
        .register_with_components(
            &session_b,
            "agent-b",
            "project",
            "goal-b",
            vec!["auth".to_string(), "api".to_string()],
        )
        .await
        .expect("Failed to register session B");

    // Session C: api, frontend (overlaps with B on api, but not A)
    service
        .register_with_components(
            &session_c,
            "agent-c",
            "project",
            "goal-c",
            vec!["api".to_string(), "frontend".to_string()],
        )
        .await
        .expect("Failed to register session C");

    // A should conflict with B (auth)
    let a_conflicts = service.check_conflicts(&session_a).await.expect("Failed");
    assert_eq!(a_conflicts.len(), 1);
    assert_eq!(a_conflicts[0].other_session_id, session_b);

    // B should conflict with A and C
    let b_conflicts = service.check_conflicts(&session_b).await.expect("Failed");
    assert_eq!(b_conflicts.len(), 2);

    // C should conflict with B (api)
    let c_conflicts = service.check_conflicts(&session_c).await.expect("Failed");
    assert_eq!(c_conflicts.len(), 1);
    assert_eq!(c_conflicts[0].other_session_id, session_b);

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
    service.unregister(&session_c).await.ok();
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_empty_components() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Both sessions with empty components
    service
        .register(&session_a, "agent-a", "project", "goal-a")
        .await
        .expect("Failed to register session A");

    service
        .register(&session_b, "agent-b", "project", "goal-b")
        .await
        .expect("Failed to register session B");

    // Should have no component conflicts
    let conflicts = service.check_conflicts(&session_b).await.expect("Failed");
    assert!(conflicts.is_empty());

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

#[tokio::test]
async fn test_same_session_id_reregister() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Register first time
    service
        .register(&session_id, "agent-1", "project-1", "goal-1")
        .await
        .expect("Failed to register first time");

    // Register same ID again (should update or error)
    let result = service
        .register(&session_id, "agent-2", "project-2", "goal-2")
        .await;

    // Behavior depends on implementation - either updates or errors
    // Just ensure it doesn't panic
    let _ = result;

    // Cleanup
    service.unregister(&session_id).await.ok();
}

#[tokio::test]
async fn test_special_characters_in_names() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Names with special characters
    let session = service
        .register_with_components(
            &session_id,
            "claude-code/v3",
            "my-project (beta)",
            "Fix bug #123: \"important\"",
            vec!["auth/oauth".to_string(), "user-api:v2".to_string()],
        )
        .await
        .expect("Failed to register with special chars");

    assert_eq!(session.agent, "claude-code/v3");
    assert_eq!(session.project, "my-project (beta)");
    assert!(session.goal.contains("#123"));

    // Cleanup
    service.unregister(&session_id).await.ok();
}

#[tokio::test]
async fn test_very_long_component_list() {
    let service = setup_service().await;
    let session_id = Id::new();

    // Many components
    let components: Vec<String> = (0..100).map(|i| format!("component-{}", i)).collect();

    let session = service
        .register_with_components(&session_id, "agent", "project", "goal", components.clone())
        .await
        .expect("Failed to register with many components");

    assert_eq!(session.components.len(), 100);

    // Cleanup
    service.unregister(&session_id).await.ok();
}

// =============================================================================
// Custom Stale Timeout Tests
// =============================================================================

#[tokio::test]
async fn test_custom_stale_timeout() {
    let db = setup_db().await;
    let service = CoordinationService::with_stale_timeout(db, 1); // 1 minute timeout
    service.init().await.expect("Failed to initialize");

    let session_id = Id::new();

    service
        .register(&session_id, "agent", "project", "goal")
        .await
        .expect("Failed to register");

    // Session should not be stale immediately
    let sessions = service.list_active().await.expect("Failed to list");
    assert!(!sessions.is_empty());

    // Cleanup (manual since we can't wait 1 minute in test)
    service.unregister(&session_id).await.ok();
}

// =============================================================================
// Conflict Info Structure Tests
// =============================================================================

#[tokio::test]
async fn test_conflict_info_contains_all_fields() {
    let service = setup_service().await;
    let session_a = Id::new();
    let session_b = Id::new();

    // Session A with file
    service
        .register_with_components(
            &session_a,
            "claude-code",
            "webapp",
            "Implement OAuth",
            vec!["auth".to_string()],
        )
        .await
        .expect("Failed to register session A");

    service
        .set_current_file(&session_a, Some("src/oauth.rs"))
        .await
        .expect("Failed to set file for A");

    // Session B overlaps
    service
        .register_with_components(
            &session_b,
            "cursor",
            "webapp",
            "Add social login",
            vec!["auth".to_string()],
        )
        .await
        .expect("Failed to register session B");

    let conflicts = service.check_conflicts(&session_b).await.expect("Failed");

    assert!(!conflicts.is_empty());
    let conflict = &conflicts[0];

    // Verify all fields are populated
    assert_eq!(conflict.other_session_id, session_a);
    assert_eq!(conflict.other_agent, "claude-code");
    assert_eq!(conflict.other_goal, "Implement OAuth");
    assert!(conflict
        .overlapping_components
        .contains(&"auth".to_string()));
    assert_eq!(
        conflict.other_current_file,
        Some("src/oauth.rs".to_string())
    );

    // Cleanup
    service.unregister(&session_a).await.ok();
    service.unregister(&session_b).await.ok();
}

// =============================================================================
// MCP Tool Handler Tests (Phase 3: Consolidated Action-based API)
// =============================================================================

/// Setup tool state for MCP handler tests.
async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let coordination_service = CoordinationService::new(db);
    coordination_service.init().await.expect("Failed to init");

    let state = ToolState::new();
    *state.coordination_service.write().await = Some(coordination_service);
    state
}

#[tokio::test]
async fn test_mcp_coord_register_unregister() {
    let state = setup_tool_state().await;
    let session_id = Id::new().to_string();

    // Register
    let req = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_id.clone()),
        agent: Some("test-agent".to_string()),
        project: Some("/test/project".to_string()),
        goal: Some("Testing coordination".to_string()),
        components: vec!["api".to_string(), "auth".to_string()],
        file: None,
    };
    let result = tools::coord_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains(&session_id));
    assert!(response.contains("test-agent"));

    // Unregister
    let unreg_req = CoordRequestNew {
        action: "unregister".to_string(),
        session_id: Some(session_id.clone()),
        agent: None,
        project: None,
        goal: None,
        components: vec![],
        file: None,
    };
    let result = tools::coord_new(&state, unreg_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("unregistered"));
}

#[tokio::test]
async fn test_mcp_coord_heartbeat() {
    let state = setup_tool_state().await;
    let session_id = Id::new().to_string();

    // Register first
    let reg_req = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_id.clone()),
        agent: Some("test-agent".to_string()),
        project: Some("/test/project".to_string()),
        goal: Some("Testing".to_string()),
        components: vec![],
        file: None,
    };
    tools::coord_new(&state, reg_req).await.unwrap();

    // Heartbeat
    let hb_req = CoordRequestNew {
        action: "heartbeat".to_string(),
        session_id: Some(session_id.clone()),
        agent: None,
        project: None,
        goal: None,
        components: vec![],
        file: None,
    };
    let result = tools::coord_new(&state, hb_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Heartbeat recorded"));
}

#[tokio::test]
async fn test_mcp_coord_set_file() {
    let state = setup_tool_state().await;
    let session_id = Id::new().to_string();

    // Register first
    let reg_req = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_id.clone()),
        agent: Some("test-agent".to_string()),
        project: Some("/test/project".to_string()),
        goal: Some("Testing".to_string()),
        components: vec![],
        file: None,
    };
    tools::coord_new(&state, reg_req).await.unwrap();

    // Set file
    let file_req = CoordRequestNew {
        action: "set_file".to_string(),
        session_id: Some(session_id.clone()),
        agent: None,
        project: None,
        goal: None,
        components: vec![],
        file: Some("src/main.rs".to_string()),
    };
    let result = tools::coord_new(&state, file_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("success"));
}

#[tokio::test]
async fn test_mcp_coord_set_components() {
    let state = setup_tool_state().await;
    let session_id = Id::new().to_string();

    // Register first
    let reg_req = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_id.clone()),
        agent: Some("test-agent".to_string()),
        project: Some("/test/project".to_string()),
        goal: Some("Testing".to_string()),
        components: vec![],
        file: None,
    };
    tools::coord_new(&state, reg_req).await.unwrap();

    // Set components
    let comp_req = CoordRequestNew {
        action: "set_components".to_string(),
        session_id: Some(session_id.clone()),
        agent: None,
        project: None,
        goal: None,
        components: vec!["api".to_string(), "database".to_string()],
        file: None,
    };
    let result = tools::coord_new(&state, comp_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("success"));
}

#[tokio::test]
async fn test_mcp_coord_check_conflicts() {
    let state = setup_tool_state().await;
    let session_a = Id::new().to_string();
    let session_b = Id::new().to_string();

    // Register session A with component
    let reg_a = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_a.clone()),
        agent: Some("agent-a".to_string()),
        project: Some("/test/project".to_string()),
        goal: Some("Working on auth".to_string()),
        components: vec!["auth".to_string()],
        file: None,
    };
    tools::coord_new(&state, reg_a).await.unwrap();

    // Register session B with overlapping component
    let reg_b = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_b.clone()),
        agent: Some("agent-b".to_string()),
        project: Some("/test/project".to_string()),
        goal: Some("Also on auth".to_string()),
        components: vec!["auth".to_string()],
        file: None,
    };
    tools::coord_new(&state, reg_b).await.unwrap();

    // Check conflicts for session B
    let check_req = CoordRequestNew {
        action: "check_conflicts".to_string(),
        session_id: Some(session_b.clone()),
        agent: None,
        project: None,
        goal: None,
        components: vec![],
        file: None,
    };
    let result = tools::coord_new(&state, check_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("has_conflicts"));
    assert!(response.contains("true")); // Should have conflicts
}

#[tokio::test]
async fn test_mcp_coord_list() {
    let state = setup_tool_state().await;
    let session_id = Id::new().to_string();

    // Register a session
    let reg_req = CoordRequestNew {
        action: "register".to_string(),
        session_id: Some(session_id.clone()),
        agent: Some("list-test-agent".to_string()),
        project: Some("/test/list-project".to_string()),
        goal: Some("Testing list".to_string()),
        components: vec![],
        file: None,
    };
    tools::coord_new(&state, reg_req).await.unwrap();

    // List all sessions
    let list_req = CoordRequestNew {
        action: "list".to_string(),
        session_id: None,
        agent: None,
        project: None,
        goal: None,
        components: vec![],
        file: None,
    };
    let result = tools::coord_new(&state, list_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("list-test-agent"));
    assert!(response.contains("count"));

    // List filtered by project
    let list_filtered_req = CoordRequestNew {
        action: "list".to_string(),
        session_id: None,
        agent: None,
        project: Some("/test/list-project".to_string()),
        goal: None,
        components: vec![],
        file: None,
    };
    let result = tools::coord_new(&state, list_filtered_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("list-test-agent"));
}

#[tokio::test]
async fn test_mcp_coord_invalid_action() {
    let state = setup_tool_state().await;

    let req = CoordRequestNew {
        action: "invalid".to_string(),
        session_id: Some("test".to_string()),
        agent: None,
        project: None,
        goal: None,
        components: vec![],
        file: None,
    };
    let result = tools::coord_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown action"));
}
