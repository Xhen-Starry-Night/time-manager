use chrono::Utc;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Fill, Task};
use tracing::{debug, error, info, trace, warn};

use crate::data::models::{CategoryInsert, Difficulty, NodeType, Quality, SessionParams};
use crate::data::Database;
use crate::modules::learning::category::obsidian::vault_parser;
use crate::modules::learning::category::tree::{CategoryForest, CategoryNode};
use crate::modules::learning::prediction::algorithm::PredictionAlgorithm;
use crate::modules::learning::prediction::fsrs::adapter::FsrsAdapter;
use crate::modules::learning::timer::params::{apply_preset, resolve_params};
use crate::modules::learning::timer::state_machine::TimerStateMachine;
use crate::settings::defaults;

const FONT_SM: f32 = 24.0;
const FONT_MD: f32 = 28.0;
const FONT_LG: f32 = 36.0;
const FONT_XL: f32 = 48.0;

#[derive(Debug, Clone)]
pub enum Screen {
    CategoryTree,
    ReviewDashboard,
}

#[derive(Debug, Clone)]
pub enum Dialog {
    NewCategory {
        parent_id: Option<i64>,
        name: String,
        node_type: NodeType,
    },
    ObsidianImport {
        path: String,
    },
    Search {
        query: String,
    },
    ConfirmDeleteCategory {
        id: i64,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchScreen(Screen),
    SelectCategory(i64),
    OpenNewCategory,
    OpenSearch,
    OpenImport,
    OpenSettings,
    SetNewCategoryName(String),
    SetNewCategoryType(NodeType),
    CreateCategory,
    SetObsidianPath(String),
    ImportObsidian,
    SetSearchQuery(String),
    DismissDialog,
    DeleteCategory(i64),
    ConfirmDeleteCategory,
    StartTimer(i64),
    PauseTimer,
    ResumeTimer,
    StopTimer,
    ConfirmSession,
    CancelSession,
    SetQuality(Quality),
    SetUnderstandingDifficulty(Difficulty),
    SetMemoryDifficulty(Difficulty),
    SetCompletionRate(u8),
    ApplyPreset(usize),
    RefreshReviewDashboard,
    Tick,
    BackFromTimer,
}

#[derive(Debug, Clone)]
enum Urgency {
    Overdue,
    Within3Days,
    Within7Days,
    Normal,
}

pub struct App {
    db: Database,
    forest: CategoryForest,
    screen: Screen,
    selected_category_id: Option<i64>,
    timer: TimerStateMachine,
    timer_display_secs: i64,
    global_params: SessionParams,
    current_params: SessionParams,
    pending_session: Option<PendingSession>,
    fsrs: FsrsAdapter,
    dialog: Option<Dialog>,
    return_screen: Screen,
}

pub struct PendingSession {
    pub category_id: i64,
    pub category_path: String,
    pub duration_secs: i64,
    pub params: SessionParams,
}

impl App {
    pub fn new() -> Self {
        info!("Initializing Time Manager App");

        let db = Database::open_default().expect("failed to open database");
        debug!("Database opened successfully");

        let cats = db.get_all_categories().unwrap_or_default();
        trace!("Loaded {} categories", cats.len());

        let forest = CategoryForest::from_categories(&cats);
        debug!("Category forest built with {} root nodes", forest.roots.len());

        let global_params = defaults::load_global_params(&db);
        trace!("Global params loaded");

        let fsrs = FsrsAdapter::new();
        debug!("FSRS adapter initialized");

        info!("App initialization complete");

        Self {
            db,
            forest,
            screen: Screen::CategoryTree,
            selected_category_id: None,
            timer: TimerStateMachine::new(),
            timer_display_secs: 0,
            global_params,
            current_params: SessionParams::default(),
            pending_session: None,
            fsrs,
            dialog: None,
            return_screen: Screen::CategoryTree,
        }
    }

    fn refresh_forest(&mut self) {
        debug!("Refreshing category forest");
        let cats = self.db.get_all_categories().unwrap_or_default();
        trace!("Loaded {} categories for refresh", cats.len());
        self.forest = CategoryForest::from_categories(&cats);
        debug!("Forest refreshed with {} roots", self.forest.roots.len());
    }

    fn is_timer_active(&self) -> bool {
        self.timer.is_running()
            || matches!(
                &self.timer.state,
                crate::modules::learning::timer::state_machine::TimerState::Paused { .. }
            )
    }

    fn urgency_for(&self, category_id: i64) -> Urgency {
        if let Ok(Some(pred)) = self.db.get_prediction_state(category_id) {
            let now = Utc::now();
            if pred.next_review <= now {
                return Urgency::Overdue;
            }
            let days = (pred.next_review - now).num_seconds() as f64 / 86400.0;
            if days <= 3.0 {
                return Urgency::Within3Days;
            }
            if days <= 7.0 {
                return Urgency::Within7Days;
            }
        }
        Urgency::Normal
    }

