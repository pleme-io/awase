# Awase (合わせ) — Global Hotkey Abstraction

## Build & Test

```bash
cargo build          # compile
cargo test           # unit tests + doc-tests
```

## Architecture

**Single source of truth** for all hotkey functionality across pleme-io apps.
No app should implement its own hotkey logic — use awase. Platform-agnostic
core types with feature-gated OS backends.

### Current Module Map

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Re-exports |
| `src/hotkey.rs` | `Hotkey`, `Key`, `Modifiers` — parsing and display (14 tests) |
| `src/manager.rs` | `HotkeyManager` trait + `NoopManager` stub (4 tests) |
| `src/error.rs` | `AwaseError` — invalid/duplicate/platform/permission errors |

### Target Module Map (after enrichment)

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Re-exports |
| `src/hotkey.rs` | `Hotkey`, `Key`, `Modifiers` — all keys, modifiers, parsing |
| `src/mode.rs` | `KeyMode`, `BindingMap` — mode system with passthrough control |
| `src/chord.rs` | `KeyChord`, `ChordState` — multi-step key sequences |
| `src/condition.rs` | `Condition` — per-app, per-title, per-display filters |
| `src/remap.rs` | `KeyRemap` — key-to-key remapping |
| `src/action.rs` | `Action` — command, mode switch, exec, script, chain |
| `src/binding.rs` | `Binding`, `BindingConfig` — complete binding with conditions |
| `src/conflict.rs` | `ConflictReport`, `ConflictEntry` — conflict detection |
| `src/synth.rs` | `send_key_event()`, `type_text()` — synthesized key events |
| `src/manager.rs` | `HotkeyManager` trait + `NoopManager` |
| `src/macos/mod.rs` | `CgEventTapManager` — macOS CGEventTap backend |
| `src/macos/keycode.rs` | `key_to_macos_keycode()`, `macos_keycode_to_key()` |
| `src/macos/flags.rs` | `cg_flags_to_modifiers()` |
| `src/macos/permissions.rs` | `check_permissions()` — AXIsProcessTrusted |
| `src/error.rs` | `AwaseError` |

## Key Types

### Hotkey (exists — no changes needed)

```rust
pub struct Hotkey {
    pub modifiers: Modifiers,
    pub key: Key,
}
```

Parse: `"cmd+space"`, `"ctrl+alt+shift+k"`, `"f5"`, `"escape"`.
Also support skhd format: `"cmd - h"` (spaces around `-`).

### Modifiers (exists — needs enrichment)

```rust
pub struct Modifiers(u8);  // → u16 if needed

// Existing:
Modifiers::NONE, CMD, CTRL, ALT, SHIFT

// Add:
Modifiers::FN         // macOS Fn key
Modifiers::HYPER      // Cmd+Ctrl+Alt+Shift combo (convenience alias)
Modifiers::CAPS_LOCK  // Caps Lock as modifier
```

### Key (exists — needs enrichment)

Current: A-Z, 0-9, F1-F12, Space, Return, Escape, Tab, Backspace, Delete,
Up, Down, Left, Right.

Add:

```rust
// Navigation
Key::Home, Key::End, Key::PageUp, Key::PageDown,

// Punctuation / symbols
Key::Grave,         // ` / ~
Key::Minus,         // - / _
Key::Equal,         // = / +
Key::LeftBracket,   // [ / {
Key::RightBracket,  // ] / }
Key::Backslash,     // \ / |
Key::Semicolon,     // ; / :
Key::Quote,         // ' / "
Key::Comma,         // , / <
Key::Period,        // . / >
Key::Slash,         // / / ?

// Numpad
Key::Numpad0..Key::Numpad9,
Key::NumpadAdd, Key::NumpadSubtract,
Key::NumpadMultiply, Key::NumpadDivide,
Key::NumpadDecimal, Key::NumpadEnter,

// Media / special
Key::VolumeUp, Key::VolumeDown, Key::Mute,
Key::BrightnessUp, Key::BrightnessDown,
Key::PlayPause, Key::NextTrack, Key::PreviousTrack,
Key::PrintScreen, Key::Insert, Key::Pause,
Key::CapsLock, Key::NumLock, Key::ScrollLock,

// Mouse buttons (for mouse bindings)
Key::MouseLeft, Key::MouseRight, Key::MouseMiddle,
Key::MouseButton4, Key::MouseButton5,
```

## Feature Spec

### 1. Mode System (inspired by skhd)

Named keybinding modes with independent binding sets. Mode switching is
itself a binding action.

```rust
pub struct KeyMode {
    pub name: String,
    pub bindings: HashMap<Hotkey, Binding>,
    /// When true, unmatched keys pass through to the focused app.
    /// When false, all keys are consumed (modal, like vim normal mode).
    pub passthrough: bool,
}
```

YAML:
```yaml
modes:
  default:
    passthrough: true
    bindings:
      "cmd+h": window_focus_west
      "ctrl+alt+r": { mode: resize }

  resize:
    passthrough: false
    bindings:
      "h": window_shrink_west
      "l": window_grow_east
      "escape": { mode: default }
