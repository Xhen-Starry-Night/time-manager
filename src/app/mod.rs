use std::path::PathBuf;
use std::sync::Arc;

use iced::{application, Element, Task};
use iced::widget::{button, column, row, text, container, rule, text_input, pick_list, Space};
use iced::Length;
use chrono::{Datelike, Utc};

use crate::data::config::Config;
use crate::data::DataFs;
use crate::data::models::{Card, Preset, Todo};
use crate::gui::{Message, TabId, Modal, DataSnapshot, NodeType};
use crate::gui::styles::AppTheme;
use crate::gui::components::{tree_view::TreeNode, ModalView, NewTodoForm, NewNodeModal};
use crate::timer::TimerManager;

pub mod category_tab;
pub mod review_tab;
pub mod timer_tab;
pub mod schedule_tab;
pub mod todo_tab;
pub mod preset_tab;
pub mod settings_tab;

use category_tab::CategoryTabState;
use review_tab::ReviewTabState;
use timer_tab::TimerTabState;
use schedule_tab::ScheduleTabState;
use todo_tab::TodoTabState;
use preset_tab::PresetTabState;
use settings_tab::SettingsTabState;

pub struct App {
    pub default_preset: String,
    pub theme: AppTheme,
    active_tab: TabId,
    data_dir: PathBuf,
    data_fs: DataFs,
    
    category_tab: CategoryTabState,
    review_tab: ReviewTabState,
    timer_tab: TimerTabState,
    schedule_tab: ScheduleTabState,
    todo_tab: TodoTabState,
    preset_tab: PresetTabState,
    settings_tab: SettingsTabState,
    
    timer_manager: TimerManager,
    error_message: Option<String>,
    modal: Option<Modal>,
}

impl App {
    pub fn run(data_dir: PathBuf) -> iced::Result {
        static DATA_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        DATA_DIR.get_or_init(|| data_dir);
        
        application(
            || {
                let data_dir = DATA_DIR.get().cloned().unwrap_or_else(|| {
                    directories::ProjectDirs::from("com", "time-manager", "time-manager")
                            .map(|p| p.data_dir().to_path_buf())
                            .unwrap_or_else(|| PathBuf::from("./data"))
                });
                
                let config = Config::load(&Config::config_path()).unwrap_or_default();
                let data_dir = config.data_dir.clone()
                    .map(PathBuf::from)
                    .unwrap_or(data_dir);
                let default_preset = config.default_preset.clone()
                    .unwrap_or_else(|| "default".into());
                let theme = config.theme.as_deref().map(AppTheme::parse).unwrap_or_else(AppTheme::platform_default);
                
                let data_fs = DataFs::init(data_dir.clone()).expect("Failed to init data dir");
                let timer_manager = TimerManager::new(data_dir.clone());
                let data_fs_arc = Arc::new(data_fs);
                
                let timer_tab = TimerTabState::new(timer_manager.get_state().clone(), &default_preset);
                let state = App {
                    default_preset,
                    theme,
                    active_tab: TabId::Category,
                    data_dir: data_dir.clone(),
                    data_fs: (*data_fs_arc).clone(),
                    
                    category_tab: CategoryTabState::default(),
                    review_tab: ReviewTabState::default(),
                    timer_tab,
                    schedule_tab: ScheduleTabState::default(),
                    todo_tab: TodoTabState::default(),
                    preset_tab: PresetTabState::default(),
                    settings_tab: SettingsTabState::new(
                        Config::config_dir(),
                        Config::config_path(),
                        config,
                        &data_dir.to_string_lossy(),
                        theme,
                    ),
                    
                    timer_manager,
                    error_message: None,
                    modal: None,
                };
                
                let load_task = Task::future(async move {
                    let trees = data_fs_arc.list_trees().unwrap_or_default();
                    let mut all_cards = Vec::new();
                    for tree in &trees {
                        let cards = data_fs_arc.list_cards(tree).unwrap_or_default();
                        all_cards.extend(cards);
                    }
                    let presets = data_fs_arc.list_presets().unwrap_or_default();
                    let todos = data_fs_arc.list_todos().unwrap_or_default();
                    
                    Message::DataLoaded(Ok(DataSnapshot {
                        trees,
                        cards: all_cards,
                        presets,
                        todos,
                    }))
                });

                (state, load_task)
            },
            App::update,
            App::view,
        )
        .window(iced::window::Settings {
            #[cfg(target_os = "linux")]
            transparent: true,
            ..Default::default()
        })
        .title(App::title)
        .theme(|app: &App| app.theme.to_iced_theme())
        .subscription(App::subscription)
        .run()
    }
    
    fn title(&self) -> String {
        String::from("Time Manager")
    }
    
