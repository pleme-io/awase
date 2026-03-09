//! macOS-specific hotkey backend.
//!
//! Provides:
//! - Virtual keycode ↔ `Key` mapping tables
//! - `CGEventFlags` ↔ `Modifiers` conversion
//!
//! This module is always compiled (no feature gate needed) because the
//! mapping tables are pure data with no platform dependencies. The actual
//! CGEventTap manager lives in the consumer (ayatsuri) since it requires
//! objc2 and platform-specific event loop integration.

pub mod flags;
pub mod keycode;

pub use flags::{cg_flags_to_modifiers, modifiers_to_cg_flags};
pub use keycode::{key_to_keycode, keycode_to_key};
