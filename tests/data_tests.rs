use chrono::Utc;
use tempfile::tempdir;
use time_manager::data::{
    DataFs,
    models::{Card, MemoryQuality, Preset, ReviewRecord, Timer, Todo},
};
use time_manager::fsrs::FsrsPredictor;

#[test]
fn test_init_creates_all_directories() {
    let dir = tempdir().unwrap();
    let _fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    assert!(dir.path().join("categories").exists());
    assert!(dir.path().join("timers").exists());
    assert!(dir.path().join("presets").exists());
    assert!(dir.path().join("todos").exists());
    assert!(dir.path().join("schedules").exists());
}

#[test]
fn test_create_and_list_trees() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("tree1").unwrap();
    fs.create_tree("tree2").unwrap();
    fs.create_tree("tree3").unwrap();

    let trees = fs.list_trees().unwrap();
    assert_eq!(trees.len(), 3);
    assert!(trees.contains(&"tree1".to_string()));
    assert!(trees.contains(&"tree2".to_string()));
    assert!(trees.contains(&"tree3".to_string()));
}

#[test]
fn test_save_and_retrieve_card() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();
    fs.create_tree("main").unwrap();

    let card = Card::new();
    fs.save_card("main/path/to/card", &card).unwrap();

    let retrieved = fs.get_card("main/path/to/card").unwrap();
    assert_eq!(retrieved.review_records.len(), 0);
}

#[test]
fn test_card_not_found_error() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let result = fs.get_card("nonexistent/card");
    assert!(result.is_err());
}

#[test]
fn test_list_cards_in_tree() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();
    fs.create_tree("study").unwrap();

    fs.save_card("study/card1", &Card::new())
        .unwrap();
    fs.save_card("study/card2", &Card::new())
        .unwrap();
    fs.save_card("study/sub/card3", &Card::new())
        .unwrap();

    let cards = fs.list_cards("study").unwrap();
    assert_eq!(cards.len(), 3);
}

#[test]
fn test_save_and_retrieve_timer() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let started = Utc::now();
    let stopped = started + chrono::Duration::seconds(90);
    let timer = Timer::new(started, stopped, 90000);
    let filename = timer.filename();

    fs.save_timer(&timer).unwrap();
    let retrieved = fs.get_timer(&filename).unwrap();

    assert_eq!(retrieved.started_at, timer.started_at);
    assert_eq!(retrieved.duration_ms, 90000);
}

#[test]
fn test_list_timers() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let now = Utc::now();
    let timer1 = Timer::new(now, now + chrono::Duration::seconds(60), 60000);
    let timer2 = Timer::new(now + chrono::Duration::hours(1), now + chrono::Duration::hours(2), 3600000);

    fs.save_timer(&timer1).unwrap();
    fs.save_timer(&timer2).unwrap();

    let timers = fs.list_timers().unwrap();
    assert_eq!(timers.len(), 2);
}

#[test]
fn test_save_and_retrieve_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = Preset {
        name: "custom".to_string(),
        description: Some("Custom preset".to_string()),
        match_rules: vec!["study/*".to_string()],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&preset).unwrap();
    let retrieved = fs.get_preset("custom").unwrap();

    assert_eq!(retrieved.name, "custom");
    assert_eq!(retrieved.match_rules.len(), 1);
}

#[test]
fn test_list_presets() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let p1 = Preset {
        name: "p1".to_string(),
        description: None,
        match_rules: vec![],
        fsrs_parameters: None,
        trained_at: None,
    };
    let p2 = Preset {
        name: "p2".to_string(),
        description: None,
        match_rules: vec![],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&p1).unwrap();
    fs.save_preset(&p2).unwrap();

    let presets = fs.list_presets().unwrap();
    assert!(presets.len() >= 3);
}

#[test]
fn test_preset_not_found() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let result = fs.get_preset("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_delete_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = Preset {
        name: "to_delete".to_string(),
        description: None,
        match_rules: vec![],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&preset).unwrap();
    assert!(fs.get_preset("to_delete").is_ok());

    fs.delete_preset("to_delete").unwrap();
    assert!(fs.get_preset("to_delete").is_err());
}

