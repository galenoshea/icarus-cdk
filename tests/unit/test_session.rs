//! Unit tests for the session module

use icarus_core::session::{Session, SessionManager};
use candid::Principal;

#[test]
fn test_session_creation() {
    let principal = Principal::from_text("aaaaa-aa").unwrap();
    let session = Session::new("test-session".to_string(), principal);
    
    assert_eq!(session.id, "test-session");
    assert_eq!(session.principal, principal);
    assert!(session.created_at > 0);
    assert_eq!(session.last_activity, session.created_at);
}

#[test]
fn test_session_manager() {
    let mut manager = SessionManager::new();
    let principal = Principal::from_text("aaaaa-aa").unwrap();
    
    // Create session
    let session = manager.create_session(principal.clone());
    assert!(session.starts_with("session_"));
    
    // Get session
    let retrieved = manager.get_session(&session);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().principal, principal);
    
    // Validate session
    assert!(manager.validate_session(&session, principal));
    assert!(!manager.validate_session(&session, Principal::anonymous()));
    assert!(!manager.validate_session("invalid", principal));
}

#[test]
fn test_session_removal() {
    let mut manager = SessionManager::new();
    let principal = Principal::from_text("aaaaa-aa").unwrap();
    
    let session_id = manager.create_session(principal.clone());
    assert!(manager.get_session(&session_id).is_some());
    
    manager.remove_session(&session_id);
    assert!(manager.get_session(&session_id).is_none());
}

#[test]
fn test_session_update_activity() {
    let mut manager = SessionManager::new();
    let principal = Principal::from_text("aaaaa-aa").unwrap();
    
    let session_id = manager.create_session(principal);
    let initial_activity = manager.get_session(&session_id).unwrap().last_activity;
    
    // Simulate time passing (in a real scenario)
    manager.update_activity(&session_id);
    
    let updated_activity = manager.get_session(&session_id).unwrap().last_activity;
    assert!(updated_activity >= initial_activity);
}

#[test]
fn test_multiple_sessions() {
    let mut manager = SessionManager::new();
    let principal1 = Principal::from_text("aaaaa-aa").unwrap();
    let principal2 = Principal::from_text("aaaaa-ab").unwrap();
    
    let session1 = manager.create_session(principal1.clone());
    let session2 = manager.create_session(principal2.clone());
    
    assert_ne!(session1, session2);
    assert!(manager.validate_session(&session1, principal1));
    assert!(manager.validate_session(&session2, principal2));
    assert!(!manager.validate_session(&session1, principal2));
    assert!(!manager.validate_session(&session2, principal1));
}