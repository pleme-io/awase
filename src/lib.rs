//! Kukan (空間) --- global hotkey abstraction.
//!
//! Provides platform-agnostic types and traits for global hotkey
//! registration. macOS and Linux backends can be added as separate
//! feature-gated modules.
//!
//! # Quick Start
//!
//! ```rust
//! use kukan::{Hotkey, Modifiers, Key, NoopManager, HotkeyManager};
//!
//! let hk = Hotkey::parse("cmd+space").unwrap();
//! assert_eq!(hk.modifiers, Modifiers::CMD);
//! assert_eq!(hk.key, Key::Space);
//!
//! let mut manager = NoopManager::new();
//! manager.register(1, hk).unwrap();
//! manager.unregister(1).unwrap();
//! ```

mod error;
mod hotkey;
mod manager;

pub use error::KukanError;
pub use hotkey::{Hotkey, Key, Modifiers};
pub use manager::{HotkeyManager, NoopManager};
