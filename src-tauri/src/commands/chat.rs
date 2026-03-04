use crate::db::{Database, Message, Session};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<rusqlite::Error> for CommandError {
    fn from(err: rusqlite::Error) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

#[tauri::command]
pub fn create_session(state: State<'_, AppState>, title: String) -> Result<Session, CommandError> {
    let db = state.db.lock().unwrap();
    let session = db.create_session(&title)?;
    Ok(session)
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, CommandError> {
    let db = state.db.lock().unwrap();
    let sessions = db.get_sessions()?;
    Ok(sessions)
}

#[tauri::command]
pub fn get_messages(state: State<'_, AppState>, session_id: String) -> Result<Vec<Message>, CommandError> {
    let db = state.db.lock().unwrap();
    let messages = db.get_messages(&session_id)?;
    Ok(messages)
}

#[tauri::command]
pub fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), CommandError> {
    let db = state.db.lock().unwrap();
    db.delete_session(&session_id)?;
    Ok(())
}

#[tauri::command]
pub fn add_message(
    state: State<'_, AppState>,
    session_id: String,
    role: String,
    content: String,
) -> Result<Message, CommandError> {
    let db = state.db.lock().unwrap();
    let message = db.add_message(&session_id, &role, &content)?;
    Ok(message)
}
