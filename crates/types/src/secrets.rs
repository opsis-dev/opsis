pub const SECRETS_DIR: &str = "secrets";
pub const SECRET_EXT: &str = ".json.enc";

pub fn secret_name(raw: &str) -> String {
    match raw.trim() {
        "" => "default".to_owned(),
        name => name.to_owned(),
    }
}

pub fn secret_file_name(raw: &str) -> String {
    let name = slug::slugify(secret_name(raw));
    if name.is_empty() {
        "default".to_owned()
    } else {
        name
    }
}

pub fn secret_file(project_dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    project_dir
        .join(SECRETS_DIR)
        .join(format!("{}{SECRET_EXT}", secret_file_name(name)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: f64,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SavedItem {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedOrigin {
    pub origin: String,
    pub local_storage: Vec<SavedItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct StorageState {
    pub cookies: Vec<SavedCookie>,
    pub origins: Vec<SavedOrigin>,
}
