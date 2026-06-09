use time_manager::data::Database;
use tracing_subscriber::{fmt, EnvFilter};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::builder()
                .parse_lossy("time_manager=debug,tmd_cli=debug,fsrs=debug,info")
        });

    fmt()
        .with_env_filter(filter)
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .compact()
        .init();

    tracing::info!("tmd-cli starting");
}

fn main() {
    init_tracing();

    let args: Vec<String> = std::env::args().collect();
    tracing::debug!("Command line args: {:?}", args);

    if args.len() < 2 {
        print_usage();
        return;
    }

    let db = match Database::open_default() {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Failed to open database: {}", e);
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

    tracing::info!("tmd-cli finished");
}

fn cmd_status(db: &Database) {
    tracing::debug!("Executing status command");

    if let Ok(cats) = db.get_all_categories() {
        tracing::trace!("Loaded {} categories", cats.len());
        println!("Categories: {} nodes", cats.len());
    }
    if let Ok(sessions) = db.get_all_sessions() {
        tracing::trace!("Loaded {} sessions", sessions.len());
        let total_secs: i64 = sessions.iter().map(|s| s.duration_secs).sum();
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        println!("Sessions: {} total, {}h{}m total time", sessions.len(), h, m);
    }
}

fn cmd_list_due(db: &Database) {
    tracing::debug!("Executing list-due command");

    let now = chrono::Utc::now();
    match db.get_due_predictions(&now) {
        Ok(predictions) => {
            tracing::trace!("Found {} due predictions", predictions.len());

            if predictions.is_empty() {
                println!("No due predictions.");
                return;
            }
            for p in &predictions {
                let overdue = now > p.next_review;
                let marker = if overdue { "OVERDUE" } else { "DUE" };
                if let Ok(cat) = db.get_category(p.category_id) {
                    tracing::trace!(
                        category_id = p.category_id,
                        category_path = %cat.path,
                        algorithm = %p.algorithm,
                        next_review = %p.next_review.format("%Y-%m-%d"),
                        overdue,
                        "Due prediction"
                    );
                    println!("[{}] {} - {} (next: {})", marker, cat.path, p.algorithm, p.next_review.format("%Y-%m-%d"));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get due predictions: {}", e);
            eprintln!("Error: {}", e);
        }
    }
}

fn cmd_stats(db: &Database, date_arg: Option<&str>) {
    let date = date_arg.unwrap_or("today");
    tracing::debug!("Executing stats command for: {}", date);
    println!("Stats for: {}", date);

    if let Ok(sessions) = db.get_all_sessions() {
        tracing::trace!("Processing {} sessions for stats", sessions.len());
        let total: i64 = sessions.iter().map(|s| s.duration_secs).sum();
        let h = total / 3600;
        let m = (total % 3600) / 60;
        println!("Total time: {}h{}m, {} sessions", h, m, sessions.len());
    }
}

fn cmd_export(db: &Database, format: Option<&str>) {
    let fmt = format.unwrap_or("jsonl");
    tracing::info!("Executing export command, format: {}", fmt);

    let archive_dir = dirs_data_archive();
    std::fs::create_dir_all(&archive_dir).ok();
    tracing::debug!("Archive directory: {:?}", archive_dir);

    match fmt {
        "jsonl" => {
            use time_manager::data::export::{Exporter, JsonExporter};
            if let Ok(sessions) = db.get_all_sessions() {
                tracing::trace!("Exporting {} sessions to JSONL", sessions.len());
                match JsonExporter.export_sessions(&sessions, &archive_dir) {
                    Ok(path) => {
                        tracing::info!("Exported sessions to: {:?}", path);
                        println!("Exported sessions to: {}", path.display());
                    }
                    Err(e) => {
                        tracing::error!("Export error: {}", e);
                        eprintln!("Export error: {}", e);
                    }
                }
            }
        }
        "csv" => {
            use time_manager::data::export::{CsvExporter, Exporter};
            if let Ok(sessions) = db.get_all_sessions() {
                tracing::trace!("Exporting {} sessions to CSV", sessions.len());
                match CsvExporter.export_sessions(&sessions, &archive_dir) {
                    Ok(path) => {
                        tracing::info!("Exported sessions to: {:?}", path);
                        println!("Exported sessions to: {}", path.display());
                    }
                    Err(e) => {
                        tracing::error!("Export error: {}", e);
                        eprintln!("Export error: {}", e);
                    }
                }
            }
        }
        _ => {
            tracing::warn!("Unknown export format: {}", fmt);
            eprintln!("Unknown format: {}. Use jsonl or csv.", fmt);
        }
    }
}

fn cmd_import_obsidian(db: &Database, path: Option<&str>) {
    let vault_path = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            tracing::warn!("No vault path provided");
            eprintln!("Usage: tmd-cli import-obsidian <vault-path>");
            return;
        }
    };

    tracing::info!("Importing Obsidian vault from: {:?}", vault_path);

    if !vault_path.exists() {
        tracing::error!("Path does not exist: {:?}", vault_path);
        eprintln!("Path does not exist: {}", vault_path.display());
        return;
    }

    let entries = time_manager::modules::learning::category::obsidian::vault_parser::parse_vault(&vault_path);
    tracing::debug!("Found {} entries in vault", entries.len());

    let mut count = 0;
    for entry in &entries {
        let existing = db.get_all_categories().unwrap_or_default();
        if existing.iter().any(|c| c.path == entry.relative_path) {
            tracing::trace!("Skipping existing entry: {}", entry.relative_path);
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
            node_type: if entry.is_dir {
                time_manager::data::models::NodeType::Directory
            } else {
                time_manager::data::models::NodeType::Learning
            },
            source: Some("obsidian".into()),
            default_quality: None,
            default_understanding_difficulty: None,
            default_memory_difficulty: None,
        }) {
            Ok(id) => {
                tracing::trace!(
                    id,
                    path = %entry.relative_path,
                    is_dir = entry.is_dir,
                    "Inserted category"
                );
                count += 1;
            }
            Err(e) => {
                tracing::error!("Error inserting '{}': {}", entry.name, e);
                eprintln!("Error inserting '{}': {}", entry.name, e);
            }
        }
    }

    tracing::info!("Imported {} new nodes from Obsidian vault", count);
    println!("Imported {} new nodes from Obsidian vault.", count);
}

fn dirs_data_archive() -> std::path::PathBuf {
    directories::ProjectDirs::from("", "", "time-manager")
        .map(|d| d.data_dir().join("archive"))
        .unwrap_or_else(|| std::path::PathBuf::from("./archive"))
}

fn print_usage() {
    tracing::debug!("Printing usage");
    eprintln!("Usage: tmd-cli <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  status                    Show current status");
    eprintln!("  list-due                  List due predictions");
    eprintln!("  stats [date]              Show statistics");
    eprintln!("  export [format]           Export data (jsonl/csv)");
    eprintln!("  import-obsidian <path>    Import Obsidian vault");
}
