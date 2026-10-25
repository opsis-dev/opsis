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
