//! `KeyRepeatGate<K>` — debouncer for OS key-repeat storms.
//!
//! # The class of bug this prevents
//!
//! GUI shells (NSEvent on macOS, X11 KeyPress, Wayland keyboard,
//! winit's `KeyboardInput`) deliver one `KeyDown` per OS key-repeat
//! tick when a key is held. That's typically 30-50ms between ticks.
//! A naive handler that mutates state on every tick produces runaway
//! behaviour:
//!
//! - **mado 2026-05-21**: operator held Cmd-=; 25 `FontIncrease`
//!   events fired in 1.5s; font grew 14 → 32pt onscreen; "screen
//!   froze" (was actually keeping up, just looked wrong).
//! - **Same shape applies to**: navigation arrows in editors,
//!   tab-cycle keys in window managers, zoom-in/out in image
//!   viewers, scroll-page in PDFs.
//!
//! The fix is temporal: only let ONE event per `min_interval` window
//! per key actually fire. The rest get dropped at the dispatcher.
//!
//! # API
//!
//! ```rust,ignore
//! use awase::repeat_gate::{KeyRepeatGate, DEFAULT_MIN_INTERVAL};
//!
//! let mut gate = KeyRepeatGate::<MyAction>::new();
//!
//! // In the event handler:
//! if !gate.try_pass(action) {
//!     // OS key-repeat storm tick; drop and return.
//!     return EventResponse::consumed();
//! }
//! // Otherwise process the action normally.
//! ```
//!
//! `K` is any `Eq + Hash + Copy` token — typically an enum
//! representing the action being dispatched (`Action::FontIncrease`,
//! `Action::FontDecrease`, etc.). Each `K` has its own independent
//! clock — holding Cmd-= doesn't block Cmd-C.
//!
//! # Choosing `min_interval`
//!
//! - **80ms (default)**: filters OS key-repeat (30-50ms) while
//!   allowing up to 12 intentional presses/sec — well above human
//!   cadence.
//! - **120-150ms**: more aggressive; useful when the underlying
//!   action is expensive (re-render, network call).
//! - **30-50ms**: more permissive; useful for cursor movement
//!   where smooth-feeling repeat IS the desired behaviour.
//! - **`Duration::ZERO`**: pass-through (no debouncing) — useful in
//!   tests or for actions that should always fire.
//!
//! # Historical
//!
//! Extracted from `mado/src/key_repeat_gate.rs` (2026-05-21,
//! mado@68d74c0). Same code, now reusable across the 16 GPU apps
//! that already depend on awase.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Default minimum interval between accepted events for the same
/// key. 80ms = filters OS key-repeat (~30-50ms intervals) while
/// allowing 12 intentional presses/sec.
pub const DEFAULT_MIN_INTERVAL: Duration = Duration::from_millis(80);

/// The debouncer. Generic over the key type so consumers can use
/// `Action`, `&'static str`, or any other Eq+Hash+Copy token.
///
/// `Default::default()` gives a gate with the 80ms `DEFAULT_MIN_INTERVAL`.
#[derive(Debug)]
pub struct KeyRepeatGate<K: Eq + Hash + Copy> {
    last_pass: HashMap<K, Instant>,
    min_interval: Duration,
}

impl<K: Eq + Hash + Copy> Default for KeyRepeatGate<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Copy> KeyRepeatGate<K> {
    /// Construct with the default 80ms window.
    #[must_use]
    pub fn new() -> Self {
        Self::with_interval(DEFAULT_MIN_INTERVAL)
    }

    /// Construct with a custom min-interval. Shorter = less
    /// debouncing; longer = more aggressive throttling;
    /// `Duration::ZERO` = pass-through.
    #[must_use]
    pub fn with_interval(min_interval: Duration) -> Self {
        Self {
            last_pass: HashMap::new(),
            min_interval,
        }
    }

    /// Attempt to pass an event for `key`. Returns `true` if at
    /// least `min_interval` has elapsed since the last accepted
    /// pass (or there was no prior pass). On `true`, the
    /// timestamp is updated.
    ///
    /// Returns `false` (and leaves the timestamp untouched) when
    /// the call lands within the window — the caller should drop
    /// the event.
    pub fn try_pass(&mut self, key: K) -> bool {
        self.try_pass_at(key, Instant::now())
    }

