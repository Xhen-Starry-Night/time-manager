use time_manager::data::models::{Difficulty, Quality, SessionParams};
use time_manager::modules::learning::prediction::algorithm::PredictionAlgorithm;
use time_manager::modules::learning::prediction::fsrs::adapter::FsrsAdapter;
use time_manager::modules::learning::prediction::fsrs::state::FsrsCardState;
use time_manager::modules::learning::timer::params::{resolve_params, apply_preset, presets_from_json};
use time_manager::data::models::Preset;
use chrono::Utc;

#[test]
fn test_fsrs_adapter_first_review() {
    let adapter = FsrsAdapter::new();
    let result = adapter.predict_next_review(&[], &[], &Quality::High);
    assert_eq!(adapter.algorithm_name(), "fsrs");
    assert!(result.next_review > Utc::now());
    assert!(!result.state_bytes.is_empty());

    let state = FsrsCardState::from_bytes(&result.state_bytes).expect("deserialize");
    assert!(state.stability > 0.0);
    assert!(state.difficulty > 0.0);
}

#[test]
fn test_fsrs_adapter_second_review() {
    let adapter = FsrsAdapter::new();
    let first = adapter.predict_next_review(&[], &[], &Quality::High);
    let second = adapter.predict_next_review(&[], &first.state_bytes, &Quality::Complete);
    assert!(second.next_review > first.next_review);

    let state = FsrsCardState::from_bytes(&second.state_bytes).expect("deserialize");
    assert!(state.stability > 0.0);
}

#[test]
fn test_fsrs_rating_mapping() {
    let adapter = FsrsAdapter::new();
    let low = adapter.predict_next_review(&[], &[], &Quality::VeryLow);
    let high = adapter.predict_next_review(&[], &[], &Quality::Complete);
    assert!(low.next_review <= high.next_review);
}

#[test]
fn test_fsrs_state_serialization() {
    let state = FsrsCardState {
        difficulty: 5.0,
        stability: 10.0,
        last_date: 3.0,
        due: 7.0,
    };
    let bytes = state.to_bytes();
    let restored = FsrsCardState::from_bytes(&bytes).expect("restore");
    assert_eq!(restored.difficulty, 5.0);
    assert_eq!(restored.stability, 10.0);
    assert_eq!(restored.last_date, 3.0);
    assert_eq!(restored.due, 7.0);
}

#[test]
fn test_fsrs_state_from_invalid_bytes() {
    assert!(FsrsCardState::from_bytes(b"garbage").is_none());
    assert!(FsrsCardState::from_bytes(&[]).is_none());
}

#[test]
fn test_fsrs_five_consecutive_reviews() {
    let adapter = FsrsAdapter::new();
    let mut state_bytes = Vec::new();
    let mut last_next = Utc::now();

    for (i, quality) in [Quality::High, Quality::Complete, Quality::High, Quality::Complete, Quality::Complete].iter().enumerate() {
        let result = adapter.predict_next_review(&[], &state_bytes, quality);
        if i > 0 {
            assert!(result.next_review >= last_next, "review {} should have later next_review", i);
        }
        last_next = result.next_review;
        state_bytes = result.state_bytes;
    }

    let final_state = FsrsCardState::from_bytes(&state_bytes).expect("final state");
    assert!(final_state.stability > 1.0);
}

#[test]
fn test_fsrs_again_resets_interval() {
    let adapter = FsrsAdapter::new();
    let good = adapter.predict_next_review(&[], &[], &Quality::High);
    let after_good = adapter.predict_next_review(&[], &good.state_bytes, &Quality::High);
    let after_again = adapter.predict_next_review(&[], &after_good.state_bytes, &Quality::VeryLow);
    assert!(after_again.next_review < after_good.next_review);
}

#[test]
fn test_fsrs_all_quality_levels() {
    let adapter = FsrsAdapter::new();
    for q in [Quality::VeryLow, Quality::Low, Quality::Medium, Quality::High, Quality::Complete] {
        let result = adapter.predict_next_review(&[], &[], &q);
        assert!(result.next_review >= Utc::now() - chrono::Duration::days(1));
        assert!(!result.state_bytes.is_empty());
    }
}

#[test]
fn test_fsrs_state_preserves_across_serialization() {
    let adapter = FsrsAdapter::new();
    let result = adapter.predict_next_review(&[], &[], &Quality::Complete);
    let state = FsrsCardState::from_bytes(&result.state_bytes).expect("state");

    let serialized = state.to_bytes();
    let deserialized = FsrsCardState::from_bytes(&serialized).expect("roundtrip");

    let result2 = adapter.predict_next_review(&[], &deserialized.to_bytes(), &Quality::High);
    let state2 = FsrsCardState::from_bytes(&result2.state_bytes).expect("state2");
    assert!(state2.stability > state.stability);
}

#[test]
fn test_fsrs_adapter_default() {
    let adapter = FsrsAdapter::default();
    assert_eq!(adapter.algorithm_name(), "fsrs");
}

#[test]
fn test_fsrs_adapter_with_retention() {
    let adapter = FsrsAdapter::new().with_retention(0.85);
    let result = adapter.predict_next_review(&[], &[], &Quality::High);
    assert!(result.next_review > Utc::now());
}

