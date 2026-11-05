//! The chess clock. Pure: the caller passes the time, so tests need no sleeping.

use gb_model::{ClockView, Color, TimeControl};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockTick {
    Running,
    Flagged(Color),
}

/// The state to return to when a move is taken back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSnapshot {
    white_ms: u64,
    black_ms: u64,
    running: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct Clock {
    white_ms: u64,
    black_ms: u64,
    increment_ms: u64,
    running: Option<Color>,
    /// When `running` last started to lose time.
    anchor_ms: u64,
}

impl Clock {
    pub fn new(control: TimeControl) -> Self {
        Self {
            white_ms: u64::from(control.base_ms),
            black_ms: u64::from(control.base_ms),
            increment_ms: u64::from(control.increment_ms),
            running: None,
            anchor_ms: 0,
        }
    }

    fn stored_ms(&self, color: Color) -> u64 {
        match color {
            Color::White => self.white_ms,
            Color::Black => self.black_ms,
        }
    }

    fn set_ms(&mut self, color: Color, value: u64) {
        match color {
            Color::White => self.white_ms = value,
            Color::Black => self.black_ms = value,
        }
    }

    pub fn remaining_ms(&self, color: Color, now_ms: u64) -> u64 {
        let stored = self.stored_ms(color);

        if self.running == Some(color) {
            stored.saturating_sub(now_ms.saturating_sub(self.anchor_ms))
        } else {
            stored
        }
    }

    /// The very first move starts the clock: it costs nothing and earns nothing.
    pub fn on_move(&mut self, mover: Color, now_ms: u64) -> ClockTick {
        if self.running.is_none() {
            self.running = Some(mover.opposite());
            self.anchor_ms = now_ms;

            return ClockTick::Running;
        }

        let left = self.remaining_ms(mover, now_ms);

        if left == 0 {
            self.set_ms(mover, 0);
            self.running = None;

            return ClockTick::Flagged(mover);
        }

        self.set_ms(mover, left + self.increment_ms);
        self.running = Some(mover.opposite());
        self.anchor_ms = now_ms;

        ClockTick::Running
    }

    /// Milliseconds until the running side flags, for the timer.
    pub fn ms_to_flag(&self, now_ms: u64) -> Option<(Color, u64)> {
        self.running
            .map(|color| (color, self.remaining_ms(color, now_ms)))
    }

    pub fn stop(&mut self, now_ms: u64) {
        if let Some(color) = self.running {
            self.set_ms(color, self.remaining_ms(color, now_ms));
            self.running = None;
        }
    }

    /// `snapshot` is taken at a moment when a side's stored time is exact.
    pub fn snapshot(&self, now_ms: u64) -> ClockSnapshot {
        ClockSnapshot {
            white_ms: self.remaining_ms(Color::White, now_ms),
            black_ms: self.remaining_ms(Color::Black, now_ms),
            running: self.running,
        }
    }

    pub fn restore(&mut self, snapshot: ClockSnapshot, now_ms: u64) {
        self.white_ms = snapshot.white_ms;
        self.black_ms = snapshot.black_ms;
        self.running = snapshot.running;
        self.anchor_ms = now_ms;
    }

    pub fn view(&self, now_ms: u64) -> ClockView {
        ClockView {
            white_ms: clamp_u32(self.remaining_ms(Color::White, now_ms)),
            black_ms: clamp_u32(self.remaining_ms(Color::Black, now_ms)),
            running: self.running,
            sync_ms: clamp_u32(now_ms),
        }
    }
}

fn clamp_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE_MS: u32 = 60_000;
    const INCREMENT_MS: u32 = 2_000;

    fn clock() -> Clock {
        Clock::new(TimeControl {
            base_ms: MINUTE_MS,
            increment_ms: INCREMENT_MS,
        })
    }

    #[test]
    fn the_first_move_starts_the_clock_for_free() {
        let mut clock = clock();

        assert_eq!(clock.on_move(Color::White, 5_000), ClockTick::Running);
        assert_eq!(clock.remaining_ms(Color::White, 5_000), 60_000);
        assert_eq!(clock.remaining_ms(Color::Black, 6_000), 59_000);
    }

    #[test]
    fn a_move_charges_the_time_and_adds_the_increment() {
        let mut clock = clock();

        clock.on_move(Color::White, 0);
        clock.on_move(Color::Black, 10_000);

        assert_eq!(clock.remaining_ms(Color::Black, 10_000), 52_000);
        assert_eq!(clock.remaining_ms(Color::White, 13_000), 57_000);
    }

    #[test]
    fn a_move_after_the_time_ran_out_flags_and_earns_nothing() {
        let mut clock = clock();

        clock.on_move(Color::White, 0);

        assert_eq!(
            clock.on_move(Color::Black, 70_000),
            ClockTick::Flagged(Color::Black)
        );
        assert_eq!(clock.remaining_ms(Color::Black, 70_000), 0);
    }

    #[test]
    fn ms_to_flag_names_the_running_side() {
        let mut clock = clock();

        assert_eq!(clock.ms_to_flag(0), None);

        clock.on_move(Color::White, 0);

        assert_eq!(clock.ms_to_flag(1_000), Some((Color::Black, 59_000)));
    }

    #[test]
    fn restore_returns_the_snapshot_and_the_clock_runs_from_now() {
        let mut clock = clock();

        clock.on_move(Color::White, 0);

        let before = clock.snapshot(0);

        clock.on_move(Color::Black, 30_000);
        clock.restore(before, 40_000);

        assert_eq!(clock.remaining_ms(Color::Black, 40_000), 60_000);
        assert_eq!(clock.remaining_ms(Color::Black, 41_000), 59_000);
    }

    #[test]
    fn stop_freezes_the_running_side() {
        let mut clock = clock();

        clock.on_move(Color::White, 0);
        clock.stop(1_000);

        assert_eq!(clock.remaining_ms(Color::Black, 9_000), 59_000);
        assert_eq!(clock.ms_to_flag(9_000), None);
    }
}