    fn search_nodes(&self, query: &str) -> Vec<category_tab::SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }
        
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        fn search_recursive(
            nodes: &[TreeNode],
            query: &str,
            results: &mut Vec<category_tab::SearchResult>,
        ) {
            for node in nodes {
                if node.name.to_lowercase().contains(query) {
                    results.push(category_tab::SearchResult {
                        path: node.path.clone(),
                        name: node.name.clone(),
                        is_card: node.is_card,
                    });
                    if results.len() >= 10 {
                        return;
                    }
                }
                if !node.children.is_empty() {
                    search_recursive(&node.children, query, results);
                }
            }
        }
        
        search_recursive(&self.category_tab.tree_nodes, &query_lower, &mut results);
        results
    }
    
    fn flatten_tree(node: &TreeNode) -> Vec<TreeNode> {
        let mut result = vec![TreeNode {
            name: node.name.clone(),
            path: node.path.clone(),
            is_card: node.is_card,
            children: Vec::new(),
        }];
        for child in &node.children {
            result.extend(Self::flatten_tree(child));
        }
        result
    }
    
    fn count_tree_children(&self, path: &str) -> usize {
        fn count_recursive(nodes: &[TreeNode], target_path: &str) -> usize {
            for node in nodes {
                if node.path == target_path {
                    return count_all_children(node);
                }
                let count = count_recursive(&node.children, target_path);
                if count > 0 {
                    return count;
                }
            }
            0
        }
        
        fn count_all_children(node: &TreeNode) -> usize {
            if node.children.is_empty() {
                return 0;
            }
            node.children.len() + node.children.iter().map(count_all_children).sum::<usize>()
        }
        
        count_recursive(&self.category_tab.tree_nodes, path)
    }
    
    fn subscription(&self) -> iced::Subscription<Message> {
        use crate::timer::TimerState;
        
        if matches!(self.timer_tab.state, TimerState::Running { .. }) {
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| Message::TimerTick(0))
        } else {
            iced::Subscription::none()
        }
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab_id) => {
                self.active_tab = tab_id;
                Task::none()
            }
            
            Message::DataLoaded(result) => {
                match result {
                    Ok(snapshot) => {
                        let tree_nodes = self.build_tree_nodes(&snapshot.trees, &snapshot.cards);
                        self.category_tab.tree_nodes = tree_nodes.clone();
                        self.category_tab.tree_view.expand_all(&tree_nodes);
                        
                        self.review_tab.cards = snapshot.cards.clone();
                        self.preset_tab.presets = snapshot.presets.clone();
                        self.todo_tab.todos = snapshot.todos.clone();
                        self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }

            Message::PresetsLoaded(result) => {
                match result {
                    Ok(presets) => {
                        self.preset_tab.presets = presets;
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }

            Message::PresetCreateOpen => {
                self.preset_tab.reset_form();
                self.preset_tab.show_form = true;
                Task::none()
            }

            Message::PresetEditOpen(name) => {
                self.preset_tab.load_from_preset(&name);
                self.preset_tab.show_form = true;
                Task::none()
            }

            Message::PresetFormDismissed => {
                self.preset_tab.show_form = false;
                self.preset_tab.reset_form();
                Task::none()
            }

            Message::PresetFormNameChanged(val) => {
                self.preset_tab.form_name = val;
                Task::none()
            }

            Message::PresetFormDescriptionChanged(val) => {
                self.preset_tab.form_description = val;
                Task::none()
            }

            Message::PresetFormMatchRuleAdded => {
                self.preset_tab.form_match_rules.push(String::new());
                Task::none()
            }

            Message::PresetFormMatchRuleChanged(index, val) => {
                if index < self.preset_tab.form_match_rules.len() {
                    self.preset_tab.form_match_rules[index] = val;
                }
                Task::none()
            }

            Message::PresetFormMatchRuleRemoved(index) => {
                if index < self.preset_tab.form_match_rules.len() {
                    self.preset_tab.form_match_rules.remove(index);
                }
                Task::none()
            }

            Message::PresetFormSaveRequested => {
                match self.preset_tab.validate() {
                    Ok(preset) => {
                        let data_fs = self.data_fs.clone();
                        let old_name = self.preset_tab.editing_name.clone();
                        self.preset_tab.show_form = false;
                        self.preset_tab.reset_form();
                        Task::perform(
                            async move {
                                if let Some(ref old) = old_name {
                                    if old != &preset.name {
                                        data_fs.rename_preset(old, &preset.name).map_err(|e| e.to_string())?;
                                        data_fs.list_presets().map_err(|e| e.to_string())
                                    } else {
                                        data_fs.save_preset(&preset).map_err(|e| e.to_string())?;
                                        data_fs.list_presets().map_err(|e| e.to_string())
                                    }
                                } else {
                                    data_fs.save_preset(&preset).map_err(|e| e.to_string())?;
                                    data_fs.list_presets().map_err(|e| e.to_string())
                                }
                            },
                            |result: Result<Vec<Preset>, String>| {
                                match result {
                                    Ok(presets) => Message::PresetsLoaded(Ok(presets)),
                                    Err(e) => Message::Error(e),
                                }
                            },
                        )
                    }
                    Err(e) => {
                        self.preset_tab.form_error = Some(e);
                        Task::none()
                    }
                }
            }

            Message::PresetDeleteRequested(name) => {
                self.preset_tab.delete_target = Some(name);
                Task::none()
            }

            Message::PresetDeleteConfirmed(name) => {
                self.preset_tab.delete_target = None;
                let data_fs = self.data_fs.clone();
                Task::perform(
                    async move {
                        data_fs.delete_preset(&name).map_err(|e| e.to_string())?;
                        data_fs.list_presets().map_err(|e| e.to_string())
                    },
                    |result: Result<Vec<Preset>, String>| {
                        match result {
                            Ok(presets) => Message::PresetsLoaded(Ok(presets)),
                            Err(e) => Message::Error(e),
                        }
                    },
                )
            }

            Message::PresetDeleteDismissed => {
                self.preset_tab.delete_target = None;
                Task::none()
            }

            Message::SettingsFormDataDirChanged(val) => {
                self.settings_tab.form_data_dir = val;
                Task::none()
            }

            Message::SettingsFormDefaultPresetChanged(val) => {
                self.settings_tab.form_default_preset = val;
                Task::none()
            }

            Message::SettingsFormSaveRequested => {
                let data_dir_val = self.settings_tab.form_data_dir.trim().to_string();
                let default_preset_val = self.settings_tab.form_default_preset.trim().to_string();

                let new_config = crate::data::config::Config {
                    data_dir: if data_dir_val.is_empty() { None } else { Some(data_dir_val.clone()) },
                    default_preset: if default_preset_val.is_empty() { None } else { Some(default_preset_val.clone()) },
                    theme: self.settings_tab.config.theme.clone(),
                };

                let path = self.settings_tab.config_path.clone();
                let data_dir_changed = data_dir_val != self.data_dir.to_string_lossy();

                match new_config.save(&path) {
                    Ok(()) => {
                        self.settings_tab.config = new_config;
                        if !default_preset_val.is_empty() {
                            self.default_preset = default_preset_val;
                        }
                        let msg = if data_dir_changed {
                            "配置已保存，数据目录修改需重启生效".to_string()
                        } else {
                            "配置已保存".to_string()
                        };
                        self.settings_tab.message = Some(msg);
                        self.settings_tab.message_is_error = false;
                    }
                    Err(e) => {
                        self.settings_tab.message = Some(format!("保存失败: {}", e));
                        self.settings_tab.message_is_error = true;
                    }
                }
                Task::none()
            }

            Message::SettingsFormDismissMessage => {
                self.settings_tab.dismiss_message();
                Task::none()
            }

            Message::ThemeChanged(theme) => {
                self.theme = theme;
                self.settings_tab.form_theme = theme;
                self.settings_tab.config.theme = Some(theme.as_str().to_string());
                let path = self.settings_tab.config_path.clone();
                let config = self.settings_tab.config.clone();
                let _ = config.save(&path);
                Task::none()
            }

            Message::CardSelected(path) => {
                self.category_tab.selected_path = Some(path.clone());
                Task::none()
            }
            
            Message::SearchChanged(query) => {
                self.review_tab.search_query = query;
                Task::none()
            }
            
            Message::UrgencyFilterChanged(filter) => {
                self.review_tab.urgency_filter = filter;
                Task::none()
            }
            
            Message::SearchQueryChanged(query) => {
                self.category_tab.search_query = query.clone();
                self.category_tab.search_results = self.search_nodes(&query);
                self.category_tab.search_selected_index = if self.category_tab.search_results.is_empty() {
                    None
                } else {
                    Some(0)
                };
                Task::none()
            }
            
            Message::SearchItemSelected(index) => {
                if let Some(result) = self.category_tab.search_results.get(index) {
                    let path = result.path.clone();
                    self.category_tab.tree_view.expand_to_path(&self.category_tab.tree_nodes, &path);
                    self.category_tab.selected_path = Some(path);
                    self.category_tab.search_query.clear();
                    self.category_tab.search_results.clear();
                    self.category_tab.search_selected_index = None;
                    self.category_tab.search_focused = false;
                }
                Task::none()
            }
            
            Message::SearchFocusChanged(focused) => {
                self.category_tab.search_focused = focused;
                Task::none()
            }
            
            Message::NewNodeOpen(default_type) => {
                if let Some(ref path) = self.category_tab.selected_path {
                    let form = category_tab::NewNodeForm::new(path.clone(), default_type, &self.default_preset);
                    self.category_tab.new_node_form = Some(form);
                    self.modal = Some(Modal::NewNode);
                }
                Task::none()
            }
            
            Message::CreateTreeNameChanged(name) => {
                self.category_tab.new_tree_name = name;
                Task::none()
            }

            Message::CreateTreeImportPathChanged(path) => {
                self.category_tab.new_tree_import_path = path;
                Task::none()
            }

            Message::CreateTreeImportRulesPathChanged(val) => {
                self.category_tab.new_tree_import_rules_path = val;
                Task::none()
            }

            Message::NewNodeNameChanged(name) => {
                if let Some(ref mut form) = self.category_tab.new_node_form {
                    form.name = name;
                }
                Task::none()
            }
            
            Message::NewNodeTypeChanged(node_type) => {
                if let Some(ref mut form) = self.category_tab.new_node_form {
                    form.node_type = node_type;
                }
                Task::none()
            }
            
            Message::NewNodePresetChanged(preset) => {
                if let Some(ref mut form) = self.category_tab.new_node_form {
                    form.preset = preset;
                }
                Task::none()
            }
            
            Message::NewNodeConfirm => {
                if let Some(ref form) = self.category_tab.new_node_form {
                    if form.name.is_empty() {
                        self.error_message = Some("名称不能为空".to_string());
                        return Task::none();
                    }
                    
                    let new_path = format!("{}/{}", form.parent_path, form.name);
                    
                    let result = match form.node_type {
                        NodeType::Folder => {
                            self.data_fs.create_folder(&new_path)
                        }
                        NodeType::Card => {
                            let card = Card::new_with_preset(form.preset.clone());
                            self.data_fs.save_card(&new_path, &card)
                        }
                    };
                    
                    match result {
                        Ok(()) => {
                            self.modal = None;
                            self.category_tab.new_node_form = None;
                            
                            let data_fs = Arc::new(self.data_fs.clone());
                            return Task::future(async move {
                                let trees = data_fs.list_trees().unwrap_or_default();
                                let mut all_cards = Vec::new();
                                for tree in &trees {
                                    let cards = data_fs.list_cards(tree).unwrap_or_default();
                                    all_cards.extend(cards);
                                }
                                let presets = data_fs.list_presets().unwrap_or_default();
                                let todos = data_fs.list_todos().unwrap_or_default();
                                Message::DataLoaded(Ok(DataSnapshot {
                                    trees,
                                    cards: all_cards,
                                    presets,
                                    todos,
                                }))
                            });
                        }
                        Err(e) => {
                            self.error_message = Some(e.to_string());
                        }
                    }
                }
                Task::none()
            }
            
            Message::EditCardOpen(path) => {
                match self.data_fs.get_card(&path) {
                    Ok(card) => {
                        let form = category_tab::EditCardForm::new(path.clone(), &card);
                        self.category_tab.edit_card_form = Some(form);
                        self.modal = Some(Modal::EditCard { path });
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }
            
            Message::EditCardNameChanged(name) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    form.new_name = name;
                }
                Task::none()
            }
            
            Message::EditCardPresetChanged(preset) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    form.preset = preset;
                }
                Task::none()
            }
            
            Message::EditCardNextReviewChanged(date_str) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    form.next_review = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));
                }
                Task::none()
            }
            
            Message::EditCardClearPrediction => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    form.clear_prediction = true;
                    form.next_review = None;
                }
                Task::none()
            }
            
            Message::EditCardPredict => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    use crate::fsrs::FsrsPredictor;
                    
                    let predictor = FsrsPredictor::new().ok();
                    if let Some(predictor) = predictor {
                        let current_state = form.prediction.as_ref()
                            .and_then(|p| FsrsPredictor::bytes_to_memory_state(&p.fsrs_state_bytes));
                        
                        match predictor.predict_from_records(&form.review_records, current_state, 0.9) {
                            Ok((next_review, new_state)) => {
                                form.next_review = Some(next_review);
                                form.fsrs_state_bytes = FsrsPredictor::memory_state_to_bytes(&new_state);
                                form.clear_prediction = false;
                            }
                            Err(_) => {
                                form.next_review = Some(chrono::Utc::now() + chrono::Duration::days(1));
                            }
                        }
                    } else {
                        form.next_review = Some(chrono::Utc::now() + chrono::Duration::days(1));
                    }
                }
                Task::none()
            }
            
            Message::EditCardToggleGroup(group) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    if form.expanded_groups.contains(&group) {
                        form.expanded_groups.remove(&group);
                    } else {
                        form.expanded_groups.insert(group);
                    }
                }
                Task::none()
            }
            
            Message::EditCardAddReview => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    let duration: i64 = form.new_review_duration.parse().unwrap_or(0);
                    if duration > 0 {
                        use crate::data::models::ReviewRecord;
                        let record = ReviewRecord {
                            timestamp: chrono::Utc::now(),
                            duration_ms: duration * 60000,
                            memory_quality: form.new_review_quality.clone(),
                        };
                        form.review_records.push(record);
                        form.new_review_duration.clear();
                    }
                }
                Task::none()
            }
            
            Message::EditCardRemoveReview(index) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form
                    && index < form.review_records.len() {
                    form.review_records.remove(index);
                }
                Task::none()
            }
            
            Message::EditCardNewReviewDurationChanged(duration) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    form.new_review_duration = duration;
                }
                Task::none()
            }
            
            Message::EditCardNewReviewQualityChanged(quality) => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    form.new_review_quality = quality;
                }
                Task::none()
            }
            
            Message::EditCardConfirm => {
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    let mut card = Card::new();
                    card.review_records = form.review_records.clone();
                    
                    if !form.clear_prediction {
                        use crate::data::models::Prediction;
                        card.prediction = Some(Prediction {
                            algorithm: "fsrs".to_string(),
                            next_review: form.next_review.unwrap_or_else(chrono::Utc::now),
                            fsrs_state_bytes: form.fsrs_state_bytes.clone(),
                            preset_used: form.preset.clone(),
                        });
                    }
                    
                    let final_path = if form.original_name != form.new_name {
                        match self.data_fs.rename_card(&form.path, &form.new_name) {
                            Ok(new_path) => new_path,
                            Err(e) => {
                                self.error_message = Some(e.to_string());
                                return Task::none();
                            }
                        }
                    } else {
                        form.path.clone()
                    };
                    
                    match self.data_fs.save_card(&final_path, &card) {
                        Ok(()) => {
                            self.modal = None;
                            self.category_tab.edit_card_form = None;
                            
                            let data_fs = Arc::new(self.data_fs.clone());
                            return Task::future(async move {
                                let trees = data_fs.list_trees().unwrap_or_default();
                                let mut all_cards = Vec::new();
                                for tree in &trees {
                                    let cards = data_fs.list_cards(tree).unwrap_or_default();
                                    all_cards.extend(cards);
                                }
                                let presets = data_fs.list_presets().unwrap_or_default();
                                let todos = data_fs.list_todos().unwrap_or_default();
                                Message::DataLoaded(Ok(DataSnapshot {
                                    trees,
                                    cards: all_cards,
                                    presets,
                                    todos,
                                }))
                            });
                        }
                        Err(e) => {
                            self.error_message = Some(e.to_string());
                        }
                    }
                }
                Task::none()
            }
            
            Message::DeleteNodeOpen(path, is_folder) => {
                self.category_tab.delete_target = Some((path.clone(), is_folder));
                self.modal = Some(Modal::ConfirmDelete { item: path });
                Task::none()
            }
            
            Message::DeleteNodeConfirm => {
                if let Some((path, is_folder)) = self.category_tab.delete_target.take() {
                    let result = if !path.contains('/') {
                        self.data_fs.delete_tree(&path)
                    } else if is_folder {
                        self.data_fs.delete_folder(&path)
                    } else {
                        self.data_fs.delete_card(&path)
                    };
                    
                    match result {
                        Ok(()) => {
                            self.modal = None;
                            self.category_tab.selected_path = None;
                            
                            let data_fs = Arc::new(self.data_fs.clone());
                            return Task::future(async move {
                                let trees = data_fs.list_trees().unwrap_or_default();
                                let mut all_cards = Vec::new();
                                for tree in &trees {
                                    let cards = data_fs.list_cards(tree).unwrap_or_default();
                                    all_cards.extend(cards);
                                }
                                let presets = data_fs.list_presets().unwrap_or_default();
                                let todos = data_fs.list_todos().unwrap_or_default();
                                Message::DataLoaded(Ok(DataSnapshot {
                                    trees,
                                    cards: all_cards,
                                    presets,
                                    todos,
                                }))
                            });
                        }
                        Err(e) => {
                            self.error_message = Some(e.to_string());
                        }
                    }
                }
                Task::none()
            }
            
            Message::ReviewCardSelected(path) => {
                self.review_tab.selected_card = Some(path);
                Task::none()
            }
            
            Message::StartReviewTimer(path) => {
                self.timer_tab.current_card = Some(path);
                self.active_tab = TabId::Timer;
                let _ = self.timer_manager.start();
                self.timer_tab.state = self.timer_manager.get_state().clone();
                Task::none()
            }
            
            Message::RefreshPredictions => {
                use crate::fsrs::FsrsPredictor;
                
                let data_fs = Arc::new(self.data_fs.clone());
                let default_preset = self.default_preset.clone();
                Task::future(async move {
                    let predictor = FsrsPredictor::new().expect("Failed to create FSRS predictor");
                    let trees = data_fs.list_trees().unwrap_or_default();
                    let mut all_cards = Vec::new();
                    for tree in &trees {
                        let cards = data_fs.list_cards(tree).unwrap_or_default();
                        all_cards.extend(cards);
                    }
                    
                    for (path, mut card) in all_cards.clone() {
                        if !card.review_records.is_empty() {
                            let current_state = card.prediction.as_ref()
                                .and_then(|p| crate::fsrs::FsrsPredictor::bytes_to_memory_state(&p.fsrs_state_bytes));
                            
                            match predictor.predict_from_records(&card.review_records, current_state, 0.9) {
                                Ok((next_review, new_state)) => {
                                    let existing = card.prediction.as_ref().map(|p| p.preset_used.clone());
                                    use crate::data::models::Prediction;
                                    card.prediction = Some(Prediction {
                                        algorithm: "fsrs".to_string(),
                                        next_review,
                                        fsrs_state_bytes: crate::fsrs::FsrsPredictor::memory_state_to_bytes(&new_state),
                                        preset_used: existing.unwrap_or_else(|| default_preset.clone()),
                                    });
                                    let _ = data_fs.save_card(&path, &card);
                                }
                                Err(e) => {
                                    eprintln!("Failed to predict for {}: {}", path, e);
                                }
                            }
                        }
                    }
                    
                    let trees = data_fs.list_trees().unwrap_or_default();
                    let mut all_cards = Vec::new();
                    for tree in &trees {
                        let cards = data_fs.list_cards(tree).unwrap_or_default();
                        all_cards.extend(cards);
                    }
                    let presets = data_fs.list_presets().unwrap_or_default();
                    let todos = data_fs.list_todos().unwrap_or_default();
                    
                    Message::DataLoaded(Ok(DataSnapshot {
                        trees,
                        cards: all_cards,
                        presets,
                        todos,
                    }))
                })
            }
            
            Message::TimerStarted => {
                let _ = self.timer_manager.start();
                self.timer_tab.state = self.timer_manager.get_state().clone();
                Task::none()
            }
            
            Message::TimerPaused => {
                let _ = self.timer_manager.pause();
                self.timer_tab.state = self.timer_manager.get_state().clone();
                Task::none()
            }
            
            Message::TimerStopped(result) => {
                match result {
                    Ok(_path) => {
                        // 先保存计时时间，再停止计时器
                        let final_elapsed = self.timer_tab.elapsed_ms;
                        let _ = self.timer_manager.stop();
                        self.timer_tab.state = self.timer_manager.get_state().clone();
                        
                        // 如果有预设卡片路径，进入链接模式
                        if let Some(card_path) = self.timer_tab.current_card.take() {
                            self.timer_tab.link_mode = true;
                            self.timer_tab.card_path_input = card_path;
                            self.timer_tab.elapsed_ms = final_elapsed;
                        } else {
                            // 即使没有关联卡片，也进入链接模式
                            self.timer_tab.link_mode = true;
                            self.timer_tab.elapsed_ms = final_elapsed;
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            
            Message::TimerTick(_) => {
                self.timer_tab.elapsed_ms = self.timer_manager.get_state().elapsed_ms();
                Task::none()
            }
            
            Message::TimerLinkModeOpen => {
                self.timer_tab.link_mode = true;
                self.error_message = None;
                Task::none()
            }
            
            Message::TimerLinkModeClose => {
                self.timer_tab.link_mode = false;
                Task::none()
            }
            
            Message::TimerCardPathChanged(path) => {
                self.timer_tab.card_path_input = path.replace('\\', "/");
                Task::none()
            }
            
            Message::TimerCardSelected(path) => {
                self.timer_tab.selected_card = Some(path.clone());
                self.timer_tab.card_path_input = path;
                Task::none()
            }
            
            Message::TimerMemoryQualityChanged(quality) => {
                self.timer_tab.memory_quality = quality;
                Task::none()
            }
            
            Message::TimerLinkConfirm => {
                // 使用卡片路径输入或下拉选择的卡片
                let card_path = if !self.timer_tab.card_path_input.is_empty() {
                    self.timer_tab.card_path_input.clone()
                } else if let Some(ref selected) = self.timer_tab.selected_card {
                    selected.clone()
                } else {
                    String::new()
                };
                
                if !card_path.is_empty() {
                    let duration_ms = self.timer_tab.elapsed_ms;
                    let quality = self.timer_tab.memory_quality.clone();
                    
                    // 创建 ReviewRecord
                    let record = crate::data::models::ReviewRecord {
                        timestamp: chrono::Utc::now(),
                        duration_ms,
                        memory_quality: quality.clone(),
                    };
                    
                    // 保存到卡片
                    if let Ok(mut card) = self.data_fs.get_card(&card_path) {
                        card.review_records.push(record);
                        
                        // 触发 FSRS 预测
                        if let Ok(predictor) = crate::fsrs::FsrsPredictor::new()
                            && let Ok((next_review, new_state)) = predictor.predict_from_records(
                                &card.review_records,
                                None,
                                0.9
                            ) {
                                card.prediction = Some(crate::data::models::Prediction {
                                    algorithm: "fsrs".to_string(),
                                    next_review,
                                    fsrs_state_bytes: crate::fsrs::FsrsPredictor::memory_state_to_bytes(&new_state),
                                    preset_used: self.default_preset.clone(),
                                });
                            }
                        
                        let _ = self.data_fs.save_card(&card_path, &card);
                    }
                    
                    self.timer_tab.link_mode = false;
                    self.timer_tab.elapsed_ms = 0;
                }
                Task::none()
            }
            
            Message::TimerCreateNewCard => {
                // 新卡片表单现在常态显示，此消息不再需要
                Task::none()
            }
            
            Message::TimerNewCardNameChanged(name) => {
                self.timer_tab.new_card_name = name;
                self.error_message = None;
                Task::none()
            }
            
            Message::TimerNewCardPresetChanged(preset) => {
                self.timer_tab.new_card_preset = preset;
                self.error_message = None;
                Task::none()
            }
            
            Message::TimerNewCardTypeChanged(node_type) => {
                self.timer_tab.new_card_type = node_type;
                self.error_message = None;
                Task::none()
            }
            
            Message::TimerNewCardConfirm => {
                if !self.timer_tab.new_card_name.is_empty() {
                    let input_path = self.timer_tab.card_path_input.replace('\\', "/");
                    let node_name = self.timer_tab.new_card_name.clone();
                    let node_type = self.timer_tab.new_card_type;
                    let preset = self.timer_tab.new_card_preset.clone();
                    
                    let is_card_path = self.category_tab.tree_nodes.iter()
                        .flat_map(Self::flatten_tree)
                        .any(|n| n.path == input_path && n.is_card);
                    
                    let target_path = if input_path.is_empty() {
                        node_name.clone()
                    } else if is_card_path {
                        let parent = input_path.rsplit_once('/')
                            .map(|(p, _)| p.to_string())
                            .unwrap_or_default();
                        if parent.is_empty() {
                            node_name.clone()
                        } else {
                            format!("{}/{}", parent, node_name)
                        }
                    } else {
                        format!("{}/{}", input_path, node_name)
                    };
                    
                    let result = match node_type {
                        NodeType::Folder => self.data_fs.create_folder(&target_path),
                        NodeType::Card => {
                            let card = Card::new_with_preset(preset);
                            self.data_fs.save_card(&target_path, &card)
                        }
                    };
                    
                    if result.is_ok() {
                        self.timer_tab.new_card_name.clear();
                        
                        if let Ok(trees) = self.data_fs.list_trees() {
                            let mut all_cards = Vec::new();
                            for tree in &trees {
                                if let Ok(cards) = self.data_fs.list_cards(tree) {
                                    all_cards.extend(cards);
                                }
                            }
                            let new_nodes = self.build_tree_nodes(&trees, &all_cards);
                            self.category_tab.tree_nodes = new_nodes.clone();
                            self.category_tab.tree_view.expand_all(&new_nodes);
                        }
                    } else if let Err(e) = result {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }
            
            Message::TimerHistoryShow => {
                self.timer_tab.show_history = true;
                Task::none()
            }
            
            Message::TimerHistoryHide => {
                self.timer_tab.show_history = false;
                Task::none()
            }
            
            Message::QuickTimerStart(path) => {
                self.timer_tab.current_card = Some(path);
                self.active_tab = TabId::Timer;
                
                // 自动开始计时
                let _ = self.timer_manager.start();
                self.timer_tab.state = self.timer_manager.get_state().clone();
                
                Task::none()
            }
            
            Message::Error(msg) => {
                self.error_message = Some(msg);
                Task::none()
            }
            
            Message::ClearError => {
                self.error_message = None;
                Task::none()
            }
            
            Message::ModalOpen(modal) => {
                match &modal {
                    Modal::NewTodo => {
                        self.todo_tab.set_default_due_date();
                    }
                    Modal::EditTodo { id } => {
                        if let Some(todo) = self.todo_tab.todos.iter().find(|t| t.id == *id) {
                            let todo_clone = todo.clone();
                            self.todo_tab.load_todo_for_edit(&todo_clone);
                        }
                    }
                    Modal::CreateTree => {
                        self.category_tab.new_tree_name.clear();
                        self.category_tab.new_tree_import_path.clear();
                        self.category_tab.new_tree_import_rules_path.clear();
                    }
                    _ => {}
                }
                self.modal = Some(modal);
                Task::none()
            }
            
            Message::ModalClose => {
                self.modal = None;
                Task::none()
            }
            
            Message::NewTodoContentChanged(text) => {
                self.todo_tab.form_content = text;
                Task::none()
            }
            
            Message::NewTodoPriorityChanged(p) => {
                self.todo_tab.form_priority = p;
                Task::none()
            }
            
            Message::NewTodoDueYearChanged(text) => {
                self.todo_tab.form_due_year = text;
                Task::none()
            }
            
            Message::NewTodoDueMonthChanged(text) => {
                self.todo_tab.form_due_month = text;
                Task::none()
            }
            
            Message::NewTodoDueDayChanged(text) => {
                self.todo_tab.form_due_day = text;
                Task::none()
            }
            
            Message::NewTodoDueHourChanged(text) => {
                self.todo_tab.form_due_hour = text;
                Task::none()
            }
            
            Message::NewTodoDueMinuteChanged(text) => {
                self.todo_tab.form_due_minute = text;
                Task::none()
            }
            
            Message::NewTodoTagsChanged(text) => {
                self.todo_tab.form_tags = text;
                Task::none()
            }
            
            Message::ModalConfirm => {
                if let Some(modal) = &self.modal {
                    match modal {
                        Modal::NewTodo | Modal::EditTodo { .. } => {
                            let content = self.todo_tab.form_content.trim();
                            if !content.is_empty() {
                                let year = self.todo_tab.form_due_year.trim();
                                let month = self.todo_tab.form_due_month.trim();
                                let day = self.todo_tab.form_due_day.trim();
                                let hour = self.todo_tab.form_due_hour.trim();
                                let minute = self.todo_tab.form_due_minute.trim();
                                
                                let due_date = if !year.is_empty() && !month.is_empty() && !day.is_empty() 
                                    && !hour.is_empty() && !minute.is_empty() {
                                    let date_str = format!(
                                        "{}-{}-{} {}:{}", 
                                        year, month, day, hour, minute
                                    );
                                    chrono::NaiveDateTime::parse_from_str(
                                        &date_str,
                                        "%Y-%m-%d %H:%M"
                                    )
                                    .ok()
                                    .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, Utc))
                                } else {
                                    None
                                };
                                
                                let tags: Vec<String> = if !self.todo_tab.form_tags.trim().is_empty() {
                                    self.todo_tab.form_tags
                                        .split_whitespace()
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect()
                                } else {
                                    Vec::new()
                                };
                                
                                if let Modal::EditTodo { id } = modal {
                                    if let Some(todo) = self.todo_tab.todos.iter_mut().find(|t| t.id == *id) {
                                        todo.content = content.to_string();
                                        todo.priority = self.todo_tab.form_priority;
                                        todo.due_date = due_date;
                                        todo.tags = tags;
                                        
                                        match self.data_fs.save_todo(todo) {
                                            Ok(_) => {
                                                self.modal = None;
                                                self.todo_tab.editing_todo_id = None;
                                                self.todo_tab.form_content.clear();
                                                self.todo_tab.form_priority = None;
                                                self.todo_tab.form_due_year.clear();
                                                self.todo_tab.form_due_month.clear();
                                                self.todo_tab.form_due_day.clear();
                                                self.todo_tab.form_due_hour.clear();
                                                self.todo_tab.form_due_minute.clear();
                                                self.todo_tab.form_tags.clear();
                                            }
                                            Err(e) => {
                                                self.error_message = Some(e.to_string());
                                            }
                                        }
                                    }
                                } else {
                                    let mut todo = Todo::new(content.to_string());
                                    todo.priority = self.todo_tab.form_priority;
                                    todo.due_date = due_date;
                                    todo.tags = tags;
                                    
                                    match self.data_fs.save_todo(&todo) {
                                        Ok(_) => {
                                            self.todo_tab.todos.push(todo);
                                            self.modal = None;
                                            self.todo_tab.form_content.clear();
                                            self.todo_tab.form_priority = None;
                                            self.todo_tab.form_due_year.clear();
                                            self.todo_tab.form_due_month.clear();
                                            self.todo_tab.form_due_day.clear();
                                            self.todo_tab.form_due_hour.clear();
                                            self.todo_tab.form_due_minute.clear();
                                            self.todo_tab.form_tags.clear();
                                        }
                                        Err(e) => {
                                            self.error_message = Some(e.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        Modal::CreateTree => {
                            let name = self.category_tab.new_tree_name.trim().to_string();
                            if !name.is_empty() {
                                let result = self.data_fs.create_tree(&name);
                                if let Err(e) = result {
                                    self.error_message = Some(e.to_string());
                                } else {
                                    let import_path = self.category_tab.new_tree_import_path.trim().to_string();
                                    let mut import_result = None;
                                    if !import_path.is_empty() {
                                        let import_dir = std::path::PathBuf::from(&import_path);
                                        if import_dir.exists() {
                                            let data_dir = self.data_dir.clone();
                                            let tree_name = name.clone();
                                            let rules_path = self.category_tab.new_tree_import_rules_path.clone();
                                            let ignore_file = if rules_path.is_empty() {
                                                data_dir.join(".timeignore")
                                            } else {
                                                std::path::PathBuf::from(&rules_path)
                                            };
                                            let rules = crate::obsidian::TimeignoreRules::from_file(&ignore_file)
                                                .unwrap_or_else(|_| crate::obsidian::TimeignoreRules::default_rules());
                                            match crate::obsidian::import_from_obsidian(
                                                &import_dir,
                                                &tree_name,
                                                &data_dir,
                                                &rules,
                                            ) {
                                                Ok(result) => {
                                                    import_result = Some(result);
                                                }
                                                Err(e) => {
                                                    self.error_message = Some(format!("导入失败: {}", e));
                                                }
                                            }
                                        } else {
                                            self.error_message = Some("导入路径不存在".into());
                                        }
                                    }
                                    self.category_tab.new_tree_name.clear();
                                    self.category_tab.new_tree_import_path.clear();
                                    self.category_tab.new_tree_import_rules_path.clear();
                                    // always reload trees
                                    let trees = self.data_fs.list_trees().unwrap_or_default();
                                    let tree_nodes = self.build_tree_nodes(&trees, &self.review_tab.cards);
                                    self.category_tab.tree_nodes = tree_nodes.clone();
                                    self.category_tab.tree_view.expand_all(&tree_nodes);
                                    // show import result in modal, otherwise close
                                    if let Some(result) = import_result {
                                        self.modal = Some(Modal::Info {
                                            message: format!("导入完成: 创建 {} 个目录, 跳过 {} 个路径",
                                                result.created_dirs.len(),
                                                result.skipped_paths.len()),
                                        });
                                    } else {
                                        self.modal = None;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Task::none()
            }
            
            Message::ToggleTodo(id) => {
                if let Some(todo) = self.todo_tab.todos.iter_mut().find(|t| t.id == id) {
                    todo.completed = !todo.completed;
                    if todo.completed {
                        todo.completed_at = Some(Utc::now());
                    } else {
                        todo.completed_at = None;
                    }
                    if let Err(e) = self.data_fs.save_todo(todo) {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }
            
            Message::DeleteTodo(id) => {
                match self.data_fs.delete_todo(&id) {
                    Ok(_) => {
                        self.todo_tab.todos.retain(|t| t.id != id);
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }
            
            Message::TodoSelected(id) => {
                self.todo_tab.selected_todo = Some(id);
                Task::none()
            }
            
            Message::ScheduleCreateOpen => {
                self.schedule_tab.form.reset();
                self.schedule_tab.editing_id = None;
                self.schedule_tab.show_form = true;
                self.schedule_tab.form_error = None;
                self.modal = Some(Modal::NewSchedule);
                Task::none()
            }

            Message::ScheduleCreateConfirm => {
                match self.schedule_tab.form.validate() {
                    Ok(schedule) => {
                        match self.data_fs.save_schedule(&schedule) {
                            Ok(()) => {
                                self.schedule_tab.show_form = false;
                                self.modal = None;
                                self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();
                            }
                            Err(e) => {
                                self.schedule_tab.form_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.schedule_tab.form_error = Some(e);
                    }
                }
                Task::none()
            }

            Message::ScheduleEditOpen(id) => {
                if let Some(schedule) = self.schedule_tab.schedules.iter().find(|s| s.id == id) {
                    self.schedule_tab.form.load_from_schedule(schedule);
                    self.schedule_tab.editing_id = Some(id);
                    self.schedule_tab.show_form = true;
                    self.schedule_tab.form_error = None;
                    self.modal = Some(Modal::NewSchedule);
                }
                Task::none()
            }

            Message::ScheduleEditConfirm => {
                match self.schedule_tab.form.validate() {
                    Ok(mut schedule) => {
                        if let Some(editing_id) = self.schedule_tab.editing_id {
                            schedule.id = editing_id;
                        }
                        match self.data_fs.save_schedule(&schedule) {
                            Ok(()) => {
                                self.schedule_tab.show_form = false;
                                self.modal = None;
                                self.schedule_tab.editing_id = None;
                                self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();
                            }
                            Err(e) => {
                                self.schedule_tab.form_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.schedule_tab.form_error = Some(e);
                    }
                }
                Task::none()
            }

            Message::ScheduleDelete(id) => {
                match self.data_fs.delete_schedule(&id) {
                    Ok(()) => {
                        self.schedule_tab.schedules.retain(|s| s.id != id);
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }

            Message::ScheduleFormSummaryChanged(val) => {
                self.schedule_tab.form.summary = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormStartDateChanged(val) => {
                self.schedule_tab.form.start_date = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormStartTimeChanged(val) => {
                self.schedule_tab.form.start_time = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormEndDateChanged(val) => {
                self.schedule_tab.form.end_date = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormEndTimeChanged(val) => {
                self.schedule_tab.form.end_time = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormDescriptionChanged(val) => {
                self.schedule_tab.form.description = val;
                Task::none()
            }
            Message::ScheduleFormLocationChanged(val) => {
                self.schedule_tab.form.location = val;
                Task::none()
            }
            Message::ScheduleFormReminderChanged(val) => {
                self.schedule_tab.form.reminder = val;
                Task::none()
            }
            Message::ScheduleFormRruleChanged(val) => {
                self.schedule_tab.form.rrule = val;
                Task::none()
            }
            
            _ => Task::none(),
        }
    }
    
    fn build_tree_nodes(&self, trees: &[String], cards: &[(String, Card)]) -> Vec<TreeNode> {
        let data_dir = self.data_fs.data_dir().clone();
        let mut result: Vec<TreeNode> = Vec::new();

        for tree_name in trees {
            let mut tree_node = TreeNode {
                name: tree_name.clone(),
                path: tree_name.clone(),
                is_card: false,
                children: Self::scan_folder_nodes(&data_dir, tree_name, ""),
            };

            for (card_path, _card) in cards.iter().filter(|(p, _)| p.starts_with(&format!("{}/", tree_name))) {
                let relative = card_path.strip_prefix(&format!("{}/", tree_name)).unwrap_or(card_path);
                let parts: Vec<&str> = relative.split('/').collect();
                let card_name = parts.last().unwrap().to_string();

                if parts.len() == 1 {
                    tree_node.children.push(TreeNode {
                        name: card_name,
                        path: card_path.clone(),
                        is_card: true,
                        children: Vec::new(),
                    });
                } else {
                    Self::add_card_to_node(&mut tree_node.children, card_path, &parts[..parts.len() - 1], &card_name);
                }
            }

            result.push(tree_node);
        }

        result
    }

    fn scan_folder_nodes(data_dir: &std::path::Path, tree_name: &str, rel_path: &str) -> Vec<TreeNode> {
        let dir_path = data_dir.join("categories").join(tree_name).join(rel_path);
        let mut nodes = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let child_rel = if rel_path.is_empty() {
                        folder_name.clone()
                    } else {
                        format!("{}/{}", rel_path, folder_name)
                    };
                    let full_path = format!("{}/{}", tree_name, child_rel);
                    nodes.push(TreeNode {
                        name: folder_name,
                        path: full_path,
                        is_card: false,
                        children: Self::scan_folder_nodes(data_dir, tree_name, &child_rel),
                    });
                }
            }
        }

        nodes
    }

    fn add_card_to_node(nodes: &mut Vec<TreeNode>, card_path: &str, folder_parts: &[&str], card_name: &str) {
        if let Some(first) = folder_parts.first() {
            let tree_name = card_path.split('/').next().unwrap_or("");
            let folder_path = {
                let mut p = tree_name.to_string();
                for part in folder_parts {
                    p.push('/');
                    p.push_str(part);
                }
                p
            };

            if let Some(node) = nodes.iter_mut().find(|n| n.name == *first) {
                if folder_parts.len() == 1 {
                    node.children.push(TreeNode {
                        name: card_name.to_string(),
                        path: card_path.to_string(),
                        is_card: true,
                        children: Vec::new(),
                    });
                } else {
                    Self::add_card_to_node(&mut node.children, card_path, &folder_parts[1..], card_name);
                }
            } else {
                let mut new_node = TreeNode {
                    name: first.to_string(),
                    path: folder_path,
                    is_card: false,
                    children: Vec::new(),
                };
                if folder_parts.len() == 1 {
                    new_node.children.push(TreeNode {
                        name: card_name.to_string(),
                        path: card_path.to_string(),
                        is_card: true,
                        children: Vec::new(),
                    });
                } else {
                    Self::add_card_to_node(&mut new_node.children, card_path, &folder_parts[1..], card_name);
                }
                nodes.push(new_node);
            }
        }
    }
    
    fn view(&self) -> Element<'_, Message> {
        use iced::widget::scrollable;
        
        let theme = self.theme.to_iced_theme();
        let palette = theme.extended_palette();
        let hint_text = palette.background.weak.text;
        
        let tabs = row![
            tab_button("分类树", TabId::Category, self.active_tab),
            tab_button("复习看板", TabId::Review, self.active_tab),
            tab_button("计时器", TabId::Timer, self.active_tab),
            tab_button("日程", TabId::Schedule, self.active_tab),
            tab_button("待办", TabId::Todo, self.active_tab),
            tab_button("预设", TabId::Preset, self.active_tab),
            tab_button("设置", TabId::Settings, self.active_tab),
        ]
        .spacing(4)
        .padding(8);
        
        let content: Element<Message> = match self.active_tab {
            TabId::Category => {
                if self.category_tab.tree_nodes.is_empty() {
                    container(
                        column![
                            text("暂无数据").size(16),
                            Space::new().height(12),
                            button(text("新建分类树"))
                                .on_press(Message::ModalOpen(Modal::CreateTree)),
                        ]
                        .spacing(4)
                        .align_x(iced::Alignment::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                } else {
                    let search_input = text_input(
                        "搜索卡片或文件夹...",
                        &self.category_tab.search_query,
                    )
                    .on_input(Message::SearchQueryChanged)
                    .width(Length::Fill)
                    .style(move |theme: &iced::Theme, _| {
                        let palette = theme.extended_palette();
                        iced::widget::text_input::Style {
                            background: palette.background.strong.color.into(),
                            border: iced::Border {
                                color: palette.background.neutral.color,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            icon: hint_text,
                            placeholder: hint_text,
                            value: palette.background.base.text,
                            selection: iced::Color::from_rgb(0.3, 0.6, 0.9),
                        }
                    });
                    
                    let search_results: Element<Message> = if !self.category_tab.search_results.is_empty() && !self.category_tab.search_query.is_empty() {
                        column(
                            self.category_tab.search_results.iter().enumerate()
                                .map(|(i, result)| {
                                    let is_selected = self.category_tab.search_selected_index == Some(i);
                                    let icon = if result.is_card { "📄" } else { "📁" };
                                    
                                    button(
                                        row![
                                            text(format!("{} {}", icon, result.name))
                                                .color(palette.background.base.text)
                                                .size(14),
                                        ]
                                        .spacing(4)
                                    )
                                    .on_press(Message::SearchItemSelected(i))
                                    .style(move |theme: &iced::Theme, _| {
                                        let palette = theme.extended_palette();
                                        if is_selected {
                                            iced::widget::button::Style {
                                                background: Some(iced::Color::from_rgb(0.3, 0.6, 0.9).into()),
                                                text_color: iced::Color::WHITE,
                                                ..Default::default()
                                            }
                                        } else {
                                            iced::widget::button::Style {
                                                background: Some(palette.background.strong.color.into()),
                                                text_color: palette.background.base.text,
                                                ..Default::default()
                                            }
                                        }
                                    })
                                    .width(Length::Fill)
                                    .into()
                                })
                        )
                        .spacing(1)
                        .into()
                    } else {
                        column![].into()
                    };
                    
                    let tree_element = self.category_tab.tree_view.view_static(
                        &self.category_tab.tree_nodes,
                        self.category_tab.selected_path.clone(),
                    ).map(Message::CardSelected);
                    
                    let left_content = column![
                        search_input,
                        search_results,
                        Space::new().height(8),
                        button(text("+ 新建分类树"))
                            .on_press(Message::ModalOpen(Modal::CreateTree))
                            .width(Length::Fill),
                        Space::new().height(4),
                        scrollable(tree_element),
                    ]
                    .spacing(4)
                    .padding(4);
                    
                    let left_panel = container(left_content)
                        .width(Length::FillPortion(2))
                        .height(Length::Fill);
                    
                    let right_panel = if let Some(ref selected_path) = self.category_tab.selected_path {
                        let card_data = self.review_tab.cards.iter()
                            .find(|(path, _)| path == selected_path);
                        
                        if let Some((path, card)) = card_data {
                                container(
                                    column![
                                        row![
                                            text("路径: "),
                                            text(path),
                                        ],
                                    Space::new().height(12),
                                    card_action_buttons(path),
                                    Space::new().height(12),
                                    button(text("开始计时").color(iced::Color::WHITE))
                                        .on_press(Message::QuickTimerStart(path.to_string()))
                                        .style(|_, _| iced::widget::button::Style {
                                            background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                                            text_color: iced::Color::WHITE,
                                            ..Default::default()
                                        }),
                                    Space::new().height(12),
                                    rule::horizontal(1.0),
                                    Space::new().height(12),
                                    crate::gui::components::CardDetail::view(path, card)
                                        .map(|_| Message::ClearError),
                                ]
                                .padding(16)
                            )
                            .width(Length::FillPortion(3))
                            .height(Length::Fill)
.style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.weak.color.into()),
                ..Default::default()
            }
        })
                        } else {
                            let children_count = self.count_tree_children(selected_path);
                            
                            container(
                                column![
                                    row![
                                        text("路径: "),
                                        text(selected_path),
                                    ],
                                    Space::new().height(12),
                                    folder_action_buttons(selected_path),
                                    Space::new().height(12),
                                    rule::horizontal(1.0),
                                    Space::new().height(12),
                                    row![
                                        text("子项数量: "),
                                        text(format!("{} 个", children_count)),
                                    ],
                                    Space::new().height(8),
                                    text("(选中具体节点查看详情)")
                                        .color(hint_text)
                                        .size(14),
                                ]
                                .padding(16)
                            )
                            .width(Length::FillPortion(3))
                            .height(Length::Fill)
.style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.weak.color.into()),
                ..Default::default()
            }
        })
                        }
                    } else {
                        container(
                            text("选择卡片查看详情")
                                .size(16)
                        )
                        .width(Length::FillPortion(3))
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::container::Style {
                                    background: Some(palette.background.weak.color.into()),
                                    ..Default::default()
                                }
                            })
                    };
                    
                    row![left_panel, right_panel]
                        .spacing(1)
                        .width(Length::Fill)
                        .height(Length::Fill)
    .into()
}

            }
            TabId::Review => {
                use crate::fsrs::FsrsPredictor;
                use review_tab::CardStats;
                
                let cards_with_urgency: Vec<(String, i32, Card)> = self.review_tab.cards.iter()
                    .filter_map(|(path, card)| {
                        card.prediction.as_ref().map(|pred| {
                            let urgency = FsrsPredictor::calculate_urgency(pred.next_review);
                            (path.clone(), urgency, card.clone())
                        })
                    })
                    .filter(|(_, urgency, _)| {
                        self.review_tab.urgency_filter.is_none_or(|filter| {
                            match filter {
                                3 => *urgency >= 3,
                                2 => *urgency >= 2,
                                _ => true,
                            }
                        })
                    })
                    .collect();
                
                let filter_buttons = row![
                    filter_button("全部", None, self.review_tab.urgency_filter),
                    filter_button("已过期", Some(3), self.review_tab.urgency_filter),
                    filter_button("今日", Some(2), self.review_tab.urgency_filter),
                    filter_button("本周", Some(1), self.review_tab.urgency_filter),
                    button(text("重新预测").color(iced::Color::WHITE))
                        .on_press(Message::RefreshPredictions)
                        .style(|_, _| iced::widget::button::Style {
                            background: Some(iced::Color::from_rgb(0.3, 0.5, 0.7).into()),
                            text_color: iced::Color::WHITE,
                            ..Default::default()
                        }),
                ]
                .spacing(8)
                .padding(8);
                
                let card_list: Element<Message> = if cards_with_urgency.is_empty() {
                    container(
                        text("暂无需要复习的卡片")
                            .size(16)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                } else {
                    scrollable(
                        column(
                            cards_with_urgency.iter().map(|(path, urgency, _)| {
                                let is_selected = self.review_tab.selected_card.as_ref() == Some(path);
                                let card_name = path.split('/').next_back().unwrap_or(path);
                                review_card_item(card_name.to_string(), path.clone(), *urgency, is_selected)
                            })
                        )
                        .spacing(4)
                    )
                    .into()
                };
                
                let left_panel = container(card_list)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
                    .padding(8);
                
                let right_panel: Element<Message> = if let Some(ref selected_path) = self.review_tab.selected_card {
                    if let Some((_, _, card)) = cards_with_urgency.iter()
                        .find(|(path, _, _)| path == selected_path) 
                    {
                        let stats = CardStats::from_card(card);
                        review_detail_panel(selected_path, card, &stats, hint_text)
                    } else {
                        container(
                            text("选择卡片查看详情")
                                .size(16)
                                .color(hint_text)
                        )
                        .width(Length::FillPortion(3))
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .into()
                    }
                } else {
                    container(
                        text("选择卡片查看详情")
                            .size(16)
                            .color(hint_text)
                    )
                    .width(Length::FillPortion(3))
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                };
                
                let content = column![
                    filter_buttons,
                    rule::horizontal(1.0),
                    row![left_panel, right_panel]
                        .spacing(1)
                        .width(Length::Fill)
                        .height(Length::Fill),
                ]
                .width(Length::Fill)
                .height(Length::Fill);
                
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::container::Style {
                            background: Some(palette.background.weak.color.into()),
                            ..Default::default()
                        }
                    })
                    .into()
            }
            TabId::Timer => {
                use crate::timer::TimerState;
                use crate::gui::components::TimerDisplay;
                
                if self.timer_tab.link_mode {
                    // 链接模态框模式 - 直接在 App::view 中构建，避免生命周期问题
                    let duration_ms = self.timer_tab.elapsed_ms;
                    let card_path = self.timer_tab.card_path_input.clone();
                    let selected_card = self.timer_tab.selected_card.clone();
                    let memory_quality = self.timer_tab.memory_quality.clone();
                    let new_card_name = self.timer_tab.new_card_name.clone();
                    let new_card_preset = self.timer_tab.new_card_preset.clone();
                    let new_card_type = self.timer_tab.new_card_type;
                    let error_message = self.error_message.clone();
                    let tree_nodes = &self.category_tab.tree_nodes;
                    let tree_view = &self.category_tab.tree_view;
                    
                    build_link_timer_modal(
                        duration_ms,
                        card_path,
                        selected_card,
                        memory_quality,
                        new_card_name,
                        new_card_preset,
                        new_card_type,
                        error_message,
                        tree_view,
                        tree_nodes,
                    )
                } else {
                    let timer_display = TimerDisplay::view(&self.timer_tab.state, self.timer_tab.elapsed_ms, self.timer_tab.current_card.as_deref())
                        .map(|_| Message::ClearError);
                    
                    let (start_btn, pause_btn, stop_btn) = match &self.timer_tab.state {
                        TimerState::Idle => {
                            (
                                button(text("开始").color(iced::Color::WHITE))
                                    .on_press(Message::TimerStarted)
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.2, 0.7, 0.4).into()),
                                        text_color: iced::Color::WHITE,
                                        ..Default::default()
                                    }),
                                button(text("暂停").color(hint_text)),
                                button(text("停止").color(hint_text)),
                            )
                        }
                        TimerState::Running { .. } => {
                            (
                                button(text("开始").color(hint_text)),
                                button(text("暂停").color(iced::Color::WHITE))
                                    .on_press(Message::TimerPaused)
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.95, 0.61, 0.07).into()),
                                        text_color: iced::Color::WHITE,
                                        ..Default::default()
                                    }),
                                button(text("停止").color(iced::Color::WHITE))
                                    .on_press(Message::TimerStopped(Ok(std::path::PathBuf::new())))
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.91, 0.30, 0.24).into()),
                                        text_color: iced::Color::WHITE,
                                        ..Default::default()
                                    }),
                            )
                        }
                        TimerState::Paused { .. } => {
                            (
                                button(text("开始").color(iced::Color::WHITE))
                                    .on_press(Message::TimerStarted)
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.2, 0.7, 0.4).into()),
                                        text_color: iced::Color::WHITE,
                                        ..Default::default()
                                    }),
                                button(text("暂停").color(hint_text)),
                                button(text("停止").color(iced::Color::WHITE))
                                    .on_press(Message::TimerStopped(Ok(std::path::PathBuf::new())))
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.91, 0.30, 0.24).into()),
                                        text_color: iced::Color::WHITE,
                                        ..Default::default()
                                    }),
                            )
                        }
                        TimerState::Stopped { .. } => {
                            (
                                button(text("开始").color(iced::Color::WHITE))
                                    .on_press(Message::TimerStarted)
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.2, 0.7, 0.4).into()),
                                        text_color: iced::Color::WHITE,
                                        ..Default::default()
                                    }),
                                button(text("暂停").color(hint_text)),
                                button(text("停止").color(hint_text)),
                            )
                        }
                    };
                    
                    let controls = row![start_btn, pause_btn, stop_btn]
                        .spacing(16)
                        .padding(16);
                    
                    container(
                        column![timer_display, controls]
                            .spacing(32)
                            .align_x(iced::Alignment::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::container::Style {
                            background: Some(palette.background.weak.color.into()),
                            ..Default::default()
                        }
                    })
                    .into()
                }
            }
            TabId::Todo => {
                let add_button = button(text("添加待办").color(iced::Color::WHITE))
                    .on_press(Message::ModalOpen(Modal::NewTodo))
                    .style(|_, _| iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    });
                
                let header = row![add_button]
                    .spacing(8)
                    .padding(8);
                
                let todo_list = if self.todo_tab.todos.is_empty() {
                    container(
                        text("暂无待办事项")
                            .size(16)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                } else {
                    container(
                        scrollable(
                            column(
                                self.todo_tab.todos.iter().map(|todo| {
                                    let checkbox_text = if todo.completed { "☑" } else { "☐" };
                                    
                                    let due_text = todo.due_date
                                        .map(|d| format!("截止: {}", d.format("%m-%d %H:%M")))
                                        .unwrap_or_default();
                                    
                                    let priority_badge = todo.priority
                                        .map(|p| format!("[P{}]", p))
                                        .unwrap_or_default();
                                    
                                    let tags_text = if todo.tags.is_empty() {
                                        String::new()
                                    } else {
                                        format!(" #{}", todo.tags.join(",#"))
                                    };
                                    
                                    let is_selected = self.todo_tab.selected_todo == Some(todo.id);
                                    
                                    let checkbox = button(text(checkbox_text))
                                        .on_press(Message::ToggleTodo(todo.id))
                                        .style(|theme: &iced::Theme, _| {
                                            let palette = theme.extended_palette();
                                            iced::widget::button::Style {
                                                background: Some(palette.background.strong.color.into()),
                                                text_color: palette.background.base.text,
                                                ..Default::default()
                                            }
                                        });
                                    
                                    let delete_btn = button(text("删除").color(iced::Color::from_rgb(0.9, 0.3, 0.2)))
                                        .on_press(Message::DeleteTodo(todo.id))
                                        .style(|theme: &iced::Theme, _| {
                                            let palette = theme.extended_palette();
                                            iced::widget::button::Style {
                                                background: Some(palette.background.strong.color.into()),
                                                text_color: iced::Color::from_rgb(0.9, 0.3, 0.2),
                                                ..Default::default()
                                            }
                                        });
                                    
                                    let edit_btn = button(text("编辑").color(iced::Color::from_rgb(0.4, 0.7, 0.9)))
                                        .on_press(Message::ModalOpen(Modal::EditTodo { id: todo.id }))
                                        .style(|theme: &iced::Theme, _| {
                                            let palette = theme.extended_palette();
                                            iced::widget::button::Style {
                                                background: Some(palette.background.strong.color.into()),
                                                text_color: iced::Color::from_rgb(0.4, 0.7, 0.9),
                                                ..Default::default()
                                            }
                                        });
                                    
                                    let content_row = row![
                                        checkbox,
                                        text(priority_badge).color(iced::Color::from_rgb(0.95, 0.61, 0.07)),
                                        text(&todo.content).color(if todo.completed { hint_text } else { palette.background.base.text }),
                                        text(due_text).color(hint_text),
                                        text(tags_text).color(iced::Color::from_rgb(0.4, 0.7, 0.9)),
                                        Space::new().width(Length::Fill),
                                        edit_btn,
                                        delete_btn,
                                    ]
                                    .spacing(8)
                                    .padding(8)
                                    .width(Length::Fill);
                                    
                                    button(content_row)
                                        .on_press(Message::TodoSelected(todo.id))
                                        .style(move |theme: &iced::Theme, _| {
                                            let palette = theme.extended_palette();
                                            iced::widget::button::Style {
                                                background: if is_selected {
                                                    Some(palette.background.strong.color.into())
                                                } else {
                                                    Some(palette.background.weak.color.into())
                                                },
                                                ..Default::default()
                                            }
                                        })
                                        .width(Length::Fill)
                                        .into()
                                })
                            )
                            .spacing(4)
                        )
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                };
                
                container(
                    column![
                        header,
                        rule::horizontal(1.0),
                        todo_list,
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        ..Default::default()
                    }
                })
                .into()
            }
            TabId::Schedule => {
                let add_button = button(text("新建日程").color(iced::Color::WHITE))
                    .on_press(Message::ScheduleCreateOpen)
                    .style(|_, _| iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    });

                let header = row![
                    text("日程").size(20),
                    Space::new().width(Length::Fill),
                    add_button,
                ]
                .padding(8);

                let schedule_list: Element<Message> = if self.schedule_tab.schedules.is_empty() {
                    container(
                        text("暂无日程安排")
                            .size(16)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                } else {
                    scrollable(
                        column(
                            self.schedule_tab.schedules.iter().map(|schedule| {
                                let weekday = weekday_cn(schedule.dtstart.weekday().num_days_from_monday());
                                let start_str = format!("{}/{} ({})",
                                    schedule.dtstart.format("%m"),
                                    schedule.dtstart.format("%d"),
                                    weekday);
                                let time_range = format!("{} - {}",
                                    schedule.dtstart.format("%H:%M"),
                                    schedule.dtend.format("%H:%M"));

                                let edit_btn = button(text("编辑").color(iced::Color::from_rgb(0.4, 0.7, 0.9)))
                                    .on_press(Message::ScheduleEditOpen(schedule.id))
                                    .style(|theme: &iced::Theme, _| {
                                        let palette = theme.extended_palette();
                                        iced::widget::button::Style {
                                            background: Some(palette.background.strong.color.into()),
                                            text_color: iced::Color::from_rgb(0.4, 0.7, 0.9),
                                            ..Default::default()
                                        }
                                    });

                                let delete_btn = button(text("删除").color(iced::Color::from_rgb(0.9, 0.3, 0.2)))
                                    .on_press(Message::ScheduleDelete(schedule.id))
                                    .style(|theme: &iced::Theme, _| {
                                        let palette = theme.extended_palette();
                                        iced::widget::button::Style {
                                            background: Some(palette.background.strong.color.into()),
                                            text_color: iced::Color::from_rgb(0.9, 0.3, 0.2),
                                            ..Default::default()
                                        }
                                    });

                                let location_text = schedule.location.as_ref()
                                    .map(|loc| format!("地点: {}", loc))
                                    .unwrap_or_default();

                                container(
                                    row![
                                        column![
                                            row![
                                                text(start_str).color(hint_text).size(13),
                                                Space::new().width(Length::Fixed(8.0)),
                                                text(time_range).color(hint_text).size(13),
                                                Space::new().width(Length::Fixed(8.0)),
                                                text(&schedule.summary).size(14),
                                            ],
                                            {
                                                let loc_elem: Element<Message> = if !location_text.is_empty() {
                                                    text(location_text).color(hint_text).size(12).into()
                                                } else {
                                                    row![].into()
                                                };
                                                loc_elem
                                            },
                                        ]
                                        .spacing(2),
                                        Space::new().width(Length::Fill),
                                        edit_btn,
                                        delete_btn,
                                    ]
                                    .spacing(8)
                                    .padding(8)
                                )
                                .style(|theme: &iced::Theme| {
                                    let palette = theme.extended_palette();
                                    iced::widget::container::Style {
                                        background: Some(palette.background.strong.color.into()),
                                        ..Default::default()
                                    }
                                })
                                .width(Length::Fill)
                                .into()
                            })
                        )
                        .spacing(4)
                    )
                    .into()
                };

                container(
                    column![
                        header,
                        rule::horizontal(1.0),
                        schedule_list,
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        ..Default::default()
                    }
                })
                .into()
            }
            TabId::Preset => {
                let preset_list: Vec<_> = self.preset_tab.presets.iter()
                    .map(|preset| {
                        let name = preset.name.clone();
                        let trained = if let Some(ts) = preset.trained_at {
                            ts.format("%Y-%m-%d %H:%M").to_string()
                        } else {
                            "未训练".to_string()
                        };
                        row![
                            text(preset.name.clone()).width(Length::Fill),
                            text(trained).color(hint_text),
                            button(text("编辑").size(12))
                                .style(iced::widget::button::text)
                                .on_press(Message::PresetEditOpen(name.clone())),
                            button(text("删除").size(12))
                                .style(iced::widget::button::text)
                                .on_press(Message::PresetDeleteRequested(name.clone())),
                        ]
                        .spacing(8)
                        .padding(8)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();

                let list = if self.preset_tab.presets.is_empty() {
                    container(text("暂无预设").color(hint_text))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                } else {
                    container(scrollable(column(preset_list).spacing(4)))
                        .width(Length::Fill)
                        .height(Length::Fill)
                };

                let mut col = column![
                    row![
                        text("预设").size(20),
                        iced::widget::Space::new().width(Length::Fill),
                        button(text("+ 新建"))
                            .on_press(Message::PresetCreateOpen),
                    ].padding(8),
                    rule::horizontal(1.0),
                    list,
                ];

                // form modal overlay
                if self.preset_tab.show_form {
                    let title = if self.preset_tab.editing_name.is_some() { "编辑预设" } else { "新建预设" };
                    let name_input = text_input("预设名称", &self.preset_tab.form_name)
                        .on_input(Message::PresetFormNameChanged);
                    let desc_input = text_input("描述（可选）", &self.preset_tab.form_description)
                        .on_input(Message::PresetFormDescriptionChanged);
                    let rules_label = row![
                        text("匹配规则"),
                        iced::widget::Space::new().width(Length::Fill),
                        button(text("+ 添加规则").size(12))
                            .style(iced::widget::button::text)
                            .on_press(Message::PresetFormMatchRuleAdded),
                    ];

                    let rule_inputs: Vec<iced::Element<'_, Message, iced::Theme>> = self.preset_tab.form_match_rules.iter().enumerate().map(|(i, rule)| {
                        row![
                            iced::widget::text_input("例如: main/english/**", rule.as_str())
                                .on_input(move |val| Message::PresetFormMatchRuleChanged(i, val)),
                            iced::widget::button(iced::widget::text("×").size(12))
                                .style(iced::widget::button::text)
                                .on_press(Message::PresetFormMatchRuleRemoved(i)),
                        ]
                        .spacing(4)
                        .into()
                    }).collect();

                    let mut form_col = column![
                        text(title).size(16),
                        name_input,
                        desc_input,
                        rules_label,
                    ]
                    .spacing(8);

                    for rule_el in rule_inputs {
                        form_col = form_col.push(rule_el);
                    }

                    if let Some(ref err) = self.preset_tab.form_error {
                        form_col = form_col.push(text(err.clone()).color(iced::Color::from_rgb(1.0, 0.3, 0.3)));
                    }

                    form_col = form_col.push(
                        row![
                            iced::widget::Space::new().width(Length::Fill),
                            button(text("取消")).on_press(Message::PresetFormDismissed),
                            button(text("保存")).on_press(Message::PresetFormSaveRequested),
                        ].spacing(8)
                    );

                    col = col.push(
                        container(form_col.spacing(8).padding(16))
                            .width(Length::Fill)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::container::Style {
                                    background: Some(palette.background.base.color.into()),
                                    border: iced::Border::default().rounded(4),
                                    ..Default::default()
                                }
                            })
                    );
                }

                // delete confirm overlay
                if let Some(ref target) = self.preset_tab.delete_target {
                    let confirm_col = column![
                        text(format!("确定删除预设「{target}」？")),
                        row![
                            iced::widget::Space::new().width(Length::Fill),
                            button(text("取消")).on_press(Message::PresetDeleteDismissed),
                            button(text("删除")).on_press(Message::PresetDeleteConfirmed(target.clone())),
                        ].spacing(8),
                    ].spacing(8).padding(16);

                    col = col.push(
                        container(confirm_col)
                            .width(Length::Fill)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::container::Style {
                                    background: Some(palette.background.base.color.into()),
                                    border: iced::Border::default().rounded(4),
                                    ..Default::default()
                                }
                            })
                    );
                }

                container(col.spacing(4))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::container::Style {
                            background: Some(palette.background.weak.color.into()),
                            ..Default::default()
                        }
                    })
                    .into()
            }
            TabId::Settings => {
                let mut col = column![
                    text("设置").size(20),
                    rule::horizontal(1.0),
                    row![
                        text("配置目录:"),
                        text(self.settings_tab.config_dir.to_string_lossy().to_string())
                            .color(hint_text),
                    ].spacing(8).padding(8),
                    row![
                        text("数据目录:"),
                        iced::widget::text_input("数据目录路径", &self.settings_tab.form_data_dir)
                            .on_input(Message::SettingsFormDataDirChanged),
                    ].spacing(8).padding(8),
                    row![
                        text("默认预设:"),
                        {
                            let presets: Vec<String> = self.preset_tab.presets.iter()
                                .map(|p| p.name.clone())
                                .collect();
                            iced::widget::pick_list(
                                presets,
                                Some(self.settings_tab.form_default_preset.clone()),
                                Message::SettingsFormDefaultPresetChanged,
                            )
                        },
                    ].spacing(8).padding(8),
                    row![
                        text("主题:"),
                        iced::widget::pick_list(
                            vec![
                                AppTheme::Light,
                                AppTheme::Dark,
                                #[cfg(target_os = "linux")]
                                AppTheme::Transparent,
                            ],
                            Some(self.theme),
                            Message::ThemeChanged,
                        ),
                    ].spacing(8).padding(8),
                    iced::widget::button(iced::widget::text("保存"))
                        .on_press(Message::SettingsFormSaveRequested)
                        .padding(8),
                ]
                .spacing(4);

                if let Some(ref msg) = self.settings_tab.message {
                    let color = if self.settings_tab.message_is_error {
                        iced::Color::from_rgb(1.0, 0.3, 0.3)
                    } else {
                        iced::Color::from_rgb(0.3, 1.0, 0.3)
                    };
                    let msg_row = row![
                        text(msg.clone()).color(color),
                        iced::widget::button(iced::widget::text("×").size(12))
                            .style(iced::widget::button::text)
                            .on_press(Message::SettingsFormDismissMessage),
                    ].spacing(8).padding(8);
                    col = col.push(msg_row);
                }

                container(col)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::container::Style {
                            background: Some(palette.background.weak.color.into()),
                            ..Default::default()
                        }
                    })
                    .into()
            }
        };
        
        let base_view = column![tabs, rule::horizontal(1.0), content]
            .width(Length::Fill)
            .height(Length::Fill);
        
        if let Some(ref modal) = self.modal {
            let modal_title = match modal {
                Modal::NewTodo => "新建待办",
                Modal::EditTodo { .. } => "编辑待办",
                Modal::NewSchedule => "新建日程",
                Modal::NewNode => "新建节点",
                Modal::EditCard { .. } => "编辑卡片",
                Modal::ConfirmDelete { .. } => "确认删除",
                Modal::Error { .. } => "错误",
                Modal::Info { .. } => "提示",
                Modal::CreateTree => "新建分类树",
                _ => "确认",
            };
            
            let (modal_content, on_confirm_tuple) = match modal {
                Modal::NewTodo | Modal::EditTodo { .. } => {
                    (
                        NewTodoForm::view(
                            &self.todo_tab.form_content,
                            self.todo_tab.form_priority,
                            &self.todo_tab.form_due_year,
                            &self.todo_tab.form_due_month,
                            &self.todo_tab.form_due_day,
                            &self.todo_tab.form_due_hour,
                            &self.todo_tab.form_due_minute,
                            &self.todo_tab.form_tags,
                        ),
                        (Message::ModalConfirm, "确认", true),
                    )
                }
                Modal::ConfirmDelete { item } => {
                    let is_folder = self.category_tab.delete_target
                        .as_ref()
                        .map(|(_, is_folder)| *is_folder)
                        .unwrap_or(false);
                    
                    let warning = if is_folder {
                        "⚠️ 文件夹及其所有内容将被删除"
                    } else {
                        "⚠️ 此操作不可撤销"
                    };
                    
                    (
                        column![
                            text(format!("确定要删除 \"{}\" 吗？", item))
                                .size(16),
                            Space::new().height(12),
                            text(warning)
                                .color(iced::Color::from_rgb(0.9, 0.5, 0.3))
                                .size(14),
                        ]
                        .spacing(12)
                        .into(),
                        (Message::DeleteNodeConfirm, "删除", true),
                    )
                }
                Modal::Info { message } => {
                    (
                        column![
                            text(message.clone())
                                .color(iced::Color::from_rgb(0.3, 0.8, 0.3)),
                        ]
                        .spacing(12)
                        .into(),
                        (Message::ModalClose, "关闭", false),
                    )
                }

                Modal::Error { message } => {
                    (
                        column![
                            text("错误")
                                .size(16),
                            text(message.clone())
                                .color(iced::Color::from_rgb(0.9, 0.3, 0.2)),
                        ]
                        .spacing(12)
                        .into(),
                        (Message::ModalClose, "关闭", false),
                    )
                }
                Modal::NewSchedule => {
                    use crate::data::models::RecurrenceRule;
                    let form = &self.schedule_tab.form;
                    let rrule_options = vec!["无".to_string(), "每天".to_string(), "每周".to_string(), "每月".to_string()];

                    let content = column![
                        row![
                            text("标题:"),
                            text_input("日程标题", &form.summary)
                                .on_input(Message::ScheduleFormSummaryChanged)
                                .width(Length::Fill),
                        ].spacing(8).padding(4),
                        row![
                            text("开始:").color(hint_text),
                            text_input("日期", &form.start_date)
                                .on_input(Message::ScheduleFormStartDateChanged)
                                .width(Length::Fixed(120.0)),
                            Space::new().width(Length::Fixed(8.0)),
                            text_input("时间", &form.start_time)
                                .on_input(Message::ScheduleFormStartTimeChanged)
                                .width(Length::Fixed(80.0)),
                        ].spacing(8).padding(4),
                        row![
                            text("结束:").color(hint_text),
                            text_input("日期", &form.end_date)
                                .on_input(Message::ScheduleFormEndDateChanged)
                                .width(Length::Fixed(120.0)),
                            Space::new().width(Length::Fixed(8.0)),
                            text_input("时间", &form.end_time)
                                .on_input(Message::ScheduleFormEndTimeChanged)
                                .width(Length::Fixed(80.0)),
                        ].spacing(8).padding(4),
                        row![
                            text("地点:").color(hint_text),
                            text_input("可选", &form.location)
                                .on_input(Message::ScheduleFormLocationChanged)
                                .width(Length::Fill),
                        ].spacing(8).padding(4),
                        row![
                            text("描述:").color(hint_text),
                            text_input("可选", &form.description)
                                .on_input(Message::ScheduleFormDescriptionChanged)
                                .width(Length::Fill),
                        ].spacing(8).padding(4),
                        row![
                            text("提醒:").color(hint_text),
                            text_input("分钟前", &form.reminder)
                                .on_input(Message::ScheduleFormReminderChanged)
                                .width(Length::Fixed(80.0)),
                        ].spacing(8).padding(4),
                        row![
                            text("重复:").color(hint_text),
                            pick_list(rrule_options, Some(form.rrule.as_str().to_string()), move |s| {
                                Message::ScheduleFormRruleChanged(RecurrenceRule::parse(&s))
                            }).width(Length::Fixed(100.0)),
                        ].spacing(8).padding(4),
                        {
                            let err_elem: Element<Message> = if let Some(ref err) = self.schedule_tab.form_error {
                                text(err.clone()).color(iced::Color::from_rgb(0.9, 0.3, 0.2)).size(13).into()
                            } else {
                                row![].into()
                            };
                            err_elem
                        },
                    ].spacing(4).padding(8);

                    let confirm_msg = if self.schedule_tab.editing_id.is_some() {
                        Message::ScheduleEditConfirm
                    } else {
                        Message::ScheduleCreateConfirm
                    };

                    (content.into(), (confirm_msg, "保存", true))
                }

                Modal::NewNode => {
                    if let Some(ref form) = self.category_tab.new_node_form {
                        let presets: Vec<String> = self.preset_tab.presets.iter()
                            .map(|p| p.name.clone())
                            .collect();
                        (NewNodeModal::view(form, &presets), (Message::NewNodeConfirm, "创建", true))
                    } else {
                        (text("开发中").into(), (Message::ModalClose, "关闭", false))
                    }
                }
                Modal::EditCard { .. } => {
                    if let Some(ref form) = self.category_tab.edit_card_form {
                        let presets: Vec<String> = self.preset_tab.presets.iter()
                            .map(|p| p.name.clone())
                            .collect();
                        (simple_edit_card_view(form, &presets, hint_text), (Message::EditCardConfirm, "保存", true))
                    } else {
                        (text("开发中").into(), (Message::ModalClose, "关闭", false))
                    }
                }
                Modal::CreateTree => {
                    (
                        column![
                            text("输入新分类树名称:"),
                            text_input("分类树名称", &self.category_tab.new_tree_name)
                                .on_input(Message::CreateTreeNameChanged),
                            Space::new().height(8),
                            text("从 Obsidian 导入（可选）:").color(hint_text),
                            text_input("Obsidian 仓库路径", &self.category_tab.new_tree_import_path)
                                .on_input(Message::CreateTreeImportPathChanged),
                            text("忽略规则文件（可选）:").color(hint_text),
                            text_input("规则文件路径，留空使用默认", &self.category_tab.new_tree_import_rules_path)
                                .on_input(Message::CreateTreeImportRulesPathChanged),
                        ]
                        .spacing(8)
                        .into(),
                        (Message::ModalConfirm, "创建", true),
                    )
                }
                _ => (text("开发中").into(), (Message::ModalClose, "关闭", false))
            };
            
            let (on_confirm, confirm_label, show_cancel) = on_confirm_tuple;
            
            if show_cancel {
                ModalView::view_with_options(
                    modal_title,
                    modal_content,
                    on_confirm,
                    Message::ModalClose,
                    confirm_label,
                    true,
                )
            } else {
                ModalView::view_with_options(
                    modal_title,
                    modal_content,
                    on_confirm,
                    Message::ModalClose,
                    confirm_label,
                    false,
                )
            }
        } else {
            base_view.into()
        }
    }
}

