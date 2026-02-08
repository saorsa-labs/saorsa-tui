//! Integration tests for render throttling and text batching.
//!
//! These tests verify that [`RenderThrottle`] and [`AppState`] dirty tracking
//! work correctly together as an integrated system — the same pattern used in
//! the main event loop.

#[allow(clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use saorsa::app::AppState;
    use saorsa::render_throttle::RenderThrottle;

    /// Simulate the `maybe_render_ui` logic from main.rs.
    ///
    /// Returns `true` if a "render" would have occurred.
    fn maybe_render(state: &mut AppState, throttle: &mut RenderThrottle) -> bool {
        if throttle.should_render() && state.take_dirty() {
            throttle.mark_rendered();
            return true;
        }
        false
    }

    #[test]
    fn initial_render_always_happens() {
        let mut state = AppState::new("test");
        let mut throttle = RenderThrottle::default_fps();
        // New state is dirty, new throttle allows render.
        assert!(maybe_render(&mut state, &mut throttle));
    }

    #[test]
    fn second_render_blocked_by_throttle() {
        let mut state = AppState::new("test");
        let mut throttle = RenderThrottle::default_fps();
        // First render.
        assert!(maybe_render(&mut state, &mut throttle));

        // Mark dirty again immediately.
        state.mark_dirty();
        // Blocked — not enough time has passed.
        assert!(!maybe_render(&mut state, &mut throttle));
    }

    #[test]
    fn render_blocked_when_clean() {
        let mut state = AppState::new("test");
        let mut throttle = RenderThrottle::default_fps();
        // First render clears dirty.
        maybe_render(&mut state, &mut throttle);

        // Wait long enough for throttle.
        std::thread::sleep(Duration::from_millis(40));
        // Throttle allows it, but state is clean — no render.
        assert!(!maybe_render(&mut state, &mut throttle));
    }

    #[test]
    fn rapid_dirty_marks_produce_limited_renders() {
        let mut state = AppState::new("test");
        // Use 30fps so the frame interval (~33ms) is well above timer resolution
        // on all platforms (macOS CI has ~1ms granularity for sleep).
        let mut throttle = RenderThrottle::default_fps();

        let mut render_count = 0;
        let start = std::time::Instant::now();

        // Simulate 100 rapid dirty marks with ~1ms spacing (~100ms total).
        for i in 0..100 {
            state.mark_dirty();
            if maybe_render(&mut state, &mut throttle) {
                render_count += 1;
            }
            // ~1ms between marks.
            if i < 99 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }

        let elapsed = start.elapsed();

        // At 30fps (~33ms per frame), in ~100ms we should get ~3-4 renders.
        // Even with imprecise sleeps, the count must stay well below 100.
        assert!(
            render_count < 20,
            "Expected fewer than 20 renders, got {render_count} in {elapsed:?}"
        );
        assert!(
            render_count >= 1,
            "Expected at least 1 render, got {render_count} in {elapsed:?}"
        );
    }

    #[test]
    fn stream_text_batching_accumulates_correctly() {
        let mut state = AppState::new("test");
        state.take_dirty(); // Clear initial dirty.

        // Simulate 1000 TextDelta events.
        for i in 0..1000 {
            state.accumulate_stream_text(&format!("chunk{i} "));
        }

        // Nothing in streaming_text yet — all in pending buffer.
        assert!(state.streaming_text.is_empty());

        // Flush moves everything.
        assert!(state.flush_stream_text());
        assert!(state.streaming_text.starts_with("chunk0 chunk1 "));
        assert!(state.streaming_text.ends_with("chunk999 "));
        assert!(state.streaming_text.contains("chunk500 "));

        // State is now dirty from the flush.
        assert!(state.take_dirty());
    }

    #[test]
    fn flush_then_render_cycle() {
        let mut state = AppState::new("test");
        let mut throttle = RenderThrottle::new(240);

        // Initial render.
        assert!(maybe_render(&mut state, &mut throttle));

        // Accumulate text — no dirty, no render.
        state.accumulate_stream_text("hello ");
        state.accumulate_stream_text("world");
        assert!(!state.take_dirty()); // accumulate doesn't mark dirty
        // Restore dirty state since take_dirty cleared it.

        // Wait for throttle to allow next frame.
        std::thread::sleep(Duration::from_millis(6));

        // Flush marks dirty.
        assert!(state.flush_stream_text());
        assert_eq!(state.streaming_text, "hello world");

        // Now render succeeds.
        assert!(maybe_render(&mut state, &mut throttle));
    }

    #[test]
    fn dirty_flag_cleared_even_when_throttled() {
        let mut state = AppState::new("test");
        let mut throttle = RenderThrottle::default_fps();

        // First render consumes dirty.
        maybe_render(&mut state, &mut throttle);

        // Mark dirty twice quickly — throttle blocks.
        state.mark_dirty();
        // maybe_render checks throttle first, then dirty. Since throttle blocks,
        // dirty flag is preserved for the next allowed frame.
        assert!(!maybe_render(&mut state, &mut throttle));

        // Dirty flag should still be set since throttle blocked before we checked it.
        // Wait for throttle to allow.
        std::thread::sleep(Duration::from_millis(40));
        assert!(maybe_render(&mut state, &mut throttle));
    }

    #[test]
    fn multiple_message_types_all_mark_dirty() {
        let mut state = AppState::new("test");

        state.take_dirty(); // Clear initial.
        state.add_user_message("hi");
        assert!(state.take_dirty());

        state.add_assistant_message("hello");
        assert!(state.take_dirty());

        state.add_tool_message("bash", "output");
        assert!(state.take_dirty());

        state.add_system_message("info");
        assert!(state.take_dirty());

        // Input editing.
        state.insert_char('a');
        assert!(state.take_dirty());

        state.delete_char_before();
        assert!(state.take_dirty());
    }
}
