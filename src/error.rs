/// Errors from the hotkey system.
#[derive(Debug, thiserror::Error)]
pub enum KukanError {
    /// The hotkey string could not be parsed.
    #[error("invalid hotkey: {0}")]
    InvalidHotkey(String),

    /// A hotkey with this ID is already registered.
    #[error("hotkey already registered: id={0}")]
    AlreadyRegistered(u32),

    /// A platform-specific error.
    #[error("platform error: {0}")]
    Platform(String),
}
