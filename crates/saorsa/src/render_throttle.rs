//! Frame-rate limiter for TUI rendering.
//!
//! Prevents excessive rendering by enforcing a minimum interval between frames.
//! Default is 30 FPS (~33ms between renders), which balances smooth updates
//! with reduced CPU usage during high-frequency streaming.

use std::time::{Duration, Instant};

/// Default target frame rate in frames per second.
const DEFAULT_FPS: u32 = 30;

/// Frame-rate limiter that tracks render timing.
///
/// Use this to cap the number of UI renders per second. During AI streaming,
/// text deltas arrive at high frequency (10-50+ per second). Without throttling,
/// each delta triggers a full re-render, causing input lag.
///
/// # Example
///
/// ```
/// use saorsa::render_throttle::RenderThrottle;
///
/// let mut throttle = RenderThrottle::default_fps();
///
/// // First render is always allowed
/// assert!(throttle.should_render());
/// throttle.mark_rendered();
///
/// // Immediately after, rendering is blocked
/// assert!(!throttle.should_render());
/// ```
pub struct RenderThrottle {
    /// Minimum duration between consecutive renders.
    frame_duration: Duration,
    /// When the last render occurred.
    last_render: Instant,
}

impl RenderThrottle {
    /// Create a new throttle with the given FPS limit.
    ///
    /// # Arguments
    ///
    /// * `fps` - Target frame rate. Clamped to 1..=240. A value of 30
    ///   yields ~33ms between frames.
    pub fn new(fps: u32) -> Self {
        let clamped = fps.clamp(1, 240);
        Self {
            frame_duration: Duration::from_micros(1_000_000 / u64::from(clamped)),
            // Set last_render far enough in the past that the first render is allowed.
            last_render: Instant::now() - Duration::from_secs(1),
        }
    }

    /// Create a 30 FPS throttle (the default for interactive use).
    pub fn default_fps() -> Self {
        Self::new(DEFAULT_FPS)
    }

    /// Check whether enough time has passed since the last render.
    ///
    /// Returns `true` if the frame interval has elapsed and a render should
    /// proceed.
    pub fn should_render(&self) -> bool {
        self.last_render.elapsed() >= self.frame_duration
    }

    /// Record that a render just occurred, resetting the frame timer.
    pub fn mark_rendered(&mut self) {
        self.last_render = Instant::now();
    }

    /// Return the time remaining until the next render is allowed.
    ///
    /// Returns [`Duration::ZERO`] if a render is already permitted.
    pub fn time_until_next_frame(&self) -> Duration {
        self.frame_duration
            .saturating_sub(self.last_render.elapsed())
    }

    /// Return the configured frame duration.
    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn new_creates_correct_frame_duration() {
        let throttle = RenderThrottle::new(60);
        // 1_000_000 / 60 = 16_666 microseconds
        assert_eq!(throttle.frame_duration, Duration::from_micros(16_666));
    }

    #[test]
    fn default_fps_is_30() {
        let throttle = RenderThrottle::default_fps();
        // 1_000_000 / 30 = 33_333 microseconds
        assert_eq!(throttle.frame_duration, Duration::from_micros(33_333));
    }

    #[test]
    fn should_render_true_immediately_after_creation() {
        let throttle = RenderThrottle::default_fps();
        assert!(throttle.should_render());
    }

    #[test]
    fn should_render_false_immediately_after_mark() {
        let mut throttle = RenderThrottle::default_fps();
        throttle.mark_rendered();
        assert!(!throttle.should_render());
    }

    #[test]
    fn should_render_true_after_frame_duration() {
        // 240fps (max) = ~4.16ms per frame. Sleep 6ms to be safe.
        let mut throttle = RenderThrottle::new(240);
        throttle.mark_rendered();
        assert!(!throttle.should_render());
        thread::sleep(Duration::from_millis(6));
        assert!(throttle.should_render());
    }

    #[test]
    fn time_until_next_frame_zero_when_ready() {
        let throttle = RenderThrottle::default_fps();
        assert_eq!(throttle.time_until_next_frame(), Duration::ZERO);
    }

    #[test]
    fn time_until_next_frame_nonzero_after_render() {
        let mut throttle = RenderThrottle::default_fps();
        throttle.mark_rendered();
        let remaining = throttle.time_until_next_frame();
        assert!(remaining > Duration::ZERO);
        assert!(remaining <= throttle.frame_duration);
    }

    #[test]
    fn fps_clamped_to_minimum_1() {
        let throttle = RenderThrottle::new(0);
        assert_eq!(throttle.frame_duration, Duration::from_micros(1_000_000));
    }

    #[test]
    fn fps_clamped_to_maximum_240() {
        let throttle = RenderThrottle::new(1000);
        let expected = Duration::from_micros(1_000_000 / 240);
        assert_eq!(throttle.frame_duration, expected);
    }

    #[test]
    fn fps_1_creates_1_second_frame_duration() {
        let throttle = RenderThrottle::new(1);
        assert_eq!(throttle.frame_duration, Duration::from_micros(1_000_000));
    }

    #[test]
    fn frame_duration_accessor() {
        let throttle = RenderThrottle::new(60);
        assert_eq!(throttle.frame_duration(), Duration::from_micros(16_666));
    }
}
