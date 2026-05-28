use time_manager::data::Database;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let db = match Database::open_default() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Database error: {}", e);
            std::process::exit(1);
        }
    };

    match args[1].as_str() {
        "status" => cmd_status(&db),
        "list-due" => cmd_list_due(&db),
        "stats" => cmd_stats(&db, args.get(2).map(|s| s.as_str())),
        "export" => cmd_export(&db, args.get(2).map(|s| s.as_str())),
        "import-obsidian" => cmd_import_obsidian(&db, args.get(2).map(|s| s.as_str())),
        _ => print_usage(),
    }
}

fn cmd_status(db: &Database) {
    if let Ok(cats) = db.get_all_categories() {
        println!("Categories: {} nodes", cats.len());
    }
    if let Ok(sessions) = db.get_all_sessions() {
        let total_secs: i64 = sessions.iter().map(|s| s.duration_secs).sum();
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        println!("Sessions: {} total, {}h{}m total time", sessions.len(), h, m);
    }
}

fn cmd_list_due(db: &Database) {
    let now = chrono::Utc::now();
    match db.get_due_predictions(&now) {
        Ok(predictions) => {
            if predictions.is_empty() {
                println!("No due predictions.");
                return;
            }
            for p in &predictions {
                let overdue = now > p.next_review;
                let marker = if overdue { "OVERDUE" } else { "DUE" };
                if let Ok(cat) = db.get_category(p.category_id) {
                    println!("[{}] {} - {} (next: {})", marker, cat.path, p.algorithm, p.next_review.format("%Y-%m-%d"));
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn cmd_stats(db: &Database, date_arg: Option<&str>) {
    let date = date_arg.unwrap_or("today");
    println!("Stats for: {}", date);
    if let Ok(sessions) = db.get_all_sessions() {
        let total: i64 = sessions.iter().map(|s| s.duration_secs).sum();
        let h = total / 3600;
        let m = (total % 3600) / 60;
        println!("Total time: {}h{}m, {} sessions", h, m, sessions.len());
    }
}

fn cmd_export(db: &Database, format: Option<&str>) {
    let fmt = format.unwrap_or("jsonl");
    let archive_dir = dirs_data_archive();
    std::fs::create_dir_all(&archive_dir).ok();

    match fmt {
        "jsonl" => {
            use time_manager::data::export::{Exporter, JsonExporter};
            if let Ok(sessions) = db.get_all_sessions() {
                match JsonExporter.export_sessions(&sessions, &archive_dir) {
                    Ok(path) => println!("Exported sessions to: {}", path.display()),
                    Err(e) => eprintln!("Export error: {}", e),
                }
            }
        }
        "csv" => {
            use time_manager::data::export::{CsvExporter, Exporter};
            if let Ok(sessions) = db.get_all_sessions() {
                match CsvExporter.export_sessions(&sessions, &archive_dir) {
                    Ok(path) => println!("Exported sessions to: {}", path.display()),
                    Err(e) => eprintln!("Export error: {}", e),
                }
            }
        }
        _ => eprintln!("Unknown format: {}. Use jsonl or csv.", fmt),
    }
}

fn cmd_import_obsidian(db: &Database, path: Option<&str>) {
    let vault_path = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            eprintln!("Usage: tmd-cli import-obsidian <vault-path>");
            return;
        }
    };

    if !vault_path.exists() {
        eprintln!("Path does not exist: {}", vault_path.display());
        return;
    }

    let entries = time_manager::modules::learning::category::obsidian::vault_parser::parse_vault(&vault_path);
    let mut count = 0;
    for entry in &entries {
        let existing = db.get_all_categories().unwrap_or_default();
        if existing.iter().any(|c| c.path == entry.relative_path) {
            continue;
        }
        let parent_path = entry.relative_path.rsplit_once('/').map(|(p, _)| p);
        let parent_id = parent_path.and_then(|pp| {
            existing.iter().find(|c| c.path == pp).map(|c| c.id)
        });

        match db.insert_category(&time_manager::data::models::CategoryInsert {
            parent_id,
            name: entry.name.clone(),
            path: entry.relative_path.clone(),
            source: Some("obsidian".into()),
            default_quality: None,
            default_understanding_difficulty: None,
            default_memory_difficulty: None,
        }) {
            Ok(_) => count += 1,
            Err(e) => eprintln!("Error inserting '{}': {}", entry.name, e),
        }
    }
    println!("Imported {} new nodes from Obsidian vault.", count);
}

fn dirs_data_archive() -> std::path::PathBuf {
    directories::ProjectDirs::from("", "", "time-manager")
        .map(|d| d.data_dir().join("archive"))
        .unwrap_or_else(|| std::path::PathBuf::from("./archive"))
}

fn print_usage() {
    eprintln!("Usage: tmd-cli <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  status                    Show current status");
    eprintln!("  list-due                  List due predictions");
    eprintln!("  stats [date]              Show statistics");
    eprintln!("  export [format]           Export data (jsonl/csv)");
    eprintln!("  import-obsidian <path>    Import Obsidian vault");
}