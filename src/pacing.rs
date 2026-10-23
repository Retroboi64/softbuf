//! High-precision frame pacing, delta time tracking, adaptive sleep, and framerate management.

use std::collections::VecDeque;
use std::hint::spin_loop;
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

/// Target framerate configuration for the frame pacer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Framerate {
    /// Renders as fast as possible without frame time capping.
    Uncapped,
    /// Caps the render loop to a constant target framerate (e.g. 30.0, 60.0, 120.0, 144.0).
    Fps(f64),
    /// Oscillates dynamically across a range of framerates for stress-testing or variable refresh simulation.
    Ranging {
        /// Lower bound FPS.
        min_fps: f64,
        /// Upper bound FPS.
        max_fps: f64,
        /// Time in seconds for one full cycle between min and max FPS.
        cycle_secs: f64,
    },
}

impl Default for Framerate {
    fn default() -> Self {
        Framerate::Fps(60.0)
    }
}

/// Real-time timing metrics and diagnostics for the current frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStats {
    /// Time elapsed between the start of the previous frame and this frame.
    pub delta_time: Duration,
    /// Time delta expressed as fractional seconds (`dt` for physics/animation).
    pub delta_secs: f32,
    /// Instantaneous framerate for this specific frame.
    pub fps: f64,
    /// Windowed exponential moving average framerate.
    pub avg_fps: f64,
    /// Frame duration in milliseconds.
    pub frame_time_ms: f64,
    /// Monotonically increasing frame counter.
    pub frame_count: u64,
}

/// A high-precision frame pacer and timing governor.
///
/// Uses a hybrid OS sleep + spin-yield strategy to prevent timer jitter and achieve
/// sub-millisecond precision frame pacing on Windows, macOS, and Linux without pinning CPU cores.
pub struct FramePacer {
    target: Framerate,
    last_frame_start: Instant,
    last_target_time: Instant,
    start_time: Instant,
    frame_count: u64,
    history: VecDeque<f32>, // recent frame times in ms
    history_capacity: usize,
    avg_fps: f64,
}

impl FramePacer {
    /// Creates a frame pacer capped at `target_fps` (e.g. 60.0).
    pub fn new(target_fps: f64) -> Self {
        Self::with_target(Framerate::Fps(target_fps))
    }

    /// Creates an uncapped frame pacer.
    pub fn uncapped() -> Self {
        Self::with_target(Framerate::Uncapped)
    }

    /// Creates a frame pacer with the specified framerate mode.
    pub fn with_target(target: Framerate) -> Self {
        let now = Instant::now();
        Self {
            target,
            last_frame_start: now,
            last_target_time: now,
            start_time: now,
            frame_count: 0,
            history: VecDeque::with_capacity(120),
            history_capacity: 120,
            avg_fps: 60.0,
        }
    }

    /// Sets the target framerate mode.
    pub fn set_target(&mut self, target: Framerate) {
        self.target = target;
    }

    /// Gets the current target framerate mode.
    pub fn target(&self) -> Framerate {
        self.target
    }

    /// Returns the active effective target FPS, or `None` if uncapped.
    pub fn effective_target_fps(&self) -> Option<f64> {
        match self.target {
            Framerate::Uncapped => None,
            Framerate::Fps(fps) => Some(fps.max(1.0)),
            Framerate::Ranging {
                min_fps,
                max_fps,
                cycle_secs,
            } => {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let period = cycle_secs.max(0.1);
                // Normalized sine oscillation [0.0, 1.0]
                let t = 0.5 + 0.5 * (elapsed * (2.0 * std::f64::consts::PI / period)).sin();
                Some(min_fps + (max_fps - min_fps) * t)
            }
        }
    }

    /// Advances to the next frame, sleeping for the required duration to pace the frame,
    /// and returns the timing statistics for the new frame.
    pub fn tick(&mut self) -> FrameStats {
        // 1. Calculate required delay to hit target framerate
        if let Some(target_fps) = self.effective_target_fps() {
            let target_frame_duration = Duration::from_secs_f64(1.0 / target_fps);
            let next_target = self.last_frame_start + target_frame_duration;

            // Hybrid sleep: sleep for the bulk of the time, then yield/spin for sub-ms accuracy
            precise_sleep_until(next_target);
        }

        let frame_start = Instant::now();
        let delta = frame_start.duration_since(self.last_frame_start);
        self.last_frame_start = frame_start;
        self.frame_count = self.frame_count.saturating_add(1);

        let delta_secs = delta.as_secs_f32().max(1e-6);
        let frame_time_ms = delta.as_secs_f64() * 1000.0;
        let fps = 1.0 / delta.as_secs_f64().max(1e-6);

        // Update moving average (alpha = 0.1)
        self.avg_fps = self.avg_fps * 0.9 + fps * 0.1;

        // Record history
        if self.history.len() >= self.history_capacity {
            self.history.pop_front();
        }
        self.history.push_back(frame_time_ms as f32);

        FrameStats {
            delta_time: delta,
            delta_secs,
            fps,
            avg_fps: self.avg_fps,
            frame_time_ms,
            frame_count: self.frame_count,
        }
    }

    /// Returns a slice of recent frame times in milliseconds (useful for graphs).
    pub fn history(&self) -> &VecDeque<f32> {
        &self.history
    }

    /// Clears the recorded timing history.
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.last_frame_start = now;
        self.last_target_time = now;
        self.start_time = now;
        self.history.clear();
    }
}

/// Sleeps until `target_instant` using a hybrid OS-sleep + spin-wait strategy.
#[inline]
fn precise_sleep_until(target_instant: Instant) {
    let mut now = Instant::now();
    if now >= target_instant {
        return;
    }

    let mut remaining = target_instant - now;

    // If remaining time is greater than 3ms, perform coarse OS sleep for all but 2ms
    if remaining > Duration::from_millis(3) {
        sleep(remaining - Duration::from_millis(2));
        now = Instant::now();
        if now >= target_instant {
            return;
        }
        remaining = target_instant - now;
    }

    // Yield CPU time if between 0.5ms and 2ms
    while remaining > Duration::from_micros(500) {
        yield_now();
        now = Instant::now();
        if now >= target_instant {
            return;
        }
        remaining = target_instant - now;
    }

    // Spin-wait for the final fraction of a millisecond to achieve sub-millisecond precision
    while Instant::now() < target_instant {
        spin_loop();
    }
}
