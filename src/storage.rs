use crate::app_state::TOTPEntry;
use dirs::home_dir;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn load_entries() -> Result<HashMap<String, TOTPEntry>, Box<dyn std::error::Error>> {
    let path = get_config_file_path();
    if path.exists() {
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(HashMap::new())
    }
}

pub fn save_entries(
    entries: &HashMap<String, TOTPEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string(entries)?;
    let path = get_config_file_path();
    std::fs::write(path, data)?;
    Ok(())
}

pub fn backup_entries(
    entries: &HashMap<String, TOTPEntry>,
    path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string(entries)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn import_entries(
    path: PathBuf,
) -> Result<HashMap<String, TOTPEntry>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

fn get_config_file_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("authenticatorapp");
    std::fs::create_dir_all(&path).unwrap();
    path.push("entries.json");
    path
}
