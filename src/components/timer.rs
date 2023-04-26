use instant::Duration;

#[derive(Clone, Debug, Default)]
pub struct Timer {
    elapsed: Duration,
    paused: bool,
    duration: Duration,
    mode: TimerMode,
    finished: bool,
    times_finished_this_tick: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum TimerMode {
    /// Run once and stop.
    #[default]
    Once,
    /// Reset when finished.
    Repeating,
}

impl Timer {
    pub fn tick(&mut self, delta: Duration) -> &Self {
        if self.paused {
            self.times_finished_this_tick = 0;
            if self.mode == TimerMode::Repeating {
                self.finished = false;
            }
            return self;
        }

        if self.mode != TimerMode::Repeating && self.finished {
            self.times_finished_this_tick = 0;
            return self;
        }

        self.elapsed += delta;
        self.finished = self.elapsed >= self.duration;

        if self.finished {
            if self.mode == TimerMode::Repeating {
                self.times_finished_this_tick =
                    (self.elapsed.as_nanos() / self.duration.as_nanos()) as u32;
                // Duration does not have a modulo
                self.elapsed -= self.duration * self.times_finished_this_tick;
            } else {
                self.times_finished_this_tick = 1;
                self.elapsed = self.duration;
            }
        } else {
            self.times_finished_this_tick = 0;
        }

        self
    }

    pub fn just_finished(&self) -> bool {
        self.times_finished_this_tick > 0
    }

    pub fn from_seconds(duration: f32, mode: TimerMode) -> Self {
        Self {
            duration: Duration::from_secs_f32(duration),
            mode,
            ..Default::default()
        }
    }
}
