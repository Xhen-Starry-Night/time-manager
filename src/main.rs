use time_manager::data::Database;

fn main() {
    match Database::open_default() {
        Ok(db) => {
            println!("Database opened at: {}", db.db_path().display());
        }
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    }
}