fn simple_edit_card_view(form: &category_tab::EditCardForm, presets: &[String], hint_text: iced::Color) -> Element<'static, Message> {
    let presets_owned = presets.to_vec();
    let new_name = form.new_name.clone();
    let preset = form.preset.clone();
    
    let review_list: Element<Message> = if form.review_records.is_empty() {
        text("暂无复习记录").color(hint_text).into()
    } else {
        column(
            form.review_records.iter().enumerate().map(|(i, r)| {
                let timestamp = r.timestamp.format("%Y-%m-%d").to_string();
                let duration = r.duration_ms / 60000;
                let quality = r.memory_quality.as_str();
                
                row![
                    text(timestamp).width(Length::Fixed(100.0)),
                    text(format!("{}分", duration)).width(Length::Fixed(60.0)),
                    text(quality).width(Length::Fixed(60.0)),
                    button(text("×").color(iced::Color::WHITE))
                        .on_press(Message::EditCardRemoveReview(i))
                        .style(|_, _| iced::widget::button::Style {
                            background: Some(iced::Color::from_rgb(0.5, 0.3, 0.3).into()),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(8)
                .into()
            })
        )
        .spacing(4)
        .into()
    };
    
    column![
        text("名称:"),
        text_input("名称", &new_name)
            .on_input(Message::EditCardNameChanged)
            .width(Length::Fill),
        Space::new().height(12),
        
        text("预设:"),
        pick_list(presets_owned, Some(preset), Message::EditCardPresetChanged)
            .width(Length::Fill),
        Space::new().height(12),
        
        text("下次复习:"),
        row![
            if let Some(next) = form.next_review {
                text(next.format("%Y-%m-%d").to_string()).color(iced::Color::from_rgb(0.5, 0.8, 0.5))
            } else {
                text("未设置").color(hint_text)
            },
            button(text("清除").color(iced::Color::WHITE).size(12))
                .on_press(Message::EditCardClearPrediction)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.5, 0.3, 0.3).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text("预测").color(iced::Color::WHITE).size(12))
                .on_press(Message::EditCardPredict)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.5, 0.7).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8),
        Space::new().height(12),
        
        text(format!("复习记录 ({} 条)", form.review_records.len())),
        review_list,
        Space::new().height(8),
        
        text("添加新记录:"),
        row![
            text_input("时长(分钟)", &form.new_review_duration)
                .on_input(Message::EditCardNewReviewDurationChanged)
                .width(Length::Fixed(100.0)),
            pick_list(
                vec!["好".to_string(), "简单".to_string(), "困难".to_string(), "重学".to_string()],
                Some(form.new_review_quality.as_str().to_string()),
                move |s| {
                    let quality = crate::data::models::MemoryQuality::parse(&s)
                        .unwrap_or(crate::data::models::MemoryQuality::Good);
                    Message::EditCardNewReviewQualityChanged(quality)
                }
            ),
            button(text("添加").color(iced::Color::WHITE).size(12))
                .on_press(Message::EditCardAddReview)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.5, 0.7).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8),
    ]
    .padding(16)
    .spacing(8)
    .into()
}