    fn urgency_sort_key(urgency: &Urgency) -> u8 {
        match urgency {
            Urgency::Overdue => 0,
            Urgency::Within3Days => 1,
            Urgency::Within7Days => 2,
            Urgency::Normal => 3,
        }
    }

    fn urgency_color(urgency: &Urgency) -> Color {
        match urgency {
            Urgency::Overdue => Color::from_rgb8(220, 50, 50),
            Urgency::Within3Days => Color::from_rgb8(210, 170, 30),
            Urgency::Within7Days => Color::from_rgb8(60, 170, 80),
            Urgency::Normal => Color::WHITE,
        }
    }

    fn urgency_label(urgency: &Urgency, pred: &Option<crate::data::models::PredictionState>) -> String {
        match urgency {
            Urgency::Overdue => {
                if let Some(p) = pred {
                    Self::format_time_until(&p.next_review)
                } else {
                    "已过期".into()
                }
            }
            Urgency::Within3Days => {
                if let Some(p) = pred {
                    Self::format_time_until(&p.next_review)
                } else {
                    "临近到期".into()
                }
            }
            Urgency::Within7Days => {
                if let Some(p) = pred {
                    Self::format_time_until(&p.next_review)
                } else {
                    "本周内".into()
                }
            }
            Urgency::Normal => {
                if let Some(p) = pred {
                    Self::format_time_until(&p.next_review)
                } else {
                    "未复习".into()
                }
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        trace!("Processing message: {:?}", std::mem::discriminant(&message));

        match message {
            Message::SwitchScreen(screen) => {
                info!("Switching screen to: {:?}", screen);
                self.selected_category_id = None;
                self.screen = screen;
            }
            Message::SelectCategory(id) => {
                debug!("Selecting category: {}", id);
                self.selected_category_id = Some(id);
            }

            Message::OpenNewCategory => {
                debug!("Opening new category dialog, parent: {:?}", self.selected_category_id);
                self.dialog = Some(Dialog::NewCategory {
                    parent_id: self.selected_category_id,
                    name: String::new(),
                    node_type: NodeType::Directory,
                });
            }
            Message::OpenSearch => {
                debug!("Opening search dialog");
                self.dialog = Some(Dialog::Search {
                    query: String::new(),
                });
            }
            Message::OpenImport => {
                debug!("Opening Obsidian import dialog");
                self.dialog = Some(Dialog::ObsidianImport {
                    path: String::new(),
                });
            }
            Message::OpenSettings => {
                trace!("Settings opened (not implemented)");
            }

            Message::SetNewCategoryName(name) => {
                if let Some(Dialog::NewCategory { name: ref mut n, .. }) = self.dialog {
                    *n = name;
                }
            }
            Message::SetNewCategoryType(nt) => {
                if let Some(Dialog::NewCategory { node_type: ref mut t, .. }) = self.dialog {
                    *t = nt;
                }
            }
            Message::SetObsidianPath(path) => {
                if let Some(Dialog::ObsidianImport { path: ref mut p }) = self.dialog {
                    *p = path;
                }
            }
            Message::SetSearchQuery(query) => {
                if let Some(Dialog::Search { query: ref mut q }) = self.dialog {
                    *q = query;
                }
            }

            Message::CreateCategory => {
                if let Some(Dialog::NewCategory { parent_id, name, node_type }) = self.dialog.take() {
                    if !name.trim().is_empty() {
                        let path = if let Some(pid) = parent_id {
                            let parent_path = self
                                .forest
                                .find(pid)
                                .map(|n| n.path.clone())
                                .unwrap_or_default();
                            format!("{}/{}", parent_path, name.trim())
                        } else {
                            name.trim().to_string()
                        };

                        info!("Creating category: '{}' at path '{}' (type: {:?})", name.trim(), path, node_type);

                        let insert = CategoryInsert {
                            parent_id,
                            name: name.trim().to_string(),
                            path,
                            node_type,
                            source: None,
                            default_quality: None,
                            default_understanding_difficulty: None,
                            default_memory_difficulty: None,
                        };

                        if self.db.insert_category(&insert).is_ok() {
                            debug!("Category created successfully");
                            self.refresh_forest();
                        } else {
                            error!("Failed to create category");
                        }
                    } else {
                        warn!("Attempted to create category with empty name");
                    }
                }
            }

            Message::ImportObsidian => {
                if let Some(Dialog::ObsidianImport { path }) = self.dialog.take() {
                    if !path.trim().is_empty() {
                        info!("Starting Obsidian import from: {}", path);

                        let vault_path = std::path::Path::new(&path);
                        let entries = vault_parser::parse_vault(vault_path);
                        debug!("Found {} entries in vault", entries.len());

                        let mut dir_count = 0;
                        let mut file_count = 0;

                        for entry in &entries {
                            if entry.is_dir {
                                let name = entry.name.clone();
                                let cat_path = entry.relative_path.clone();
                                if self.forest.find_by_path(&cat_path).is_none() {
                                    let parent_id = if let Some(pos) = cat_path.rfind('/') {
                                        let parent_path = &cat_path[..pos];
                                        self.forest.find_by_path(parent_path).map(|n| n.id)
                                    } else {
                                        None
                                    };

                                    let insert = CategoryInsert {
                                        parent_id,
                                        name,
                                        path: cat_path,
                                        node_type: NodeType::Directory,
                                        source: Some("obsidian".into()),
                                        default_quality: None,
                                        default_understanding_difficulty: None,
                                        default_memory_difficulty: None,
                                    };
                                    if self.db.insert_category(&insert).is_ok() {
                                        dir_count += 1;
                                        trace!("Imported directory: {}", entry.relative_path);
                                    }
                                }
                            }
                        }

                        for entry in &entries {
                            if !entry.is_dir {
                                let name = entry.name.clone();
                                let cat_path = entry.relative_path.clone();
                                if self.forest.find_by_path(&cat_path).is_none() {
                                    let parent_id = if let Some(pos) = cat_path.rfind('/') {
                                        let parent_path = &cat_path[..pos];
                                        self.forest.find_by_path(parent_path).map(|n| n.id)
                                    } else {
                                        None
                                    };

                                    let insert = CategoryInsert {
                                        parent_id,
                                        name,
                                        path: cat_path,
                                        node_type: NodeType::Learning,
                                        source: Some("obsidian".into()),
                                        default_quality: None,
                                        default_understanding_difficulty: None,
                                        default_memory_difficulty: None,
                                    };
                                    if self.db.insert_category(&insert).is_ok() {
                                        file_count += 1;
                                        trace!("Imported learning node: {}", entry.relative_path);
                                    }
                                }
                            }
                        }

                        info!("Obsidian import complete: {} directories, {} learning nodes", dir_count, file_count);
                        self.refresh_forest();
                    }
                }
            }

            Message::DismissDialog => {
                debug!("Dismissing dialog");
                self.dialog = None;
            }

            Message::DeleteCategory(id) => {
                if let Some(node) = self.forest.find(id) {
                    debug!("Requesting delete confirmation for category: {} (id: {})", node.name, id);
                    self.dialog = Some(Dialog::ConfirmDeleteCategory {
                        id,
                        name: node.name.clone(),
                    });
                }
            }
            Message::ConfirmDeleteCategory => {
                if let Some(Dialog::ConfirmDeleteCategory { id, .. }) = self.dialog.take() {
                    info!("Deleting category id: {}", id);
                    if self.db.delete_category(id).is_ok() {
                        debug!("Category deleted successfully");
                        self.refresh_forest();
                        if self.selected_category_id == Some(id) {
                            self.selected_category_id = None;
                        }
                    } else {
                        error!("Failed to delete category {}", id);
                    }
                }
            }

            Message::StartTimer(category_id) => {
                info!("Starting timer for category: {}", category_id);

                let node = self.forest.find(category_id);
                let params = resolve_params(
                    &self.global_params,
                    node.and_then(|n| n.default_quality.as_ref()),
                    node.and_then(|n| n.default_understanding_difficulty.as_ref()),
                    node.and_then(|n| n.default_memory_difficulty.as_ref()),
                );

                debug!("Timer params resolved: quality={:?}, ud={:?}, md={:?}",
                    params.quality, params.understanding_difficulty, params.memory_difficulty);

                self.current_params = params;
                self.timer.start(category_id).ok();
                self.return_screen = self.screen.clone();
                debug!("Timer started, return screen: {:?}", self.return_screen);
            }
            Message::PauseTimer => {
                debug!("Pausing timer");
                self.timer.pause().ok();
            }
            Message::ResumeTimer => {
                debug!("Resuming timer");
                self.timer.resume().ok();
            }
            Message::StopTimer => {
                if let Ok(stopped) = self.timer.stop() {
                    info!("Timer stopped: category={}, duration={}s",
                        stopped.category_id, stopped.duration_secs);

                    let path = self
                        .forest
                        .find(stopped.category_id)
                        .map(|n| n.path.clone())
                        .unwrap_or_default();
                    self.pending_session = Some(PendingSession {
                        category_id: stopped.category_id,
                        category_path: path,
                        duration_secs: stopped.duration_secs,
                        params: self.current_params.clone(),
                    });
                    debug!("Pending session created, awaiting confirmation");
                }
            }
            Message::ConfirmSession => {
                if let Some(pending) = self.pending_session.take() {
                    info!("Confirming session: category={}, duration={}s, quality={:?}",
                        pending.category_id, pending.duration_secs, pending.params.quality);

                    let now = Utc::now();
                    let insert = crate::data::models::SessionInsert {
                        category_id: pending.category_id,
                        start_time: now - chrono::Duration::seconds(pending.duration_secs),
                        end_time: now,
                        duration_secs: pending.duration_secs,
                        pause_records: vec![],
                        params: pending.params.clone(),
                        note: None,
                    };
                    if self.db.insert_session(&insert).is_ok() {
                        debug!("Session saved to database");

                        let pred = self
                            .fsrs
                            .predict_next_review(&[], &[], &pending.params.quality);
                        debug!("FSRS prediction: next_review={}", pred.next_review.format("%Y-%m-%d"));

                        self.db
                            .upsert_prediction_state(&crate::data::models::PredictionStateInsert {
                                category_id: pending.category_id,
                                algorithm: self.fsrs.algorithm_name().into(),
                                last_review: now,
                                next_review: pred.next_review,
                                algorithm_state: pred.state_bytes,
                            })
                            .ok();

                        info!("Session complete and prediction updated");
                    } else {
                        error!("Failed to save session");
                    }
                }
                self.screen = self.return_screen.clone();
                self.selected_category_id = None;
            }
            Message::CancelSession => {
                info!("Session cancelled");
                self.pending_session = None;
            }

            Message::SetQuality(q) => {
                trace!("Setting quality: {:?}", q);
                if let Some(p) = &mut self.pending_session {
                    p.params.quality = q;
                } else {
                    self.current_params.quality = q;
                }
            }
            Message::SetUnderstandingDifficulty(d) => {
                trace!("Setting understanding difficulty: {:?}", d);
                if let Some(p) = &mut self.pending_session {
                    p.params.understanding_difficulty = d;
                } else {
                    self.current_params.understanding_difficulty = d;
                }
            }
            Message::SetMemoryDifficulty(d) => {
                trace!("Setting memory difficulty: {:?}", d);
                if let Some(p) = &mut self.pending_session {
                    p.params.memory_difficulty = d;
                } else {
                    self.current_params.memory_difficulty = d;
                }
            }
            Message::SetCompletionRate(r) => {
                trace!("Setting completion rate: {}%", r);
                if let Some(p) = &mut self.pending_session {
                    p.params.completion_rate = r;
                } else {
                    self.current_params.completion_rate = r;
                }
            }
            Message::ApplyPreset(idx) => {
                debug!("Applying preset index: {}", idx);
                let presets = defaults::load_presets(&self.db);
                if let Some(preset) = presets.get(idx) {
                    if let Some(p) = &mut self.pending_session {
                        apply_preset(preset, &mut p.params);
                    } else {
                        apply_preset(preset, &mut self.current_params);
                    }
                    trace!("Preset '{}' applied", preset.name);
                }
            }

            Message::RefreshReviewDashboard => {
                trace!("Review dashboard refresh requested");
            }
            Message::Tick => {
                if self.timer.is_running() {
                    self.timer_display_secs = self.timer.effective_secs();
                }
            }
            Message::BackFromTimer => {
                debug!("Returning from timer to previous screen");
                self.screen = self.return_screen.clone();
                self.selected_category_id = None;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.pending_session.is_some() {
            return self.view_session_confirm();
        }

        if self.is_timer_active() {
            return self.view_timer();
        }

        let menu_bar = self.view_menu_bar();

        let content = if let Some(ref dialog) = self.dialog {
            match dialog {
                Dialog::NewCategory { name, node_type, .. } => self.view_dialog_new_category(name, node_type),
                Dialog::ObsidianImport { path } => self.view_dialog_obsidian_import(path),
                Dialog::Search { query } => self.view_dialog_search(query),
                Dialog::ConfirmDeleteCategory { name, .. } => self.view_dialog_confirm_delete(name),
            }
        } else {
            match &self.screen {
                Screen::CategoryTree => {
                    if self.forest.roots.is_empty() {
                        self.view_welcome()
                    } else {
                        self.view_category_tree()
                    }
                }
                Screen::ReviewDashboard => self.view_review_dashboard(),
            }
        };

        column![menu_bar, content]
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.timer.is_running() {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    // ── Menu bar ──────────────────────────────────────────────────

    fn nav_btn<'a>(label: &'a str, screen: Screen, current: &Screen) -> button::Button<'a, Message> {
        let b = button(text(label).size(FONT_SM));
        if std::mem::discriminant(&screen) == std::mem::discriminant(current) {
            b.style(button::primary)
        } else {
            b.style(button::text)
        }
        .on_press(Message::SwitchScreen(screen))
    }

    fn action_btn<'a>(label: &'a str, msg: Message) -> button::Button<'a, Message> {
        button(text(label).size(FONT_SM))
            .style(button::text)
            .on_press(msg)
    }

    fn view_menu_bar(&self) -> Element<'_, Message> {
        let nav = row![
            Self::nav_btn("分类树", Screen::CategoryTree, &self.screen),
            Self::nav_btn("复习看板", Screen::ReviewDashboard, &self.screen),
        ]
        .spacing(8);

        let actions: Element<'_, Message> = match &self.screen {
            Screen::CategoryTree => row![
                Self::action_btn("搜索", Message::OpenSearch),
                Self::action_btn("导入", Message::OpenImport),
                Self::action_btn("新建", Message::OpenNewCategory),
                Self::action_btn("设置", Message::OpenSettings),
            ]
            .spacing(6)
            .into(),
            Screen::ReviewDashboard => row![Self::action_btn("刷新", Message::RefreshReviewDashboard)]
                .spacing(6)
                .into(),
        };

        row![nav, actions]
            .spacing(20)
            .padding([6, 12])
            .width(Fill)
            .into()
    }

