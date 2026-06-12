use std::fs;
use std::path::Path;
use tempfile::tempdir;
use time_manager::data::models::{Card, Preset};
use time_manager::data::DataFs;
use time_manager::obsidian::{import_from_obsidian, TimeignoreRules};

#[test]
fn test_full_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let mut preset = Preset::default_preset();
    preset.name = "test".to_string();
    preset.match_rules = vec!["test/**".to_string()];
    fs.save_preset(&preset).unwrap();

    let card = Card::new_with_preset("test/card".to_string(), "test".to_string());
    fs.save_card("test/card", &card).unwrap();

    let loaded = fs.get_preset("test").unwrap();
    assert_eq!(loaded.match_rules.len(), 1);

    let loaded_card = fs.get_card("test/card").unwrap();
    assert!(loaded_card.prediction.is_some());
    assert_eq!(loaded_card.prediction.unwrap().preset_used, "test");
}

#[test]
fn test_obsidian_import_integration() {
    let vault_dir = tempdir().unwrap();
    let data_dir = tempdir().unwrap();

    fs::create_dir_all(vault_dir.path().join("语言/英语")).unwrap();
    fs::create_dir_all(vault_dir.path().join("数学")).unwrap();
    fs::create_dir_all(vault_dir.path().join(".obsidian")).unwrap();

    let rules = TimeignoreRules::default_rules();
    let result = import_from_obsidian(vault_dir.path(), "imported", data_dir.path(), &rules).unwrap();

    assert!(result.created_dirs.len() >= 2);
    assert!(data_dir.path().join("categories/imported/语言/英语").exists());
    assert!(data_dir.path().join("categories/imported/数学").exists());
}

#[test]
fn test_preset_training_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let mut preset = Preset::default_preset();
    preset.name = "vocab".to_string();
    preset.match_rules = vec!["study/vocab/**".to_string()];
    fs.save_preset(&preset).unwrap();

    let loaded = fs.get_preset("vocab").unwrap();
    assert!(loaded.fsrs_parameters.is_none());
    assert!(loaded.trained_at.is_none());
}

#[test]
fn test_timeignore_file_reading() {
    let dir = tempdir().unwrap();
    let ignore_file = dir.path().join(".timeignore");

    fs::write(&ignore_file, "drafts/\n# comment\n*.md\n").unwrap();

    let rules = TimeignoreRules::from_file(&ignore_file).unwrap();
    assert!(rules.should_ignore("drafts/note.txt"));
    assert!(rules.should_ignore("test.md"));
    assert!(!rules.should_ignore("notes/valid.txt"));
}

#[test]
fn test_timeignore_file_not_exists() {
    let dir = tempdir().unwrap();
    let non_existent = dir.path().join(".timeignore");

    let rules = TimeignoreRules::from_file(&non_existent).unwrap();
    assert!(rules.should_ignore(".obsidian/config"));
    assert!(rules.should_ignore(".git/HEAD"));
}

#[test]
fn test_import_empty_vault() {
    let vault_dir = tempdir().unwrap();
    let data_dir = tempdir().unwrap();

    let rules = TimeignoreRules::default_rules();
    let result = import_from_obsidian(vault_dir.path(), "empty", data_dir.path(), &rules).unwrap();

    assert_eq!(result.created_dirs.len(), 0);
    assert_eq!(result.skipped_paths.len(), 0);
}

#[test]
fn test_import_invalid_vault_path() {
    let data_dir = tempdir().unwrap();
    let rules = TimeignoreRules::default_rules();

    let result = import_from_obsidian(Path::new("/nonexistent/vault"), "invalid", data_dir.path(), &rules);

    assert!(result.is_err());
}

#[test]
fn test_collect_empty_training_data() {
    let cards: Vec<Card> = vec![];
    let items = time_manager::training::collect_training_data(&cards);
    assert_eq!(items.len(), 0);
}

#[test]
fn test_card_without_reviews() {
    let card = Card::new("test/path".to_string());
    let items = time_manager::training::convert_card_to_fsrs_items(&card);
    assert!(items.is_none());
}

#[test]
fn test_card_create_with_nonexistent_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let card = Card::new_with_preset("test/card".to_string(), "nonexistent".to_string());
    fs.save_card("test/card", &card).unwrap();

    let loaded = fs.get_card("test/card").unwrap();
    assert!(loaded.prediction.is_some());
    assert_eq!(loaded.prediction.unwrap().preset_used, "nonexistent");
}

#[test]
fn test_multiple_timeignore_patterns() {
    let rules = TimeignoreRules::default_rules();
    
    assert!(rules.should_ignore("attachments/file.pdf"));
    assert!(rules.should_ignore("附件/document.pdf"));
    assert!(rules.should_ignore("images/photo.jpg"));
    assert!(rules.should_ignore("backup.bak"));
    assert!(!rules.should_ignore("notes/english/vocab.md"));
}

#[test]
fn test_preset_with_trained_parameters() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let mut preset = Preset::default_preset();
    preset.name = "trained".to_string();
    preset.fsrs_parameters = Some(vec![1.0, 2.0, 3.0, 4.0]);
    preset.trained_at = Some(chrono::Utc::now());
    fs.save_preset(&preset).unwrap();

    let loaded = fs.get_preset("trained").unwrap();
    assert!(loaded.fsrs_parameters.is_some());
    assert!(loaded.trained_at.is_some());
}
