use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;

use iced::{application, Element, Task};
use iced::widget::{button, column, row, text, container, rule, scrollable, text_input, pick_list, Space};
use iced::Length;
use uuid::Uuid;
use chrono::Utc;

use crate::data::DataFs;
use crate::data::models::{Card, Todo};
use crate::gui::{Message, TabId, Modal, DataSnapshot, NodeType};
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
                
                let data_fs = DataFs::init(data_dir.clone()).expect("Failed to init data dir");
                let timer_manager = TimerManager::new(data_dir.clone());
                let data_fs_arc = Arc::new(data_fs);
                
                let state = App {
                    active_tab: TabId::Category,
                    data_dir: data_dir.clone(),
                    data_fs: (*data_fs_arc).clone(),
                    
                    category_tab: CategoryTabState::default(),
                    review_tab: ReviewTabState::default(),
                    timer_tab: TimerTabState::new(timer_manager.get_state().clone()),
                    schedule_tab: ScheduleTabState::default(),
                    todo_tab: TodoTabState::default(),
                    preset_tab: PresetTabState::default(),
                    settings_tab: SettingsTabState::new(data_dir.clone()),
                    
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
        .title(App::title)
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
                    let form = category_tab::NewNodeForm::new(path.clone(), default_type);
                    self.category_tab.new_node_form = Some(form);
                    self.modal = Some(Modal::NewNode);
                }
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
                            self.data_fs.create_tree(&new_path)
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
                    form.clear_prediction = !form.clear_prediction;
                    if form.clear_prediction {
                        form.next_review = None;
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
                if let Some(ref mut form) = self.category_tab.edit_card_form {
                    if index < form.review_records.len() {
                        form.review_records.remove(index);
                    }
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
                if let Some(ref form) = self.category_tab.edit_card_form {
                    let mut card = Card::new();
                    card.review_records = form.review_records.clone();
                    
                    use crate::data::models::Prediction;
                    if form.clear_prediction {
                        card.prediction = None;
                    } else if let Some(next_review) = form.next_review {
                        card.prediction = Some(Prediction {
                            algorithm: "fsrs".to_string(),
                            next_review,
                            fsrs_state_bytes: vec![],
                            preset_used: form.preset.clone(),
                        });
                    }
                    
                    let final_path = if form.new_name != form.original_name {
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
                    let result = if is_folder {
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
            
            Message::TimerStarted => {
                self.timer_manager.start();
                self.timer_tab.state = self.timer_manager.get_state().clone();
                Task::none()
            }
            
            Message::TimerPaused => {
                self.timer_manager.pause();
                self.timer_tab.state = self.timer_manager.get_state().clone();
                Task::none()
            }
            
            Message::TimerStopped(result) => {
                match result {
                    Ok(_path) => {
                        self.timer_manager.stop();
                        self.timer_tab.state = self.timer_manager.get_state().clone();
                        self.timer_tab.elapsed_ms = 0;
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
            
            _ => Task::none(),
        }
    }
    
    fn build_tree_nodes(&self, trees: &[String], cards: &[(String, Card)]) -> Vec<TreeNode> {
        let mut result: Vec<TreeNode> = Vec::new();
        
        for tree_name in trees {
            let tree_cards: Vec<(String, &Card)> = cards
                .iter()
                .filter(|(path, _)| path.starts_with(&format!("{}/", tree_name)))
                .map(|(path, card)| (path.clone(), card))
                .collect();
            
            let mut tree_node = TreeNode {
                name: tree_name.clone(),
                path: tree_name.clone(),
                is_card: false,
                children: Vec::new(),
            };
            
            for (card_path, _card) in tree_cards {
                let relative_path = card_path.strip_prefix(&format!("{}/", tree_name)).unwrap_or(&card_path);
                let parts: Vec<&str> = relative_path.split('/').collect();
                
                if parts.len() == 1 {
                    tree_node.children.push(TreeNode {
                        name: parts[0].to_string(),
                        path: card_path.clone(),
                        is_card: true,
                        children: Vec::new(),
                    });
                } else {
                    let folder_name = parts[0].to_string();
                    let folder_path = format!("{}/{}", tree_name, folder_name);
                    
                    let folder = tree_node.children.iter_mut()
                        .find(|n| n.path == folder_path);
                    
                    if let Some(folder) = folder {
                        let card_name = parts.last().unwrap().to_string();
                        folder.children.push(TreeNode {
                            name: card_name,
                            path: card_path.clone(),
                            is_card: true,
                            children: Vec::new(),
                        });
                    } else {
                        let mut new_folder = TreeNode {
                            name: folder_name.clone(),
                            path: folder_path.clone(),
                            is_card: false,
                            children: Vec::new(),
                        };
                        
                        let card_name = parts.last().unwrap().to_string();
                        new_folder.children.push(TreeNode {
                            name: card_name,
                            path: card_path.clone(),
                            is_card: true,
                            children: Vec::new(),
                        });
                        
                        tree_node.children.push(new_folder);
                    }
                }
            }
            
            result.push(tree_node);
        }
        
        result
    }
    
    fn view(&self) -> Element<Message> {
        use iced::widget::scrollable;
        
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
                        text("暂无数据 - 请使用 CLI 创建分类树")
                            .size(16)
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
                    .style(move |_: &iced::Theme, _| {
                        iced::widget::text_input::Style {
                            background: iced::Color::from_rgb(0.25, 0.25, 0.25).into(),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            icon: iced::Color::from_rgb(0.5, 0.5, 0.5),
                            placeholder: iced::Color::from_rgb(0.5, 0.5, 0.5),
                            value: iced::Color::WHITE,
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
                                                .color(iced::Color::WHITE)
                                                .size(14),
                                        ]
                                        .spacing(4)
                                    )
                                    .on_press(Message::SearchItemSelected(i))
                                    .style(move |_, _| {
                                        if is_selected {
                                            iced::widget::button::Style {
                                                background: Some(iced::Color::from_rgb(0.3, 0.6, 0.9).into()),
                                                text_color: iced::Color::WHITE,
                                                ..Default::default()
                                            }
                                        } else {
                                            iced::widget::button::Style {
                                                background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
                                                text_color: iced::Color::WHITE,
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
                    
                    let tree_element = self.category_tab.tree_view.view(
                        &self.category_tab.tree_nodes,
                        self.category_tab.selected_path.as_deref(),
                    ).map(|path| Message::CardSelected(path));
                    
                    let left_content = column![
                        search_input,
                        search_results,
                        Space::new().height(8),
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
                                        text("路径: ").color(iced::Color::WHITE),
                                        text(path).color(iced::Color::WHITE),
                                    ],
                                    Space::new().height(12),
                                    card_action_buttons(path),
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
                            .style(|_: &iced::Theme| iced::widget::container::Style {
                                background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                                ..Default::default()
                            })
                        } else {
                            let children_count = self.count_tree_children(selected_path);
                            
                            container(
                                column![
                                    row![
                                        text("路径: ").color(iced::Color::WHITE),
                                        text(selected_path).color(iced::Color::WHITE),
                                    ],
                                    Space::new().height(12),
                                    folder_action_buttons(selected_path),
                                    Space::new().height(12),
                                    rule::horizontal(1.0),
                                    Space::new().height(12),
                                    row![
                                        text("子项数量: ").color(iced::Color::WHITE),
                                        text(format!("{} 个", children_count)).color(iced::Color::WHITE),
                                    ],
                                    Space::new().height(8),
                                    text("(选中具体节点查看详情)")
                                        .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                                        .size(14),
                                ]
                                .padding(16)
                            )
                            .width(Length::FillPortion(3))
                            .height(Length::Fill)
                            .style(|_: &iced::Theme| iced::widget::container::Style {
                                background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                                ..Default::default()
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
                        .style(|_: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                            ..Default::default()
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
                
                let cards_with_urgency: Vec<(String, i32, &Card)> = self.review_tab.cards.iter()
                    .filter_map(|(path, card)| {
                        card.prediction.as_ref().map(|pred| {
                            let urgency = FsrsPredictor::calculate_urgency(pred.next_review);
                            (path.clone(), urgency, card)
                        })
                    })
                    .filter(|(_, urgency, _)| {
                        self.review_tab.urgency_filter
                            .map_or(true, |filter| *urgency as u32 == filter)
                    })
                    .collect();
                
                let filter_buttons = row![
                    filter_button("全部", None, self.review_tab.urgency_filter),
                    filter_button("已过期", Some(3), self.review_tab.urgency_filter),
                    filter_button("今日", Some(2), self.review_tab.urgency_filter),
                    filter_button("近期", Some(1), self.review_tab.urgency_filter),
                    filter_button("稍后", Some(0), self.review_tab.urgency_filter),
                ]
                .spacing(8)
                .padding(8);
                
                let card_list = if cards_with_urgency.is_empty() {
                    container(
                        text("暂无需要复习的卡片")
                            .size(16)
                            .color(iced::Color::WHITE)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                } else {
                    container(
                        scrollable(
                            column(
                                cards_with_urgency.iter().map(|(path, urgency, _)| {
                                    row![
                                        container(
                                            crate::gui::components::UrgencyBadge::view(*urgency as u32)
                                                .map(|_| Message::ClearError)
                                        ),
                                        text(path.clone()).color(iced::Color::WHITE),
                                    ]
                                    .spacing(12)
                                    .padding(8)
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
                    column![filter_buttons, rule::horizontal(1.0), card_list]
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .into()
            }
            TabId::Timer => {
                use crate::timer::TimerState;
                use crate::gui::components::TimerDisplay;
                
                let timer_display = TimerDisplay::view(&self.timer_tab.state, self.timer_tab.elapsed_ms)
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
                            button(text("暂停").color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                            button(text("停止").color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                        )
                    }
                    TimerState::Running { .. } => {
                        (
                            button(text("开始").color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
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
                            button(text("暂停").color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
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
                            button(text("暂停").color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                            button(text("停止").color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
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
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .into()
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
                            .color(iced::Color::WHITE)
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
                                    let text_color = if todo.completed {
                                        iced::Color::from_rgb(0.5, 0.5, 0.5)
                                    } else {
                                        iced::Color::WHITE
                                    };
                                    
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
                                    
                                    let checkbox = button(text(checkbox_text).color(iced::Color::WHITE))
                                        .on_press(Message::ToggleTodo(todo.id))
                                        .style(|_, _| iced::widget::button::Style {
                                            background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
                                            text_color: iced::Color::WHITE,
                                            ..Default::default()
                                        });
                                    
                                    let delete_btn = button(text("删除").color(iced::Color::from_rgb(0.9, 0.3, 0.2)))
                                        .on_press(Message::DeleteTodo(todo.id))
                                        .style(|_, _| iced::widget::button::Style {
                                            background: Some(iced::Color::from_rgb(0.25, 0.25, 0.25).into()),
                                            text_color: iced::Color::from_rgb(0.9, 0.3, 0.2),
                                            ..Default::default()
                                        });
                                    
                                    let edit_btn = button(text("编辑").color(iced::Color::from_rgb(0.4, 0.7, 0.9)))
                                        .on_press(Message::ModalOpen(Modal::EditTodo { id: todo.id }))
                                        .style(|_, _| iced::widget::button::Style {
                                            background: Some(iced::Color::from_rgb(0.25, 0.25, 0.25).into()),
                                            text_color: iced::Color::from_rgb(0.4, 0.7, 0.9),
                                            ..Default::default()
                                        });
                                    
                                    let content_row = row![
                                        checkbox,
                                        text(priority_badge).color(iced::Color::from_rgb(0.95, 0.61, 0.07)),
                                        text(&todo.content).color(text_color),
                                        text(due_text).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
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
                                        .style(move |_, _| iced::widget::button::Style {
                                            background: if is_selected {
                                                Some(iced::Color::from_rgb(0.25, 0.25, 0.3).into())
                                            } else {
                                                Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into())
                                            },
                                            ..Default::default()
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
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .into()
            }
            TabId::Schedule => {
                let schedule_list = if self.schedule_tab.schedules.is_empty() {
                    container(
                        text("暂无日程安排")
                            .size(16)
                            .color(iced::Color::WHITE)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                } else {
                    container(
                        scrollable(
                            column(
                                self.schedule_tab.schedules.iter().map(|(_id, ics_content)| {
                                    let summary = ics_content
                                        .lines()
                                        .find(|line| line.starts_with("SUMMARY:"))
                                        .map(|line| line.strip_prefix("SUMMARY:").unwrap_or("未知日程"))
                                        .unwrap_or("未知日程");
                                    
                                    let dtstart = ics_content
                                        .lines()
                                        .find(|line| line.starts_with("DTSTART:"))
                                        .and_then(|line| line.strip_prefix("DTSTART:"))
                                        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
                                        .map(|dt| dt.format("%m-%d %H:%M").to_string())
                                        .unwrap_or_else(|| "未知时间".to_string());
                                    
                                    row![
                                        text(dtstart).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                                        text(summary).color(iced::Color::WHITE),
                                    ]
                                    .spacing(12)
                                    .padding(8)
                                    .width(Length::Fill)
                                    .into()
                                })
                            )
                            .spacing(8)
                        )
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                };
                
                container(
                    column![
                        row![
                            container(text("日程").size(20).color(iced::Color::WHITE))
                                .padding(8),
                        ],
                        rule::horizontal(1.0),
                        schedule_list,
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .into()
            }
            TabId::Preset => {
                let preset_list: Vec<_> = self.preset_tab.presets.iter()
                    .map(|preset| {
                        row![
                            text(preset.name.clone()).color(iced::Color::WHITE),
                            text(if preset.fsrs_parameters.as_ref().map_or(true, |p| p.is_empty()) { "默认参数" } else { "已训练" })
                                .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(8)
                        .padding(8)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();
                
                let content = if self.preset_tab.presets.is_empty() {
                    container(text("暂无预设").color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                } else {
                    container(scrollable(column(preset_list).spacing(4)))
                        .width(Length::Fill)
                        .height(Length::Fill)
                };
                
                container(
                    column![
                        row![text("预设").size(20).color(iced::Color::WHITE)].padding(8),
                        rule::horizontal(1.0),
                        content
                    ]
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .into()
            }
            TabId::Settings => {
                container(
                    column![
                        text("设置").size(20).color(iced::Color::WHITE),
                        rule::horizontal(1.0),
                        row![
                            text("数据目录:").color(iced::Color::WHITE),
                            text(self.settings_tab.data_dir.to_string_lossy().to_string())
                                .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(8)
                        .padding(16),
                    ]
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .into()
            }
            _ => {
                container(
                    text(format!("{:?} Tab - Under Construction", self.active_tab))
                        .size(24)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
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
                _ => "确认",
            };
            
            let modal_content = match modal {
                Modal::NewTodo | Modal::EditTodo { .. } => {
                    NewTodoForm::view(
                        &self.todo_tab.form_content,
                        self.todo_tab.form_priority,
                        &self.todo_tab.form_due_year,
                        &self.todo_tab.form_due_month,
                        &self.todo_tab.form_due_day,
                        &self.todo_tab.form_due_hour,
                        &self.todo_tab.form_due_minute,
                        &self.todo_tab.form_tags,
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
                    
                    column![
                        text(format!("确定要删除 \"{}\" 吗？", item))
                            .color(iced::Color::WHITE)
                            .size(16),
                        Space::new().height(12),
                        text(warning)
                            .color(iced::Color::from_rgb(0.9, 0.5, 0.3))
                            .size(14),
                    ]
                    .spacing(12)
                    .into()
                }
                Modal::Error { message } => {
                    column![
                        text("错误")
                            .color(iced::Color::WHITE)
                            .size(16),
                        text(message.clone())
                            .color(iced::Color::from_rgb(0.9, 0.3, 0.2)),
                    ]
                    .spacing(12)
                    .into()
                }
                Modal::NewNode => {
                    if let Some(ref form) = self.category_tab.new_node_form {
                        let presets: Vec<String> = self.preset_tab.presets.iter()
                            .map(|p| p.name.clone())
                            .collect();
                        NewNodeModal::view(form, &presets)
                    } else {
                        text("开发中").color(iced::Color::WHITE).into()
                    }
                }
                Modal::EditCard { .. } => {
                    if let Some(ref form) = self.category_tab.edit_card_form {
                        let presets: Vec<String> = self.preset_tab.presets.iter()
                            .map(|p| p.name.clone())
                            .collect();
                        simple_edit_card_view(form, &presets)
                    } else {
                        text("开发中").color(iced::Color::WHITE).into()
                    }
                }
                _ => text("开发中").color(iced::Color::WHITE).into()
            };
            
            ModalView::view(
                modal_title,
                modal_content,
                Message::ModalConfirm,
                Message::ModalClose,
            )
        } else {
            base_view.into()
        }
    }
}

fn simple_edit_card_view(form: &category_tab::EditCardForm, presets: &[String]) -> Element<'static, Message> {
    use crate::gui::components::NewNodeModal;
    
    let presets_owned = presets.to_vec();
    let new_name = form.new_name.clone();
    let preset = form.preset.clone();
    
    let next_review_str = form.next_review
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    
    let review_list: Element<Message> = if form.review_records.is_empty() {
        text("暂无复习记录").color(iced::Color::from_rgb(0.6, 0.6, 0.6)).into()
    } else {
        column(
            form.review_records.iter().enumerate().map(|(i, r)| {
                let timestamp = r.timestamp.format("%Y-%m-%d").to_string();
                let duration = r.duration_ms / 60000;
                let quality = r.memory_quality.as_str();
                
                row![
                    text(timestamp).color(iced::Color::WHITE).width(Length::Fixed(100.0)),
                    text(format!("{}分", duration)).color(iced::Color::WHITE).width(Length::Fixed(60.0)),
                    text(quality).color(iced::Color::WHITE).width(Length::Fixed(60.0)),
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
        text("名称:").color(iced::Color::WHITE),
        text_input("名称", &new_name)
            .on_input(Message::EditCardNameChanged)
            .width(Length::Fill),
        Space::new().height(12),
        
        text("预设:").color(iced::Color::WHITE),
        pick_list(presets_owned, Some(preset), Message::EditCardPresetChanged)
            .width(Length::Fill),
        Space::new().height(12),
        
        text("下次复习 (YYYY-MM-DD):").color(iced::Color::WHITE),
        row![
            text_input("下次复习", &next_review_str)
                .on_input(Message::EditCardNextReviewChanged)
                .width(Length::Fixed(150.0)),
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
        ]
        .spacing(8),
        Space::new().height(12),
        
        text(format!("复习记录 ({} 条)", form.review_records.len())).color(iced::Color::WHITE),
        review_list,
        Space::new().height(24),
        
        row![
            button(text("取消").color(iced::Color::WHITE))
                .on_press(Message::ModalClose)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.4, 0.4, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text("保存").color(iced::Color::WHITE))
                .on_press(Message::EditCardConfirm)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8)
        .width(Length::Fill)
        .push(Space::new().width(Length::Fill)),
    ]
    .padding(16)
    .spacing(8)
    .into()
}

fn card_action_buttons(path: &str) -> Element<Message> {
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

fn folder_action_buttons(path: &str) -> Element<Message> {
    let path = path.to_string();
    row![
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
        button(
            text("删除").color(iced::Color::WHITE)
        )
        .on_press(Message::DeleteNodeOpen(path.clone(), true))
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

fn tab_button(label: &str, tab_id: TabId, active_tab: TabId) -> Element<Message> {
    let is_active = tab_id == active_tab;
    
    let btn = button(text(label).color(iced::Color::WHITE))
        .on_press(Message::SwitchTab(tab_id));
    
    if is_active {
        btn.style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        })
    } else {
        btn.style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        })
    }
    .into()
}

fn filter_button(label: &str, filter: Option<u32>, current: Option<u32>) -> Element<Message> {
    let is_active = filter == current;
    
    let btn = button(text(label).color(iced::Color::WHITE))
        .on_press(Message::UrgencyFilterChanged(filter));
    
    if is_active {
        btn.style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        })
    } else {
        btn.style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        })
    }
    .into()
}