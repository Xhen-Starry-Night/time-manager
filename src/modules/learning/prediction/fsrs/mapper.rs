use crate::data::models::Quality;

pub fn quality_to_rating(quality: &Quality) -> u32 {
    match quality {
        Quality::VeryLow => 1,
        Quality::Low => 1,
        Quality::Medium => 2,
        Quality::High => 3,
        Quality::Complete => 4,
    }
}