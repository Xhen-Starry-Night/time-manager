use chrono::Utc;
use time_manager::data::{
    models::{CategoryInsert, Difficulty, Quality, SessionParams},
    Database,
};
use time_manager::modules::learning::category::tree::CategoryForest;
use time_manager::modules::learning::prediction::algorithm::PredictionAlgorithm;
use time_manager::modules::learning::prediction::fsrs::adapter::FsrsAdapter;
use time_manager::modules::learning::timer::params::{resolve_params, apply_preset};
use time_manager::modules::learning::timer::session::create_session_insert;
use time_manager::modules::learning::timer::state_machine::TimerStateMachine;
use time_manager::data::models::Preset;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn test_db() -> Database {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("tm-integ-{}-{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dir);
    Database::open(&dir.join("test.db")).expect("open")
}

#[test]
fn test_full_session_flow() {
    let db = test_db();

    let lang_id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "语言".into(),
        path: "语言".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert 语言");

    let en_id = db.insert_category(&CategoryInsert {
        parent_id: Some(lang_id),
        name: "英语".into(),
        path: "语言/英语".into(),
        source: None,
        default_quality: Some(Quality::High),
        default_understanding_difficulty: Some(Difficulty::Medium),
        default_memory_difficulty: None,
    }).expect("insert 英语");

    let cats = db.get_all_categories().expect("all cats");
    let forest = CategoryForest::from_categories(&cats);
    let en_node = forest.find(en_id).expect("find 英语");
    assert_eq!(en_node.default_quality, Some(Quality::High));

    let global_defaults = SessionParams::default();
    let params = resolve_params(
        &global_defaults,
        en_node.default_quality.as_ref(),
        en_node.default_understanding_difficulty.as_ref(),
        en_node.default_memory_difficulty.as_ref(),
    );
    assert_eq!(params.quality, Quality::High);
    assert_eq!(params.understanding_difficulty, Difficulty::Medium);

    let mut timer = TimerStateMachine::new();
    timer.start(en_id).expect("start");
    std::thread::sleep(std::time::Duration::from_millis(50));
    timer.pause().expect("pause");
    std::thread::sleep(std::time::Duration::from_millis(100));
    timer.resume().expect("resume");
    std::thread::sleep(std::time::Duration::from_millis(50));

    let stopped = timer.stop().expect("stop");
    assert!(!timer.is_running());
    assert_eq!(stopped.category_id, en_id);
    assert!(stopped.duration_secs >= 0);
    assert_eq!(stopped.pause_records.len(), 1);

    let session_insert = create_session_insert(stopped, params.clone(), Some("学习英语".into()));
    let session_id = db.insert_session(&session_insert).expect("save session");

    let saved = db.get_session(session_id).expect("get session");
    assert_eq!(saved.category_id, en_id);
    assert_eq!(saved.quality, Quality::High);
    assert_eq!(saved.understanding_difficulty, Difficulty::Medium);
    assert_eq!(saved.note.as_deref(), Some("学习英语"));
    assert_eq!(saved.pause_records.len(), 1);

    let adapter = FsrsAdapter::new();
    let pred_result = adapter.predict_next_review(&[], &[], &params.quality);
    db.upsert_prediction_state(&time_manager::data::models::PredictionStateInsert {
        category_id: en_id,
        algorithm: adapter.algorithm_name().into(),
        last_review: Utc::now(),
        next_review: pred_result.next_review,
        algorithm_state: pred_result.state_bytes,
    }).expect("save prediction");

    let ps = db.get_prediction_state(en_id).expect("get prediction").expect("some");
    assert_eq!(ps.algorithm, "fsrs");
    assert!(ps.next_review >= Utc::now() - chrono::Duration::days(1));

    let due = db.get_due_predictions(&(Utc::now() + chrono::Duration::days(30))).expect("due");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].category_id, en_id);
}

#[test]
fn test_session_flow_with_preset() {
    let db = test_db();

    let cat_id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "Rust".into(),
        path: "Rust".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert");

    let mut timer = TimerStateMachine::new();
    timer.start(cat_id).expect("start");
    let stopped = timer.stop().expect("stop");

    let mut params = SessionParams::default();
    let preset = Preset::built_in()[0].clone();
    apply_preset(&preset, &mut params);

    let insert = create_session_insert(stopped, params, None);
    let sid = db.insert_session(&insert).expect("save");

    let saved = db.get_session(sid).expect("get");
    assert_eq!(saved.quality, Quality::High);
    assert_eq!(saved.completion_rate, 100);
}

#[test]
fn test_multiple_sessions_accumulate() {
    let db = test_db();
    let cat_id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "数学".into(),
        path: "数学".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert");

    for i in 0..5 {
        let now = Utc::now() + chrono::Duration::hours(i);
        db.insert_session(&time_manager::data::models::SessionInsert {
            category_id: cat_id,
            start_time: now,
            end_time: now + chrono::Duration::seconds(1800),
            duration_secs: 1800,
            pause_records: vec![],
            params: SessionParams::default(),
            note: Some(format!("session {}", i)),
        }).expect("insert");
    }

    let sessions = db.get_sessions_by_category(cat_id).expect("list");
    assert_eq!(sessions.len(), 5);
}

