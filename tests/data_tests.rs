use chrono::Utc;
use time_manager::data::{
    models::{CategoryInsert, Difficulty, PauseRecord, PredictionStateInsert, Quality, SessionInsert, SessionParams},
    Database,
};

use std::sync::atomic::{AtomicU32, Ordering};
static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn test_db() -> Database {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("time-manager-test-{}-{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("test.db");
    Database::open(&path).expect("open test db")
}

fn insert_root(db: &Database, name: &str) -> i64 {
    db.insert_category(&CategoryInsert {
        parent_id: None,
        name: name.into(),
        path: name.into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert root")
}

fn insert_child(db: &Database, parent_id: i64, name: &str, path: &str) -> i64 {
    db.insert_category(&CategoryInsert {
        parent_id: Some(parent_id),
        name: name.into(),
        path: path.into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert child")
}

#[test]
fn test_open_and_create_tables() {
    let db = test_db();
    assert!(db.db_path().exists());
}

#[test]
fn test_open_creates_parent_dirs() {
    let dir = std::env::temp_dir().join(format!("tm-nested-{}-{}", std::process::id(), TEST_COUNTER.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&dir);
    let nested = dir.join("a/b/c");
    let path = nested.join("test.db");
    let db = Database::open(&path).expect("open nested");
    assert!(db.db_path().exists());
    assert!(nested.exists());
}

#[test]
fn test_category_crud() {
    let db = test_db();
    let id = insert_root(&db, "语言");
    let cat = db.get_category(id).expect("get");
    assert_eq!(cat.name, "语言");
    assert_eq!(cat.path, "语言");
    assert!(cat.parent_id.is_none());

    let child_id = insert_child(&db, id, "英语", "语言/英语");
    let children = db.get_children(Some(id)).expect("children");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, child_id);

    db.update_category_name(child_id, "日语").expect("update");
    let updated = db.get_category(child_id).expect("get");
    assert_eq!(updated.name, "日语");
    assert_eq!(updated.path, "语言/英语");

    db.delete_category(child_id).expect("delete");
    let children = db.get_children(Some(id)).expect("children");
    assert!(children.is_empty());
}

#[test]
fn test_category_with_defaults() {
    let db = test_db();
    let id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "Rust".into(),
        path: "Rust".into(),
        source: Some("obsidian".into()),
        default_quality: Some(Quality::High),
        default_understanding_difficulty: Some(Difficulty::Hard),
        default_memory_difficulty: Some(Difficulty::VeryHard),
    }).expect("insert");
    let cat = db.get_category(id).expect("get");
    assert_eq!(cat.source.as_deref(), Some("obsidian"));
    assert_eq!(cat.default_quality, Some(Quality::High));
    assert_eq!(cat.default_understanding_difficulty, Some(Difficulty::Hard));
    assert_eq!(cat.default_memory_difficulty, Some(Difficulty::VeryHard));
}

#[test]
fn test_category_get_children_root() {
    let db = test_db();
    let _ = insert_root(&db, "A");
    let _ = insert_root(&db, "B");
    let roots = db.get_children(None).expect("roots");
    assert_eq!(roots.len(), 2);
}

#[test]
fn test_category_get_all() {
    let db = test_db();
    let a = insert_root(&db, "A");
    let _ = insert_child(&db, a, "A1", "A/A1");
    let _ = insert_child(&db, a, "A2", "A/A2");
    let b = insert_root(&db, "B");
    let _ = insert_child(&db, b, "B1", "B/B1");
    let all = db.get_all_categories().expect("all");
    assert_eq!(all.len(), 5);
}

#[test]
fn test_category_delete_cascade() {
    let db = test_db();
    let parent = insert_root(&db, "P");
    let child = insert_child(&db, parent, "C", "P/C");
    let sessions_before = db.get_sessions_by_category(child).expect("sessions");
    assert!(sessions_before.is_empty());
    db.delete_category(parent).expect("delete parent");
    assert!(db.get_category(child).is_err());
}

#[test]
fn test_category_duplicate_path_rejected() {
    let db = test_db();
    let _ = insert_root(&db, "X");
    let result = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "X2".into(),
        path: "X".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    });
    assert!(result.is_err());
}

#[test]
fn test_session_crud() {
    let db = test_db();
    let cat_id = insert_root(&db, "数学");

    let now = Utc::now();
    let session_id = db.insert_session(&SessionInsert {
        category_id: cat_id,
        start_time: now,
        end_time: now + chrono::Duration::seconds(3600),
        duration_secs: 3600,
        pause_records: vec![],
        params: SessionParams::default(),
        note: Some("test note".into()),
    }).expect("insert session");

    let session = db.get_session(session_id).expect("get");
    assert_eq!(session.category_id, cat_id);
    assert_eq!(session.duration_secs, 3600);
    assert_eq!(session.note.as_deref(), Some("test note"));
    assert!(!session.is_manual_edit);

    let sessions = db.get_sessions_by_category(cat_id).expect("list");
    assert_eq!(sessions.len(), 1);
}

#[test]
fn test_session_with_pause_records() {
    let db = test_db();
    let cat_id = insert_root(&db, "Rust");

    let now = Utc::now();
    let pauses = vec![
        PauseRecord {
            pause_start: now + chrono::Duration::seconds(600),
            resume_time: now + chrono::Duration::seconds(900),
        },
        PauseRecord {
            pause_start: now + chrono::Duration::seconds(1200),
            resume_time: now + chrono::Duration::seconds(1350),
        },
    ];

    let session_id = db.insert_session(&SessionInsert {
        category_id: cat_id,
        start_time: now,
        end_time: now + chrono::Duration::seconds(1800),
        duration_secs: 1350,
        pause_records: pauses.clone(),
        params: SessionParams::default(),
        note: None,
    }).expect("insert");

    let session = db.get_session(session_id).expect("get");
    assert_eq!(session.pause_records.len(), 2);
    assert_eq!(session.pause_records[0].resume_time, pauses[0].resume_time);
}

#[test]
fn test_session_with_all_params() {
    let db = test_db();
    let cat_id = insert_root(&db, "英语");

    let now = Utc::now();
    let session_id = db.insert_session(&SessionInsert {
        category_id: cat_id,
        start_time: now,
        end_time: now + chrono::Duration::seconds(1800),
        duration_secs: 1800,
        pause_records: vec![],
        params: SessionParams {
            quality: Quality::Complete,
            understanding_difficulty: Difficulty::VeryEasy,
            memory_difficulty: Difficulty::VeryHard,
            completion_rate: 75,
        },
        note: Some("advanced session".into()),
    }).expect("insert");

    let session = db.get_session(session_id).expect("get");
    assert_eq!(session.quality, Quality::Complete);
    assert_eq!(session.understanding_difficulty, Difficulty::VeryEasy);
    assert_eq!(session.memory_difficulty, Difficulty::VeryHard);
    assert_eq!(session.completion_rate, 75);
}

#[test]
fn test_session_time_range_query() {
    let db = test_db();
    let cat_id = insert_root(&db, "X");

    let base = Utc::now() - chrono::Duration::days(3);
    for i in 0..5 {
        let start = base + chrono::Duration::days(i);
        db.insert_session(&SessionInsert {
            category_id: cat_id,
            start_time: start,
            end_time: start + chrono::Duration::seconds(3600),
            duration_secs: 3600,
            pause_records: vec![],
            params: SessionParams::default(),
            note: None,
        }).expect("insert");
    }

    let range_start = base + chrono::Duration::days(1);
    let range_end = base + chrono::Duration::days(4);
    let sessions = db.get_sessions_in_range(&range_start, &range_end).expect("range");
    assert_eq!(sessions.len(), 3);
}

#[test]
fn test_session_update_and_delete() {
    let db = test_db();
    let cat_id = insert_root(&db, "Y");

    let now = Utc::now();
    let sid = db.insert_session(&SessionInsert {
        category_id: cat_id,
        start_time: now,
        end_time: now + chrono::Duration::seconds(600),
        duration_secs: 600,
        pause_records: vec![],
        params: SessionParams::default(),
        note: None,
    }).expect("insert");

    db.update_session_note(sid, "updated note").expect("update");
    let s = db.get_session(sid).expect("get");
    assert_eq!(s.note.as_deref(), Some("updated note"));
    assert!(s.is_manual_edit);

    db.delete_session(sid).expect("delete");
    assert!(db.get_session(sid).is_err());
}

#[test]
fn test_prediction_state_crud() {
    let db = test_db();
    let cat_id = insert_root(&db, "Rust");

    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(3),
        algorithm_state: vec![1, 2, 3],
    }).expect("upsert");

    let ps = db.get_prediction_state(cat_id).expect("get");
    assert!(ps.is_some());
    let ps = ps.unwrap();
    assert_eq!(ps.algorithm, "fsrs");
    assert_eq!(ps.algorithm_state, vec![1, 2, 3]);

    let due = db.get_due_predictions(&(now + chrono::Duration::days(4))).expect("due");
    assert_eq!(due.len(), 1);
}

