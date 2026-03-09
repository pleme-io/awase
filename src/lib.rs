//! Awase (合わせ) --- global hotkey abstraction.
//!
//! Provides platform-agnostic types and traits for global hotkey
//! registration, mode systems, key chords, conditional bindings, and
//! key remapping. macOS and Linux backends can be added as separate
//! feature-gated modules.
//!
//! # Quick Start
//!
//! ```rust
//! use awase::{Hotkey, Modifiers, Key, NoopManager, HotkeyManager};
//! use awase::{Action, Binding, Condition};
//!
//! // Parse hotkeys in plus-separated or skhd format
//! let hk = Hotkey::parse("cmd+space").unwrap();
//! let hk2 = Hotkey::parse("cmd - h").unwrap(); // skhd style
//!
//! // Create bindings with actions and conditions
//! let binding = Binding::new(hk, Action::command("launcher_toggle"))
//!     .with_condition(Condition {
//!         app_exclude: Some("com.apple.Terminal".to_string()),
//!         ..Default::default()
//!     });
//!
//! // Use the hotkey manager
//! let mut manager = NoopManager::new();
//! manager.register(1, hk).unwrap();
//! manager.unregister(1).unwrap();
//! ```

pub mod action;
pub mod binding;
pub mod chord;
pub mod condition;
pub mod conflict;
mod error;
mod hotkey;
mod manager;
pub mod mode;
pub mod macos;
pub mod remap;

pub use action::Action;
pub use binding::Binding;
pub use chord::{ChordState, KeyChord};
pub use condition::{Condition, MatchContext};
pub use conflict::{detect_conflicts, ConflictEntry, ConflictReport};
pub use error::AwaseError;
pub use hotkey::{Hotkey, Key, Modifiers};
pub use manager::{HotkeyManager, NoopManager};
pub use mode::{BindingMap, KeyMode, MatchResult};
pub use remap::KeyRemap;
