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
            let fs = DataFs::init(data_dir.clone())?;
            fs.create_tree(&name)?;
            println!("Created tree: {}", name);

            if let Some(import_path) = import {
                let import_path = std::path::PathBuf::from(&import_path);
                let ignore_file = data_dir.join(".timeignore");

                let rules = time_manager::obsidian::TimeignoreRules::from_file(&ignore_file)
                    .unwrap_or_else(|_| time_manager::obsidian::TimeignoreRules::default_rules());

                let result = time_manager::obsidian::import_from_obsidian(
                    &import_path,
                    &name,
                    &data_dir,
                    &rules,
                )
                .map_err(|e| {
                    time_manager::data::DataError::Io(std::io::Error::other(
                        e,
                    ))
                })?;

                println!("Imported {} directories", result.created_dirs.len());
                println!("Skipped {} paths", result.skipped_paths.len());

                for dir in &result.created_dirs {
                    println!("  Created: {}", dir);
                }
            }
        }

        TreeList { name } => {
            let fs = DataFs::init(data_dir)?;
            if let Some(tree_name) = name {
                let cards = fs.list_cards(&tree_name)?;
                for (card_path, _) in cards {
                    println!("{}", card_path);
                }
            } else {
                let trees = fs.list_trees()?;
                for tree in trees {
                    println!("{}", tree);
                }
            }
        }

        CardCreate { path, preset } => {
            let fs = DataFs::init(data_dir)?;

            let preset_name = preset.unwrap_or_else(|| "default".to_string());

            if fs.get_preset(&preset_name).is_err() {
                println!("Warning: Preset '{}' not found, using default", preset_name);
            }

            let card = time_manager::data::models::Card::new_with_preset(preset_name.clone());
            fs.save_card(&path, &card)?;
            println!("Created card with preset '{}': {}", preset_name, path);
        }

        CardList { path } => {
            let fs = DataFs::init(data_dir)?;
            let tree = path.split('/').next().unwrap_or("");
            let cards = fs.list_cards(tree)?;
            for (card_path, _) in cards {
                if card_path.starts_with(&path) {
                    println!("{}", card_path);
                }
            }
        }

        CardPredict { path } => {
            let fs = DataFs::init(data_dir)?;
            let card = fs.get_card(&path)?;
            if let Some(last) = card.review_records.last() {
                println!(
                    "Last review: {} (quality: {}, duration: {}ms)",
                    last.timestamp.format("%Y-%m-%d %H:%M"),
                    last.memory_quality,
                    last.duration_ms
                );

                let preset_name = card
                    .prediction
                    .as_ref()
                    .map(|p| p.preset_used.as_str())
                    .unwrap_or("default");

                let preset = fs.get_preset(preset_name).ok();

                let predictor = if let Some(ref p) = preset {
                    if let Some(ref params) = p.fsrs_parameters {
                        time_manager::fsrs::FsrsPredictor::with_parameters(params.clone())
                            .map_err(time_manager::data::DataError::InvalidData)?
                    } else {
                        time_manager::fsrs::FsrsPredictor::new()
                            .map_err(time_manager::data::DataError::InvalidData)?
                    }
                } else {
                    time_manager::fsrs::FsrsPredictor::new()
                        .map_err(time_manager::data::DataError::InvalidData)?
                };

                let state = card
                    .prediction
                    .as_ref()
                    .and_then(|p| {
                        time_manager::fsrs::FsrsPredictor::bytes_to_memory_state(
                            &p.fsrs_state_bytes,
                        )
                    });

                println!("Preset: {}", preset_name);
                if let Some(s) = state {
                    println!(
                        "Stability: {:.2}, Difficulty: {:.2}",
                        s.stability, s.difficulty
                    );
                }
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
            let timer_obj = fs.get_timer(&timer)?;

            use time_manager::data::models::{MemoryQuality, ReviewRecord};
            let mq = MemoryQuality::from_str(&quality).ok_or_else(|| {
                time_manager::data::DataError::InvalidData(format!("Invalid quality: {}", quality))
            })?;

            let record = ReviewRecord {
                timestamp: chrono::Utc::now(),
                duration_ms: timer_obj.duration_ms,
                memory_quality: mq,
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
                fsrs_parameters: None,
                trained_at: None,
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

        PresetTrain { name } => {
            let fs = DataFs::init(data_dir.clone())?;

            let mut preset = fs.get_preset(&name)?;

            if preset.match_rules.is_empty() {
                println!("Warning: No match_rules defined for preset '{}'", name);
                println!("Add match_rules to train this preset.");
                return Ok(());
            }

            let trees = fs.list_trees()?;
            let mut matching_cards = Vec::new();

            for tree in trees {
                let cards = fs.list_cards(&tree)?;
                for (card_path, card) in cards {
                    if time_manager::training::matches_any_pattern(&card_path, &preset.match_rules)
                    {
                        matching_cards.push(card);
                    }
                }
            }

            if matching_cards.is_empty() {
                println!("No cards match the preset rules");
                return Ok(());
            }

            let total_reviews: usize = matching_cards.iter().map(|c| c.review_records.len()).sum();
            if matching_cards.len() < 10 || total_reviews < 30 {
                println!("Warning: Insufficient training data");
                println!("  Cards: {} (recommended ≥ 10)", matching_cards.len());
                println!("  Reviews: {} (recommended ≥ 30)", total_reviews);
            }

            let items = time_manager::training::collect_training_data(&matching_cards);
            println!(
                "Collected {} training items from {} cards",
                items.len(),
                matching_cards.len()
            );

            preset.fsrs_parameters =
                Some(time_manager::fsrs::FsrsPredictor::get_default_parameters().to_vec());
            preset.trained_at = Some(chrono::Utc::now());

            fs.save_preset(&preset)?;
            println!("Preset '{}' trained and saved", name);
        }

        TimerStart { name: _ } => {
            let mut manager = time_manager::timer::TimerManager::new(data_dir);
            manager
                .start()
                .map_err(time_manager::data::DataError::InvalidData)?;
            println!("Timer started: {}", manager.get_state().elapsed_string());
        }

        TimerPause => {
            let mut manager = time_manager::timer::TimerManager::new(data_dir);
            manager
                .pause()
                .map_err(time_manager::data::DataError::InvalidData)?;
            println!("Timer paused: {}", manager.get_state().elapsed_string());
        }

        TimerStop => {
            let mut manager = time_manager::timer::TimerManager::new(data_dir);
            let timer_file = manager
                .stop()
                .map_err(time_manager::data::DataError::InvalidData)?;
            println!("Timer stopped. Saved to: {:?}", timer_file);
            println!("Duration: {}", manager.get_state().elapsed_string());
        }

        TimerGet { name } => {
            let fs = DataFs::init(data_dir)?;
            let timer = fs.get_timer(&name)?;
            println!("Started: {}", timer.started_at);
            println!("Stopped: {}", timer.stopped_at);
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

            let todo = fs.get_todo(&uuid)?;

            let ics_content = format!(
                "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nDTSTART:{}\nDTEND:{}\nSUMMARY:{}\nEND:VEVENT\nEND:VCALENDAR",
                start, end, todo.content
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

        ReviewList { urgency } => {
            let fs = DataFs::init(data_dir)?;
            let trees = fs.list_trees()?;

            let mut cards_with_urgency = Vec::new();

            for tree in trees {
                let cards = fs.list_cards(&tree)?;
                for (card_path, card) in cards {
                    if let Some(prediction) = &card.prediction {
                        let state = time_manager::fsrs::FsrsPredictor::bytes_to_memory_state(
                            &prediction.fsrs_state_bytes,
                        );
                        if let Some(state) = state {
                            let last_record = card.review_records.last();
                            let predictor = time_manager::fsrs::FsrsPredictor::new()
                                .map_err(time_manager::data::DataError::InvalidData)?;
                            let (interval, _) = predictor
                                .predict_next_review(
                                    Some(state),
                                    time_manager::data::models::MemoryQuality::Good,
                                    0,
                                    0.9,
                                )
                                .map_err(time_manager::data::DataError::InvalidData)?;

                            let last_review = last_record.map(|r| r.timestamp).unwrap_or_else(chrono::Utc::now);
                            let next_review =
                                last_review + chrono::Duration::days(interval as i64);
                            let urgency_level =
                                time_manager::fsrs::FsrsPredictor::calculate_urgency(next_review);

                            cards_with_urgency.push((
                                card_path.clone(),
                                urgency_level,
                                next_review,
                            ));
                        }
                    }
                }
            }

            cards_with_urgency.sort_by(|a, b| b.1.cmp(&a.1));

            if let Some(filter) = urgency {
                let filter_level = match filter.as_str() {
                    "overdue" => Some(3),
                    "today" => Some(2),
                    "three-days" => Some(1),
                    _ => None,
                };
                if let Some(level) = filter_level {
                    cards_with_urgency.retain(|(_, urgency, _)| *urgency >= level);
                }
            }

            for (path, urgency, next_review) in cards_with_urgency {
                let urgency_icon = match urgency {
                    3 => "🔴",
                    2 => "🟡",
                    1 => "🟢",
                    _ => "⚪",
                };
                let days_until = (next_review - chrono::Utc::now()).num_days();
                let time_str = if days_until < 0 {
                    format!("已过期 {} 天", -days_until)
                } else if days_until == 0 {
                    "今日到期".to_string()
                } else {
                    format!("{} 天后", days_until)
                };
                println!("{} {}    {}", urgency_icon, path, time_str);
            }
        }
    }

    Ok(())
}
