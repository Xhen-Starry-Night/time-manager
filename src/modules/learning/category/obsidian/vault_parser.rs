use std::path::Path;
use walkdir::WalkDir;

pub struct VaultEntry {
    pub name: String,
    pub relative_path: String,
    pub is_dir: bool,
}

pub fn parse_vault(vault_path: &Path) -> Vec<VaultEntry> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(vault_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if name.starts_with('.') {
            continue;
        }

        let relative = path
            .strip_prefix(vault_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if relative.is_empty() {
            continue;
        }

        let is_dir = entry.file_type().is_dir();

        if is_dir {
            entries.push(VaultEntry {
                name,
                relative_path: relative,
                is_dir: true,
            });
        } else if is_markdown_file(&name) {
            let stem = name.trim_end_matches(".md");
            entries.push(VaultEntry {
                name: stem.to_string(),
                relative_path: relative.trim_end_matches(".md").to_string(),
                is_dir: false,
            });
        }
    }
    entries
}

fn is_markdown_file(name: &str) -> bool {
    name.to_lowercase().ends_with(".md")
}