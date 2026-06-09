use std::path::PathBuf;
use tempfile::tempdir;
use time_manager::data::DataFs;
use time_manager::data::models::{Card, Preset, Timer, Todo};

#[test]
fn test_init_creates_directories() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    assert!(dir.path().join("categories").exists());
    assert!(dir.path().join("timers").exists());
    assert!(dir.path().join("presets").exists());
    assert!(dir.path().join("presets/default.json").exists());
    assert!(dir.path().join("todos").exists());
    assert!(dir.path().join("schedules").exists());
}

#[test]
fn test_create_tree() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("main").unwrap();
    assert!(dir.path().join("categories/main").exists());
}

#[test]
fn test_save_and_get_card() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("main").unwrap();

    let card = Card::new("main/语言/英语".into());
    fs.save_card("main/语言/英语", &card).unwrap();

    let path = dir.path().join("categories/main/语言/英语.json");
    assert!(path.exists());

    let loaded = fs.get_card("main/语言/英语").unwrap();
    assert_eq!(loaded.path, "main/语言/英语");
}

#[test]
fn test_list_cards() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    fs.create_tree("main").unwrap();

    let card1 = Card::new("main/语言/英语".into());
    fs.save_card("main/语言/英语", &card1).unwrap();

    let card2 = Card::new("main/语言/日语".into());
    fs.save_card("main/语言/日语", &card2).unwrap();

    let cards = fs.list_cards("main").unwrap();
    assert_eq!(cards.len(), 2);
}

#[test]
fn test_save_and_get_timer() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let timer = Timer::new(chrono::Utc::now());
    fs.save_timer(&timer).unwrap();

    let loaded = fs.get_timer(&timer.filename()).unwrap();
    assert_eq!(loaded.started_at, timer.started_at);
}

#[test]
fn test_save_and_get_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = Preset {
        name: "test".into(),
        description: Some("测试预设".into()),
        match_rules: vec!["main/*".into()],
    };
    fs.save_preset(&preset).unwrap();

    let loaded = fs.get_preset("test").unwrap();
    assert_eq!(loaded.name, "test");
}

#[test]
fn test_save_and_get_todo() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let todo = Todo::new("复习英语".into());
    fs.save_todo(&todo).unwrap();

    let loaded = fs.get_todo(&todo.id).unwrap();
    assert_eq!(loaded.content, "复习英语");
}

#[test]
fn test_list_todos() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let todo1 = Todo::new("任务1".into());
    fs.save_todo(&todo1).unwrap();

    let todo2 = Todo::new("任务2".into());
    fs.save_todo(&todo2).unwrap();

    let todos = fs.list_todos().unwrap();
    assert_eq!(todos.len(), 2);
}

#[test]
fn test_delete_todo() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let todo = Todo::new("待删除".into());
    fs.save_todo(&todo).unwrap();

    fs.delete_todo(&todo.id).unwrap();

    let todos = fs.list_todos().unwrap();
    assert_eq!(todos.len(), 0);
}

#[test]
fn test_save_schedule() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let id = uuid::Uuid::new_v4();
    let ics_content = "BEGIN:VCALENDAR\nEND:VCALENDAR";
    fs.save_schedule(&id, ics_content).unwrap();

    let loaded = fs.get_schedule(&id).unwrap();
    assert_eq!(loaded, ics_content);
}
