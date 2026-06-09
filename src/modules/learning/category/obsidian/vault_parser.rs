use std::path::Path;
use walkdir::WalkDir;
use tracing::{info, trace, warn};

pub struct VaultEntry {
    pub name: String,
    pub relative_path: String,
    pub is_dir: bool,
}

#[tracing::instrument(level = "info", skip(vault_path), fields(vault_path = %vault_path.display()))]
pub fn parse_vault(vault_path: &Path) -> Vec<VaultEntry> {
    info!("Parsing Obsidian vault");

    let mut entries = Vec::new();
    let mut dir_count = 0;
    let mut md_count = 0;

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
            trace!("Skipping hidden entry: {}", name);
            continue;
        }

        let relative = path
            .strip_prefix(vault_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if relative.is_empty() {
            trace!("Skipping empty relative path");
            continue;
        }

        let is_dir = entry.file_type().is_dir();

        if is_dir {
            trace!("Found directory: {}", relative);
            entries.push(VaultEntry {
                name,
                relative_path: relative,
                is_dir: true,
            });
            dir_count += 1;
        } else if is_markdown_file(&name) {
            let stem = name.trim_end_matches(".md");
            trace!("Found markdown file: {} -> {}", relative, stem);
            entries.push(VaultEntry {
                name: stem.to_string(),
                relative_path: relative.trim_end_matches(".md").to_string(),
                is_dir: false,
            });
            md_count += 1;
        } else {
            trace!("Skipping non-markdown file: {}", name);
        }
    }

    info!("Vault parsed: {} directories, {} markdown files, {} total entries", dir_count, md_count, entries.len());
    entries
}

fn is_markdown_file(name: &str) -> bool {
    name.to_lowercase().ends_with(".md")
}