#[test]
fn test_rename_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = Preset {
        name: "old_name".to_string(),
        description: Some("test".to_string()),
        match_rules: vec!["study/*".to_string()],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&preset).unwrap();
    fs.rename_preset("old_name", "new_name").unwrap();

    assert!(fs.get_preset("old_name").is_err());
    let renamed = fs.get_preset("new_name").unwrap();
    assert_eq!(renamed.name, "new_name");
    assert_eq!(renamed.description, Some("test".to_string()));
    assert_eq!(renamed.match_rules, vec!["study/*".to_string()]);
}

#[test]
fn test_preset_crud_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    // create
    let p1 = Preset {
        name: "english".to_string(),
        description: Some("英语高频词".to_string()),
        match_rules: vec!["main/english/**".to_string()],
        fsrs_parameters: None,
        trained_at: None,
    };
    fs.save_preset(&p1).unwrap();
    let presets = fs.list_presets().unwrap();
    assert!(presets.iter().any(|p| p.name == "english"));

    // rename
    fs.rename_preset("english", "english-v2").unwrap();
    let presets = fs.list_presets().unwrap();
    assert!(presets.iter().any(|p| p.name == "english-v2"));
    assert!(!presets.iter().any(|p| p.name == "english"));

    // delete
    fs.delete_preset("english-v2").unwrap();
    let presets = fs.list_presets().unwrap();
    assert!(!presets.iter().any(|p| p.name == "english-v2"));
}

#[test]
fn test_todo_crud() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let todo = Todo::new("Test task".to_string());
    let id = todo.id;

    fs.save_todo(&todo).unwrap();

    let retrieved = fs.get_todo(&id).unwrap();
    assert_eq!(retrieved.content, "Test task");

    let todos = fs.list_todos().unwrap();
    assert_eq!(todos.len(), 1);

    fs.delete_todo(&id).unwrap();
    let todos_after = fs.list_todos().unwrap();
    assert_eq!(todos_after.len(), 0);
}

#[test]
fn test_schedule_operations() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let schedule = time_manager::data::models::Schedule {
        id: uuid::Uuid::new_v4(),
        summary: "测试".to_string(),
        dtstart: chrono::DateTime::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 14).unwrap().and_hms_opt(14, 0, 0).unwrap(),
            chrono::Utc,
        ),
        dtend: chrono::DateTime::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 14).unwrap().and_hms_opt(15, 0, 0).unwrap(),
            chrono::Utc,
        ),
        description: None,
        location: None,
        categories: Vec::new(),
        priority: None,
        rrule: time_manager::data::models::RecurrenceRule::None,
        reminder_minutes: None,
    };

    fs.save_schedule(&schedule).unwrap();

    let retrieved = fs.get_schedule(&schedule.id).unwrap();
    assert_eq!(retrieved.summary, "测试");

    let schedules = fs.list_schedules().unwrap();
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].summary, "测试");
}

#[test]
fn test_card_with_review_records() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();
    fs.create_tree("test").unwrap();

    let predictor = FsrsPredictor::new().unwrap();
    let (_, state) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();
    let state_bytes = FsrsPredictor::memory_state_to_bytes(&state);

    let mut card = Card::new();
    card.prediction = Some(time_manager::data::models::Prediction {
        algorithm: "fsrs".to_string(),
        next_review: Utc::now() + chrono::Duration::days(7),
        fsrs_state_bytes: state_bytes,
        preset_used: "default".to_string(),
    });
    card.review_records.push(ReviewRecord {
        timestamp: Utc::now(),
        duration_ms: 1800000,
        memory_quality: MemoryQuality::Good,
    });

    fs.save_card("test/card", &card).unwrap();

    let retrieved = fs.get_card("test/card").unwrap();
    assert_eq!(retrieved.review_records.len(), 1);
}

#[test]
fn test_multiple_operations_sequence() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("project").unwrap();

    fs.save_card("project/a", &Card::new())
        .unwrap();
    fs.save_card("project/b", &Card::new())
        .unwrap();

    let todo1 = Todo::new("Task 1".to_string());
    let todo2 = Todo::new("Task 2".to_string());
    fs.save_todo(&todo1).unwrap();
    fs.save_todo(&todo2).unwrap();

    let cards = fs.list_cards("project").unwrap();
    assert_eq!(cards.len(), 2);

    let todos = fs.list_todos().unwrap();
    assert_eq!(todos.len(), 2);

    fs.delete_todo(&todo1.id).unwrap();
    let todos_after = fs.list_todos().unwrap();
    assert_eq!(todos_after.len(), 1);
}

