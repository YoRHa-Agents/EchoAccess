//! In-memory session: master key lifecycle and UI-gated unlock.

use zeroize::Zeroize;

use crate::error::Result;
use crate::ui::adapter::UIAdapter;

/// Placeholder salt until persisted profile salt is wired (see unlock).
const PLACEHOLDER_SALT: [u8; 16] = [0u8; 16];

/// Holds the derived master key while the session is unlocked.
pub struct SessionManager {
    master_key: Option<[u8; 32]>,
    locked: bool,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            master_key: None,
            locked: true,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Unlocks using Argon2 KDF. In a full implementation, `PLACEHOLDER_SALT` is replaced
    /// by salt loaded from storage.
    pub fn unlock(&mut self, password: &str) -> Result<()> {
        let key = super::kdf::derive_key(password.as_bytes(), &PLACEHOLDER_SALT)?;
        if let Some(mut old) = self.master_key.take() {
            old.zeroize();
        }
        self.master_key = Some(key);
        self.locked = false;
        Ok(())
    }

    pub fn lock(&mut self) {
        if let Some(mut k) = self.master_key.take() {
            k.zeroize();
        }
        self.locked = true;
    }

    pub fn ensure_unlocked(&mut self, ui: &mut dyn UIAdapter) -> Result<()> {
        if self.is_locked() {
            let password = ui.prompt_password()?;
            self.unlock(&password)?;
        }
        Ok(())
    }

    pub fn get_key(&self) -> Result<&[u8; 32]> {
        self.master_key.as_ref().ok_or_else(|| {
            crate::error::EchoAccessError::Crypto("Session is locked".into())
        })
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        if let Some(mut k) = self.master_key.take() {
            k.zeroize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::adapter::{
        AppState, ConflictInfo, DiffView, MockAdapter, Notification, PendingAction,
        Resolution, UIAdapter,
    };

    #[test]
    fn lock_unlock_cycle() {
        let mut s = SessionManager::new();
        assert!(s.is_locked());
        assert!(s.get_key().is_err());

        s.unlock("secret").unwrap();
        assert!(!s.is_locked());
        assert_eq!(s.get_key().unwrap().len(), 32);

        s.lock();
        assert!(s.is_locked());
        assert!(s.get_key().is_err());
    }

    #[test]
    fn ensure_unlocked_with_mock_adapter() {
        let mut s = SessionManager::new();
        let mut ui = MockAdapter;
        s.ensure_unlocked(&mut ui).unwrap();
        assert!(!s.is_locked());
        assert!(s.get_key().is_ok());
    }

    /// [`MockAdapter`] returns an empty passphrase; this adapter supplies a real password.
    struct PasswordUi {
        password: String,
    }

    impl UIAdapter for PasswordUi {
        fn show_status(&mut self, _state: &AppState) -> Result<()> {
            Ok(())
        }

        fn show_diff(&mut self, _diff: &DiffView) -> Result<()> {
            Ok(())
        }

        fn prompt_conflict_resolution(&mut self, _conflict: &ConflictInfo) -> Result<Resolution> {
            Ok(Resolution::KeepLocal)
        }

        fn prompt_password(&mut self) -> Result<String> {
            Ok(self.password.clone())
        }

        fn confirm_action(&mut self, _action: &PendingAction) -> Result<bool> {
            Ok(true)
        }

        fn show_notification(&mut self, _msg: &Notification) -> Result<()> {
            Ok(())
        }

        fn show_progress(&mut self, _label: &str, _current: u64, _total: u64) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn ensure_unlocked_prompts_when_locked() {
        let mut s = SessionManager::new();
        let mut ui = PasswordUi {
            password: "correct horse".into(),
        };
        s.ensure_unlocked(&mut ui).unwrap();
        let k1 = *s.get_key().unwrap();

        s.lock();
        s.ensure_unlocked(&mut ui).unwrap();
        let k2 = *s.get_key().unwrap();
        assert_eq!(k1, k2, "same password and salt should yield same key");
    }
}