#[test]
fn test_fsrs_low_retention_longer_interval() {
    let high_r = FsrsAdapter::new().with_retention(0.95);
    let low_r = FsrsAdapter::new().with_retention(0.80);
    let result_high = high_r.predict_next_review(&[], &[], &Quality::High);
    let result_low = low_r.predict_next_review(&[], &[], &Quality::High);
    assert!(result_low.next_review >= result_high.next_review);
}

#[test]
fn test_params_resolution() {
    let global = SessionParams::default();
    let resolved = resolve_params(&global, Some(&Quality::High), Some(&Difficulty::Hard), Some(&Difficulty::Easy));
    assert_eq!(resolved.quality, Quality::High);
    assert_eq!(resolved.understanding_difficulty, Difficulty::Hard);
    assert_eq!(resolved.memory_difficulty, Difficulty::Easy);
    assert_eq!(resolved.completion_rate, 100);
}

#[test]
fn test_params_resolution_fallback() {
    let global = SessionParams::default();
    let resolved = resolve_params(&global, None, None, None);
    assert_eq!(resolved.quality, Quality::Medium);
    assert_eq!(resolved.understanding_difficulty, Difficulty::Medium);
}

#[test]
fn test_params_resolution_partial_override() {
    let global = SessionParams::default();
    let resolved = resolve_params(&global, Some(&Quality::Complete), None, None);
    assert_eq!(resolved.quality, Quality::Complete);
    assert_eq!(resolved.understanding_difficulty, Difficulty::Medium);
    assert_eq!(resolved.memory_difficulty, Difficulty::Medium);
}

#[test]
fn test_apply_preset() {
    let mut params = SessionParams::default();
    let preset = Preset::built_in()[0].clone();
    apply_preset(&preset, &mut params);
    assert_eq!(params.quality, Quality::High);
    assert_eq!(params.completion_rate, 100);
}

#[test]
fn test_apply_all_built_in_presets() {
    for preset in Preset::built_in() {
        let mut params = SessionParams::default();
        apply_preset(&preset, &mut params);
        assert_eq!(params.quality, preset.quality);
        assert_eq!(params.understanding_difficulty, preset.understanding_difficulty);
        assert_eq!(params.memory_difficulty, preset.memory_difficulty);
        assert_eq!(params.completion_rate, preset.completion_rate);
    }
}

#[test]
fn test_presets_json_roundtrip() {
    let presets = Preset::built_in();
    let json = time_manager::modules::learning::timer::params::presets_to_json(&presets);
    let restored = presets_from_json(&json);
    assert_eq!(restored.len(), 3);
    assert_eq!(restored[0].name, "专注学习");
    assert_eq!(restored[1].name, "轻松复习");
    assert_eq!(restored[2].name, "快速浏览");
}

#[test]
fn test_presets_from_invalid_json() {
    let restored = presets_from_json("not json");
    assert_eq!(restored.len(), 3);
}

#[test]
fn test_presets_custom_roundtrip() {
    let custom = vec![Preset {
        name: "自定义".into(),
        quality: Quality::Low,
        understanding_difficulty: Difficulty::VeryHard,
        memory_difficulty: Difficulty::Hard,
        completion_rate: 30,
    }];
    let json = time_manager::modules::learning::timer::params::presets_to_json(&custom);
    let restored = presets_from_json(&json);
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].name, "自定义");
    assert_eq!(restored[0].completion_rate, 30);
}

#[test]
fn test_quality_display() {
    assert_eq!(Quality::VeryLow.to_string(), "极低");
    assert_eq!(Quality::Low.to_string(), "低");
    assert_eq!(Quality::Medium.to_string(), "中");
    assert_eq!(Quality::High.to_string(), "高");
    assert_eq!(Quality::Complete.to_string(), "完整");
}

#[test]
fn test_difficulty_display() {
    assert_eq!(Difficulty::VeryEasy.to_string(), "极易");
    assert_eq!(Difficulty::Easy.to_string(), "易");
    assert_eq!(Difficulty::Medium.to_string(), "中");
    assert_eq!(Difficulty::Hard.to_string(), "难");
    assert_eq!(Difficulty::VeryHard.to_string(), "极难");
}

#[test]
fn test_quality_try_from() {
    assert_eq!("高".try_into(), Ok(Quality::High));
    assert_eq!("中".try_into(), Ok(Quality::Medium));
    assert!(Quality::try_from("invalid").is_err());
}

#[test]
fn test_difficulty_try_from() {
    assert_eq!("难".try_into(), Ok(Difficulty::Hard));
    assert_eq!("易".try_into(), Ok(Difficulty::Easy));
    assert!(Difficulty::try_from("invalid").is_err());
}

#[test]
fn test_session_params_default() {
    let p = SessionParams::default();
    assert_eq!(p.quality, Quality::Medium);
    assert_eq!(p.understanding_difficulty, Difficulty::Medium);
    assert_eq!(p.memory_difficulty, Difficulty::Medium);
    assert_eq!(p.completion_rate, 100);
}

#[test]
fn test_quality_all() {
    assert_eq!(Quality::all().len(), 5);
}

#[test]
fn test_difficulty_all() {
    assert_eq!(Difficulty::all().len(), 5);
}