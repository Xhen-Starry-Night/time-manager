use time_manager::data::models::{Category, Difficulty, NodeType, Quality};
use time_manager::modules::learning::category::tree::{CategoryForest, CategoryNode};
use time_manager::modules::learning::timer::state_machine::TimerStateMachine;

fn sample_categories() -> Vec<Category> {
    vec![
        Category { id: 1, parent_id: None, name: "语言".into(), path: "语言".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 2, parent_id: Some(1), name: "英语".into(), path: "语言/英语".into(), node_type: NodeType::Learning, source: None, default_quality: Some(Quality::High), default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 3, parent_id: Some(2), name: "六级".into(), path: "语言/英语/六级".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 4, parent_id: None, name: "数学".into(), path: "数学".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
    ]
}

#[test]
fn test_forest_construction() {
    let forest = CategoryForest::from_categories(&sample_categories());
    assert_eq!(forest.roots.len(), 2);
    assert_eq!(forest.roots[0].name, "语言");
    assert_eq!(forest.roots[0].children.len(), 1);
    assert_eq!(forest.roots[0].children[0].name, "英语");
    assert_eq!(forest.roots[0].children[0].children[0].name, "六级");
}

#[test]
fn test_forest_find() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let node = forest.find(3).expect("find 六级");
    assert_eq!(node.name, "六级");
    assert!(forest.find(999).is_none());
}

#[test]
fn test_forest_leaf_nodes() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let leaves = forest.leaf_nodes();
    assert_eq!(leaves.len(), 2);
    let leaf_names: Vec<&str> = leaves.iter().map(|n| n.name.as_str()).collect();
    assert!(leaf_names.contains(&"六级"));
    assert!(leaf_names.contains(&"数学"));
}

#[test]
fn test_node_effective_params() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let en = forest.find(2).expect("find 英语");
    assert_eq!(en.default_quality, Some(Quality::High));
}

#[test]
fn test_empty_forest() {
    let forest = CategoryForest::from_categories(&[]);
    assert!(forest.roots.is_empty());
    assert!(forest.all_nodes().is_empty());
    assert!(forest.leaf_nodes().is_empty());
    assert!(forest.find(1).is_none());
}

#[test]
fn test_forest_find_mut() {
    let mut forest = CategoryForest::from_categories(&sample_categories());
    if let Some(node) = forest.find_mut(3) {
        node.name = "四级".into();
    }
    let node = forest.find(3).expect("find modified");
    assert_eq!(node.name, "四级");
}

#[test]
fn test_forest_all_nodes() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let all = forest.all_nodes();
    assert_eq!(all.len(), 4);
}

#[test]
fn test_forest_find_by_path() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let node = forest.roots[0].find_by_path("语言/英语/六级");
    assert!(node.is_some());
    assert_eq!(node.unwrap().name, "六级");
    assert!(forest.roots[0].find_by_path("不存在").is_none());
}

#[test]
fn test_forest_deep_nesting() {
    let mut cats = Vec::new();
    for i in 0..20 {
        cats.push(Category {
            node_type: NodeType::Learning,
            id: i as i64 + 1,
            parent_id: if i == 0 { None } else { Some(i as i64) },
            name: format!("L{}", i),
            path: (0..=i).map(|j| format!("L{}", j)).collect::<Vec<_>>().join("/"),
            source: None,
            default_quality: None,
            default_understanding_difficulty: None,
            default_memory_difficulty: None,
        });
    }
    let forest = CategoryForest::from_categories(&cats);
    assert_eq!(forest.roots.len(), 1);
    let deepest = forest.find(20);
    assert!(deepest.is_some());
    assert_eq!(deepest.unwrap().name, "L19");
    assert_eq!(forest.leaf_nodes().len(), 1);
}

