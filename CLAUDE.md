# Awase (合わせ) -- Global Hotkey Abstraction

## Build & Test

```bash
cargo build          # compile
cargo test           # 17 unit tests + 1 doc-test
```

## Architecture

Platform-agnostic hotkey types and traits. Provides the common vocabulary for hotkey registration without coupling to any OS. macOS and Linux backends can be added as feature-gated modules.

### Module Map

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Re-exports |
| `src/hotkey.rs` | `Hotkey`, `Key`, `Modifiers` -- parsing and display (14 tests) |
| `src/manager.rs` | `HotkeyManager` trait + `NoopManager` stub (4 tests) |
| `src/error.rs` | `AwaseError` -- invalid/duplicate/platform errors |

### Key Types

- **`Hotkey`** -- `{ modifiers: Modifiers, key: Key }` with `parse()` and `display()`
- **`Modifiers`** -- bitflags: `CMD`, `CTRL`, `ALT`, `SHIFT` with `|` operator and `contains()`
- **`Key`** -- A-Z, 0-9, F1-F12, Space, Return, Escape, Tab, arrows, etc.
- **`HotkeyManager`** -- `trait { register(id, hotkey), unregister(id) }`
- **`NoopManager`** -- default stub that tracks IDs without OS interaction
- **`AwaseError`** -- `InvalidHotkey`, `AlreadyRegistered`, `Platform`

### Parse Format

Case-insensitive, `+`-separated: `"cmd+space"`, `"ctrl+alt+shift+k"`, `"f5"`.
Aliases: `command`/`super`/`meta` for CMD, `control` for CTRL, `option`/`opt` for ALT, `enter` for Return, `esc` for Escape, `del` for Delete.

### Usage Pattern

```rust
use awase::{Hotkey, Modifiers, Key, NoopManager, HotkeyManager};

let hk = Hotkey::parse("cmd+space").unwrap();
assert_eq!(hk.modifiers, Modifiers::CMD);
assert_eq!(hk.key, Key::Space);
assert_eq!(hk.display(), "cmd+space");

let mut manager = NoopManager::new();
manager.register(1, hk).unwrap();
manager.unregister(1).unwrap();
```

## Consumers

- **tobira** -- app launcher hotkey activation
- **karakuri** -- window manager keyboard shortcuts