#[test]
fn test_prediction_state_upsert_overwrite() {
    let db = test_db();
    let cat_id = insert_root(&db, "Go");

    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(1),
        algorithm_state: vec![1],
    }).expect("upsert1");

    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: now + chrono::Duration::days(1),
        next_review: now + chrono::Duration::days(5),
        algorithm_state: vec![2, 3],
    }).expect("upsert2");

    let ps = db.get_prediction_state(cat_id).expect("get").expect("some");
    assert_eq!(ps.algorithm_state, vec![2, 3]);
    assert!(ps.next_review > now + chrono::Duration::days(4));
}

#[test]
fn test_prediction_state_not_due() {
    let db = test_db();
    let cat_id = insert_root(&db, "Py");

    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(10),
        algorithm_state: vec![],
    }).expect("upsert");

    let due = db.get_due_predictions(&(now + chrono::Duration::days(5))).expect("due");
    assert!(due.is_empty());
}

#[test]
fn test_prediction_state_delete() {
    let db = test_db();
    let cat_id = insert_root(&db, "C");

    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(1),
        algorithm_state: vec![],
    }).expect("upsert");

    db.delete_prediction_state(cat_id).expect("delete");
    let ps = db.get_prediction_state(cat_id).expect("get");
    assert!(ps.is_none());
}

