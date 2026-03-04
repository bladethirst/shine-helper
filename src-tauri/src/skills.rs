use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub icon: Option<String>,
    pub installed: bool,
    pub enabled: bool,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub icon: Option<String>,
    pub download_url: String,
}

pub struct SkillsManager {
    skills_dir: PathBuf,
}

impl SkillsManager {
    pub fn new() -> Self {
        let skills_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shine_helper")
            .join("skills");
            
        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir).ok();
        }
        
        Self { skills_dir }
    }

    pub fn get_local_skills(&self) -> Vec<Skill> {
        let mut skills = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(meta) = fs::metadata(path.join("skill.json")) {
                        if meta.is_file() {
                            if let Ok(content) = fs::read_to_string(path.join("skill.json")) {
                                if let Ok(skill) = serde_json::from_str::<Skill>(&content) {
                                    skills.push(skill);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        skills
    }

    pub fn install_skill(&self, market_skill: &MarketSkill) -> Result<(), String> {
        let skill_dir = self.skills_dir.join(&market_skill.id);
        
        if skill_dir.exists() {
            return Err("Skill already installed".to_string());
        }
        
        fs::create_dir_all(&skill_dir).map_err(|e| e.to_string())?;
        
        let skill = Skill {
            id: market_skill.id.clone(),
            name: market_skill.name.clone(),
            description: market_skill.description.clone(),
            version: market_skill.version.clone(),
            author: market_skill.author.clone(),
            icon: market_skill.icon.clone(),
            installed: true,
            enabled: true,
            installed_version: Some(market_skill.version.clone()),
        };
        
        let content = serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())?;
        fs::write(skill_dir.join("skill.json"), content).map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub fn uninstall_skill(&self, skill_id: &str) -> Result<(), String> {
        let skill_dir = self.skills_dir.join(skill_id);
        
        if !skill_dir.exists() {
            return Err("Skill not installed".to_string());
        }
        
        fs::remove_dir_all(&skill_dir).map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
