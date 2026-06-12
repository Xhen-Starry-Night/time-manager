use tempfile::tempdir;
use time_manager::data::DataFs;
use time_manager::data::models::{Card, Preset};
use time_manager::obsidian::{TimeignoreRules, import_from_obsidian};

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

    std::fs::create_dir_all(vault_dir.path().join("语言/英语")).unwrap();
    std::fs::create_dir_all(vault_dir.path().join("数学")).unwrap();
    std::fs::create_dir_all(vault_dir.path().join(".obsidian")).unwrap();

    let rules = TimeignoreRules::default_rules();
    let result =
        import_from_obsidian(vault_dir.path(), "imported", data_dir.path(), &rules).unwrap();

    assert!(result.created_dirs.len() >= 2);
    assert!(
        data_dir
            .path()
            .join("categories/imported/语言/英语")
            .exists()
    );
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
