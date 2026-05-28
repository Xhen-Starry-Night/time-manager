use crate::data::models::{Preset, SessionParams};

pub fn load_global_params(db: &crate::data::Database) -> SessionParams {
    let quality = db
        .get_setting("default_quality")
        .ok()
        .flatten()
        .as_deref()
        .and_then(|s| s.try_into().ok())
        .unwrap_or(crate::data::models::Quality::Medium);

    let ud = db
        .get_setting("default_understanding_difficulty")
        .ok()
        .flatten()
        .as_deref()
        .and_then(|s| s.try_into().ok())
        .unwrap_or(crate::data::models::Difficulty::Medium);

    let md = db
        .get_setting("default_memory_difficulty")
        .ok()
        .flatten()
        .as_deref()
        .and_then(|s| s.try_into().ok())
        .unwrap_or(crate::data::models::Difficulty::Medium);

    let cr = db
        .get_setting("default_completion_rate")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(100);

    SessionParams {
        quality,
        understanding_difficulty: ud,
        memory_difficulty: md,
        completion_rate: cr,
    }
}

pub fn save_global_params(db: &crate::data::Database, params: &SessionParams) -> crate::data::error::Result<()> {
    db.set_setting("default_quality", params.quality.as_str())?;
    db.set_setting("default_understanding_difficulty", params.understanding_difficulty.as_str())?;
    db.set_setting("default_memory_difficulty", params.memory_difficulty.as_str())?;
    db.set_setting("default_completion_rate", &params.completion_rate.to_string())?;
    Ok(())
}

pub fn load_presets(db: &crate::data::Database) -> Vec<Preset> {
    db.get_setting("presets")
        .ok()
        .flatten()
        .as_deref()
        .map(crate::modules::learning::timer::params::presets_from_json)
        .unwrap_or_else(Preset::built_in)
}

pub fn save_presets(db: &crate::data::Database, presets: &[Preset]) -> crate::data::error::Result<()> {
    let json = crate::modules::learning::timer::params::presets_to_json(presets);
    db.set_setting("presets", &json)
}