    // ── Welcome / onboarding ──────────────────────────────────────

    fn view_welcome(&self) -> Element<'_, Message> {
        let inner = column![
            text("欢迎使用 Time Manager").size(FONT_LG),
            text("开始跟踪你的学习时间，建立复习节奏。").size(FONT_SM),
            row![
                button(text("创建第一个分类").size(FONT_SM))
                    .on_press(Message::OpenNewCategory)
                    .style(button::primary)
                    .padding([8, 16]),
                button(text("从 Obsidian 导入").size(FONT_SM))
                    .on_press(Message::OpenImport)
                    .style(button::secondary)
                    .padding([8, 16]),
            ]
            .spacing(15),
            text("你也可以稍后在菜单栏中操作。").size(FONT_SM),
        ]
        .spacing(20)
        .align_x(iced::Center);

        container(inner)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }

    // ── Dialogs ───────────────────────────────────────────────────

    fn view_dialog_new_category(&self, name: &str, node_type: &NodeType) -> Element<'_, Message> {
        let parent_label = if let Some(id) = self.selected_category_id {
            self.forest
                .find(id)
                .map(|n| format!("父节点: {}", n.path))
                .unwrap_or_else(|| "根节点".into())
        } else {
            "根节点".into()
        };

        let is_dir = matches!(node_type, NodeType::Directory);
        let dir_btn = if is_dir {
            button(text("分类目录").size(FONT_SM)).style(button::primary)
        } else {
            button(text("分类目录").size(FONT_SM)).style(button::text)
        }
        .on_press(Message::SetNewCategoryType(NodeType::Directory));

