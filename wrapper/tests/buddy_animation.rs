use buddy_wrapper::buddy::animation::{
    BuddyAnimation, BuddyAnimationMode, IDLE_QUIET_DELAY, IDLE_STEP_DURATION, PET_STEP_DURATION,
};
use std::time::{Duration, Instant};

#[test]
fn idle_animation_stays_resting_until_quiet_delay_expires() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);

    animation.tick(now + IDLE_QUIET_DELAY - Duration::from_millis(1));
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);

    animation.tick(now + IDLE_QUIET_DELAY);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);
}

#[test]
fn idle_animation_advances_through_burst_and_returns_to_rest() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    animation.tick(now + IDLE_QUIET_DELAY);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(now + IDLE_QUIET_DELAY + IDLE_STEP_DURATION);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 2);

    animation.tick(now + IDLE_QUIET_DELAY + IDLE_STEP_DURATION * 2);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(now + IDLE_QUIET_DELAY + IDLE_STEP_DURATION * 3);
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);
}

#[test]
fn delayed_first_idle_tick_still_starts_on_frame_one() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    animation.tick(now + IDLE_QUIET_DELAY + IDLE_STEP_DURATION * 3);

    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);
}

#[test]
fn pet_animation_starts_immediately_and_overrides_idle() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    animation.tick(now + IDLE_QUIET_DELAY);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    animation.start_pet(now + IDLE_QUIET_DELAY + Duration::from_millis(10));
    assert_eq!(animation.mode(), BuddyAnimationMode::Pet);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(now + IDLE_QUIET_DELAY + Duration::from_millis(10) + PET_STEP_DURATION);
    assert_eq!(animation.mode(), BuddyAnimationMode::Pet);
    assert_eq!(animation.current_frame(), 2);

    animation.tick(now + IDLE_QUIET_DELAY + Duration::from_millis(10) + PET_STEP_DURATION * 2);
    assert_eq!(animation.mode(), BuddyAnimationMode::Pet);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(now + IDLE_QUIET_DELAY + Duration::from_millis(10) + PET_STEP_DURATION * 3);
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);
}

#[test]
fn reset_returns_to_rest_and_reschedules_idle_animation() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    animation.tick(now + IDLE_QUIET_DELAY);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    let reset_at = now + IDLE_QUIET_DELAY + Duration::from_millis(25);
    animation.reset(reset_at);

    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);

    animation.tick(reset_at + IDLE_QUIET_DELAY - Duration::from_millis(1));
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);

    animation.tick(reset_at + IDLE_QUIET_DELAY);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);
}