```

Default mode is `"default"`. If no modes section exists, all bindings go
into a single default mode with passthrough=true.

### 2. Key Chords / Sequences (inspired by tmux, Emacs)

Multi-step key sequences: leader key activates chord mode, next key
selects action.

```rust
pub struct KeyChord {
    pub leader: Hotkey,
    pub follower: Hotkey,
    pub timeout_ms: u32,  // cancel chord after timeout
}

/// Internal state machine for chord tracking.
pub enum ChordState {
    Idle,
    Pending { leader: Hotkey, started: Instant, timeout_ms: u32 },
}
```

YAML:
```yaml
chords:
  "ctrl+a":
    timeout_ms: 1000
    bindings:
      "c": { exec: "open -a Terminal" }
      "n": window_focus_next
      "1": workspace_1
```

Implementation: when leader fires → enter `Pending` state → next key
within timeout matches follower bindings → dispatch action. On timeout
or non-match, optionally pass leader key through.

### 3. Actions

```rust
pub enum Action {
    /// Named command string (consumer interprets).
    Command(String),
    /// Switch to a different keybinding mode.
    ModeSwitch(String),
    /// Execute shell command.
    Exec(String),
    /// Rhai script evaluation.
    Script(String),
    /// Multiple actions in sequence.
    Chain(Vec<Action>),
}
```

### 4. Conditional Bindings (inspired by Karabiner)

```rust
pub struct Condition {
    /// Active only when focused app bundle_id matches (regex).
    pub app: Option<String>,
    /// Active only when focused app does NOT match (regex).
    pub app_exclude: Option<String>,
    /// Active only when window title matches (regex).
    pub title: Option<String>,
    /// Active only on specified display index.
    pub display: Option<u32>,
}

pub struct Binding {
    pub hotkey: Hotkey,
    pub action: Action,
    /// Whether to consume the event (not pass to app). Default: true.
    pub consume: bool,
    /// Optional conditions for when this binding is active.
    pub condition: Option<Condition>,
}
```

YAML:
```yaml
conditional_bindings:
  - hotkey: "cmd+h"
    action: window_focus_west
    consume: true
    conditions:
      app_exclude: "com.apple.Terminal|com.mitchellh.ghostty"
```

### 5. Key Remapping (inspired by Karabiner simple modifications)

```rust
pub struct KeyRemap {
    pub from: Hotkey,
    pub to: Hotkey,
    pub condition: Option<Condition>,
}
```

YAML:
```yaml
remaps:
  - from: caps_lock
    to: escape
  - from: "fn+h"
    to: left
  - from: "fn+j"
    to: down
  - from: "fn+k"
    to: up
  - from: "fn+l"
    to: right
```

Remaps are applied before binding matching — they transform the raw event.

### 6. BindingMap (mode-aware lookup)

```rust
pub struct BindingMap {
    modes: HashMap<String, KeyMode>,
    chords: Vec<(Hotkey, Vec<(Hotkey, Action, u32)>)>, // (leader, [(follower, action, timeout)])
    remaps: Vec<KeyRemap>,
    current_mode: String,
    chord_state: ChordState,
}

impl BindingMap {
    /// Match a key event against current mode bindings + chords.
    pub fn match_key(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        context: &MatchContext,
    ) -> MatchResult;
}

/// Context for condition evaluation.
pub struct MatchContext {
    pub focused_app_bundle_id: Option<String>,
    pub focused_window_title: Option<String>,
    pub display_index: u32,
}

pub enum MatchResult {
    Matched { action: Action, consume: bool },
    ChordPending { leader: Hotkey, timeout_ms: u32 },
    Remapped { to: Hotkey },
    NoMatch,
}
```

### 7. Conflict Detection

```rust
pub fn detect_conflicts(config: &BindingConfig) -> ConflictReport;

pub struct ConflictReport {
    pub conflicts: Vec<ConflictEntry>,
}

pub struct ConflictEntry {
    pub mode: String,
    pub hotkey: Hotkey,
    pub existing: String,
    pub new: String,
}
```

Detected conflicts:
- Same hotkey bound twice in the same mode
- Chord leader that conflicts with a regular binding in same mode
- (Warning) binding that shadows a known macOS system shortcut

### 8. HotkeyManager trait (enriched)

```rust
pub trait HotkeyManager: Send + Sync {
    // Existing:
    fn register(&mut self, id: u32, hotkey: Hotkey) -> Result<(), AwaseError>;
    fn unregister(&mut self, id: u32) -> Result<(), AwaseError>;

    // New:
    fn set_mode(&mut self, mode: &str) -> Result<(), AwaseError>;
    fn current_mode(&self) -> &str;
    fn load_config(&mut self, config: &BindingConfig) -> Result<ConflictReport, AwaseError>;
    fn list_bindings(&self) -> Vec<(Hotkey, String)>;
    fn register_chord(&mut self, id: u32, chord: KeyChord) -> Result<(), AwaseError>;
    fn register_remap(&mut self, remap: KeyRemap) -> Result<(), AwaseError>;
}
```

### 9. macOS Backend (feature-gated)

Behind `#[cfg(target_os = "macos")]` or `feature = "macos"`.