#[test]
fn test_prediction_multiple_due_sorted() {
    let db = test_db();
    let cat_a = insert_root(&db, "A");
    let cat_b = insert_root(&db, "B");

    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_a,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(1),
        algorithm_state: vec![],
    }).expect("upsert a");

    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_b,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(3),
        algorithm_state: vec![],
    }).expect("upsert b");

    let due = db.get_due_predictions(&(now + chrono::Duration::days(5))).expect("due");
    assert_eq!(due.len(), 2);
    assert_eq!(due[0].category_id, cat_a);
    assert_eq!(due[1].category_id, cat_b);
}

#[test]
fn test_settings_crud() {
    let db = test_db();
    assert!(db.get_setting("test_key").expect("get missing").is_none());
    db.set_setting("test_key", "test_value").expect("set");
    assert_eq!(db.get_setting("test_key").expect("get").unwrap(), "test_value");
    db.set_setting("test_key", "new_value").expect("update");
    assert_eq!(db.get_setting("test_key").expect("get").unwrap(), "new_value");
}

#[test]
fn test_settings_multiple_keys() {
    let db = test_db();
    db.set_setting("k1", "v1").expect("set");
    db.set_setting("k2", "v2").expect("set");
    db.set_setting("k3", "v3").expect("set");
    assert_eq!(db.get_setting("k1").expect("g").unwrap(), "v1");
    assert_eq!(db.get_setting("k2").expect("g").unwrap(), "v2");
    assert_eq!(db.get_setting("k3").expect("g").unwrap(), "v3");
    assert!(db.get_setting("k4").expect("g").is_none());
}

#[test]
fn test_settings_json_value() {
    let db = test_db();
    let json = r#"[{"name":"test","quality":"高","understanding_difficulty":"中","memory_difficulty":"中","completion_rate":100}]"#;
    db.set_setting("presets", json).expect("set");
    let val = db.get_setting("presets").expect("get").unwrap();
    assert!(val.contains("test"));
}

mod export_tests {
    use chrono::Utc;
    use time_manager::data::export::{CsvExporter, Exporter, JsonExporter, MarkdownExporter};
    use time_manager::data::models::{
        Difficulty, PauseRecord, PredictionState, Quality, Session,
    };

