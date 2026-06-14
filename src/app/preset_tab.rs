use crate::data::models::Preset;

#[derive(Default)]
pub struct PresetTabState {
    pub selected_preset: Option<String>,
    pub presets: Vec<Preset>,

    // form state
    pub show_form: bool,
    pub editing_name: Option<String>,
    pub form_name: String,
    pub form_description: String,
    pub form_match_rules: Vec<String>,
    pub form_error: Option<String>,

    // delete confirm
    pub delete_target: Option<String>,
}

impl PresetTabState {
    pub fn reset_form(&mut self) {
        self.form_name.clear();
        self.form_description.clear();
        self.form_match_rules.clear();
        self.form_error = None;
        self.editing_name = None;
    }

    pub fn load_from_preset(&mut self, name: &str) {
        if let Some(preset) = self.presets.iter().find(|p| p.name == name) {
            self.form_name = preset.name.clone();
            self.form_description = preset.description.clone().unwrap_or_default();
            self.form_match_rules = preset.match_rules.clone();
            self.editing_name = Some(preset.name.clone());
            self.form_error = None;
        }
    }

    pub fn validate(&self) -> Result<Preset, String> {
        let name = self.form_name.trim();
        if name.is_empty() {
            return Err("名称不能为空".into());
        }
        let duplicate = self.presets.iter().any(|p| {
            if let Some(ref editing) = self.editing_name {
                p.name == name && p.name != *editing
            } else {
                p.name == name
            }
        });
        if duplicate {
            return Err("名称已存在".into());
        }

        let description = if self.form_description.trim().is_empty() {
            None
        } else {
            Some(self.form_description.trim().to_string())
        };

        let match_rules: Vec<String> = self
            .form_match_rules
            .iter()
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
            .collect();

        Ok(Preset {
            name: name.to_string(),
            description,
            match_rules,
            fsrs_parameters: None,
            trained_at: None,
        })
    }
}