        let learn_btn = if !is_dir {
            button(text("学习节点").size(FONT_SM)).style(button::primary)
        } else {
            button(text("学习节点").size(FONT_SM)).style(button::text)
        }
        .on_press(Message::SetNewCategoryType(NodeType::Learning));

        column![
            text("新建分类").size(FONT_MD),
            text(parent_label).size(FONT_SM),
            text("节点类型:").size(FONT_SM),
            row![dir_btn, learn_btn].spacing(10),
            text_input("输入分类名称...", name)
                .on_input(Message::SetNewCategoryName)
                .padding(8)
                .size(FONT_SM),
            row![
                button(text("取消").size(FONT_SM))
                    .on_press(Message::DismissDialog)
                    .style(button::text),
                button(text("创建").size(FONT_SM))
                    .on_press(Message::CreateCategory)
                    .style(button::primary),
            ]
            .spacing(15),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }

    fn view_dialog_obsidian_import(&self, path: &str) -> Element<'_, Message> {
        column![
            text("从 Obsidian 导入").size(FONT_MD),
            text("指定 Obsidian 仓库根目录路径：").size(FONT_SM),
            text_input("/path/to/vault", path)
                .on_input(Message::SetObsidianPath)
                .padding(8)
                .size(FONT_SM),
            row![
                button(text("取消").size(FONT_SM))
                    .on_press(Message::DismissDialog)
                    .style(button::text),
                button(text("导入").size(FONT_SM))
                    .on_press(Message::ImportObsidian)
                    .style(button::primary),
            ]
            .spacing(15),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }

