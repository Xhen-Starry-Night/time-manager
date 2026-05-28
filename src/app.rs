use chrono::Utc;
use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Fill, Task};

use crate::data::models::{Difficulty, Quality, SessionParams};
use crate::data::Database;
use crate::modules::learning::category::tree::CategoryForest;
use crate::modules::learning::prediction::fsrs::adapter::FsrsAdapter;
use crate::modules::learning::prediction::algorithm::PredictionAlgorithm;
use crate::modules::learning::timer::params::{apply_preset, resolve_params};
use crate::modules::learning::timer::state_machine::TimerStateMachine;
use crate::settings::defaults;

#[derive(Debug, Clone)]
pub enum Screen {
    CategoryTree,
    ReviewDashboard,
    Timer,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchScreen(Screen),
    SelectCategory(i64),
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
    due_predictions: Vec<crate::data::models::PredictionState>,
    fsrs: FsrsAdapter,
}

pub struct PendingSession {
    pub category_id: i64,
    pub category_path: String,
    pub duration_secs: i64,
    pub params: SessionParams,
}

impl App {
    pub fn new() -> Self {
        let db = Database::open_default().expect("failed to open database");
        let cats = db.get_all_categories().unwrap_or_default();
        let forest = CategoryForest::from_categories(&cats);
        let global_params = defaults::load_global_params(&db);
        let due_predictions = db.get_due_predictions(&Utc::now()).unwrap_or_default();
        let fsrs = FsrsAdapter::new();

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
            due_predictions,
            fsrs,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchScreen(screen) => {
                self.screen = screen;
                if matches!(self.screen, Screen::ReviewDashboard) {
                    self.due_predictions = self.db.get_due_predictions(&Utc::now()).unwrap_or_default();
                }
            }
            Message::SelectCategory(id) => {
                self.selected_category_id = Some(id);
            }
            Message::StartTimer(category_id) => {
                let node = self.forest.find(category_id);
                let params = resolve_params(
                    &self.global_params,
                    node.and_then(|n| n.default_quality.as_ref()),
                    node.and_then(|n| n.default_understanding_difficulty.as_ref()),
                    node.and_then(|n| n.default_memory_difficulty.as_ref()),
                );
                self.current_params = params;
                self.timer.start(category_id).ok();
                self.screen = Screen::Timer;
            }
            Message::PauseTimer => {
                self.timer.pause().ok();
            }
            Message::ResumeTimer => {
                self.timer.resume().ok();
            }
            Message::StopTimer => {
                if let Ok(stopped) = self.timer.stop() {
                    let path = self.forest.find(stopped.category_id)
                        .map(|n| n.path.clone())
                        .unwrap_or_default();
                    self.pending_session = Some(PendingSession {
                        category_id: stopped.category_id,
                        category_path: path,
                        duration_secs: stopped.duration_secs,
                        params: self.current_params.clone(),
                    });
                }
            }
            Message::ConfirmSession => {
                if let Some(pending) = self.pending_session.take() {
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
                        let pred = self.fsrs.predict_next_review(&[], &[], &pending.params.quality);
                        self.db.upsert_prediction_state(&crate::data::models::PredictionStateInsert {
                            category_id: pending.category_id,
                            algorithm: self.fsrs.algorithm_name().into(),
                            last_review: now,
                            next_review: pred.next_review,
                            algorithm_state: pred.state_bytes,
                        }).ok();
                    }
                }
                self.screen = Screen::CategoryTree;
            }
            Message::CancelSession => {
                self.pending_session = None;
            }
            Message::SetQuality(q) => {
                if let Some(p) = &mut self.pending_session {
                    p.params.quality = q;
                } else {
                    self.current_params.quality = q;
                }
            }
            Message::SetUnderstandingDifficulty(d) => {
                if let Some(p) = &mut self.pending_session {
                    p.params.understanding_difficulty = d;
                } else {
                    self.current_params.understanding_difficulty = d;
                }
            }
            Message::SetMemoryDifficulty(d) => {
                if let Some(p) = &mut self.pending_session {
                    p.params.memory_difficulty = d;
                } else {
                    self.current_params.memory_difficulty = d;
                }
            }
            Message::SetCompletionRate(r) => {
                if let Some(p) = &mut self.pending_session {
                    p.params.completion_rate = r;
                } else {
                    self.current_params.completion_rate = r;
                }
            }
            Message::ApplyPreset(idx) => {
                let presets = defaults::load_presets(&self.db);
                if let Some(preset) = presets.get(idx) {
                    if let Some(p) = &mut self.pending_session {
                        apply_preset(preset, &mut p.params);
                    } else {
                        apply_preset(preset, &mut self.current_params);
                    }
                }
            }
            Message::RefreshReviewDashboard => {
                self.due_predictions = self.db.get_due_predictions(&Utc::now()).unwrap_or_default();
            }
            Message::Tick => {
                if self.timer.is_running() {
                    self.timer_display_secs = self.timer.effective_secs();
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.pending_session.is_some() {
            return self.view_session_confirm();
        }

        let content = match &self.screen {
            Screen::CategoryTree => self.view_category_tree(),
            Screen::ReviewDashboard => self.view_review_dashboard(),
            Screen::Timer => self.view_timer(),
        };

        column![content, self.view_nav_bar()].into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.timer.is_running() {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    fn nav_btn<'a>(label: &'a str, screen: Screen, current: &'a Screen) -> button::Button<'a, Message> {
        let b = button(text(label).size(14));
        if std::mem::discriminant(&screen) == std::mem::discriminant(current) {
            b.style(button::primary)
        } else {
            b.style(button::text)
        }
        .on_press(Message::SwitchScreen(screen))
    }

    fn view_nav_bar(&self) -> Element<'_, Message> {
        row![
            Self::nav_btn("分类树", Screen::CategoryTree, &self.screen),
            Self::nav_btn("复习看板", Screen::ReviewDashboard, &self.screen),
            Self::nav_btn("计时", Screen::Timer, &self.screen),
        ]
        .spacing(10)
        .padding(5)
        .into()
    }

    fn format_duration(secs: i64) -> String {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    fn view_category_tree(&self) -> Element<'_, Message> {
        let mut content = column![text("分类树").size(20)].spacing(5).padding(10);

        for root in &self.forest.roots {
            content = content.push(Self::view_tree_node(root));
        }

        if self.forest.roots.is_empty() {
            content = content.push(text("暂无分类"));
        }

        scrollable(content).into()
    }

    fn view_tree_node(node: &crate::modules::learning::category::tree::CategoryNode) -> Element<'_, Message> {
        let indent = node.path.matches('/').count();
        let prefix = "  ".repeat(indent);
        let icon = if node.children.is_empty() { "📄" } else { "📁" };

        let mut col = column![button(text(format!("{}{} {}", prefix, icon, node.name)))
            .on_press(Message::SelectCategory(node.id))
            .style(button::text)
            .padding([2, 5])];

        for child in &node.children {
            col = col.push(Self::view_tree_node(child));
        }

        col.into()
    }

    fn view_review_dashboard(&self) -> Element<'_, Message> {
        let mut content = column![text("复习看板").size(20)].spacing(5).padding(10);

        if self.due_predictions.is_empty() {
            content = content.push(text("暂无待复习项目"));
        }

        for pred in &self.due_predictions {
            let cat_name = self.forest.find(pred.category_id)
                .map(|n| n.path.clone())
                .unwrap_or_else(|| format!("ID:{}", pred.category_id));

            let overdue = pred.next_review < Utc::now();
            let marker = if overdue { "🔴" } else { "🟡" };
            let days_until = (pred.next_review - Utc::now()).num_days();
            let due_text = if overdue {
                format!("已过期 {} 天", -days_until)
            } else {
                format!("{} 天后到期", days_until)
            };

            content = content.push(
                row![
                    text(format!("{} {}", marker, cat_name)).width(Fill),
                    text(due_text).size(12),
                    button("开始计时")
                        .on_press(Message::StartTimer(pred.category_id))
                        .style(button::primary),
                ]
                .spacing(10)
                .padding(5),
            );
        }

        scrollable(content).into()
    }

