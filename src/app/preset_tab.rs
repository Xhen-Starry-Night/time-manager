use crate::data::models::Preset;

#[derive(Default)]
pub struct PresetTabState {
    pub selected_preset: Option<String>,
    pub presets: Vec<Preset>,
}