fn card_action_buttons(path: &str) -> Element<'_, Message> {
    let path = path.to_string();
    row![
        button(
            text("编辑").color(iced::Color::WHITE)
        )
        .on_press(Message::EditCardOpen(path.clone()))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.3, 0.5, 0.7).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        button(
            text("删除").color(iced::Color::WHITE)
        )
        .on_press(Message::DeleteNodeOpen(path.clone(), false))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.7, 0.3, 0.3).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .spacing(8)
    .into()
}

fn folder_action_buttons(path: &str) -> Element<'_, Message> {
    let path = path.to_string();
    let is_tree_root = !path.contains('/');
    let btns = row![
        button(
            text("新建卡片").color(iced::Color::WHITE)
        )
        .on_press(Message::NewNodeOpen(NodeType::Card))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.4).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        button(
            text("新建文件夹").color(iced::Color::WHITE)
        )
        .on_press(Message::NewNodeOpen(NodeType::Folder))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.4, 0.5, 0.6).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        button(text("删除").color(iced::Color::WHITE))
            .on_press(Message::DeleteNodeOpen(path.clone(), is_tree_root))
            .style(|_, _| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.7, 0.3, 0.3).into()),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .spacing(8);

    btns.into()
}

fn tab_button(label: &str, tab_id: TabId, active_tab: TabId) -> Element<'_, Message> {
    let is_active = tab_id == active_tab;
    
    if is_active {
        button(text(label).color(iced::Color::WHITE))
            .on_press(Message::SwitchTab(tab_id))
            .style(|_, _| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                text_color: iced::Color::WHITE,
                ..Default::default()
            })
            .into()
    } else {
        button(text(label))
            .on_press(Message::SwitchTab(tab_id))
            .style(|theme: &iced::Theme, _| {
                let palette = theme.extended_palette();
                iced::widget::button::Style {
                    background: Some(palette.background.weak.color.into()),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            })
            .into()
    }
}

