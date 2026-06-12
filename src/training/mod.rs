use crate::data::models::{Card, MemoryQuality};
use fsrs::{FSRSItem, FSRSReview};
use globset::{Glob, GlobSetBuilder};

pub fn matches_any_pattern(path: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).unwrap_or_else(|e| {
            eprintln!("Invalid glob pattern '{}': {}", pattern, e);
            Glob::new("*").unwrap()
        });
        builder.add(glob);
    }

    let set = builder.build().unwrap();
    set.is_match(path)
}

pub fn convert_card_to_fsrs_items(card: &Card) -> Option<Vec<FSRSItem>> {
    if card.review_records.is_empty() {
        return None;
    }

    let mut reviews = Vec::new();

    for (i, record) in card.review_records.iter().enumerate() {
        let delta_t = if i == 0 {
            0
        } else {
            let prev_time = card.review_records[i - 1].reviewed_at;
            let curr_time = record.reviewed_at;
            (curr_time - prev_time).num_days() as u32
        };

        reviews.push(FSRSReview {
            rating: memory_quality_to_rating(&record.memory_quality),
            delta_t,
        });
    }

    Some(vec![FSRSItem { reviews }])
}

fn memory_quality_to_rating(quality: &MemoryQuality) -> u32 {
    match quality {
        MemoryQuality::Relearn => 1,
        MemoryQuality::Hard => 2,
        MemoryQuality::Good => 3,
        MemoryQuality::Easy => 4,
    }
}

pub fn collect_training_data(cards: &[Card]) -> Vec<FSRSItem> {
    cards
        .iter()
        .filter_map(convert_card_to_fsrs_items)
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::ReviewRecord;
    use chrono::{Duration, Utc};

    #[test]
    fn test_pattern_matching() {
        let patterns = vec![
            "main/语言/英语/**".to_string(),
            "main/数学/**".to_string(),
        ];

        assert!(matches_any_pattern("main/语言/英语/单词", &patterns));
        assert!(matches_any_pattern("main/语言/英语/六级/单词", &patterns));
        assert!(matches_any_pattern("main/数学/微积分", &patterns));
        assert!(!matches_any_pattern("main/编程/Rust", &patterns));
    }

    #[test]
    fn test_convert_card_to_fsrs_items() {
        let mut card = Card::new("test".to_string());
        card.review_records.push(ReviewRecord {
            timer_path: "t1".to_string(),
            reviewed_at: Utc::now() - Duration::days(3),
            memory_quality: MemoryQuality::Good,
            state_bytes: vec![],
            fsrs_state_bytes: vec![],
        });
        card.review_records.push(ReviewRecord {
            timer_path: "t2".to_string(),
            reviewed_at: Utc::now(),
            memory_quality: MemoryQuality::Easy,
            state_bytes: vec![],
            fsrs_state_bytes: vec![],
        });

        let items = convert_card_to_fsrs_items(&card).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].reviews.len(), 2);
    }
}
