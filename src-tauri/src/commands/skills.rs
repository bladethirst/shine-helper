use crate::skills::{Skill, SkillsManager};
use std::sync::Mutex;
use tauri::State;

pub struct SkillsState {
    pub manager: Mutex<SkillsManager>,
}

#[tauri::command]
pub fn get_local_skills(state: State<'_, SkillsState>) -> Result<Vec<Skill>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_local_skills())
}

#[tauri::command]
pub fn get_skills_dir(state: State<'_, SkillsState>) -> String {
    let manager = state.manager.lock().unwrap();
    manager.get_skills_dir_str()
}

#[tauri::command]
pub fn install_skill(
    state: State<'_, SkillsState>,
    skill_id: String,
    skill_name: String,
    skill_description: String,
    skill_version: String,
    skill_file_name: String,
    skill_data: Vec<u8>,
) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.install_skill_with_data(
        &skill_id,
        &skill_name,
        &skill_description,
        &skill_version,
        &skill_file_name,
        &skill_data,
    )
}

#[tauri::command]
pub fn uninstall_skill(state: State<'_, SkillsState>, skill_id: String) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.uninstall_skill(&skill_id)
}