    fn view_dialog_search(&self, query: &str) -> Element<'_, Message> {
        let input = text_input("搜索分类...", query)
            .on_input(Message::SetSearchQuery)
            .padding(8)
            .size(FONT_SM);

        let mut results = column![].spacing(4);
        if !query.is_empty() {
            let q = query.to_lowercase();
            for node in self.forest.all_nodes() {
                if node.name.to_lowercase().contains(&q)
                    || node.path.to_lowercase().contains(&q)
                {
                    results = results.push(
                        button(text(node.path.clone()).size(FONT_SM))
                            .on_press(Message::SelectCategory(node.id))
                            .style(button::text)
                            .padding([2, 6]),
                    );
                }
            }
        }

        column![
            text("搜索分类").size(FONT_MD),
            input,
            scrollable(results).height(200),
            button(text("关闭").size(FONT_SM))
                .on_press(Message::DismissDialog)
                .style(button::text),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }

    fn view_dialog_confirm_delete(&self, name: &str) -> Element<'_, Message> {
        column![
            text("确认删除").size(FONT_MD),
            text(format!("确定要删除「{}」及其所有子节点吗？", name)).size(FONT_SM),
            row![
                button(text("取消").size(FONT_SM))
                    .on_press(Message::DismissDialog)
                    .style(button::text),
                button(text("删除").size(FONT_SM))
                    .on_press(Message::ConfirmDeleteCategory)
                    .style(button::danger),
            ]
            .spacing(15),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }

