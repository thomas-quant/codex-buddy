use std::time::{Duration, Instant};

pub const IDLE_QUIET_DELAY: Duration = Duration::from_secs(8);
pub const IDLE_STEP_DURATION: Duration = Duration::from_millis(180);
pub const PET_STEP_DURATION: Duration = Duration::from_millis(90);

const BURST_FRAMES: [usize; 4] = [1, 2, 1, 0];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuddyAnimationMode {
    None,
    Idle,
    Pet,
}

#[derive(Clone, Debug)]
pub struct BuddyAnimation {
    current_frame: usize,
    mode: BuddyAnimationMode,
    step_index: usize,
    step_started_at: Instant,
    next_idle_at: Instant,
}

impl BuddyAnimation {
    pub fn new(now: Instant) -> Self {
        Self {
            current_frame: 0,
            mode: BuddyAnimationMode::None,
            step_index: 0,
            step_started_at: now,
            next_idle_at: now + IDLE_QUIET_DELAY,
        }
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn mode(&self) -> BuddyAnimationMode {
        self.mode
    }

    pub fn reset(&mut self, now: Instant) {
        *self = Self::new(now);
    }

    pub fn start_pet(&mut self, now: Instant) {
        self.mode = BuddyAnimationMode::Pet;
        self.current_frame = BURST_FRAMES[0];
        self.step_index = 0;
        self.step_started_at = now;
    }

    pub fn tick(&mut self, now: Instant) {
        if self.mode == BuddyAnimationMode::None {
            if now < self.next_idle_at {
                return;
            }

            self.mode = BuddyAnimationMode::Idle;
            self.current_frame = BURST_FRAMES[0];
            self.step_index = 0;
            self.step_started_at = now;
            return;
        }

        let step_duration = match self.mode {
            BuddyAnimationMode::Idle => IDLE_STEP_DURATION,
            BuddyAnimationMode::Pet => PET_STEP_DURATION,
            BuddyAnimationMode::None => return,
        };

        while now >= self.step_started_at + step_duration {
            if self.step_index + 1 >= BURST_FRAMES.len() {
                self.mode = BuddyAnimationMode::None;
                self.current_frame = 0;
                self.step_index = 0;
                self.step_started_at = now;
                self.next_idle_at = now + IDLE_QUIET_DELAY;
                return;
            }

            self.step_index += 1;
            self.current_frame = BURST_FRAMES[self.step_index];
            self.step_started_at += step_duration;

            if self.current_frame == 0 {
                self.mode = BuddyAnimationMode::None;
                self.step_index = 0;
                self.step_started_at = now;
                self.next_idle_at = now + IDLE_QUIET_DELAY;
                return;
            }
        }
    }
}
