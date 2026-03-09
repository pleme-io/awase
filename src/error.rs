/// Errors from the hotkey system.
#[derive(Debug, thiserror::Error)]
pub enum AwaseError {
    /// The hotkey string could not be parsed.
    #[error("invalid hotkey: {0}")]
    InvalidHotkey(String),

    /// A hotkey with this ID is already registered.
    #[error("hotkey already registered: id={0}")]
    AlreadyRegistered(u32),

    /// The requested mode does not exist.
    #[error("mode not found: {0}")]
    ModeNotFound(String),

    /// A duplicate binding was detected in the same mode.
    #[error("duplicate binding for {hotkey} in mode '{mode}'")]
    DuplicateBinding {
        mode: String,
        hotkey: String,
    },

    /// Accessibility or input monitoring permissions not granted.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// A platform-specific error.
    #[error("platform error: {0}")]
    Platform(String),
}
