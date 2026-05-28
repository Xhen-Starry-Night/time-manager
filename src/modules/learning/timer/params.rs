use crate::data::models::{Difficulty, Preset, Quality, SessionParams};

pub fn resolve_params(
    global_defaults: &SessionParams,
    node_quality: Option<&Quality>,
    node_ud: Option<&Difficulty>,
    node_md: Option<&Difficulty>,
) -> SessionParams {
    SessionParams {
        quality: node_quality.cloned().unwrap_or_else(|| global_defaults.quality.clone()),
        understanding_difficulty: node_ud
            .cloned()
            .unwrap_or_else(|| global_defaults.understanding_difficulty.clone()),
        memory_difficulty: node_md
            .cloned()
            .unwrap_or_else(|| global_defaults.memory_difficulty.clone()),
        completion_rate: global_defaults.completion_rate,
    }
}

pub fn apply_preset(preset: &Preset, params: &mut SessionParams) {
    params.quality = preset.quality.clone();
    params.understanding_difficulty = preset.understanding_difficulty.clone();
    params.memory_difficulty = preset.memory_difficulty.clone();
    params.completion_rate = preset.completion_rate;
}

pub fn presets_from_json(json: &str) -> Vec<Preset> {
    serde_json::from_str(json).unwrap_or_else(|_| Preset::built_in())
}

pub fn presets_to_json(presets: &[Preset]) -> String {
    serde_json::to_string(presets).unwrap_or_else(|_| "[]".into())
}