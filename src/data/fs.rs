use std::path::PathBuf;
use walkdir::WalkDir;

use crate::data::models::{Card, Preset, Timer, Todo};
use crate::data::{DataError, Result};

#[derive(Clone)]
pub struct DataFs {
    data_dir: PathBuf,
}

impl DataFs {
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
    
    pub fn init(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir).map_err(|e| DataError::Io(e.to_string()))?;
        std::fs::create_dir_all(data_dir.join("categories")).map_err(|e| DataError::Io(e.to_string()))?;
        std::fs::create_dir_all(data_dir.join("timers")).map_err(|e| DataError::Io(e.to_string()))?;
        std::fs::create_dir_all(data_dir.join("presets")).map_err(|e| DataError::Io(e.to_string()))?;
        std::fs::create_dir_all(data_dir.join("todos")).map_err(|e| DataError::Io(e.to_string()))?;
        std::fs::create_dir_all(data_dir.join("schedules")).map_err(|e| DataError::Io(e.to_string()))?;

        let presets_dir = data_dir.join("presets");
        let default_preset_path = presets_dir.join("default.json");
        if !default_preset_path.exists() {
            let default_preset = Preset::default_preset();
            let json = serde_json::to_string_pretty(&default_preset).map_err(|e| DataError::Json(e.to_string()))?;
            std::fs::write(&default_preset_path, json).map_err(|e| DataError::Io(e.to_string()))?;
        }

