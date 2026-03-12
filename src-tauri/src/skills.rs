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
    pub file_name: Option<String>,
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
    pub skills_dir: PathBuf,
    pub openclaw_skills_dir: Option<PathBuf>,
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

        // openclaw data/skills 目录
        let openclaw_skills_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|exe_dir| {
                exe_dir
                    .join("resources")
                    .join("openclaw")
                    .join("openclaw")
                    .join("data")
                    .join("skills")
            });

        if let Some(ref dir) = openclaw_skills_dir {
            fs::create_dir_all(dir).ok();
        }

        Self { skills_dir, openclaw_skills_dir }
    }

    pub fn get_skills_dir_str(&self) -> String {
        if let Some(ref dir) = self.openclaw_skills_dir {
            if dir.exists() {
                return dir.to_string_lossy().to_string();
            }
        }
        self.skills_dir.to_string_lossy().to_string()
    }

    pub fn get_local_skills(&self) -> Vec<Skill> {
        let mut skills = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let meta_path = path.join("skill.json");
                    if meta_path.is_file() {
                        if let Ok(content) = fs::read_to_string(&meta_path) {
                            if let Ok(skill) = serde_json::from_str::<Skill>(&content) {
                                skills.push(skill);
                            }
                        }
                    }
                }
            }
        }

        skills
    }

    pub fn install_skill_with_data(
        &self,
        skill_id: &str,
        skill_name: &str,
        skill_description: &str,
        skill_version: &str,
        skill_file_name: &str,
        skill_data: &[u8],
    ) -> Result<(), String> {
        let safe_id = skill_id.replace('/', "__");
        let skill_dir = self.skills_dir.join(&safe_id);
        fs::create_dir_all(&skill_dir).map_err(|e| e.to_string())?;

        // 写入本地管理目录
        fs::write(skill_dir.join(skill_file_name), skill_data).map_err(|e| e.to_string())?;

        // 同时写入 openclaw 的 skills 目录
        if let Some(ref oc_dir) = self.openclaw_skills_dir {
            fs::create_dir_all(oc_dir).ok();
            fs::write(oc_dir.join(skill_file_name), skill_data).ok();
        }

        // 写入元数据
        let skill = Skill {
            id: skill_id.to_string(),
            name: skill_name.to_string(),
            description: skill_description.to_string(),
            version: skill_version.to_string(),
            author: "Skills Market".to_string(),
            icon: None,
            installed: true,
            enabled: true,
            installed_version: Some(skill_version.to_string()),
            file_name: Some(skill_file_name.to_string()),
        };

        let content = serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())?;
        fs::write(skill_dir.join("skill.json"), content).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn uninstall_skill(&self, skill_id: &str) -> Result<(), String> {
        let safe_id = skill_id.replace('/', "__");
        let skill_dir = self.skills_dir.join(&safe_id);

        if !skill_dir.exists() {
            return Err("Skill not installed".to_string());
        }

        // 读取 file_name 以便从 openclaw 目录也删除
        let file_name = fs::read_to_string(skill_dir.join("skill.json"))
            .ok()
            .and_then(|c| serde_json::from_str::<Skill>(&c).ok())
            .and_then(|s| s.file_name);

        fs::remove_dir_all(&skill_dir).map_err(|e| e.to_string())?;

        // 同步从 openclaw 目录删除
        if let (Some(ref oc_dir), Some(fname)) = (&self.openclaw_skills_dir, file_name) {
            fs::remove_file(oc_dir.join(&fname)).ok();
        }

        Ok(())
    }

    pub fn is_installed(&self, skill_id: &str) -> bool {
        let safe_id = skill_id.replace('/', "__");
        self.skills_dir.join(&safe_id).join("skill.json").exists()
    }
}
