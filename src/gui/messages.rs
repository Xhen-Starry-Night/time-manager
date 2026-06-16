use std::path::PathBuf;
use uuid::Uuid;

use crate::data::DataError;
use crate::data::models::{Card, Preset, Todo, MemoryQuality};
use crate::gui::styles::AppTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Folder,
    Card,
}

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
    
    TimerLinkModeOpen,
    TimerLinkModeClose,
    TimerCardPathChanged(String),
    TimerCardSelected(String),
    TimerMemoryQualityChanged(MemoryQuality),
    TimerLinkConfirm,
    TimerCreateNewCard,
    TimerNewCardNameChanged(String),
    TimerNewCardPresetChanged(String),
    TimerNewCardTypeChanged(NodeType),
    TimerNewCardConfirm,
    
    TimerHistoryLoad,
    TimerHistoryLoaded(Vec<crate::app::timer_tab::TimerRecord>),
    TimerHistoryEdit(Uuid),
    TimerHistoryDelete(Uuid),
    TimerHistoryUpdate(crate::app::timer_tab::TimerRecord),
    TimerHistoryShow,
    TimerHistoryHide,
    
    QuickTimerStart(String),
    
    CategorySelected(String),
    CardSelected(String),
    SearchChanged(String),
    SearchQueryChanged(String),
    SearchItemSelected(usize),
    SearchFocusChanged(bool),
    UrgencyFilterChanged(Option<u32>),
    
    ButtonPressed(ButtonId),
    InputChanged(String),
    
    ModalOpen(Modal),
    ModalClose,
    ModalConfirm,
    
    NewTodoContentChanged(String),
    NewTodoPriorityChanged(Option<u32>),
    NewTodoDueYearChanged(String),
    NewTodoDueMonthChanged(String),
    NewTodoDueDayChanged(String),
    NewTodoDueHourChanged(String),
    NewTodoDueMinuteChanged(String),
    NewTodoTagsChanged(String),
    
    TodoSelected(Uuid),
    ToggleTodo(Uuid),
    DeleteTodo(Uuid),
    
    NewNodeOpen(NodeType),
    NewNodeNameChanged(String),
    NewNodeTypeChanged(NodeType),
    NewNodePresetChanged(String),
    NewNodeConfirm,
    
    EditCardOpen(String),
    EditCardNameChanged(String),
    EditCardPresetChanged(String),
    EditCardNextReviewChanged(String),
    EditCardClearPrediction,
    EditCardPredict,
    EditCardToggleGroup(String),
    EditCardAddReview,
    EditCardRemoveReview(usize),
    EditCardNewReviewDurationChanged(String),
    EditCardNewReviewQualityChanged(MemoryQuality),
    EditCardConfirm,
    
    CreateTreeNameChanged(String),
    CreateTreeImportPathChanged(String),
    CreateTreeImportRulesPathChanged(String),
    DeleteNodeOpen(String, bool),
    DeleteNodeConfirm,
    
    ReviewCardSelected(String),
    StartReviewTimer(String),
    RefreshPredictions,
    
    ScheduleCreateOpen,
    ScheduleCreateConfirm,
    ScheduleEditOpen(Uuid),
    ScheduleEditConfirm,
    ScheduleDelete(Uuid),
    ScheduleFormSummaryChanged(String),
    ScheduleFormStartDateChanged(String),
    ScheduleFormStartTimeChanged(String),
    ScheduleFormEndDateChanged(String),
    ScheduleFormEndTimeChanged(String),
    ScheduleFormDescriptionChanged(String),
    ScheduleFormLocationChanged(String),
    ScheduleFormReminderChanged(String),
    ScheduleFormRruleChanged(crate::data::models::RecurrenceRule),

    PresetCreateOpen,
    PresetEditOpen(String),
    PresetFormDismissed,
    PresetFormNameChanged(String),
    PresetFormDescriptionChanged(String),
    PresetFormMatchRuleAdded,
    PresetFormMatchRuleChanged(usize, String),
    PresetFormMatchRuleRemoved(usize),
    PresetFormSaveRequested,
    PresetDeleteRequested(String),
    PresetDeleteConfirmed(String),
    PresetDeleteDismissed,

    SettingsFormDataDirChanged(String),
    SettingsFormDefaultPresetChanged(String),
    SettingsFormSaveRequested,
    SettingsFormDismissMessage,

    ThemeChanged(AppTheme),

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
    EditTodo { id: Uuid },
    NewSchedule,
    PresetDetail { name: String },
    ConfirmDelete { item: String },
    Error { message: String },
    Info { message: String },
    
    NewNode,
    EditCard { path: String },
    CreateTree,
}

#[derive(Debug, Clone)]
pub struct DataSnapshot {
    pub trees: Vec<String>,
    pub cards: Vec<(String, Card)>,
    pub presets: Vec<Preset>,
    pub todos: Vec<Todo>,
}
