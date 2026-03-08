use crate::hotkey::Hotkey;
use crate::AwaseError;

/// Trait for platform-specific hotkey registration.
///
/// Implementations should track registered hotkeys by their `id`
/// and forward key events to the application.
pub trait HotkeyManager: Send + Sync {
    /// Register a global hotkey with the given ID.
    ///
    /// Returns `Err(AwaseError::AlreadyRegistered)` if the ID is in use.
    fn register(&mut self, id: u32, hotkey: Hotkey) -> Result<(), AwaseError>;

    /// Unregister a previously registered hotkey by ID.
    fn unregister(&mut self, id: u32) -> Result<(), AwaseError>;
}

/// A no-op hotkey manager for testing and platforms without hotkey support.
///
/// All operations succeed without side effects.
#[derive(Debug, Default)]
pub struct NoopManager {
    registered: std::collections::HashSet<u32>,
}

impl NoopManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl HotkeyManager for NoopManager {
    fn register(&mut self, id: u32, _hotkey: Hotkey) -> Result<(), AwaseError> {
        if self.registered.contains(&id) {
            return Err(AwaseError::AlreadyRegistered(id));
        }
        self.registered.insert(id);
        tracing::debug!(id, "noop: registered hotkey");
        Ok(())
    }

    fn unregister(&mut self, id: u32) -> Result<(), AwaseError> {
        self.registered.remove(&id);
        tracing::debug!(id, "noop: unregistered hotkey");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkey::{Key, Modifiers};

    #[test]
    fn noop_register_and_unregister() {
        let mut manager = NoopManager::new();
        let hk = Hotkey::new(Modifiers::CMD, Key::Space);

        manager.register(1, hk).unwrap();
        manager.unregister(1).unwrap();
    }

    #[test]
    fn noop_duplicate_register_fails() {
        let mut manager = NoopManager::new();
        let hk = Hotkey::new(Modifiers::CMD, Key::Space);

        manager.register(1, hk).unwrap();
        let result = manager.register(1, hk);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AwaseError::AlreadyRegistered(1)));
    }

    #[test]
    fn noop_unregister_nonexistent_succeeds() {
        let mut manager = NoopManager::new();
        // Unregistering a non-existent ID should not error
        manager.unregister(999).unwrap();
    }

    #[test]
    fn noop_re_register_after_unregister() {
        let mut manager = NoopManager::new();
        let hk = Hotkey::new(Modifiers::CMD, Key::A);

        manager.register(1, hk).unwrap();
        manager.unregister(1).unwrap();
        // Should succeed after unregister
        manager.register(1, hk).unwrap();
    }
}