fn filter_button(label: &str, filter: Option<u32>, current: Option<u32>) -> Element<'_, Message> {
    let is_active = filter == current;
    
    if is_active {
        button(text(label).color(iced::Color::WHITE))
            .on_press(Message::UrgencyFilterChanged(filter))
            .style(|_, _| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                text_color: iced::Color::WHITE,
                ..Default::default()
            })
            .into()
    } else {
        button(text(label))
            .on_press(Message::UrgencyFilterChanged(filter))
            .style(|theme: &iced::Theme, _| {
                let palette = theme.extended_palette();
                iced::widget::button::Style {
                    background: Some(palette.background.weak.color.into()),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            })
            .into()
    }
}

fn review_card_item(card_name: String, path: String, urgency: i32, is_selected: bool) -> Element<'static, Message> {
    let (urgency_text, urgency_color) = match urgency {
        3 => ("已过期", iced::Color::from_rgb(0.9, 0.3, 0.2)),
        2 => ("今日", iced::Color::from_rgb(0.9, 0.7, 0.2)),
        1 => ("本周", iced::Color::from_rgb(0.3, 0.7, 0.4)),
        _ => ("稍后", iced::Color::from_rgb(0.5, 0.5, 0.5)),
    };
    
    button(
        column![
            text(card_name).size(13),
            text(urgency_text).color(urgency_color).size(11),
        ]
        .spacing(2)
    )
    .on_press(Message::ReviewCardSelected(path))
    .style(move |theme: &iced::Theme, _| {
        let palette = theme.extended_palette();
        if is_selected {
            iced::widget::button::Style {
                background: Some(palette.primary.strong.color.into()),
                text_color: palette.primary.strong.text,
                ..Default::default()
            }
        } else {
            iced::widget::button::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: palette.background.base.text,
                ..Default::default()
            }
        }
    })
    .width(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn build_link_timer_modal(
    duration_ms: i64,
    card_path: String,
    selected_card: Option<String>,
    memory_quality: crate::data::models::MemoryQuality,
    new_card_name: String,
    new_card_preset: String,
    new_card_type: NodeType,
    error_message: Option<String>,
    tree_view: &crate::gui::components::TreeView,
    tree_nodes: &[crate::gui::components::tree_view::TreeNode],
) -> Element<'static, Message> {
    use iced::widget::{button, column, row, text, container, text_input, pick_list, Space, scrollable};
    use iced::{Length, Color};
    
    let seconds = duration_ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    
    let duration_text = if hours > 0 {
        format!("{}小时 {:02}分钟 {:02}秒", hours, minutes % 60, seconds % 60)
    } else if minutes > 0 {
        format!("{}分钟 {:02}秒", minutes, seconds % 60)
    } else {
        format!("{}秒", seconds)
    };
    
    // 树形选择器部分 - 使用静态引用避免生命周期问题
    let selected_card_str = selected_card.clone();
    let tree_picker: Element<Message> = container(
        scrollable(
            tree_view.view_static(tree_nodes, selected_card_str)
                .map(Message::TimerCardSelected)
        )
        .height(Length::Fixed(200.0))
    )
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.base.color.into()),
            ..Default::default()
        }
    })
    .padding(8)
    .into();

    let type_options = [
        (NodeType::Folder, "目录"),
        (NodeType::Card, "学习卡片"),
    ];
    let type_labels: Vec<String> = type_options.iter().map(|(_, l)| l.to_string()).collect();
    let current_type_label = type_options.iter()
        .find(|(t, _)| *t == new_card_type)
        .map(|(_, l)| l.to_string())
        .unwrap_or_default();
    
    let mut col = column![]
        .spacing(8);
    
    col = col
        .push(text(format!("学习时长: {}", duration_text))
            .size(16))
        .push(Space::new().height(16))
        .push(row![
            text("关联:"),
            text_input("输入卡片路径或从下方选择...", &card_path)
                .on_input(Message::TimerCardPathChanged),
        ])
        .push(tree_picker)
        .push(text("创建:").size(14))
        .push(row![
            text("类型:"),
            pick_list(type_labels, Some(current_type_label), move |s| {
                let t = if s == "目录" { NodeType::Folder } else { NodeType::Card };
                Message::TimerNewCardTypeChanged(t)
            }).width(Length::Fixed(150.0)),
        ]
        .spacing(16))
        .push(row![
            text("名称:"),
            text_input("输入名称...", &new_card_name)
                .on_input(Message::TimerNewCardNameChanged),
        ]
        .spacing(8));
    
    if new_card_type == NodeType::Card {
        col = col.push(row![
            text("预设:"),
            text_input("default", &new_card_preset)
                .on_input(Message::TimerNewCardPresetChanged),
        ]
        .spacing(8));
    }
    
    let error_text = error_message.clone().unwrap_or_default();
    if !error_text.is_empty() {
        col = col.push(text(error_text).color(Color::from_rgb(0.9, 0.3, 0.2)).size(13));
    }
    
    col = col
        .push(button(text("创建"))
            .on_press(Message::TimerNewCardConfirm)
            .style(|theme: &iced::Theme, _| {
                let palette = theme.extended_palette();
                iced::widget::button::Style {
                    background: Some(palette.primary.strong.color.into()),
                    text_color: palette.primary.strong.text,
                    ..Default::default()
                }
            }))
        .push(Space::new().height(16))
        .push(row![
            text("记忆质量:"),
            MemoryQualitySelector::view(&memory_quality),
        ])
        .push(Space::new().height(24))
        .push(row![
            button(text("取消"))
                .on_press(Message::TimerLinkModeClose)
                .style(|theme: &iced::Theme, _| {
                    let palette = theme.extended_palette();
                    iced::widget::button::Style {
                        background: Some(palette.background.strong.color.into()),
                        text_color: palette.background.base.text,
                        ..Default::default()
                    }
                }),
            button(text("保存并预测"))
                .on_press(Message::TimerLinkConfirm)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                    text_color: iced::Color::WHITE,
                    ..Default::default()
                }),
        ]
        .spacing(12));
    
    container(col)
        .padding(24)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                ..Default::default()
            }
        })
        .into()
}

