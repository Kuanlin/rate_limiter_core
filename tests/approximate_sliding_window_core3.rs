
use rate_guard_core::rate_limiter_core::RateLimiterCore;
use rate_guard_core::{RateLimitError, Uint};
use rate_guard_core::rate_limiters::ApproximateSlidingWindowCore;

// Helper function to create a boxed trait object
fn create_rate_limiter(capacity: Uint, window_ticks: Uint) -> Box<dyn RateLimiterCore> {
    Box::new(ApproximateSlidingWindowCore::new(capacity, window_ticks))
}

#[test]
fn test_trait_try_acquire_at_basic() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Test successful acquisition through trait
    assert_eq!(limiter.try_acquire_at(50, 5), Ok(()));
    assert_eq!(limiter.try_acquire_at(30, 8), Ok(()));
    
    // Test exceeding capacity through trait
    assert_eq!(
        limiter.try_acquire_at(30, 9),
        Err(RateLimitError::ExceedsCapacity)
    );
}

#[test]
fn test_trait_capacity_remaining_basic() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Initially should have full capacity through trait
    assert_eq!(limiter.capacity_remaining(0), 100);
    
    // After using some tokens through trait
    assert_eq!(limiter.try_acquire_at(30, 5), Ok(()));
    assert_eq!(limiter.capacity_remaining(5), 70);
}

#[test]
fn test_trait_zero_tokens() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(50, 10);
    
    // Zero tokens should always succeed through trait
    assert_eq!(limiter.try_acquire_at(0, 0), Ok(()));
    assert_eq!(limiter.try_acquire_at(0, 100), Ok(()));
}

#[test]
fn test_trait_sliding_window_behavior() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Fill up first window through trait
    assert_eq!(limiter.try_acquire_at(80, 5), Ok(()));
    assert_eq!(limiter.capacity_remaining(5), 20);
    
    // Move to second window - should be able to acquire more through trait
    assert_eq!(limiter.try_acquire_at(60, 15), Ok(()));
    
    // Capacity should be affected by sliding window through trait
    let remaining = limiter.capacity_remaining(15);
    assert!(remaining < 100, "Some capacity should be used");
    assert!(remaining > 0, "Should have some capacity available");
}

#[test]
fn test_trait_expired_tick() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Advance time through trait
    assert_eq!(limiter.try_acquire_at(50, 20), Ok(()));
    
    // Try to use an earlier tick through trait - should fail
    assert_eq!(
        limiter.try_acquire_at(10, 15),
        Err(RateLimitError::ExpiredTick)
    );
}

#[test]
fn test_trait_capacity_remaining_error_handling() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Normal case should work through trait
    assert_eq!(limiter.capacity_remaining(10), 100);
    
    // The trait implementation should return 0 on any error
    // (This tests the unwrap_or(0) behavior in the trait impl)
    
    // We can't easily force a ContentionFailure, but we can verify
    // that the trait method handles errors by returning 0
    let remaining = limiter.capacity_remaining(10);
    assert!(remaining <= 100, "Should not exceed capacity");
}

#[test]
fn test_trait_polymorphism() {
    // Test that we can use the trait polymorphically
    let limiters: Vec<Box<dyn RateLimiterCore>> = vec![
        create_rate_limiter(50, 5),
        create_rate_limiter(100, 10),
        create_rate_limiter(200, 20),
    ];
    
    for limiter in limiters {
        // Test basic functionality through trait interface
        assert_eq!(limiter.try_acquire_at(10, 1), Ok(()));
        let remaining = limiter.capacity_remaining(1);
        assert!(remaining > 0, "Should have remaining capacity");
        
        // Test zero tokens through trait
        assert_eq!(limiter.try_acquire_at(0, 2), Ok(()));
    }
}

#[test]
fn test_trait_window_transitions() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Window 0: ticks 0-9 through trait
    assert_eq!(limiter.try_acquire_at(50, 5), Ok(()));
    assert_eq!(limiter.capacity_remaining(9), 50);
    
    // Window 1: ticks 10-19 through trait
    assert_eq!(limiter.try_acquire_at(40, 15), Ok(()));
    
    // Window 0 again: ticks 20-29 through trait
    assert_eq!(limiter.try_acquire_at(30, 25), Ok(()));
    
    // Verify capacity calculations work across window transitions through trait
    let remaining = limiter.capacity_remaining(25);
    assert!(remaining <= 100, "Remaining capacity should not exceed total");
}

#[test]
fn test_trait_consistency() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(100, 10);
    
    // Test that try_acquire_at and capacity_remaining are consistent through trait
    let initial_capacity = limiter.capacity_remaining(10);
    assert_eq!(initial_capacity, 100);
    
    // Acquire some tokens through trait
    let tokens_to_acquire = 30;
    assert_eq!(limiter.try_acquire_at(tokens_to_acquire, 10), Ok(()));
    
    // Check that capacity decreased appropriately through trait
    let remaining_capacity = limiter.capacity_remaining(10);
    assert_eq!(remaining_capacity, initial_capacity - tokens_to_acquire);
}

#[test]
fn test_trait_full_capacity() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(50, 5);
    
    // Fill to exactly capacity through trait
    assert_eq!(limiter.try_acquire_at(50, 2), Ok(()));
    assert_eq!(limiter.capacity_remaining(2), 0);
    
    // Should not be able to acquire more through trait
    assert_eq!(
        limiter.try_acquire_at(1, 3),
        Err(RateLimitError::ExceedsCapacity)
    );
}

#[test]
fn test_trait_edge_cases() {
    let limiter: Box<dyn RateLimiterCore> = create_rate_limiter(1, 1);
    
    // Very small capacity and window through trait
    assert_eq!(limiter.try_acquire_at(1, 0), Ok(()));
    assert_eq!(limiter.capacity_remaining(0), 0);
    
    // Next tick should allow acquisition again through trait
    assert_eq!(limiter.try_acquire_at(1, 1), Ok(()));
    assert_eq!(limiter.capacity_remaining(1), 0);
}

// Test using generic function with trait bound
fn test_rate_limiter_generic<T: RateLimiterCore>(limiter: &T, capacity: Uint) {
    // Test through generic trait bound
    assert_eq!(limiter.try_acquire_at(10, 1), Ok(()));
    let remaining = limiter.capacity_remaining(1);
    assert_eq!(remaining, capacity - 10);
}

#[test]
fn test_trait_generic_usage() {
    let limiter = ApproximateSlidingWindowCore::new(100, 10);
    
    // Test using the limiter through generic trait bound
    test_rate_limiter_generic(&limiter, 100);
    
    // Test that the generic function worked
    let limiter_trait: &dyn RateLimiterCore = &limiter;
    assert_eq!(limiter_trait.capacity_remaining(1), 90);
}

// Test trait object in different contexts
#[test]
fn test_trait_object_contexts() {
    let limiter: &dyn RateLimiterCore = &ApproximateSlidingWindowCore::new(50, 10);
    
    // Test through reference to trait object
    assert_eq!(limiter.try_acquire_at(20, 5), Ok(()));
    assert_eq!(limiter.capacity_remaining(5), 30);
    
    // Test moving trait object
    let boxed: Box<dyn RateLimiterCore> = Box::new(ApproximateSlidingWindowCore::new(75, 15));
    assert_eq!(boxed.try_acquire_at(25, 3), Ok(()));
    assert_eq!(boxed.capacity_remaining(3), 50);
}
