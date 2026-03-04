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
pub fn install_skill(state: State<'_, SkillsState>, skill_id: String) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    // 模拟安装 - 实际应从市场API获取
    let market_skill = crate::skills::MarketSkill {
        id: skill_id.clone(),
        name: skill_id.clone(),
        description: "从市场安装的Skill".to_string(),
        version: "1.0.0".to_string(),
        author: "官方".to_string(),
        icon: None,
        download_url: "".to_string(),
    };
    manager.install_skill(&market_skill)
}

#[tauri::command]
pub fn uninstall_skill(state: State<'_, SkillsState>, skill_id: String) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.uninstall_skill(&skill_id)
}