```rust
pub struct CgEventTapManager {
    tap_port: Option<CFRetained<CFMachPort>>,
    binding_map: BindingMap,
    callback: Box<dyn Fn(MatchResult) + Send>,
}
```

Dependencies (macos feature only):
- `objc2-core-graphics` — CGEventTap, CGEvent, CGEventFlags
- `objc2-core-foundation` — CFMachPort, CFRunLoop

Functions extracted from ayatsuri:
- `key_to_macos_keycode(Key) -> Option<u16>` — from `generate_virtual_keymap()`
- `macos_keycode_to_key(u16) -> Option<Key>` — reverse mapping
- `cg_flags_to_modifiers(CGEventFlags) -> Modifiers` — from `MODIFIER_MASKS`
- `check_permissions() -> bool` — `AXIsProcessTrusted()`

### 10. Synthesized Key Events

```rust
/// Send a single key event (key down or key up).
pub fn send_key_event(key: Key, modifiers: Modifiers, key_down: bool);

/// Type a string by synthesizing key events for each character.
pub fn type_text(text: &str);
```

## Implementation Phases

### Phase 1: Core Type Enrichment
- Add all new Key variants (media, numpad, punctuation, mouse)
- Add FN, HYPER, CAPS_LOCK modifiers
- Add skhd-format parsing ("cmd - h")
- Add `Action` enum
- Add `Binding` struct with `consume` and `Condition`
- Tests for all new types

### Phase 2: Mode System + Chords
- `KeyMode` struct
- `BindingMap` with mode-aware lookup
- `KeyChord` + `ChordState` state machine
- `ConflictReport` detection
- Enriched `HotkeyManager` trait
- Pure logic tests for mode switching, chord timeout, condition matching

### Phase 3: Remapping + Binding Config
- `KeyRemap` struct
- Remap application in `BindingMap::match_key()`
- `BindingConfig` serde struct for YAML/TOML config parsing
- Config loading into `BindingMap`

### Phase 4: macOS Backend
- Extract CGEventTap code from ayatsuri into `src/macos/`
- Virtual keycode mapping tables
- CGEventFlags → Modifiers conversion
- `CgEventTapManager` implementing `HotkeyManager`
- `check_permissions()`
- Synthesized key events (`send_key_event`, `type_text`)

### Phase 5: ayatsuri Integration
- Replace ayatsuri's `Keybinding`, `Modifiers`, `parse_modifiers` with awase types
- Replace `generate_virtual_keymap()` / `literal_keycode()` with awase functions
- Replace `find_keybind()` with `BindingMap::match_key()`
- `InputHandler` becomes thin wrapper around `CgEventTapManager`
- Add `modes:`, `chords:`, `remaps:` to ayatsuri YAML config
- Rhai API: `bind()`, `unbind()`, `set_mode()`, `get_bindings()`
- MCP tools: `list_bindings`, `set_binding`, `remove_binding`, `set_mode`

## Nix Configuration

### HM Module Options (for apps consuming awase)

```nix
bindings = mkOption {
  type = types.attrsOf (types.either types.str (types.listOf types.str));
  default = {};
  description = "Keybindings: command_name = 'modifier+key'";
};

modes = mkOption {
  type = types.attrsOf (types.submodule {
    options = {
      passthrough = mkOption { type = types.bool; default = true; };
      bindings = mkOption { type = types.attrsOf types.anything; default = {}; };
    };
  });
  default = {};
};

chords = mkOption {
  type = types.attrsOf (types.submodule {
    options = {
      timeout_ms = mkOption { type = types.int; default = 1000; };
      bindings = mkOption { type = types.attrsOf types.anything; default = {}; };
    };
  });
  default = {};
};

remaps = mkOption {
  type = types.listOf (types.submodule {
    options = {
      from = mkOption { type = types.str; };
      to = mkOption { type = types.str; };
    };
  });
  default = [];
};
```

These Nix options generate YAML that awase's `BindingConfig` deserializes.

## Consumers

- **ayatsuri** — window manager keyboard shortcuts (primary consumer)
- **tobirato** — app launcher hotkey activation
- **all future GPU apps** — any app needing global hotkeys

## Key Constraints

- **No duplication** — consumers MUST NOT implement their own hotkey logic
- **Pure logic testable** — mode matching, chord state, conflict detection
  have no platform deps (unit-testable on any OS)
- **Platform code feature-gated** — `#[cfg(target_os = "macos")]`
- **Zero unwrap policy** — all error paths use Result
- **Serde-friendly** — all config types derive Serialize + Deserialize
- **Hot-reload compatible** — `load_config()` can be called repeatedly
  as shikumi/ArcSwap detects config file changes
