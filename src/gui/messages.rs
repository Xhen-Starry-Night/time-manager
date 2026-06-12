use chrono::{DateTime, Utc};
use std::path::PathBuf;

use crate::data::{DataError, DataFs};
use crate::data::models::{Card, Preset, Todo};

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(TabId),
    
    DataLoaded(Result<DataSnapshot, DataError>),
    CardsLoaded(Result<Vec<(String, Card)>, DataError>),
    PresetsLoaded(Result<Vec<Preset>, DataError>),
    TodosLoaded(Result<Vec<Todo>, DataError>),
    
    TimerStarted,
    TimerPaused,
    TimerStopped(Result<PathBuf, String>),
    TimerTick(i64),
    
    CategorySelected(String),
    CardSelected(String),
    SearchChanged(String),
    UrgencyFilterChanged(Option<u32>),
    
    ButtonPressed(ButtonId),
    InputChanged(String),
    
    ModalOpen(Modal),
    ModalClose,
    ModalConfirm,
    
    Error(String),
    ClearError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Category,
    Review,
    Timer,
    Schedule,
    Todo,
    Preset,
    Settings,
}

#[derive(Debug, Clone)]
pub enum ButtonId {
    StartTimer,
    PauseTimer,
    StopTimer,
    NewCard,
    NewTodo,
    NewSchedule,
    NewPreset,
    ImportObsidian,
    TrainPreset,
    Delete,
}

#[derive(Debug, Clone)]
pub enum Modal {
    NewCard { path: String },
    LinkTimer { timer_path: PathBuf, duration_ms: i64 },
    NewTodo,
    NewSchedule,
    PresetDetail { name: String },
    ConfirmDelete { item: String },
    Error { message: String },
}

#[derive(Debug, Clone)]
pub struct DataSnapshot {
    pub trees: Vec<String>,
    pub cards: Vec<(String, Card)>,
    pub presets: Vec<Preset>,
    pub todos: Vec<Todo>,
}
