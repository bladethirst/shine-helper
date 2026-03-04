use crate::db::{Database, Message, Session};
use crate::openclaw::OpenClawClient;
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

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    session_id: String,
    message: String,
) -> Result<String, CommandError> {
    // 构建 OpenClaw 请求
    let config = crate::config::load_config().unwrap_or_default();
    let mut client = OpenClawClient::new(&config.openclaw.url);
    
    // 调用 OpenClaw API
    let response = client.chat(&message).await
        .map_err(|e| CommandError { message: e })?;
    
    // 保存助手回复
    {
        let db = state.db.lock().unwrap();
        db.add_message(&session_id, "assistant", &response).ok();
    }
    
    Ok(response)
}