struct MemoryQualitySelector;

impl MemoryQualitySelector {
    fn view(quality: &crate::data::models::MemoryQuality) -> Element<'static, Message> {
        use iced::widget::row;
        
        row![
            quality_button("重学", crate::data::models::MemoryQuality::Relearn, quality),
            quality_button("困难", crate::data::models::MemoryQuality::Hard, quality),
            quality_button("好", crate::data::models::MemoryQuality::Good, quality),
            quality_button("简单", crate::data::models::MemoryQuality::Easy, quality),
        ]
        .spacing(8)
        .into()
    }
}

fn quality_button(label: &'static str, quality: crate::data::models::MemoryQuality, current: &crate::data::models::MemoryQuality) -> Element<'static, Message> {
    let is_selected = *current == quality;
    
    button(text(label))
        .on_press(Message::TimerMemoryQualityChanged(quality))
        .style(move |theme: &iced::Theme, _| {
            let palette = theme.extended_palette();
            if is_selected {
                iced::widget::button::Style {
                    background: Some(palette.primary.strong.color.into()),
                    text_color: palette.primary.strong.text,
                    ..Default::default()
                }
            } else {
                iced::widget::button::Style {
                    background: Some(palette.background.strong.color.into()),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            }
        })
        .into()
}
fn review_detail_panel(path: &str, _card: &Card, stats: &review_tab::CardStats, hint_text: iced::Color) -> Element<'static, Message> {
    let path_display = path.split('/')
        .collect::<Vec<_>>()
        .join(" > ");
    
    let total_min = stats.total_duration_ms / 60000;
    let total_hours = total_min / 60;
    let total_mins = total_min % 60;
    let avg_min = stats.avg_duration_ms / 60000;
    
    let last_review = stats.last_review
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "无".to_string());
    
    let next_review = stats.next_review
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "未预测".to_string());
    
    let path_str = path.to_string();
    container(
        column![
            text(path_display).color(hint_text).size(12),
            Space::new().height(16),
            
            row![
                text("会话: ").color(hint_text),
                text(format!("{} 次", stats.sessions)),
            ],
            row![
                text("时长: ").color(hint_text),
                text(format!("{}h {}m", total_hours, total_mins)),
            ],
            row![
                text("平均: ").color(hint_text),
                text(format!("{} 分钟", avg_min)),
            ],
            Space::new().height(8),
            
            row![
                text("上次: ").color(hint_text),
                text(last_review),
            ],
            row![
                text("下次: ").color(hint_text),
                text(next_review),
            ],
            Space::new().height(24),
            
            row![
                button(text("开始计时").color(iced::Color::WHITE))
                    .on_press(Message::StartReviewTimer(path_str.clone()))
                    .style(|_, _| iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                button(text("编辑").color(iced::Color::WHITE))
                    .on_press(Message::EditCardOpen(path_str))
                    .style(|_, _| iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.4, 0.4, 0.5).into()),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(12),
        ]
        .spacing(4)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(16)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(palette.background.weak.color.into()),
            ..Default::default()
        }
    })
    .into()
}

fn weekday_cn(days_from_monday: u32) -> &'static str {
    match days_from_monday {
        0 => "一",
        1 => "二",
        2 => "三",
        3 => "四",
        4 => "五",
        5 => "六",
        6 => "日",
        _ => "",
    }
}