    fn view_timer(&self) -> Element<'_, Message> {
        let category_name = self.timer.category_id
            .and_then(|id| self.forest.find(id))
            .map(|n| n.path.clone())
            .unwrap_or_else(|| "未选择".into());

        let duration = Self::format_duration(self.timer_display_secs);
        let is_paused = matches!(&self.timer.state, crate::modules::learning::timer::state_machine::TimerState::Paused { .. });
        let is_running = self.timer.is_running();

        let pause_resume = if is_paused {
            button("继续").on_press(Message::ResumeTimer).style(button::primary)
        } else {
            button("暂停").on_press(Message::PauseTimer).style(button::secondary)
        };

        let stop = if is_running {
            button("结束").on_press(Message::StopTimer).style(button::danger)
        } else {
            button("结束").style(button::text)
        };

        column![
            text(format!("当前节点: {}", category_name)).size(16),
            text(duration).size(48),
            text("有效时长").size(12),
            row![pause_resume, stop].spacing(20),
        ]
        .spacing(15)
        .padding(30)
        .into()
    }

    fn view_session_confirm(&self) -> Element<'_, Message> {
        let pending = self.pending_session.as_ref().unwrap();
        let duration = Self::format_duration(pending.duration_secs);

        let quality_row = row![text("学习质量:").width(100)]
            .push(Self::view_quality_buttons(&pending.params.quality));

        let ud_row = row![text("理解难度:").width(100)]
            .push(Self::view_difficulty_buttons(&pending.params.understanding_difficulty, Message::SetUnderstandingDifficulty));

        let md_row = row![text("记忆难度:").width(100)]
            .push(Self::view_difficulty_buttons(&pending.params.memory_difficulty, Message::SetMemoryDifficulty));

        let preset_names: Vec<String> = defaults::load_presets(&self.db)
            .iter().map(|p| p.name.clone()).collect();
        let mut preset_row = row![text("预设:").width(50)];
        for (i, name) in preset_names.into_iter().enumerate() {
            preset_row = preset_row.push(
                button(text(name).size(12))
                    .on_press(Message::ApplyPreset(i))
                    .style(button::text),
            );
        }

        column![
            text("会话记录确认").size(20),
            text(format!("节点: {}", pending.category_path)),
            text(format!("有效时长: {}", duration)),
            quality_row,
            ud_row,
            md_row,
            text(format!("完成度: {}%", pending.params.completion_rate)),
            preset_row,
            row![
                button("取消").on_press(Message::CancelSession).style(button::text),
                button("保存并退出").on_press(Message::ConfirmSession).style(button::primary),
            ].spacing(20),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn view_quality_buttons(current: &Quality) -> Element<'_, Message> {
        let mut r = row![].spacing(5);
        for q in Quality::all() {
            let is_current = std::mem::discriminant(&q) == std::mem::discriminant(current);
            let b = if is_current {
                button(text(q.as_str()).size(12)).style(button::primary)
            } else {
                button(text(q.as_str()).size(12)).style(button::text)
            };
            r = r.push(b.on_press(Message::SetQuality(q)));
        }
        r.into()
    }

    fn view_difficulty_buttons(current: &Difficulty, msg_fn: fn(Difficulty) -> Message) -> Element<'_, Message> {
        let mut r = row![].spacing(5);
        for d in Difficulty::all() {
            let is_current = std::mem::discriminant(&d) == std::mem::discriminant(current);
            let b = if is_current {
                button(text(d.as_str()).size(12)).style(button::primary)
            } else {
                button(text(d.as_str()).size(12)).style(button::text)
            };
            r = r.push(b.on_press(msg_fn(d)));
        }
        r.into()
    }
}