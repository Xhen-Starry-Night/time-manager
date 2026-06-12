use chrono::Utc;
use tempfile::tempdir;
use time_manager::data::{
    DataFs,
    models::{Card, MemoryQuality, ReviewRecord},
};
use time_manager::fsrs::FsrsPredictor;

#[test]
fn test_full_learning_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("study").unwrap();

    let card_path = "study/languages/english";
    let mut card = Card::new(card_path.to_string());
    fs.save_card(card_path, &card).unwrap();

    let loaded = fs.get_card(card_path).unwrap();
    assert_eq!(loaded.path, card_path);

    let predictor = FsrsPredictor::new().unwrap();
    let (interval, memory_state) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();

    let state_bytes = FsrsPredictor::memory_state_to_bytes(&memory_state);

    card.prediction = Some(time_manager::data::models::Prediction {
        algorithm: "fsrs".to_string(),
        next_review: Utc::now() + chrono::Duration::days(interval as i64),
        fsrs_state_bytes: state_bytes,
        preset_used: "default".to_string(),
    });

    let record = ReviewRecord {
        timestamp: Utc::now(),
        duration_ms: 1800000,
        memory_quality: MemoryQuality::Good,
    };

    card.review_records.push(record);
    fs.save_card(card_path, &card).unwrap();

    let updated = fs.get_card(card_path).unwrap();
    assert_eq!(updated.review_records.len(), 1);

    if let Some(prediction) = &updated.prediction {
        let recovered_state = FsrsPredictor::bytes_to_memory_state(&prediction.fsrs_state_bytes);
        assert!(recovered_state.is_some());
    }
}

#[test]
fn test_preset_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = time_manager::data::models::Preset {
        name: "intensive".to_string(),
        description: Some("高强度学习预设".to_string()),
        match_rules: vec!["study/*".to_string()],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&preset).unwrap();

    let loaded = fs.get_preset("intensive").unwrap();
    assert_eq!(loaded.name, "intensive");
    assert_eq!(loaded.match_rules.len(), 1);

    let presets = fs.list_presets().unwrap();
    assert!(presets.len() >= 2);
}

#[test]
fn test_todo_to_schedule_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let todo = time_manager::data::models::Todo::new("复习数学".to_string());
    let todo_id = todo.id;
    fs.save_todo(&todo).unwrap();

    let todos = fs.list_todos().unwrap();
    assert_eq!(todos.len(), 1);

    let ics_content = "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nDTSTART:20260610T100000\nDTEND:20260610T110000\nSUMMARY:复习数学\nEND:VEVENT\nEND:VCALENDAR";
    let schedule_id = uuid::Uuid::new_v4();
    fs.save_schedule(&schedule_id, ics_content).unwrap();
    fs.delete_todo(&todo_id).unwrap();

    let todos_after = fs.list_todos().unwrap();
    assert_eq!(todos_after.len(), 0);

    let schedules = fs.list_schedules().unwrap();
    assert_eq!(schedules.len(), 1);
}

#[test]
fn test_fsrs_memory_state_persistence() {
    let predictor = FsrsPredictor::new().unwrap();

    let (_, state1) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();

    let bytes = FsrsPredictor::memory_state_to_bytes(&state1);
    let recovered = FsrsPredictor::bytes_to_memory_state(&bytes).unwrap();

    let (_, state2) = predictor
        .predict_next_review(Some(recovered), MemoryQuality::Hard, 3, 0.9)
        .unwrap();

    assert!(state2.stability > 0.0);
}

#[test]
fn test_multiple_cards_in_tree() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("project").unwrap();

    let paths = vec![
        "project/module1/submodule1",
        "project/module1/submodule2",
        "project/module2",
    ];

    for path in &paths {
        let card = Card::new(path.to_string());
        fs.save_card(path, &card).unwrap();
    }

    let cards = fs.list_cards("project").unwrap();
    assert_eq!(cards.len(), 3);

    for card in &cards {
        assert!(paths.contains(&card.path.as_str()));
    }
}

#[test]
fn test_urgency_levels() {
    use chrono::Duration;

    let now = Utc::now();

    assert_eq!(FsrsPredictor::calculate_urgency(now - Duration::days(2)), 3);

    assert_eq!(FsrsPredictor::calculate_urgency(now), 2);
    assert_eq!(
        FsrsPredictor::calculate_urgency(now + Duration::hours(12)),
        2
    );

    assert_eq!(FsrsPredictor::calculate_urgency(now + Duration::days(2)), 1);
    assert_eq!(FsrsPredictor::calculate_urgency(now + Duration::days(3)), 1);

    assert_eq!(FsrsPredictor::calculate_urgency(now + Duration::days(7)), 0);
    assert_eq!(
        FsrsPredictor::calculate_urgency(now + Duration::days(30)),
        0
    );
}
