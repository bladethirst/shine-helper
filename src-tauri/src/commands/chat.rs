use crate::db::{Database, Message, Session};
use crate::openclaw::OpenClawClient;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::{State, Manager};

pub struct AppState {
    pub db: Mutex<Database>,
    pub openclaw_sessions: Mutex<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
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
pub async fn send_message_stream(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message: String,
) -> Result<(), CommandError> {
    let config = crate::config::load_config().unwrap_or_default();
    let token = crate::config::get_openclaw_token()
        .map_err(|e| CommandError { message: e.to_string() })?;
    let url = crate::config::get_openclaw_url();
    
    let mut client = OpenClawClient::new(&url);
    client.set_token(token);
    
    let mut full_response = String::new();
    
    let result = client.chat_stream(&message, Some(&session_id), |chunk| {
        full_response.push_str(&chunk);
        let _ = app_handle.emit_all("chat_chunk", StreamChunk {
            content: chunk,
            done: false,
        });
    }).await;
    
    match result {
        Ok(_) => {
            let _ = app_handle.emit_all("chat_chunk", StreamChunk {
                content: String::new(),
                done: true,
            });
            
            let db = state.db.lock().unwrap();
            db.add_message(&session_id, "assistant", &full_response).ok();
        }
        Err(e) => {
            let _ = app_handle.emit_all("chat_error", e.clone());
            return Err(CommandError { message: e });
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    session_id: String,
    message: String,
) -> Result<String, CommandError> {
    let config = crate::config::load_config().unwrap_or_default();
    let token = crate::config::get_openclaw_token()
        .map_err(|e| CommandError { message: e.to_string() })?;
    let url = crate::config::get_openclaw_url();
    
    let mut client = OpenClawClient::new(&url);
    client.set_token(token);
    
    let (response, _) = client.chat(&message, Some(&session_id)).await
        .map_err(|e| CommandError { message: e })?;
    
    {
        let db = state.db.lock().unwrap();
        db.add_message(&session_id, "assistant", &response).ok();
    }
    
    Ok(response)
}