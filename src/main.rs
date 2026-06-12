use std::path::PathBuf;

fn main() -> iced::Result {
    let data_dir = directories::ProjectDirs::from("com", "time-manager", "time-manager")
        .map(|p| p.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data"));
    
    time_manager::app::App::run(data_dir)
}