    fn make_session(id: i64, cat_id: i64, duration: i64, quality: Quality) -> Session {
        let now = Utc::now();
        Session {
            id,
            category_id: cat_id,
            start_time: now,
            end_time: now + chrono::Duration::seconds(duration),
            duration_secs: duration,
            pause_records: vec![],
            quality,
            understanding_difficulty: Difficulty::Medium,
            memory_difficulty: Difficulty::Medium,
            completion_rate: 100,
            note: None,
            is_manual_edit: false,
        }
    }

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("tm-export-{}-{}", name, std::process::id()))
    }

    #[test]
    fn test_json_export() {
        let sessions = vec![make_session(1, 1, 1800, Quality::High)];
        let dir = tmp_dir("json");
        let _ = std::fs::remove_dir_all(&dir);
        let path = JsonExporter.export_sessions(&sessions, &dir).expect("export");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.contains("\"duration_secs\":1800"));
        assert!(content.contains("\"quality\":\"High\""));
    }

    #[test]
    fn test_json_export_multiple() {
        let sessions = vec![
            make_session(1, 1, 1800, Quality::High),
            make_session(2, 1, 3600, Quality::Medium),
        ];
        let dir = tmp_dir("json-multi");
        let _ = std::fs::remove_dir_all(&dir);
        let path = JsonExporter.export_sessions(&sessions, &dir).expect("export");
        let content = std::fs::read_to_string(path).expect("read");
        let line_count = content.lines().filter(|l| !l.is_empty()).count();
        assert_eq!(line_count, 2);
    }

    #[test]
    fn test_json_export_with_pauses() {
        let now = Utc::now();
        let sessions = vec![Session {
            id: 1, category_id: 1, start_time: now, end_time: now,
            duration_secs: 600, pause_records: vec![PauseRecord {
                pause_start: now, resume_time: now + chrono::Duration::seconds(300),
            }],
            quality: Quality::Medium, understanding_difficulty: Difficulty::Medium,
            memory_difficulty: Difficulty::Medium, completion_rate: 80,
            note: None, is_manual_edit: false,
        }];
        let dir = tmp_dir("json-pause");
        let _ = std::fs::remove_dir_all(&dir);
        let path = JsonExporter.export_sessions(&sessions, &dir).expect("export");
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.contains("pause_records"));
    }

    #[test]
    fn test_csv_export() {
        let sessions = vec![make_session(1, 1, 600, Quality::Medium)];
        let dir = tmp_dir("csv");
        let _ = std::fs::remove_dir_all(&dir);
        let path = CsvExporter.export_sessions(&sessions, &dir).expect("export");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.starts_with("id,category_id"));
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_csv_export_predictions() {
        let predictions = vec![PredictionState {
            id: 1, category_id: 1, algorithm: "fsrs".into(),
            last_review: Utc::now(), next_review: Utc::now() + chrono::Duration::days(1),
            algorithm_state: vec![],
        }];
        let dir = tmp_dir("csv-pred");
        let _ = std::fs::remove_dir_all(&dir);
        let path = CsvExporter.export_predictions(&predictions, &dir).expect("export");
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.starts_with("id,category_id,algorithm"));
    }

    #[test]
    fn test_markdown_export() {
        let predictions = vec![PredictionState {
            id: 1, category_id: 1, algorithm: "fsrs".into(),
            last_review: Utc::now(), next_review: Utc::now() + chrono::Duration::days(1),
            algorithm_state: vec![],
        }];
        let dir = tmp_dir("md");
        let _ = std::fs::remove_dir_all(&dir);
        let path = MarkdownExporter.export_predictions(&predictions, &dir).expect("export");
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.contains("# Prediction States"));
        assert!(content.contains("| Category |"));
    }

    #[test]
    fn test_markdown_export_sessions() {
        let sessions = vec![make_session(1, 42, 7325, Quality::High)];
        let dir = tmp_dir("md-sess");
        let _ = std::fs::remove_dir_all(&dir);
        let path = MarkdownExporter.export_sessions(&sessions, &dir).expect("export");
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.contains("# Sessions"));
        assert!(content.contains("02:02:05"));
    }

    #[test]
    fn test_export_empty_data() {
        let dir = tmp_dir("empty");
        let _ = std::fs::remove_dir_all(&dir);
        let path = JsonExporter.export_sessions(&[], &dir).expect("export empty");
        let content = std::fs::read_to_string(path).expect("read");
        assert!(content.is_empty() || content.trim().is_empty());
    }
}