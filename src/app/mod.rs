use std::path::PathBuf;

use iced::{application, Element, Task};
use iced::widget::{button, column, row, text, container, rule};
use iced::Length;

use crate::data::DataFs;
use crate::gui::{Message, TabId, Modal};
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
                
                let state = App {
                    active_tab: TabId::Category,
                    data_dir: data_dir.clone(),
                    data_fs,
                    
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
                
                (state, Task::none())
            },
            App::update,
            App::view,
        )
        .title(App::title)
        .run()
    }
    
    fn title(&self) -> String {
        String::from("Time Manager")
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab_id) => {
                self.active_tab = tab_id;
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
                self.modal = Some(modal);
                Task::none()
            }
            
            Message::ModalClose => {
                self.modal = None;
                Task::none()
            }
            
            _ => Task::none(),
        }
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
                    let tree_element = self.category_tab.tree_view.view(
                        &self.category_tab.tree_nodes,
                        self.category_tab.selected_path.as_deref(),
                    ).map(|path| Message::CardSelected(path));
                    
                    container(scrollable(tree_element))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
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
        
        column![tabs, rule::horizontal(1.0), content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn tab_button(label: &str, tab_id: TabId, active_tab: TabId) -> Element<Message> {
    let is_active = tab_id == active_tab;
    
    let btn = button(text(label))
        .on_press(Message::SwitchTab(tab_id));
    
    if is_active {
        btn.style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        })
    } else {
        btn
    }
    .into()
}