    // ── Category tree with detail panel ───────────────────────────

    fn view_category_tree(&self) -> Element<'_, Message> {
        let mut tree_col = column![].spacing(2).padding(10);

        for root in &self.forest.roots {
            tree_col = tree_col.push(Self::view_tree_node(root, self.selected_category_id));
        }

        let left = scrollable(tree_col).width(Fill).height(Fill);
        let right = self.view_category_detail();

        row![left, right,]
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn view_tree_node(node: &CategoryNode, selected_id: Option<i64>) -> Element<'_, Message> {
        let indent = node.path.matches('/').count();
        let prefix = "  ".repeat(indent);
        let icon = match &node.node_type {
            NodeType::Directory if node.children.is_empty() => "📁",
            NodeType::Directory => "📁",
            NodeType::Learning => "📖",
        };

        let is_selected = selected_id == Some(node.id);
        let btn = button(text(format!("{}{} {}", prefix, icon, node.name)).size(FONT_SM))
            .on_press(Message::SelectCategory(node.id))
            .padding([2, 5])
            .width(Fill);

        let btn = if is_selected {
            btn.style(button::primary)
        } else {
            btn.style(button::text)
        };

        let mut col = column![btn];

        for child in &node.children {
            col = col.push(Self::view_tree_node(child, selected_id));
        }