        Ok(Self { data_dir })
    }

    pub fn create_tree(&self, name: &str) -> Result<()> {
        let tree_dir = self.data_dir.join("categories").join(name);
        std::fs::create_dir_all(&tree_dir).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(())
    }
    
    pub fn create_folder(&self, path: &str) -> Result<()> {
        let (tree, folder_path) = Self::parse_path(path)?;
        let dir_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(folder_path);
        std::fs::create_dir_all(&dir_path).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn list_trees(&self) -> Result<Vec<String>> {
        let categories_dir = self.data_dir.join("categories");
        if !categories_dir.exists() {
            return Ok(Vec::new());
        }

        let trees: Vec<String> = WalkDir::new(&categories_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        Ok(trees)
    }

    fn parse_path(path: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() < 2 {
            return Err(DataError::InvalidPath(path.into()));
        }
        Ok((parts[0].into(), parts[1].into()))
    }

    pub fn save_card(&self, path: &str, card: &Card) -> Result<()> {
        let (tree, card_path) = Self::parse_path(path)?;
        let file_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", card_path));

        let parent_dir = file_path
            .parent()
            .ok_or_else(|| DataError::InvalidPath(path.into()))?;
        std::fs::create_dir_all(parent_dir).map_err(|e| DataError::Io(e.to_string()))?;

        let json = serde_json::to_string_pretty(card).map_err(|e| DataError::Json(e.to_string()))?;
        std::fs::write(&file_path, json).map_err(|e| DataError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn get_card(&self, path: &str) -> Result<Card> {
        let (tree, card_path) = Self::parse_path(path)?;
        let file_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", card_path));

        if !file_path.exists() {
            return Err(DataError::CardNotFound(path.into()));
        }

        let json = std::fs::read_to_string(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        let card: Card = serde_json::from_str(&json).map_err(|e| DataError::Json(e.to_string()))?;

        Ok(card)
    }

    pub fn list_cards(&self, tree: &str) -> Result<Vec<(String, Card)>> {
        let tree_dir = self.data_dir.join("categories").join(tree);
        if !tree_dir.exists() {
            return Ok(Vec::new());
        }

        let cards: Vec<(String, Card)> = WalkDir::new(&tree_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let path = e.path();
                let card_path = path
                    .strip_prefix(&self.data_dir.join("categories"))
                    .ok()?;
                let card_path_str = card_path
                    .to_string_lossy()
                    .replace(".json", "");
                
                let json = std::fs::read_to_string(path).ok()?;
                let card: Card = serde_json::from_str(&json).ok()?;
                Some((card_path_str, card))
            })
            .collect();

        Ok(cards)
    }

    pub fn save_timer(&self, timer: &Timer) -> Result<()> {
        let filename = timer.filename();
        let file_path = self
            .data_dir
            .join("timers")
            .join(format!("{}.json", filename));

        let json = serde_json::to_string_pretty(timer).map_err(|e| DataError::Json(e.to_string()))?;
        std::fs::write(&file_path, json).map_err(|e| DataError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn get_timer(&self, filename: &str) -> Result<Timer> {
        let file_path = self
            .data_dir
            .join("timers")
            .join(format!("{}.json", filename));

        if !file_path.exists() {
            return Err(DataError::TimerNotFound(filename.into()));
        }

        let json = std::fs::read_to_string(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        let timer: Timer = serde_json::from_str(&json).map_err(|e| DataError::Json(e.to_string()))?;

        Ok(timer)
    }

    pub fn list_timers(&self) -> Result<Vec<Timer>> {
        let timers_dir = self.data_dir.join("timers");
        if !timers_dir.exists() {
            return Ok(Vec::new());
        }

        let timers: Vec<Timer> = WalkDir::new(&timers_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let json = std::fs::read_to_string(e.path()).ok()?;
                serde_json::from_str::<Timer>(&json).ok()
            })
            .collect();

        Ok(timers)
    }

    pub fn save_preset(&self, preset: &Preset) -> Result<()> {
        let file_path = self
            .data_dir
            .join("presets")
            .join(format!("{}.json", preset.name));

        let json = serde_json::to_string_pretty(preset).map_err(|e| DataError::Json(e.to_string()))?;
        std::fs::write(&file_path, json).map_err(|e| DataError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn get_preset(&self, name: &str) -> Result<Preset> {
        let file_path = self.data_dir.join("presets").join(format!("{}.json", name));

        if !file_path.exists() {
            return Err(DataError::PresetNotFound(name.into()));
        }

        let json = std::fs::read_to_string(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        let preset: Preset = serde_json::from_str(&json).map_err(|e| DataError::Json(e.to_string()))?;

        Ok(preset)
    }

    pub fn list_presets(&self) -> Result<Vec<Preset>> {
        let presets_dir = self.data_dir.join("presets");
        if !presets_dir.exists() {
            return Ok(Vec::new());
        }

        let presets: Vec<Preset> = WalkDir::new(&presets_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let json = std::fs::read_to_string(e.path()).ok()?;
                serde_json::from_str::<Preset>(&json).ok()
            })
            .collect();

        Ok(presets)
    }

    pub fn save_todo(&self, todo: &Todo) -> Result<()> {
        let file_path = self
            .data_dir
            .join("todos")
            .join(format!("{}.ics", todo.id));

        let ics = todo.to_ics();
        std::fs::write(&file_path, ics).map_err(|e| DataError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn get_todo(&self, id: &uuid::Uuid) -> Result<Todo> {
        let file_path = self.data_dir.join("todos").join(format!("{}.ics", id));

        if !file_path.exists() {
            return Err(DataError::TodoNotFound(id.to_string()));
        }

        let ics = std::fs::read_to_string(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        let todo = Todo::from_ics(&ics)
            .ok_or_else(|| DataError::InvalidData("Failed to parse ICS".to_string()))?;

        Ok(todo)
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        let todos_dir = self.data_dir.join("todos");
        if !todos_dir.exists() {
            return Ok(Vec::new());
        }

        let todos: Vec<Todo> = WalkDir::new(&todos_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "ics")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let ics = std::fs::read_to_string(e.path()).ok()?;
                Todo::from_ics(&ics)
            })
            .collect();

        Ok(todos)
    }

    pub fn delete_todo(&self, id: &uuid::Uuid) -> Result<()> {
        let file_path = self.data_dir.join("todos").join(format!("{}.ics", id));
        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        }
        Ok(())
    }

    pub fn save_schedule(&self, id: &uuid::Uuid, ics_content: &str) -> Result<()> {
        let file_path = self.data_dir.join("schedules").join(format!("{}.ics", id));

        std::fs::write(&file_path, ics_content).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn get_schedule(&self, id: &uuid::Uuid) -> Result<String> {
        let file_path = self.data_dir.join("schedules").join(format!("{}.ics", id));

        if !file_path.exists() {
            return Err(DataError::ScheduleNotFound(id.to_string()));
        }

        let content = std::fs::read_to_string(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(content)
    }

    pub fn list_schedules(&self) -> Result<Vec<(uuid::Uuid, String)>> {
        let schedules_dir = self.data_dir.join("schedules");
        if !schedules_dir.exists() {
            return Ok(Vec::new());
        }

        let schedules: Vec<(uuid::Uuid, String)> = WalkDir::new(&schedules_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "ics")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let filename = e.file_name().to_string_lossy();
                let id_str = filename.replace(".ics", "");
                let id = uuid::Uuid::parse_str(&id_str).ok()?;
                let content = std::fs::read_to_string(e.path()).ok()?;
                Some((id, content))
            })
            .collect();

        Ok(schedules)
    }

    pub fn delete_card(&self, path: &str) -> Result<()> {
        let (tree, card_path) = Self::parse_path(path)?;
        let file_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", card_path));
        
        if !file_path.exists() {
            return Err(DataError::CardNotFound(path.into()));
        }
        
        std::fs::remove_file(&file_path)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(())
    }

    pub fn delete_folder(&self, path: &str) -> Result<()> {
        let (tree, folder_path) = Self::parse_path(path)?;
        let dir_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(folder_path);
        
        if !dir_path.exists() {
            return Err(DataError::FolderNotFound(path.into()));
        }
        
        std::fs::remove_dir_all(&dir_path)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(())
    }

    pub fn rename_card(&self, old_path: &str, new_name: &str) -> Result<String> {
        let (tree, old_card_path) = Self::parse_path(old_path)?;
        
        let old_path_buf = std::path::Path::new(&old_card_path);
        let parent = old_path_buf.parent()
            .ok_or_else(|| DataError::InvalidPath(old_path.into()))?;
        let new_card_path = parent.join(new_name);
        let new_card_path_str = new_card_path.to_string_lossy();
        let new_full_path = format!("{}/{}", tree, new_card_path_str);
        
        let old_file = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", old_card_path));
        let new_file = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", new_card_path_str));
        
        if new_file.exists() {
            return Err(DataError::NodeAlreadyExists(new_full_path));
        }
        
        std::fs::rename(&old_file, &new_file)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(new_full_path)
    }
}
