use std::path::Path;
use walkdir::WalkDir;

pub struct TimeignoreRules {
    patterns: Vec<String>,
}

impl TimeignoreRules {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default_rules());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read .timeignore: {}", e))?;

        let patterns: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|s| s.to_string())
            .collect();

        Ok(Self { patterns })
    }

    pub fn default_rules() -> Self {
        Self {
            patterns: vec![
                ".obsidian/".to_string(),
                ".trash/".to_string(),
                ".git/".to_string(),
                ".templates/".to_string(),
                "attachments/".to_string(),
                "附件/".to_string(),
                "images/".to_string(),
                "*.tmp".to_string(),
                "*.bak".to_string(),
            ],
        }
    }

    pub fn should_ignore(&self, path: &str) -> bool {
        for pattern in &self.patterns {
            if Self::matches_pattern(path, pattern) {
                return true;
            }
        }
        false
    }

    fn matches_pattern(path: &str, pattern: &str) -> bool {
        let pattern = pattern.trim_end_matches('/');

        if let Some(prefix) = pattern.strip_suffix("/**") {
            return path.starts_with(prefix);
        }

        if let Some(prefix) = pattern.strip_suffix("/*") {
            if !path.starts_with(prefix) {
                return false;
            }
            let rest = &path[prefix.len()..];
            return !rest[1..].contains('/');
        }

        if let Some(suffix) = pattern.strip_prefix('*') {
            return path.ends_with(suffix);
        }

        if pattern.ends_with('/') {
            return path.starts_with(pattern) || path == &pattern[..pattern.len() - 1];
        }

        path.contains(pattern)
    }
}

pub struct ImportResult {
    pub created_dirs: Vec<String>,
    pub skipped_paths: Vec<String>,
}

pub fn import_from_obsidian(
    vault_path: &Path,
    tree_name: &str,
    data_dir: &Path,
    ignore_rules: &TimeignoreRules,
) -> Result<ImportResult, String> {
    if !vault_path.exists() {
        return Err(format!("Vault path does not exist: {:?}", vault_path));
    }

    let categories_dir = data_dir.join("categories").join(tree_name);
    std::fs::create_dir_all(&categories_dir)
        .map_err(|e| format!("Failed to create tree directory: {}", e))?;

    let mut created_dirs = Vec::new();
    let mut skipped_paths = Vec::new();

    for entry in WalkDir::new(vault_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(vault_path)
            .map_err(|_| "Failed to get relative path")?;

        let relative_str = relative.to_string_lossy();

        if ignore_rules.should_ignore(&relative_str) {
            skipped_paths.push(relative_str.to_string());
            continue;
        }

        if path.is_dir() {
            let target_dir = categories_dir.join(relative);
            std::fs::create_dir_all(&target_dir)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
            created_dirs.push(relative_str.to_string());
        }
    }

    Ok(ImportResult {
        created_dirs,
        skipped_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_rules() {
        let rules = TimeignoreRules::default_rules();
        assert!(rules.should_ignore(".obsidian/workspace.json"));
        assert!(rules.should_ignore(".git/config"));
        assert!(rules.should_ignore("test.tmp"));
        assert!(!rules.should_ignore("notes/english.md"));
    }

    #[test]
    fn test_pattern_matching() {
        let rules = TimeignoreRules {
            patterns: vec!["drafts/**".to_string()],
        };
        assert!(rules.should_ignore("drafts/note1.md"));
        assert!(rules.should_ignore("drafts/sub/note2.md"));
        assert!(!rules.should_ignore("notes/drafts.md"));
    }

    #[test]
    fn test_import_from_obsidian() {
        let vault_dir = tempdir().unwrap();
        let data_dir = tempdir().unwrap();

        std::fs::create_dir_all(vault_dir.path().join("语言/英语")).unwrap();
        std::fs::create_dir_all(vault_dir.path().join(".obsidian")).unwrap();
        std::fs::write(vault_dir.path().join("语言/英语/note.md"), "").unwrap();

        let rules = TimeignoreRules::default_rules();
        let result =
            import_from_obsidian(vault_dir.path(), "test", data_dir.path(), &rules).unwrap();

        assert!(result.created_dirs.contains(&"语言".to_string()));
        assert!(result.created_dirs.contains(&"语言/英语".to_string()));
        assert!(result.skipped_paths.iter().any(|p| p.contains(".obsidian")));

        assert!(data_dir.path().join("categories/test/语言/英语").exists());
    }
}
