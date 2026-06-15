use chrono::{Duration, Utc};
use time_manager::data::models::MemoryQuality;
use time_manager::fsrs::FsrsPredictor;

#[test]
fn test_fsrs_all_quality_levels() {
    let predictor = FsrsPredictor::new().unwrap();

    // Test that all quality levels produce valid state
    let (_, state_relearn) = predictor
        .predict_next_review(None, MemoryQuality::Relearn, 0, 0.9)
        .unwrap();

    let (_, state_hard) = predictor
        .predict_next_review(None, MemoryQuality::Hard, 0, 0.9)
        .unwrap();

    let (_, state_good) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();

    let (_, state_easy) = predictor
        .predict_next_review(None, MemoryQuality::Easy, 0, 0.9)
        .unwrap();

    // All should produce valid memory states
    assert!(state_relearn.stability > 0.0);
    assert!(state_hard.stability > 0.0);
    assert!(state_good.stability > 0.0);
    assert!(state_easy.stability > 0.0);
}

#[test]
fn test_fsrs_with_existing_state() {
    let predictor = FsrsPredictor::new().unwrap();

    let (_, initial_state) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();

    let (second_interval, _) = predictor
        .predict_next_review(Some(initial_state), MemoryQuality::Good, 7, 0.9)
        .unwrap();

    assert!(second_interval > 0);
}

#[test]
fn test_fsrs_different_days_elapsed() {
    let predictor = FsrsPredictor::new().unwrap();

    let (_, state) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();

    let (interval_1, _) = predictor
        .predict_next_review(Some(state), MemoryQuality::Good, 1, 0.9)
        .unwrap();

    let (interval_7, _) = predictor
        .predict_next_review(Some(state), MemoryQuality::Good, 7, 0.9)
        .unwrap();

    assert!(interval_7 > interval_1);
}

#[test]
fn test_fsrs_different_retention_targets() {
    let predictor = FsrsPredictor::new().unwrap();

    let (_, state_low) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.8)
        .unwrap();

    let (_, state_high) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.95)
        .unwrap();

    // Both should produce valid memory states
    assert!(state_low.stability > 0.0);
    assert!(state_high.stability > 0.0);
}

#[test]
fn test_memory_state_serialization_roundtrip() {
    let state = fsrs::MemoryState {
        stability: 5.5,
        difficulty: 0.3,
    };

    let bytes = FsrsPredictor::memory_state_to_bytes(&state);
    assert_eq!(bytes.len(), 8);

    let recovered = FsrsPredictor::bytes_to_memory_state(&bytes);
    assert!(recovered.is_some());

    let recovered = recovered.unwrap();
    assert!((recovered.stability - state.stability).abs() < 0.001);
    assert!((recovered.difficulty - state.difficulty).abs() < 0.001);
}

#[test]
fn test_memory_state_serialization_edge_cases() {
    let empty_result = FsrsPredictor::bytes_to_memory_state(&[]);
    assert!(empty_result.is_none());

    let partial_result = FsrsPredictor::bytes_to_memory_state(&[1, 2, 3]);
    assert!(partial_result.is_none());
}

#[test]
fn test_memory_state_extreme_values() {
    let state = fsrs::MemoryState {
        stability: 1000000.0,
        difficulty: 0.001,
    };

    let bytes = FsrsPredictor::memory_state_to_bytes(&state);
    let recovered = FsrsPredictor::bytes_to_memory_state(&bytes).unwrap();

    assert!((recovered.stability - state.stability).abs() < 0.001);
}

#[test]
fn test_urgency_overdue_cards() {
    let overdue = Utc::now() - Duration::days(5);
    assert_eq!(FsrsPredictor::calculate_urgency(overdue), 3);
}

#[test]
fn test_urgency_today_cards() {
    let today = Utc::now();
    assert_eq!(FsrsPredictor::calculate_urgency(today), 2);
}

#[test]
fn test_urgency_near_future_cards() {
    let near_future = Utc::now() + Duration::days(2);
    assert_eq!(FsrsPredictor::calculate_urgency(near_future), 1);
}

#[test]
fn test_urgency_far_future_cards() {
    let far_future = Utc::now() + Duration::days(10);
    assert_eq!(FsrsPredictor::calculate_urgency(far_future), 0);
}

#[test]
fn test_fsrs_repeated_reviews() {
    let predictor = FsrsPredictor::new().unwrap();

    let (_, state1) = predictor
        .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
        .unwrap();

    let (_, state2) = predictor
        .predict_next_review(Some(state1), MemoryQuality::Hard, 3, 0.9)
        .unwrap();

    let (interval3, _) = predictor
        .predict_next_review(Some(state2), MemoryQuality::Good, 10, 0.9)
        .unwrap();

    assert!(interval3 > 0);
}