#[test]
fn test_forest_multiple_children() {
    let cats = vec![
        Category { id: 1, parent_id: None, name: "Root".into(), path: "Root".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 2, parent_id: Some(1), name: "A".into(), path: "Root/A".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 3, parent_id: Some(1), name: "B".into(), path: "Root/B".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 4, parent_id: Some(1), name: "C".into(), path: "Root/C".into(), node_type: NodeType::Learning, source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
    ];
    let forest = CategoryForest::from_categories(&cats);
    assert_eq!(forest.roots[0].children.len(), 3);
}

#[test]
fn test_node_is_leaf() {
    let forest = CategoryForest::from_categories(&sample_categories());
    assert!(!forest.find(1).unwrap().is_leaf());
    assert!(!forest.find(2).unwrap().is_leaf());
    assert!(forest.find(3).unwrap().is_leaf());
    assert!(forest.find(4).unwrap().is_leaf());
}

#[test]
fn test_timer_lifecycle() {
    let mut tm = TimerStateMachine::new();
    assert!(!tm.is_running());

    tm.start(42).expect("start");
    assert!(tm.is_running());

    let secs = tm.effective_secs();
    assert!(secs >= 0);

    tm.pause().expect("pause");
    assert!(tm.is_running());

    tm.resume().expect("resume");
    assert!(tm.is_running());

    let stopped = tm.stop().expect("stop");
    assert!(!tm.is_running());
    assert_eq!(stopped.category_id, 42);
    assert!(stopped.duration_secs >= 0);
}

#[test]
fn test_timer_invalid_transitions() {
    let mut tm = TimerStateMachine::new();
    assert!(tm.pause().is_err());
    assert!(tm.resume().is_err());
    assert!(tm.stop().is_err());

    tm.start(1).expect("start");
    assert!(tm.start(2).is_err());
    assert!(tm.resume().is_err());

    tm.pause().expect("pause");
    assert!(tm.pause().is_err());
}

#[test]
fn test_timer_pause_deducts_time() {
    let mut tm = TimerStateMachine::new();
    tm.start(1).expect("start");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tm.pause().expect("pause");
    std::thread::sleep(std::time::Duration::from_millis(200));
    tm.resume().expect("resume");
    let stopped = tm.stop().expect("stop");
    assert!(stopped.duration_secs < 1);
}

#[test]
fn test_timer_multiple_pause_resume() {
    let mut tm = TimerStateMachine::new();
    tm.start(1).expect("start");
    std::thread::sleep(std::time::Duration::from_millis(50));
    tm.pause().expect("pause1");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tm.resume().expect("resume1");
    std::thread::sleep(std::time::Duration::from_millis(50));
    tm.pause().expect("pause2");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tm.resume().expect("resume2");
    let stopped = tm.stop().expect("stop");
    assert_eq!(stopped.pause_records.len(), 2);
    assert!(stopped.duration_secs < 1);
}

#[test]
fn test_timer_stop_from_paused() {
    let mut tm = TimerStateMachine::new();
    tm.start(1).expect("start");
    std::thread::sleep(std::time::Duration::from_millis(50));
    tm.pause().expect("pause");
    let stopped = tm.stop().expect("stop from paused");
    assert!(!tm.is_running());
    assert!(stopped.duration_secs >= 0);
    assert_eq!(stopped.pause_records.len(), 1);
}

#[test]
fn test_timer_effective_secs_while_paused() {
    let mut tm = TimerStateMachine::new();
    tm.start(1).expect("start");
    std::thread::sleep(std::time::Duration::from_millis(50));
    tm.pause().expect("pause");
    std::thread::sleep(std::time::Duration::from_millis(100));
    let secs_paused = tm.effective_secs();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let secs_paused2 = tm.effective_secs();
    assert_eq!(secs_paused, secs_paused2);
}

#[test]
fn test_timer_reuse_after_stop() {
    let mut tm = TimerStateMachine::new();
    tm.start(1).expect("start1");
    let _ = tm.stop().expect("stop1");
    assert!(!tm.is_running());
    tm.start(2).expect("start2");
    assert!(tm.is_running());
    let stopped = tm.stop().expect("stop2");
    assert_eq!(stopped.category_id, 2);
}

#[test]
fn test_obsidian_vault_parse() {
    use std::fs;
    use time_manager::modules::learning::category::obsidian::vault_parser;

    let vault_dir = std::env::temp_dir().join(format!("tm-obsidian-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&vault_dir);
    fs::create_dir_all(vault_dir.join("语言/英语")).expect("create dirs");
    fs::write(vault_dir.join("语言/英语/六级.md"), "# 六级").expect("write");
    fs::write(vault_dir.join("语言/英语/语法.md"), "# 语法").expect("write");
    fs::create_dir_all(vault_dir.join(".obsidian")).expect("create hidden");
    fs::write(vault_dir.join(".obsidian/config"), "").expect("write hidden");

    let entries = vault_parser::parse_vault(&vault_dir);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"英语"));
    assert!(names.contains(&"六级"));
    assert!(names.contains(&"语法"));
    assert!(!names.iter().any(|n| *n == ".obsidian" || *n == "config"));
}

#[test]
fn test_obsidian_deep_nested() {
    use std::fs;
    use time_manager::modules::learning::category::obsidian::vault_parser;

    let dir = std::env::temp_dir().join(format!("tm-obsidian-deep-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("A/B/C/D")).expect("deep dirs");
    fs::write(dir.join("A/B/C/D/leaf.md"), "# leaf").expect("write");
    let entries = vault_parser::parse_vault(&dir);
    let dirs: Vec<&str> = entries.iter().filter(|e| e.is_dir).map(|e| e.name.as_str()).collect();
    assert!(dirs.contains(&"B"));
    assert!(dirs.contains(&"C"));
    assert!(dirs.contains(&"D"));
    let files: Vec<&str> = entries.iter().filter(|e| !e.is_dir).map(|e| e.name.as_str()).collect();
    assert!(files.contains(&"leaf"));
}

#[test]
fn test_obsidian_empty_vault() {
    use std::fs;
    use time_manager::modules::learning::category::obsidian::vault_parser;

    let dir = std::env::temp_dir().join(format!("tm-obsidian-empty-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create empty vault");
    let entries = vault_parser::parse_vault(&dir);
    assert!(entries.is_empty());
}

#[test]
fn test_obsidian_ignores_non_md() {
    use std::fs;
    use time_manager::modules::learning::category::obsidian::vault_parser;

    let dir = std::env::temp_dir().join(format!("tm-obsidian-nomd-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create");
    fs::write(dir.join("image.png"), "fake").expect("write png");
    fs::write(dir.join("data.csv"), "a,b").expect("write csv");
    fs::write(dir.join("note.md"), "# note").expect("write md");
    let entries = vault_parser::parse_vault(&dir);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"note"));
    assert!(!names.contains(&"image"));
    assert!(!names.contains(&"data"));
}