#[test]
fn test_obsidian_import_to_forest() {
    let db = test_db();
    let vault_dir = std::env::temp_dir().join(format!("tm-integ-obsidian-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&vault_dir);
    std::fs::create_dir_all(vault_dir.join("编程/Rust")).expect("create");
    std::fs::create_dir_all(vault_dir.join("语言/日语")).expect("create");
    std::fs::write(vault_dir.join("编程/Rust/所有权.md"), "# 所有权").expect("write");
    std::fs::write(vault_dir.join("语言/日语/N5.md"), "# N5").expect("write");

    let entries = time_manager::modules::learning::category::obsidian::vault_parser::parse_vault(&vault_dir);
    assert!(!entries.is_empty());

    for entry in &entries {
        let existing = db.get_all_categories().unwrap_or_default();
        if existing.iter().any(|c| c.path == entry.relative_path) {
            continue;
        }
        let parent_path = entry.relative_path.rsplit_once('/').map(|(p, _)| p);
        let parent_id = parent_path.and_then(|pp| {
            existing.iter().find(|c| c.path == pp).map(|c| c.id)
        });

        db.insert_category(&CategoryInsert {
            parent_id,
            name: entry.name.clone(),
            path: entry.relative_path.clone(),
            source: Some("obsidian".into()),
            default_quality: None,
            default_understanding_difficulty: None,
            default_memory_difficulty: None,
        }).expect("insert");
    }

    let cats = db.get_all_categories().expect("all");
    let forest = CategoryForest::from_categories(&cats);

    let rust = forest.roots.iter().find(|r| r.name == "编程").expect("find 编程");
    assert!(rust.find_by_path("编程/Rust").is_some());
    assert!(rust.find_by_path("编程/Rust/所有权").is_some());

    let lang = forest.roots.iter().find(|r| r.name == "语言").expect("find 语言");
    assert!(lang.find_by_path("语言/日语").is_some());
    assert!(lang.find_by_path("语言/日语/N5").is_some());

    let leaves = forest.leaf_nodes();
    let leaf_names: Vec<&str> = leaves.iter().map(|n| n.name.as_str()).collect();
    assert!(leaf_names.contains(&"所有权"));
    assert!(leaf_names.contains(&"N5"));
}

#[test]
fn test_prediction_update_after_session() {
    let db = test_db();
    let adapter = FsrsAdapter::new();

    let cat_id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "Go".into(),
        path: "Go".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert");

    assert!(db.get_prediction_state(cat_id).expect("get").is_none());

    let r1 = adapter.predict_next_review(&[], &[], &Quality::High);
    db.upsert_prediction_state(&time_manager::data::models::PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: Utc::now(),
        next_review: r1.next_review,
        algorithm_state: r1.state_bytes.clone(),
    }).expect("upsert1");

    let r2 = adapter.predict_next_review(&[], &r1.state_bytes, &Quality::Complete);
    db.upsert_prediction_state(&time_manager::data::models::PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: Utc::now(),
        next_review: r2.next_review,
        algorithm_state: r2.state_bytes,
    }).expect("upsert2");

    let ps = db.get_prediction_state(cat_id).expect("get").expect("some");
    assert_eq!(ps.next_review, r2.next_review);
    assert!(ps.next_review >= r1.next_review);
}

#[test]
fn test_settings_persistence_across_sessions() {
    let db = test_db();

    db.set_setting("default_quality", "高").expect("set");
    db.set_setting("default_completion_rate", "80").expect("set");

    let loaded = time_manager::settings::defaults::load_global_params(&db);
    assert_eq!(loaded.quality, Quality::High);
    assert_eq!(loaded.completion_rate, 80);
    assert_eq!(loaded.understanding_difficulty, Difficulty::Medium);

    time_manager::settings::defaults::save_global_params(&db, &SessionParams {
        quality: Quality::Complete,
        understanding_difficulty: Difficulty::Easy,
        memory_difficulty: Difficulty::Hard,
        completion_rate: 95,
    }).expect("save");

    let reloaded = time_manager::settings::defaults::load_global_params(&db);
    assert_eq!(reloaded.quality, Quality::Complete);
    assert_eq!(reloaded.understanding_difficulty, Difficulty::Easy);
    assert_eq!(reloaded.memory_difficulty, Difficulty::Hard);
    assert_eq!(reloaded.completion_rate, 95);
}

#[test]
fn test_presets_persistence() {
    let db = test_db();

    let presets = time_manager::settings::defaults::load_presets(&db);
    assert_eq!(presets.len(), 3);

    let mut custom = presets;
    custom.push(Preset {
        name: "冲刺".into(),
        quality: Quality::Complete,
        understanding_difficulty: Difficulty::VeryHard,
        memory_difficulty: Difficulty::VeryHard,
        completion_rate: 100,
    });
    time_manager::settings::defaults::save_presets(&db, &custom).expect("save");

    let loaded = time_manager::settings::defaults::load_presets(&db);
    assert_eq!(loaded.len(), 4);
    assert_eq!(loaded[3].name, "冲刺");
}