use clap::Parser;
use std::path::PathBuf;
use time_manager::cli::Cli;
use time_manager::data::DataFs;

fn main() {
    let cli = Cli::parse();

    let data_dir = cli.data_dir.clone().map(PathBuf::from).unwrap_or_else(|| {
        directories::ProjectDirs::from("com", "time-manager", "time-manager")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./data"))
    });

    if let Err(e) = run_command(cli, data_dir) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_command(cli: Cli, data_dir: PathBuf) -> time_manager::data::Result<()> {
    use time_manager::cli::Commands::*;

    match cli.command {
        Init => {
            let dir = data_dir.clone();
            DataFs::init(data_dir)?;
            println!("Initialized data directory: {:?}", dir);
        }

        TreeCreate {
            name,
            description: _,
            import,
        } => {
            let fs = DataFs::init(data_dir)?;
            fs.create_tree(&name)?;
            println!("Created tree: {}", name);

            if let Some(_import_path) = import {
                println!("Import not implemented yet");
            }
        }

        TreeList { name } => {
            let fs = DataFs::init(data_dir)?;
            if let Some(tree_name) = name {
                let cards = fs.list_cards(&tree_name)?;
                for card in cards {
                    println!("{}", card.path);
                }
            } else {
                let trees = fs.list_trees()?;
                for tree in trees {
                    println!("{}", tree);
                }
            }
        }

        CardCreate { path } => {
            let fs = DataFs::init(data_dir)?;
            let card = time_manager::data::models::Card::new(path.clone());
            fs.save_card(&path, &card)?;
            println!("Created card: {}", path);
        }

        CardList { path } => {
            let fs = DataFs::init(data_dir)?;
            let tree = path.split('/').next().unwrap_or("");
            let cards = fs.list_cards(tree)?;
            for card in cards {
                if card.path.starts_with(&path) {
                    println!("{}", card.path);
                }
            }
        }

        CardPredict { path } => {
            let fs = DataFs::init(data_dir)?;
            let card = fs.get_card(&path)?;
            if let Some(last) = card.review_records.last() {
                println!(
                    "Last review: {} (quality: {})",
                    last.reviewed_at.format("%Y-%m-%d %H:%M"),
                    last.memory_quality
                );
            } else {
                println!("No review history");
            }
        }

        CardLink {
            timer,
            path,
            quality,
        } => {
            let fs = DataFs::init(data_dir)?;
            let mut card = fs.get_card(&path)?;
            let _timer_obj = fs.get_timer(&timer)?;

            use time_manager::data::models::{MemoryQuality, ReviewRecord};
            let mq = MemoryQuality::from_str(&quality).ok_or_else(|| {
                time_manager::data::DataError::InvalidData(format!("Invalid quality: {}", quality))
            })?;

            let record = ReviewRecord {
                timer_path: timer,
                reviewed_at: chrono::Utc::now(),
                memory_quality: mq,
                state_bytes: vec![],
                fsrs_state_bytes: vec![],
            };

            card.review_records.push(record);
            fs.save_card(&path, &card)?;
            println!("Linked timer to card: {}", path);
        }

        PresetCreate { name, description } => {
            let fs = DataFs::init(data_dir)?;
            let preset = time_manager::data::models::Preset {
                name: name.clone(),
                description,
                match_rules: vec![],
            };
            fs.save_preset(&preset)?;
            println!("Created preset: {}", name);
        }

        PresetList { name } => {
            let fs = DataFs::init(data_dir)?;
            if let Some(preset_name) = name {
                let preset = fs.get_preset(&preset_name)?;
                println!("{}: {:?}", preset.name, preset.description);
            } else {
                let presets = fs.list_presets()?;
                for preset in presets {
                    println!("{}", preset.name);
                }
            }
        }

        PresetTrain { name: _ } => {
            println!("Preset training not implemented yet");
        }

        TimerStart { name: _ } => {
            println!("Timer not implemented yet - use GUI");
        }

        TimerPause => {
            println!("Timer not implemented yet - use GUI");
        }

        TimerStop => {
            println!("Timer not implemented yet - use GUI");
        }

        TimerGet { name } => {
            let fs = DataFs::init(data_dir)?;
            let timer = fs.get_timer(&name)?;
            println!("Started: {}", timer.started_at);
            if let Some(stopped) = timer.stopped_at {
                println!("Stopped: {}", stopped);
            }
            println!("Duration: {}ms", timer.duration_ms);
        }

        TodoCreate { content } => {
            let fs = DataFs::init(data_dir)?;
            let todo = time_manager::data::models::Todo::new(content);
            let id = todo.id;
            fs.save_todo(&todo)?;
            println!("Created todo: {}", id);
        }

        TodoList => {
            let fs = DataFs::init(data_dir)?;
            let todos = fs.list_todos()?;
            for todo in todos {
                println!("{}: {}", todo.id, todo.content);
            }
        }

        TodoToSchedule { id, start, end } => {
            let fs = DataFs::init(data_dir)?;
            let uuid = uuid::Uuid::parse_str(&id).map_err(|_| {
                time_manager::data::DataError::InvalidData(format!("Invalid UUID: {}", id))
            })?;

            let ics_content = format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nDTSTART:{}\nDTEND:{}\nSUMMARY:Todo\nEND:VEVENT\nEND:VCALENDAR",
                start, end
            );

            let schedule_id = uuid::Uuid::new_v4();
            fs.save_schedule(&schedule_id, &ics_content)?;
            fs.delete_todo(&uuid)?;
            println!("Converted todo to schedule: {}", schedule_id);
        }

        ScheduleCreate {
            start,
            end,
            summary,
        } => {
            let fs = DataFs::init(data_dir)?;
            let id = uuid::Uuid::new_v4();
            let ics_content = format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nDTSTART:{}\nDTEND:{}\nSUMMARY:{}\nEND:VEVENT\nEND:VCALENDAR",
                start, end, summary
            );
            fs.save_schedule(&id, &ics_content)?;
            println!("Created schedule: {}", id);
        }

        ScheduleList => {
            let fs = DataFs::init(data_dir)?;
            let schedules = fs.list_schedules()?;
            for (id, _content) in schedules {
                println!("{}", id);
            }
        }

        ReviewList { urgency: _ } => {
            println!("Review list not implemented yet");
        }
    }

    Ok(())
}