    /// Same as `try_pass` but with an explicit timestamp — for
    /// tests that don't depend on wall-clock timing.
    pub fn try_pass_at(&mut self, key: K, now: Instant) -> bool {
        match self.last_pass.get(&key) {
            Some(prev) if now.duration_since(*prev) < self.min_interval => false,
            _ => {
                self.last_pass.insert(key, now);
                true
            }
        }
    }

    /// Reset all timestamps. Use when window focus changes or the
    /// operator explicitly clears keybind state.
    pub fn clear(&mut self) {
        self.last_pass.clear();
    }

    /// Forget timestamps just for `key`. Useful when one action's
    /// debounce state should reset without affecting others.
    pub fn clear_key(&mut self, key: K) {
        self.last_pass.remove(&key);
    }

    /// Read the configured min-interval (diagnostic / introspection).
    #[must_use]
    pub fn min_interval(&self) -> Duration {
        self.min_interval
    }

    /// Update the min-interval at runtime. Existing timestamps are
    /// preserved — the new interval applies to subsequent checks.
    pub fn set_min_interval(&mut self, interval: Duration) {
        self.min_interval = interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestAction {
        FontIncrease,
        FontDecrease,
        Copy,
    }

    #[test]
    fn first_pass_always_succeeds() {
        let mut gate = KeyRepeatGate::new();
        assert!(gate.try_pass(TestAction::FontIncrease));
    }

    #[test]
    fn rapid_repeats_within_window_are_dropped() {
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0));
        assert!(!gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(10)));
        assert!(!gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(50)));
        assert!(!gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(79)));
    }

    #[test]
    fn pass_after_window_succeeds() {
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0));
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(80)));
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(160)));
    }

    #[test]
    fn different_keys_have_independent_clocks() {
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0));
        assert!(!gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(10)));
        assert!(gate.try_pass_at(TestAction::Copy, t0 + Duration::from_millis(11)));
        assert!(gate.try_pass_at(TestAction::FontDecrease, t0 + Duration::from_millis(12)));
    }

    #[test]
    fn runaway_font_storm_drops_to_one_per_window() {
        // The exact 2026-05-21 mado incident: 25 events in 1.5s.
        // With an 80ms gate ~13 events pass (one per 80ms slot).
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        let mut passes = 0;
        for i in 0..25 {
            let when = t0 + Duration::from_millis(i * 60);
            if gate.try_pass_at(TestAction::FontIncrease, when) {
                passes += 1;
            }
        }
        assert!(
            (12..=14).contains(&passes),
            "expected ~13 passes for 25 events @60ms with 80ms gate, got {passes}"
        );
    }

    #[test]
    fn clear_resets_all_clocks() {
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0));
        assert!(!gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(10)));
        gate.clear();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(11)));
    }

    #[test]
    fn clear_key_resets_only_one_clock() {
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0));
        assert!(gate.try_pass_at(TestAction::Copy, t0));
        gate.clear_key(TestAction::FontIncrease);
        // FontIncrease can re-pass immediately…
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(1)));
        // …but Copy still blocked.
        assert!(!gate.try_pass_at(TestAction::Copy, t0 + Duration::from_millis(1)));
    }

    #[test]
    fn min_interval_zero_is_a_passthrough() {
        let mut gate = KeyRepeatGate::with_interval(Duration::ZERO);
        let t0 = Instant::now();
        for i in 0..100 {
            assert!(gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_nanos(i)));
        }
    }

    #[test]
    fn set_min_interval_takes_effect_immediately() {
        let mut gate = KeyRepeatGate::with_interval(Duration::from_millis(80));
        let t0 = Instant::now();
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0));
        assert!(!gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(50)));
        // Bump the interval down — subsequent checks should pass.
        gate.set_min_interval(Duration::from_millis(20));
        assert!(gate.try_pass_at(TestAction::FontIncrease, t0 + Duration::from_millis(51)));
    }

    #[test]
    fn default_constructor_uses_default_min_interval() {
        let gate: KeyRepeatGate<TestAction> = KeyRepeatGate::default();
        assert_eq!(gate.min_interval(), DEFAULT_MIN_INTERVAL);
    }
}
