use serde::{Deserialize, Serialize};

/// Deterministic fixed-timestep accumulator (60 Hz default simulation rate).
#[derive(Debug, Clone)]
pub struct FixedTimestep {
    pub fixed_dt: f32,
    accumulator: f32,
    max_frame_dt: f32,
    pub total_ticks: u64,
    pub total_sim_time: f64,
}

impl Default for FixedTimestep {
    fn default() -> Self {
        Self::new(1.0 / 60.0)
    }
}

impl FixedTimestep {
    pub fn new(fixed_dt: f32) -> Self {
        Self {
            fixed_dt: fixed_dt.max(1.0 / 240.0),
            accumulator: 0.0,
            max_frame_dt: 0.1,
            total_ticks: 0,
            total_sim_time: 0.0,
        }
    }

    /// Accumulate elapsed wall-clock seconds and return how many simulation ticks to run.
    pub fn accumulate(&mut self, frame_dt: f32) -> u32 {
        let clamped = frame_dt.clamp(0.0, self.max_frame_dt);
        self.accumulator += clamped;
        let mut steps = 0u32;
        while self.accumulator >= self.fixed_dt && steps < 8 {
            self.accumulator -= self.fixed_dt;
            self.total_ticks += 1;
            self.total_sim_time += f64::from(self.fixed_dt);
            steps += 1;
        }
        steps
    }

    /// Interpolation factor `[0.0, 1.0)` for smooth rendering between ticks.
    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.fixed_dt).clamp(0.0, 1.0)
    }
}

/// San Andreas 24-hour in-game clock (1 real-world second = 1 in-game minute by default).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameClock {
    pub hour: u8,
    pub minute: u8,
    pub second_fraction: f32,
    pub day: u32,
    /// In-game minutes elapsed per real-world second (original SA = 1.0: 24 min = 1 day).
    pub minutes_per_real_second: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        // GTA:SA starts at 08:00 AM on a sunny day after CJ's intro.
        Self {
            hour: 8,
            minute: 0,
            second_fraction: 0.0,
            day: 1,
            minutes_per_real_second: 1.0,
        }
    }
}

impl GameClock {
    pub fn advance(&mut self, dt_seconds: f32) {
        self.second_fraction += dt_seconds * self.minutes_per_real_second;
        while self.second_fraction >= 1.0 {
            self.second_fraction -= 1.0;
            self.minute += 1;
            if self.minute >= 60 {
                self.minute = 0;
                self.hour += 1;
                if self.hour >= 24 {
                    self.hour = 0;
                    self.day += 1;
                }
            }
        }
    }

    /// Continuous time of day in `[0.0, 24.0)` hours for timecycle interpolation.
    pub fn hours_f32(&self) -> f32 {
        f32::from(self.hour) + (f32::from(self.minute) + self.second_fraction) / 60.0
    }

    /// Formatted `HH:MM` HUD string matching original SA display.
    pub fn hud_string(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_clock_wraps_midnight() {
        let mut clk = GameClock {
            hour: 23,
            minute: 59,
            second_fraction: 0.5,
            day: 1,
            minutes_per_real_second: 1.0,
        };
        clk.advance(1.0);
        assert_eq!(clk.hour, 0);
        assert_eq!(clk.minute, 0);
        assert_eq!(clk.day, 2);
    }

    #[test]
    fn fixed_timestep_clamps_huge_spikes() {
        let mut ts = FixedTimestep::default();
        let steps = ts.accumulate(5.0);
        assert!(steps <= 8);
        assert_eq!(steps, 6); // 0.1s clamped / (1/60) = 6 ticks
    }
}