        col.into()
    }

    fn view_category_detail(&self) -> Element<'_, Message> {
        let Some(id) = self.selected_category_id else {
            return container(text("选择一个节点查看详情").size(FONT_SM))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .into();
        };

        let Some(node) = self.forest.find(id) else {
            return container(text("节点未找到").size(FONT_SM))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .into();
        };

        let path_display = node.path.replace("/", " > ");

        let mut detail = column![
            text(&node.name).size(FONT_LG),
            text(format!("路径: {}", path_display)).size(FONT_SM),
            text("───────────────────").size(FONT_SM),
        ]
        .spacing(8)
        .padding(15)
        .width(Fill);

        if let Some(source) = &node.source {
            detail = detail.push(text(format!("来源: {}", source)).size(FONT_SM));
        }

        let sessions = self.db.get_sessions_by_category(id).unwrap_or_default();
        let total_secs: i64 = sessions.iter().map(|s| s.duration_secs).sum();
        let session_count = sessions.len();

        detail = detail.push(text("───────────────────").size(FONT_SM));
        detail = detail.push(text("学习统计").size(FONT_MD));
        detail = detail.push(text(format!("会话次数: {}", session_count)).size(FONT_SM));
        detail = detail.push(text(format!("总学习时长: {}", Self::format_duration(total_secs))).size(FONT_SM));

        if let Some(pred) = self.db.get_prediction_state(id).unwrap_or(None) {
            let label = format!("下次复习: {}", Self::format_time_until(&pred.next_review));
            detail = detail.push(text(label).size(FONT_SM));
        }

        detail = detail.push(text("───────────────────").size(FONT_SM));
        detail = detail.push(
            button(text("开始计时").size(FONT_SM))
                .on_press(Message::StartTimer(id))
                .style(button::primary)
                .padding([8, 16]),
        );
        detail = detail.push(
            button(text("删除节点").size(FONT_SM))
                .on_press(Message::DeleteCategory(id))
                .style(button::danger)
                .padding([8, 16]),
        );

        scrollable(detail).width(Fill).height(Fill).into()
    }

    // ── Review dashboard: all leaf nodes sorted by urgency ─────────

    fn view_review_dashboard(&self) -> Element<'_, Message> {
        let learning = self.forest.learning_nodes();

        if learning.is_empty() {
            return container(
                if self.forest.roots.is_empty() {
                    text("暂无分类。请先在分类树中添加学习内容。").size(FONT_SM)
                } else {
                    text("暂无学习节点。请在分类树中添加学习节点。").size(FONT_SM)
                },
            )
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }

        let mut items: Vec<(i64, String, Urgency)> = learning
            .iter()
            .map(|n| (n.id, n.name.clone(), self.urgency_for(n.id)))
            .collect();

        items.sort_by_key(|(_, _, u)| Self::urgency_sort_key(u));

        let mut list_col = column![text("复习看板").size(FONT_MD)].spacing(4).padding(10);

        for (cat_id, cat_name, urgency) in &items {
            let pred = self.db.get_prediction_state(*cat_id).unwrap_or(None);
            let label = Self::urgency_label(urgency, &pred);
            let color = Self::urgency_color(urgency);
            let marker = match urgency {
                Urgency::Overdue => "🔴",
                Urgency::Within3Days => "🟡",
                Urgency::Within7Days => "🟢",
                Urgency::Normal => "⚪",
            };

            let is_selected = self.selected_category_id == Some(*cat_id);
            let btn_style = if is_selected {
                button::primary
            } else {
                button::text
            };

            list_col = list_col.push(
                button(
                    row![
                        text(format!("{} {}", marker, cat_name))
                            .size(FONT_SM)
                            .width(Fill)
                            .color(color),
                        text(label).size(FONT_SM).color(color),
                    ]
                    .spacing(6),
                )
                .on_press(Message::SelectCategory(*cat_id))
                .style(btn_style)
                .padding([4, 8])
                .width(Fill),
            );
        }

        let left = scrollable(list_col).width(Fill).height(Fill);
        let right = self.view_review_detail();

        row![left, right,]
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn view_review_detail(&self) -> Element<'_, Message> {
        let Some(id) = self.selected_category_id else {
            return container(text("选择一个节点查看详情").size(FONT_SM))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .into();
        };

        let Some(node) = self.forest.find(id) else {
            return container(text("节点未找到").size(FONT_SM))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .into();
        };

        let path_display = node.path.replace("/", " > ");

        let mut detail = column![
            text(&node.name).size(FONT_LG),
            text(format!("路径: {}", path_display)).size(FONT_SM),
            text("───────────────────").size(FONT_SM),
        ]
        .spacing(8)
        .padding(15)
        .width(Fill);

        if let Some(pred) = self.db.get_prediction_state(id).unwrap_or(None) {
            detail = detail.push(text("复习状态").size(FONT_MD));
            detail = detail.push(
                text(format!(
                    "上次复习: {}",
                    pred.last_review.format("%Y-%m-%d")
                ))
                .size(FONT_SM),
            );

            let next_label = format!("下次复习: {}", Self::format_time_until(&pred.next_review));
            detail = detail.push(text(next_label).size(FONT_SM));
        } else {
            detail = detail.push(text("复习状态: 尚未学习").size(FONT_SM));
        }

        let sessions = self.db.get_sessions_by_category(id).unwrap_or_default();
        let total_secs: i64 = sessions.iter().map(|s| s.duration_secs).sum();
        let session_count = sessions.len();

        detail = detail.push(text("───────────────────").size(FONT_SM));
        detail = detail.push(text("学习统计").size(FONT_MD));
        detail = detail.push(text(format!("会话次数: {}", session_count)).size(FONT_SM));
        detail = detail.push(text(format!("总学习时长: {}", Self::format_duration(total_secs))).size(FONT_SM));

        if session_count > 0 {
            let avg_secs = total_secs / session_count as i64;
            detail = detail.push(text(format!("平均时长: {}", Self::format_duration(avg_secs))).size(FONT_SM));
        }

        detail = detail.push(text("───────────────────").size(FONT_SM));
        detail = detail.push(
            button(text("开始计时").size(FONT_SM))
                .on_press(Message::StartTimer(id))
                .style(button::primary)
                .padding([8, 16]),
        );

        scrollable(detail).width(Fill).height(Fill).into()
    }

    // ── Timer ─────────────────────────────────────────────────────

    fn format_duration(secs: i64) -> String {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    fn format_time_until(next: &chrono::DateTime<Utc>) -> String {
        let now = Utc::now();
        if next <= &now {
            let overdue_secs = (now - *next).num_seconds();
            let h = overdue_secs / 3600;
            if h >= 24 {
                format!("已过期 {} 天", h / 24)
            } else {
                format!("已过期 {} 小时", h)
            }
        } else {
            let secs = (*next - now).num_seconds();
            let h = secs / 3600;
            if h >= 24 {
                let days = h / 24;
                let remain_h = h % 24;
                if remain_h > 0 {
                    format!("{} 天 {} 小时后", days, remain_h)
                } else {
                    format!("{} 天后", days)
                }
            } else if h > 0 {
                format!("{} 小时后", h)
            } else {
                let m = secs / 60;
                if m > 0 {
                    format!("{} 分钟后", m)
                } else {
                    "即将到期".into()
                }
            }
        }
    }

    fn view_timer(&self) -> Element<'_, Message> {
        let category_name = self
            .timer
            .category_id
            .and_then(|id| self.forest.find(id))
            .map(|n| n.path.clone())
            .unwrap_or_else(|| "未选择".into());

        let duration = Self::format_duration(self.timer_display_secs);
        let is_paused = matches!(
            &self.timer.state,
            crate::modules::learning::timer::state_machine::TimerState::Paused { .. }
        );
        let is_running = self.timer.is_running();

        let pause_resume = if is_paused {
            button(text("继续").size(FONT_SM))
                .on_press(Message::ResumeTimer)
                .style(button::primary)
        } else {
            button(text("暂停").size(FONT_SM))
                .on_press(Message::PauseTimer)
                .style(button::secondary)
        };

        let stop = if is_running || is_paused {
            button(text("结束").size(FONT_SM))
                .on_press(Message::StopTimer)
                .style(button::danger)
        } else {
            button(text("结束").size(FONT_SM)).style(button::text)
        };

        container(
            column![
                text(format!("当前节点: {}", category_name)).size(FONT_MD),
                text(duration).size(FONT_XL),
                text("有效时长").size(FONT_SM),
                row![pause_resume, stop].spacing(20),
                button(text("返回").size(FONT_SM))
                    .on_press(Message::BackFromTimer)
                    .style(button::text),
            ]
            .spacing(15)
            .align_x(iced::Center),
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    // ── Session confirm ───────────────────────────────────────────

    fn view_session_confirm(&self) -> Element<'_, Message> {
        let pending = self.pending_session.as_ref().unwrap();
        let duration = Self::format_duration(pending.duration_secs);

        let quality_row = row![text("学习质量:").size(FONT_SM).width(150)]
            .push(Self::view_quality_buttons(&pending.params.quality));

        let ud_row = row![text("理解难度:").size(FONT_SM).width(150)].push(Self::view_difficulty_buttons(
            &pending.params.understanding_difficulty,
            Message::SetUnderstandingDifficulty,
        ));

        let md_row = row![text("记忆难度:").size(FONT_SM).width(150)].push(Self::view_difficulty_buttons(
            &pending.params.memory_difficulty,
            Message::SetMemoryDifficulty,
        ));

        let preset_names: Vec<String> = defaults::load_presets(&self.db)
            .iter()
            .map(|p| p.name.clone())
            .collect();
        let mut preset_row = row![text("预设:").size(FONT_SM).width(80)];
        for (i, name) in preset_names.into_iter().enumerate() {
            preset_row = preset_row.push(
                button(text(name).size(FONT_SM))
                    .on_press(Message::ApplyPreset(i))
                    .style(button::text),
            );
        }

        container(
            column![
                text("会话记录确认").size(FONT_LG),
                text(format!("节点: {}", pending.category_path)).size(FONT_SM),
                text(format!("有效时长: {}", duration)).size(FONT_SM),
                quality_row,
                ud_row,
                md_row,
                text(format!("完成度: {}%", pending.params.completion_rate)).size(FONT_SM),
                preset_row,
                row![
                    button(text("取消").size(FONT_SM))
                        .on_press(Message::CancelSession)
                        .style(button::text),
                    button(text("保存并退出").size(FONT_SM))
                        .on_press(Message::ConfirmSession)
                        .style(button::primary),
                ]
                .spacing(20),
            ]
            .spacing(10)
            .padding(20)
            .width(Fill),
        )
        .width(Fill)
        .height(Fill)
        .center_y(Fill)
        .into()
    }

    fn view_quality_buttons(current: &Quality) -> Element<'_, Message> {
        let mut r = row![].spacing(5);
        for q in Quality::all() {
            let is_current = std::mem::discriminant(&q) == std::mem::discriminant(current);
            let b = if is_current {
                button(text(q.as_str()).size(FONT_SM)).style(button::primary)
            } else {
                button(text(q.as_str()).size(FONT_SM)).style(button::text)
            };
            r = r.push(b.on_press(Message::SetQuality(q)));
        }
        r.into()
    }

    fn view_difficulty_buttons(
        current: &Difficulty,
        msg_fn: fn(Difficulty) -> Message,
    ) -> Element<'_, Message> {
        let mut r = row![].spacing(5);
        for d in Difficulty::all() {
            let is_current = std::mem::discriminant(&d) == std::mem::discriminant(current);
            let b = if is_current {
                button(text(d.as_str()).size(FONT_SM)).style(button::primary)
            } else {
                button(text(d.as_str()).size(FONT_SM)).style(button::text)
            };
            r = r.push(b.on_press(msg_fn(d)));
        }
        r.into()
    }
}