#[test]
fn test_deep_nested_card_path() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();
    fs.create_tree("deep").unwrap();

    let deep_path = "deep/level1/level2/level3/level4/card";
    let card = Card::new();

    fs.save_card(deep_path, &card).unwrap();

    let _retrieved = fs.get_card(deep_path).unwrap();

    assert!(
        dir.path()
            .join("categories/deep/level1/level2/level3/level4/card.json")
            .exists()
    );
}

#[test]
fn test_card_with_prediction() {
    let card = Card::new_with_preset("vocabulary".to_string());

    assert!(card.prediction.is_some());
    let pred = card.prediction.unwrap();
    assert_eq!(pred.preset_used, "vocabulary");
    assert_eq!(pred.algorithm, "fsrs");
}

#[test]
fn test_preset_with_parameters() {
    let mut preset = Preset::default_preset();
    preset.fsrs_parameters = Some(vec![1.0, 2.0, 3.0]);
    preset.match_rules = vec!["main/语言/**".to_string()];

    assert!(preset.fsrs_parameters.is_some());
    assert_eq!(preset.match_rules.len(), 1);
}

#[test]
fn test_schedule_ics_roundtrip() {
    use time_manager::data::schedule::{format_ics, parse_ics};
    use time_manager::data::models::{RecurrenceRule, Schedule};
    use uuid::Uuid;

    let dt = |y: i32, m: u32, d: u32, h: u32, min: u32| {
        chrono::DateTime::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_opt(h, min, 0).unwrap(),
            chrono::Utc,
        )
    };
    let schedule = Schedule {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        summary: "测试日程".to_string(),
        dtstart: dt(2026, 6, 14, 14, 0),
        dtend: dt(2026, 6, 14, 15, 0),
        description: Some("描述内容".to_string()),
        location: Some("图书馆".to_string()),
        categories: vec!["学习".to_string(), "Rust".to_string()],
        priority: Some(5),
        rrule: RecurrenceRule::Weekly,
        reminder_minutes: Some(15),
    };

    let ics = format_ics(&schedule);
    let parsed = parse_ics(&ics).expect("Failed to parse generated ICS");

    assert_eq!(parsed.id, schedule.id);
    assert_eq!(parsed.summary, schedule.summary);
    assert_eq!(parsed.dtstart, schedule.dtstart);
    assert_eq!(parsed.dtend, schedule.dtend);
    assert_eq!(parsed.description, schedule.description);
    assert_eq!(parsed.location, schedule.location);
    assert_eq!(parsed.categories, schedule.categories);
    assert_eq!(parsed.priority, schedule.priority);
    assert_eq!(parsed.rrule, schedule.rrule);
    assert_eq!(parsed.reminder_minutes, schedule.reminder_minutes);
}

#[test]
fn test_parse_invalid_ics() {
    use time_manager::data::schedule::parse_ics;

    assert!(parse_ics("").is_err());
    assert!(parse_ics("BEGIN:VCALENDAR\nEND:VCALENDAR").is_err());
    assert!(parse_ics("INVALID").is_err());
}

#[test]
fn test_config_load_save() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let config = time_manager::data::config::Config {
        data_dir: Some("/tmp/data".to_string()),
        default_preset: Some("english".to_string()),
        theme: None,
    };
    config.save(&config_path).unwrap();

    let loaded = time_manager::data::config::Config::load(&config_path).unwrap();
    assert_eq!(loaded.data_dir, Some("/tmp/data".to_string()));
    assert_eq!(loaded.default_preset, Some("english".to_string()));
}

#[test]
fn test_config_partial() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    std::fs::write(&config_path, "default_preset = \"math\"").unwrap();
    let loaded = time_manager::data::config::Config::load(&config_path).unwrap();
    assert!(loaded.data_dir.is_none());
    assert_eq!(loaded.default_preset, Some("math".to_string()));
}

#[test]
fn test_config_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("nonexistent.toml");

    let loaded = time_manager::data::config::Config::load(&config_path).unwrap();
    assert!(loaded.data_dir.is_none());
    assert!(loaded.default_preset.is_none());
}
