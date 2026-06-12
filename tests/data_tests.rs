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
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

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

    let card = Card::new("main/path/to/card".to_string());
    fs.save_card("main/path/to/card", &card).unwrap();

    let retrieved = fs.get_card("main/path/to/card").unwrap();
    assert_eq!(retrieved.path, "main/path/to/card");
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

    fs.save_card("study/card1", &Card::new("study/card1".to_string()))
        .unwrap();
    fs.save_card("study/card2", &Card::new("study/card2".to_string()))
        .unwrap();
    fs.save_card("study/sub/card3", &Card::new("study/sub/card3".to_string()))
        .unwrap();

    let cards = fs.list_cards("study").unwrap();
    assert_eq!(cards.len(), 3);
}

#[test]
fn test_save_and_retrieve_timer() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let timer = Timer::new(Utc::now());
    let filename = timer.filename();

    fs.save_timer(&timer).unwrap();
    let retrieved = fs.get_timer(&filename).unwrap();

    assert_eq!(retrieved.started_at, timer.started_at);
    assert_eq!(retrieved.duration_ms, 0);
}

#[test]
fn test_list_timers() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let timer1 = Timer::new(Utc::now());
    let timer2 = Timer::new(Utc::now() + chrono::Duration::hours(1));

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

    let id = uuid::Uuid::new_v4();
    let ics_content = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR";

    fs.save_schedule(&id, ics_content).unwrap();

    let retrieved = fs.get_schedule(&id).unwrap();
    assert!(retrieved.contains("VCALENDAR"));

    let schedules = fs.list_schedules().unwrap();
    assert_eq!(schedules.len(), 1);
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

    let mut card = Card::new("test/card".to_string());
    card.review_records.push(ReviewRecord {
        timer_path: "timer1".to_string(),
        reviewed_at: Utc::now(),
        memory_quality: MemoryQuality::Good,
        state_bytes: state_bytes.clone(),
        fsrs_state_bytes: state_bytes,
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

    fs.save_card("project/a", &Card::new("project/a".to_string()))
        .unwrap();
    fs.save_card("project/b", &Card::new("project/b".to_string()))
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
    let card = Card::new(deep_path.to_string());

    fs.save_card(deep_path, &card).unwrap();

    let retrieved = fs.get_card(deep_path).unwrap();
    assert_eq!(retrieved.path, deep_path);

    assert!(
        dir.path()
            .join("categories/deep/level1/level2/level3/level4/card.json")
            .exists()
    );
}

#[test]
fn test_card_with_prediction() {
    let card = Card::new_with_preset("test/path".to_string(), "vocabulary".to_string());

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
